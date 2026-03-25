use std::io::{self, IsTerminal, Read};

use anyhow::Result;

use crate::api::models::{StageCreate, stages_table_config};
use crate::cli::stages::StagesCreateArgs;
use crate::context::AppContext;
use crate::error::CliError;
use crate::output;

/// Create a new stage (or batch create from stdin).
///
/// When --stdin is set, reads a JSON array and creates each individually
/// (no batch endpoint for stages). Otherwise, builds a single StageCreate
/// from CLI flags. Both --name and --pipeline are required.
pub async fn run(ctx: &AppContext, args: &StagesCreateArgs) -> Result<()> {
    if args.stdin {
        batch_create(ctx).await
    } else {
        single_create(ctx, args).await
    }
}

/// Create a single stage from CLI flags.
async fn single_create(ctx: &AppContext, args: &StagesCreateArgs) -> Result<()> {
    let name = args.name.as_ref().ok_or_else(|| CliError::Validation {
        detail: "Missing required flag: --name".to_string(),
        hint: "Usage: pipelite stages create --name <name> --pipeline <pipeline_id>".to_string(),
    })?;

    let pipeline_id = args.pipeline.as_ref().ok_or_else(|| CliError::Validation {
        detail: "Missing required flag: --pipeline".to_string(),
        hint: "Usage: pipelite stages create --name <name> --pipeline <pipeline_id>".to_string(),
    })?;

    let data = StageCreate {
        name: name.clone(),
        pipeline_id: pipeline_id.clone(),
        description: args.description.clone(),
        color: args.color.clone(),
        stage_type: args.stage_type.clone(),
    };

    let stage = ctx.client.create_stage(&data).await?;
    let item = serde_json::to_value(&stage)?;

    let config = stages_table_config();
    let columns: Vec<String> = config
        .default_columns
        .iter()
        .map(|s| s.to_string())
        .collect();

    output::render_single(&item, &ctx.output_format, &columns, &None, ctx.color)
}

/// Batch create stages from JSON array on stdin (individual-create loop).
async fn batch_create(ctx: &AppContext) -> Result<()> {
    if io::stdin().is_terminal() {
        return Err(CliError::Validation {
            detail: "No data on stdin".to_string(),
            hint: "Pipe JSON data via stdin: echo '[{...}]' | pipelite stages create --stdin"
                .to_string(),
        }
        .into());
    }

    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let stages: Vec<StageCreate> =
        serde_json::from_str(&input).map_err(|e| CliError::Validation {
            detail: format!("Invalid JSON input: {}", e),
            hint: "Stdin must contain a JSON array of stage objects with 'name' and 'pipeline_id' fields."
                .to_string(),
        })?;

    let total = stages.len();
    let mut created = Vec::new();
    let mut errors = Vec::new();

    for (i, stage_data) in stages.iter().enumerate() {
        match ctx.client.create_stage(stage_data).await {
            Ok(stage) => created.push(stage),
            Err(e) => {
                eprintln!("[{}/{}] Failed to create stage: {}", i + 1, total, e);
                errors.push(e);
            }
        }
    }

    if !errors.is_empty() {
        eprintln!(
            "Created {}/{} stages ({} failed)",
            created.len(),
            total,
            errors.len()
        );
    }

    let items: Vec<serde_json::Value> = created
        .iter()
        .map(|s| serde_json::to_value(s).map_err(Into::into))
        .collect::<Result<Vec<_>>>()?;

    let config = stages_table_config();
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
