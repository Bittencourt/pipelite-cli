use std::io::{self, IsTerminal, Read};

use anyhow::Result;

use crate::api::models::{DealCreate, deals_table_config};
use crate::cache::KEY_DEALS;
use crate::cli::deals::DealsCreateArgs;
use crate::context::AppContext;
use crate::custom_fields;
use crate::dry_run;
use crate::error::CliError;
use crate::output;
use crate::prompt;

/// Create a new deal (or batch create from stdin).
///
/// When --stdin is set, reads a JSON array of DealCreate objects from stdin.
/// Otherwise, builds a single DealCreate from CLI flags (with interactive
/// prompts on TTY when flags are missing).
pub async fn run(ctx: &AppContext, args: &DealsCreateArgs) -> Result<()> {
    // Validate mutual exclusivity: --stdin vs individual flags
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

        return batch_create(ctx, args).await;
    }

    single_create(ctx, args).await
}

/// Create a single deal from CLI flags, with interactive prompts for missing fields.
async fn single_create(ctx: &AppContext, args: &DealsCreateArgs) -> Result<()> {
    let mut missing = Vec::new();

    // Required: title
    let title = prompt::require_text(
        &args.title,
        "title",
        "Deal title",
        &mut missing,
        ctx.no_input,
    )?;

    // Required: stage (via pipeline -> stage FuzzySelect flow)
    let stage_id = if args.stage.is_some() {
        args.stage.clone()
    } else if std::io::stdin().is_terminal() && !ctx.no_input {
        // Interactive pipeline selection, then stage selection
        select_stage_interactive(ctx).await?
    } else {
        missing.push("--stage".to_string());
        None
    };

    // Check required fields before proceeding to optional ones
    prompt::check_missing(
        &missing,
        "Usage: pipelite deals create --title <title> --stage <stage_id>",
    )?;

    // At this point title and stage_id are guaranteed Some
    let title = title.unwrap();
    let stage_id = stage_id.unwrap();

    // Optional fields
    let value = prompt::optional_number(&args.value, "Deal value", ctx.no_input)?;
    let org = prompt::optional_text(&args.org, "Organization ID", ctx.no_input)?;
    let person = prompt::optional_text(&args.person, "Person ID", ctx.no_input)?;
    let expected_close_date = prompt::optional_text(
        &args.expected_close_date,
        "Expected close date (YYYY-MM-DD)",
        ctx.no_input,
    )?;
    let notes = prompt::optional_text(&args.notes, "Notes", ctx.no_input)?;

    let custom_fields = custom_fields::resolve_custom_fields(
        ctx,
        custom_fields::CfEntityType::Deal,
        &args.custom_field,
        ctx.dry_run,
        args.custom_field_json.as_deref(),
    )
    .await?;

    let data = DealCreate {
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
        let url = format!("{}/api/v1/deals", ctx.client.base_url());
        return dry_run::render_dry_run("POST", &url, &body, &ctx.output_format, ctx.color);
    }

    let deal = ctx.client.create_deal(&data).await?;

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

/// Interactive stage selection: first pick a pipeline, then pick a stage within it.
///
/// Uses cache-through helpers for instant response when cache is warm.
async fn select_stage_interactive(ctx: &AppContext) -> Result<Option<String>> {
    // Fetch pipelines (cache-first, API fallback)
    let pipeline_options = prompt::get_pipelines_cached(ctx.cache.as_ref(), &ctx.client).await?;

    let mut missing = Vec::new();
    let pipeline_id = prompt::require_select(
        &None,
        "pipeline",
        "Select pipeline",
        &pipeline_options,
        &mut missing,
        false, // We already checked TTY above
    )?;

    let pipeline_id = match pipeline_id {
        Some(id) => id,
        None => return Ok(None),
    };

    // Fetch stages for selected pipeline (cache-first, API fallback)
    let stage_options = prompt::get_stages_cached(ctx.cache.as_ref(), &ctx.client, &pipeline_id).await?;

    let stage_id = prompt::require_select(
        &None,
        "stage",
        "Select stage",
        &stage_options,
        &mut missing,
        false,
    )?;

    Ok(stage_id)
}

/// Batch create deals from JSON array on stdin.
async fn batch_create(ctx: &AppContext, _args: &DealsCreateArgs) -> Result<()> {
    if io::stdin().is_terminal() {
        return Err(CliError::Validation {
            detail: "No data on stdin".to_string(),
            hint: "Pipe JSON data via stdin: echo '[{...}]' | pipelite deals create --stdin"
                .to_string(),
        }
        .into());
    }

    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let deals: Vec<DealCreate> = serde_json::from_str(&input).map_err(|e| CliError::Validation {
        detail: format!("Invalid JSON input: {}", e),
        hint: "Stdin must contain a JSON array of deal objects with 'title' and 'stage_id' fields."
            .to_string(),
    })?;

    // Dry-run: show each payload that would be sent
    if ctx.dry_run {
        let url = format!("{}/api/v1/deals/batch", ctx.client.base_url());
        let body = serde_json::to_value(&deals)?;
        return dry_run::render_dry_run("POST", &url, &body, &ctx.output_format, ctx.color);
    }

    let created = ctx.client.batch_create_deals(&deals).await?;

    if let Some(ref cache) = ctx.cache {
        cache.invalidate(KEY_DEALS);
    }

    let items: Vec<serde_json::Value> = created
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
        &items,
        &ctx.output_format,
        &columns,
        &None,
        ctx.color,
        None,
    )
}
