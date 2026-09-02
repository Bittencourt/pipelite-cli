use anyhow::Result;

use crate::batch;
use crate::cache::KEY_PIPELINES;
use crate::cli::pipelines::PipelinesDeleteArgs;
use crate::context::AppContext;
use crate::dry_run;

/// Delete one or more pipelines.
///
/// Single ID executes the original delete flow (v1.0 behavior). Multiple IDs
/// (or --stdin with a JSON array of string IDs) run a batch delete with
/// confirmation prompt, continue-on-error semantics, and a summary report.
pub async fn run(ctx: &AppContext, args: &PipelinesDeleteArgs) -> Result<()> {
    let ids = batch::collect_delete_ids("pipelines", args.stdin, &args.ids, r#"["pl_1","pl_2"]"#)?;

    if ids.len() == 1 {
        return single_delete(ctx, &ids[0]).await;
    }

    batch::run_batch_delete(
        ctx,
        "pipeline",
        "pipeline(s)",
        // TODO(07): wire to a --force flag in the CR-01 fix commit.
        false,
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
