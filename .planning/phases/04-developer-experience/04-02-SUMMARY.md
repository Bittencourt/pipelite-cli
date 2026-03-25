---
phase: 04-developer-experience
plan: 02
subsystem: cli
tags: [dialoguer, fuzzy-select, interactive-prompts, dry-run, headless-validation]

requires:
  - phase: 04-developer-experience
    provides: "prompt.rs, dry_run.rs shared modules, AppContext with no_input/dry_run, MissingInput error"
  - phase: 03-full-entity-coverage
    provides: "All 5 entity CRUD commands, API client methods, CLI arg structs"
provides:
  - "All 18 mutation commands (6 entities x 3 operations) support interactive prompts, headless validation, and dry-run"
  - "FuzzySelect for org in people create, pipeline in stages create"
  - "--stdin/flags mutual exclusivity on all 6 create commands"
  - "Headless mode fails with MissingInput listing ALL missing flags on all create commands"
affects: [05-power-features]

tech-stack:
  added: []
  patterns: [prompt-collect-then-check-missing replicated across all entities, optional-fuzzy-select-with-skip for people create org]

key-files:
  created: []
  modified:
    - src/commands/orgs/create.rs
    - src/commands/orgs/update.rs
    - src/commands/orgs/delete.rs
    - src/commands/people/create.rs
    - src/commands/people/update.rs
    - src/commands/people/delete.rs
    - src/commands/activities/create.rs
    - src/commands/activities/update.rs
    - src/commands/activities/delete.rs
    - src/commands/pipelines/create.rs
    - src/commands/pipelines/update.rs
    - src/commands/pipelines/delete.rs
    - src/commands/stages/create.rs
    - src/commands/stages/update.rs
    - src/commands/stages/delete.rs

key-decisions:
  - "People org selection uses optional FuzzySelect with '(none - skip)' at top, since org is not required for people"
  - "Pipeline default boolean uses dialoguer::Confirm on TTY rather than text prompt"
  - "Activities batch dry-run shows each individual payload (no batch endpoint)"

patterns-established:
  - "Optional FuzzySelect with skip: prepend '(none - skip)' to options list, return None on index 0"

requirements-completed: [HEAD-03, INTR-01, INTR-02]

duration: 5min
completed: 2026-03-25
---

# Phase 4 Plan 02: Entity-Wide Prompts, Headless Validation, and Dry-Run Summary

**Interactive prompts, headless validation, and dry-run support replicated from deals to all 5 remaining entities (orgs, people, activities, pipelines, stages)**

## Performance

- **Duration:** 5 min
- **Started:** 2026-03-25T12:11:46Z
- **Completed:** 2026-03-25T12:16:36Z
- **Tasks:** 2
- **Files modified:** 15

## Accomplishments
- All 6 entities' create commands prompt for required fields on TTY and fail with MissingInput listing all missing flags in headless mode
- All 18 mutation commands (create/update/delete x 6 entities) support --dry-run
- FuzzySelect for org selection in people create (optional with skip) and pipeline selection in stages create (required)
- --stdin and individual field flags mutually exclusive on all 6 create commands
- All update commands prompt interactively when no flags on TTY, fail with validation error in headless mode

## Task Commits

Each task was committed atomically:

1. **Task 1: Refactor orgs, people, and activities create/update/delete** - `716a66f` (feat)
2. **Task 2: Refactor pipelines and stages create/update/delete** - `5f184b8` (feat)

## Files Created/Modified
- `src/commands/orgs/create.rs` - Interactive prompts for name (required), website/industry/notes (optional), dry-run, --stdin exclusivity
- `src/commands/orgs/update.rs` - Interactive prompts when no flags on TTY, headless validation, dry-run
- `src/commands/orgs/delete.rs` - Dry-run intercept via render_dry_run_delete
- `src/commands/people/create.rs` - Interactive prompts for first_name/last_name (required), FuzzySelect for org (optional with skip), dry-run, --stdin exclusivity
- `src/commands/people/update.rs` - Interactive prompts when no flags on TTY, headless validation, dry-run
- `src/commands/people/delete.rs` - Dry-run intercept
- `src/commands/activities/create.rs` - Interactive prompts for title/type_id (required), deal/due_at/notes (optional), dry-run, --stdin exclusivity, batch dry-run per-item
- `src/commands/activities/update.rs` - Interactive prompts (excluding mark_done/mark_undone action flags), headless validation, dry-run including mark-undone raw JSON
- `src/commands/activities/delete.rs` - Dry-run intercept
- `src/commands/pipelines/create.rs` - Interactive prompt for name (required), Confirm for default boolean, dry-run, --stdin exclusivity, batch dry-run per-item
- `src/commands/pipelines/update.rs` - Interactive prompts with Confirm for default, headless validation, dry-run
- `src/commands/pipelines/delete.rs` - Dry-run intercept
- `src/commands/stages/create.rs` - Interactive prompt for name (required), FuzzySelect for pipeline (required), optional stage_type/color/description, dry-run, --stdin exclusivity, batch dry-run per-item
- `src/commands/stages/update.rs` - Interactive prompts for name/stage_type/color/description, headless validation, dry-run
- `src/commands/stages/delete.rs` - Dry-run intercept

## Decisions Made
- People org selection uses optional FuzzySelect with "(none - skip)" at top since org is not required for people, unlike stage which is required for deals
- Pipeline default boolean uses dialoguer::Confirm on TTY with default false, rather than text prompt, for natural boolean UX
- Activities batch dry-run shows each individual payload since there is no batch endpoint

## Deviations from Plan
None - plan executed exactly as written.

## Issues Encountered
- Two pre-existing test failures in cli_skeleton (global_flags_parse_without_error and config_set_parses_positional_args) continue from Plan 01 -- not caused by this plan

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- All 18 mutation commands across 6 entities fully support interactive prompts, headless validation, and dry-run
- Phase 4 (Developer Experience) is now complete
- Ready for Phase 5 (Power Features) when planned

---
*Phase: 04-developer-experience*
*Completed: 2026-03-25*
