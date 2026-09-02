use anyhow::Result;

use crate::cache::KEY_DEALS;
use crate::cli::deals::DealsDeleteArgs;
use crate::context::AppContext;
use crate::dry_run;
use crate::error::CliError;

/// Delete a deal by ID.
///
/// With --dry-run, prints what would be deleted without executing.
/// Prints a confirmation message unless --quiet is set.
pub async fn run(ctx: &AppContext, args: &DealsDeleteArgs) -> Result<()> {
    let id = args.ids.first().ok_or_else(|| CliError::Validation {
        detail: "Missing deal ID".to_string(),
        hint: "Provide a deal ID, multiple IDs, or use --stdin.".to_string(),
    })?;

    // Dry-run intercept
    if ctx.dry_run {
        let url = format!("{}/api/v1/deals/{}", ctx.client.base_url(), id);
        return dry_run::render_dry_run_delete("deal", id, &url, &ctx.output_format, ctx.color);
    }

    ctx.client.delete_deal(id).await?;

    if let Some(ref cache) = ctx.cache {
        cache.invalidate(KEY_DEALS);
    }

    if !ctx.quiet {
        println!("Deleted deal {}", id);
    }

    Ok(())
}
