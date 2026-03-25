use std::io::{self, IsTerminal, Read};

use anyhow::Result;

use crate::api::models::{PipelineCreate, pipelines_table_config};
use crate::cli::pipelines::PipelinesCreateArgs;
use crate::context::AppContext;
use crate::error::CliError;
use crate::output;

/// Create a new pipeline (or batch create from stdin).
///
/// When --stdin is set, reads a JSON array and creates each individually
/// (no batch endpoint for pipelines). Otherwise, builds a single PipelineCreate
/// from CLI flags.
pub async fn run(ctx: &AppContext, args: &PipelinesCreateArgs) -> Result<()> {
    if args.stdin {
        batch_create(ctx).await
    } else {
        single_create(ctx, args).await
    }
}

/// Create a single pipeline from CLI flags.
async fn single_create(ctx: &AppContext, args: &PipelinesCreateArgs) -> Result<()> {
    let name = args.name.as_ref().ok_or_else(|| CliError::Validation {
        detail: "Missing required flag: --name".to_string(),
        hint: "Usage: pipelite pipelines create --name <name>".to_string(),
    })?;

    let is_default = if args.default { Some(true) } else { None };

    let data = PipelineCreate {
        name: name.clone(),
        is_default,
    };

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
