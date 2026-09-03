pub mod list;

use anyhow::Result;

use crate::cli::notes::NotesCommands;
use crate::context::AppContext;
use crate::error::CliError;

/// Dispatch note subcommands to their handlers.
///
/// The hidden Get variant is the parse-then-error pattern (Phase 8): it
/// exists only so `pipelite notes get ...` parses and can be rejected with
/// the locked hint BEFORE any HTTP — the server exposes no single-note GET
/// (the item route offers PATCH/DELETE only).
pub async fn run(ctx: &AppContext, cmd: &NotesCommands) -> Result<()> {
    match cmd {
        NotesCommands::List(args) => list::run(ctx, args).await,
        NotesCommands::Get(_) => Err(CliError::InvalidInput {
            detail: "The server exposes no single-note GET".to_string(),
            hint: "the server has no single-note GET — use `notes list <type> <id> --json`"
                .to_string(),
        }
        .into()),
    }
}

/// Map a CLI entity-type positional to its REST route segment.
///
/// This is the ONLY gate for the positional — deliberately NOT a clap
/// ValueEnum, so the locked "notes are only available on…" rejection (exit 2,
/// zero HTTP) fires instead of clap's generic invalid-value message
/// (Pitfall 6). Case-sensitive by design.
///
/// CLI plural name → collection route segment:
/// deals → deals | orgs → organizations | people → people | activities →
/// activities. Note response payloads keep the SINGULAR discriminator
/// ("deal" etc.) — this mapping never appears in a response.
pub fn resolve_entity_type(entity_type: &str) -> Result<&'static str> {
    match entity_type {
        "deals" => Ok("deals"),
        "orgs" => Ok("organizations"),
        "people" => Ok("people"),
        "activities" => Ok("activities"),
        other => Err(CliError::InvalidInput {
            detail: format!(
                "Notes are not available on '{other}' — notes are only available on deals, orgs, people, activities"
            ),
            hint: "Valid entity types: deals, orgs, people, activities (e.g. pipelite notes list deals <deal-id>)"
                .to_string(),
        }
        .into()),
    }
}
