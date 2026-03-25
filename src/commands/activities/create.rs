use std::io::{self, IsTerminal, Read};

use anyhow::Result;

use crate::api::models::{ActivityCreate, activities_table_config};
use crate::cli::activities::ActivitiesCreateArgs;
use crate::context::AppContext;
use crate::error::CliError;
use crate::output;

/// Create a new activity (or batch create from stdin).
///
/// When --stdin is set, reads a JSON array of ActivityCreate objects from stdin
/// and creates them one-by-one (no batch endpoint for activities).
/// Otherwise, builds a single ActivityCreate from CLI flags.
pub async fn run(ctx: &AppContext, args: &ActivitiesCreateArgs) -> Result<()> {
    if args.stdin {
        batch_create(ctx).await
    } else {
        single_create(ctx, args).await
    }
}

/// Create a single activity from CLI flags.
async fn single_create(ctx: &AppContext, args: &ActivitiesCreateArgs) -> Result<()> {
    let title = args.title.as_ref().ok_or_else(|| CliError::Validation {
        detail: "Missing required flag: --title".to_string(),
        hint: "Usage: pipelite activities create --title <title> --type <type_id>".to_string(),
    })?;

    let type_id = args.type_id.as_ref().ok_or_else(|| CliError::Validation {
        detail: "Missing required flag: --type".to_string(),
        hint: "Usage: pipelite activities create --title <title> --type <type_id>".to_string(),
    })?;

    let custom_fields = parse_custom_fields(&args.custom_field)?;

    let data = ActivityCreate {
        title: title.clone(),
        type_id: type_id.clone(),
        deal_id: args.deal.clone(),
        owner_id: None,
        due_at: args.due_at.clone(),
        notes: args.notes.clone(),
        custom_fields,
    };

    let activity = ctx.client.create_activity(&data).await?;
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
