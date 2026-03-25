pub mod create;
pub mod delete;
pub mod get;
pub mod list;
pub mod update;

use anyhow::Result;

use crate::cli::activities::ActivitiesCommands;
use crate::context::AppContext;

/// Dispatch activity subcommands to their handlers.
pub async fn run(ctx: &AppContext, cmd: &ActivitiesCommands) -> Result<()> {
    match cmd {
        ActivitiesCommands::List(args) => list::run(ctx, args).await,
        ActivitiesCommands::Get(args) => get::run(ctx, args).await,
        ActivitiesCommands::Create(args) => create::run(ctx, args).await,
        ActivitiesCommands::Update(args) => update::run(ctx, args).await,
        ActivitiesCommands::Delete(args) => delete::run(ctx, args).await,
    }
}
