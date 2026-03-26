pub mod clear;
pub mod refresh;

use anyhow::Result;

use crate::cli::cache::CacheCommands;
use crate::context::AppContext;

/// Dispatch cache subcommands.
pub async fn run(ctx: &AppContext, cmd: &CacheCommands) -> Result<()> {
    match cmd {
        CacheCommands::Clear => clear::run(ctx),
        CacheCommands::Refresh => refresh::run(ctx).await,
    }
}
