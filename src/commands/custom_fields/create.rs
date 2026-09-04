use std::io::{self, IsTerminal, Read};

use anyhow::Result;

use crate::api::models::{custom_fields_table_config, CustomFieldDefinitionCreate};
use crate::cli::custom_fields::CustomFieldsCreateArgs;
use crate::commands::custom_fields;
use crate::context::AppContext;
use crate::dry_run;
use crate::error::CliError;
use crate::output;
use crate::output::OutputFormat;
use crate::prompt;

/// Create a custom field definition.
///
/// The body resolves from EXACTLY ONE source, fully validated BEFORE any
/// POST (the server validates nothing on the v1 path — these checks are the
/// only line of defense):
/// - flags: `--entity-type`/`--type` are allow-listed pre-HTTP (exit 2),
///   `--key` maps to the wire `name` (blob keys are definition NAMES),
///   `--options` is REQUIRED for select/single_select/multi_select and
///   rejected for every other type, and the body always carries
///   required/show_in_list (booleans defaulting false);
/// - `--stdin`: raw JSON body passed through VERBATIM (no defaults
///   injected); an `entity_type`/`type` string present in the body must
///   still be a valid server token — full control does not include
///   silently-poisoned vocabularies.
pub async fn run(ctx: &AppContext, args: &CustomFieldsCreateArgs) -> Result<()> {
    if args.stdin
        && (args.entity_type.is_some()
            || args.key.is_some()
            || args.field_type.is_some()
            || args.options.is_some()
            || args.required
            || args.show_in_list)
    {
        return Err(CliError::InvalidInput {
            detail: "--stdin and create flags are mutually exclusive".to_string(),
            hint: "Use either --stdin (raw JSON body, verbatim) or the create flags, not both."
                .to_string(),
        }
        .into());
    }

    // The wire body: verbatim stdin JSON, or the typed flags payload.
    let (body, typed): (serde_json::Value, Option<CustomFieldDefinitionCreate>) = if args.stdin {
        (read_stdin_body()?, None)
    } else {
        let mut missing = Vec::new();
        if args.entity_type.is_none() {
            missing.push("--entity-type".to_string());
        }
        if args.key.is_none() {
            missing.push("--key".to_string());
        }
        if args.field_type.is_none() {
            missing.push("--type".to_string());
        }
        prompt::check_missing(
            &missing,
            "Usage: pipelite custom-fields create --entity-type <type> --key <name> --type <t>",
        )?;

        let entity_type = custom_fields::normalize_entity_type(
            args.entity_type.as_deref().unwrap_or_default(),
        )?;
        let type_ = custom_fields::normalize_definition_type(
            args.field_type.as_deref().unwrap_or_default(),
        )?;

        let options = args.options.clone().unwrap_or_default();
        let is_select_family = type_ == "single_select" || type_ == "multi_select";
        if is_select_family && options.is_empty() {
            return Err(CliError::InvalidInput {
                detail: format!("--type {type_} requires --options"),
                hint: "Pass --options a,b,c — becomes config.options (the dropdown entries)."
                    .to_string(),
            }
            .into());
        }
        if !is_select_family && !options.is_empty() {
            return Err(CliError::InvalidInput {
                detail: format!(
                    "--options only applies to select/single_select/multi_select (got --type {type_})"
                ),
                hint: "Remove --options, or pick --type single_select or multi_select for a dropdown field."
                    .to_string(),
            }
            .into());
        }

        // THE MAPPING: --key is the wire "name" — blob keys are definition
        // names, so the definition name IS the --custom-field key.
        let data = CustomFieldDefinitionCreate {
            name: args.key.clone().unwrap_or_default(),
            entity_type: entity_type.to_string(),
            type_: type_.to_string(),
            required: args.required,
            show_in_list: args.show_in_list,
            config: if options.is_empty() {
                None
            } else {
                Some(custom_fields::build_options_config(&options))
            },
        };
        (serde_json::to_value(&data)?, Some(data))
    };

    // Dry-run intercept: preview the POST, zero HTTP.
    if ctx.dry_run {
        let url = format!("{}/api/v1/custom-field-definitions", ctx.client.base_url());
        return dry_run::render_dry_run("POST", &url, &body, &ctx.output_format, ctx.color);
    }

    let created = if let Some(data) = &typed {
        ctx.client.create_custom_field_definition(data).await?
    } else {
        ctx.client.create_custom_field_definition_raw(&body).await?
    };

    // Render: json IS the output (render_single); other formats render a
    // quiet-suppressed confirmation line.
    if matches!(ctx.output_format, OutputFormat::Json) {
        let config = custom_fields_table_config();
        let columns: Vec<String> = config
            .default_columns
            .iter()
            .map(|s| s.to_string())
            .collect();
        let item = serde_json::to_value(&created)?;
        output::render_single(&item, &ctx.output_format, &columns, &None, ctx.color)?;
    } else if !ctx.quiet {
        println!(
            "Created custom field definition {} ({})",
            created.name, created.id
        );
    }

    Ok(())
}

/// Read and validate a raw JSON body from stdin (the full-control path).
fn read_stdin_body() -> Result<serde_json::Value> {
    if io::stdin().is_terminal() {
        return Err(CliError::Validation {
            detail: "No data on stdin".to_string(),
            hint: "Pipe JSON data: echo '{\"name\":\"price\",\"entity_type\":\"deal\",\"type\":\"number\"}' | pipelite custom-fields create --stdin"
                .to_string(),
        }
        .into());
    }

    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let body: serde_json::Value = serde_json::from_str(&input).map_err(|e| CliError::Validation {
        detail: format!("Invalid JSON input: {e}"),
        hint: "Stdin must contain a JSON definition object with 'name', 'entity_type' and 'type'."
            .to_string(),
    })?;

    validate_stdin_definition(&body)?;
    Ok(body)
}

/// Validate the raw `--stdin` body's vocabulary when the keys are present.
///
/// Permissive otherwise: unknown keys are left for the server to strip —
/// full control passes through verbatim, but the entity_type/type
/// allow-lists still fire (exit 2, zero HTTP) so a mistyped vocabulary
/// cannot be injected even over the full-control path.
fn validate_stdin_definition(body: &serde_json::Value) -> Result<()> {
    if let Some(t) = body.get("entity_type").and_then(|v| v.as_str()) {
        custom_fields::normalize_entity_type(t)?;
    }
    if let Some(t) = body.get("type").and_then(|v| v.as_str()) {
        custom_fields::normalize_definition_type(t)?;
    }
    Ok(())
}
