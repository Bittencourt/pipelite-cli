use std::io::{self, IsTerminal, Read};

use anyhow::Result;

use crate::api::models::{PersonCreate, people_table_config};
use crate::api::OrgsListParams;
use crate::cli::people::PeopleCreateArgs;
use crate::context::AppContext;
use crate::dry_run;
use crate::error::CliError;
use crate::output;
use crate::prompt;

/// Create a new person (or batch create from stdin).
///
/// When --stdin is set, reads a JSON array of PersonCreate objects from stdin.
/// Otherwise, builds a single PersonCreate from CLI flags (with interactive
/// prompts on TTY when flags are missing).
pub async fn run(ctx: &AppContext, args: &PeopleCreateArgs) -> Result<()> {
    // Validate mutual exclusivity: --stdin vs individual flags
    if args.stdin {
        let has_flags = args.first_name.is_some()
            || args.last_name.is_some()
            || args.email.is_some()
            || args.phone.is_some()
            || args.notes.is_some()
            || args.org.is_some()
            || !args.custom_field.is_empty();

        if has_flags {
            return Err(CliError::Validation {
                detail: "--stdin and individual field flags are mutually exclusive".to_string(),
                hint: "Use either --stdin or individual flags, not both.".to_string(),
            }
            .into());
        }

        return batch_create(ctx).await;
    }

    single_create(ctx, args).await
}

/// Create a single person from CLI flags, with interactive prompts for missing fields.
async fn single_create(ctx: &AppContext, args: &PeopleCreateArgs) -> Result<()> {
    let mut missing = Vec::new();

    // Required: first_name
    let first_name = prompt::require_text(
        &args.first_name,
        "first-name",
        "First name",
        &mut missing,
        ctx.no_input,
    )?;

    // Required: last_name
    let last_name = prompt::require_text(
        &args.last_name,
        "last-name",
        "Last name",
        &mut missing,
        ctx.no_input,
    )?;

    // Check required fields before proceeding to optional ones
    prompt::check_missing(
        &missing,
        "Usage: pipelite people create --first-name <name> --last-name <name>",
    )?;

    let first_name = first_name.unwrap();
    let last_name = last_name.unwrap();

    // Optional fields
    let email = prompt::optional_text(&args.email, "Email", ctx.no_input)?;
    let phone = prompt::optional_text(&args.phone, "Phone", ctx.no_input)?;

    // Optional: org (FuzzySelect when on TTY)
    let org_id = if args.org.is_some() {
        args.org.clone()
    } else if std::io::stdin().is_terminal() && !ctx.no_input {
        select_org_interactive(ctx).await?
    } else {
        None
    };

    let notes = prompt::optional_text(&args.notes, "Notes", ctx.no_input)?;

    let custom_fields = parse_custom_fields(&args.custom_field)?;

    let data = PersonCreate {
        first_name,
        last_name,
        email,
        phone,
        notes,
        organization_id: org_id,
        custom_fields,
    };

    // Dry-run intercept
    if ctx.dry_run {
        let body = serde_json::to_value(&data)?;
        let url = format!("{}/api/v1/people", ctx.client.base_url());
        return dry_run::render_dry_run("POST", &url, &body, &ctx.output_format, ctx.color);
    }

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

/// Interactive org selection via FuzzySelect with a "None/Skip" option at top.
async fn select_org_interactive(ctx: &AppContext) -> Result<Option<String>> {
    let orgs_resp = ctx
        .client
        .list_orgs(&OrgsListParams {
            owner: None,
            limit: 100,
            offset: 0,
            expand: None,
        })
        .await?;

    if orgs_resp.data.is_empty() {
        return Ok(None);
    }

    let mut display: Vec<String> = vec!["(none - skip)".to_string()];
    let mut ids: Vec<Option<String>> = vec![None];

    for org in &orgs_resp.data {
        display.push(format!("{} ({})", org.name, org.id));
        ids.push(Some(org.id.clone()));
    }

    let selection = dialoguer::FuzzySelect::new()
        .with_prompt("Select organization (optional)")
        .items(&display)
        .default(0)
        .interact()?;

    Ok(ids[selection].clone())
}

/// Batch create people from JSON array on stdin.
async fn batch_create(ctx: &AppContext) -> Result<()> {
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

    // Dry-run: show the batch payload
    if ctx.dry_run {
        let url = format!("{}/api/v1/people/batch", ctx.client.base_url());
        let body = serde_json::to_value(&people)?;
        return dry_run::render_dry_run("POST", &url, &body, &ctx.output_format, ctx.color);
    }

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
