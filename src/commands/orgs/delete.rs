use anyhow::Result;

use crate::cache::KEY_ORGS;
use crate::cli::orgs::OrgsDeleteArgs;
use crate::context::AppContext;
use crate::dry_run;

/// Delete an organization by ID.
///
/// With --dry-run, prints what would be deleted without executing.
/// Prints a confirmation message unless --quiet is set.
pub async fn run(ctx: &AppContext, args: &OrgsDeleteArgs) -> Result<()> {
    // Dry-run intercept
    if ctx.dry_run {
        let url = format!("{}/api/v1/organizations/{}", ctx.client.base_url(), args.id);
        return dry_run::render_dry_run_delete(
            "organization",
            &args.id,
            &url,
            &ctx.output_format,
            ctx.color,
        );
    }

    ctx.client.delete_org(&args.id).await?;

    if let Some(ref cache) = ctx.cache {
        cache.invalidate(KEY_ORGS);
    }

    if !ctx.quiet {
        println!("Deleted organization {}", args.id);
    }

    Ok(())
}
