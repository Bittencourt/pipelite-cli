use anyhow::Result;

use crate::batch;
use crate::cache::KEY_PEOPLE;
use crate::cli::people::PeopleDeleteArgs;
use crate::context::AppContext;
use crate::dry_run;

/// Delete one or more people.
///
/// A single positional ID executes the original delete flow (v1.0 behavior).
/// Multiple IDs — or --stdin with a JSON array of string IDs, even a 1-ID
/// list — run a batch delete with confirmation prompt, continue-on-error
/// semantics, and a summary report. Non-interactive runs (e.g. piped --stdin)
/// must pass --force: stdin cannot carry both the IDs and a confirmation
/// prompt.
pub async fn run(ctx: &AppContext, args: &PeopleDeleteArgs) -> Result<()> {
    let ids = batch::collect_delete_ids("people", args.stdin, &args.ids, r#"["per_1","per_2"]"#)?;

    // WR-06: piped --stdin input always takes the batch path (with its
    // --force gate) regardless of item count; only a single positional ID
    // keeps the v1.0 gate-free flow.
    if ids.len() == 1 && !args.stdin {
        return single_delete(ctx, &ids[0]).await;
    }

    batch::run_batch_delete(
        ctx,
        "person",
        "person(s)",
        args.force,
        &ids,
        "people",
        KEY_PEOPLE,
        None,
        async |id: &str| ctx.client.delete_person(id).await,
    )
    .await
}

/// Delete a single person (original behavior).
async fn single_delete(ctx: &AppContext, id: &str) -> Result<()> {
    if ctx.dry_run {
        let url = format!("{}/api/v1/people/{}", ctx.client.base_url(), id);
        return dry_run::render_dry_run_delete("person", id, &url, &ctx.output_format, ctx.color);
    }

    ctx.client.delete_person(id).await?;

    if let Some(ref cache) = ctx.cache {
        cache.invalidate(KEY_PEOPLE);
    }

    if !ctx.quiet {
        println!("Deleted person {}", id);
    }

    Ok(())
}
