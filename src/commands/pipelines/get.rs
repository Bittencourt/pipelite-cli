use anyhow::Result;

use crate::cli::pipelines::PipelinesGetArgs;
use crate::context::AppContext;
use crate::output;

/// Get a single pipeline by ID and render as key-value layout.
pub async fn run(ctx: &AppContext, args: &PipelinesGetArgs) -> Result<()> {
    let pipeline = ctx
        .client
        .get_pipeline(&args.id, args.expand.as_deref())
        .await?;

    let item = serde_json::to_value(&pipeline)?;

    let all_columns: Vec<String> = vec![
        "id".to_string(),
        "name".to_string(),
        "is_default".to_string(),
        "owner_id".to_string(),
        "created_at".to_string(),
        "updated_at".to_string(),
    ];

    let columns = all_columns;

    output::render_single(&item, &ctx.output_format, &columns, &args.fields, ctx.color)
}
