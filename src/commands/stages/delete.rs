use anyhow::Result;

use crate::batch;
use crate::cache::KEY_STAGES;
use crate::cli::stages::StagesDeleteArgs;
use crate::context::AppContext;
use crate::dry_run;

/// Delete one or more stages.
///
/// A single positional ID executes the original delete flow (v1.0 behavior).
/// Multiple IDs — or --stdin with a JSON array of string IDs, even a 1-ID
/// list — run a batch delete with confirmation prompt, continue-on-error
/// semantics, and a summary report. Non-interactive runs (e.g. piped --stdin)
/// must pass --force: stdin cannot carry both the IDs and a confirmation
/// prompt.
pub async fn run(ctx: &AppContext, args: &StagesDeleteArgs) -> Result<()> {
    let ids = batch::collect_delete_ids("stages", args.stdin, &args.ids, r#"["stg_1","stg_2"]"#)?;

    // WR-06: piped --stdin input always takes the batch path (with its
    // --force gate) regardless of item count; only a single positional ID
    // keeps the v1.0 gate-free flow.
    if ids.len() == 1 && !args.stdin {
        return single_delete(ctx, &ids[0]).await;
    }

    batch::run_batch_delete(
        ctx,
        "stage",
        "stage(s)",
        args.force,
        &ids,
        "stages",
        KEY_STAGES,
        Some("stages_"),
        async |id: &str| ctx.client.delete_stage(id).await,
    )
    .await
}

/// Delete a single stage (original behavior).
async fn single_delete(ctx: &AppContext, id: &str) -> Result<()> {
    if ctx.dry_run {
        let url = format!("{}/api/v1/stages/{}", ctx.client.base_url(), id);
        return dry_run::render_dry_run_delete(
            "stage",
            id,
            &url,
            &ctx.output_format,
            ctx.color,
        );
    }

    ctx.client.delete_stage(id).await?;

    if let Some(ref cache) = ctx.cache {
        cache.invalidate(KEY_STAGES);
        cache.invalidate_prefix("stages_");
    }

    if !ctx.quiet {
        println!("Deleted stage {}", id);
    }

    Ok(())
}
