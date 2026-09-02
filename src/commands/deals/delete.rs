use std::io::{self, IsTerminal};

use anyhow::Result;

use crate::batch;
use crate::cache::KEY_DEALS;
use crate::cli::deals::DealsDeleteArgs;
use crate::context::AppContext;
use crate::dry_run;
use crate::error::CliError;

/// Delete one or more deals.
///
/// Single ID executes the original delete flow. Multiple IDs (or --stdin with
/// a JSON array of string IDs) run a batch delete with confirmation prompt,
/// continue-on-error semantics, and a summary report.
pub async fn run(ctx: &AppContext, args: &DealsDeleteArgs) -> Result<()> {
    let ids = collect_ids(args)?;

    if ids.len() == 1 {
        return single_delete(ctx, &ids[0]).await;
    }

    batch_delete(ctx, &ids).await
}

/// Collect IDs from positional args or --stdin (mutually exclusive per D-04).
fn collect_ids(args: &DealsDeleteArgs) -> Result<Vec<String>> {
    if args.stdin {
        if !args.ids.is_empty() {
            return Err(CliError::Validation {
                detail: "--stdin and positional IDs are mutually exclusive".to_string(),
                hint: "Use either positional IDs or --stdin, not both.".to_string(),
            }
            .into());
        }
        if io::stdin().is_terminal() {
            return Err(CliError::Validation {
                detail: "No data on stdin".to_string(),
                hint: "Pipe JSON data: echo '[\"id1\",\"id2\"]' | pipelite deals delete --stdin"
                    .to_string(),
            }
            .into());
        }
        let ids: Vec<String> = batch::read_stdin_json("string IDs")?;
        if ids.is_empty() {
            return Err(CliError::Validation {
                detail: "Empty ID list".to_string(),
                hint: "Provide at least one ID to delete.".to_string(),
            }
            .into());
        }
        return Ok(ids);
    }
    Ok(args.ids.clone())
}

/// Delete a single deal (original behavior).
async fn single_delete(ctx: &AppContext, id: &str) -> Result<()> {
    if ctx.dry_run {
        let url = format!("{}/api/v1/deals/{}", ctx.client.base_url(), id);
        return dry_run::render_dry_run_delete("deal", id, &url, &ctx.output_format, ctx.color);
    }

    ctx.client.delete_deal(id).await?;

    if let Some(ref cache) = ctx.cache {
        cache.invalidate(KEY_DEALS);
    }

    if !ctx.quiet {
        println!("Deleted deal {}", id);
    }

    Ok(())
}

/// Batch delete deals with confirmation prompt (per D-04, D-05, D-06).
async fn batch_delete(ctx: &AppContext, ids: &[String]) -> Result<()> {
    let total = ids.len();

    // FIRST: Dry-run check (per D-05) — show what would be deleted, no prompt needed.
    // This MUST come before the confirmation prompt so --dry-run never prompts.
    if ctx.dry_run {
        for id in ids {
            let url = format!("{}/api/v1/deals/{}", ctx.client.base_url(), id);
            dry_run::render_dry_run_delete("deal", id, &url, &ctx.output_format, ctx.color)?;
        }
        return Ok(());
    }

    // SECOND: Confirmation prompt (per D-05) — only shown for actual deletions,
    // when stdin is TTY and --no-input is not set.
    if !ctx.no_input && io::stdin().is_terminal() {
        let confirm = dialoguer::Confirm::new()
            .with_prompt(format!("Delete {} deal(s)?", total))
            .default(false)
            .interact()?;
        if !confirm {
            return Ok(());
        }
    }

    let mut outcome = batch::BatchOutcome::new(total);

    for (i, id) in ids.iter().enumerate() {
        match ctx.client.delete_deal(id).await {
            Ok(()) => {
                outcome.record_success();
                if !ctx.quiet {
                    println!("Deleted deal {}", id);
                }
            }
            Err(e) => {
                outcome.record_failure(i, id, &e);
            }
        }
    }

    if outcome.succeeded > 0 {
        if let Some(ref cache) = ctx.cache {
            cache.invalidate(KEY_DEALS);
        }
    }

    outcome.finalize("deal", "delete")
}
