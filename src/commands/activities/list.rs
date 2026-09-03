use anyhow::Result;

use crate::api::ActivitiesListParams;
use crate::api::models::{Activity, PaginationMeta, activities_table_config};
use crate::cli::activities::ActivitiesListArgs;
use crate::context::AppContext;
use crate::output;

/// List activities with filtering and pagination.
///
/// When --all is set, auto-paginates in batches of 100 up to 1000 records.
/// When --done is set, filters to only completed activities (client-side).
pub async fn run(ctx: &AppContext, args: &ActivitiesListArgs) -> Result<()> {
    let config = activities_table_config();
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

/// Fetch a single page of activities.
async fn fetch_page(ctx: &AppContext, args: &ActivitiesListArgs, columns: &[String]) -> Result<()> {
    let params = ActivitiesListParams {
        type_id: args.type_id.clone(),
        deal_id: args.deal.clone(),
        owner_id: args.owner.clone(),
        limit: args.limit,
        offset: args.offset,
        expand: args.expand.clone(),
    };

    let response = ctx.client.list_activities(&params).await?;
    let items = filter_done(activities_to_values(&response.data)?, args.done);

    output::render_list(
        &items,
        &ctx.output_format,
        columns,
        &args.fields,
        ctx.color,
        Some(&response.meta),
    )
}

/// Auto-paginate to fetch all activities, up to 1000 records.
async fn fetch_all(ctx: &AppContext, args: &ActivitiesListArgs, columns: &[String]) -> Result<()> {
    let batch_size: u64 = 100;
    let max_records: u64 = 1000;
    let mut all_activities: Vec<Activity> = Vec::new();
    let mut offset: u64 = 0;
    let mut total: u64;

    loop {
        let params = ActivitiesListParams {
            type_id: args.type_id.clone(),
            deal_id: args.deal.clone(),
            owner_id: args.owner.clone(),
            limit: batch_size,
            offset,
            expand: args.expand.clone(),
        };

        let response = ctx.client.list_activities(&params).await?;
        total = response.meta.total;
        all_activities.extend(response.data);

        offset += batch_size;

        if all_activities.len() as u64 >= total || all_activities.len() as u64 >= max_records {
            break;
        }
    }

    if total > max_records {
        eprintln!("warning: --all stopped at 1000 records (server ceiling); results may be incomplete");
    }

    let items = filter_done(activities_to_values(&all_activities)?, args.done);
    let meta = PaginationMeta {
        total,
        offset: 0,
        limit: all_activities.len() as u64,
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

/// Convert a slice of Activity structs to serde_json::Value for the output layer.
fn activities_to_values(activities: &[Activity]) -> Result<Vec<serde_json::Value>> {
    activities
        .iter()
        .map(|a| serde_json::to_value(a).map_err(Into::into))
        .collect()
}

/// Filter activities by done status (client-side).
/// When done=true, keep only activities where completed_at is set.
/// When done=false (default), return all activities.
fn filter_done(items: Vec<serde_json::Value>, done: bool) -> Vec<serde_json::Value> {
    if !done {
        return items;
    }
    items
        .into_iter()
        .filter(|item| {
            item.get("completed_at")
                .map(|v| !v.is_null())
                .unwrap_or(false)
        })
        .collect()
}
