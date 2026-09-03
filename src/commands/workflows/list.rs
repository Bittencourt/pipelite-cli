use anyhow::Result;

use crate::api::WorkflowsListParams;
use crate::api::models::{PaginationMeta, Workflow, workflows_table_config};
use crate::cli::workflows::WorkflowsListArgs;
use crate::context::AppContext;
use crate::output;

/// List workflows with pagination.
///
/// The server ignores the `active` query param, so `--active` is applied as a
/// client-side filter AFTER fetching. When set, one stderr warning is printed
/// per invocation so scripts can detect the changed semantics (FIX-01).
/// When --all is set, auto-paginates in batches of 100 up to 1000 records.
pub async fn run(ctx: &AppContext, args: &WorkflowsListArgs) -> Result<()> {
    let config = workflows_table_config();
    let columns: Vec<String> = config
        .default_columns
        .iter()
        .map(|s| s.to_string())
        .collect();

    if args.active.is_some() {
        eprintln!("warning: --active filters client-side after fetching all records");
    }

    if args.all {
        fetch_all(ctx, args, &columns).await
    } else {
        fetch_page(ctx, args, &columns).await
    }
}

/// Fetch a single page of workflows.
async fn fetch_page(ctx: &AppContext, args: &WorkflowsListArgs, columns: &[String]) -> Result<()> {
    let params = WorkflowsListParams {
        limit: args.limit,
        offset: args.offset,
        expand: args.expand.clone(),
    };

    let response = ctx.client.list_workflows(&params).await?;

    let (data, meta) = apply_active_filter(response.data, response.meta, args.active);
    let items = workflows_to_values(&data)?;

    output::render_list(
        &items,
        &ctx.output_format,
        columns,
        &args.fields,
        ctx.color,
        Some(&meta),
    )
}

/// Auto-paginate to fetch all workflows, up to 1000 records.
async fn fetch_all(ctx: &AppContext, args: &WorkflowsListArgs, columns: &[String]) -> Result<()> {
    let batch_size: u64 = 100;
    let max_records: u64 = 1000;
    let mut all_workflows: Vec<Workflow> = Vec::new();
    let mut offset: u64 = 0;
    let mut total: u64;

    loop {
        let params = WorkflowsListParams {
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
        eprintln!("warning: --all stopped at 1000 records (server ceiling); results may be incomplete");
    }

    let pre_meta = PaginationMeta {
        total,
        offset: 0,
        limit: all_workflows.len() as u64,
    };
    let (data, meta) = apply_active_filter(all_workflows, pre_meta, args.active);
    let items = workflows_to_values(&data)?;

    output::render_list(
        &items,
        &ctx.output_format,
        columns,
        &args.fields,
        ctx.color,
        Some(&meta),
    )
}

/// Apply the client-side --active filter and rebuild the pagination metadata
/// from the filtered items, so the footer never claims more rows than shown.
/// Unfiltered input passes through with its original meta.
fn apply_active_filter(
    data: Vec<Workflow>,
    meta: PaginationMeta,
    active: Option<bool>,
) -> (Vec<Workflow>, PaginationMeta) {
    match active {
        None => (data, meta),
        Some(want) => {
            let filtered: Vec<Workflow> =
                data.into_iter().filter(|w| w.active == want).collect();
            let meta = PaginationMeta {
                total: filtered.len() as u64,
                offset: 0,
                limit: filtered.len() as u64,
            };
            (filtered, meta)
        }
    }
}

/// Convert a slice of Workflow structs to serde_json::Value for the output layer.
fn workflows_to_values(workflows: &[Workflow]) -> Result<Vec<serde_json::Value>> {
    workflows
        .iter()
        .map(|w| serde_json::to_value(w).map_err(Into::into))
        .collect()
}
