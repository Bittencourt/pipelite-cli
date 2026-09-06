use anyhow::Result;

use crate::batch;
use crate::cache::KEY_TEMPLATES;
use crate::cli::templates::TemplatesDeleteArgs;
use crate::context::AppContext;
use crate::dry_run;

/// Delete one or more workflow templates.
///
/// Mirrors the workflows delete flow exactly: single ID executes the
/// single-delete flow (shared consent gate, --force skips), multiple IDs (or
/// --stdin with a JSON array of string IDs) route through the Phase 7 batch
/// delete (confirmation, continue-on-error, summary report). The dry-run
/// preview comes FIRST — --dry-run never prompts.
///
/// Templates are deployment-global and deletion is a hard delete, so the
/// confirmation prompt is the only gate; help states the shared-resource
/// consequence and the prompt shows the id.
pub async fn run(ctx: &AppContext, args: &TemplatesDeleteArgs) -> Result<()> {
    let ids = batch::collect_delete_ids("templates", args.stdin, &args.ids, r#"["tpl_1","tpl_2"]"#)?;

    if ids.len() == 1 {
        return single_delete(ctx, args, &ids[0]).await;
    }

    batch::run_batch_delete(
        ctx,
        "workflow template",
        "workflow template(s)",
        args.force,
        &ids,
        "workflow-templates",
        KEY_TEMPLATES,
        None,
        async |id: &str| ctx.client.delete_workflow_template(id).await,
    )
    .await
}

/// Delete a single workflow template (original behavior, --force skips
/// confirmation). Non-interactive refusal is CliError::Validation (exit 1),
/// matching workflows delete and the Phase 7 batch deletes.
async fn single_delete(ctx: &AppContext, args: &TemplatesDeleteArgs, id: &str) -> Result<()> {
    // Dry-run intercept FIRST — this MUST come before the confirmation prompt
    // so --dry-run never prompts (matching workflows delete and batch deletes).
    if ctx.dry_run {
        let url = format!("{}/api/v1/workflow-templates/{}", ctx.client.base_url(), id);
        return dry_run::render_dry_run_delete(
            "workflow template",
            id,
            &url,
            &ctx.output_format,
            ctx.color,
        );
    }

    match batch::ensure_delete_consent(
        ctx,
        args.force,
        format!("Delete template {id}?"),
        "Use --force to skip confirmation: pipelite templates delete <id> --force".to_string(),
    )? {
        batch::Consent::Declined => return Ok(()),
        batch::Consent::Granted => {}
    }

    ctx.client.delete_workflow_template(id).await?;

    if let Some(ref cache) = ctx.cache {
        cache.invalidate(KEY_TEMPLATES);
    }

    if !ctx.quiet {
        println!("Deleted workflow template {id}");
    }

    Ok(())
}
