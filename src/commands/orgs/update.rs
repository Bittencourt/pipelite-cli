use std::io::{self, IsTerminal, Read};

use anyhow::Result;

use crate::api::models::{OrganizationUpdate, orgs_table_config};
use crate::batch;
use crate::cache::KEY_ORGS;
use crate::cli::orgs::OrgsUpdateArgs;
use crate::context::AppContext;
use crate::dry_run;
use crate::error::CliError;
use crate::output;
use crate::prompt;

/// Update an existing organization.
///
/// With --stdin, reads a JSON array of objects (each with an "id" field plus
/// update fields) from stdin and batch-updates with continue-on-error.
/// If no flags are provided on a TTY, prompts interactively for all fields
/// (all optional -- user can skip any). In headless mode with no flags,
/// returns a validation error.
pub async fn run(ctx: &AppContext, args: &OrgsUpdateArgs) -> Result<()> {
    if args.stdin {
        let has_flags = args.name.is_some()
            || args.website.is_some()
            || args.industry.is_some()
            || args.notes.is_some()
            || !args.custom_field.is_empty();

        if has_flags {
            return Err(CliError::Validation {
                detail: "--stdin and individual field flags are mutually exclusive".to_string(),
                hint: "Use either --stdin or individual flags, not both.".to_string(),
            }
            .into());
        }

        return batch_update(ctx).await;
    }

    // Extract id from Option — CLAUDE.md forbids unwrap() in production code.
    // Use CliError::Validation with an actionable hint instead.
    let id = args.id.as_deref().ok_or_else(|| CliError::Validation {
        detail: "Missing organization ID".to_string(),
        hint: "Provide an organization ID or use --stdin.".to_string(),
    })?;

    let has_flags = args.name.is_some()
        || args.website.is_some()
        || args.industry.is_some()
        || args.notes.is_some()
        || !args.custom_field.is_empty();

    // If no flags provided in headless mode, error out
    if !has_flags && (ctx.no_input || !std::io::stdin().is_terminal()) {
        return Err(CliError::Validation {
            detail: "No fields to update. Provide at least one flag.".to_string(),
            hint: "Usage: pipelite orgs update <id> --name <name> [--website <url>] [--industry <industry>]".to_string(),
        }
        .into());
    }

    // Collect field values (from flags or interactive prompts)
    let name = if has_flags {
        args.name.clone()
    } else {
        prompt::optional_text(&args.name, "Name", ctx.no_input)?
    };

    let website = if has_flags {
        args.website.clone()
    } else {
        prompt::optional_text(&args.website, "Website URL", ctx.no_input)?
    };

    let industry = if has_flags {
        args.industry.clone()
    } else {
        prompt::optional_text(&args.industry, "Industry", ctx.no_input)?
    };

    let notes = if has_flags {
        args.notes.clone()
    } else {
        prompt::optional_text(&args.notes, "Notes", ctx.no_input)?
    };

    let custom_fields = parse_custom_fields(&args.custom_field)?;

    let data = OrganizationUpdate {
        name,
        website,
        industry,
        notes,
        custom_fields,
    };

    // Dry-run intercept
    if ctx.dry_run {
        let body = serde_json::to_value(&data)?;
        let url = format!("{}/api/v1/organizations/{}", ctx.client.base_url(), id);
        return dry_run::render_dry_run("PUT", &url, &body, &ctx.output_format, ctx.color);
    }

    let org = ctx.client.update_org(id, &data).await?;

    if let Some(ref cache) = ctx.cache {
        cache.invalidate(KEY_ORGS);
    }

    let item = serde_json::to_value(&org)?;

    let config = orgs_table_config();
    let columns: Vec<String> = config
        .default_columns
        .iter()
        .map(|s| s.to_string())
        .collect();

    output::render_single(&item, &ctx.output_format, &columns, &None, ctx.color)
}

/// Batch update organizations from JSON array on stdin (per D-02, D-03).
///
/// Each JSON object must contain an "id" field plus update fields.
/// Processes all items with continue-on-error semantics (per D-06).
async fn batch_update(ctx: &AppContext) -> Result<()> {
    if io::stdin().is_terminal() {
        return Err(CliError::Validation {
            detail: "No data on stdin".to_string(),
            hint: "Pipe JSON data: echo '[{\"id\":\"org_1\",\"name\":\"New\"}]' | pipelite orgs update --stdin".to_string(),
        }
        .into());
    }

    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let items: Vec<serde_json::Value> =
        serde_json::from_str(&input).map_err(|e| CliError::Validation {
            detail: format!("Invalid JSON input: {}", e),
            hint: "Stdin must contain a JSON array of objects with 'id' field plus update fields."
                .to_string(),
        })?;

    if ctx.dry_run {
        for item in &items {
            let id = item.get("id").and_then(|v| v.as_str()).unwrap_or("unknown");
            let url = format!("{}/api/v1/organizations/{}", ctx.client.base_url(), id);
            dry_run::render_dry_run("PUT", &url, item, &ctx.output_format, ctx.color)?;
        }
        return Ok(());
    }

    let mut outcome = batch::BatchOutcome::new(items.len());
    let mut succeeded = Vec::new();

    for (i, item) in items.into_iter().enumerate() {
        let id = match item.get("id").and_then(|v| v.as_str()) {
            Some(id) => id.to_string(),
            None => {
                outcome.record_failure(i, "unknown", &"missing 'id' field");
                continue;
            }
        };

        let data: OrganizationUpdate = match serde_json::from_value(item) {
            Ok(d) => d,
            Err(e) => {
                outcome.record_failure(i, &id, &e);
                continue;
            }
        };

        match ctx.client.update_org(&id, &data).await {
            Ok(org) => {
                outcome.record_success();
                succeeded.push(org);
            }
            Err(e) => {
                outcome.record_failure(i, &id, &e);
            }
        }
    }

    // Render successes to stdout (per D-07)
    if !succeeded.is_empty() {
        if let Some(ref cache) = ctx.cache {
            cache.invalidate(KEY_ORGS);
        }
        let items_json: Vec<serde_json::Value> = succeeded
            .iter()
            .map(|d| serde_json::to_value(d).map_err(Into::into))
            .collect::<Result<Vec<_>>>()?;

        let config = orgs_table_config();
        let columns: Vec<String> = config
            .default_columns
            .iter()
            .map(|s| s.to_string())
            .collect();
        output::render_list(
            &items_json,
            &ctx.output_format,
            &columns,
            &None,
            ctx.color,
            None,
        )?;
    }

    outcome.finalize("organization", "update")
}

/// Parse --custom-field key=value pairs into a serde_json::Value object.
fn parse_custom_fields(pairs: &[String]) -> Result<Option<serde_json::Value>> {
    if pairs.is_empty() {
        return Ok(None);
    }

    let mut map = serde_json::Map::new();
    for pair in pairs {
        let (key, value) = pair.split_once('=').ok_or_else(|| CliError::Validation {
            detail: format!("Invalid custom field format: '{}'", pair),
            hint: "Use key=value format: --custom-field industry=Tech".to_string(),
        })?;
        map.insert(key.to_string(), serde_json::Value::String(value.to_string()));
    }

    Ok(Some(serde_json::Value::Object(map)))
}
