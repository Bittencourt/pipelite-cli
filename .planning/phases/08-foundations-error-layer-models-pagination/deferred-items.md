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

- `src/api/models.rs:446` — struct `WorkflowRunTrigger` never constructed
- `src/cache.rs:30` — constant `TTL_WORKFLOWS` never used
- `src/prompt.rs:318` — function `get_workflows_cached` never used

**Note:** models/cache/prompt dead items may become live again in Phases
10-12 (workflows cache usage); re-check before deleting.
**Resolved in 08-02:** the `workflows/list.rs` unused-`total` warning
disappeared when the file was rewritten for the client-side `--active`
filter (initializer removed, matching the other 6 list files).

### 3. Intermittent config unit-test flake (pre-existing, proven at 7627c15)

- `config::tests::env_var_precedence_server_url`
- `config::tests::load_and_save_roundtrip`

Observed during 08-02 full-suite runs: these two bin-unit tests fail
~1 in 20 runs, and WHICH one fails varies (or none). NOT caused by 08-02:
reproduced at the pre-08-02 commit `7627c15` via throwaway worktree
(19 pass / 1 fail across 20 runs). Same root-cause family as item 1 —
env-var/config-file mutation racing between parallel test threads with a
real `~/.pipelite/config.toml` present. The 08-02 test additions shift
thread timing, which surfaces it more often.

**Candidate fix (deferred, same as item 1):** hermetic `HOME`/temp config
for the config test module, or a shared mutex around env mutation.
