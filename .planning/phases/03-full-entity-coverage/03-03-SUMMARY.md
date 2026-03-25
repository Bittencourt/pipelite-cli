---
phase: 03-full-entity-coverage
plan: 03
subsystem: pipelines-stages-crud
tags: [pipelines, stages, crud, cli, api]
dependency_graph:
  requires: [02-01, 02-02, 03-01, 03-02]
  provides: [pipelines-crud, stages-crud, pipeline-enforcement]
  affects: [src/api/models.rs, src/api/mod.rs, src/cli/mod.rs, src/commands/mod.rs, src/main.rs]
tech_stack:
  added: []
  patterns: [individual-create-loop, runtime-flag-validation, serde-rename]
key_files:
  created:
    - src/cli/stages.rs
    - src/commands/stages/mod.rs
    - src/commands/stages/list.rs
    - src/commands/stages/get.rs
    - src/commands/stages/create.rs
    - src/commands/stages/update.rs
    - src/commands/stages/delete.rs
    - tests/stages_integration.rs
  modified:
    - src/api/models.rs
    - src/api/mod.rs
    - src/cli/mod.rs
    - src/commands/mod.rs
    - src/main.rs
decisions:
  - "Stage 'type' field uses #[serde(rename = \"type\")] with stage_type Rust field name"
  - "stages list validates --pipeline at runtime (CliError::Validation) for helpful error"
  - "stages get/update/delete take only stage ID (no --pipeline needed)"
  - "Pipeline alias 'pl', Stages alias 's'"
metrics:
  duration: 12min
  completed: 2026-03-25
---

# Phase 03 Plan 03: Pipelines + Stages CRUD Summary

Stages CRUD with required --pipeline validation on list/create, serde rename for reserved "type" keyword, and individual-create loop for stdin batch mode.

## What Was Done

### Task 1: Pipelines entity (already implemented)

Pipeline CRUD was fully implemented in the prior plan execution (03-02 commit `06d7263`). All pipeline code -- models, API client, CLI args, command handlers, and integration tests -- was already committed and working. No additional work was needed.

Verified: 7 integration tests + 2 model unit tests all passing.

### Task 2: Stages entity (new implementation)

**Models (src/api/models.rs):**
- `Stage` struct with `#[serde(rename = "type")] pub stage_type: String` to handle Rust reserved keyword
- `StageCreate` with required name + pipeline_id, optional description/color/stage_type
- `StageUpdate` with all optional fields
- `stages_table_config()` with default columns: id, name, pipeline_id, position
- 2 unit tests: `stage_deserializes_from_json`, `stage_create_required_only`

**API Client (src/api/mod.rs):**
- `list_stages`, `get_stage`, `create_stage`, `update_stage`, `delete_stage`
- `StagesListParams` with required `pipeline_id: String` (not Option)
- No batch endpoint -- uses individual-create loop

**CLI Args (src/cli/stages.rs):**
- `StagesCommands` enum with List, Get, Create, Update, Delete
- `--pipeline` is `Option<String>` in clap (validated at runtime for better error messages)
- `--type` maps to `stage_type` via `#[arg(long = "type")]`
- Alias "s" registered in Commands enum

**Command Handlers (src/commands/stages/):**
- `list.rs`: Validates `--pipeline` is provided via `CliError::Validation` before API call
- `create.rs`: Validates both `--name` and `--pipeline` required; individual-create loop for stdin
- `get.rs`, `update.rs`, `delete.rs`: Take stage ID only (no --pipeline)
- `update.rs`: Maps --name, --color, --type, --description to StageUpdate

**Integration Tests (tests/stages_integration.rs):**
- 8 tests covering help output, argument validation, exit codes, and runtime behavior

## Commits

| Commit | Type | Description |
|--------|------|-------------|
| 06d7263 | feat | (prior) Pipelines CRUD in 03-02 plan |
| 727f92a | feat | Stages CRUD with --pipeline enforcement |

## Deviations from Plan

### Auto-discovered Issue

**[Rule 3 - Blocking] Pipeline entity already existed from prior plan**
- **Found during:** Task 1
- **Issue:** The prior plan execution (03-02) had already implemented the complete pipelines entity including all models, API methods, CLI args, command handlers, and tests
- **Fix:** Verified all existing pipeline code matched plan requirements, skipped redundant commit
- **Impact:** None -- Task 1 was verified as complete without changes

## Verification Results

- `cargo build` -- compiles successfully (warnings only, no errors)
- `cargo test --test pipelines_integration` -- 7/7 passed
- `cargo test --test stages_integration` -- 8/8 passed
- `cargo test api::models::tests::pipeline` -- 2/2 passed
- `cargo test api::models::tests::stage` -- 2/2 passed
- Pipeline alias "pl" works, Stages alias "s" works
- `stages list --help` shows --pipeline flag
- `stages list` without --pipeline fails with runtime validation error (exit 1, not exit 2)

## Decisions Made

1. **Stage type field handling:** Used `#[serde(rename = "type")]` with `stage_type` as the Rust field name. This correctly serializes/deserializes the JSON "type" field while avoiding the Rust reserved keyword.

2. **Runtime --pipeline validation:** The `--pipeline` flag on `stages list` and `stages create` is `Option<String>` in clap (not required by clap) so we can provide a helpful `CliError::Validation` error message instead of clap's generic error. This matches the locked decision from CONTEXT.md.

3. **Pipeline entity reuse:** Recognized that the prior plan (03-02) had already implemented pipelines as part of its scope. Verified correctness rather than reimplementing.

## Self-Check: PASSED

- All 8 created files exist on disk
- Commit 727f92a verified in git log
- All pipeline + stage tests pass (15 integration + 4 model)
