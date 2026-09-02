use std::io::{self, IsTerminal, Read};

use anyhow::Result;

use crate::api::models::{DealUpdate, deals_table_config};
use crate::batch;
use crate::cache::KEY_DEALS;
use crate::cli::deals::DealsUpdateArgs;
use crate::context::AppContext;
use crate::dry_run;
use crate::error::CliError;
use crate::output;
use crate::prompt;

/// Update an existing deal.
///
/// With --stdin, reads a JSON array of objects (each with an "id" field plus
/// update fields) from stdin and batch-updates with continue-on-error.
/// If no flags are provided on a TTY, prompts interactively for all fields
/// (all optional -- user can skip any). In headless mode with no flags,
/// returns a validation error.
pub async fn run(ctx: &AppContext, args: &DealsUpdateArgs) -> Result<()> {
    if args.stdin {
        let has_flags = args.title.is_some()
            || args.stage.is_some()
            || args.value.is_some()
            || args.org.is_some()
            || args.person.is_some()
            || args.expected_close_date.is_some()
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
        detail: "Missing deal ID".to_string(),
        hint: "Provide a deal ID or use --stdin.".to_string(),
    })?;

    let has_flags = args.title.is_some()
        || args.stage.is_some()
        || args.value.is_some()
        || args.org.is_some()
        || args.person.is_some()
        || args.expected_close_date.is_some()
        || args.notes.is_some()
        || !args.custom_field.is_empty();

    // If no flags provided in headless mode, error out
    if !has_flags && (ctx.no_input || !std::io::stdin().is_terminal()) {
        return Err(CliError::Validation {
            detail: "No fields to update. Provide at least one flag.".to_string(),
            hint: "Usage: pipelite deals update <id> --title <title> [--stage <stage_id>] [--value <value>]".to_string(),
        }
        .into());
    }

    // Collect field values (from flags or interactive prompts)
    let title = if has_flags {
        args.title.clone()
    } else {
        prompt::optional_text(&args.title, "Title", ctx.no_input)?
    };

    let stage_id = if has_flags {
        args.stage.clone()
    } else {
        prompt::optional_text(&args.stage, "Stage ID", ctx.no_input)?
    };

    let value = if has_flags {
        args.value
    } else {
        prompt::optional_number(&args.value, "Value", ctx.no_input)?
    };

    let org = if has_flags {
        args.org.clone()
    } else {
        prompt::optional_text(&args.org, "Organization ID", ctx.no_input)?
    };

    let person = if has_flags {
        args.person.clone()
    } else {
        prompt::optional_text(&args.person, "Person ID", ctx.no_input)?
    };

    let expected_close_date = if has_flags {
        args.expected_close_date.clone()
    } else {
        prompt::optional_text(&args.expected_close_date, "Expected close date (YYYY-MM-DD)", ctx.no_input)?
    };

    let notes = if has_flags {
        args.notes.clone()
    } else {
        prompt::optional_text(&args.notes, "Notes", ctx.no_input)?
    };

    let custom_fields = parse_custom_fields(&args.custom_field)?;

    let data = DealUpdate {
        title,
        stage_id,
        value,
        organization_id: org,
        person_id: person,
        expected_close_date,
        notes,
        custom_fields,
    };

    // Dry-run intercept
    if ctx.dry_run {
        let body = serde_json::to_value(&data)?;
        let url = format!("{}/api/v1/deals/{}", ctx.client.base_url(), id);
        return dry_run::render_dry_run("PUT", &url, &body, &ctx.output_format, ctx.color);
    }

    let deal = ctx.client.update_deal(id, &data).await?;

    if let Some(ref cache) = ctx.cache {
        cache.invalidate(KEY_DEALS);
    }

    let item = serde_json::to_value(&deal)?;

    let config = deals_table_config();
    let columns: Vec<String> = config
        .default_columns
        .iter()
        .map(|s| s.to_string())
        .collect();

    output::render_single(&item, &ctx.output_format, &columns, &None, ctx.color)
}

/// Batch update deals from JSON array on stdin (per D-02, D-03).
///
/// Each JSON object must contain an "id" field plus update fields.
/// Processes all items with continue-on-error semantics (per D-06).
async fn batch_update(ctx: &AppContext) -> Result<()> {
    if io::stdin().is_terminal() {
        return Err(CliError::Validation {
            detail: "No data on stdin".to_string(),
            hint: "Pipe JSON data: echo '[{\"id\":\"deal_1\",\"title\":\"New\"}]' | pipelite deals update --stdin".to_string(),
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
            let url = format!("{}/api/v1/deals/{}", ctx.client.base_url(), id);
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

        let data: DealUpdate = match serde_json::from_value(item) {
            Ok(d) => d,
            Err(e) => {
                outcome.record_failure(i, &id, &e);
                continue;
            }
        };

        match ctx.client.update_deal(&id, &data).await {
            Ok(deal) => {
                outcome.record_success();
                succeeded.push(deal);
            }
            Err(e) => {
                outcome.record_failure(i, &id, &e);
            }
        }
    }

    // Render successes to stdout (per D-07)
    if !succeeded.is_empty() {
        if let Some(ref cache) = ctx.cache {
            cache.invalidate(KEY_DEALS);
        }
        let items_json: Vec<serde_json::Value> = succeeded
            .iter()
            .map(|d| serde_json::to_value(d).map_err(Into::into))
            .collect::<Result<Vec<_>>>()?;

        let config = deals_table_config();
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

    outcome.finalize("deal", "update")
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
