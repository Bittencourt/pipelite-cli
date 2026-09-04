use std::io::IsTerminal;

use anyhow::Result;

use crate::api::models::{DealUpdate, deals_table_config};
use crate::batch;
use crate::cache::KEY_DEALS;
use crate::cli::deals::DealsUpdateArgs;
use crate::context::AppContext;
use crate::custom_fields;
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
            || !args.custom_field.is_empty()
            || args.custom_field_json.is_some();

        if has_flags {
            return Err(CliError::InvalidInput {
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
        || !args.custom_field.is_empty()
        || args.custom_field_json.is_some();

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

    let custom_fields = custom_fields::resolve_custom_fields(
        ctx,
        custom_fields::CfEntityType::Deal,
        &args.custom_field,
        ctx.dry_run,
        args.custom_field_json.as_deref(),
    )
    .await?;

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
/// The shared flow lives in [`crate::batch::run_batch_update`].
async fn batch_update(ctx: &AppContext) -> Result<()> {
    batch::run_batch_update::<DealUpdate, _>(
        ctx,
        "deal",
        "deals",
        r#"[{"id":"deal_1","title":"New"}]"#,
        "deals",
        KEY_DEALS,
        None,
        &deals_table_config().default_columns,
        async |id: String, data: DealUpdate, _raw: &serde_json::Value| {
            batch::ensure_update_fields(&data, "deals")?;
            let deal = ctx.client.update_deal(&id, &data).await?;
            Ok(serde_json::to_value(deal)?)
        },
    )
    .await
}
