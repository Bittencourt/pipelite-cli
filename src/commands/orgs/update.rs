use anyhow::Result;

use crate::api::models::{OrganizationUpdate, orgs_table_config};
use crate::cli::orgs::OrgsUpdateArgs;
use crate::context::AppContext;
use crate::error::CliError;
use crate::output;

/// Update an existing organization.
///
/// Builds an OrganizationUpdate from optional CLI flags and sends to the API.
/// Renders the updated organization on success.
pub async fn run(ctx: &AppContext, args: &OrgsUpdateArgs) -> Result<()> {
    let custom_fields = parse_custom_fields(&args.custom_field)?;

    let data = OrganizationUpdate {
        name: args.name.clone(),
        website: args.website.clone(),
        industry: args.industry.clone(),
        notes: args.notes.clone(),
        custom_fields,
    };

    let org = ctx.client.update_org(&args.id, &data).await?;
    let item = serde_json::to_value(&org)?;

    let config = orgs_table_config();
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
