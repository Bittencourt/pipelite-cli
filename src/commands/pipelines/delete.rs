use anyhow::Result;

use crate::batch;
use crate::cache::KEY_PIPELINES;
use crate::cli::pipelines::PipelinesDeleteArgs;
use crate::context::AppContext;
use crate::dry_run;

/// Delete one or more pipelines.
///
/// A single positional ID executes the original delete flow (v1.0 behavior).
/// Multiple IDs — or --stdin with a JSON array of string IDs, even a 1-ID
/// list — run a batch delete with confirmation prompt, continue-on-error
/// semantics, and a summary report. Non-interactive runs (e.g. piped --stdin)
/// must pass --force: stdin cannot carry both the IDs and a confirmation
/// prompt.
pub async fn run(ctx: &AppContext, args: &PipelinesDeleteArgs) -> Result<()> {
    let ids = batch::collect_delete_ids("pipelines", args.stdin, &args.ids, r#"["pl_1","pl_2"]"#)?;

    // WR-06: piped --stdin input always takes the batch path (with its
    // --force gate) regardless of item count; only a single positional ID
    // keeps the v1.0 gate-free flow.
    if ids.len() == 1 && !args.stdin {
        return single_delete(ctx, &ids[0]).await;
    }

    batch::run_batch_delete(
        ctx,
        "pipeline",
        "pipeline(s)",
        args.force,
        &ids,
        "pipelines",
        KEY_PIPELINES,
        Some("stages_"),
        async |id: &str| ctx.client.delete_pipeline(id).await,
    )
    .await
}

/// Delete a single pipeline (original behavior).
async fn single_delete(ctx: &AppContext, id: &str) -> Result<()> {
    if ctx.dry_run {
        let url = format!("{}/api/v1/pipelines/{}", ctx.client.base_url(), id);
        return dry_run::render_dry_run_delete(
            "pipeline",
            id,
            &url,
            &ctx.output_format,
            ctx.color,
        );
    }

    ctx.client.delete_pipeline(id).await?;

    if let Some(ref cache) = ctx.cache {
        cache.invalidate(KEY_PIPELINES);
        cache.invalidate_prefix("stages_");
    }

    if !ctx.quiet {
        println!("Deleted pipeline {}", id);
    }

    Ok(())
}
