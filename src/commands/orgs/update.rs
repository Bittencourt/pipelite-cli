use std::io::IsTerminal;

use anyhow::Result;

use crate::api::models::{OrganizationUpdate, orgs_table_config};
use crate::batch;
use crate::cache::KEY_ORGS;
use crate::cli::orgs::OrgsUpdateArgs;
use crate::context::AppContext;
use crate::dry_run;
use crate::error::CliError;
use crate::output;
use crate::prompt;

/// Update an existing organization.
///
/// With --stdin, reads a JSON array of objects (each with an "id" field plus
/// update fields) from stdin and batch-updates with continue-on-error.
/// If no flags are provided on a TTY, prompts interactively for all fields
/// (all optional -- user can skip any). In headless mode with no flags,
/// returns a validation error.
pub async fn run(ctx: &AppContext, args: &OrgsUpdateArgs) -> Result<()> {
    if args.stdin {
        let has_flags = args.name.is_some()
            || args.website.is_some()
            || args.industry.is_some()
            || args.notes.is_some()
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
        detail: "Missing organization ID".to_string(),
        hint: "Provide an organization ID or use --stdin.".to_string(),
    })?;

    let has_flags = args.name.is_some()
        || args.website.is_some()
        || args.industry.is_some()
        || args.notes.is_some()
        || !args.custom_field.is_empty();

    // If no flags provided in headless mode, error out
    if !has_flags && (ctx.no_input || !std::io::stdin().is_terminal()) {
        return Err(CliError::Validation {
            detail: "No fields to update. Provide at least one flag.".to_string(),
            hint: "Usage: pipelite orgs update <id> --name <name> [--website <url>] [--industry <industry>]".to_string(),
        }
        .into());
    }

    // Collect field values (from flags or interactive prompts)
    let name = if has_flags {
        args.name.clone()
    } else {
        prompt::optional_text(&args.name, "Name", ctx.no_input)?
    };

    let website = if has_flags {
        args.website.clone()
    } else {
        prompt::optional_text(&args.website, "Website URL", ctx.no_input)?
    };

    let industry = if has_flags {
        args.industry.clone()
    } else {
        prompt::optional_text(&args.industry, "Industry", ctx.no_input)?
    };

    let notes = if has_flags {
        args.notes.clone()
    } else {
        prompt::optional_text(&args.notes, "Notes", ctx.no_input)?
    };

    let custom_fields = batch::parse_custom_fields(&args.custom_field)?;

    let data = OrganizationUpdate {
        name,
        website,
        industry,
        notes,
        custom_fields,
    };

    // Dry-run intercept
    if ctx.dry_run {
        let body = serde_json::to_value(&data)?;
        let url = format!("{}/api/v1/organizations/{}", ctx.client.base_url(), id);
        return dry_run::render_dry_run("PUT", &url, &body, &ctx.output_format, ctx.color);
    }

    let org = ctx.client.update_org(id, &data).await?;

    if let Some(ref cache) = ctx.cache {
        cache.invalidate(KEY_ORGS);
    }

    let item = serde_json::to_value(&org)?;

    let config = orgs_table_config();
    let columns: Vec<String> = config
        .default_columns
        .iter()
        .map(|s| s.to_string())
        .collect();

    output::render_single(&item, &ctx.output_format, &columns, &None, ctx.color)
}

/// Batch update organizations from JSON array on stdin (per D-02, D-03).
///
/// Each JSON object must contain an "id" field plus update fields.
/// Processes all items with continue-on-error semantics (per D-06).
/// The shared flow lives in [`crate::batch::run_batch_update`].
async fn batch_update(ctx: &AppContext) -> Result<()> {
    batch::run_batch_update::<OrganizationUpdate, _>(
        ctx,
        "organization",
        "orgs",
        r#"[{"id":"org_1","name":"New"}]"#,
        "organizations",
        KEY_ORGS,
        None,
        &orgs_table_config().default_columns,
        async |id: String, data: OrganizationUpdate, _raw: &serde_json::Value| {
            batch::ensure_update_fields(&data, "orgs")?;
            let org = ctx.client.update_org(&id, &data).await?;
            Ok(serde_json::to_value(org)?)
        },
    )
    .await
}
