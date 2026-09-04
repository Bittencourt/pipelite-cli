use anyhow::Result;
use serde_json::json;

use crate::cli::trash::TrashRestoreArgs;
use crate::commands::trash::normalize_trash_type;
use crate::context::AppContext;
use crate::dry_run;
use crate::error::CliError;

/// Hint substituted into 404s (docs-command re-wrap precedent): the server
/// 404s anything not currently in the trash — no existence oracle.
const NOT_IN_TRASH_HINT: &str =
    "The record is not in the trash — it may already be restored or purged, or never existed.";

/// Restore one trashed record.
///
/// restore IS the recovery act — NO confirmation, NO prompt. The type
/// normalizes to the plural tab BEFORE the URL is built (exit 2 pre-HTTP
/// for unknown types); a 404 re-wraps preserving the server's detail with
/// the not-in-trash hint; every other error — including a foreign-record
/// 403, which carries the GENERAL permission hint via surface "general"
/// (never the purge hint, P6) — propagates untouched.
pub async fn run(ctx: &AppContext, args: &TrashRestoreArgs) -> Result<()> {
    let tab = normalize_trash_type(&args.trash_type)?;
    let url = format!(
        "{}/api/v1/trash/{}/{}/restore",
        ctx.client.base_url(),
        tab,
        args.id
    );

    if ctx.dry_run {
        return dry_run::render_dry_run("POST", &url, &json!({}), &ctx.output_format, ctx.color);
    }

    if let Err(err) = ctx.client.restore_trash(tab, &args.id).await {
        // Re-wrap 404s: PRESERVE the server's detail, swap in the
        // not-in-trash hint. Connection/Auth/Forbidden/Api and every other
        // variant pass through untouched.
        let rewritten: anyhow::Error = match err.downcast::<CliError>() {
            Ok(CliError::NotFound { detail, .. }) => CliError::NotFound {
                detail,
                hint: NOT_IN_TRASH_HINT.to_string(),
            }
            .into(),
            Ok(other) => other.into(),
            Err(other) => other,
        };
        return Err(rewritten);
    }

    if !ctx.quiet {
        println!("Restored {tab} {}", args.id);
    }

    Ok(())
}
