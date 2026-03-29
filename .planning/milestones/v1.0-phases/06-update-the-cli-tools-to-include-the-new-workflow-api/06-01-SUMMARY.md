---
phase: "06"
plan: "01"
subsystem: workflows
tags: [crud, api-client, cli-commands, workflow-trigger]
dependency_graph:
  requires: [phases 01-05 complete]
  provides: [workflow CRUD + trigger commands, workflow models, workflow API client methods]
  affects: [src/api/models.rs, src/api/mod.rs, src/cli/mod.rs, src/commands/mod.rs, src/main.rs, src/cache.rs]
tech_stack:
  added: []
  patterns: [entity-crud-replication, fire-and-forget-trigger, json-flag-parsing, tty-confirmation-with-force]
key_files:
  created:
    - src/cli/workflows.rs
    - src/commands/workflows/mod.rs
    - src/commands/workflows/list.rs
    - src/commands/workflows/get.rs
    - src/commands/workflows/create.rs
    - src/commands/workflows/update.rs
    - src/commands/workflows/delete.rs
    - src/commands/workflows/trigger.rs
  modified:
    - src/api/models.rs
    - src/api/mod.rs
    - src/cli/mod.rs
    - src/commands/mod.rs
    - src/main.rs
    - src/cache.rs
decisions:
  - "Triggers/nodes stored as serde_json::Value (not full Rust enums) for flexibility"
  - "Trigger --data supports inline JSON and @filepath syntax (curl convention)"
  - "Delete command adds TTY confirmation with --force override (new pattern for workflows)"
  - "KEY_WORKFLOWS cache key with 1-hour TTL (same as pipelines, slow-changing config)"
  - "WorkflowRunTrigger struct defined but not directly used by trigger command (uses raw Value)"
metrics:
  duration: 8min
  completed: "2026-03-29T20:41:00Z"
  tasks_completed: 2
  tasks_total: 2
  files_changed: 20
---

# Phase 6 Plan 01: Workflow Entity CRUD + Trigger Summary

Full workflow entity with 6 subcommands (list, get, create, update, delete, trigger) following all established patterns, plus fire-and-forget trigger with @filepath data support

## What Was Done

### Task 1: Workflow models, API client methods, and WorkflowsListParams
**Commit:** dc99985

- Added `Workflow`, `WorkflowCreate`, `WorkflowUpdate`, `WorkflowRunTrigger`, `WorkflowRunResponse` structs to `src/api/models.rs`
- Triggers and nodes stored as `Option<Vec<serde_json::Value>>` with `#[serde(default)]` for flexibility
- `workflows_table_config()` with compact 4-column default: id, name, active, updated_at
- 6 new `PipeliteClient` methods: `list_workflows`, `get_workflow`, `create_workflow`, `update_workflow`, `delete_workflow`, `trigger_workflow`
- `WorkflowsListParams` with `active` filter, pagination, expand
- `trigger_workflow` posts to `/api/v1/workflows/{id}/run` with optional JSON body, returns `WorkflowRunResponse`
- 3 unit tests: workflow deserialization, create serialization, run response deserialization

### Task 2: CLI args, command handlers, and main.rs dispatch
**Commit:** c2aceaf

- `src/cli/workflows.rs`: `WorkflowsCommands` enum with 6 subcommands, `workflow_id_candidates()` for shell completions
- `src/commands/workflows/`: 6 command handler files following deals pattern
- Create: interactive prompts for name (required), description, active toggle via `dialoguer::Confirm`; `--triggers`/`--nodes` as JSON string flags; `--stdin` for full JSON input
- Update: interactive prompts + JSON flags; headless mode validation
- Delete: TTY confirmation via `dialoguer::Confirm`, `--force` flag for non-interactive mode, error without confirmation in headless
- Trigger: `--data` flag parsing with inline JSON and `@filepath` syntax; fire-and-forget output showing run_id + status
- Dry-run support on all mutation commands (create, update, delete, trigger)
- Cache invalidation via `KEY_WORKFLOWS` on create/update/delete
- `Commands::Workflows` variant with alias `"w"` in `cli/mod.rs`
- Main.rs dispatch wiring

## Deviations from Plan

None -- plan executed exactly as written.

## Decisions Made

1. **Triggers/nodes as serde_json::Value**: Following research recommendation, no full Rust enums for TriggerConfig/WorkflowNode -- server validates
2. **@filepath syntax on --data**: Common CLI pattern (curl, gh) for loading JSON from files
3. **TTY confirmation on delete**: New pattern (existing deletes lack --force); implemented per plan spec with dialoguer::Confirm
4. **KEY_WORKFLOWS + TTL_WORKFLOWS**: Added to cache.rs for workflow caching support (1 hour TTL, same as pipelines)

## Verification Results

- `cargo build` succeeds
- `cargo test api::models` passes all 17 tests (14 existing + 3 new workflow tests)
- `pipelite workflows --help` shows all 6 subcommands with alias `w`
- `pipelite workflows list --help` shows --active, --limit, --offset, --all, --fields, --expand
- `pipelite workflows trigger --help` shows positional id and --data flag
- `pipelite workflows create --help` shows --name, --triggers, --nodes, --stdin
- `pipelite workflows delete --help` shows --force flag
- Pre-existing test failure in `config_set_parses_positional_args` (unrelated to this plan)

## Known Stubs

None -- all commands are fully wired to API client methods.

## Self-Check: PASSED

- All 8 created files exist on disk
- Both task commits found in git log (dc99985, c2aceaf)
