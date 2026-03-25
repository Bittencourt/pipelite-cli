use std::io::{self, IsTerminal, Read};

use anyhow::Result;

use crate::api::models::{DealCreate, deals_table_config};
use crate::cli::deals::DealsCreateArgs;
use crate::context::AppContext;
use crate::error::CliError;
use crate::output;

/// Create a new deal (or batch create from stdin).
///
/// When --stdin is set, reads a JSON array of DealCreate objects from stdin.
/// Otherwise, builds a single DealCreate from CLI flags.
pub async fn run(ctx: &AppContext, args: &DealsCreateArgs) -> Result<()> {
    if args.stdin {
        batch_create(ctx, args).await
    } else {
        single_create(ctx, args).await
    }
}

/// Create a single deal from CLI flags.
async fn single_create(ctx: &AppContext, args: &DealsCreateArgs) -> Result<()> {
    let title = args.title.as_ref().ok_or_else(|| CliError::Validation {
        detail: "Missing required flag: --title".to_string(),
        hint: "Usage: pipelite deals create --title <title> --stage <stage_id>".to_string(),
    })?;

    let stage_id = args.stage.as_ref().ok_or_else(|| CliError::Validation {
        detail: "Missing required flag: --stage".to_string(),
        hint: "Usage: pipelite deals create --title <title> --stage <stage_id>".to_string(),
    })?;

    let custom_fields = parse_custom_fields(&args.custom_field)?;

    let data = DealCreate {
        title: title.clone(),
        stage_id: stage_id.clone(),
        value: args.value,
        organization_id: args.org.clone(),
        person_id: args.person.clone(),
        expected_close_date: args.expected_close_date.clone(),
        notes: args.notes.clone(),
        custom_fields,
    };

    let deal = ctx.client.create_deal(&data).await?;
    let item = serde_json::to_value(&deal)?;

    let config = deals_table_config();
    let columns: Vec<String> = config
        .default_columns
        .iter()
        .map(|s| s.to_string())
        .collect();

    output::render_single(&item, &ctx.output_format, &columns, &None, ctx.color)
}

/// Batch create deals from JSON array on stdin.
async fn batch_create(ctx: &AppContext, _args: &DealsCreateArgs) -> Result<()> {
    if io::stdin().is_terminal() {
        return Err(CliError::Validation {
            detail: "No data on stdin".to_string(),
            hint: "Pipe JSON data via stdin: echo '[{...}]' | pipelite deals create --stdin"
                .to_string(),
        }
        .into());
    }

    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let deals: Vec<DealCreate> = serde_json::from_str(&input).map_err(|e| CliError::Validation {
        detail: format!("Invalid JSON input: {}", e),
        hint: "Stdin must contain a JSON array of deal objects with 'title' and 'stage_id' fields."
            .to_string(),
    })?;

    let created = ctx.client.batch_create_deals(&deals).await?;
    let items: Vec<serde_json::Value> = created
        .iter()
        .map(|d| serde_json::to_value(d).map_err(Into::into))
        .collect::<Result<Vec<_>>>()?;

    let config = deals_table_config();
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
