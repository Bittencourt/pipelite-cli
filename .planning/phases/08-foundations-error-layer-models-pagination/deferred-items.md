# Phase 8 — Deferred Items

## Out-of-Scope Discoveries (08-01, 2026-09-03)

Logged per executor scope-boundary rules; not fixed during 08-01.

### 1. Environment-dependent test failures (pre-existing, proven at c3c2dcf)

- `cli_skeleton::config_set_parses_positional_args`
- `cli_skeleton::global_flags_parse_without_error`
- `deals_integration::deals_list_limit_zero_is_accepted`

**Cause:** the dev machine has a real `~/.pipelite/config.toml`; these tests
assume a no-config environment ("exits with an error about missing config" /
"no server") and assert `.failure()` — the real config makes the commands
succeed. Reproduced identically at pre-plan commit `c3c2dcf` via throwaway
worktree; zero relation to 08-01 changes.

**Candidate fix (deferred):** make the tests hermetic by overriding `HOME`
(tempfile::tempdir) in `cmd()` helpers, matching the no-config suite default
described in CLAUDE.md ("tests use fake credentials 127.0.0.1:1 +
fake-test-key").

### 2. Pre-existing build warnings (untouched files)

- `src/commands/workflows/list.rs:56` — value assigned to `total` never read
- `src/api/models.rs:416` — struct `WorkflowRunTrigger` never constructed
- `src/cache.rs:30` — constant `TTL_WORKFLOWS` never used
- `src/prompt.rs:318` — function `get_workflows_cached` never used

**Note:** models/cache/prompt dead items may become live again in 08-02 or
Phases 10-12 (workflows cache usage); re-check before deleting.
