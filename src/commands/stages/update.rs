use std::io::IsTerminal;

use anyhow::Result;

use crate::api::models::{StageUpdate, stages_table_config};
use crate::cache::KEY_STAGES;
use crate::cli::stages::StagesUpdateArgs;
use crate::context::AppContext;
use crate::dry_run;
use crate::error::CliError;
use crate::output;
use crate::prompt;

/// Update an existing stage.
///
/// If no flags are provided on a TTY, prompts interactively for all fields
/// (all optional -- user can skip any). No --pipeline needed -- stage ID is unique.
/// In headless mode with no flags, returns a validation error.
pub async fn run(ctx: &AppContext, args: &StagesUpdateArgs) -> Result<()> {
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
        let url = format!("{}/api/v1/stages/{}", ctx.client.base_url(), args.id);
        return dry_run::render_dry_run("PUT", &url, &body, &ctx.output_format, ctx.color);
    }

    let stage = ctx.client.update_stage(&args.id, &data).await?;

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
