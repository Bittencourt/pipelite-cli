use anyhow::Result;

use crate::batch;

use crate::cli::notes::NotesDeleteArgs;
use crate::commands::notes::resolve_entity_type;
use crate::context::AppContext;
use crate::dry_run;

/// Delete (soft-delete) a note by ID — the exact templates `single_delete`
/// contract (Phase 7/9/10 lock), verbatim-adjusted:
///
/// 1. Dry-run intercept FIRST — --dry-run previews the DELETE URL and
///    never prompts.
/// 2. TTY && !--force → dialoguer Confirm (default false, "Aborted" on
///    decline); non-TTY && !--force → CliError::Validation refusal
///    (exit 1, zero HTTP).
/// 3. `delete_note` (204 soft delete; a sequential re-delete 404s as the
///    normal NotFound failure path). No cache invalidation — notes are
///    never cached.
///
/// Single delete only: notes have no batch path (locked). The entity type
/// and parent id are grammar-locked (validated, then unused) — the request
/// carries only the note id.
pub async fn run(ctx: &AppContext, args: &NotesDeleteArgs) -> Result<()> {
    // Grammar validation; the value is otherwise unused by the request.
    resolve_entity_type(&args.entity_type)?;

    let id = &args.note_id;

    // Dry-run intercept FIRST — this MUST come before the confirmation
    // prompt so --dry-run never prompts (matching templates delete).
    if ctx.dry_run {
        let url = format!("{}/api/v1/notes/{}", ctx.client.base_url(), id);
        return dry_run::render_dry_run_delete("note", id, &url, &ctx.output_format, ctx.color);
    }

    match batch::ensure_delete_consent(
        ctx,
        args.force,
        format!("Delete note {id}?"),
        "Use --force to skip confirmation: pipelite notes delete <type> <parent-id> <note-id> --force".to_string(),
    )? {
        batch::Consent::Declined => return Ok(()),
        batch::Consent::Granted => {}
    }

    ctx.client.delete_note(id).await?;

    if !ctx.quiet {
        println!("Deleted note {id}");
    }

    Ok(())
}
