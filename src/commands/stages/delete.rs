use anyhow::Result;

use crate::cli::stages::StagesDeleteArgs;
use crate::context::AppContext;

/// Delete a stage by ID.
///
/// No --pipeline needed -- stage ID is unique.
/// Prints a confirmation message unless --quiet is set.
pub async fn run(ctx: &AppContext, args: &StagesDeleteArgs) -> Result<()> {
    ctx.client.delete_stage(&args.id).await?;

    if !ctx.quiet {
        println!("Deleted stage {}", args.id);
    }

    Ok(())
}
