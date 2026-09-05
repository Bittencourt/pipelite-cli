---
phase: 07-batch-operations-for-all-entities
plan: 01
subsystem: cli-batch-operations
tags: [batch, deals, stdin, continue-on-error, cli]
requires:
  - src/api/mod.rs (update_deal, delete_deal, base_url)
  - src/error.rs (CliError::Validation)
  - src/dry_run.rs (render_dry_run, render_dry_run_delete)
  - src/output (render_list)
provides:
  - src/batch.rs (BatchOutcome, read_stdin_json) — shared batch utility consumed by Plans 02-03
  - deals batch update (--stdin, id-in-payload) as reference implementation
  - deals batch delete (multi-ID + --stdin + confirmation) as reference implementation
affects:
  - Plan 07-02 (orgs/people/activities replicate this pattern)
  - Plan 07-03 (pipelines/stages/workflows replicate this pattern)
  - Plan 07-04 (integration tests build on these handlers)
tech-stack:
  added: []
  patterns:
    - BatchOutcome continue-on-error loop with stderr per-item errors + stdout success rendering after
    - Dry-run check before confirmation prompt (D-05 ordering)
    - required_unless_present clap pattern for positional-or---stdin args
key-files:
  created:
    - src/batch.rs
    - tests/batch_cli_stub_test.rs
  modified:
    - src/main.rs
    - src/cli/deals.rs
    - src/commands/deals/update.rs
    - src/commands/deals/delete.rs
decisions:
  - BatchOutcome records failures to stderr immediately; successes buffered and rendered to stdout after the loop (D-06/D-07)
  - Batch delete confirmation only on TTY without --no-input; --dry-run checked BEFORE prompt so it never prompts (D-05)
  - --stdin and positional args/flags are mutually exclusive on update and delete (mirrors create.rs)
  - mod batch placed alphabetically between mod api and mod cache (plan's stated intent)
metrics:
  duration: 17 min
  completed: 2026-09-02T11:04:18Z
  tasks: 3
  files: 6
---

# Phase 7 Plan 1: Batch Utility + Deals Reference Implementation Summary

Shared batch utility module (BatchOutcome + read_stdin_json) with deals batch update (--stdin, id-in-payload, continue-on-error) and batch delete (multi-ID + --stdin + TTY confirmation) as the canonical pattern for Plans 02-03.

## Tasks Completed

| Task | Name | Commit | Files |
| ---- | ---- | ------ | ----- |
| 1 | Batch utility module, deals CLI args, batch test stubs | bc7f8e1 | src/batch.rs, src/main.rs, src/cli/deals.rs, tests/batch_cli_stub_test.rs, src/commands/deals/update.rs, src/commands/deals/delete.rs |
| 2 | Deals batch update handler | 42ad82f | src/commands/deals/update.rs |
| 3 | Deals batch delete handler with confirmation | 68b2a46 | src/commands/deals/delete.rs |

## What Was Built

- **src/batch.rs** — `BatchOutcome` (new/record_success/record_failure/finalize): per-item failures print `[i/n] Failed <id>: <err>` to stderr immediately; `finalize` prints `N/M deal updated, K failed` and returns `CliError::Validation` (exit 1) when any failed. `read_stdin_json<T>` parses stdin as a JSON array with Validation error on bad input. Includes 3 unit tests.
- **Deals CLI** — `DealsUpdateArgs.id` is now `Option<String>` with `required_unless_present = "stdin"` plus `--stdin` flag; `DealsDeleteArgs.id` became `ids: Vec<String>` (multi-ID positional, same required_unless_present) plus `--stdin`; after_help examples updated for both.
- **Batch update** — `deals update --stdin` reads a JSON array, extracts `id` per object, deserializes the rest as `DealUpdate` (no `deny_unknown_fields`, so the `id` key is ignored), calls `update_deal` per item with continue-on-error; successes rendered via `render_list` after the loop; dry-run renders one PUT per item.
- **Batch delete** — `deals delete d1 d2 d3` or `--stdin` with JSON string array; single ID keeps original behavior; batch path checks dry-run FIRST (never prompts), then `dialoguer::Confirm` ("Delete N deal(s)?", default false) only on TTY without `--no-input`; cache invalidated once if any succeeded.

## Test Results

- `cargo test --test batch_cli_stub_test` — 2 passed (`--stdin` present in deals update/delete `--help`)
- `cargo test --bin pipelite batch` — 3 passed (BatchOutcome unit tests)
- Full suite `cargo test` (with scratch HOME, the suite's designed no-config environment) — **all pass, 0 failures** (101 unit + integration tests)
- Functional smoke: multi-ID dry-run renders N DELETEs; `--stdin` JSON array renders N DELETEs; empty list → exit 1 with hint; stdin+flags → mutual-exclusivity error exit 1; single-ID paths unchanged
- `cargo check` — clean (only 4 pre-existing warnings: workflows/list.rs unused assign, WorkflowRunTrigger, TTL_WORKFLOWS, get_workflows_cached)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Minimal handler compile fixes in Task 1 commit**
- **Found during:** Task 1
- **Issue:** Changing `DealsUpdateArgs.id` to `Option` and `DealsDeleteArgs.id` to `Vec` broke the existing update/delete handlers (6 compile errors), but Task 1's done criteria requires `cargo check` to pass.
- **Fix:** Task 1 commit included the minimal adaptations: update.rs extracts `id` via `ok_or_else` + CliError::Validation (exactly Task 2 step 2), delete.rs uses `args.ids.first()` with a Validation error. Full batch handlers still landed in their own tasks/commits.
- **Files modified:** src/commands/deals/update.rs, src/commands/deals/delete.rs (in commit bc7f8e1)
- **Commit:** bc7f8e1

**2. [Rule 2 - Missing validation] --stdin vs field-flags mutual exclusivity on batch update**
- **Found during:** Task 2
- **Issue:** Plan's update code checked `args.stdin` first without rejecting conflicting field flags; the codebase precedent (create.rs) and D-04 spirit require mutual exclusivity — silently ignoring flags would mislead users.
- **Fix:** Added the same exclusivity check used by `deals create --stdin` before dispatching to `batch_update`.
- **Files modified:** src/commands/deals/update.rs
- **Commit:** 42ad82f

**3. [Rule 1 - Bug] Fixed my own BatchOutcome unit-test assertion**
- **Found during:** Task 1
- **Issue:** Added unit test asserted on `format!("{:#}", err)`, but `CliError::Validation`'s Display is only "Validation error" (detail/hint live in fields, shown by display_error).
- **Fix:** Test now downcasts to `CliError` and asserts the `detail`/`hint` fields directly.
- **Files modified:** src/batch.rs
- **Commit:** bc7f8e1

### Plan-intent resolution
- Plan said to add `mod batch;` "after mod cache" but also "alphabetical order" — placed between `mod api;` and `mod cache;` per the stated alphabetical intent.

## Requirements Note

BATCH-01..BATCH-04 are phase-level requirements also claimed by Plans 02-04 (per-entity coverage and integration tests). They were intentionally **not** marked complete — this plan delivers the shared utility + deals reference implementation only. Recommend marking BATCH-01..03 complete after Plan 03/04.

## Deferred Issues

See `deferred-items.md` in this directory: three pre-existing environment-dependent tests (`deals_list_limit_zero_is_accepted`, `config_set_parses_positional_args`, `global_flags_parse_without_error`) fail when a real `~/.pipelite/config.toml` exists — verified pre-existing at commit 0990737, unrelated to this plan. Side effect noted: the `config set` test writes to the real user config when present (user config was restored during execution).

## Known Stubs

None — all handlers fully implemented and wired.

## Self-Check: PASSED

- src/batch.rs exists — FOUND
- tests/batch_cli_stub_test.rs exists — FOUND
- Commits bc7f8e1, 42ad82f, 68b2a46 — FOUND in git log
- Acceptance criteria per task verified against source (grep) and test runs
