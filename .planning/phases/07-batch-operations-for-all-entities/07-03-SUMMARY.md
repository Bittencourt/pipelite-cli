---
phase: 07-batch-operations-for-all-entities
plan: 03
subsystem: cli-batch-operations
tags: [batch, pipelines, stages, workflows, stdin, continue-on-error, cli]
requires:
  - src/batch.rs (BatchOutcome, read_stdin_json — from Plan 07-01)
  - deals/orgs batch implementation as canonical reference pattern
  - src/api/mod.rs (update_pipeline, delete_pipeline, update_stage, delete_stage, update_workflow, delete_workflow)
provides:
  - pipelines batch update (--stdin, id-in-payload) + batch delete (multi-ID + --stdin + confirmation)
  - stages batch update + batch delete (same pattern)
  - workflows batch update + batch delete (preserving --force)
  - Full 7-entity batch coverage — deals, orgs, people, activities, pipelines, stages, workflows
affects:
  - Plan 07-04 (integration tests build on these handlers)
tech-stack:
  added: []
  patterns:
    - BatchOutcome continue-on-error loop (verbatim from deals/orgs reference)
    - required_unless_present clap pattern for positional-or---stdin args
    - dry-run-before-confirmation ordering (D-05) in all batch delete paths
    - confirmation-skip via force || no_input || !is_terminal (workflows batch delete)
key-files:
  created: []
  modified:
    - src/cli/pipelines.rs
    - src/cli/stages.rs
    - src/cli/workflows.rs
    - src/commands/pipelines/update.rs
    - src/commands/pipelines/delete.rs
    - src/commands/stages/update.rs
    - src/commands/stages/delete.rs
    - src/commands/workflows/update.rs
    - src/commands/workflows/delete.rs
decisions:
  - Replicated the deals/orgs reference implementation verbatim including the --stdin vs field-flags mutual-exclusivity check (canonical pattern from 07-01 Rule 2 fix)
  - Pipelines batch update AND delete invalidate KEY_PIPELINES + stages_ cache prefix in both single and batch paths (extends existing pipeline-mutation cache pattern)
  - Batch stages update invalidates KEY_STAGES + stages_ prefix (multi-pipeline safe) instead of the single-update's per-pipeline key, since batch items can span pipelines
  - Workflows batch delete skips confirmation on args.force || ctx.no_input || !is_terminal (per plan); single-delete force/prompt/refusal behavior preserved verbatim
metrics:
  duration: 10 min
  completed: 2026-09-02T11:29:14Z
  tasks: 2
  files: 9
---

# Phase 7 Plan 3: Batch Operations for Pipelines, Stages, Workflows Summary

Deals batch pattern replicated to pipelines, stages, and workflows (with `--force` preserved on workflow delete): batch update via `--stdin` (JSON array with per-item `id`, continue-on-error) and batch delete via multi-ID positionals or `--stdin` with dry-run-first confirmation — **all 7 entities are now batch-capable**, completing per-entity coverage for the phase.

## Tasks Completed

| Task | Name | Commit | Files |
| ---- | ---- | ------ | ----- |
| 1 | Batch update + delete for pipelines and stages | 8e732ab | src/cli/pipelines.rs, src/cli/stages.rs, src/commands/pipelines/update.rs, src/commands/pipelines/delete.rs, src/commands/stages/update.rs, src/commands/stages/delete.rs |
| 2 | Batch update + delete for workflows | 052150f | src/cli/workflows.rs, src/commands/workflows/update.rs, src/commands/workflows/delete.rs |

## What Was Built

- **CLI args (all 3 entities)** — `UpdateArgs.id` is now `Option<String>` with `required_unless_present = "stdin"` plus a `--stdin` flag; `DeleteArgs.id` became `ids: Vec<String>` (multi-ID positional, same `required_unless_present`) plus `--stdin`. `ArgValueCandidates` completion annotations preserved on id/ids fields. `WorkflowsDeleteArgs.force: bool` kept untouched. after_help batch examples added to all six subcommands.
- **Batch update (pipelines/stages/workflows)** — `<entity> update --stdin` reads a JSON array, rejects field flags alongside `--stdin` (mutual exclusivity, canonical pattern), extracts `id` per object, deserializes the rest as `*Update`, calls the entity's `update_*` client method per item with continue-on-error via `BatchOutcome`; successes rendered via `render_list` after the loop; cache invalidated once if anything succeeded; dry-run renders one PUT per item.
- **Batch delete (pipelines/stages/workflows)** — `<entity> delete id1 id2 id3` or `--stdin` with a JSON string array (mutually exclusive, empty-list rejected); single ID keeps the original single-delete flow; batch path checks dry-run FIRST (never prompts), then `dialoguer::Confirm` ("Delete N pipeline(s)? / stage(s)? / workflow(s)?", default false); per-item errors to stderr, cache invalidated once if any succeeded, `finalize` summary with exit 1 on any failure.
- **Cache correctness** — pipelines update/delete invalidate `KEY_PIPELINES` + `invalidate_prefix("stages_")` (single and batch paths); batch stages update invalidates `KEY_STAGES` + `stages_` prefix because items may span multiple pipelines; workflows invalidate `KEY_WORKFLOWS` only.
- **Workflows --force** — batch delete confirmation skipped when `args.force || ctx.no_input || !io::stdin().is_terminal()`; the single-delete flow (force skips prompt, TTY prompt, non-TTY refusal with hint) preserved verbatim.

## Test Results

- `cargo check` — clean (only the 4 pre-existing warnings documented in 07-01)
- Full suite `cargo test` (scratch HOME + real RUSTUP/CARGO_HOME — the suite's designed no-config environment) — **212 passed, 0 failed** across all 20 test binaries (same totals as the 07-02 baseline), including `workflows_integration` (13/13) and `stages_integration` (8/8)
- CLI smoke: `pipelines update --help` shows `[ID]` + `--stdin`; `stages delete --help` shows `[IDS]...` + `--stdin`; `workflows delete --help` shows `[IDS]...` + `--stdin` + `--force`; `workflows update --help` shows `[ID]` + `--stdin`
- Functional smoke (dry-run/fake server `127.0.0.1:1`, no real HTTP): `pipelines delete pl_1 pl_2 pl_3 --dry-run` renders 3 DELETEs; `echo '["stg_1","stg_2"]' | stages delete --stdin --dry-run` renders 2 DELETEs; workflow stdin update array renders 2 PUTs; empty stdin list → "Empty ID list" + hint, exit 1; stdin+positional IDs → mutual-exclusivity error exit 1; `pipelines update --name X --stdin` → mutual-exclusivity error exit 1; single workflow delete without `--force` in non-TTY → refusal with hint (legacy behavior); with `--force` → proceeds to HTTP

## Deviations from Plan

None — plan executed exactly as written. The `--stdin`/field-flags mutual-exclusivity check replicated in all three update handlers is part of the deals reference implementation the plan mandates following. The batch stages update prefix invalidation is a direct application of the plan's pipeline cache-invalidation instruction to the stage entity's existing `stages_{pipeline_id}` cache scheme (the single-update per-pipeline key cannot cover batch items spanning pipelines).

## Requirements Note

BATCH-01, BATCH-02, BATCH-03 (claimed in this plan's frontmatter) are marked complete: all 7 entities now support batch update via `--stdin`, batch delete via multi-ID/`--stdin`, with continue-on-error per-item results and `N ok, M failed` summaries. BATCH-04 (contract hardening: exit-code matrix, `--quiet` summary survival, stdin full validation, 429 Retry-After handling) remains open for Plan 07-04.

## Known Stubs

None — all handlers fully implemented and wired.

## Self-Check: PASSED

- 9 modified files present on disk (grep: `pub id: Option<String>`, `pub ids: Vec<String>`, `pub stdin: bool`, `pub force: bool` in the three CLI files) — FOUND
- `async fn batch_update` in pipelines/stages/workflows update.rs; `async fn batch_delete` in pipelines/stages/workflows delete.rs; `BatchOutcome::new` + `invalidate_prefix("stages_")` in pipeline handlers — FOUND
- No `.unwrap()` in the 6 handler files — VERIFIED via grep (only the explanatory comment references)
- Commits 8e732ab, 052150f — FOUND in git log
- cargo check clean; full suite 212/0 with scratch HOME; CLI help + functional dry-run smokes — VERIFIED
