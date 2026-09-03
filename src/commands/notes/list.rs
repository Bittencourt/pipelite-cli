use anyhow::Result;

use crate::api::models::{notes_table_config, Note};
use crate::cli::notes::NotesListArgs;
use crate::commands::notes::resolve_entity_type;
use crate::context::AppContext;
use crate::output;
use crate::output::format::truncate_with_ellipsis;
use crate::output::OutputFormat;

/// List notes on a parent record.
///
/// The server owns ordering (createdAt DESC, id DESC — newest first) and
/// the page cap (limit clamped to [1, 100]): `--limit`/`--offset` pass
/// through untouched and there is no `--all` flag by design. Notes are
/// never cached (append-heavy parent-scoped data goes stale instantly).
///
/// Table mode flattens newlines in content and truncates the cell to ~80
/// chars so multi-line bodies cannot break the row (T-10-01); json, plain,
/// and csv keep the full raw text — plain is a machine-readable format like
/// json. An empty page prints ONE stderr hint (exit 0, suppressed by
/// --quiet); unlike workflow runs there are no hidden rows server-side, so
/// no probe request is needed.
pub async fn run(ctx: &AppContext, args: &NotesListArgs) -> Result<()> {
    let segment = resolve_entity_type(&args.entity_type)?;

    let config = notes_table_config();
    let columns: Vec<String> = config
        .default_columns
        .iter()
        .map(|s| s.to_string())
        .collect();

    let response = ctx
        .client
        .list_notes(segment, &args.parent_id, args.limit, args.offset)
        .await?;

    if response.data.is_empty() && !ctx.quiet {
        eprintln!(
            "No notes on {entity_type} {parent_id} yet — add one with: pipelite notes add {entity_type} {parent_id} --body \"<text>\"",
            entity_type = args.entity_type,
            parent_id = args.parent_id,
        );
    }

    let truncate = matches!(ctx.output_format, OutputFormat::Table);
    let items = notes_to_values(&response.data, truncate)?;

    output::render_list(
        &items,
        &ctx.output_format,
        &columns,
        &args.fields,
        ctx.color,
        Some(&response.meta),
    )
}

/// Convert notes to serde_json::Value for the output layer.
///
/// `truncate` (table mode ONLY) flattens \n/\r to spaces and truncates the
/// content cell to ~80 chars (T-10-01); every other format keeps the full
/// raw text so scripts and the view-before-edit path see the real body.
fn notes_to_values(notes: &[Note], truncate: bool) -> Result<Vec<serde_json::Value>> {
    notes
        .iter()
        .map(|n| {
            let mut value = serde_json::to_value(n)?;
            if truncate {
                if let serde_json::Value::Object(ref mut map) = value {
                    let flattened: String = n.content.replace(['\n', '\r'], " ");
                    map.insert(
                        "content".to_string(),
                        serde_json::Value::String(truncate_with_ellipsis(&flattened, 80)),
                    );
                }
            }
            Ok(value)
        })
        .collect()
}
