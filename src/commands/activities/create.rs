use std::io::{self, IsTerminal, Read};

use anyhow::Result;

use crate::api::models::{ActivityCreate, activities_table_config};
use crate::cache::KEY_ACTIVITIES;
use crate::cli::activities::ActivitiesCreateArgs;
use crate::context::AppContext;
use crate::custom_fields;
use crate::dry_run;
use crate::error::CliError;
use crate::output;
use crate::prompt;

/// Create a new activity (or batch create from stdin).
///
/// When --stdin is set, reads a JSON array of ActivityCreate objects from stdin
/// and creates them one-by-one (no batch endpoint for activities).
/// Otherwise, builds a single ActivityCreate from CLI flags (with interactive
/// prompts on TTY when flags are missing).
pub async fn run(ctx: &AppContext, args: &ActivitiesCreateArgs) -> Result<()> {
    // Validate mutual exclusivity: --stdin vs individual flags
    if args.stdin {
        let has_flags = args.title.is_some()
            || args.type_id.is_some()
            || args.deal.is_some()
            || args.due_at.is_some()
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

        return batch_create(ctx).await;
    }

    single_create(ctx, args).await
}

/// Create a single activity from CLI flags, with interactive prompts for missing fields.
async fn single_create(ctx: &AppContext, args: &ActivitiesCreateArgs) -> Result<()> {
    let mut missing = Vec::new();

    // Required: title
    let title = prompt::require_text(
        &args.title,
        "title",
        "Activity title",
        &mut missing,
        ctx.no_input,
    )?;

    // Required: type_id (text input, no FuzzySelect since activity types are not listable)
    let type_id = prompt::require_text(
        &args.type_id,
        "type",
        "Activity type ID",
        &mut missing,
        ctx.no_input,
    )?;

    // Check required fields before proceeding to optional ones
    prompt::check_missing(
        &missing,
        "Usage: pipelite activities create --title <title> --type <type_id>",
    )?;

    let title = title.unwrap();
    let type_id = type_id.unwrap();

    // Optional fields
    let deal = prompt::optional_text(&args.deal, "Deal ID", ctx.no_input)?;
    let due_at = prompt::optional_text(&args.due_at, "Due date/time (ISO format)", ctx.no_input)?;
    let notes = prompt::optional_text(&args.notes, "Notes", ctx.no_input)?;

    let custom_fields = custom_fields::resolve_custom_fields(
        ctx,
        custom_fields::CfEntityType::Activity,
        &args.custom_field,
        ctx.dry_run,
        args.custom_field_json.as_deref(),
    )
    .await?;

    let data = ActivityCreate {
        title,
        type_id,
        deal_id: deal,
        owner_id: None,
        due_at,
        notes,
        custom_fields,
    };

    // Dry-run intercept
    if ctx.dry_run {
        let body = serde_json::to_value(&data)?;
        let url = format!("{}/api/v1/activities", ctx.client.base_url());
        return dry_run::render_dry_run("POST", &url, &body, &ctx.output_format, ctx.color);
    }

    let activity = ctx.client.create_activity(&data).await?;

    if let Some(ref cache) = ctx.cache {
        cache.invalidate(KEY_ACTIVITIES);
    }

    let item = serde_json::to_value(&activity)?;

    let config = activities_table_config();
    let columns: Vec<String> = config
        .default_columns
        .iter()
        .map(|s| s.to_string())
        .collect();

    output::render_single(&item, &ctx.output_format, &columns, &None, ctx.color)
}

/// Batch create activities from JSON array on stdin using individual-create loop.
///
/// Activities have no batch API endpoint, so each item is created individually
/// with progress output and partial failure handling.
async fn batch_create(ctx: &AppContext) -> Result<()> {
    if io::stdin().is_terminal() {
        return Err(CliError::Validation {
            detail: "No data on stdin".to_string(),
            hint: "Pipe JSON data via stdin: echo '[{...}]' | pipelite activities create --stdin"
                .to_string(),
        }
        .into());
    }

    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let items: Vec<ActivityCreate> =
        serde_json::from_str(&input).map_err(|e| CliError::Validation {
            detail: format!("Invalid JSON input: {}", e),
            hint: "Stdin must contain a JSON array of activity objects with 'title' and 'type_id' fields."
                .to_string(),
        })?;

    // Dry-run: show each payload that would be sent
    if ctx.dry_run {
        let url = format!("{}/api/v1/activities", ctx.client.base_url());
        for item in &items {
            let body = serde_json::to_value(item)?;
            dry_run::render_dry_run("POST", &url, &body, &ctx.output_format, ctx.color)?;
        }
        return Ok(());
    }

    let total = items.len();
    let mut created_items: Vec<serde_json::Value> = Vec::new();
    let mut failed: u32 = 0;

    for (i, item) in items.iter().enumerate() {
        if !ctx.quiet {
            eprint!("\rCreating {}/{}...", i + 1, total);
        }
        match ctx.client.create_activity(item).await {
            Ok(entity) => {
                if let Ok(val) = serde_json::to_value(&entity) {
                    created_items.push(val);
                }
            }
            Err(e) => {
                eprintln!("\nFailed item {}: {}", i + 1, e);
                failed += 1;
            }
        }
    }
    if !ctx.quiet && total > 0 {
        eprintln!(); // clear progress line
    }

    if !created_items.is_empty() {
        if let Some(ref cache) = ctx.cache {
            cache.invalidate(KEY_ACTIVITIES);
        }
    }

    let config = activities_table_config();
    let columns: Vec<String> = config
        .default_columns
        .iter()
        .map(|s| s.to_string())
        .collect();

    output::render_list(
        &created_items,
        &ctx.output_format,
        &columns,
        &None,
        ctx.color,
        None,
    )?;

    if failed > 0 {
        eprintln!(
            "Created {}/{}. {} failed (see errors above).",
            created_items.len(),
            total,
            failed
        );
        return Err(CliError::Api {
            status: 0,
            detail: format!("{} of {} items failed", failed, total),
            hint: "Review errors above and retry failed items.".to_string(),
        }
        .into());
    }
    Ok(())
}
