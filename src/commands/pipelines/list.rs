use anyhow::Result;

use crate::api::PipelinesListParams;
use crate::api::models::{Pipeline, PaginationMeta, pipelines_table_config};
use crate::cli::pipelines::PipelinesListArgs;
use crate::context::AppContext;
use crate::output;

/// List pipelines with pagination.
///
/// When --all is set, auto-paginates in batches of 100 up to 1000 records.
/// Prints a warning to stderr if more results exist beyond the cap.
pub async fn run(ctx: &AppContext, args: &PipelinesListArgs) -> Result<()> {
    let config = pipelines_table_config();
    let columns: Vec<String> = config
        .default_columns
        .iter()
        .map(|s| s.to_string())
        .collect();

    if args.all {
        fetch_all(ctx, args, &columns).await
    } else {
        fetch_page(ctx, args, &columns).await
    }
}

/// Fetch a single page of pipelines.
async fn fetch_page(ctx: &AppContext, args: &PipelinesListArgs, columns: &[String]) -> Result<()> {
    let params = PipelinesListParams {
        limit: args.limit,
        offset: args.offset,
        expand: args.expand.clone(),
    };

    let response = ctx.client.list_pipelines(&params).await?;
    let items = pipelines_to_values(&response.data)?;

    output::render_list(
        &items,
        &ctx.output_format,
        columns,
        &args.fields,
        ctx.color,
        Some(&response.meta),
    )
}

/// Auto-paginate to fetch all pipelines, up to 1000 records.
async fn fetch_all(ctx: &AppContext, args: &PipelinesListArgs, columns: &[String]) -> Result<()> {
    let batch_size: u64 = 100;
    let max_records: u64 = 1000;
    let mut all_pipelines: Vec<Pipeline> = Vec::new();
    let mut offset: u64 = 0;
    let mut total: u64 = 0;

    loop {
        let params = PipelinesListParams {
            limit: batch_size,
            offset,
            expand: args.expand.clone(),
        };

        let response = ctx.client.list_pipelines(&params).await?;
        total = response.meta.total;
        all_pipelines.extend(response.data);

        offset += batch_size;

        if all_pipelines.len() as u64 >= total || all_pipelines.len() as u64 >= max_records {
            break;
        }
    }

    if total > max_records {
        eprintln!(
            "Showing {} of {}. Use --limit/--offset for more.",
            max_records, total
        );
    }

    let items = pipelines_to_values(&all_pipelines)?;
    let meta = PaginationMeta {
        total,
        offset: 0,
        limit: all_pipelines.len() as u64,
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

/// Convert a slice of Pipeline structs to serde_json::Value for the output layer.
fn pipelines_to_values(pipelines: &[Pipeline]) -> Result<Vec<serde_json::Value>> {
    pipelines
        .iter()
        .map(|p| serde_json::to_value(p).map_err(Into::into))
        .collect()
}
