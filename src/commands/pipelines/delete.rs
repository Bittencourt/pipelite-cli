use anyhow::Result;

use crate::cli::pipelines::PipelinesDeleteArgs;
use crate::context::AppContext;

/// Delete a pipeline by ID.
///
/// Prints a confirmation message unless --quiet is set.
pub async fn run(ctx: &AppContext, args: &PipelinesDeleteArgs) -> Result<()> {
    ctx.client.delete_pipeline(&args.id).await?;

    if !ctx.quiet {
        println!("Deleted pipeline {}", args.id);
    }

    Ok(())
}
