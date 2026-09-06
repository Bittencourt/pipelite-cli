use anyhow::Result;

use crate::batch;
use crate::cache::KEY_WORKFLOWS;
use crate::cli::workflows::WorkflowsDeleteArgs;
use crate::context::AppContext;
use crate::dry_run;

/// Delete one or more workflows.
///
/// Single ID executes the single-delete flow (shared consent gate; --force skips).
/// Multiple IDs (or --stdin with a JSON array of string IDs) run a batch
/// delete with confirmation prompt, continue-on-error semantics, and a
/// summary report. --force skips the batch confirmation too, and is required
/// for non-interactive runs (e.g. piped --stdin), making the 1-ID and
/// multi-ID boundaries consistent.
pub async fn run(ctx: &AppContext, args: &WorkflowsDeleteArgs) -> Result<()> {
    let ids = batch::collect_delete_ids("workflows", args.stdin, &args.ids, r#"["wf_1","wf_2"]"#)?;

    if ids.len() == 1 {
        return single_delete(ctx, args, &ids[0]).await;
    }

    batch::run_batch_delete(
        ctx,
        "workflow",
        "workflow(s)",
        args.force,
        &ids,
        "workflows",
        KEY_WORKFLOWS,
        None,
        async |id: &str| ctx.client.delete_workflow(id).await,
    )
    .await
}

/// Delete a single workflow (original behavior, --force skips confirmation).
async fn single_delete(ctx: &AppContext, args: &WorkflowsDeleteArgs, id: &str) -> Result<()> {
    // Dry-run intercept FIRST — this MUST come before the confirmation prompt
    // so --dry-run never prompts (WR-01, matching batch_delete below).
    if ctx.dry_run {
        let url = format!("{}/api/v1/workflows/{}", ctx.client.base_url(), id);
        return dry_run::render_dry_run_delete(
            "workflow",
            id,
            &url,
            &ctx.output_format,
            ctx.color,
        );
    }

    match batch::ensure_delete_consent(
        ctx,
        args.force,
        format!("Delete workflow {}?", id),
        "Use --force to skip confirmation: pipelite workflows delete <id> --force".to_string(),
    )? {
        batch::Consent::Declined => return Ok(()),
        batch::Consent::Granted => {}
    }

    ctx.client.delete_workflow(id).await?;

    if let Some(ref cache) = ctx.cache {
        cache.invalidate(KEY_WORKFLOWS);
    }

    if !ctx.quiet {
        println!("Deleted workflow {}", id);
    }

    Ok(())
}
