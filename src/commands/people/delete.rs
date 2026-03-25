use anyhow::Result;

use crate::cli::people::PeopleDeleteArgs;
use crate::context::AppContext;

/// Delete a person by ID.
///
/// Prints a confirmation message unless --quiet is set.
pub async fn run(ctx: &AppContext, args: &PeopleDeleteArgs) -> Result<()> {
    ctx.client.delete_person(&args.id).await?;

    if !ctx.quiet {
        println!("Deleted person {}", args.id);
    }

    Ok(())
}
