use std::io::IsTerminal;

use anyhow::Result;

use crate::api::models::{WorkflowUpdate, workflows_table_config};
use crate::cache::KEY_WORKFLOWS;
use crate::cli::workflows::WorkflowsUpdateArgs;
use crate::context::AppContext;
use crate::dry_run;
use crate::error::CliError;
use crate::output;
use crate::prompt;

/// Update an existing workflow.
///
/// If no flags are provided on a TTY, prompts interactively for basic fields
/// (name, description, active). Triggers and nodes are JSON-only via flags.
/// In headless mode with no flags, returns a validation error.
pub async fn run(ctx: &AppContext, args: &WorkflowsUpdateArgs) -> Result<()> {
    let has_flags = args.name.is_some()
        || args.description.is_some()
        || args.active.is_some()
        || args.triggers.is_some()
        || args.nodes.is_some();

    // If no flags provided in headless mode, error out
    if !has_flags && (ctx.no_input || !std::io::stdin().is_terminal()) {
        return Err(CliError::Validation {
            detail: "No fields to update. Provide at least one flag.".to_string(),
            hint: "Usage: pipelite workflows update <id> --name <name> [--active <bool>]"
                .to_string(),
        }
        .into());
    }

    // Collect field values (from flags or interactive prompts)
    let name = if has_flags {
        args.name.clone()
    } else {
        prompt::optional_text(&args.name, "Name", ctx.no_input)?
    };

    let description = if has_flags {
        args.description.clone()
    } else {
        prompt::optional_text(&args.description, "Description", ctx.no_input)?
    };

    let active = if has_flags {
        args.active
    } else if std::io::stdin().is_terminal() && !ctx.no_input {
        let confirmed = dialoguer::Confirm::new()
            .with_prompt("Set workflow active?")
            .default(false)
            .interact()?;
        Some(confirmed)
    } else {
        None
    };

    // Parse JSON flags for triggers and nodes
    let triggers = parse_json_array_flag(&args.triggers, "triggers")?;
    let nodes = parse_json_array_flag(&args.nodes, "nodes")?;

    let data = WorkflowUpdate {
        name,
        description,
        triggers,
        nodes,
        active,
    };

    // Dry-run intercept
    if ctx.dry_run {
        let body = serde_json::to_value(&data)?;
        let url = format!("{}/api/v1/workflows/{}", ctx.client.base_url(), args.id);
        return dry_run::render_dry_run("PUT", &url, &body, &ctx.output_format, ctx.color);
    }

    let workflow = ctx.client.update_workflow(&args.id, &data).await?;

    if let Some(ref cache) = ctx.cache {
        cache.invalidate(KEY_WORKFLOWS);
    }

    let item = serde_json::to_value(&workflow)?;

    let config = workflows_table_config();
    let columns: Vec<String> = config
        .default_columns
        .iter()
        .map(|s| s.to_string())
        .collect();

    output::render_single(&item, &ctx.output_format, &columns, &None, ctx.color)
}

/// Parse a JSON array flag value into a Vec of serde_json::Value.
fn parse_json_array_flag(
    flag: &Option<String>,
    name: &str,
) -> Result<Option<Vec<serde_json::Value>>> {
    match flag {
        Some(s) => {
            let val: Vec<serde_json::Value> =
                serde_json::from_str(s).map_err(|e| CliError::Validation {
                    detail: format!("Invalid JSON for --{}: {}", name, e),
                    hint: format!("--{} must be a valid JSON array.", name),
                })?;
            Ok(Some(val))
        }
        None => Ok(None),
    }
}
