use std::io::{self, IsTerminal, Read};

use anyhow::Result;

use crate::api::models::{StageUpdate, stages_table_config};
use crate::batch;
use crate::cache::KEY_STAGES;
use crate::cli::stages::StagesUpdateArgs;
use crate::context::AppContext;
use crate::dry_run;
use crate::error::CliError;
use crate::output;
use crate::prompt;

/// Update an existing stage.
///
/// With --stdin, reads a JSON array of objects (each with an "id" field plus
/// update fields) from stdin and batch-updates with continue-on-error.
/// If no flags are provided on a TTY, prompts interactively for all fields
/// (all optional -- user can skip any). No --pipeline needed -- stage ID is unique.
/// In headless mode with no flags, returns a validation error.
pub async fn run(ctx: &AppContext, args: &StagesUpdateArgs) -> Result<()> {
    if args.stdin {
        let has_flags = args.name.is_some()
            || args.description.is_some()
            || args.color.is_some()
            || args.stage_type.is_some();

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
        detail: "Missing stage ID".to_string(),
        hint: "Provide a stage ID or use --stdin.".to_string(),
    })?;

    let has_flags = args.name.is_some()
        || args.description.is_some()
        || args.color.is_some()
        || args.stage_type.is_some();

    // If no flags provided in headless mode, error out
    if !has_flags && (ctx.no_input || !std::io::stdin().is_terminal()) {
        return Err(CliError::Validation {
            detail: "No fields to update. Provide at least one flag.".to_string(),
            hint: "Usage: pipelite stages update <id> --name <name> [--type <type>] [--color <color>]".to_string(),
        }
        .into());
    }

    // Collect field values (from flags or interactive prompts)
    let name = if has_flags {
        args.name.clone()
    } else {
        prompt::optional_text(&args.name, "Stage name", ctx.no_input)?
    };

    let stage_type = if has_flags {
        args.stage_type.clone()
    } else {
        prompt::optional_text(&args.stage_type, "Stage type (open, won, or lost)", ctx.no_input)?
    };

    let color = if has_flags {
        args.color.clone()
    } else {
        prompt::optional_text(&args.color, "Color (hex)", ctx.no_input)?
    };

    let description = if has_flags {
        args.description.clone()
    } else {
        prompt::optional_text(&args.description, "Description", ctx.no_input)?
    };

    let data = StageUpdate {
        name,
        description,
        color,
        stage_type,
    };

    // Dry-run intercept
    if ctx.dry_run {
        let body = serde_json::to_value(&data)?;
        let url = format!("{}/api/v1/stages/{}", ctx.client.base_url(), id);
        return dry_run::render_dry_run("PUT", &url, &body, &ctx.output_format, ctx.color);
    }

    let stage = ctx.client.update_stage(id, &data).await?;

    if let Some(ref cache) = ctx.cache {
        cache.invalidate(KEY_STAGES);
        cache.invalidate(&format!("stages_{}", stage.pipeline_id));
    }

    let item = serde_json::to_value(&stage)?;

    let config = stages_table_config();
    let columns: Vec<String> = config
        .default_columns
        .iter()
        .map(|s| s.to_string())
        .collect();

    output::render_single(&item, &ctx.output_format, &columns, &None, ctx.color)
}

/// Batch update stages from JSON array on stdin (per D-02, D-03).
///
/// Each JSON object must contain an "id" field plus update fields.
/// Processes all items with continue-on-error semantics (per D-06).
async fn batch_update(ctx: &AppContext) -> Result<()> {
    if io::stdin().is_terminal() {
        return Err(CliError::Validation {
            detail: "No data on stdin".to_string(),
            hint: "Pipe JSON data: echo '[{\"id\":\"stg_1\",\"name\":\"New\"}]' | pipelite stages update --stdin".to_string(),
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
            let url = format!("{}/api/v1/stages/{}", ctx.client.base_url(), id);
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

        let data: StageUpdate = match serde_json::from_value(item) {
            Ok(d) => d,
            Err(e) => {
                outcome.record_failure(i, &id, &e);
                continue;
            }
        };

        match ctx.client.update_stage(&id, &data).await {
            Ok(stage) => {
                outcome.record_success();
                succeeded.push(stage);
            }
            Err(e) => {
                outcome.record_failure(i, &id, &e);
            }
        }
    }

    // Render successes to stdout (per D-07). Batch updates may touch stages in
    // multiple pipelines, so invalidate every per-pipeline stages cache entry.
    if !succeeded.is_empty() {
        if let Some(ref cache) = ctx.cache {
            cache.invalidate(KEY_STAGES);
            cache.invalidate_prefix("stages_");
        }
        let items_json: Vec<serde_json::Value> = succeeded
            .iter()
            .map(|d| serde_json::to_value(d).map_err(Into::into))
            .collect::<Result<Vec<_>>>()?;

        let config = stages_table_config();
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

    outcome.finalize("stage", "update")
}
