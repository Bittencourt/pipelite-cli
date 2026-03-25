use std::io::{self, IsTerminal, Read};

use anyhow::Result;

use crate::api::models::{OrganizationCreate, orgs_table_config};
use crate::cli::orgs::OrgsCreateArgs;
use crate::context::AppContext;
use crate::error::CliError;
use crate::output;

/// Create a new organization (or batch create from stdin).
///
/// When --stdin is set, reads a JSON array of OrganizationCreate objects from stdin.
/// Otherwise, builds a single OrganizationCreate from CLI flags.
pub async fn run(ctx: &AppContext, args: &OrgsCreateArgs) -> Result<()> {
    if args.stdin {
        batch_create(ctx, args).await
    } else {
        single_create(ctx, args).await
    }
}

/// Create a single organization from CLI flags.
async fn single_create(ctx: &AppContext, args: &OrgsCreateArgs) -> Result<()> {
    let name = args.name.as_ref().ok_or_else(|| CliError::Validation {
        detail: "Missing required flag: --name".to_string(),
        hint: "Usage: pipelite orgs create --name <name>".to_string(),
    })?;

    let custom_fields = parse_custom_fields(&args.custom_field)?;

    let data = OrganizationCreate {
        name: name.clone(),
        website: args.website.clone(),
        industry: args.industry.clone(),
        notes: args.notes.clone(),
        custom_fields,
    };

    let org = ctx.client.create_org(&data).await?;
    let item = serde_json::to_value(&org)?;

    let config = orgs_table_config();
    let columns: Vec<String> = config
        .default_columns
        .iter()
        .map(|s| s.to_string())
        .collect();

    output::render_single(&item, &ctx.output_format, &columns, &None, ctx.color)
}

/// Batch create organizations from JSON array on stdin.
async fn batch_create(ctx: &AppContext, _args: &OrgsCreateArgs) -> Result<()> {
    if io::stdin().is_terminal() {
        return Err(CliError::Validation {
            detail: "No data on stdin".to_string(),
            hint: "Pipe JSON data via stdin: echo '[{...}]' | pipelite orgs create --stdin"
                .to_string(),
        }
        .into());
    }

    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let orgs: Vec<OrganizationCreate> =
        serde_json::from_str(&input).map_err(|e| CliError::Validation {
            detail: format!("Invalid JSON input: {}", e),
            hint: "Stdin must contain a JSON array of organization objects with a 'name' field."
                .to_string(),
        })?;

    let created = ctx.client.batch_create_orgs(&orgs).await?;
    let items: Vec<serde_json::Value> = created
        .iter()
        .map(|o| serde_json::to_value(o).map_err(Into::into))
        .collect::<Result<Vec<_>>>()?;

    let config = orgs_table_config();
    let columns: Vec<String> = config
        .default_columns
        .iter()
        .map(|s| s.to_string())
        .collect();

    output::render_list(
        &items,
        &ctx.output_format,
        &columns,
        &None,
        ctx.color,
        None,
    )
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
