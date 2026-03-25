pub mod create;
pub mod delete;
pub mod get;
pub mod list;
pub mod update;

use anyhow::Result;

use crate::cli::stages::StagesCommands;
use crate::context::AppContext;

/// Dispatch stage subcommands to their handlers.
pub async fn run(ctx: &AppContext, cmd: &StagesCommands) -> Result<()> {
    match cmd {
        StagesCommands::List(args) => list::run(ctx, args).await,
        StagesCommands::Get(args) => get::run(ctx, args).await,
        StagesCommands::Create(args) => create::run(ctx, args).await,
        StagesCommands::Update(args) => update::run(ctx, args).await,
        StagesCommands::Delete(args) => delete::run(ctx, args).await,
    }
}
