---
phase: 09-workflow-runs-templates-docs
fixed_at: 2026-09-03T20:33:28Z
review_path: .planning/phases/09-workflow-runs-templates-docs/09-REVIEW.md
iteration: 1
findings_in_scope: 4
fixed: 4
skipped: 0
status: all_fixed
---

# Phase 9: Code Review Fix Report

**Fixed at:** 2026-09-03T20:33:28Z
**Source review:** .planning/phases/09-workflow-runs-templates-docs/09-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 4 (WR-01, WR-02, plus trivial fold-ins IN-01 and IN-02)
- Fixed: 4
- Skipped: 0

**Final full-suite gate:** `cargo test --no-fail-fast` — 371 passed, 0 failed, 0 ignored (28 test binaries). `cargo check --tests` clean apart from two pre-existing, out-of-scope warnings (`tests/common/mod.rs` `cmd()` conditional dead-code under `docs_stub_test`; `WorkflowRunTrigger` never constructed).

## Fixed Issues

### WR-01: `templates create --workflow` silently ignores an explicit `--nodes` flag

**Files modified:** `src/commands/templates/create.rs`, `src/cli/templates.rs`, `tests/templates_stub_test.rs`
**Commit:** 691bd0b
**Applied fix:** Chose the REJECT option (safer than override precedence surprises, per review guidance): a new mutual-exclusion guard in `templates create` returns `CliError::InvalidInput` (exit 2) BEFORE any HTTP when `--workflow` and `--nodes` are combined, with the hint "--nodes cannot be combined with --workflow (the template copies the workflow's nodes); use --trigger <json> with --nodes <json> to supply custom nodes." Documented the behavior in the `--nodes` flag help (`src/cli/templates.rs`) and the `run()` doc comment. Added pre-HTTP stub test `create_workflow_with_nodes_rejected_pre_http` (unreachable server `127.0.0.1:1`, asserts exit 2 + both message fragments + zero "Connection failed").

### WR-02: Watch loop dies on a single transient poll error after an arbitrarily long wait

**Files modified:** `src/commands/workflows/runs/detail.rs`, `src/cli/workflows.rs`, `tests/common/mod.rs`, `tests/workflow_runs_stub_test.rs`
**Commit:** de11961
**Applied fix:** The `--watch` refetch no longer propagates via `?`. A `consecutive_failures` counter tolerates up to 3 consecutive poll failures (`MAX_CONSECUTIVE_POLL_FAILURES = 3`): each failure prints one stderr warning (`warning: run <id>: poll failed (<err>); retrying (n/3)`, suppressed under `--quiet`) and polling continues on the locked 2s interval; any successful poll resets the counter. The 4th consecutive failure aborts with `CliError::Validation` (exit 1) carrying detail + an actionable re-run hint. Terminal-state exit contract unchanged. The `--watch` flag help documents the new tolerance. Test support: the shared stub server gained a status-`0` scripted transport-failure convention (accept + read + count, then drop the connection without responding). Three new integration tests through the real binary: `watch_survives_a_transient_poll_failure_then_completes` (drop → warning → final detail rendered, exit 0, 4 requests), `watch_gives_up_after_persistent_poll_failures` (4 consecutive drops → exit 1 + hint, 5 requests), `watch_quiet_suppresses_poll_failure_warnings`. All 11 pre-existing watch tests (locked contract) still pass.

### IN-01: Unused `assert_cmd::Command` imports in phase-09 test files

**Files modified:** `tests/docs_stub_test.rs`, `tests/workflow_runs_stub_test.rs`
**Commit:** ce23b08
**Applied fix:** Deleted both unused `use assert_cmd::Command;` imports (docs_stub_test.rs:10, workflow_runs_stub_test.rs:19) — both files use commands only via `common::cmd_with_server`/`common::cmd` return values. Both `unused_imports` warnings are gone. Note: the related `tests/common/mod.rs` `cmd()` conditional dead-code warning (IN-01 "Related noise") was outside the requested fold-in scope and is left untouched.

### IN-02: Stale `#[allow(dead_code)]` on now-wired watch building blocks

**Files modified:** `src/api/models.rs`
**Commit:** ce23b08
**Applied fix:** Removed the three stale `#[allow(dead_code)]` attributes from `WorkflowRunStatus::as_str`, `run_status_is_terminal`, and `watch_exit_code` (all wired into `src/commands/workflows/runs/detail.rs`). Compiler-confirmed: `cargo check --tests` reports no dead-code warnings for these items, so genuine future dead-code detection on them is restored.

## Skipped Issues

None — all in-scope findings were fixed.

---

_Fixed: 2026-09-03T20:33:28Z_
_Fixer: the agent (gsd-code-fixer)_
_Iteration: 1_
