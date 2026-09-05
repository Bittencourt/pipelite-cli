# Deferred Items — Phase 7

## Out-of-Scope Discoveries

### 1. Environment-dependent test failures when a real config exists

- **Found during:** Task 3 verification (`cargo test` with real `~/.pipelite/config.toml` present)
- **Tests affected:**
  - `tests/deals_integration.rs::deals_list_limit_zero_is_accepted`
  - `tests/cli_skeleton.rs::config_set_parses_positional_args`
  - `tests/cli_skeleton.rs::global_flags_parse_without_error`
- **Issue:** These tests assert `.failure().code(1)` assuming no `~/.pipelite/config.toml` exists. On a machine with a configured (and reachable) server, the commands succeed and the tests fail with "Unexpected success".
- **Evidence pre-existing:** Fails identically at pre-plan commit `0990737` (verified via temp worktree). Unrelated to batch changes (`deals list`, `config set`, `ping` paths untouched).
- **Also passes:** Full suite passes with zero failures when run with `HOME=<scratch dir>` (the no-config environment the suite was designed for).
- **Side-effect warning:** `config_set_parses_positional_args` WRITES `output.format = "json"` to the real user config when one exists. The user config was restored manually during this plan; consider making these tests hermetic (env-injected config path) in a future test-hygiene pass.
- **Owner:** Phase 8 (Foundations / fixes) or a dedicated test-hygiene task.

**RESOLVED 2026-09-04:** All three tests are now hermetic via the WR-03 env fix — both `cmd()` helpers (`tests/cli_skeleton.rs`, `tests/common/mod.rs`) and the inline test env (`tests/deals_integration.rs::deals_list_limit_zero_is_accepted`) override `PIPELITE_CONFIG` to a nonexistent path plus `PIPELITE_URL`/`PIPELITE_SERVER_URL=http://127.0.0.1:1` and `PIPELITE_API_KEY=fake-test-key`, so no real config is read or written and no real API call is issued. Verified empirically on 2026-09-04 with the real `~/.pipelite/config.toml` present: all three tests pass (cli_skeleton 7/7, deals_integration 12/12).
