use anyhow::Result;

use crate::api::WorkflowsListParams;
use crate::api::models::{PaginationMeta, Workflow, workflows_table_config};
use crate::cli::workflows::WorkflowsListArgs;
use crate::context::AppContext;
use crate::output;

/// List workflows with filtering and pagination.
///
/// When --all is set, auto-paginates in batches of 100 up to 1000 records.
/// Prints a warning to stderr if more results exist beyond the cap.
pub async fn run(ctx: &AppContext, args: &WorkflowsListArgs) -> Result<()> {
    let config = workflows_table_config();
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

/// Fetch a single page of workflows.
async fn fetch_page(ctx: &AppContext, args: &WorkflowsListArgs, columns: &[String]) -> Result<()> {
    let params = WorkflowsListParams {
        active: args.active,
        limit: args.limit,
        offset: args.offset,
        expand: args.expand.clone(),
    };

    let response = ctx.client.list_workflows(&params).await?;
    let items = workflows_to_values(&response.data)?;

    output::render_list(
        &items,
        &ctx.output_format,
        columns,
        &args.fields,
        ctx.color,
        Some(&response.meta),
    )
}

/// Auto-paginate to fetch all workflows, up to 1000 records.
async fn fetch_all(ctx: &AppContext, args: &WorkflowsListArgs, columns: &[String]) -> Result<()> {
    let batch_size: u64 = 100;
    let max_records: u64 = 1000;
    let mut all_workflows: Vec<Workflow> = Vec::new();
    let mut offset: u64 = 0;
    let mut total: u64 = 0;

    loop {
        let params = WorkflowsListParams {
            active: args.active,
            limit: batch_size,
            offset,
            expand: args.expand.clone(),
        };

        let response = ctx.client.list_workflows(&params).await?;
        total = response.meta.total;
        all_workflows.extend(response.data);

        offset += batch_size;

        if all_workflows.len() as u64 >= total || all_workflows.len() as u64 >= max_records {
            break;
        }
    }

    if total > max_records {
        eprintln!(
            "Showing {} of {}. Use --limit/--offset for more.",
            max_records, total
        );
    }

    let items = workflows_to_values(&all_workflows)?;
    let meta = PaginationMeta {
        total,
        offset: 0,
        limit: all_workflows.len() as u64,
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

/// Convert a slice of Workflow structs to serde_json::Value for the output layer.
fn workflows_to_values(workflows: &[Workflow]) -> Result<Vec<serde_json::Value>> {
    workflows
        .iter()
        .map(|w| serde_json::to_value(w).map_err(Into::into))
        .collect()
}
