use anyhow::Result;

use crate::cli::people::PeopleGetArgs;
use crate::context::AppContext;
use crate::output;

/// Get a single person by ID and render as key-value layout.
pub async fn run(ctx: &AppContext, args: &PeopleGetArgs) -> Result<()> {
    let person = ctx
        .client
        .get_person(&args.id, args.expand.as_deref())
        .await?;

    let item = serde_json::to_value(&person)?;

    let all_columns: Vec<String> = vec![
        "id".to_string(),
        "first_name".to_string(),
        "last_name".to_string(),
        "full_name".to_string(),
        "email".to_string(),
        "phone".to_string(),
        "notes".to_string(),
        "organization_id".to_string(),
        "owner_id".to_string(),
        "custom_fields".to_string(),
        "created_at".to_string(),
        "updated_at".to_string(),
    ];

    output::render_single(&item, &ctx.output_format, &all_columns, &args.fields, ctx.color)
}
