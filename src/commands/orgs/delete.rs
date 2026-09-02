use anyhow::Result;

use crate::batch;
use crate::cache::KEY_ORGS;
use crate::cli::orgs::OrgsDeleteArgs;
use crate::context::AppContext;
use crate::dry_run;

/// Delete one or more organizations.
///
/// Single ID executes the original delete flow (v1.0 behavior). Multiple IDs
/// (or --stdin with a JSON array of string IDs) run a batch delete with
/// confirmation prompt, continue-on-error semantics, and a summary report.
/// Non-interactive runs (e.g. piped --stdin) must pass --force: stdin cannot
/// carry both the IDs and a confirmation prompt.
pub async fn run(ctx: &AppContext, args: &OrgsDeleteArgs) -> Result<()> {
    let ids = batch::collect_delete_ids("orgs", args.stdin, &args.ids, r#"["org_1","org_2"]"#)?;

    if ids.len() == 1 {
        return single_delete(ctx, &ids[0]).await;
    }

    batch::run_batch_delete(
        ctx,
        "organization",
        "organization(s)",
        args.force,
        &ids,
        "organizations",
        KEY_ORGS,
        None,
        async |id: &str| ctx.client.delete_org(id).await,
    )
    .await
}

/// Delete a single organization (original behavior).
async fn single_delete(ctx: &AppContext, id: &str) -> Result<()> {
    if ctx.dry_run {
        let url = format!("{}/api/v1/organizations/{}", ctx.client.base_url(), id);
        return dry_run::render_dry_run_delete(
            "organization",
            id,
            &url,
            &ctx.output_format,
            ctx.color,
        );
    }

    ctx.client.delete_org(id).await?;

    if let Some(ref cache) = ctx.cache {
        cache.invalidate(KEY_ORGS);
    }

    if !ctx.quiet {
        println!("Deleted organization {}", id);
    }

    Ok(())
}
