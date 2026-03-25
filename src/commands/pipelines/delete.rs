use anyhow::Result;

use crate::cli::pipelines::PipelinesDeleteArgs;
use crate::context::AppContext;
use crate::dry_run;

/// Delete a pipeline by ID.
///
/// With --dry-run, prints what would be deleted without executing.
/// Prints a confirmation message unless --quiet is set.
pub async fn run(ctx: &AppContext, args: &PipelinesDeleteArgs) -> Result<()> {
    // Dry-run intercept
    if ctx.dry_run {
        let url = format!("{}/api/v1/pipelines/{}", ctx.client.base_url(), args.id);
        return dry_run::render_dry_run_delete(
            "pipeline",
            &args.id,
            &url,
            &ctx.output_format,
            ctx.color,
        );
    }

    ctx.client.delete_pipeline(&args.id).await?;

    if !ctx.quiet {
        println!("Deleted pipeline {}", args.id);
    }

    Ok(())
}
