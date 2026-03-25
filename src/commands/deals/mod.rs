pub mod create;
pub mod delete;
pub mod get;
pub mod list;
pub mod update;

use anyhow::Result;

use crate::cli::deals::DealsCommands;
use crate::context::AppContext;

/// Dispatch deal subcommands to their handlers.
pub async fn run(ctx: &AppContext, cmd: &DealsCommands) -> Result<()> {
    match cmd {
        DealsCommands::List(args) => list::run(ctx, args).await,
        DealsCommands::Get(args) => get::run(ctx, args).await,
        DealsCommands::Create(args) => create::run(ctx, args).await,
        DealsCommands::Update(args) => update::run(ctx, args).await,
        DealsCommands::Delete(args) => delete::run(ctx, args).await,
    }
}
