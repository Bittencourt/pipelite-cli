---
phase: 01-foundation
plan: 01
subsystem: cli
tags: [rust, clap, toml, thiserror, anyhow, colored, cli-skeleton]

# Dependency graph
requires:
  - phase: none
    provides: greenfield project
provides:
  - Cli struct with global flags and Commands enum (init, ping, config)
  - OutputFormat enum with TTY detection
  - CliError enum with structured display_error on stderr
  - AppConfig with TOML load/save, env var merge, 0600 permissions
  - set_config_value with toml_edit comment preservation
  - Rich version string via build.rs
affects: [01-02, 02-core-crud]

# Tech tracking
tech-stack:
  added: [clap 4.6, tokio 1, reqwest 0.13, serde 1.0, toml 1.1, toml_edit 0.22, dialoguer 0.12, anyhow 1.0, thiserror 2.0, colored 3.1, indicatif 0.18, comfy-table 7.2, assert_cmd 2.0, predicates 3.0, tempfile 3.0]
  patterns: [clap derive for CLI structure, thiserror enums with hint fields, toml_edit for format-preserving config edits, build.rs for compile-time metadata, 0600 file permissions via OpenOptionsExt]

key-files:
  created: [build.rs, src/cli/mod.rs, src/cli/init.rs, src/cli/config.rs, src/output/mod.rs, src/error.rs, src/config.rs, src/context.rs, src/api/mod.rs, src/commands/mod.rs, src/commands/init.rs, src/commands/ping.rs, src/commands/config.rs, tests/cli_skeleton.rs, tests/fixtures/valid_config.toml]
  modified: [Cargo.toml, src/main.rs]

key-decisions:
  - "Used build.rs setting single BUILD_VERSION env var rather than separate RUSTC_VERSION + OS/ARCH concat (simpler, avoids concat! limitations)"
  - "reqwest 0.13 uses 'rustls' feature not 'rustls-tls' (feature renamed in 0.13)"
  - "Rust 2024 edition requires unsafe blocks for env::set_var/remove_var in tests"

patterns-established:
  - "Pattern: clap derive with global flags and after_help examples on every command"
  - "Pattern: Structured errors with thiserror enum variants containing detail + hint fields"
  - "Pattern: Config file written with 0600 permissions via OpenOptionsExt::mode()"
  - "Pattern: toml_edit for config set to preserve comments"
  - "Pattern: Stub modules with todo!() for unimplemented commands"

requirements-completed: [AUTH-02, AUTH-03, CONF-01, CONF-04, ERRH-01, ERRH-02, UX-03]

# Metrics
duration: 7min
completed: 2026-03-24
---

# Phase 1 Plan 01: Project Skeleton and Config Summary

**Clap derive CLI skeleton with rich --version, TOML config with 0600 permissions and env var merge, and thiserror structured error display on stderr**

## Performance

- **Duration:** 7 min
- **Started:** 2026-03-24T23:40:38Z
- **Completed:** 2026-03-24T23:47:50Z
- **Tasks:** 2
- **Files modified:** 17

## Accomplishments
- Full CLI skeleton with --version showing "pipelite 0.1.0 (rustc 1.94.0, linux-x86_64)"
- Config module with TOML round-trip, env var precedence (PIPELITE_API_KEY/PIPELITE_SERVER_URL), 0600 file permissions, and toml_edit comment-preserving set
- Error module with CliError enum (Auth/Connection/Config/NotFound) and structured display on stderr with color support
- 26 tests passing (19 unit + 7 integration)

## Task Commits

Each task was committed atomically:

1. **Task 1: Project skeleton** - `abfac7f` (feat)
2. **Task 2: Error types and config module** - `7621978` (feat)

## Files Created/Modified
- `Cargo.toml` - All Phase 1 dependencies (clap, tokio, reqwest, serde, toml, etc.)
- `build.rs` - Captures rustc version and platform for rich --version string
- `src/main.rs` - Entry point with clap parsing, command dispatch, error handling
- `src/cli/mod.rs` - Cli struct with global flags, Commands enum with after_help
- `src/cli/init.rs` - InitArgs with --url and --key optional flags
- `src/cli/config.rs` - ConfigCommands enum (Show, Set, Get) with after_help
- `src/output/mod.rs` - OutputFormat enum with TTY detection helper
- `src/error.rs` - CliError thiserror enum with display_error and exit_code
- `src/config.rs` - AppConfig with load/save, env var merge, 0600 perms, toml_edit set
- `src/context.rs` - AppContext placeholder for Plan 02
- `src/api/mod.rs` - API client placeholder for Plan 02
- `src/commands/{mod,init,ping,config}.rs` - Command stubs with todo!()
- `tests/cli_skeleton.rs` - 7 integration tests for CLI behavior
- `tests/fixtures/valid_config.toml` - Test fixture for config loading

## Decisions Made
- Used build.rs setting single BUILD_VERSION env var (combines version + rustc + os-arch) rather than trying to concatenate at compile time with concat! macro, which cannot use runtime constants
- reqwest 0.13 renamed the "rustls-tls" feature to "rustls" -- adjusted from research recommendation
- Rust 2024 edition makes env::set_var and env::remove_var unsafe -- wrapped test env var manipulation in unsafe blocks with helper functions

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] reqwest 0.13 feature name change**
- **Found during:** Task 1 (Cargo.toml setup)
- **Issue:** Research recommended `rustls-tls` feature but reqwest 0.13 renamed it to `rustls`
- **Fix:** Changed feature from `rustls-tls` to `rustls` in Cargo.toml
- **Files modified:** Cargo.toml
- **Verification:** `cargo build` succeeds
- **Committed in:** abfac7f (Task 1 commit)

**2. [Rule 1 - Bug] Rust 2024 unsafe env var operations**
- **Found during:** Task 2 (config tests)
- **Issue:** `std::env::set_var` and `remove_var` are unsafe in Rust 2024 edition
- **Fix:** Wrapped all env var mutations in tests with unsafe blocks via helper functions
- **Files modified:** src/config.rs
- **Verification:** All 19 config/error unit tests pass
- **Committed in:** 7621978 (Task 2 commit)

**3. [Rule 1 - Bug] colored crate suppresses ANSI in non-TTY test environment**
- **Found during:** Task 2 (error color test)
- **Issue:** Test asserted ANSI codes present, but colored crate auto-disables in non-TTY
- **Fix:** Used `colored::control::set_override(true)` and compared color vs no-color output
- **Files modified:** src/error.rs
- **Verification:** Color test passes
- **Committed in:** 7621978 (Task 2 commit)

---

**Total deviations:** 3 auto-fixed (2 bugs, 1 blocking)
**Impact on plan:** All auto-fixes necessary for correctness. No scope creep.

## Issues Encountered
None beyond the auto-fixed deviations above.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- CLI skeleton complete with all module stubs compiling
- Config and error modules fully functional and tested
- Ready for Plan 02: HTTP client, AppContext, and all command implementations (init/ping/config)

## Self-Check: PASSED

All 15 created files verified on disk. Both task commits (abfac7f, 7621978) verified in git log.

---
*Phase: 01-foundation*
*Completed: 2026-03-24*
