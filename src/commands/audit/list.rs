use anyhow::Result;

use crate::api::models::{audit_table_config, AuditEntry};
use crate::cli::audit::AuditListArgs;
use crate::context::AppContext;
use crate::output;
use crate::output::format::truncate_with_ellipsis;
use crate::output::OutputFormat;

/// The server clamps limit into [1, 100] (parsePagination) — the CLI clamps
/// client-side so the wire always carries the effective value (T-11-11):
/// --limit 0 → 1, --limit 150 → 100.
const MIN_LIMIT: u64 = 1;
const MAX_LIMIT: u64 = 100;

/// Table-only cell width for the actor and entity cells. `--format json`
/// keeps every field in full.
const CELL_WIDTH: usize = 40;

/// List audit-log entries (read-only; the server rejects non-admin keys
/// with 403 before the query is even validated — the client renders the
/// registered audit hint).
///
/// `--limit` is clamped into 1..=100 before the request; the four filters
/// pass through verbatim (the client omits EMPTY values from the wire —
/// the server 422s `entity_id=`). There is deliberately NO `--all`: the
/// server's only offset bound is the global 1e6 clamp, not a practical
/// limit, so deep history is reached by iterating `--offset`.
///
/// Table cells render the actor (kind + first present id) and the entity
/// (type/id) truncated; the `changes` payload NEVER renders in the table —
/// `--format json` exposes the full entry including verbatim changes and
/// all actor ids (AUDT-03 diff rendering deliberately deferred). Audit
/// results are never cached.
pub async fn run(ctx: &AppContext, args: &AuditListArgs) -> Result<()> {
    let limit = args.limit.clamp(MIN_LIMIT, MAX_LIMIT);

    let response = ctx
        .client
        .list_audit(
            args.entity_type.as_deref(),
            args.entity_id.as_deref(),
            args.actor_kind.as_deref(),
            args.workflow_run_id.as_deref(),
            limit,
            args.offset,
        )
        .await?;

    let rows = response.data;
    let meta = response.meta;

    if rows.is_empty() && !ctx.quiet {
        eprintln!("No audit entries — adjust filters or widen the offset window");
    }

    let config = audit_table_config();
    let columns: Vec<String> = config
        .default_columns
        .iter()
        .map(|s| s.to_string())
        .collect();

    let truncate = matches!(ctx.output_format, OutputFormat::Table);
    let items = audit_to_values(&rows, truncate)?;

    output::render_list(
        &items,
        &ctx.output_format,
        &columns,
        &args.fields,
        ctx.color,
        Some(&meta),
    )
}

/// Convert audit entries to serde_json::Value for the output layer.
///
/// Table mode ONLY adds the composed `actor` and `entity` cells (truncated
/// to 40 chars); every other format keeps the full entry — verbatim
/// changes payload and all actor id fields included.
fn audit_to_values(rows: &[AuditEntry], truncate: bool) -> Result<Vec<serde_json::Value>> {
    rows.iter()
        .map(|r| {
            let mut value = serde_json::to_value(r)?;
            if truncate {
                if let serde_json::Value::Object(ref mut map) = value {
                    map.insert(
                        "actor".to_string(),
                        serde_json::Value::String(actor_cell(r)),
                    );
                    map.insert(
                        "entity".to_string(),
                        serde_json::Value::String(entity_cell(r)),
                    );
                }
            }
            Ok(value)
        })
        .collect()
}

/// Actor table cell: the actor kind, plus " ({id})" from the FIRST present
/// id field (actor_user_id → workflow_run_id → import_session_id), truncated
/// for the table only — `--format json` keeps every id field in full.
fn actor_cell(r: &AuditEntry) -> String {
    let id = r
        .actor_user_id
        .as_deref()
        .or(r.workflow_run_id.as_deref())
        .or(r.import_session_id.as_deref());
    let raw = match id {
        Some(i) => format!("{} ({})", r.actor_kind, i),
        None => r.actor_kind.clone(),
    };
    truncate_with_ellipsis(&raw, CELL_WIDTH)
}

/// Entity table cell: "{entity_type}/{entity_id}", truncated for the table
/// only — `--format json` keeps both fields in full (a 100-char id must
/// survive json intact).
fn entity_cell(r: &AuditEntry) -> String {
    truncate_with_ellipsis(&format!("{}/{}", r.entity_type, r.entity_id), CELL_WIDTH)
}
