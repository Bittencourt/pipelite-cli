use anyhow::Result;

use crate::batch;
use crate::cache::KEY_DEALS;
use crate::cli::deals::DealsDeleteArgs;
use crate::context::AppContext;
use crate::dry_run;

/// Delete one or more deals.
///
/// A single positional ID executes the original delete flow (v1.0 behavior).
/// Multiple IDs — or --stdin with a JSON array of string IDs, even a 1-ID
/// list — run a batch delete with confirmation prompt, continue-on-error
/// semantics, and a summary report. Non-interactive runs (e.g. piped --stdin)
/// must pass --force: stdin cannot carry both the IDs and a confirmation
/// prompt.
pub async fn run(ctx: &AppContext, args: &DealsDeleteArgs) -> Result<()> {
    let ids = batch::collect_delete_ids("deals", args.stdin, &args.ids, r#"["id1","id2"]"#)?;

    // WR-06: piped --stdin input always takes the batch path (with its
    // --force gate) regardless of item count; only a single positional ID
    // keeps the v1.0 gate-free flow.
    if ids.len() == 1 && !args.stdin {
        return single_delete(ctx, &ids[0]).await;
    }

    batch::run_batch_delete(
        ctx,
        "deal",
        "deal(s)",
        args.force,
        &ids,
        "deals",
        KEY_DEALS,
        None,
        async |id: &str| ctx.client.delete_deal(id).await,
    )
    .await
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
