use std::io::IsTerminal;

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
/// The shared flow lives in [`crate::batch::run_batch_update`].
async fn batch_update(ctx: &AppContext) -> Result<()> {
    batch::run_batch_update::<StageUpdate, _>(
        ctx,
        "stage",
        r#"[{"id":"stg_1","name":"New"}]"#,
        "stages",
        KEY_STAGES,
        // Batch updates may touch stages in multiple pipelines, so invalidate
        // every per-pipeline stages cache entry too.
        Some("stages_"),
        &stages_table_config().default_columns,
        async |id: String, data: StageUpdate, _raw: &serde_json::Value| {
            batch::ensure_update_fields(&data, "stage")?;
            let stage = ctx.client.update_stage(&id, &data).await?;
            Ok(serde_json::to_value(stage)?)
        },
    )
    .await
}
