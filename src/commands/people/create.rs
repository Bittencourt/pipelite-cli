use std::io::{self, IsTerminal, Read};

use anyhow::Result;

use crate::api::models::{PersonCreate, people_table_config};
use crate::cli::people::PeopleCreateArgs;
use crate::context::AppContext;
use crate::error::CliError;
use crate::output;

/// Create a new person (or batch create from stdin).
///
/// When --stdin is set, reads a JSON array of PersonCreate objects from stdin.
/// Otherwise, builds a single PersonCreate from CLI flags.
pub async fn run(ctx: &AppContext, args: &PeopleCreateArgs) -> Result<()> {
    if args.stdin {
        batch_create(ctx, args).await
    } else {
        single_create(ctx, args).await
    }
}

/// Create a single person from CLI flags.
async fn single_create(ctx: &AppContext, args: &PeopleCreateArgs) -> Result<()> {
    let first_name = args
        .first_name
        .as_ref()
        .ok_or_else(|| CliError::Validation {
            detail: "Missing required flag: --first-name".to_string(),
            hint: "Usage: pipelite people create --first-name <name> --last-name <name>"
                .to_string(),
        })?;

    let last_name = args
        .last_name
        .as_ref()
        .ok_or_else(|| CliError::Validation {
            detail: "Missing required flag: --last-name".to_string(),
            hint: "Usage: pipelite people create --first-name <name> --last-name <name>"
                .to_string(),
        })?;

    let custom_fields = parse_custom_fields(&args.custom_field)?;

    let data = PersonCreate {
        first_name: first_name.clone(),
        last_name: last_name.clone(),
        email: args.email.clone(),
        phone: args.phone.clone(),
        notes: args.notes.clone(),
        organization_id: args.org.clone(),
        custom_fields,
    };

    let person = ctx.client.create_person(&data).await?;
    let item = serde_json::to_value(&person)?;

    let config = people_table_config();
    let columns: Vec<String> = config
        .default_columns
        .iter()
        .map(|s| s.to_string())
        .collect();

    output::render_single(&item, &ctx.output_format, &columns, &None, ctx.color)
}

/// Batch create people from JSON array on stdin.
async fn batch_create(ctx: &AppContext, _args: &PeopleCreateArgs) -> Result<()> {
    if io::stdin().is_terminal() {
        return Err(CliError::Validation {
            detail: "No data on stdin".to_string(),
            hint: "Pipe JSON data via stdin: echo '[{...}]' | pipelite people create --stdin"
                .to_string(),
        }
        .into());
    }

    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let people: Vec<PersonCreate> =
        serde_json::from_str(&input).map_err(|e| CliError::Validation {
            detail: format!("Invalid JSON input: {}", e),
            hint: "Stdin must contain a JSON array of person objects with 'first_name' and 'last_name' fields."
                .to_string(),
        })?;

    let created = ctx.client.batch_create_people(&people).await?;
    let items: Vec<serde_json::Value> = created
        .iter()
        .map(|p| serde_json::to_value(p).map_err(Into::into))
        .collect::<Result<Vec<_>>>()?;

    let config = people_table_config();
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
            hint: "Use key=value format: --custom-field role=CTO".to_string(),
        })?;
        map.insert(key.to_string(), serde_json::Value::String(value.to_string()));
    }

    Ok(Some(serde_json::Value::Object(map)))
}
