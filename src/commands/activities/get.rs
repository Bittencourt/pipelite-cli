use anyhow::Result;

use crate::cli::activities::ActivitiesGetArgs;
use crate::context::AppContext;
use crate::output;

/// Get a single activity by ID and render as key-value layout.
pub async fn run(ctx: &AppContext, args: &ActivitiesGetArgs) -> Result<()> {
    let activity = ctx
        .client
        .get_activity(&args.id, args.expand.as_deref())
        .await?;

    let item = serde_json::to_value(&activity)?;

    let all_columns: Vec<String> = vec![
        "id".to_string(),
        "title".to_string(),
        "type_id".to_string(),
        "deal_id".to_string(),
        "owner_id".to_string(),
        "due_at".to_string(),
        "completed_at".to_string(),
        "notes".to_string(),
        "custom_fields".to_string(),
        "created_at".to_string(),
        "updated_at".to_string(),
    ];

    let columns = all_columns;

    output::render_single(&item, &ctx.output_format, &columns, &args.fields, ctx.color)
}
