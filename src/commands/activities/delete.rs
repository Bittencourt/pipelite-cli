use anyhow::Result;

use crate::cli::activities::ActivitiesDeleteArgs;
use crate::context::AppContext;
use crate::dry_run;

/// Delete an activity by ID.
///
/// With --dry-run, prints what would be deleted without executing.
/// Prints a confirmation message unless --quiet is set.
pub async fn run(ctx: &AppContext, args: &ActivitiesDeleteArgs) -> Result<()> {
    // Dry-run intercept
    if ctx.dry_run {
        let url = format!("{}/api/v1/activities/{}", ctx.client.base_url(), args.id);
        return dry_run::render_dry_run_delete(
            "activity",
            &args.id,
            &url,
            &ctx.output_format,
            ctx.color,
        );
    }

    ctx.client.delete_activity(&args.id).await?;

    if !ctx.quiet {
        println!("Deleted activity {}", args.id);
    }

    Ok(())
}
