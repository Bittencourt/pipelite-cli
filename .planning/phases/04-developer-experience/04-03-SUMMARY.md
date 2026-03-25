---
phase: 04-developer-experience
plan: 03
subsystem: testing
tags: [assert_cmd, predicates, integration-tests, headless, completions, dry-run]

requires:
  - phase: 04-developer-experience
    plan: 01
    provides: "Global --no-input/--dry-run flags, prompt.rs, dry_run.rs, completions command, MissingInput error"
  - phase: 04-developer-experience
    plan: 02
    provides: "Prompt/dry-run/headless patterns replicated to all entities"
provides:
  - "Integration tests for headless mode validation across all 6 entities"
  - "Integration tests for shell completion generation (bash, zsh, fish)"
  - "Integration tests for dry-run on deals create/update/delete, orgs create, people create"
  - "Proof that dry-run never makes HTTP requests (unreachable server test)"
affects: [05-power-features]

tech-stack:
  added: []
  patterns: [env-var-config-for-tests, write-stdin-non-tty-pattern, unreachable-server-dry-run-proof]

key-files:
  created:
    - tests/headless_test.rs
    - tests/completions_test.rs
    - tests/dry_run_test.rs
  modified: []

key-decisions:
  - "Tests use PIPELITE_URL/PIPELITE_API_KEY env vars with fake values to bypass AppContext config loading"
  - "Dry-run tests point to unreachable server (localhost:1) to prove no HTTP request is made"
  - "Non-TTY detection verified via write_stdin which makes stdin non-TTY in assert_cmd"
  - "Adapted test expectations to match actual entity behavior (all entities now use exit code 2 with batch missing reporting)"

patterns-established:
  - "Integration test helper: cmd() returns Command with fake env vars for config bypass"
  - "Headless validation test pattern: write_stdin('') -> assert code(2) -> assert stderr contains flag names"
  - "Dry-run proof pattern: unreachable server URL + assert success proves no HTTP call"

requirements-completed: [INTR-03, HEAD-01, HEAD-02, SHLL-01, SHLL-02, SHLL-03, UX-04]

duration: 5min
completed: 2026-03-25
---

# Phase 4 Plan 03: Integration Tests for Headless Mode, Shell Completions, and Dry-Run Summary

**21 integration tests validating headless validation (exit code 2, batch missing), shell completions (bash/zsh/fish), and dry-run request preview without HTTP calls**

## Performance

- **Duration:** 5 min
- **Started:** 2026-03-25T12:11:45Z
- **Completed:** 2026-03-25T12:16:46Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments
- 10 headless mode tests covering all 6 entities (deals, orgs, people, activities, stages, pipelines) with exit code 2 and batch missing field reporting
- 4 shell completion tests verifying bash/zsh/fish script generation and install instructions on stderr
- 7 dry-run tests proving request preview output (method/URL/body) without making HTTP calls, including JSON format mode
- All 21 new tests pass; no new test infrastructure or dependencies needed

## Task Commits

Each task was committed atomically:

1. **Task 1: Headless mode integration tests** - `79764cc` (test)
2. **Task 2: Shell completions and dry-run integration tests** - `97224fc` (test)

## Files Created/Modified
- `tests/headless_test.rs` - 10 tests for headless mode: missing fields validation, --no-input flag, --stdin mutual exclusivity
- `tests/completions_test.rs` - 4 tests for shell completion generation: bash, zsh, fish, invalid shell rejection
- `tests/dry_run_test.rs` - 7 tests for --dry-run: deals CRUD, orgs create, people create, unreachable server proof

## Decisions Made
- Used env vars (PIPELITE_URL, PIPELITE_API_KEY) with fake values to bypass config file requirement in tests -- simpler than fixture files
- Dry-run tests use localhost:1 (unreachable) as PIPELITE_URL to prove no HTTP request is made (exit 0 = success = no connection attempt)
- Adapted plan expectations to match actual code state: all entities already refactored to use MissingInput (exit 2) with batch error reporting, not Validation (exit 1) with fail-on-first
- Delete dry-run test checks JSON output (not "Would delete" text) because non-TTY stdout auto-selects JSON format

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Corrected exit code expectations for non-deals entities**
- **Found during:** Task 1 (headless tests)
- **Issue:** Plan assumed orgs/people/activities/stages/pipelines still used Validation (exit 1, fail-on-first). In reality, all entities have been refactored to MissingInput (exit 2, batch reporting) by 04-02.
- **Fix:** Updated all entity tests to expect exit code 2 with batch missing field assertions
- **Files modified:** tests/headless_test.rs
- **Verification:** All 10 headless tests pass
- **Committed in:** 97224fc (updated in Task 2 commit)

**2. [Rule 1 - Bug] Corrected dry-run delete output format expectation**
- **Found during:** Task 2 (dry-run tests)
- **Issue:** Plan expected "Would delete" text output, but non-TTY stdout (piped in tests) auto-selects JSON format
- **Fix:** Changed assertion to check JSON output with "dry_run": true and "method": "DELETE"
- **Files modified:** tests/dry_run_test.rs
- **Verification:** All 7 dry-run tests pass
- **Committed in:** 97224fc

---

**Total deviations:** 2 auto-fixed (2 bugs in test expectations)
**Impact on plan:** Both fixes aligned tests with actual code behavior. No scope change.

## Issues Encountered
- Two pre-existing test failures in cli_skeleton.rs (config_set_parses_positional_args and global_flags_parse_without_error) -- these were already documented in 04-01-SUMMARY as out-of-scope. Not caused by this plan.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- All Phase 4 requirements now have automated test coverage
- Test patterns (env var config, write_stdin, unreachable server proof) are reusable for Phase 5
- No blockers for Phase 5

---
*Phase: 04-developer-experience*
*Completed: 2026-03-25*

## Self-Check: PASSED
