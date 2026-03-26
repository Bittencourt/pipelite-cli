use std::io::{self, IsTerminal, Read};

use anyhow::Result;

use crate::api::models::{StageCreate, stages_table_config};
use crate::cache::KEY_STAGES;
use crate::cli::stages::StagesCreateArgs;
use crate::context::AppContext;
use crate::dry_run;
use crate::error::CliError;
use crate::output;
use crate::prompt;

/// Create a new stage (or batch create from stdin).
///
/// When --stdin is set, reads a JSON array and creates each individually
/// (no batch endpoint for stages). Otherwise, builds a single StageCreate
/// from CLI flags (with interactive prompts on TTY when flags are missing).
/// Both --name and --pipeline are required.
pub async fn run(ctx: &AppContext, args: &StagesCreateArgs) -> Result<()> {
    // Validate mutual exclusivity: --stdin vs individual flags
    if args.stdin {
        let has_flags = args.name.is_some()
            || args.pipeline.is_some()
            || args.color.is_some()
            || args.stage_type.is_some()
            || args.description.is_some()
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

/// Create a single stage from CLI flags, with interactive prompts for missing fields.
async fn single_create(ctx: &AppContext, args: &StagesCreateArgs) -> Result<()> {
    let mut missing = Vec::new();

    // Required: name
    let name = prompt::require_text(
        &args.name,
        "name",
        "Stage name",
        &mut missing,
        ctx.no_input,
    )?;

    // Required: pipeline (via FuzzySelect when on TTY)
    let pipeline_id = if args.pipeline.is_some() {
        args.pipeline.clone()
    } else if std::io::stdin().is_terminal() && !ctx.no_input {
        select_pipeline_interactive(ctx).await?
    } else {
        missing.push("--pipeline".to_string());
        None
    };

    // Check required fields before proceeding to optional ones
    prompt::check_missing(
        &missing,
        "Usage: pipelite stages create --name <name> --pipeline <pipeline_id>",
    )?;

    let name = name.unwrap();
    let pipeline_id = pipeline_id.unwrap();

    // Optional fields
    let stage_type = prompt::optional_text(
        &args.stage_type,
        "Stage type (open, won, or lost)",
        ctx.no_input,
    )?;
    let color = prompt::optional_text(&args.color, "Color (hex)", ctx.no_input)?;
    let description = prompt::optional_text(&args.description, "Description", ctx.no_input)?;

    let data = StageCreate {
        name,
        pipeline_id,
        description,
        color,
        stage_type,
    };

    // Dry-run intercept
    if ctx.dry_run {
        let body = serde_json::to_value(&data)?;
        let url = format!("{}/api/v1/stages", ctx.client.base_url());
        return dry_run::render_dry_run("POST", &url, &body, &ctx.output_format, ctx.color);
    }

    let stage = ctx.client.create_stage(&data).await?;

    if let Some(ref cache) = ctx.cache {
        cache.invalidate(KEY_STAGES);
        cache.invalidate(&format!("stages_{}", data.pipeline_id));
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

/// Interactive pipeline selection via FuzzySelect.
///
/// Uses cache-through helper for instant response when cache is warm.
async fn select_pipeline_interactive(ctx: &AppContext) -> Result<Option<String>> {
    let options = prompt::get_pipelines_cached(ctx.cache.as_ref(), &ctx.client).await?;

    let mut missing = Vec::new();
    prompt::require_select(
        &None,
        "pipeline",
        "Select pipeline",
        &options,
        &mut missing,
        false, // Already checked TTY above
    )
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

    // Dry-run: show each payload that would be sent
    if ctx.dry_run {
        let url = format!("{}/api/v1/stages", ctx.client.base_url());
        for stage in &stages {
            let body = serde_json::to_value(stage)?;
            dry_run::render_dry_run("POST", &url, &body, &ctx.output_format, ctx.color)?;
        }
        return Ok(());
    }

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

    if !created.is_empty() {
        if let Some(ref cache) = ctx.cache {
            cache.invalidate(KEY_STAGES);
            cache.invalidate_prefix("stages_");
        }
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
