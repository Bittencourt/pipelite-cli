use anyhow::Result;

use crate::api::models::{notes_table_config, NoteUpdate};
use crate::cli::notes::NotesEditArgs;
use crate::commands::notes::body::resolve_body;
use crate::commands::notes::resolve_entity_type;
use crate::context::AppContext;
use crate::dry_run;
use crate::output;
use crate::output::OutputFormat;

/// Edit a note's content by note ID.
///
/// The entity type and parent id are grammar-locked (validated, then
/// unused): the PATCH request to /api/v1/notes/{noteId} carries ONLY the
/// note id (open question 2: no entity_id mismatch warning — note IDs are
/// UUIDs obtained via `notes list <type> <id> --json`; a mismatched parent
/// surfaces as the server's 404 anyway). No confirmation — editing is
/// non-destructive and idempotent; --dry-run previews the PATCH.
///
/// The prompt label carries the locked no-single-GET wording so users can
/// see why existing content is not shown; the same fact is stated in the
/// subcommand after_help. The body resolves before the dry-run intercept
/// (prompting suppressed under --dry-run) so --dry-run never prompts and
/// never hits the network.
pub async fn run(ctx: &AppContext, args: &NotesEditArgs) -> Result<()> {
    // Grammar validation; the value is otherwise unused by the request.
    resolve_entity_type(&args.entity_type)?;

    let content = resolve_body(
        &args.body,
        args.stdin,
        ctx.no_input || ctx.dry_run,
        Some((
            "Edit note",
            "New note content (the server offers no single-note GET — run `notes list <type> <id> --json` to view existing content first)",
        )),
    )?;

    let url = format!("{}/api/v1/notes/{}", ctx.client.base_url(), args.note_id);

    if ctx.dry_run {
        return dry_run::render_dry_run(
            "PATCH",
            &url,
            &serde_json::json!({ "content": content }),
            &ctx.output_format,
            ctx.color,
        );
    }

    let note = ctx
        .client
        .update_note(&args.note_id, &NoteUpdate { content })
        .await?;

    if matches!(ctx.output_format, OutputFormat::Json) {
        let value = serde_json::to_value(&note)?;
        let config = notes_table_config();
        let columns: Vec<String> = config
            .default_columns
            .iter()
            .map(|s| s.to_string())
            .collect();
        output::render_single(&value, &ctx.output_format, &columns, &None, ctx.color)?;
    } else if !ctx.quiet {
        println!("Updated note {}", note.id);
    }

    Ok(())
}
