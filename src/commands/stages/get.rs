use anyhow::Result;

use crate::cli::stages::StagesGetArgs;
use crate::context::AppContext;
use crate::output;

/// Get a single stage by ID and render as key-value layout.
///
/// No --pipeline needed -- stage ID is unique.
pub async fn run(ctx: &AppContext, args: &StagesGetArgs) -> Result<()> {
    let stage = ctx
        .client
        .get_stage(&args.id, args.expand.as_deref())
        .await?;

    let item = serde_json::to_value(&stage)?;

    let all_columns: Vec<String> = vec![
        "id".to_string(),
        "pipeline_id".to_string(),
        "name".to_string(),
        "description".to_string(),
        "color".to_string(),
        "type".to_string(),
        "position".to_string(),
        "created_at".to_string(),
        "updated_at".to_string(),
    ];

    let columns = all_columns;

    output::render_single(&item, &ctx.output_format, &columns, &args.fields, ctx.color)
}
