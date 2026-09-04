pub mod list;

use anyhow::Result;

use crate::cli::audit::AuditCommands;
use crate::context::AppContext;

/// Dispatch audit subcommands to their handlers.
///
/// No validation logic lives here — the audit surface is pure passthrough:
/// the server owns filter validation (invalid enum values 422 and flow
/// through the Phase 8 error layer untouched), and non-admin 403s render
/// the pre-registered "audit" hint from the client layer — the server gates
/// BEFORE query validation, so the hint is independent of filter validity.
pub async fn run(ctx: &AppContext, cmd: &AuditCommands) -> Result<()> {
    match cmd {
        AuditCommands::List(args) => list::run(ctx, args).await,
    }
}
