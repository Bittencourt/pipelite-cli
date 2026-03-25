pub mod create;
pub mod delete;
pub mod get;
pub mod list;
pub mod update;

use anyhow::Result;

use crate::cli::pipelines::PipelinesCommands;
use crate::context::AppContext;

/// Dispatch pipeline subcommands to their handlers.
pub async fn run(ctx: &AppContext, cmd: &PipelinesCommands) -> Result<()> {
    match cmd {
        PipelinesCommands::List(args) => list::run(ctx, args).await,
        PipelinesCommands::Get(args) => get::run(ctx, args).await,
        PipelinesCommands::Create(args) => create::run(ctx, args).await,
        PipelinesCommands::Update(args) => update::run(ctx, args).await,
        PipelinesCommands::Delete(args) => delete::run(ctx, args).await,
    }
}
