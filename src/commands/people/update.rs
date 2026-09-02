use std::io::IsTerminal;

use anyhow::Result;

use crate::api::models::{PersonUpdate, people_table_config};
use crate::batch;
use crate::cache::KEY_PEOPLE;
use crate::cli::people::PeopleUpdateArgs;
use crate::context::AppContext;
use crate::dry_run;
use crate::error::CliError;
use crate::output;
use crate::prompt;

/// Update an existing person.
///
/// With --stdin, reads a JSON array of objects (each with an "id" field plus
/// update fields) from stdin and batch-updates with continue-on-error.
/// If no flags are provided on a TTY, prompts interactively for all fields
/// (all optional -- user can skip any). In headless mode with no flags,
/// returns a validation error.
pub async fn run(ctx: &AppContext, args: &PeopleUpdateArgs) -> Result<()> {
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

        return batch_update(ctx).await;
    }

    // Extract id from Option — CLAUDE.md forbids unwrap() in production code.
    // Use CliError::Validation with an actionable hint instead.
    let id = args.id.as_deref().ok_or_else(|| CliError::Validation {
        detail: "Missing person ID".to_string(),
        hint: "Provide a person ID or use --stdin.".to_string(),
    })?;

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

    let custom_fields = batch::parse_custom_fields(&args.custom_field)?;

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
        let url = format!("{}/api/v1/people/{}", ctx.client.base_url(), id);
        return dry_run::render_dry_run("PUT", &url, &body, &ctx.output_format, ctx.color);
    }

    let person = ctx.client.update_person(id, &data).await?;

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

/// Batch update people from JSON array on stdin (per D-02, D-03).
///
/// Each JSON object must contain an "id" field plus update fields.
/// Processes all items with continue-on-error semantics (per D-06).
/// The shared flow lives in [`crate::batch::run_batch_update`].
async fn batch_update(ctx: &AppContext) -> Result<()> {
    batch::run_batch_update::<PersonUpdate, _>(
        ctx,
        "person",
        "people",
        r#"[{"id":"per_1","first_name":"Jane"}]"#,
        "people",
        KEY_PEOPLE,
        None,
        &people_table_config().default_columns,
        async |id: String, data: PersonUpdate, _raw: &serde_json::Value| {
            batch::ensure_update_fields(&data, "people")?;
            let person = ctx.client.update_person(&id, &data).await?;
            Ok(serde_json::to_value(person)?)
        },
    )
    .await
}
