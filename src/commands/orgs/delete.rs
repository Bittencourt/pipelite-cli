use anyhow::Result;

use crate::cli::orgs::OrgsDeleteArgs;
use crate::context::AppContext;

/// Delete an organization by ID.
///
/// Prints a confirmation message unless --quiet is set.
pub async fn run(ctx: &AppContext, args: &OrgsDeleteArgs) -> Result<()> {
    ctx.client.delete_org(&args.id).await?;

    if !ctx.quiet {
        println!("Deleted organization {}", args.id);
    }

    Ok(())
}
