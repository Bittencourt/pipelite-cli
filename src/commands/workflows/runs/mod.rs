pub mod detail;
pub mod list;

use anyhow::Result;

use crate::cli::workflows::WorkflowsRunsCommands;
use crate::context::AppContext;

/// Dispatch workflow-runs subcommands to their handlers.
pub async fn run(ctx: &AppContext, cmd: &WorkflowsRunsCommands) -> Result<()> {
    match cmd {
        WorkflowsRunsCommands::List(args) => list::run(ctx, args).await,
        WorkflowsRunsCommands::Get(args) => detail::run(ctx, args).await,
    }
}
