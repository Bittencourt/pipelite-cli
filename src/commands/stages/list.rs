use anyhow::Result;

use crate::api::StagesListParams;
use crate::api::models::{PaginationMeta, Stage, stages_table_config};
use crate::cli::stages::StagesListArgs;
use crate::context::AppContext;
use crate::error::CliError;
use crate::output;

/// List stages for a pipeline.
///
/// Validates that --pipeline is provided before making any API calls.
/// When --all is set, auto-paginates in batches of 100 up to 1000 records.
pub async fn run(ctx: &AppContext, args: &StagesListArgs) -> Result<()> {
    let pipeline_id = args.pipeline.as_ref().ok_or_else(|| CliError::Validation {
        detail: "Missing required flag: --pipeline".to_string(),
        hint: "Usage: pipelite stages list --pipeline <pipeline_id>".to_string(),
    })?;

    let config = stages_table_config();
    let columns: Vec<String> = config
        .default_columns
        .iter()
        .map(|s| s.to_string())
        .collect();

    if args.all {
        fetch_all(ctx, args, pipeline_id, &columns).await
    } else {
        fetch_page(ctx, args, pipeline_id, &columns).await
    }
}

/// Fetch a single page of stages.
async fn fetch_page(
    ctx: &AppContext,
    args: &StagesListArgs,
    pipeline_id: &str,
    columns: &[String],
) -> Result<()> {
    let params = StagesListParams {
        pipeline_id: pipeline_id.to_string(),
        limit: args.limit,
        offset: args.offset,
        expand: args.expand.clone(),
    };

    let response = ctx.client.list_stages(&params).await?;
    let items = stages_to_values(&response.data)?;

    output::render_list(
        &items,
        &ctx.output_format,
        columns,
        &args.fields,
        ctx.color,
        Some(&response.meta),
    )
}

/// Auto-paginate to fetch all stages, up to 1000 records.
async fn fetch_all(
    ctx: &AppContext,
    args: &StagesListArgs,
    pipeline_id: &str,
    columns: &[String],
) -> Result<()> {
    let batch_size: u64 = 100;
    let max_records: u64 = 1000;
    let mut all_stages: Vec<Stage> = Vec::new();
    let mut offset: u64 = 0;
    let mut total: u64;

    loop {
        let params = StagesListParams {
            pipeline_id: pipeline_id.to_string(),
            limit: batch_size,
            offset,
            expand: args.expand.clone(),
        };

        let response = ctx.client.list_stages(&params).await?;
        total = response.meta.total;
        all_stages.extend(response.data);

        offset += batch_size;

        if all_stages.len() as u64 >= total || all_stages.len() as u64 >= max_records {
            break;
        }
    }

    if total > max_records {
        eprintln!(
            "Showing {} of {}. Use --limit/--offset for more.",
            max_records, total
        );
    }

    let items = stages_to_values(&all_stages)?;
    let meta = PaginationMeta {
        total,
        offset: 0,
        limit: all_stages.len() as u64,
    };

    output::render_list(
        &items,
        &ctx.output_format,
        columns,
        &args.fields,
        ctx.color,
        Some(&meta),
    )
}

/// Convert a slice of Stage structs to serde_json::Value for the output layer.
fn stages_to_values(stages: &[Stage]) -> Result<Vec<serde_json::Value>> {
    stages
        .iter()
        .map(|s| serde_json::to_value(s).map_err(Into::into))
        .collect()
}
