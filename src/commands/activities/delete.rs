use anyhow::Result;

use crate::cli::activities::ActivitiesDeleteArgs;
use crate::context::AppContext;

/// Delete an activity by ID.
///
/// Prints a confirmation message unless --quiet is set.
pub async fn run(ctx: &AppContext, args: &ActivitiesDeleteArgs) -> Result<()> {
    ctx.client.delete_activity(&args.id).await?;

    if !ctx.quiet {
        println!("Deleted activity {}", args.id);
    }

    Ok(())
}
