use std::io::IsTerminal;

use anyhow::Result;

use crate::api::models::{PipelineUpdate, pipelines_table_config};
use crate::batch;
use crate::cache::KEY_PIPELINES;
use crate::cli::pipelines::PipelinesUpdateArgs;
use crate::context::AppContext;
use crate::dry_run;
use crate::error::CliError;
use crate::output;
use crate::prompt;

/// Update an existing pipeline.
///
/// With --stdin, reads a JSON array of objects (each with an "id" field plus
/// update fields) from stdin and batch-updates with continue-on-error.
/// If no flags are provided on a TTY, prompts interactively for all fields
/// (all optional -- user can skip any). In headless mode with no flags,
/// returns a validation error.
pub async fn run(ctx: &AppContext, args: &PipelinesUpdateArgs) -> Result<()> {
    if args.stdin {
        let has_flags = args.name.is_some() || args.default;

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
        detail: "Missing pipeline ID".to_string(),
        hint: "Provide a pipeline ID or use --stdin.".to_string(),
    })?;

    let has_flags = args.name.is_some() || args.default;

    // If no flags provided in headless mode, error out
    if !has_flags && (ctx.no_input || !std::io::stdin().is_terminal()) {
        return Err(CliError::Validation {
            detail: "No fields to update. Provide at least one flag.".to_string(),
            hint: "Usage: pipelite pipelines update <id> --name <name> [--default]".to_string(),
        }
        .into());
    }

    // Collect field values (from flags or interactive prompts)
    let name = if has_flags {
        args.name.clone()
    } else {
        prompt::optional_text(&args.name, "Pipeline name", ctx.no_input)?
    };

    let is_default = if args.default {
        Some(true)
    } else if !has_flags && std::io::stdin().is_terminal() && !ctx.no_input {
        let confirm = dialoguer::Confirm::new()
            .with_prompt("Set as default pipeline?")
            .default(false)
            .interact()?;
        if confirm { Some(true) } else { None }
    } else {
        None
    };

    let data = PipelineUpdate {
        name,
        is_default,
    };

    // Dry-run intercept
    if ctx.dry_run {
        let body = serde_json::to_value(&data)?;
        let url = format!("{}/api/v1/pipelines/{}", ctx.client.base_url(), id);
        return dry_run::render_dry_run("PUT", &url, &body, &ctx.output_format, ctx.color);
    }

    let pipeline = ctx.client.update_pipeline(id, &data).await?;

    if let Some(ref cache) = ctx.cache {
        cache.invalidate(KEY_PIPELINES);
        cache.invalidate_prefix("stages_");
    }

    let item = serde_json::to_value(&pipeline)?;

    let config = pipelines_table_config();
    let columns: Vec<String> = config
        .default_columns
        .iter()
        .map(|s| s.to_string())
        .collect();

    output::render_single(&item, &ctx.output_format, &columns, &None, ctx.color)
}

/// Batch update pipelines from JSON array on stdin (per D-02, D-03).
///
/// Each JSON object must contain an "id" field plus update fields.
/// Processes all items with continue-on-error semantics (per D-06).
/// The shared flow lives in [`crate::batch::run_batch_update`].
async fn batch_update(ctx: &AppContext) -> Result<()> {
    batch::run_batch_update::<PipelineUpdate, _>(
        ctx,
        "pipeline",
        r#"[{"id":"pl_1","name":"New"}]"#,
        "pipelines",
        KEY_PIPELINES,
        Some("stages_"),
        &pipelines_table_config().default_columns,
        async |id: String, data: PipelineUpdate, _raw: &serde_json::Value| {
            batch::ensure_update_fields(&data, "pipeline")?;
            let pipeline = ctx.client.update_pipeline(&id, &data).await?;
            Ok(serde_json::to_value(pipeline)?)
        },
    )
    .await
}
