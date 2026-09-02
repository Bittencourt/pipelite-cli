use anyhow::Result;

use crate::batch;
use crate::cache::KEY_STAGES;
use crate::cli::stages::StagesDeleteArgs;
use crate::context::AppContext;
use crate::dry_run;

/// Delete one or more stages.
///
/// Single ID executes the original delete flow (v1.0 behavior). Multiple IDs
/// (or --stdin with a JSON array of string IDs) run a batch delete with
/// confirmation prompt, continue-on-error semantics, and a summary report.
pub async fn run(ctx: &AppContext, args: &StagesDeleteArgs) -> Result<()> {
    let ids = batch::collect_delete_ids("stages", args.stdin, &args.ids, r#"["stg_1","stg_2"]"#)?;

    if ids.len() == 1 {
        return single_delete(ctx, &ids[0]).await;
    }

    batch::run_batch_delete(
        ctx,
        "stage",
        "stage(s)",
        // TODO(07): wire to a --force flag in the CR-01 fix commit.
        false,
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
