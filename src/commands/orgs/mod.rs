pub mod create;
pub mod delete;
pub mod get;
pub mod list;
pub mod update;

use anyhow::Result;

use crate::cli::orgs::OrgsCommands;
use crate::context::AppContext;

/// Dispatch organization subcommands to their handlers.
pub async fn run(ctx: &AppContext, cmd: &OrgsCommands) -> Result<()> {
    match cmd {
        OrgsCommands::List(args) => list::run(ctx, args).await,
        OrgsCommands::Get(args) => get::run(ctx, args).await,
        OrgsCommands::Create(args) => create::run(ctx, args).await,
        OrgsCommands::Update(args) => update::run(ctx, args).await,
        OrgsCommands::Delete(args) => delete::run(ctx, args).await,
    }
}
