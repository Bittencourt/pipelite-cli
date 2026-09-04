use std::io::{self, IsTerminal, Read};

use anyhow::Result;

use crate::api::models::custom_fields_table_config;
use crate::cli::custom_fields::CustomFieldsUpdateArgs;
use crate::context::AppContext;
use crate::dry_run;
use crate::error::CliError;
use crate::output;
use crate::output::OutputFormat;

/// Update a custom field definition (partial PUT).
///
/// Body resolution, fully validated BEFORE any PUT:
/// - flags: a serde_json::Map is built with ONLY the explicitly provided
///   keys — omitted keys are not sent at all (the server merges only what
///   arrives). `--required`/`--no-required` and
///   `--show-in-list`/`--no-show-in-list` are tri-state via paired flags;
///   `--config` must parse as a JSON OBJECT (arrays/scalars are rejected);
///   `--position` serializes as a JSON number (f64 round-trips — position
///   is a numeric(20,10) column, PUT is the only writable path).
/// - `--stdin`: the body passes through verbatim EXCEPT that immutable
///   `entity_type`/`type` keys are stripped with ONE stderr warning
///   (quiet-suppressible) — the server would silently ignore them, and
///   honesty beats a silently-ignored rename. A GET is never issued.
///
/// With no update source at all the command refuses (exit 2) rather than
/// PUT an empty body.
pub async fn run(ctx: &AppContext, args: &CustomFieldsUpdateArgs) -> Result<()> {
    let has_flags = args.name.is_some()
        || args.config.is_some()
        || args.required
        || args.no_required
        || args.show_in_list
        || args.no_show_in_list
        || args.position.is_some();

    if args.stdin && has_flags {
        return Err(CliError::InvalidInput {
            detail: "--stdin and update flags are mutually exclusive".to_string(),
            hint: "Use either --stdin (raw JSON body, immutable keys stripped) or update flags, not both."
                .to_string(),
        }
        .into());
    }

    let body = if args.stdin {
        let mut map = read_stdin_object()?;
        let mut stripped: Vec<&str> = Vec::new();
        if map.remove("entity_type").is_some() {
            stripped.push("entity_type");
        }
        if map.remove("type").is_some() {
            stripped.push("type");
        }
        if !stripped.is_empty() && !ctx.quiet {
            let verb = if stripped.len() == 1 { "is" } else { "are" };
            let names = stripped.join(" and ");
            eprintln!("Warning: {names} {verb} immutable — removed from the update.");
        }
        if map.is_empty() {
            return Err(CliError::InvalidInput {
                detail: "nothing to update — the stdin body carried only immutable keys".to_string(),
                hint: "entity_type and type are immutable; update --name, --config, --required/--no-required, --show-in-list/--no-show-in-list or --position instead."
                    .to_string(),
            }
            .into());
        }
        serde_json::Value::Object(map)
    } else {
        if !has_flags {
            return Err(CliError::InvalidInput {
                detail: "nothing to update — no --name, --config, --required/--no-required, --show-in-list/--no-show-in-list, or --position given".to_string(),
                hint: "e.g. pipelite custom-fields update <id> --position 20000".to_string(),
            }
            .into());
        }
        let mut map = serde_json::Map::new();
        if let Some(name) = &args.name {
            map.insert("name".to_string(), serde_json::json!(name));
        }
        if let Some(raw) = &args.config {
            map.insert("config".to_string(), parse_config_flag(raw)?);
        }
        if args.required {
            map.insert("required".to_string(), serde_json::json!(true));
        }
        if args.no_required {
            map.insert("required".to_string(), serde_json::json!(false));
        }
        if args.show_in_list {
            map.insert("show_in_list".to_string(), serde_json::json!(true));
        }
        if args.no_show_in_list {
            map.insert("show_in_list".to_string(), serde_json::json!(false));
        }
        if let Some(position) = args.position {
            map.insert("position".to_string(), serde_json::json!(position));
        }
        serde_json::Value::Object(map)
    };

    // Dry-run intercept: preview the PUT, zero HTTP.
    if ctx.dry_run {
        let url = format!(
            "{}/api/v1/custom-field-definitions/{}",
            ctx.client.base_url(),
            args.definition_id
        );
        return dry_run::render_dry_run("PUT", &url, &body, &ctx.output_format, ctx.color);
    }

    let updated = ctx
        .client
        .update_custom_field_definition(&args.definition_id, &body)
        .await?;

    // Invalidate ALL per-entity definition caches — the type source for
    // typed --custom-field writing must never go stale after a mutation.
    if let Some(ref cache) = ctx.cache {
        cache.invalidate_prefix("custom_fields_");
    }

    // Render: json IS the output (render_single of the PUT response);
    // other formats render a quiet-suppressed confirmation line.
    if matches!(ctx.output_format, OutputFormat::Json) {
        let config = custom_fields_table_config();
        let columns: Vec<String> = config
            .default_columns
            .iter()
            .map(|s| s.to_string())
            .collect();
        let value = serde_json::to_value(&updated)?;
        output::render_single(&value, &ctx.output_format, &columns, &None, ctx.color)?;
    } else if !ctx.quiet {
        println!("Updated custom field definition {}", args.definition_id);
    }

    Ok(())
}

/// Read a raw JSON object from stdin (the full-control path).
fn read_stdin_object() -> Result<serde_json::Map<String, serde_json::Value>> {
    if io::stdin().is_terminal() {
        return Err(CliError::Validation {
            detail: "No data on stdin".to_string(),
            hint: "Pipe JSON data: echo '{\"name\":\"price\"}' | pipelite custom-fields update <id> --stdin"
                .to_string(),
        }
        .into());
    }

    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let value: serde_json::Value = serde_json::from_str(&input).map_err(|e| CliError::Validation {
        detail: format!("Invalid JSON input: {e}"),
        hint: "Stdin must contain a JSON object with the definition keys to update.".to_string(),
    })?;

    match value {
        serde_json::Value::Object(map) => Ok(map),
        other => Err(CliError::InvalidInput {
            detail: "The stdin body must be a JSON object".to_string(),
            hint: format!("Got {other}; e.g. {{\"name\":\"price\"}}"),
        }
        .into()),
    }
}

/// Parse --config (raw JSON string) into a serde_json::Value.
///
/// Must be a JSON OBJECT — arrays and scalars are rejected (exit 2,
/// pre-HTTP): a config that is not an object would corrupt the definition's
/// option vocabulary.
fn parse_config_flag(raw: &str) -> Result<serde_json::Value> {
    let value: serde_json::Value = serde_json::from_str(raw).map_err(|e| CliError::InvalidInput {
        detail: format!("Invalid JSON for --config: {e}"),
        hint: "--config must be a valid JSON object — e.g. '{\"options\":[\"a\",\"b\"]}'".to_string(),
    })?;
    match value {
        v @ serde_json::Value::Object(_) => Ok(v),
        other => Err(CliError::InvalidInput {
            detail: format!("--config must be a JSON object (got {other})"),
            hint: "e.g. --config '{\"options\":[\"a\",\"b\"]}'".to_string(),
        }
        .into()),
    }
}
