use anyhow::Result;

use crate::api::OrgsListParams;
use crate::api::models::{Organization, PaginationMeta, orgs_table_config};
use crate::cli::orgs::OrgsListArgs;
use crate::context::AppContext;
use crate::output;

/// List organizations with filtering and pagination.
///
/// When --all is set, auto-paginates in batches of 100 up to 1000 records.
/// Prints a warning to stderr if more results exist beyond the cap.
pub async fn run(ctx: &AppContext, args: &OrgsListArgs) -> Result<()> {
    let config = orgs_table_config();
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

/// Fetch a single page of organizations.
async fn fetch_page(ctx: &AppContext, args: &OrgsListArgs, columns: &[String]) -> Result<()> {
    let params = OrgsListParams {
        owner: args.owner.clone(),
        limit: args.limit,
        offset: args.offset,
        expand: args.expand.clone(),
    };

    let response = ctx.client.list_orgs(&params).await?;
    let items = orgs_to_values(&response.data)?;

    output::render_list(
        &items,
        &ctx.output_format,
        columns,
        &args.fields,
        ctx.color,
        Some(&response.meta),
    )
}

/// Auto-paginate to fetch all organizations, up to 1000 records.
async fn fetch_all(ctx: &AppContext, args: &OrgsListArgs, columns: &[String]) -> Result<()> {
    let batch_size: u64 = 100;
    let max_records: u64 = 1000;
    let mut all_orgs: Vec<Organization> = Vec::new();
    let mut offset: u64 = 0;
    let mut total: u64 = 0;

    loop {
        let params = OrgsListParams {
            owner: args.owner.clone(),
            limit: batch_size,
            offset,
            expand: args.expand.clone(),
        };

        let response = ctx.client.list_orgs(&params).await?;
        total = response.meta.total;
        all_orgs.extend(response.data);

        offset += batch_size;

        if all_orgs.len() as u64 >= total || all_orgs.len() as u64 >= max_records {
            break;
        }
    }

    if total > max_records {
        eprintln!(
            "Showing {} of {}. Use --limit/--offset for more.",
            max_records, total
        );
    }

    let items = orgs_to_values(&all_orgs)?;
    let meta = PaginationMeta {
        total,
        offset: 0,
        limit: all_orgs.len() as u64,
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

/// Convert a slice of Organization structs to serde_json::Value for the output layer.
fn orgs_to_values(orgs: &[Organization]) -> Result<Vec<serde_json::Value>> {
    orgs.iter()
        .map(|o| serde_json::to_value(o).map_err(Into::into))
        .collect()
}
