use anyhow::Result;

use crate::api::models::custom_fields_table_config;
use crate::cli::custom_fields::CustomFieldsGetArgs;
use crate::context::AppContext;
use crate::output;

/// Get a single custom field definition by ID.
///
/// Soft-deleted definitions resolve here too (the server lookup has no
/// deletedAt filter) — the response carries no marker, so no special
/// casing exists; an absent ID renders the standard 404 NotFound path.
pub async fn run(ctx: &AppContext, args: &CustomFieldsGetArgs) -> Result<()> {
    let definition = ctx
        .client
        .get_custom_field_definition(&args.definition_id)
        .await?;

    let value = serde_json::to_value(&definition)?;

    let config = custom_fields_table_config();
    let columns: Vec<String> = config
        .default_columns
        .iter()
        .map(|s| s.to_string())
        .collect();

    output::render_single(&value, &ctx.output_format, &columns, &args.fields, ctx.color)
}
