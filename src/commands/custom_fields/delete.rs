use anyhow::Result;

use crate::batch;

use crate::cli::custom_fields::CustomFieldsDeleteArgs;
use crate::commands::custom_fields::TOMBSTONE_NOTE;
use crate::context::AppContext;
use crate::dry_run;

/// Delete a single custom field definition (SOFT delete).
///
/// Mirrors templates/webhooks delete EXACTLY — the standard delete
/// contract:
/// 1. Dry-run intercept FIRST (--dry-run never prompts, zero HTTP);
/// 2. TTY `dialoguer::Confirm` (default false), "Aborted" on decline;
/// 3. non-TTY without --force → CliError::Validation (exit 1) refusal —
///    deliberately the STANDARD contract, unlike 11-02 purge's stricter
///    InvalidInput exit-2 refusal;
/// 4. delete → invalidate the custom_fields_ cache prefix →
///    quiet-suppressed confirmation + tombstone reminder.
///
/// The delete only sets deletedAt server-side — values already stored on
/// records remain. Re-deleting an already-deleted definition 404s through
/// the standard NotFound path (no special casing).
pub async fn run(ctx: &AppContext, args: &CustomFieldsDeleteArgs) -> Result<()> {
    let id = &args.definition_id;

    // Dry-run intercept FIRST — before the confirmation prompt so --dry-run
    // never prompts (matching templates/webhooks delete).
    if ctx.dry_run {
        let url = format!(
            "{}/api/v1/custom-field-definitions/{}",
            ctx.client.base_url(),
            id
        );
        return dry_run::render_dry_run_delete(
            "custom field definition",
            id,
            &url,
            &ctx.output_format,
            ctx.color,
        );
    }

    // TTY confirmation check
    match batch::ensure_delete_consent(
        ctx,
        args.force,
        format!(
            "Delete custom field definition {id}? Values already stored on records remain."
        ),
        "Use --force to skip confirmation: pipelite custom-fields delete <id> --force".to_string(),
    )? {
        batch::Consent::Declined => return Ok(()),
        batch::Consent::Granted => {}
    }

    ctx.client.delete_custom_field_definition(id).await?;

    // Invalidate ALL per-entity definition caches — the type source for
    // typed --custom-field writing must never go stale after a mutation.
    if let Some(ref cache) = ctx.cache {
        cache.invalidate_prefix("custom_fields_");
    }

    if !ctx.quiet {
        println!("Deleted custom field definition {id}");
        println!("{TOMBSTONE_NOTE}");
    }

    Ok(())
}
