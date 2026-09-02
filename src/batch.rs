use std::io::{self, IsTerminal, Read};

use anyhow::Result;

use crate::context::AppContext;
use crate::dry_run;
use crate::error::CliError;
use crate::output;

/// Tracks the outcome of a batch operation for summary reporting.
///
/// Used by batch update, batch delete, and (retroactively) batch create
/// handlers to collect successes/failures and produce a uniform summary.
///
/// Note: failure lines and the final summary are written to stderr and
/// intentionally bypass `--quiet`, so scripted runs always learn how many
/// operations failed. Per-item success output on stdout is quiet-aware.
pub struct BatchOutcome {
    pub total: usize,
    pub succeeded: usize,
    pub failed: usize,
}

impl BatchOutcome {
    /// Create a new BatchOutcome for a batch of `total` items.
    pub fn new(total: usize) -> Self {
        Self {
            total,
            succeeded: 0,
            failed: 0,
        }
    }

    /// Record a success.
    pub fn record_success(&mut self) {
        self.succeeded += 1;
    }

    /// Record a failure, printing the error to stderr.
    pub fn record_failure(&mut self, index: usize, id: &str, err: &dyn std::fmt::Display) {
        self.failed += 1;
        eprintln!("[{}/{}] Failed {}: {}", index + 1, self.total, id, err);
    }

    /// Print summary to stderr and return error if any failures (per D-06, D-07).
    ///
    /// Successes should already be rendered to stdout before calling this.
    /// Returns Ok(()) if all succeeded, Err with non-zero exit code if any failed.
    pub fn finalize(self, entity_name: &str, operation: &str) -> Result<()> {
        if self.failed > 0 {
            eprintln!(
                "{}/{} {} {}d, {} failed",
                self.succeeded, self.total, entity_name, operation, self.failed
            );
            return Err(CliError::Validation {
                detail: format!(
                    "{} of {} {} operations failed",
                    self.failed, self.total, operation
                ),
                hint: "Review the errors above and retry failed items.".to_string(),
            }
            .into());
        }
        Ok(())
    }
}

/// Read stdin fully and parse as JSON array. Returns a CliError::Validation on failure.
///
/// Caller must verify stdin is not a terminal before calling this.
pub fn read_stdin_json<T: serde::de::DeserializeOwned>(entity_hint: &str) -> Result<Vec<T>> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).map_err(|e| {
        CliError::Validation {
            detail: format!("Could not read stdin: {}", e),
            hint: "Ensure input is piped as UTF-8 text.".to_string(),
        }
    })?;

    serde_json::from_str(&input).map_err(|e| {
        CliError::Validation {
            detail: format!("Invalid JSON input: {}", e),
            hint: format!(
                "Stdin must contain a JSON array of {} objects.",
                entity_hint
            ),
        }
        .into()
    })
}

/// Collect delete IDs from positional args or `--stdin` (mutually exclusive, per D-04).
///
/// `cli_entity` is the CLI subcommand name used in hints (e.g. "deals");
/// `example` is the JSON example shown when stdin has no data piped in.
pub fn collect_delete_ids(
    cli_entity: &str,
    stdin: bool,
    raw_ids: &[String],
    example: &str,
) -> Result<Vec<String>> {
    if stdin {
        if !raw_ids.is_empty() {
            return Err(CliError::Validation {
                detail: "--stdin and positional IDs are mutually exclusive".to_string(),
                hint: "Use either positional IDs or --stdin, not both.".to_string(),
            }
            .into());
        }
        if io::stdin().is_terminal() {
            return Err(CliError::Validation {
                detail: "No data on stdin".to_string(),
                hint: format!(
                    "Pipe JSON data: echo '{example}' | pipelite {cli_entity} delete --stdin"
                ),
            }
            .into());
        }
        let ids: Vec<String> = read_stdin_json("string IDs")?;
        if ids.is_empty() {
            return Err(CliError::Validation {
                detail: "Empty ID list".to_string(),
                hint: "Provide at least one ID to delete.".to_string(),
            }
            .into());
        }
        return Ok(ids);
    }
    Ok(raw_ids.to_vec())
}

/// Shared batch delete flow: dry-run preview, confirmation gate, per-item
/// deletes with continue-on-error, conditional cache invalidation, summary.
///
/// `entity` is the display name used in prompts/output (e.g. "deal"),
/// `plural` is the prompt label (e.g. "deal(s)"), and `api_path` is the
/// REST path segment (e.g. "deals" or "organizations").
///
/// `delete_one` performs the per-ID delete (e.g. `client.delete_deal`).
pub async fn run_batch_delete<F>(
    ctx: &AppContext,
    entity: &str,
    plural: &str,
    force: bool,
    ids: &[String],
    api_path: &str,
    cache_key: &str,
    cache_prefix: Option<&str>,
    delete_one: F,
) -> Result<()>
where
    F: AsyncFn(&str) -> Result<()>,
{
    let total = ids.len();

    // FIRST: Dry-run check (per D-05) — show what would be deleted, no prompt needed.
    // This MUST come before the confirmation prompt so --dry-run never prompts.
    if ctx.dry_run {
        for id in ids {
            let url = format!("{}/api/v1/{}/{}", ctx.client.base_url(), api_path, id);
            dry_run::render_dry_run_delete(entity, id, &url, &ctx.output_format, ctx.color)?;
        }
        return Ok(());
    }

    // SECOND: Confirmation (CR-01) — prompt on an interactive terminal; REFUSE
    // in non-interactive mode unless --force was given. Stdin cannot serve as
    // both the ID source (--stdin) and the confirmation prompt, so piped batch
    // deletes require explicit --force, matching the workflows single-delete
    // precedent. Note ctx.no_input is auto-derived from TTY-ness, so only the
    // explicit --force flag counts as opt-out here.
    if !force {
        if io::stdin().is_terminal() && !ctx.no_input {
            let confirm = dialoguer::Confirm::new()
                .with_prompt(format!("Delete {} {}?", total, plural))
                .default(false)
                .interact()?;
            if !confirm {
                return Ok(());
            }
        } else {
            return Err(CliError::Validation {
                detail:
                    "Refusing to batch-delete without confirmation in non-interactive mode."
                        .to_string(),
                hint: "Re-run with --force to skip the confirmation prompt (intended for scripts)."
                    .to_string(),
            }
            .into());
        }
    }

    let mut outcome = BatchOutcome::new(total);

    for (i, id) in ids.iter().enumerate() {
        match delete_one(id).await {
            Ok(()) => {
                outcome.record_success();
                if !ctx.quiet {
                    println!("Deleted {} {}", entity, id);
                }
            }
            Err(e) => {
                outcome.record_failure(i, id, &e);
            }
        }
    }

    if outcome.succeeded > 0 {
        if let Some(ref cache) = ctx.cache {
            cache.invalidate(cache_key);
            if let Some(prefix) = cache_prefix {
                cache.invalidate_prefix(prefix);
            }
        }
    }

    outcome.finalize(entity, "delete")
}

/// Reject an item whose deserialized update payload is entirely empty (WR-03).
///
/// The Update models ignore unknown keys, so a misspelled field name (or an
/// item with no fields besides "id") deserializes to an all-`None` struct that
/// would PUT an empty `{}` body and report success while changing nothing.
/// Returns a per-item error so the batch loop records it as a failure.
///
/// `cli_entity` is the CLI subcommand name (e.g. "deals", "orgs") used in the
/// help hint — display names like "organization" are not runnable
/// subcommands (WR-08).
pub fn ensure_update_fields<T: Default + PartialEq>(data: &T, cli_entity: &str) -> Result<()> {
    if *data == T::default() {
        anyhow::bail!(
            "no recognizable update fields (unknown or misspelled fields are ignored; check `pipelite {cli_entity} update --help`)"
        );
    }
    Ok(())
}

/// Shared batch update flow for `--stdin` JSON-array updates (per D-02, D-03).
///
/// Reads objects from stdin, previews via `--dry-run`, then updates each item
/// with continue-on-error semantics (per D-06). Each JSON object must contain
/// an "id" field plus update fields; unknown fields are ignored by the
/// underlying Update model.
///
/// `entity` is the display name used in the summary (e.g. "deal"),
/// `cli_entity` is the CLI subcommand name used in hints (e.g. "deals",
/// "orgs" — it must be the runnable subcommand, not the REST `api_path`,
/// which may differ as "organizations"; WR-08), and `api_path` is the REST
/// path segment (e.g. "deals" or "organizations").
///
/// `update_one` receives the extracted id, the deserialized update payload,
/// and the raw JSON item, performs the API update, and returns the updated
/// entity as JSON for success rendering.
pub async fn run_batch_update<T, F>(
    ctx: &AppContext,
    entity: &str,
    cli_entity: &str,
    example: &str,
    api_path: &str,
    cache_key: &str,
    cache_prefix: Option<&str>,
    default_columns: &[&str],
    update_one: F,
) -> Result<()>
where
    T: serde::de::DeserializeOwned,
    F: AsyncFn(String, T, &serde_json::Value) -> Result<serde_json::Value>,
{
    if io::stdin().is_terminal() {
        return Err(CliError::Validation {
            detail: "No data on stdin".to_string(),
            hint: format!(
                "Pipe JSON data: echo '{example}' | pipelite {cli_entity} update --stdin"
            ),
        }
        .into());
    }

    let items: Vec<serde_json::Value> = read_stdin_json("update")?;

    // WR-03: an empty array would run zero operations and exit 0 with no
    // output — fail loudly instead, consistent with batch delete's
    // "Empty ID list" behavior.
    if items.is_empty() {
        return Err(CliError::Validation {
            detail: "Empty update list".to_string(),
            hint: "Stdin must contain at least one object with an 'id' field.".to_string(),
        }
        .into());
    }

    if ctx.dry_run {
        for item in &items {
            let id = item.get("id").and_then(|v| v.as_str()).unwrap_or("unknown");
            let url = format!("{}/api/v1/{}/{}", ctx.client.base_url(), api_path, id);
            dry_run::render_dry_run("PUT", &url, item, &ctx.output_format, ctx.color)?;
        }
        return Ok(());
    }

    let mut outcome = BatchOutcome::new(items.len());
    let mut succeeded = Vec::new();

    for (i, item) in items.into_iter().enumerate() {
        let id = match item.get("id").and_then(|v| v.as_str()) {
            Some(id) => id.to_string(),
            None => {
                outcome.record_failure(i, "unknown", &"missing 'id' field");
                continue;
            }
        };

        // Deserialize via the borrowed Value so the raw item stays available
        // for `update_one` (unknown keys — including "id" — are ignored).
        let data: T = match T::deserialize(&item) {
            Ok(d) => d,
            Err(e) => {
                outcome.record_failure(i, &id, &e);
                continue;
            }
        };

        match update_one(id.clone(), data, &item).await {
            Ok(updated) => {
                outcome.record_success();
                succeeded.push(updated);
            }
            Err(e) => {
                outcome.record_failure(i, &id, &e);
            }
        }
    }

    // Invalidate cache when anything succeeded; render successes to stdout
    // (per D-07) unless --quiet (WR-04). The finalize() summary on stderr
    // still reports "N ok, M failed" so scripted quiet runs stay informed.
    if !succeeded.is_empty() {
        if let Some(ref cache) = ctx.cache {
            cache.invalidate(cache_key);
            if let Some(prefix) = cache_prefix {
                cache.invalidate_prefix(prefix);
            }
        }
        if !ctx.quiet {
            let columns: Vec<String> = default_columns.iter().map(|s| s.to_string()).collect();
            output::render_list(
                &succeeded,
                &ctx.output_format,
                &columns,
                &None,
                ctx.color,
                None,
            )?;
        }
    }

    outcome.finalize(entity, "update")
}

/// Parse --custom-field key=value pairs into a serde_json::Value object.
///
/// Shared by the entities that support custom fields; previously duplicated
/// per entity with only the hint example differing.
pub fn parse_custom_fields(pairs: &[String]) -> Result<Option<serde_json::Value>> {
    if pairs.is_empty() {
        return Ok(None);
    }

    let mut map = serde_json::Map::new();
    for pair in pairs {
        let (key, value) = pair.split_once('=').ok_or_else(|| CliError::Validation {
            detail: format!("Invalid custom field format: '{}'", pair),
            hint: "Use key=value format: --custom-field industry=Tech".to_string(),
        })?;
        map.insert(key.to_string(), serde_json::Value::String(value.to_string()));
    }

    Ok(Some(serde_json::Value::Object(map)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn batch_outcome_counts_successes_and_failures() {
        let mut outcome = BatchOutcome::new(3);
        outcome.record_success();
        outcome.record_failure(1, "item_2", &"boom");
        assert_eq!(outcome.total, 3);
        assert_eq!(outcome.succeeded, 1);
        assert_eq!(outcome.failed, 1);
    }

    #[test]
    fn batch_outcome_finalize_ok_when_no_failures() {
        let mut outcome = BatchOutcome::new(1);
        outcome.record_success();
        assert!(outcome.finalize("deal", "update").is_ok());
    }

    #[test]
    fn batch_outcome_finalize_err_on_failure() {
        let mut outcome = BatchOutcome::new(2);
        outcome.record_success();
        outcome.record_failure(1, "item_2", &"boom");
        let err = outcome.finalize("deal", "delete");
        let err = err.expect_err("expected finalize to fail with failures present");
        let cli_err = err.downcast_ref::<CliError>().expect("expected CliError");
        match cli_err {
            CliError::Validation { detail, hint } => {
                assert_eq!(detail, "1 of 2 delete operations failed");
                assert!(hint.contains("retry failed items"));
            }
            other => panic!("expected Validation variant, got {:?}", other),
        }
    }

    #[test]
    fn parse_custom_fields_rejects_missing_equals() {
        let err = parse_custom_fields(&["oops".to_string()]);
        assert!(err.is_err());
    }

    #[test]
    fn parse_custom_fields_builds_object() {
        let fields =
            parse_custom_fields(&["industry=Tech".to_string(), "size=500".to_string()])
                .expect("valid pairs");
        let fields = fields.expect("expected Some object for non-empty pairs");
        let map = fields.as_object().expect("expected JSON object");
        assert_eq!(
            map.get("industry").and_then(|v| v.as_str()),
            Some("Tech")
        );
        assert_eq!(map.get("size").and_then(|v| v.as_str()), Some("500"));
    }
}
