---
phase: 04-developer-experience
plan: 01
subsystem: cli
tags: [clap, dialoguer, fuzzy-select, clap_complete, shell-completions, dry-run, interactive-prompts]

requires:
  - phase: 02-core-crud
    provides: "Deals CRUD commands, output rendering, AppContext, CliError"
  - phase: 03-full-entity-coverage
    provides: "Pipelines and Stages API client methods for FuzzySelect"
provides:
  - "Global --no-input and --dry-run flags on all subcommands"
  - "prompt.rs shared module (require_text, require_select, optional_text, optional_number, check_missing)"
  - "dry_run.rs shared module (render_dry_run, render_dry_run_delete)"
  - "Shell completions subcommand for bash/zsh/fish"
  - "MissingInput error variant with exit code 2"
  - "Proven pattern for interactive/headless/dry-run on deals create/update/delete"
affects: [04-02, 05-power-features]

tech-stack:
  added: [clap_complete 4.6, dialoguer fuzzy-select feature]
  patterns: [prompt-collect-then-check-missing, dry-run-intercept-before-api, fuzzy-select-cascade]

key-files:
  created:
    - src/prompt.rs
    - src/dry_run.rs
    - src/cli/completions.rs
    - src/commands/completions.rs
  modified:
    - Cargo.toml
    - src/cli/mod.rs
    - src/context.rs
    - src/error.rs
    - src/main.rs
    - src/commands/mod.rs
    - src/commands/deals/create.rs
    - src/commands/deals/update.rs
    - src/commands/deals/delete.rs

key-decisions:
  - "FuzzySelect cascade: pipeline selection first, then stages within that pipeline"
  - "Non-TTY stdin auto-implies --no-input in AppContext.build()"
  - "MissingInput exit code 2 (same as clap usage errors, distinct from runtime exit 1)"
  - "Batch error reporting: collect ALL missing required flags, report once via check_missing"
  - "--stdin and individual flags are mutually exclusive (validation error)"

patterns-established:
  - "Prompt pattern: require_text/require_select for required fields, optional_text/optional_number for optional, check_missing at end"
  - "Dry-run pattern: intercept after building payload, before API call, render method+url+body"
  - "Headless detection: ctx.no_input computed from cli.no_input || !stdin.is_terminal()"

requirements-completed: [INTR-01, INTR-02, INTR-03, HEAD-01, HEAD-02, SHLL-01, SHLL-02, SHLL-03, UX-04]

duration: 7min
completed: 2026-03-25
---

# Phase 4 Plan 01: Interactive Prompts, Dry-Run, and Shell Completions Summary

**Global --no-input/--dry-run flags, shared prompt and dry-run modules, shell completions, and deals as proof-of-pattern for interactive/headless/dry-run support**

## Performance

- **Duration:** 7 min
- **Started:** 2026-03-25T12:00:58Z
- **Completed:** 2026-03-25T12:08:13Z
- **Tasks:** 2
- **Files modified:** 14

## Accomplishments
- Global --no-input and --dry-run flags available on all subcommands with auto-detection of non-TTY stdin
- Shared prompt.rs module with 5 exported helpers for consistent interactive/headless field collection
- Shared dry_run.rs module rendering JSON or human-readable previews for mutations and deletes
- Shell completions for bash, zsh, and fish via `pipelite completions <shell>`
- Deals create fully interactive with FuzzySelect cascade (pipeline -> stage), headless batch error reporting, and dry-run
- Deals update/delete support dry-run; update prompts interactively when no flags on TTY

## Task Commits

Each task was committed atomically:

1. **Task 1: Global flags, prompt module, dry-run module, error variant, completions command** - `44c7256` (feat)
2. **Task 2: Refactor deals create/update/delete with prompts, headless validation, and dry-run** - `39df0b9` (feat)

## Files Created/Modified
- `src/prompt.rs` - Shared prompt helpers: require_text, require_select, optional_text, optional_number, check_missing
- `src/dry_run.rs` - Dry-run rendering: render_dry_run (method/url/body), render_dry_run_delete
- `src/cli/completions.rs` - CompletionsArgs struct with Shell enum
- `src/commands/completions.rs` - Shell completion generation via clap_complete with install instructions on stderr
- `Cargo.toml` - Added clap_complete 4.6, dialoguer fuzzy-select feature
- `src/cli/mod.rs` - Added --no-input, --dry-run global flags and Completions variant
- `src/context.rs` - Added no_input (auto-detects non-TTY) and dry_run fields to AppContext
- `src/error.rs` - Added MissingInput variant with exit code 2
- `src/main.rs` - Added dry_run, prompt modules and Completions dispatch
- `src/commands/mod.rs` - Added completions module
- `src/commands/deals/create.rs` - Interactive prompts, FuzzySelect stage selection, headless validation, dry-run, --stdin exclusivity
- `src/commands/deals/update.rs` - Interactive prompts when no flags on TTY, headless validation, dry-run
- `src/commands/deals/delete.rs` - Dry-run intercept

## Decisions Made
- FuzzySelect cascade for stage selection: user picks pipeline first, then stage within it (provides context for which stages are relevant)
- Non-TTY stdin auto-implies --no-input in AppContext.build() so piped commands never attempt interactive prompts
- MissingInput returns exit code 2 (same bucket as clap usage errors) to distinguish from runtime errors (exit 1)
- Batch error reporting: all missing required flags collected in a Vec, then reported at once via check_missing (not fail-on-first)
- --stdin and individual field flags are mutually exclusive with a clear validation error

## Deviations from Plan
None - plan executed exactly as written.

## Issues Encountered
- Two pre-existing test failures in cli_skeleton (global_flags_parse_without_error and config_set_parses_positional_args) were verified to exist before any changes -- not caused by this plan. Logged as out-of-scope.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- prompt.rs and dry_run.rs are proven on deals -- Plan 02 can replicate the exact same pattern across orgs, people, activities, pipelines, and stages
- All public function signatures are stable and documented
- No blockers for Plan 02

---
*Phase: 04-developer-experience*
*Completed: 2026-03-25*
