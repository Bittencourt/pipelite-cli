pub mod create;
pub mod delete;
pub mod get;
pub mod list;

use anyhow::Result;

use crate::cli::templates::TemplatesCommands;
use crate::context::AppContext;
use crate::error::CliError;

/// Dispatch template subcommands to their handlers.
///
/// The hidden Update variant is the parse-then-error pattern (Phase 8): it
/// exists only so `pipelite templates update ...` parses and can be rejected
/// with the locked hint BEFORE any HTTP — the server exposes no update route.
pub async fn run(ctx: &AppContext, cmd: &TemplatesCommands) -> Result<()> {
    match cmd {
        TemplatesCommands::List(args) => list::run(ctx, args).await,
        TemplatesCommands::Get(args) => get::run(ctx, args).await,
        TemplatesCommands::Create(args) => create::run(ctx, args).await,
        TemplatesCommands::Delete(args) => delete::run(ctx, args).await,
        TemplatesCommands::Update(_) => Err(CliError::InvalidInput {
            detail: "The server exposes no template update".to_string(),
            hint: "The server exposes no template update — delete and recreate to change a template"
                .to_string(),
        }
        .into()),
    }
}
