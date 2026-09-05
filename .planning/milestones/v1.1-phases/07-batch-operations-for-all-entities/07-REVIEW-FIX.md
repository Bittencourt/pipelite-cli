---
phase: 07-batch-operations-for-all-entities
fixed_at: 2026-09-02T14:20:00Z
review_path: .planning/phases/07-batch-operations-for-all-entities/07-REVIEW.md
iteration: 2
findings_in_scope: 4
fixed: 4
skipped: 0
status: all_fixed
---

# Phase 07: Code Review Fix Report

**Fixed at:** 2026-09-02T14:20:00Z
**Source review:** .planning/phases/07-batch-operations-for-all-entities/07-REVIEW.md
**Iteration:** 2

**Summary:**
- Findings in scope: 4 (WR-06, WR-07, WR-08, WR-09)
- Fixed: 4
- Skipped: 0

## Fixed Issues

### WR-06: Piped single-ID deletes bypassed the --force gate for 6 entities

**Files modified:** `src/commands/deals/delete.rs`, `src/commands/orgs/delete.rs`, `src/commands/people/delete.rs`, `src/commands/activities/delete.rs`, `src/commands/pipelines/delete.rs`, `src/commands/stages/delete.rs`
**Commit:** 45c1cc9
**Status:** fixed: requires human verification (control-flow change on a destructive operation; behaviorally probed)
**Applied fix:** Changed the early-return branch from `ids.len() == 1` to `ids.len() == 1 && !args.stdin` in all 6 delete handlers, so piped `--stdin` input always routes through `run_batch_delete` (which owns the confirmation/`--force` gate) regardless of item count. Single positional-ID deletes keep the v1.0 gate-free flow, as the review prescribed. Updated the doc comments on all 6 files to state the boundary accurately.
**Live probes:** `echo '["deal_1"]' | pipelite deals delete --stdin` → "Refusing to batch-delete without confirmation in non-interactive mode", exit 1, no HTTP (deals and orgs probed). With `--force` → HTTP DELETE attempted (reaches per-item loop). Positional `pipelite deals delete deal_solo` → unchanged v1.0 behavior (HTTP attempted, no gate).

### WR-07: after_help examples advertised piped deletes that now fail

**Files modified:** `src/cli/deals.rs`, `src/cli/orgs.rs`, `src/cli/people.rs`, `src/cli/activities.rs`, `src/cli/pipelines.rs`, `src/cli/stages.rs`
**Commit:** 7c9344f
**Status:** fixed
**Applied fix:** Appended `--force` to the `delete --stdin` example in all six `after_help` strings, matching the workflows precedent. Positional single-delete examples were left without `--force` — correct, because the WR-06 fix keeps positional single deletes gate-free (v1.0 behavior), so the examples remain accurate.
**Live probes:** `pipelite deals/orgs/stages delete --help` all render `echo '["..."]' | pipelite <entity> delete --stdin --force`.

### WR-08: Hint texts referenced non-existent subcommands

**Files modified:** `src/batch.rs`, `src/commands/deals/update.rs`, `src/commands/orgs/update.rs`, `src/commands/people/update.rs`, `src/commands/activities/update.rs`, `src/commands/pipelines/update.rs`, `src/commands/stages/update.rs`, `src/commands/workflows/update.rs`
**Commit:** 7f5a697
**Status:** fixed
**Applied fix:** Added a dedicated `cli_entity: &str` parameter to `run_batch_update` (display `entity` and REST `api_path` were conflated before; orgs' `api_path` is `"organizations"` but the subcommand is `orgs`) and used it in the "No data on stdin" hint. Renamed `ensure_update_fields`'s parameter to `cli_entity` and updated all 7 update call sites to pass the plural CLI subcommand name ("deals", "orgs", "people", "activities", "pipelines", "stages", "workflows"), so the misspelled-field hint renders `pipelite deals update --help` instead of the invalid singular `pipelite deal update --help`.
**Live probes:** `pipelite orgs update --stdin` with TTY stdin → hint reads `echo '[{"id":"org_1","name":"New"}]' | pipelite orgs update --stdin`. Misspelled-field items (`{"id":"deal_1","tiitle":"X"}`) → "check \`pipelite deals update --help\`"; same verified for orgs (`pipelite orgs update --help`).

### WR-09: Four verbatim parse_custom_fields copies in create handlers

**Files modified:** `src/commands/deals/create.rs`, `src/commands/orgs/create.rs`, `src/commands/people/create.rs`, `src/commands/activities/create.rs`
**Commit:** 4942a81
**Status:** fixed
**Applied fix:** Deleted the four private `parse_custom_fields` functions and their doc comments; each create handler now calls the shared `batch::parse_custom_fields` (added `use crate::batch;` where needed). Zero behavior change intended; note the people copy's hint example ("--custom-field role=CTO") consolidates onto the shared generic example ("--custom-field industry=Tech") — exactly the trade-off the review prescribed.
**Verification:** `grep "fn parse_custom_fields" src/commands/` returns nothing; `cargo check` clean with no unused-import warnings (CliError remains used in all four files).

## Skipped Issues

None — all findings were fixed.

## Test Results

- `cargo check` — clean after every finding (only the 4 pre-existing warnings in workflows/list.rs, api/models.rs, cache.rs, prompt.rs; none reference modified files)
- `cargo test batch` — 19 tests passed, 0 failed (batch unit tests + batch_cli_stub + batch_error + batch update/delete integration)
- `cargo test` (full) — 149 passed, 2 failed. The 2 failures are `tests/cli_skeleton.rs` (`config_set_parses_positional_args`, `global_flags_parse_without_error`) — the documented pre-existing environment coupling (they assume no `~/.pipelite/config.toml` exists; this machine has one). That file is untouched by this fix loop and the failures are not a regression.

---

_Fixed: 2026-09-02T14:20:00Z_
_Fixer: the agent (gsd-code-fixer)_
_Iteration: 2_
