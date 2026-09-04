use std::io::IsTerminal;

use anyhow::Result;

use crate::cache::KEY_WEBHOOKS;
use crate::cli::webhooks::WebhooksDeleteArgs;
use crate::context::AppContext;
use crate::dry_run;
use crate::error::CliError;

/// Delete a single webhook.
///
/// Mirrors templates delete (single_delete) EXACTLY — the standard delete
/// contract:
/// 1. Dry-run intercept FIRST (--dry-run never prompts, zero HTTP);
/// 2. TTY `dialoguer::Confirm` (default false), "Aborted" on decline;
/// 3. non-TTY without --force → CliError::Validation (exit 1) refusal —
///    deliberately the STANDARD contract, unlike 11-02 purge's stricter
///    InvalidInput exit-2 refusal;
/// 4. delete → invalidate KEY_WEBHOOKS → quiet-suppressed confirmation.
pub async fn run(ctx: &AppContext, args: &WebhooksDeleteArgs) -> Result<()> {
    let id = &args.webhook_id;

    // Dry-run intercept FIRST — before the confirmation prompt so --dry-run
    // never prompts (matching templates delete and batch deletes).
    if ctx.dry_run {
        let url = format!("{}/api/v1/webhooks/{}", ctx.client.base_url(), id);
        return dry_run::render_dry_run_delete(
            "webhook",
            id,
            &url,
            &ctx.output_format,
            ctx.color,
        );
    }

    // TTY confirmation check
    if !args.force {
        if std::io::stdin().is_terminal() && !ctx.no_input {
            let confirmed = dialoguer::Confirm::new()
                .with_prompt(format!("Delete webhook {id}?"))
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
                hint: "Use --force to skip confirmation: pipelite webhooks delete <id> --force"
                    .to_string(),
            }
            .into());
        }
    }

    ctx.client.delete_webhook(id).await?;

    if let Some(ref cache) = ctx.cache {
        cache.invalidate(KEY_WEBHOOKS);
    }

    if !ctx.quiet {
        println!("Deleted webhook {id}");
    }

    Ok(())
}
