use std::io::IsTerminal;

use anyhow::Result;

use crate::api::models::{DealUpdate, deals_table_config};
use crate::cli::deals::DealsUpdateArgs;
use crate::context::AppContext;
use crate::dry_run;
use crate::error::CliError;
use crate::output;
use crate::prompt;

/// Update an existing deal.
///
/// If no flags are provided on a TTY, prompts interactively for all fields
/// (all optional -- user can skip any). In headless mode with no flags,
/// returns a validation error.
pub async fn run(ctx: &AppContext, args: &DealsUpdateArgs) -> Result<()> {
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
        let url = format!("{}/api/v1/deals/{}", ctx.client.base_url(), args.id);
        return dry_run::render_dry_run("PUT", &url, &body, &ctx.output_format, ctx.color);
    }

    let deal = ctx.client.update_deal(&args.id, &data).await?;
    let item = serde_json::to_value(&deal)?;

    let config = deals_table_config();
    let columns: Vec<String> = config
        .default_columns
        .iter()
        .map(|s| s.to_string())
        .collect();

    output::render_single(&item, &ctx.output_format, &columns, &None, ctx.color)
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
