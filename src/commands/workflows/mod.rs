pub mod create;
pub mod delete;
pub mod get;
pub mod list;
pub mod runs;
pub mod trigger;
pub mod update;

use anyhow::Result;

use crate::cli::workflows::WorkflowsCommands;
use crate::context::AppContext;

/// Dispatch workflow subcommands to their handlers.
pub async fn run(ctx: &AppContext, cmd: &WorkflowsCommands) -> Result<()> {
    match cmd {
        WorkflowsCommands::List(args) => list::run(ctx, args).await,
        WorkflowsCommands::Get(args) => get::run(ctx, args).await,
        WorkflowsCommands::Create(args) => create::run(ctx, args).await,
        WorkflowsCommands::Update(args) => update::run(ctx, args).await,
        WorkflowsCommands::Delete(args) => delete::run(ctx, args).await,
        WorkflowsCommands::Trigger(args) => trigger::run(ctx, args).await,
        WorkflowsCommands::Runs(args) => runs::run(ctx, &args.command).await,
    }
}
