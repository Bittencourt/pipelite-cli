use anyhow::Result;

use crate::api::DealsListParams;
use crate::api::models::{Deal, PaginationMeta, deals_table_config};
use crate::cli::deals::DealsListArgs;
use crate::context::AppContext;
use crate::output;

/// List deals with filtering and pagination.
///
/// When --all is set, auto-paginates in batches of 100 up to 1000 records.
/// Prints a warning to stderr if more results exist beyond the cap.
pub async fn run(ctx: &AppContext, args: &DealsListArgs) -> Result<()> {
    let config = deals_table_config();
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

/// Fetch a single page of deals.
async fn fetch_page(ctx: &AppContext, args: &DealsListArgs, columns: &[String]) -> Result<()> {
    let params = DealsListParams {
        stage: args.stage.clone(),
        org: args.org.clone(),
        owner: args.owner.clone(),
        limit: args.limit,
        offset: args.offset,
        expand: args.expand.clone(),
    };

    let response = ctx.client.list_deals(&params).await?;
    let items = deals_to_values(&response.data)?;

    output::render_list(
        &items,
        &ctx.output_format,
        columns,
        &args.fields,
        ctx.color,
        Some(&response.meta),
    )
}

/// Auto-paginate to fetch all deals, up to 1000 records.
async fn fetch_all(ctx: &AppContext, args: &DealsListArgs, columns: &[String]) -> Result<()> {
    let batch_size: u64 = 100;
    let max_records: u64 = 1000;
    let mut all_deals: Vec<Deal> = Vec::new();
    let mut offset: u64 = 0;
    let mut total: u64;

    loop {
        let params = DealsListParams {
            stage: args.stage.clone(),
            org: args.org.clone(),
            owner: args.owner.clone(),
            limit: batch_size,
            offset,
            expand: args.expand.clone(),
        };

        let response = ctx.client.list_deals(&params).await?;
        total = response.meta.total;
        all_deals.extend(response.data);

        offset += batch_size;

        // Stop if we have all records or hit the cap
        if all_deals.len() as u64 >= total || all_deals.len() as u64 >= max_records {
            break;
        }
    }

    // Warn if there are more results beyond the cap
    if total > max_records {
        eprintln!("warning: --all stopped at 1000 records (server ceiling); results may be incomplete");
    }

    let items = deals_to_values(&all_deals)?;
    let meta = PaginationMeta {
        total,
        offset: 0,
        limit: all_deals.len() as u64,
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

/// Convert a slice of Deal structs to serde_json::Value for the output layer.
fn deals_to_values(deals: &[Deal]) -> Result<Vec<serde_json::Value>> {
    deals
        .iter()
        .map(|d| serde_json::to_value(d).map_err(Into::into))
        .collect()
}
