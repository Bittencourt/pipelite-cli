use std::io::IsTerminal;

use anyhow::Result;

use crate::cli::trash::TrashPurgeArgs;
use crate::commands::trash::normalize_trash_type;
use crate::context::AppContext;
use crate::error::{self, CliError};

/// Page size for the victim fan-out (the server's page max).
const FANOUT_LIMIT: u64 = 100;

/// The server clamps trash offsets to ≤ 10,000 — past-cap pages return
/// EMPTY data with a truthful meta.total, so a total-driven loop would
/// never terminate (P3). The fan-out therefore also breaks when the next
/// offset would move past the cap.
const OFFSET_CAP: u64 = 10_000;

/// Build the confirmation prompt — the strongest wording in the codebase.
///
/// Pure function so the wording is unit-gated (the prompt itself is
/// TTY-only): it must name the scope label, the record count, and the exact
/// words "permanently destroys".
fn purge_prompt(scope_label: &str, count: usize) -> String {
    format!(
        "Purge {scope_label}? This permanently destroys {count} record(s) and cannot be undone"
    )
}

/// Permanently destroy trashed records — the order of operations IS the
/// contract:
///
/// 1. `--type` normalizes BEFORE anything else (unknown → exit 2, zero
///    HTTP — the gate precedes both the refusal and every request);
/// 2. `--dry-run` previews the victims with list GETs only, zero DELETEs;
/// 3. non-interactive without `--force` refuses with
///    `CliError::InvalidInput` (exit 2 — deliberately stricter than the
///    standard delete's exit-1 refusal, P2) BEFORE the fan-out list: zero
///    HTTP for the whole command;
/// 4. interactive use confirms with the strongest prompt in the codebase
///    (scope + count + "permanently destroys", default false); an empty
///    scope short-circuits with a hint, exit 0;
/// 5. execution issues one DELETE per victim, continue-on-error (a 403
///    mid-fan-out renders the admin trash hint and counts as a failure —
///    the loop continues per the orchestrator contract), then reports
///    "{ok} permanently destroyed[, M failed]" and exits 1 on any failure
///    via the pinned `anyhow::bail!`.
pub async fn run(ctx: &AppContext, args: &TrashPurgeArgs) -> Result<()> {
    // (1) Scope validation BEFORE any HTTP (test: purge_scope_validation_first).
    let trash_type = match args.trash_type.as_deref() {
        Some(t) => Some(normalize_trash_type(t)?),
        None => None,
    };
    let scope_label = match trash_type {
        Some(t) => format!("all trashed {t} records"),
        None => "all trashed records".to_string(),
    };

    // (2) Dry-run intercept FIRST — previews with list GETs only.
    if ctx.dry_run {
        let victims = collect_victims(ctx, trash_type).await?;
        for (tab, id) in &victims {
            println!("Would permanently destroy {tab} {id}");
        }
        if !ctx.quiet {
            println!("Would permanently destroy {} record(s) in total", victims.len());
        }
        return Ok(());
    }

    // (3) Non-interactive refusal BEFORE the fan-out list — zero HTTP for
    // the whole command (PINNED by the unreachable-server test). By design
    // this is InvalidInput (exit 2), NOT the batch-delete exit-1 variant.
    if !args.force && (ctx.no_input || !std::io::stdin().is_terminal()) {
        return Err(CliError::InvalidInput {
            detail: "Refusing to permanently destroy trashed records without confirmation \
                     in non-interactive mode."
                .to_string(),
            hint: "Re-run with --force to skip confirmation (scripts), or run interactively."
                .to_string(),
        }
        .into());
    }

    // (4) Collect the victims, then confirm on a TTY.
    let victims = collect_victims(ctx, trash_type).await?;
    if victims.is_empty() {
        if !ctx.quiet {
            eprintln!("Trash is empty — nothing to purge.");
        }
        return Ok(());
    }
    if !args.force {
        // The refusal gate above guarantees a terminal + prompts allowed here.
        let confirmed = dialoguer::Confirm::new()
            .with_prompt(purge_prompt(&scope_label, victims.len()))
            .default(false)
            .interact()?;
        if !confirmed {
            println!("Aborted");
            return Ok(());
        }
    }

    // (5) Execute: one DELETE per victim, continue-on-error.
    let mut ok = 0usize;
    let mut failed = 0usize;
    for (tab, id) in &victims {
        match ctx.client.purge_trash(tab, id).await {
            Ok(()) => {
                ok += 1;
                if !ctx.quiet {
                    println!("Permanently destroyed {tab} {id}");
                }
            }
            Err(e) => {
                failed += 1;
                // Full title + detail + hint (403s carry the admin trash
                // hint automatically via surface "trash").
                error::display_error(&e, ctx.color);
            }
        }
    }

    if !ctx.quiet {
        if failed == 0 {
            println!("{ok} permanently destroyed");
        } else {
            println!("{ok} permanently destroyed, {failed} failed");
        }
    }
    if failed > 0 {
        // The pinned exit-1 mechanism: a plain bail (NOT a structured
        // command error) keeps per-item failures on the item-failure tier.
        anyhow::bail!("{ok} permanently destroyed, {failed} failed — review the per-item errors above");
    }

    Ok(())
}

/// Fan out paged list requests to collect `(plural tab, id)` victims until
/// one of the four break conditions holds (P3): empty page, partial page,
/// accumulated ≥ meta.total, or next offset past the server's 10,000
/// offset cap. meta.total is the SELECTED tab's count only (S6).
async fn collect_victims(
    ctx: &AppContext,
    trash_type: Option<&str>,
) -> Result<Vec<(String, String)>> {
    let mut victims: Vec<(String, String)> = Vec::new();
    let mut offset: u64 = 0;
    loop {
        let page = ctx.client.list_trash(trash_type, FANOUT_LIMIT, offset).await?;
        let total = page.meta.total;
        let page_len = page.data.len() as u64;
        let empty_page = page_len == 0;
        let partial_page = page_len < FANOUT_LIMIT;
        for row in page.data {
            victims.push((row.tab, row.id));
        }
        offset += page_len;
        if empty_page || partial_page || (victims.len() as u64) >= total || offset > OFFSET_CAP {
            break;
        }
    }
    Ok(victims)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The TTY-only wording gate: the prompt must name the scope, the
    /// count, and the exact words "permanently destroys" — the strongest
    /// confirmation in the codebase.
    #[test]
    fn prompt_wording_names_scope_count_and_destruction() {
        let prompt = purge_prompt("all tabs", 7);
        assert!(prompt.contains("all tabs"), "scope label: {prompt}");
        assert!(prompt.contains("7"), "count: {prompt}");
        assert!(
            prompt.contains("permanently destroys"),
            "strongest wording: {prompt}"
        );
    }
}
