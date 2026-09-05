# Phase 1: Batch Operations for All Entities - Research

**Researched:** 2026-03-29
**Domain:** Rust CLI batch operations (stdin JSON, continue-on-error, multi-ID delete)
**Confidence:** HIGH

## Summary

This phase extends existing batch create patterns to batch update and batch delete across all 7 entity types. The codebase already has a well-established `--stdin` pattern for batch create in all entities, and a consistent CRUD pattern per entity. The main work is: (1) adding `--stdin` to update subcommands with an ID-in-payload pattern, (2) changing delete to accept multiple positional IDs plus `--stdin`, (3) adding a confirmation prompt for batch delete, and (4) standardizing continue-on-error with summary reporting across all batch operations (including retroactively to existing batch create).

The API does not have batch update or batch delete endpoints -- only batch create for deals, orgs, and people (`/api/v1/{entity}/batch`). The other 4 entities (activities, pipelines, stages, workflows) already use individual-create loops for batch create (see `pipelines/create.rs`). All batch update and batch delete operations will use individual API call loops with continue-on-error semantics.

**Primary recommendation:** Follow the existing pipelines batch_create pattern (individual API call loop with error collection) for all batch update and batch delete operations. Extract a shared batch execution utility to avoid duplicating the loop+error-collection logic across 14 new command paths.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** JSON only via `--stdin`. No CSV or NDJSON support. Consistent with the existing batch create pattern that all entities already use.
- **D-02:** Reuse the existing `update` subcommand with `--stdin` flag (same pattern as create). No new subcommands.
- **D-03:** Each JSON object in the stdin array includes an `"id"` field plus the fields to patch. Example: `[{"id": "deal_1", "title": "New Title"}, ...]`. Self-contained, mirrors single-update pattern.
- **D-04:** Accept multiple positional IDs: `pipelite deals delete id1 id2 id3`. Also support `--stdin` for large ID lists (JSON array of strings).
- **D-05:** Batch delete shows confirmation prompt: "Delete N [entity]s? [y/N]". Respects `--no-input` (skips prompt, proceeds) and `--dry-run` (shows what would be deleted without executing).
- **D-06:** Continue-on-error for all batch operations. Process all items, collect errors, then report summary: "N/M succeeded, K failed".
- **D-07:** Successful items rendered to stdout normally (respecting --format flag). Error summary printed to stderr. Exit code 0 if all succeed, non-zero if any fail.

### Claude's Discretion
- Specific exit code value for partial failure (e.g., exit 1 vs a custom code)
- Whether to apply continue-on-error retroactively to existing batch create operations
- Internal implementation patterns (trait-based batch handling, shared batch utilities, etc.)

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope
</user_constraints>

## Project Constraints (from CLAUDE.md)

- No `unwrap()` in production code -- use `?` with `anyhow::Result`
- All errors must include actionable `hint` text
- Respect `--dry-run` (no HTTP calls), `--no-input` (no stdin prompts), `--quiet`, `--no-color`
- Tests use fake credentials (`127.0.0.1:1` + `fake-test-key`)
- Every entity follows the same CRUD pattern

## Architecture Patterns

### Current Codebase Structure (relevant files)

```
src/
  cli/
    deals.rs           # DealsCreateArgs has --stdin; DealsUpdateArgs, DealsDeleteArgs do NOT
    orgs.rs            # Same pattern -- --stdin only on Create
    people.rs          # Same
    activities.rs      # Same
    pipelines.rs       # Same
    stages.rs          # Same
    workflows.rs       # Same
  commands/
    deals/
      create.rs        # batch_create() -- uses batch API endpoint
      update.rs        # single update only
      delete.rs        # single delete only
    pipelines/
      create.rs        # batch_create() -- individual-create loop with error collection
    (other entities follow same single-operation pattern)
  api/mod.rs           # batch_create_deals/orgs/people (POST /batch). No batch update/delete endpoints.
  dry_run.rs           # render_dry_run() and render_dry_run_delete()
  error.rs             # CliError with detail+hint, exit_code() returns 1 or 2
  prompt.rs            # No confirmation prompt exists yet (only text/number/select prompts)
```

### Pattern 1: Batch Update via --stdin

**What:** Add `--stdin` flag to all 7 UpdateArgs structs. When set, read JSON array from stdin where each object contains `"id"` plus update fields. Loop through items, call individual `update_<entity>` API methods.

**When to use:** `pipelite deals update --stdin < updates.json`

**Key implementation detail:** The UpdateArgs struct currently has a positional `id: String` field. When `--stdin` is active, this field is not needed. Use clap's `required_unless_present = "stdin"` to make `id` optional when `--stdin` is provided. Alternatively, change `id` to `Option<String>` and validate in the handler.

**Stdin JSON shape (per D-03):**
```json
[
  {"id": "deal_1", "title": "New Title"},
  {"id": "deal_2", "value": 50000}
]
```

**Deserialization approach:** Deserialize stdin as `Vec<serde_json::Value>`, extract `"id"` from each object, then deserialize the remainder into the entity's `Update` struct. This avoids creating a new `BatchUpdate` model type per entity.

### Pattern 2: Batch Delete via Multiple Positional IDs + --stdin

**What:** Change `id: String` to `ids: Vec<String>` in all 7 DeleteArgs structs, using `num_args = 1..` to require at least one. Add `--stdin` for JSON array of string IDs.

**When to use:** `pipelite deals delete id1 id2 id3` or `echo '["id1","id2"]' | pipelite deals delete --stdin`

**Key implementation detail:** With multiple IDs, the command must show a confirmation prompt (D-05) before proceeding. Use `dialoguer::Confirm` gated behind `!ctx.no_input && stdin().is_terminal()`. When `--no-input`, skip prompt and proceed. When `--dry-run`, show what would be deleted and return.

### Pattern 3: Continue-on-Error Batch Execution

**What:** A shared utility function that executes a batch of async operations, collects successes and failures, renders results to stdout, and prints error summary to stderr.

**Recommendation:** Create a `src/batch.rs` module with a generic batch executor:

```rust
pub struct BatchResult<T> {
    pub succeeded: Vec<T>,
    pub failed: Vec<(usize, String, anyhow::Error)>, // (index, id, error)
}

pub async fn execute_batch<T, F, Fut>(
    items: Vec<(String, F)>,  // (id/label, operation)
) -> BatchResult<T>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = anyhow::Result<T>>,
```

This avoids duplicating the loop+error-collection+summary logic across 14 command paths (7 entities x 2 operations).

**Exit code for partial failure:** Use exit code 1 (same as full failure). No custom exit code needed -- the stderr summary communicates partial vs. full failure. This aligns with existing `exit_code()` in `error.rs`.

### Pattern 4: Retroactive Continue-on-Error for Batch Create

**Recommendation:** YES, apply continue-on-error to existing batch create. Currently:
- `deals/create.rs` batch_create uses the batch API endpoint (`POST /deals/batch`) which sends all items in one request -- the API itself handles partial failure. No change needed here.
- `pipelines/create.rs` batch_create already implements continue-on-error with an individual loop. But its error reporting is ad-hoc (eprintln per error). Standardize it to use the shared batch utility.
- Other entities' batch create should be checked and standardized similarly.

### Anti-Patterns to Avoid
- **Don't create per-entity BatchUpdate model types.** Use `serde_json::Value` to extract `id`, then pass remaining fields to existing Update structs. This keeps the model layer clean.
- **Don't make batch operations parallel (concurrent API calls).** The API may rate-limit. Sequential calls are correct and simpler. If performance becomes a concern, it can be addressed later with `futures::stream::buffered`.
- **Don't read stdin in the Delete handler when positional IDs are provided.** The `--stdin` and positional IDs should be mutually exclusive (same pattern as `--stdin` vs flags in create).

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Confirmation prompts | Custom y/n reader | `dialoguer::Confirm` | Already a dependency, handles edge cases (non-TTY, ctrl+c) |
| JSON deserialization with extra `id` field | Manual JSON parsing | `serde_json::Value` + `serde_json::from_value` | Extract id, deserialize rest into existing Update struct |
| Colored stderr output | Manual ANSI codes | `colored` crate (already used) + `eprintln!` | Consistent with existing error display |

## Common Pitfalls

### Pitfall 1: Clap Positional Args vs --stdin Conflict in Delete
**What goes wrong:** Clap requires at least one positional `id` for delete. When `--stdin` is used, no positional ID is provided, causing a clap parse error before the handler runs.
**Why it happens:** Current `id: String` is a required positional arg.
**How to avoid:** Change to `ids: Vec<String>` with `num_args = 0..` or use `required_unless_present = "stdin"`. Validate in the handler that at least one source of IDs is provided (positional or stdin).
**Warning signs:** `cargo test` shows clap error exit code 2 for `pipelite deals delete --stdin`.

### Pitfall 2: Stdin Already Consumed
**What goes wrong:** Stdin can only be read once. If the command handler checks `is_terminal()` and then tries to read stdin later, or if both update and delete try to read stdin in the same invocation.
**Why it happens:** Stdin is a stream, not a seekable buffer.
**How to avoid:** Read stdin exactly once, early in the handler, when `--stdin` is set. Store the parsed result.
**Warning signs:** Empty stdin reads, "broken pipe" errors.

### Pitfall 3: no_input and Confirmation Prompt Interaction
**What goes wrong:** `ctx.no_input` is true when stdin is piped (non-TTY). But batch delete with `--stdin` pipes data to stdin, which makes `no_input` true. The confirmation prompt would be skipped automatically -- which is actually the correct behavior per D-05.
**Why it happens:** `no_input` is derived from `cli.no_input || !stdin().is_terminal()`.
**How to avoid:** This is actually fine -- when piping JSON via stdin, stdin is not a TTY, so no_input is true, and confirmation is skipped. When using positional IDs, stdin IS a TTY, so confirmation can be shown. Document this clearly.
**Warning signs:** None -- this is the desired behavior.

### Pitfall 4: Update --stdin Makes id Field Optional
**What goes wrong:** The UpdateArgs struct has `pub id: String` as a required positional arg. With `--stdin`, no positional ID is needed (IDs are in the JSON payload).
**Why it happens:** Clap derives require positional args by default.
**How to avoid:** Change `id: String` to `id: Option<String>` or `ids: Vec<String>` and validate in the handler. Alternatively, use `#[arg(required_unless_present = "stdin")]`.
**Warning signs:** Clap parse errors when running `pipelite deals update --stdin`.

### Pitfall 5: Exit Code After Batch Operations
**What goes wrong:** The `main()` function uses `error::exit_code()` to determine exit code from errors. Batch operations with partial failure should return non-zero, but the current flow expects either Ok(()) or Err(anyhow::Error).
**Why it happens:** Partial success is a new concept not modeled in the current error flow.
**How to avoid:** Return `Err(CliError::BatchPartialFailure { ... })` after rendering successful items to stdout. Or use `std::process::exit(1)` explicitly after printing results. The former is cleaner -- add a new `CliError` variant or use a custom error.
**Warning signs:** Exit code 0 when some operations failed.

## Code Examples

### Batch Update Handler Pattern (deals example)
```rust
// In src/commands/deals/update.rs
async fn batch_update(ctx: &AppContext) -> Result<()> {
    if io::stdin().is_terminal() {
        return Err(CliError::Validation {
            detail: "No data on stdin".to_string(),
            hint: "Pipe JSON data via stdin: echo '[{\"id\":\"deal_1\",\"title\":\"New\"}]' | pipelite deals update --stdin".to_string(),
        }.into());
    }

    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let items: Vec<serde_json::Value> = serde_json::from_str(&input)
        .map_err(|e| CliError::Validation {
            detail: format!("Invalid JSON input: {}", e),
            hint: "Stdin must contain a JSON array of objects with 'id' field.".to_string(),
        })?;

    // Dry-run: show each payload
    if ctx.dry_run {
        for item in &items {
            let id = item.get("id").and_then(|v| v.as_str()).unwrap_or("unknown");
            let url = format!("{}/api/v1/deals/{}", ctx.client.base_url(), id);
            dry_run::render_dry_run("PUT", &url, item, &ctx.output_format, ctx.color)?;
        }
        return Ok(());
    }

    let total = items.len();
    let mut succeeded = Vec::new();
    let mut errors = Vec::new();

    for (i, item) in items.into_iter().enumerate() {
        let id = match item.get("id").and_then(|v| v.as_str()) {
            Some(id) => id.to_string(),
            None => {
                eprintln!("[{}/{}] Missing 'id' field, skipping", i + 1, total);
                errors.push(format!("Item {}: missing 'id' field", i + 1));
                continue;
            }
        };

        let data: DealUpdate = match serde_json::from_value(item.clone()) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("[{}/{}] Invalid update data for {}: {}", i + 1, total, id, e);
                errors.push(format!("{}: {}", id, e));
                continue;
            }
        };

        match ctx.client.update_deal(&id, &data).await {
            Ok(deal) => succeeded.push(deal),
            Err(e) => {
                eprintln!("[{}/{}] Failed to update {}: {}", i + 1, total, id, e);
                errors.push(format!("{}: {}", id, e));
            }
        }
    }

    // Render successes to stdout
    if !succeeded.is_empty() {
        // ... render_list ...
        if let Some(ref cache) = ctx.cache {
            cache.invalidate(KEY_DEALS);
        }
    }

    // Summary to stderr
    if !errors.is_empty() {
        eprintln!("{}/{} succeeded, {} failed", succeeded.len(), total, errors.len());
        // Return error for non-zero exit code
        return Err(CliError::Validation {
            detail: format!("{} of {} updates failed", errors.len(), total),
            hint: "Review the errors above and retry failed items.".to_string(),
        }.into());
    }

    Ok(())
}
```

### Batch Delete Handler Pattern (deals example)
```rust
async fn batch_delete(ctx: &AppContext, ids: &[String]) -> Result<()> {
    let total = ids.len();

    // Confirmation prompt (D-05)
    if !ctx.dry_run && !ctx.no_input && io::stdin().is_terminal() {
        let confirm = dialoguer::Confirm::new()
            .with_prompt(format!("Delete {} deal(s)?", total))
            .default(false)
            .interact()?;
        if !confirm {
            return Ok(());
        }
    }

    // Dry-run: show what would be deleted
    if ctx.dry_run {
        for id in ids {
            dry_run::render_dry_run_delete("deal", id,
                &format!("{}/api/v1/deals/{}", ctx.client.base_url(), id),
                &ctx.output_format, ctx.color)?;
        }
        return Ok(());
    }

    let mut succeeded = 0;
    let mut errors = Vec::new();

    for (i, id) in ids.iter().enumerate() {
        match ctx.client.delete_deal(id).await {
            Ok(()) => {
                succeeded += 1;
                if !ctx.quiet {
                    println!("Deleted deal {}", id);
                }
            }
            Err(e) => {
                eprintln!("[{}/{}] Failed to delete {}: {}", i + 1, total, id, e);
                errors.push(format!("{}: {}", id, e));
            }
        }
    }

    if !errors.is_empty() {
        if let Some(ref cache) = ctx.cache {
            cache.invalidate(KEY_DEALS);
        }
        eprintln!("{}/{} deleted, {} failed", succeeded, total, errors.len());
        return Err(CliError::Validation {
            detail: format!("{} of {} deletes failed", errors.len(), total),
            hint: "Review the errors above and retry failed items.".to_string(),
        }.into());
    }

    if let Some(ref cache) = ctx.cache {
        cache.invalidate(KEY_DEALS);
    }

    Ok(())
}
```

### Clap Args Changes (DeleteArgs example)
```rust
#[derive(Args)]
pub struct DealsDeleteArgs {
    /// Deal ID(s) to delete
    #[arg(
        add = ArgValueCandidates::new(deal_id_candidates),
        required_unless_present = "stdin",
    )]
    pub ids: Vec<String>,

    /// Read JSON array of IDs from stdin
    #[arg(long)]
    pub stdin: bool,
}
```

### Clap Args Changes (UpdateArgs example)
```rust
#[derive(Args)]
pub struct DealsUpdateArgs {
    /// Deal ID (not used with --stdin)
    #[arg(
        add = ArgValueCandidates::new(deal_id_candidates),
        required_unless_present = "stdin",
    )]
    pub id: Option<String>,

    // ... existing flags ...

    /// Read JSON array from stdin for batch update
    #[arg(long)]
    pub stdin: bool,
}
```

## Shared Batch Utility Recommendation

Rather than duplicating the batch loop + error collection + summary rendering across 14 handlers (7 update + 7 delete), extract a shared module `src/batch.rs`:

```rust
use anyhow::Result;
use crate::error::CliError;

pub struct BatchOutcome {
    pub total: usize,
    pub succeeded: usize,
    pub failed: usize,
}

impl BatchOutcome {
    pub fn is_partial_failure(&self) -> bool {
        self.failed > 0
    }

    /// Print summary to stderr and return error if any failures
    pub fn finalize(self, entity_name: &str, operation: &str) -> Result<()> {
        if self.failed > 0 {
            eprintln!(
                "{}/{} {} {}d, {} failed",
                self.succeeded, self.total, entity_name, operation, self.failed
            );
            return Err(CliError::Validation {
                detail: format!("{} of {} {} operations failed", self.failed, self.total, operation),
                hint: "Review the errors above and retry failed items.".to_string(),
            }.into());
        }
        Ok(())
    }
}
```

This keeps the individual handlers focused on entity-specific logic (deserialization, API calls) while centralizing the reporting pattern.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | assert_cmd 2.0 + predicates 3.0 (integration tests) |
| Config file | Cargo.toml `[dev-dependencies]` |
| Quick run command | `cargo test` |
| Full suite command | `cargo test` |

### Phase Requirements to Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| D-01 | --stdin accepts JSON for batch update | integration | `cargo test batch_update_stdin -x` | No -- Wave 0 |
| D-02 | update --stdin reuses update subcommand | integration | `cargo test update_help -x` | Partial (help tests exist) |
| D-03 | Update JSON includes id field | integration | `cargo test batch_update_missing_id -x` | No -- Wave 0 |
| D-04 | Delete accepts multiple positional IDs | integration | `cargo test delete_multiple_ids -x` | No -- Wave 0 |
| D-04 | Delete accepts --stdin for IDs | integration | `cargo test delete_stdin -x` | No -- Wave 0 |
| D-05 | Delete confirmation prompt + --no-input + --dry-run | integration | `cargo test delete_batch_dry_run -x` | No -- Wave 0 |
| D-06 | Continue-on-error collects all results | integration | `cargo test batch_partial_failure -x` | No -- Wave 0 |
| D-07 | Successes to stdout, errors to stderr, exit code non-zero on failure | integration | `cargo test batch_exit_code -x` | No -- Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test`
- **Per wave merge:** `cargo test`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `tests/batch_update_test.rs` -- covers D-01, D-02, D-03 (dry-run update --stdin, help text, missing id validation)
- [ ] `tests/batch_delete_test.rs` -- covers D-04, D-05 (multiple positional IDs, --stdin IDs, dry-run, --no-input)
- [ ] `tests/batch_error_test.rs` -- covers D-06, D-07 (partial failure exit code, stderr output)

Note: Integration tests can only test CLI args, dry-run output, and help text without a real API. Continue-on-error with actual failures requires either a mock server or the existing pattern of using unreachable server addresses to trigger connection errors.

## Scope of Changes Summary

### Files to Modify (per entity, x7)
1. `src/cli/<entity>.rs` -- Add `--stdin` to UpdateArgs, change DeleteArgs `id` to `ids: Vec<String>` + `--stdin`
2. `src/commands/<entity>/update.rs` -- Add batch_update pathway
3. `src/commands/<entity>/delete.rs` -- Add multi-ID + batch_delete pathway with confirmation

### New Files
4. `src/batch.rs` -- Shared batch execution utilities (BatchOutcome, stdin reading helpers)

### Files to Modify (shared)
5. `src/main.rs` -- Add `mod batch;`
6. `src/error.rs` -- Optionally add `BatchPartialFailure` variant (or reuse `Validation`)
7. `src/dry_run.rs` -- Possibly add `render_dry_run_delete_batch` for multiple deletes (or loop existing)

### Test Files to Create
8. `tests/batch_update_test.rs`
9. `tests/batch_delete_test.rs`
10. `tests/batch_error_test.rs`

## Open Questions

1. **DealUpdate struct and the `id` field**
   - What we know: The stdin JSON contains `{"id": "deal_1", "title": "New"}`. The `DealUpdate` struct does NOT have an `id` field (it is only the patch fields).
   - What's unclear: Will `serde_json::from_value` fail when an unexpected `id` field is present? Serde's default is to ignore unknown fields, so it should work. But we should verify that `#[serde(deny_unknown_fields)]` is NOT set on Update structs.
   - Recommendation: Verify at implementation time. If deny_unknown_fields is set, strip the `id` key from the Value before deserializing.

2. **Batch create for deals/orgs/people uses batch API endpoint**
   - What we know: `batch_create_deals` sends all items to `POST /deals/batch` in one request. The API handles partial failure internally.
   - What's unclear: Does the API return partial results on failure? Or does it all-or-nothing?
   - Recommendation: Leave batch create for these 3 entities as-is. They already work. If we want uniform continue-on-error, we could switch them to individual loops, but that would be slower. Leave as-is unless the API's error behavior is problematic.

## Sources

### Primary (HIGH confidence)
- `src/commands/deals/create.rs` -- batch_create pattern with batch API endpoint
- `src/commands/pipelines/create.rs` -- batch_create pattern with individual loop + error collection
- `src/commands/deals/update.rs` -- single update handler pattern
- `src/commands/deals/delete.rs` -- single delete handler pattern
- `src/cli/deals.rs` -- CLI arg definitions with --stdin pattern
- `src/error.rs` -- CliError variants and exit code logic
- `src/dry_run.rs` -- dry-run rendering utilities
- `src/context.rs` -- AppContext with dry_run, no_input, quiet flags
- `docs/SKILL.md` -- Entity CRUD pattern documentation

### Secondary (MEDIUM confidence)
- Clap `required_unless_present` attribute -- verified in clap 4.x docs

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- no new dependencies needed, all tools already in Cargo.toml
- Architecture: HIGH -- extending well-established existing patterns with code evidence
- Pitfalls: HIGH -- identified from direct code inspection of existing patterns

**Research date:** 2026-03-29
**Valid until:** 2026-04-28 (stable domain, no external dependency changes expected)
