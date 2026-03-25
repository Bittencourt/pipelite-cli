use anyhow::Result;

use crate::cli::deals::DealsDeleteArgs;
use crate::context::AppContext;

/// Delete a deal by ID.
///
/// Prints a confirmation message unless --quiet is set.
pub async fn run(ctx: &AppContext, args: &DealsDeleteArgs) -> Result<()> {
    ctx.client.delete_deal(&args.id).await?;

    if !ctx.quiet {
        println!("Deleted deal {}", args.id);
    }

    Ok(())
}
