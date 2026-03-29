use std::io::IsTerminal;

use anyhow::Result;

use crate::cache::KEY_WORKFLOWS;
use crate::cli::workflows::WorkflowsDeleteArgs;
use crate::context::AppContext;
use crate::dry_run;
use crate::error::CliError;

/// Delete a workflow by ID.
///
/// On a TTY without --force, prompts for confirmation using dialoguer::Confirm.
/// In non-interactive mode without --force, returns an error.
/// With --dry-run, prints what would be deleted without executing.
/// Prints a confirmation message unless --quiet is set.
pub async fn run(ctx: &AppContext, args: &WorkflowsDeleteArgs) -> Result<()> {
    // TTY confirmation check
    if !args.force {
        if std::io::stdin().is_terminal() && !ctx.no_input {
            let confirmed = dialoguer::Confirm::new()
                .with_prompt(format!("Delete workflow {}?", args.id))
                .default(false)
                .interact()?;
            if !confirmed {
                println!("Aborted");
                return Ok(());
            }
        } else {
            return Err(CliError::Validation {
                detail: "Refusing to delete without confirmation in non-interactive mode."
                    .to_string(),
                hint: "Use --force to skip confirmation: pipelite workflows delete <id> --force"
                    .to_string(),
            }
            .into());
        }
    }

    // Dry-run intercept
    if ctx.dry_run {
        let url = format!("{}/api/v1/workflows/{}", ctx.client.base_url(), args.id);
        return dry_run::render_dry_run_delete(
            "workflow",
            &args.id,
            &url,
            &ctx.output_format,
            ctx.color,
        );
    }

    ctx.client.delete_workflow(&args.id).await?;

    if let Some(ref cache) = ctx.cache {
        cache.invalidate(KEY_WORKFLOWS);
    }

    if !ctx.quiet {
        println!("Deleted workflow {}", args.id);
    }

    Ok(())
}
