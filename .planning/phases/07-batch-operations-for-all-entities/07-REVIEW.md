---
phase: 07-batch-operations-for-all-entities
reviewed: 2026-09-02T00:00:00Z
depth: standard
files_reviewed: 28
files_reviewed_list:
  - src/batch.rs
  - src/cli/activities.rs
  - src/cli/deals.rs
  - src/cli/orgs.rs
  - src/cli/people.rs
  - src/cli/pipelines.rs
  - src/cli/stages.rs
  - src/cli/workflows.rs
  - src/commands/activities/delete.rs
  - src/commands/activities/update.rs
  - src/commands/deals/delete.rs
  - src/commands/deals/update.rs
  - src/commands/orgs/delete.rs
  - src/commands/orgs/update.rs
  - src/commands/people/delete.rs
  - src/commands/people/update.rs
  - src/commands/pipelines/delete.rs
  - src/commands/pipelines/update.rs
  - src/commands/stages/delete.rs
  - src/commands/stages/update.rs
  - src/commands/workflows/delete.rs
  - src/commands/workflows/update.rs
  - src/main.rs
  - tests/batch_cli_stub_test.rs
  - tests/batch_delete_test.rs
  - tests/batch_error_test.rs
  - tests/batch_update_test.rs
findings:
  critical: 2
  warning: 5
  info: 4
  total: 11
status: issues_found
---

# Phase 07: Code Review Report

**Reviewed:** 2026-09-02T00:00:00Z
**Depth:** standard
**Files Reviewed:** 28
**Status:** issues_found

## Summary

Phase 07 adds batch update (`--stdin` JSON array) and batch delete (multi-ID + `--stdin` JSON array of IDs) across all 7 entities, plus a shared `BatchOutcome` helper. The core mechanics are solid: continue-on-error semantics work, `--dry-run` is (mostly) honored before prompts, cache invalidation is conditional on success, IDs are extracted without `unwrap()`, and batch deserialization against the Update models works (no `deny_unknown_fields`, so the extra `"id"` key is ignored).

The two serious problems are both in the destructive/consistency domain: (1) the batch-delete confirmation prompt is **unreachable in the `--stdin` flow by construction** — `collect_ids` requires stdin to be a pipe, while the prompt requires stdin to be a TTY — so piping IDs deletes everything immediately with zero friction, directly contradicting the doc comments and the safer pattern the workflows single-delete path itself implements; and (2) activities batch update silently drops `"completed_at": null` (serde maps it to `None`, which is skipped), reporting success while never marking the activity undone.

Cross-file verification performed against `src/api/models.rs` (serde attrs), `src/output/mod.rs` (render signatures), `src/api/mod.rs` (client methods), and `src/config.rs` (env vars). All referenced functions exist; no broken call chains found.

## Critical Issues

### CR-01: Batch delete via `--stdin` never prompts for confirmation (all 7 entities)

**File:** `src/commands/deals/delete.rs:94` (same pattern in `orgs/delete.rs:100`, `people/delete.rs:94`, `activities/delete.rs:94`, `pipelines/delete.rs:101`, `stages/delete.rs:101`, `workflows/delete.rs:124`)

**Issue:** The confirmation prompt gate is `!ctx.no_input && io::stdin().is_terminal()`, but `collect_ids` (lines 28–56) requires stdin to be **non-terminal** whenever `--stdin` is used. The two conditions are mutually exclusive, so the confirmation prompt is dead code for the entire `--stdin` flow: `echo '["deal_1","deal_2"]' | pipelite deals delete --stdin` deletes every record immediately, with no prompt, no `--force`, exit 0. This contradicts:

- The doc comment on `run()` (lines 12–16): "run a batch delete with confirmation prompt" — false for half the advertised surface (the `after_help` in `src/cli/deals.rs:80` actively promotes the `--stdin` flow).
- The codebase's own safety intent: `workflows/delete.rs:72-80` (single delete) *refuses* to delete in non-interactive mode without `--force`. Yet workflows batch delete produces an absurd boundary: `echo '["wf_1"]' | pipelite workflows delete --stdin` fails (1 ID → `single_delete` → requires `--force`), while `echo '["wf_1","wf_2"]' | pipelite workflows delete --stdin` proceeds silently (2 IDs → `batch_delete` → prompt skipped). Adding more IDs makes deletion *easier*, not harder.
- CLAUDE.md's data-safety posture: this is an irrecoverable destructive operation gated on nothing.

This is a data-loss risk: one mis-piped file (e.g., a full `deals list --format json` instead of an ID array, or a list with hundreds of stale IDs) deletes production CRM records with zero confirmation.

**Fix:** Require explicit opt-out for non-interactive batch deletes, matching the workflows single-delete pattern. For all 7 `batch_delete` functions:

```rust
// SECOND: Confirmation — prompt on TTY; refuse in non-interactive mode
// unless --force (or --no-input) was given.
if io::stdin().is_terminal() && !ctx.no_input {
    let confirm = dialoguer::Confirm::new()
        .with_prompt(format!("Delete {} deal(s)?", total))
        .default(false)
        .interact()?;
    if !confirm {
        return Ok(());
    }
} else if !(args.force || ctx.no_input) {
    return Err(CliError::Validation {
        detail: "Refusing to batch-delete without confirmation in non-interactive mode.".to_string(),
        hint: "Re-run with --force to skip confirmation, or use --no-input for scripted runs.".to_string(),
    }.into());
}
```

This requires adding a `--force` flag to the delete args of the 6 non-workflow entities (`src/cli/{deals,orgs,people,activities,pipelines,stages}.rs`) and makes the workflows 1-ID vs 2-ID boundary consistent. Also correct the `run()` doc comments.

### CR-02: Activities batch update silently drops `"completed_at": null` — reports success without marking undone

**File:** `src/commands/activities/update.rs:263-269` (with `src/api/models.rs:250-267`)

**Issue:** Single-mode `--mark-undone` exists specifically because `ActivityUpdate` cannot express "send null" (`skip_serializing_if = "Option::is_none"`, see `update_with_null_completed`, lines 146–200). Batch mode has no equivalent: `serde_json::from_value::<ActivityUpdate>(item)` maps `"completed_at": null` to `None`, the field is then skipped during serialization, and the API request never clears `completed_at`. The command then reports success (D-07 rendering + exit code). So:

```bash
echo '[{"id":"act_1","completed_at":null}]' | pipelite activities update --stdin
# → "success", but the activity is still marked done
```

A requested mutation is silently not performed while the tool claims it was — the worst failure mode for a batch tool, since the user proceeds believing the state changed.

**Fix:** Either support it via the raw endpoint (mirror single mode), or explicitly reject it. Minimal safe fix — detect and fail the item:

```rust
let clears_completed_at = item.get("completed_at").map_or(false, |v| v.is_null());
let data: ActivityUpdate = match serde_json::from_value(item) { /* ... */ };
if clears_completed_at {
    match ctx.client.update_activity_raw(&id, &serde_json::json!({"completed_at": null})).await {
        // ... or merge with other fields into a raw payload
    }
}
```

At minimum, reject with a per-item failure: `outcome.record_failure(i, &id, &"\"completed_at\": null is not supported in batch mode; use --mark-undone per item")`.

## Warnings

### WR-01: Workflows single delete prompts for confirmation *before* dry-run

**File:** `src/commands/workflows/delete.rs:60-93`

**Issue:** In `single_delete`, the confirmation prompt (lines 62–81) runs before the `ctx.dry_run` intercept (lines 84–93). `pipelite workflows delete wf_1 --dry-run` on a TTY first asks "Delete workflow wf_1?" — answering "No" aborts without ever showing the preview. CLAUDE.md requires respecting `--dry-run` (pure preview, no side effects, no prompts), and `batch_delete` in the *same file* (lines 112–120) correctly checks dry-run first with a comment stating "This MUST come before the confirmation prompt so --dry-run never prompts." The single path violates its own file's invariant. All 6 other entities order this correctly.

**Fix:** Move the `ctx.dry_run` block above the confirmation block in `single_delete`, matching `batch_delete`:

```rust
async fn single_delete(ctx: &AppContext, args: &WorkflowsDeleteArgs, id: &str) -> Result<()> {
    // Dry-run intercept FIRST — never prompt for a preview.
    if ctx.dry_run { /* render and return */ }
    // THEN confirmation...
}
```

### WR-02: Workflows update interactive prompt can deactivate a workflow by pressing Enter

**File:** `src/commands/workflows/update.rs:77-87`

**Issue:** When updating interactively without flags, the "Set workflow active?" prompt uses `.default(false)` and maps the answer directly: `Some(confirmed)`. Pressing Enter (accepting the default "No") or answering "No" sends `active: false` to the API — **deactivating the workflow** — when the user plausibly meant "leave it alone". This turns a routine rename into a silent production-automation outage. The pipelines handler in this same phase gets it right (`src/commands/pipelines/update.rs:62-72`): `if confirm { Some(true) } else { None }` — "No" means don't touch the field.

**Fix:**

```rust
let confirmed = dialoguer::Confirm::new()
    .with_prompt("Set workflow active?")
    .default(false)
    .interact()?;
// "No"/Enter → leave `active` unchanged, mirroring pipelines/update.rs
let active = if confirmed { Some(true) } else { None };
```

(If deliberately deactivating interactively is desired, ask "Set workflow active? (No = leave unchanged)" and provide `--active <bool>` as the explicit path.)

### WR-03: Batch update silently no-ops: empty arrays and items with no known fields both "succeed"

**File:** `src/commands/deals/update.rs:166-180` (same in all 7 `batch_update` functions: `orgs/update.rs:139-153`, `people/update.rs:157-171`, `activities/update.rs:235-249`, `pipelines/update.rs:121-135`, `stages/update.rs:135-149`, `workflows/update.rs:142-156`)

**Issue:** Two related validation gaps, both silent successes:

1. `echo '[]' | pipelite deals update --stdin` parses fine, the loop never runs, `finalize` sees zero failures → exit 0, no output at all. Compare the delete path, which explicitly rejects empty input ("Empty ID list" + hint, `deals/delete.rs:46-52`), and single update, which errors headless with "No fields to update" (`deals/update.rs:61-67`).
2. The Update models lack `deny_unknown_fields`, so a misspelled field is silently discarded: `[{"id":"deal_1","titel":"New"}]` deserializes to an all-`None` `DealUpdate`, serializes to an empty `{}` PUT body, gets a success response, and is reported as updated — nothing changed. Same for an item that is only `{"id":"deal_1"}`.

A batch tool should never claim success while doing nothing.

**Fix:** After parsing, validate both conditions per entity:

```rust
if items.is_empty() {
    return Err(CliError::Validation {
        detail: "Empty update list".to_string(),
        hint: "Stdin must contain at least one object with an 'id' field.".to_string(),
    }.into());
}
// Per item, before the API call:
if data == DealUpdate::default() {  // derive PartialEq + Default
    outcome.record_failure(i, &id, &"no recognizable update fields");
    continue;
}
```

### WR-04: Batch update ignores `--quiet`

**File:** `src/commands/deals/update.rs:213-237` (same in all 7 `batch_update` functions)

**Issue:** Batch delete carefully gates its per-item `println!` on `!ctx.quiet` (e.g., `deals/delete.rs:110-112`), but batch update unconditionally renders the full success list via `output::render_list` (which has no quiet awareness — `src/output/mod.rs:44-62` takes only format/columns/color). `pipelite deals update --stdin --quiet < input.json` prints the entire success table, violating the CLAUDE.md convention "Respect ... `--quiet`". Inconsistent within the same phase.

**Fix:** Gate the success rendering: `if !succeeded.is_empty() && !ctx.quiet { ... }` (or short-circuit earlier). Keep failures on stderr, which is reasonable to always show.

### WR-05: ~800 lines of copy-pasted batch scaffolding across 7 entities, with divergence already occurring

**File:** `src/commands/{deals,orgs,people,activities,pipelines,stages,workflows}/{update,delete}.rs`

**Issue:** `collect_ids`, `batch_update` (stdin check → read → parse → dry-run loop → process loop → render → finalize), `batch_delete` (dry-run → prompt → loop → invalidate → finalize), and `parse_custom_fields` are duplicated near-verbatim 7 times. The duplication has already produced the behavioral drift found in this review: workflows' delete/prompt logic differs from the other six (WR-01, CR-01's inconsistent `--force` boundary), and `parse_custom_fields` exists in 4 copies with only the hint example differing. The next entity added will copy whichever variant is at hand. `src/batch.rs` exists precisely to centralize this but only owns `BatchOutcome` and `read_stdin_json`.

**Fix:** Extract the shared flow into `src/batch.rs` as generic helpers, e.g. `batch::run_json_updates<T: DeserializeOwned, F: AsyncFn>(&Ctx, entity: &str, url_for: fn(&str) -> String, update_fn: F)` and `batch::collect_ids(entity: &str, stdin: bool, raw: &[String], example: &str)`, leaving only the per-entity model type and client call in each command file. This makes fixes for CR-01/WR-03/WR-04 one-line changes instead of seven.

## Info

### IN-01: Batch failure/summary output is not gated on `--quiet`

**File:** `src/batch.rs:31-34, 41-45`

**Issue:** `record_failure` and `finalize` unconditionally `eprintln!`. Errors on stderr surviving `--quiet` is defensible (and probably desirable), but the behavior is undocumented and inconsistent with the strictly-gated delete `println!`s. Suggest documenting in the `BatchOutcome` doc comment that failure output intentionally bypasses `--quiet`, or gating only the summary line.

### IN-02: Stdin read errors propagate without hint text

**File:** `src/batch.rs:65`; also `src/commands/deals/update.rs:163-164` and the 6 sibling `batch_update` functions

**Issue:** `io::stdin().read_to_string(&mut input)?` surfaces I/O failures (e.g., invalid UTF-8: "stream did not contain valid UTF-8") as bare `anyhow` errors with no `hint`, violating the CLAUDE.md convention "All errors must include actionable hint text". Parse errors right below are handled correctly.

**Fix:** Wrap: `.map_err(|e| CliError::Validation { detail: format!("Could not read stdin: {e}"), hint: "Ensure input is piped as UTF-8 text.".to_string() })?` — ideally inside `batch::read_stdin_json` and reuse it from all 7 update paths (it already exists; the update handlers just don't use it).

### IN-03: Batch update dry-run shows `"id"` in the PUT body, which the real request omits

**File:** `src/commands/deals/update.rs:173-179` (same in all 7 `batch_update` dry-run loops)

**Issue:** The dry-run renders the raw stdin item (`dry_run::render_dry_run("PUT", &url, item, ...)`), including the `"id"` key, but the executed request deserializes into the Update model and strips unknown keys. The preview slightly misrepresents the payload.

**Fix:** In the dry-run loop, deserialize into the Update type first and render `serde_json::to_value(&data)?`, so preview and reality match (and WR-03's no-op detection could share this step).

### IN-04: Test suite: duplicated coverage, dead env var, near-tautological assertions

**File:** `tests/batch_cli_stub_test.rs:12-28`; `tests/batch_delete_test.rs:12`, `tests/batch_update_test.rs:12`, `tests/batch_error_test.rs:64`

**Issue:** (a) `batch_cli_stub_test.rs` re-tests the two deals `--help` cases already covered by `batch_update_test.rs:21-27` and `batch_delete_test.rs:21-27`. (b) All `cmd()` helpers set `PIPELITE_URL`, which the app never reads — the comments admit `PIPELITE_SERVER_URL` is the real variable; the dead line misleads future readers. (c) `batch_error_test.rs:64` asserts stderr contains `"missing"` **or** `"id"` — `"id"` matches almost any output, making the content assertion nearly vacuous (the `.failure()` check carries the test).

**Fix:** Delete the stub file; drop `PIPELITE_URL` from all helpers (or fix the comment); tighten the predicate to `contains("missing 'id' field")`.

---

_Not flagged, verified fine:_ ID extraction avoids `unwrap()` per project convention (`unwrap_or`/`ok_or_else` only); `--stdin` + positional/mutual-exclusivity checks present in all 14 handlers; clap `required_unless_present = "stdin"` correctly enforces ID presence; cache invalidation correctly conditional on success and includes `stages_` prefix invalidation for pipeline/stage mutations; `main.rs` unchanged behavior is sound; unit tests for `BatchOutcome` cover both finalize paths.

_Reviewed: 2026-09-02T00:00:00Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: standard_
