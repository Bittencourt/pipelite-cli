use anyhow::Result;

use crate::api::models::{notes_table_config, NoteCreate};
use crate::cli::notes::NotesAddArgs;
use crate::commands::notes::body::resolve_body;
use crate::commands::notes::resolve_entity_type;
use crate::context::AppContext;
use crate::dry_run;
use crate::output;
use crate::output::OutputFormat;

/// Add a note to a parent record.
///
/// The body resolves from exactly one source (--body literal > @file/@- >
/// --stdin > TTY prompt) BEFORE the dry-run intercept — the preview needs
/// the resolved text. Prompting is disabled under --dry-run (the resolver
/// gets `no_input || dry_run`), so a --dry-run with no explicit source
/// previews the MissingInput rejection instead of prompting: --dry-run
/// never prompts and never hits the network.
///
/// The wire body is exactly {"content": …} — the server forces the author
/// to the API key's user and `source` to "user"; nothing else is sent.
pub async fn run(ctx: &AppContext, args: &NotesAddArgs) -> Result<()> {
    let segment = resolve_entity_type(&args.entity_type)?;

    // Body resolution BEFORE the dry-run intercept; prompting suppressed
    // under --dry-run so the preview never blocks on stdin.
    let content = resolve_body(
        &args.body,
        args.stdin,
        ctx.no_input || ctx.dry_run,
        Some(("Add note", "Note content")),
    )?;

    if ctx.dry_run {
        let url = format!(
            "{}/api/v1/{}/{}/notes",
            ctx.client.base_url(),
            segment,
            args.parent_id
        );
        return dry_run::render_dry_run(
            "POST",
            &url,
            &serde_json::json!({ "content": content }),
            &ctx.output_format,
            ctx.color,
        );
    }

    let note = ctx
        .client
        .create_note(segment, &args.parent_id, &NoteCreate { content })
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
        println!(
            "Created note {id} on {entity_type} {parent_id}",
            id = note.id,
            entity_type = args.entity_type,
            parent_id = args.parent_id
        );
    }

    Ok(())
}
