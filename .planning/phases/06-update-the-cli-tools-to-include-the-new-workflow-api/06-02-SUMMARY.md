---
phase: "06"
plan: "02"
subsystem: workflows
tags: [dashboard, cache, prompt, integration-tests]
dependency_graph:
  requires: [06-01 workflow CRUD + trigger commands]
  provides: [workflow dashboard summary, cache-through prompt helper, 13 integration tests]
  affects: [src/prompt.rs, src/commands/dashboard.rs, tests/workflows_integration.rs]
tech_stack:
  added: []
  patterns: [cache-through-helper, dashboard-extension, integration-test-suite]
key_files:
  created:
    - tests/workflows_integration.rs
  modified:
    - src/prompt.rs
    - src/commands/dashboard.rs
decisions:
  - "Dashboard JSON output wraps pipelines array in object with workflows summary (breaking change from bare array)"
  - "Workflow summary shows active/total counts only (no runs today -- no runs API endpoint available)"
  - "Cache constants KEY_WORKFLOWS and TTL_WORKFLOWS already existed from Plan 01 -- no changes to cache.rs needed"
metrics:
  duration: 5min
  completed: "2026-03-29T20:51:00Z"
  tasks_completed: 2
  tasks_total: 2
  files_changed: 3
---

# Phase 6 Plan 02: Dashboard Workflow Summary, Cache Helper, and Integration Tests Summary

Workflow dashboard summary with active/total counts, cache-through prompt helper for workflow completions, and 13 integration tests covering all workflow subcommands

## What Was Done

### Task 1: Add cache constants, prompt helper, and dashboard workflow summary
**Commit:** 99c5b97

- Added `get_workflows_cached()` to `src/prompt.rs` following `get_pipelines_cached` pattern exactly: cache-through with auto-paginate, 1000-item cap
- Added imports for `WorkflowsListParams`, `KEY_WORKFLOWS`, `TTL_WORKFLOWS` to prompt.rs
- Added `fetch_all_workflows()` helper to `src/commands/dashboard.rs` with auto-pagination
- Dashboard `run()` now fetches workflows and passes active/total counts to render functions
- `render_json()` wraps output in `{ "pipelines": [...], "workflows": { "active": N, "total": M } }` object
- `render_display()` prints "Workflows: N active of M total" as bold line after pipeline sections
- Flat formats (CSV/Plain) also include workflow summary line
- Added `get_workflows_cached_returns_cached_data_on_hit` unit test

### Task 2: Add integration tests for workflows
**Commit:** 238a86d

- Created `tests/workflows_integration.rs` with 13 tests following deals_integration.rs pattern
- Help output tests: subcommands (6 verbs), list flags, create flags, trigger data flag, delete force flag
- Arg validation: get/delete/trigger require ID argument (exit code 2)
- Alias: `pipelite w --help` shows list and trigger subcommands
- Dry-run: create shows POST /api/v1/workflows, trigger shows POST /workflows/{id}/run, delete shows DELETE
- Headless: create without --name in --no-input mode fails exit code 2 with --name in error

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing] Cache constants already existed from Plan 01**
- **Found during:** Task 1
- **Issue:** Plan instructed to add KEY_WORKFLOWS and TTL_WORKFLOWS to cache.rs, but Plan 01 already added them
- **Fix:** Skipped cache.rs modifications -- constants were already in place
- **Files modified:** None (cache.rs unchanged)

**2. [Rule 1 - Bug] Delete dry-run needs --force flag**
- **Found during:** Task 2
- **Issue:** Delete dry-run test needs --force because non-interactive mode without --force returns an error before reaching dry-run logic
- **Fix:** Added --force to the workflows_delete_dry_run test command args
- **Files modified:** tests/workflows_integration.rs

## Decisions Made

1. **Dashboard JSON structure change**: Wrapped pipelines array in object `{ "pipelines": [...], "workflows": {...} }` -- necessary to include workflow summary
2. **Active/total counts only**: D-12 mentions "runs today" but no workflow runs API endpoint exists, so only active/total counts are shown
3. **Cache constants from Plan 01**: KEY_WORKFLOWS and TTL_WORKFLOWS were already added by Plan 01; no changes to cache.rs needed in this plan

## Verification Results

- `cargo test --bin pipelite prompt` passes all 6 tests (including new get_workflows_cached test)
- `cargo test --test workflows_integration` passes all 13 tests
- `cargo build` compiles successfully
- Pre-existing failure in `config_set_parses_positional_args` (unrelated to this plan)

## Known Stubs

None -- all functions are fully implemented with real logic.

## Self-Check: PASSED

- All 3 files exist on disk (prompt.rs, dashboard.rs, workflows_integration.rs)
- Both task commits found in git log (99c5b97, 238a86d)
