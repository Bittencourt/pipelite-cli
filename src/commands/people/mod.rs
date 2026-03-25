pub mod create;
pub mod delete;
pub mod get;
pub mod list;
pub mod update;

use anyhow::Result;

use crate::cli::people::PeopleCommands;
use crate::context::AppContext;

/// Dispatch people subcommands to their handlers.
pub async fn run(ctx: &AppContext, cmd: &PeopleCommands) -> Result<()> {
    match cmd {
        PeopleCommands::List(args) => list::run(ctx, args).await,
        PeopleCommands::Get(args) => get::run(ctx, args).await,
        PeopleCommands::Create(args) => create::run(ctx, args).await,
        PeopleCommands::Update(args) => update::run(ctx, args).await,
        PeopleCommands::Delete(args) => delete::run(ctx, args).await,
    }
}
