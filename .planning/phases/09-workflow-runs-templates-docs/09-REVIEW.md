---
phase: 09-workflow-runs-templates-docs
reviewed: 2026-09-03T20:17:28Z
depth: standard
files_reviewed: 23
files_reviewed_list:
  - src/api/mod.rs
  - src/api/models.rs
  - src/cache.rs
  - src/cli/docs.rs
  - src/cli/mod.rs
  - src/cli/templates.rs
  - src/cli/workflows.rs
  - src/commands/docs.rs
  - src/commands/mod.rs
  - src/commands/templates/create.rs
  - src/commands/templates/delete.rs
  - src/commands/templates/get.rs
  - src/commands/templates/list.rs
  - src/commands/templates/mod.rs
  - src/commands/workflows/runs/mod.rs
  - src/commands/workflows/runs/list.rs
  - src/commands/workflows/runs/detail.rs
  - src/main.rs
  - tests/common/mod.rs
  - tests/docs_stub_test.rs
  - tests/help_examples_test.rs
  - tests/templates_stub_test.rs
  - tests/workflow_runs_stub_test.rs
findings:
  critical: 0
  warning: 2
  info: 9
  total: 11
status: issues_found
---

# Phase 9: Code Review Report

**Reviewed:** 2026-09-03T20:17:28Z
**Depth:** standard
**Files Reviewed:** 23 (21 in the `9463154..HEAD` diff plus `src/commands/workflows/runs/{mod,list}.rs`, which predate the range base but are part of the phase surface and were reviewed per scope)
**Status:** issues_found

## Summary

Reviewed the full phase 09 surface (workflow runs list/get/watch, templates CRUD, docs command, shared stub-test helpers) at standard depth, tracing call chains from CLI args through client methods to wire behavior and cross-checking every locked decision against the implementation and the phase planning docs. All 41 phase stub tests plus the models unit tests pass on the reviewed tree.

**Locked contracts verified as holding (adversarial pass):**
- `--include-dry-run` sends `dry_run=true` only when opted in (`src/api/mod.rs:1274-1287`), proven on the wire by `runs_list_omits_dry_run_param_by_default`.
- Required `--workflow` on runs get/watch is clap-enforced pre-HTTP (exit 2, zero requests, `workflow_runs_stub_test.rs:319-329`).
- Watch loop: fixed 2s poll (`detail.rs:70`), terminal set {completed, failed} plus defensive Unknown, `waiting` polls on, first observation silent, exactly one stderr transition line per change, `--quiet` suppresses transitions (`detail.rs:60`), exit 0 default / 1 under `--exit-status` on failed|Unknown / death by SIGINT (no handler; test asserts signal 2 → shell 130).
- Templates: `triggers[0] → trigger` mapping with the array never leaking into the POST body, multi-trigger stderr warning (quiet-suppressed), zero-trigger rejection exit 2 after exactly 1 fetch, hidden `update` → exit 2 `InvalidInput` with the delete-and-recreate hint before any HTTP.
- Delete ordering: dry-run preview FIRST, then confirmation (`templates/delete.rs:50-59`, `batch.rs:160-193`); non-TTY refusal is `Validation` → exit 1 with zero HTTP.
- Docs: `get_docs` builds a genuinely headerless local client (`src/api/mod.rs:1020-1030`, no `default_headers`, method body never touches the authenticated instance) — the wire test asserts no `authorization:` line in the raw request head. `--save` overwrite refusal fires pre-HTTP (0-request stub test), exit 2 without `--force`.
- No `unwrap()`/`expect()` in phase 09 production code; every produced `CliError` carries a hint.

No critical issues found. Two warnings (a silent flag drop in `templates create` and watch-loop fragility against transient poll errors) and nine info items below.

## Warnings

### WR-01: `templates create --workflow` silently ignores an explicit `--nodes` flag

**File:** `src/commands/templates/create.rs:58-80`
**Issue:** The `--workflow` branch returns `(first, wf.nodes.clone())` and never reads `args.nodes`, while the `--trigger` branch consumes it. Validation (`create.rs:28-51`) rejects `--stdin`+flags and `--workflow`+`--trigger`, but `--workflow`+`--nodes` parses, runs, and silently discards the user's nodes in favor of the workflow's snapshot. A user templating a workflow but overriding its nodes gets the workflow's nodes with no error or warning — silent input drop, inconsistent with the strict "exactly one source" rigor applied to the trigger.
**Fix:** Either reject the combination (matching the existing mutual-exclusion pattern) or honor `--nodes` as an override:

```rust
if has_workflow && args.nodes.is_some() {
    return Err(CliError::InvalidInput {
        detail: "--workflow and --nodes are mutually exclusive".to_string(),
        hint: "--workflow snapshots the workflow's own nodes; use --trigger <json> with --nodes <json> to supply custom nodes.".to_string(),
    }
    .into());
}
// ...or, override semantics in the --workflow branch:
let nodes = match &args.nodes {
    Some(raw) => Some(parse_json_array(raw)?),
    None => wf.nodes.clone(),
};
```

Whichever is chosen, document it in the `--nodes` flag help (`src/cli/templates.rs:123-125`).

### WR-02: Watch loop dies on a single transient poll error after an arbitrarily long wait

**File:** `src/commands/workflows/runs/detail.rs:71-74`
**Issue:** Inside the watch loop, the refetch propagates errors via `?` (`detail = ctx.client.get_workflow_run(...).await?`). The locked contract is "no timeout — watch until terminal state or Ctrl-C", and 09-02 targets unattended scripts consuming `--watch --exit-status`. One transient failure mid-watch (Wi-Fi blip, server restart, a 5s connect-timeout) aborts the entire watch with exit 1, no final state rendered, after potentially hours of polling — the same class of run-abort the defensive `Unknown` status was specifically designed to prevent. `send_with_retry` only covers 429, not connect/timeout failures.
**Fix:** Tolerate a bounded burst of consecutive transport failures before giving up:

```rust
let mut consecutive_errors = 0u32;
loop {
    // ... status/transition/terminal handling unchanged ...
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    detail = match ctx.client.get_workflow_run(&args.workflow, &args.run_id).await {
        Ok(d) => { consecutive_errors = 0; d }
        Err(e) if consecutive_errors < 3 => { consecutive_errors += 1; continue; }  // keep watching
        Err(e) => return Err(e),
    };
}
```

If abort-on-first-error is the deliberate v1 behavior, document it in the `--watch` flag help so scripted users know a network blip is fatal.

## Info

### IN-01: Unused `assert_cmd::Command` imports produce compiler warnings in phase test files

**File:** `tests/docs_stub_test.rs:10`, `tests/workflow_runs_stub_test.rs:19`
**Issue:** Both files `use assert_cmd::Command;` but only reference the type via return values of `common::cmd_with_server`, so each test binary compiles with `unused_imports` warnings. Related noise: `tests/common/mod.rs:31` `cmd()` warns as dead code whenever a binary that includes `common` doesn't use it (e.g. `docs_stub_test`).
**Fix:** Delete both unused imports. For `cmd()`, either use it in each test file (it already is, in two) or add `#[allow(dead_code)]` on the helper.

### IN-02: Stale `#[allow(dead_code)]` on now-wired watch building blocks

**File:** `src/api/models.rs:496,549,564`
**Issue:** `WorkflowRunStatus::as_str`, `run_status_is_terminal`, and `watch_exit_code` carry `#[allow(dead_code)]` from 09-01 (they were forward-declared for 09-02). All three are now used by `src/commands/workflows/runs/detail.rs`, so the attributes suppress genuine future dead-code detection on these items.
**Fix:** Remove the three `#[allow(dead_code)]` attributes.

### IN-03: `docs --save` overwrite protection has a check-then-write race (TOCTOU)

**File:** `src/commands/docs.rs:27-35,80`
**Issue:** The locked pre-HTTP `exists()` refusal is correct for its purpose (zero HTTP on refusal, pinned by test), but between the check and `fs::write` a file can appear at the path and be silently clobbered even without `--force`. Inherent to check-then-act; the window is small.
**Fix:** When `!args.force`, open with `std::fs::OpenOptions::new().write(true).create_new(true)` after the fetch (falling back to the existing error mapping on `AlreadyExists`) so the kernel enforces the refusal atomically.

### IN-04: Nullable run/step fields lack `#[serde(default)]` — brittle to server serialization changes

**File:** `src/api/models.rs:589-601,614-621`
**Issue:** `WorkflowRun.trigger_data/error/current_node_id/started_at/completed_at` and `WorkflowRunStep.input/output/error/resume_at/started_at/completed_at` are `Option` but have no `#[serde(default)]`, unlike `depth`/`dry_run` on the same struct. Serde requires the key to be present even for `Option`, so if the server ever switches to skip-null serialization every runs request fails with "Failed to parse response". The wire shape is verified today (serializer always emits all 11 fields), so this is forward-brittleness and internal inconsistency rather than a live bug.
**Fix:** Add `#[serde(default)]` to those `Option` fields (zero cost, matches the `depth`/`dry_run` precedent and the file's own `expanded` convention).

### IN-05: Non-UTF-8 stdin in `templates create --stdin` produces a hintless error

**File:** `src/commands/templates/create.rs:139`
**Issue:** `io::stdin().read_to_string(&mut input)?` propagates a raw io::Error (e.g. "stream did not contain valid UTF-8") through anyhow with no hint, violating the every-error-carries-a-hint convention. The batch path wraps the identical call with `InvalidInput` + hint (`src/batch.rs:74-80`).
**Fix:** Map the read error the same way `batch::read_stdin_json` does:

```rust
io::stdin().read_to_string(&mut input).map_err(|e| CliError::InvalidInput {
    detail: format!("Could not read stdin: {e}"),
    hint: "Pipe the template body as UTF-8 JSON text.".to_string(),
})?;
```

### IN-06: Hintless invariant fallback after `check_missing`

**File:** `src/commands/templates/create.rs:105`
**Issue:** `name.ok_or_else(|| anyhow::anyhow!("name missing after check_missing"))` is unreachable in practice (any `None` from `require_text` also pushes to `missing`, so `check_missing` errors first), but if the invariant ever breaks the user gets a hintless error.
**Fix:** Prefer a `debug_assert!` plus a `CliError::MissingInput`-shaped fallback with a hint, or reuse the usage-hint string in the `ok_or_else`.

### IN-07: Confirmation-decline UX differs between single and batch template delete

**File:** `src/commands/templates/delete.rs:68-71` vs `src/batch.rs:180-182`
**Issue:** Declining the single-delete prompt prints `Aborted` to stdout (unsuppressed by `--quiet`), while declining the batch prompt (`run_batch_delete`) returns `Ok(())` silently. Also, `Aborted` bypasses `ctx.quiet`, unlike every other non-essential stdout line in the phase.
**Fix:** Align both paths — either print an (quiet-aware) "Aborted" in `run_batch_delete`'s decline arm or drop the message from `single_delete`; gate it on `!ctx.quiet`.

### IN-08: Docs 429 re-wrap substitutes a misleading hint

**File:** `src/commands/docs.rs:49-58`
**Issue:** The error re-wrap catches every `CliError::Api`, including the double-429 rate-limit arm whose detail says "Rate limited: ...", and replaces its hint with "the server may not expose the docs endpoint — check server version". The detail stays accurate, but the hint would send a rate-limited user down the wrong diagnostic path. Locked plan wording ("NotFound/Api → docs hint") covers it, and the path is rare (docs route 429 twice in a row), hence Info.
**Fix:** Match `Ok(CliError::Api { status: 429, .. })` separately and pass it through untouched (the generic rate-limit hint is already actionable).

### IN-09: Test helper sets a dead `PIPELITE_URL` env var

**File:** `tests/common/mod.rs:22`
**Issue:** `cmd_with_server` sets `PIPELITE_URL`, which `src/config.rs` never reads (env overrides are `PIPELITE_SERVER_URL` and `PIPELITE_API_KEY`, config.rs:107-110). Harmless (the second line does the work) but misleading for future test authors.
**Fix:** Remove the `c.env("PIPELITE_URL", url);` line or rename it to the real override key.

---

_Reviewed: 2026-09-03T20:17:28Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: standard_
