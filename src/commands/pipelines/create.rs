use std::io::{self, IsTerminal, Read};

use anyhow::Result;

use crate::api::models::{PipelineCreate, pipelines_table_config};
use crate::cli::pipelines::PipelinesCreateArgs;
use crate::context::AppContext;
use crate::dry_run;
use crate::error::CliError;
use crate::output;
use crate::prompt;

/// Create a new pipeline (or batch create from stdin).
///
/// When --stdin is set, reads a JSON array and creates each individually
/// (no batch endpoint for pipelines). Otherwise, builds a single PipelineCreate
/// from CLI flags (with interactive prompts on TTY when flags are missing).
pub async fn run(ctx: &AppContext, args: &PipelinesCreateArgs) -> Result<()> {
    // Validate mutual exclusivity: --stdin vs individual flags
    if args.stdin {
        let has_flags = args.name.is_some()
            || args.default
            || !args.custom_field.is_empty();

        if has_flags {
            return Err(CliError::Validation {
                detail: "--stdin and individual field flags are mutually exclusive".to_string(),
                hint: "Use either --stdin or individual flags, not both.".to_string(),
            }
            .into());
        }

        return batch_create(ctx).await;
    }

    single_create(ctx, args).await
}

/// Create a single pipeline from CLI flags, with interactive prompts for missing fields.
async fn single_create(ctx: &AppContext, args: &PipelinesCreateArgs) -> Result<()> {
    let mut missing = Vec::new();

    // Required: name
    let name = prompt::require_text(
        &args.name,
        "name",
        "Pipeline name",
        &mut missing,
        ctx.no_input,
    )?;

    // Check required fields before proceeding to optional ones
    prompt::check_missing(
        &missing,
        "Usage: pipelite pipelines create --name <name>",
    )?;

    let name = name.unwrap();

    // Optional: default (boolean -- use Confirm on TTY, flag value in headless)
    let is_default = if args.default {
        Some(true)
    } else if std::io::stdin().is_terminal() && !ctx.no_input {
        let confirm = dialoguer::Confirm::new()
            .with_prompt("Set as default pipeline?")
            .default(false)
            .interact()?;
        if confirm { Some(true) } else { None }
    } else {
        None
    };

    let data = PipelineCreate {
        name,
        is_default,
    };

    // Dry-run intercept
    if ctx.dry_run {
        let body = serde_json::to_value(&data)?;
        let url = format!("{}/api/v1/pipelines", ctx.client.base_url());
        return dry_run::render_dry_run("POST", &url, &body, &ctx.output_format, ctx.color);
    }

    let pipeline = ctx.client.create_pipeline(&data).await?;
    let item = serde_json::to_value(&pipeline)?;

    let config = pipelines_table_config();
    let columns: Vec<String> = config
        .default_columns
        .iter()
        .map(|s| s.to_string())
        .collect();

    output::render_single(&item, &ctx.output_format, &columns, &None, ctx.color)
}

/// Batch create pipelines from JSON array on stdin (individual-create loop).
async fn batch_create(ctx: &AppContext) -> Result<()> {
    if io::stdin().is_terminal() {
        return Err(CliError::Validation {
            detail: "No data on stdin".to_string(),
            hint: "Pipe JSON data via stdin: echo '[{...}]' | pipelite pipelines create --stdin"
                .to_string(),
        }
        .into());
    }

    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let pipelines: Vec<PipelineCreate> =
        serde_json::from_str(&input).map_err(|e| CliError::Validation {
            detail: format!("Invalid JSON input: {}", e),
            hint: "Stdin must contain a JSON array of pipeline objects with 'name' field."
                .to_string(),
        })?;

    // Dry-run: show each payload that would be sent
    if ctx.dry_run {
        let url = format!("{}/api/v1/pipelines", ctx.client.base_url());
        for pipeline in &pipelines {
            let body = serde_json::to_value(pipeline)?;
            dry_run::render_dry_run("POST", &url, &body, &ctx.output_format, ctx.color)?;
        }
        return Ok(());
    }

    let total = pipelines.len();
    let mut created = Vec::new();
    let mut errors = Vec::new();

    for (i, pipeline_data) in pipelines.iter().enumerate() {
        match ctx.client.create_pipeline(pipeline_data).await {
            Ok(pipeline) => created.push(pipeline),
            Err(e) => {
                eprintln!("[{}/{}] Failed to create pipeline: {}", i + 1, total, e);
                errors.push(e);
            }
        }
    }

    if !errors.is_empty() {
        eprintln!(
            "Created {}/{} pipelines ({} failed)",
            created.len(),
            total,
            errors.len()
        );
    }

    let items: Vec<serde_json::Value> = created
        .iter()
        .map(|p| serde_json::to_value(p).map_err(Into::into))
        .collect::<Result<Vec<_>>>()?;

    let config = pipelines_table_config();
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
