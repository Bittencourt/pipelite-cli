use std::io::IsTerminal;

use anyhow::Result;

use crate::api::models::{PipelineUpdate, pipelines_table_config};
use crate::cache::KEY_PIPELINES;
use crate::cli::pipelines::PipelinesUpdateArgs;
use crate::context::AppContext;
use crate::dry_run;
use crate::error::CliError;
use crate::output;
use crate::prompt;

/// Update an existing pipeline.
///
/// If no flags are provided on a TTY, prompts interactively for all fields
/// (all optional -- user can skip any). In headless mode with no flags,
/// returns a validation error.
pub async fn run(ctx: &AppContext, args: &PipelinesUpdateArgs) -> Result<()> {
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
        let url = format!("{}/api/v1/pipelines/{}", ctx.client.base_url(), args.id);
        return dry_run::render_dry_run("PUT", &url, &body, &ctx.output_format, ctx.color);
    }

    let pipeline = ctx.client.update_pipeline(&args.id, &data).await?;

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
