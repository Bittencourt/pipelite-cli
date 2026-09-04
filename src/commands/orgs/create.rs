use std::io::{self, IsTerminal, Read};

use anyhow::Result;

use crate::api::models::{OrganizationCreate, orgs_table_config};
use crate::cache::KEY_ORGS;
use crate::cli::orgs::OrgsCreateArgs;
use crate::context::AppContext;
use crate::custom_fields;
use crate::dry_run;
use crate::error::CliError;
use crate::output;
use crate::prompt;

/// Create a new organization (or batch create from stdin).
///
/// When --stdin is set, reads a JSON array of OrganizationCreate objects from stdin.
/// Otherwise, builds a single OrganizationCreate from CLI flags (with interactive
/// prompts on TTY when flags are missing).
pub async fn run(ctx: &AppContext, args: &OrgsCreateArgs) -> Result<()> {
    // Validate mutual exclusivity: --stdin vs individual flags
    if args.stdin {
        let has_flags = args.name.is_some()
            || args.website.is_some()
            || args.industry.is_some()
            || args.notes.is_some()
            || !args.custom_field.is_empty()
            || args.custom_field_json.is_some();

        if has_flags {
            return Err(CliError::InvalidInput {
                detail: "--stdin and individual field flags are mutually exclusive".to_string(),
                hint: "Use either --stdin or individual flags, not both.".to_string(),
            }
            .into());
        }

        return batch_create(ctx).await;
    }

    single_create(ctx, args).await
}

/// Create a single organization from CLI flags, with interactive prompts for missing fields.
async fn single_create(ctx: &AppContext, args: &OrgsCreateArgs) -> Result<()> {
    let mut missing = Vec::new();

    // Required: name
    let name = prompt::require_text(
        &args.name,
        "name",
        "Organization name",
        &mut missing,
        ctx.no_input,
    )?;

    // Check required fields before proceeding to optional ones
    prompt::check_missing(
        &missing,
        "Usage: pipelite orgs create --name <name>",
    )?;

    let name = name.unwrap();

    // Optional fields
    let website = prompt::optional_text(&args.website, "Website URL", ctx.no_input)?;
    let industry = prompt::optional_text(&args.industry, "Industry", ctx.no_input)?;
    let notes = prompt::optional_text(&args.notes, "Notes", ctx.no_input)?;

    let custom_fields = custom_fields::resolve_custom_fields(
        ctx,
        custom_fields::CfEntityType::Organization,
        &args.custom_field,
        ctx.dry_run,
        args.custom_field_json.as_deref(),
    )
    .await?;

    let data = OrganizationCreate {
        name,
        website,
        industry,
        notes,
        custom_fields,
    };

    // Dry-run intercept
    if ctx.dry_run {
        let body = serde_json::to_value(&data)?;
        let url = format!("{}/api/v1/organizations", ctx.client.base_url());
        return dry_run::render_dry_run("POST", &url, &body, &ctx.output_format, ctx.color);
    }

    let org = ctx.client.create_org(&data).await?;

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

/// Batch create organizations from JSON array on stdin.
async fn batch_create(ctx: &AppContext) -> Result<()> {
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

    // Dry-run: show the batch payload
    if ctx.dry_run {
        let url = format!("{}/api/v1/organizations/batch", ctx.client.base_url());
        let body = serde_json::to_value(&orgs)?;
        return dry_run::render_dry_run("POST", &url, &body, &ctx.output_format, ctx.color);
    }

    let created = ctx.client.batch_create_orgs(&orgs).await?;

    if let Some(ref cache) = ctx.cache {
        cache.invalidate(KEY_ORGS);
    }

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
