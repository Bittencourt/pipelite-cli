use anyhow::Result;

use crate::api::models::deals_table_config;
use crate::cli::deals::DealsGetArgs;
use crate::context::AppContext;
use crate::output;

/// Get a single deal by ID and render as key-value layout.
///
/// Uses all Deal fields as columns for the vertical key-value display,
/// rather than just the compact default list columns.
pub async fn run(ctx: &AppContext, args: &DealsGetArgs) -> Result<()> {
    let deal = ctx
        .client
        .get_deal(&args.id, args.expand.as_deref())
        .await?;

    let item = serde_json::to_value(&deal)?;

    // For single-item view, use all deal fields (not just default list columns)
    // unless the user specified --fields
    let all_columns: Vec<String> = vec![
        "id".to_string(),
        "title".to_string(),
        "value".to_string(),
        "stage_id".to_string(),
        "organization_id".to_string(),
        "person_id".to_string(),
        "owner_id".to_string(),
        "position".to_string(),
        "expected_close_date".to_string(),
        "notes".to_string(),
        "custom_fields".to_string(),
        "created_at".to_string(),
        "updated_at".to_string(),
    ];

    // Fall back to default table config columns if none provided
    let columns = if args.fields.is_some() {
        let config = deals_table_config();
        config
            .default_columns
            .iter()
            .map(|s| s.to_string())
            .collect()
    } else {
        all_columns
    };

    output::render_single(&item, &ctx.output_format, &columns, &args.fields, ctx.color)
}
