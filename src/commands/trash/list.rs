use anyhow::Result;

use crate::api::models::{trash_table_config, PaginationMeta, TrashRow};
use crate::cli::trash::TrashListArgs;
use crate::commands::trash::{deleted_by_label, normalize_trash_type};
use crate::context::AppContext;
use crate::output;
use crate::output::format::truncate_with_ellipsis;
use crate::output::OutputFormat;

/// Page size for the `--all` fan-out (the server's page max).
const FANOUT_LIMIT: u64 = 100;

/// The server clamps trash offsets to ≤ 10,000 — past-cap pages return
/// EMPTY data with a truthful meta.total, so a total-driven loop would
/// never terminate (P3). The fan-out therefore also breaks when the next
/// offset would move past the cap.
const OFFSET_CAP: u64 = 10_000;

/// List trashed records.
///
/// `--type` accepts nine singular/plural aliases, normalized to the plural
/// tab BEFORE any request (unknown → exit 2 with zero HTTP); omitting it
/// sends no type param and the server defaults to the deals tab.
///
/// `--all` fans out at limit=100 and breaks on ALL FOUR conditions (P3):
/// empty page, partial page, accumulated ≥ meta.total, or next offset past
/// the server's 10,000 offset cap. meta.total is the SELECTED tab's count
/// only (S6) — the loop trusts page emptiness, never total reachability.
///
/// Table cells truncate linked_parents (joined, ~80 chars) and collapse
/// deleted_by to a kind label; json/plain/csv keep the full array and the
/// full object. Trash is never persisted locally — no local copy of any
/// kind is written or read by this command.
pub async fn run(ctx: &AppContext, args: &TrashListArgs) -> Result<()> {
    // Normalize BEFORE any HTTP — unknown types exit 2 with zero requests.
    let trash_type = match args.trash_type.as_deref() {
        Some(t) => Some(normalize_trash_type(t)?),
        None => None,
    };

    let (rows, meta) = if args.all {
        let rows = fetch_all(ctx, trash_type).await?;
        let meta = PaginationMeta {
            total: rows.len() as u64,
            offset: 0,
            limit: FANOUT_LIMIT,
        };
        (rows, meta)
    } else {
        let response = ctx
            .client
            .list_trash(trash_type, args.limit, args.offset)
            .await?;
        (response.data, response.meta)
    };

    if rows.is_empty() && !ctx.quiet {
        match trash_type {
            Some(t) => eprintln!(
                "No trashed {t} records — restore one with: pipelite trash restore {t} <id>"
            ),
            None => eprintln!(
                "No trashed records — restore one with: pipelite trash restore <type> <id>"
            ),
        }
    }

    let config = trash_table_config();
    let columns: Vec<String> = config
        .default_columns
        .iter()
        .map(|s| s.to_string())
        .collect();

    let truncate = matches!(ctx.output_format, OutputFormat::Table);
    let items = trash_to_values(&rows, truncate)?;

    output::render_list(
        &items,
        &ctx.output_format,
        &columns,
        &args.fields,
        ctx.color,
        Some(&meta),
    )
}

/// Fan out paged list requests until one of the four break conditions holds
/// (P3): empty page, partial page, accumulated ≥ meta.total, or next offset
/// past the server's 10,000 offset cap.
async fn fetch_all(ctx: &AppContext, trash_type: Option<&str>) -> Result<Vec<TrashRow>> {
    let mut rows: Vec<TrashRow> = Vec::new();
    let mut offset: u64 = 0;
    loop {
        let page = ctx.client.list_trash(trash_type, FANOUT_LIMIT, offset).await?;
        let total = page.meta.total;
        let page_len = page.data.len() as u64;
        let empty_page = page_len == 0;
        let partial_page = page_len < FANOUT_LIMIT;
        rows.extend(page.data);
        offset += page_len;
        if empty_page || partial_page || (rows.len() as u64) >= total || offset > OFFSET_CAP {
            break;
        }
    }
    Ok(rows)
}

/// Convert trash rows to serde_json::Value for the output layer.
///
/// Table mode ONLY joins linked_parents with ", " and truncates the cell to
/// ~80 chars, and collapses deleted_by to its kind label; every other
/// format keeps the full array and the full deleted_by object (T-11-09).
/// The `type` key stays the PLURAL tab in every format — it is the
/// round-trip token for restore/purge.
fn trash_to_values(rows: &[TrashRow], truncate: bool) -> Result<Vec<serde_json::Value>> {
    rows.iter()
        .map(|r| {
            let mut value = serde_json::to_value(r)?;
            if truncate {
                if let serde_json::Value::Object(ref mut map) = value {
                    let parents = r.linked_parents.join(", ");
                    map.insert(
                        "linked_parents".to_string(),
                        serde_json::Value::String(truncate_with_ellipsis(&parents, 80)),
                    );
                    map.insert(
                        "deleted_by".to_string(),
                        serde_json::Value::String(deleted_by_label(&r.deleted_by)),
                    );
                }
            }
            Ok(value)
        })
        .collect()
}
