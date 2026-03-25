use anyhow::Result;

use crate::cli::orgs::OrgsGetArgs;
use crate::context::AppContext;
use crate::output;

/// Get a single organization by ID and render as key-value layout.
pub async fn run(ctx: &AppContext, args: &OrgsGetArgs) -> Result<()> {
    let org = ctx
        .client
        .get_org(&args.id, args.expand.as_deref())
        .await?;

    let item = serde_json::to_value(&org)?;

    let all_columns: Vec<String> = vec![
        "id".to_string(),
        "name".to_string(),
        "website".to_string(),
        "industry".to_string(),
        "notes".to_string(),
        "owner_id".to_string(),
        "custom_fields".to_string(),
        "created_at".to_string(),
        "updated_at".to_string(),
    ];

    output::render_single(&item, &ctx.output_format, &all_columns, &args.fields, ctx.color)
}
