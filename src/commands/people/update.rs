use std::io::IsTerminal;

use anyhow::Result;

use crate::api::models::{PersonUpdate, people_table_config};
use crate::cache::KEY_PEOPLE;
use crate::cli::people::PeopleUpdateArgs;
use crate::context::AppContext;
use crate::dry_run;
use crate::error::CliError;
use crate::output;
use crate::prompt;

/// Update an existing person.
///
/// If no flags are provided on a TTY, prompts interactively for all fields
/// (all optional -- user can skip any). In headless mode with no flags,
/// returns a validation error.
pub async fn run(ctx: &AppContext, args: &PeopleUpdateArgs) -> Result<()> {
    let has_flags = args.first_name.is_some()
        || args.last_name.is_some()
        || args.email.is_some()
        || args.phone.is_some()
        || args.notes.is_some()
        || args.org.is_some()
        || !args.custom_field.is_empty();

    // If no flags provided in headless mode, error out
    if !has_flags && (ctx.no_input || !std::io::stdin().is_terminal()) {
        return Err(CliError::Validation {
            detail: "No fields to update. Provide at least one flag.".to_string(),
            hint: "Usage: pipelite people update <id> --first-name <name> [--last-name <name>] [--email <email>]".to_string(),
        }
        .into());
    }

    // Collect field values (from flags or interactive prompts)
    let first_name = if has_flags {
        args.first_name.clone()
    } else {
        prompt::optional_text(&args.first_name, "First name", ctx.no_input)?
    };

    let last_name = if has_flags {
        args.last_name.clone()
    } else {
        prompt::optional_text(&args.last_name, "Last name", ctx.no_input)?
    };

    let email = if has_flags {
        args.email.clone()
    } else {
        prompt::optional_text(&args.email, "Email", ctx.no_input)?
    };

    let phone = if has_flags {
        args.phone.clone()
    } else {
        prompt::optional_text(&args.phone, "Phone", ctx.no_input)?
    };

    let notes = if has_flags {
        args.notes.clone()
    } else {
        prompt::optional_text(&args.notes, "Notes", ctx.no_input)?
    };

    let org = if has_flags {
        args.org.clone()
    } else {
        prompt::optional_text(&args.org, "Organization ID", ctx.no_input)?
    };

    let custom_fields = parse_custom_fields(&args.custom_field)?;

    let data = PersonUpdate {
        first_name,
        last_name,
        email,
        phone,
        notes,
        organization_id: org,
        custom_fields,
    };

    // Dry-run intercept
    if ctx.dry_run {
        let body = serde_json::to_value(&data)?;
        let url = format!("{}/api/v1/people/{}", ctx.client.base_url(), args.id);
        return dry_run::render_dry_run("PUT", &url, &body, &ctx.output_format, ctx.color);
    }

    let person = ctx.client.update_person(&args.id, &data).await?;

    if let Some(ref cache) = ctx.cache {
        cache.invalidate(KEY_PEOPLE);
    }

    let item = serde_json::to_value(&person)?;

    let config = people_table_config();
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
            hint: "Use key=value format: --custom-field role=CTO".to_string(),
        })?;
        map.insert(key.to_string(), serde_json::Value::String(value.to_string()));
    }

    Ok(Some(serde_json::Value::Object(map)))
}
