use std::io::{self, IsTerminal, Read};

use anyhow::Result;

use crate::api::models::{WorkflowCreate, workflows_table_config};
use crate::cache::KEY_WORKFLOWS;
use crate::cli::workflows::WorkflowsCreateArgs;
use crate::context::AppContext;
use crate::dry_run;
use crate::error::CliError;
use crate::output;
use crate::prompt;

/// Create a new workflow (or from stdin JSON).
///
/// When --stdin is set, reads a JSON WorkflowCreate object from stdin.
/// Otherwise, builds from CLI flags with interactive prompts for basic
/// fields (name, description, active) on TTY. Triggers and nodes are
/// JSON-only via --triggers/--nodes flags.
pub async fn run(ctx: &AppContext, args: &WorkflowsCreateArgs) -> Result<()> {
    if args.stdin {
        let has_flags = args.name.is_some()
            || args.description.is_some()
            || args.active.is_some()
            || args.triggers.is_some()
            || args.nodes.is_some();

        if has_flags {
            return Err(CliError::Validation {
                detail: "--stdin and individual field flags are mutually exclusive".to_string(),
                hint: "Use either --stdin or individual flags, not both.".to_string(),
            }
            .into());
        }

        return stdin_create(ctx).await;
    }

    single_create(ctx, args).await
}

/// Create a single workflow from CLI flags, with interactive prompts for basic fields.
async fn single_create(ctx: &AppContext, args: &WorkflowsCreateArgs) -> Result<()> {
    let mut missing = Vec::new();

    // Required: name
    let name = prompt::require_text(
        &args.name,
        "name",
        "Workflow name",
        &mut missing,
        ctx.no_input,
    )?;

    prompt::check_missing(
        &missing,
        "Usage: pipelite workflows create --name <name>",
    )?;

    let name = name.unwrap();

    // Optional: description
    let description = prompt::optional_text(&args.description, "Description", ctx.no_input)?;

    // Optional: active toggle (interactive Confirm on TTY)
    let active = if args.active.is_some() {
        args.active
    } else if io::stdin().is_terminal() && !ctx.no_input {
        let confirmed = dialoguer::Confirm::new()
            .with_prompt("Set workflow active?")
            .default(false)
            .interact()?;
        Some(confirmed)
    } else {
        None
    };

    // Optional: triggers and nodes as JSON string flags
    let triggers = parse_json_array_flag(&args.triggers, "triggers")?;
    let nodes = parse_json_array_flag(&args.nodes, "nodes")?;

    let data = WorkflowCreate {
        name,
        description,
        triggers,
        nodes,
        active,
    };

    // Dry-run intercept
    if ctx.dry_run {
        let body = serde_json::to_value(&data)?;
        let url = format!("{}/api/v1/workflows", ctx.client.base_url());
        return dry_run::render_dry_run("POST", &url, &body, &ctx.output_format, ctx.color);
    }

    let workflow = ctx.client.create_workflow(&data).await?;

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

/// Create a workflow from JSON on stdin.
async fn stdin_create(ctx: &AppContext) -> Result<()> {
    if io::stdin().is_terminal() {
        return Err(CliError::Validation {
            detail: "No data on stdin".to_string(),
            hint: "Pipe JSON data via stdin: echo '{\"name\":\"WF\"}' | pipelite workflows create --stdin"
                .to_string(),
        }
        .into());
    }

    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let data: WorkflowCreate = serde_json::from_str(&input).map_err(|e| CliError::Validation {
        detail: format!("Invalid JSON input: {}", e),
        hint: "Stdin must contain a JSON object with at least a 'name' field.".to_string(),
    })?;

    // Dry-run: show what would be sent
    if ctx.dry_run {
        let body = serde_json::to_value(&data)?;
        let url = format!("{}/api/v1/workflows", ctx.client.base_url());
        return dry_run::render_dry_run("POST", &url, &body, &ctx.output_format, ctx.color);
    }

    let workflow = ctx.client.create_workflow(&data).await?;

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

/// Parse a JSON array flag value (e.g., --triggers, --nodes) into a Vec of serde_json::Value.
///
/// Returns CliError::Validation with a helpful message on parse failure.
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
