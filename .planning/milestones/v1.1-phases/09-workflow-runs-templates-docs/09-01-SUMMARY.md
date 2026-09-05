---
phase: 09-workflow-runs-templates-docs
plan: 01
subsystem: workflow-runs
tags: [workflows, runs, api-client, cli, output-rendering, stub-tests]
requires:
  - Phase 8 error layer (parse_rfc7807, Forbidden hints, send_with_retry)
  - output renderers (render_list/render_single, comfy-table Dynamic arrangement)
provides:
  - WorkflowRun/WorkflowRunStep/WorkflowRunDetail models with typed 5- and 6-value status enums + defensive Unknown fallback
  - list_workflow_runs / get_workflow_run client methods + WorkflowRunsListParams (conditional status/dry_run query pairs)
  - pipelite workflows runs list|get command group (required --workflow)
  - step_duration duration helper + run_status_is_terminal + watch_exit_code (09-02 watch building blocks)
  - tests/common/mod.rs head-capturing stub server (shared by 09-02/09-03 test files)
affects:
  - 09-02 (watch loop reuses render_detail seam, watch_exit_code, common helpers)
  - 09-03 (docs/templates tests reuse tests/common/mod.rs)
tech-stack:
  added: []
  patterns:
    - serde(other) Unknown fallback enum for forward-safe wire statuses
    - one-probe empty-result hint pattern (limit-1 dry_run=true probe, meta.total-gated)
    - render-seam separation (render_detail) so watch re-renders without refetching
key-files:
  created:
    - src/commands/workflows/runs/mod.rs
    - src/commands/workflows/runs/list.rs
    - src/commands/workflows/runs/detail.rs
    - tests/common/mod.rs
    - tests/workflow_runs_stub_test.rs
  modified:
    - src/api/models.rs
    - src/api/mod.rs
    - src/cli/workflows.rs
    - src/commands/workflows/mod.rs
    - tests/help_examples_test.rs
decisions:
  - "Probe hint carries the user's --status by construction (else-branch implies status=None); statuses hint takes precedence so no probe fires on status-filtered empty pages"
  - "step_duration uses to_text_en(Rough, Present) — HumanTime Display renders positive deltas as 'in 5 minutes' (relative-to-now tense), wrong for durations"
  - "WorkflowRunStatus::as_str / run_status_is_terminal / watch_exit_code land now with #[allow(dead_code)] — wired up by the 09-02 watch loop"
metrics:
  duration: 13 min
  completed: 2026-09-03T19:19:37Z
  tasks: 3
  files: 10
---

# Phase 9 Plan 01: Workflow Runs Read Surface Summary

**One-liner:** Workflow-runs list/get with typed 5/6-value status enums, honest server-side dry-run hiding (`dry_run=true` only on `--include-dry-run`), locked empty-result hints backed by a single probe, flattened steps table with computed durations, and verbatim JSON passthrough — proven by 10 head-capturing stub tests.

## What Was Built

- **Task 1 — Runs contracts** (TDD: e33e956 RED → 0edf328 GREEN): `WorkflowRun`/`WorkflowRunStep`/`WorkflowRunDetail` models matching the verified wire shapes exactly (11 fields each; serializer-excluded `context`/`replayed_from_run_id` never modeled), `WorkflowRunStatus` (5 values) + `WorkflowRunStepStatus` (6, incl. `skipped`) with `#[serde(other)]` Unknown fallback, `run_status_is_terminal`/`watch_exit_code` (09-02 building blocks), `workflow_runs_table_config`, `step_duration`; `WorkflowRunsListParams` (workflow_id is a path segment; status only when set; `dry_run=true` only when opted in) + `list_workflow_runs`/`get_workflow_run` client methods on the double-id path with surface `"workflows"` (404 anti-enumeration verbatim).
- **Task 2 — CLI + handlers** (8f2b089): nested `pipelite workflows runs list|get` with required `--workflow` (clap exit 2 pre-HTTP); list renders via `output::render_list` and emits the two locked hints to stderr (quiet-suppressed, exit 0): valid-statuses hint on `--status` empty results, `N test run(s) hidden — pass --include-dry-run` gated on exactly one limit-1 `dry_run=true` probe's `meta.total`; detail exposes a `render_detail` seam for 09-02's watch, emits pretty-JSON passthrough (flatten, no wrapper key), a run summary + bespoke comfy-table steps view (Dynamic arrangement, width 120 non-TTY), and CSV/plain step-rows-only output.
- **Task 3 — Integration proof** (fc86b93 + 5d85797 fix): shared `tests/common/mod.rs` (`cmd_with_server`, unreachable `cmd()`, `spawn_head_capturing_stub_server` recording lowercased request heads) + ten `workflow_runs_stub_test.rs` tests proving query params on the wire, hint semantics/probe discipline, quiet suppression, steps flattening with durations, JSON verbatim-ness, and pre-HTTP `--workflow` enforcement; help examples extended.

## Tasks Completed

| Task | Name | Commit(s) | Files |
| ---- | ---- | --------- | ----- |
| 1 | Runs contracts — models, status enums, duration helper, API client methods | e33e956 (test/RED), 0edf328 (feat/GREEN) | src/api/models.rs, src/api/mod.rs |
| 2 | Runs CLI group + list handler (hints + probe) + detail handler (steps table + JSON) | 8f2b089 | src/cli/workflows.rs, src/commands/workflows/{mod.rs,runs/*} |
| 3 | Shared head-capturing stub helper + workflow runs integration tests | fc86b93 (test), 5d85797 (fix) | tests/common/mod.rs, tests/workflow_runs_stub_test.rs, tests/help_examples_test.rs, src/api/models.rs |

## Verification Results

- Full suite: **331 passed, 0 failed** (all pre-existing suites green; 13 new unit tests + 11 new integration/help tests)
- Task gates: all three `<verify><automated>` gates printed GATE-OK
- No `unwrap()`/`expect()` in new production code; runs never touch the cache (only the doc comment says so); zero new crates
- Remaining build warnings are pre-existing only (WorkflowRunTrigger, TTL_WORKFLOWS, get_workflows_cached — out of scope)

## Deviations from Plan

**1. [Rule 1 - Bug] step_duration rendered relative-to-now tense**
- **Found during:** Task 3 (integration fixture asserted the duration cell)
- **Issue:** `HumanTime::from(delta).to_string()` renders positive deltas as "in 5 minutes" (Display picks tense relative to *now*), and sub-11s deltas as "now" — misleading for a plain duration column
- **Fix:** `to_text_en(Accuracy::Rough, Tense::Present)` → "5 minutes" / "now"; unit test pins both renderings; integration fixture moved to a 5-minute step
- **Files modified:** src/api/models.rs, tests/workflow_runs_stub_test.rs
- **Commit:** 5d85797

**2. [Note - expected TDD behavior] Task 3 tests passed at first run**
The Task 3 integration tests compiled and passed immediately because the functionality landed in Task 2 (type="auto", non-TDD). This is expected, not an unexplained RED-phase pass: the plan-level TDD gate is satisfied by Task 1's test-before-feat commit pair (e33e956 → 0edf328). No investigation needed.

## Auth Gates

None.

## Known Stubs

None. All surface area is wired to the live API client methods; the `#[allow(dead_code)]` items (`WorkflowRunStatus::as_str`, `run_status_is_terminal`, `watch_exit_code`) are deliberate 09-02 building blocks with tests, not stubs.

## Threat Flags

None — no new trust-boundary surface beyond the plan's threat model (table truncation + typed as_str rendering mitigations T-09-01..03 are implemented and request-count-tested).

## Self-Check: PASSED

- FOUND: src/api/models.rs, src/api/mod.rs, src/cli/workflows.rs, src/commands/workflows/runs/{mod,list,detail}.rs
- FOUND: tests/common/mod.rs, tests/workflow_runs_stub_test.rs, tests/help_examples_test.rs (extended)
- FOUND commits: e33e956, 0edf328, 8f2b089, fc86b93, 5d85797 (all in `git log`)
