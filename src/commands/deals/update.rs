use anyhow::Result;

use crate::api::models::{DealUpdate, deals_table_config};
use crate::cli::deals::DealsUpdateArgs;
use crate::context::AppContext;
use crate::error::CliError;
use crate::output;

/// Update an existing deal.
///
/// Builds a DealUpdate from optional CLI flags and sends to the API.
/// Renders the updated deal on success.
pub async fn run(ctx: &AppContext, args: &DealsUpdateArgs) -> Result<()> {
    let custom_fields = parse_custom_fields(&args.custom_field)?;

    let data = DealUpdate {
        title: args.title.clone(),
        stage_id: args.stage.clone(),
        value: args.value,
        organization_id: args.org.clone(),
        person_id: args.person.clone(),
        expected_close_date: args.expected_close_date.clone(),
        notes: args.notes.clone(),
        custom_fields,
    };

    let deal = ctx.client.update_deal(&args.id, &data).await?;
    let item = serde_json::to_value(&deal)?;

    let config = deals_table_config();
    let columns: Vec<String> = config
        .default_columns
        .iter()
        .map(|s| s.to_string())
        .collect();

    output::render_single(&item, &ctx.output_format, &columns, &None, ctx.color)
}

/// Parse --custom-field key=value pairs into a serde_json::Value object.
fn parse_custom_fields(pairs: &[String]) -> Result<Option<serde_json::Value>> {
    if pairs.is_empty() {
        return Ok(None);
    }

    let mut map = serde_json::Map::new();
    for pair in pairs {
        let (key, value) = pair.split_once('=').ok_or_else(|| CliError::Validation {
            detail: format!("Invalid custom field format: '{}'", pair),
            hint: "Use key=value format: --custom-field industry=Tech".to_string(),
        })?;
        map.insert(key.to_string(), serde_json::Value::String(value.to_string()));
    }

    Ok(Some(serde_json::Value::Object(map)))
}
