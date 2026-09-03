---
phase: 07-batch-operations-for-all-entities
verified: 2026-09-03T11:04:07Z
status: passed
score: 5/5 must-haves verified
overrides_applied: 3
overrides:
  - must_have: "User can pipe JSON objects (array or NDJSON) to <entity> update --stdin"
    reason: "NDJSON descoped by documented user decision D-01 in 07-CONTEXT.md ('JSON only via --stdin. No CSV or NDJSON support'). JSON-array piping works for all 7 entities; NDJSON input is cleanly rejected with an actionable Validation error. Re-verified: NDJSON rejection probe still passes (D-01 respected by gap closure)."
    accepted_by: "pedro (documented user decision D-01, 07-CONTEXT.md)"
    accepted_at: "2026-09-02T15:10:00Z"
  - must_have: "With --continue-on-error, one failing item doesn't abort the batch"
    reason: "Continue-on-error is unconditional (always-on) for all batch ops per user decision D-06 in 07-CONTEXT.md ('Continue-on-error for all batch operations'); no opt-in flag exists by design. The behavior the SC describes — one failing item doesn't abort, per-item results, summary, exit-code contract — is fully implemented and probe-verified again in re-verification."
    accepted_by: "pedro (documented user decision D-06, 07-CONTEXT.md)"
    accepted_at: "2026-09-02T15:10:00Z"
  - must_have: "User can batch-delete by piping IDs via --stdin"
    reason: "CR-01 hardening (review fix, iteration 2): piped batch deletes require --force in non-interactive mode because stdin cannot carry both IDs and a confirmation answer. Developer explicitly accepted this design in verification instructions ('intentional hardened behavior'). Piped delete works with --force; TTY flow prompts; single positional delete keeps v1.0 behavior. Re-verified: gate intact in src/batch.rs after gap closure."
    accepted_by: "pedro (orchestrator verification instructions)"
    accepted_at: "2026-09-02T15:10:00Z"
re_verification:
  previous_status: gaps_found
  previous_score: 3/5
  gaps_closed:
    - "Structurally broken input (malformed JSON, missing IDs) is fully rejected before the first HTTP call — zero mutations on invalid input (closed by prevalidate_item_ids + mixed-batch zero-HTTP test + probes)"
    - "A rate-limited (429) item retries once per Retry-After; if it still fails it is classified as a failed item with detail (closed by send_with_retry on all 40 entity send sites + 2 stub-server tests)"
    - "BATCH-04 contract: exit 2 for structural input failure before any HTTP (closed by CliError::InvalidInput → exit_code()==2 at all 7+7 structural sites, probe-verified)"
  gaps_remaining: []
  regressions: []
human_verification:
  - test: "Interactive batch delete confirmation: run `pipelite deals delete deal_1 deal_2` from a real terminal (TTY)"
    expected: "Prompt 'Delete 2 deal(s)?' appears (default No); answering No aborts with no HTTP; answering Yes proceeds. Respect of --no-input (skips prompt, proceeds) and --dry-run (previews without prompting) can be confirmed in the same session."
    why_human: "dialoguer TTY prompt rendering and keyboard interaction cannot be exercised by non-TTY automated probes"
  - test: "Live-server batch success path: against a real Pipelite instance, pipe a small JSON array to `pipelite deals update --stdin` and run a multi-ID `pipelite orgs delete id1 id2 --force` (or dry-run on real data)"
    expected: "Updated records render as a table on stdout (respecting --format), 'Deleted deal <id>' lines print, cache invalidates (subsequent list reflects changes), exit 0 with no summary line when all succeed"
    why_human: "Requires a live API server and real credentials; all automated probes use the unreachable 127.0.0.1:1 endpoint, so success-path rendering and cache invalidation are unverified"
---

# Phase 7: Batch Operations Verification Report

**Phase Goal:** Users can update or delete many records of any entity in one command, with results scripts can trust
**Verified:** 2026-09-03T11:04:07Z
**Status:** human_needed
**Re-verification:** Yes — after gap closure (07-05)

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | Pipe JSON objects to `<entity> update --stdin`, update every record in one invocation, all 7 entity types | ✓ PASSED (override — NDJSON clause, carried) | Regression check: `batch::run_batch_update` still wired 7/7 in `src/commands/*/update.rs`; batch_update_test 11/11 green; NDJSON still rejected as Invalid JSON (D-01 respected — probe re-run this verification). Gap closure did not regress the update path |
| 2 | Batch-delete by multiple positional IDs or piping IDs via `--stdin` | ✓ PASSED (override — CR-01 gate, carried) | Regression check: `collect_delete_ids` + `run_batch_delete` wired 7/7 in `src/commands/*/delete.rs`; batch_delete_test 15/15 green; CR-01 `--force` gate intact (src/batch.rs:174-193, probe: non-interactive piped delete still refuses without `--force`) |
| 3 | Continue-on-error: per-item results + final `N ok, M failed` summary surviving `--quiet`; exit 0 only when all succeed, 1 if any failed | ✓ PASSED (override — flag name D-06, carried) | Re-probed: 2 valid items vs unreachable server → `[1/2] Failed` + `[2/2] Failed` + `0/2 deal updated, 2 failed`, exit 1; `--quiet` → stdout empty (0 bytes), full per-item failures + summary on stderr, exit 1; all-valid dry-run → exit 0. Contract unchanged by gap closure |
| 4 | Structurally broken input (malformed JSON, missing IDs) fully rejected before the first HTTP call — zero mutations on invalid input | ✓ VERIFIED (was FAILED) | **Gap 1 closed.** `prevalidate_item_ids` (src/batch.rs:249-273) scans every parsed item for a string `id` (absent AND non-string both count as missing, matching in-loop `as_str()` semantics) and rejects the WHOLE input with `CliError::InvalidInput` listing 1-based positions. Called at batch.rs:332 — after the empty-list check, BEFORE the `ctx.dry_run` block and any HTTP. Probe (THE regression that failed last time): `echo '[{"id":"deal_1","title":"X"},{"title":"No ID"}]' \| pipelite deals update --stdin` → exit 2, `Item(s) missing a string 'id' field: 2`, **zero** "Connection failed" lines, **zero** `[1/2] Failed` lines — no HTTP attempted, valid item not mutated. Tests: `batch_update_mixed_batch_missing_id_zero_http` (asserts both absences), `batch_update_non_string_id_exits_2`, `batch_update_dry_run_missing_id_exits_2` (exit 2 + empty stdout), rewritten `batch_update_missing_id_in_item_shows_error` (exact message, resolving the carried IN-04 weak-predicate finding). In-loop extraction kept as unreachable defense-in-depth, no unwrap |
| 5 | 429 item retries once per `Retry-After`; still-failing classified as failed item with detail — never crash or silent skip | ✓ VERIFIED (was FAILED) | **Gap 2 closed.** `send_with_retry` (src/api/mod.rs:134-160): first send → if 429, parse `Retry-After` as u64 seconds (missing/non-numeric → 1s), clamp 0..=60 (T-07G-01/02 mitigations), `tokio::time::sleep`, resend the SAME request once via `try_clone`, return second response regardless of status. Routed through **40** entity send sites (41 grep occurrences incl. definition); the only remaining direct `.send()` calls are the two inside the wrapper itself plus `send_ping_request` (intentionally untouched). 429 detail arms in both `handle_response` (line 252) and `handle_delete_response` (line 295): "Rate limited: server returned 429 again after one retry (Retry-After applied)" + reduce-batch-size hint. `CliError::Api` title interpolates status (`API error (HTTP {status})`) so per-item failure lines carry the code. tokio gained only the `time` feature — no new crates. Tests (hand-rolled TcpListener stub server, no new deps): `batch_update_429_retries_once_then_succeeds` (429+`Retry-After: 0` → 200: exit 0, no "Failed" line, request count == 2) and `batch_update_still_429_after_retry_fails_item_with_detail` (429 twice: exit 1, stderr contains "429"/"Rate limited", count == 2 — one retry, never a storm). Both green in the 14/14 batch_error_test run |

**Score:** 5/5 truths verified (3 with carried overrides; the 2 previously-failed truths now verified without overrides)

### Deferred Items

No deferred contract items — all 5 truths hold and no gap matches a later phase (re-checked). Observation (not a phase-7 contract item): 3 tests fail only on this machine's real `HOME` (live `~/.pipelite/config.toml` → live server): the 2 known `cli_skeleton` tests plus `deals_integration::deals_list_limit_zero_is_accepted`, whose own comment says it assumes "no config" — it passed with an empty-HOME rerun (see Behavioral Spot-Checks). Same pre-existing environmental class; the full suite is 100% green in the no-config environment it is designed for.

### Required Artifacts

| Artifact | Expected | Status | Details |
| -------- | -------- | ------ | ------- |
| `src/error.rs` | `CliError::InvalidInput` variant mapped to exit code 2 | ✓ VERIFIED | Variant at line 38 with doc comment; `exit_code()` (lines 77-90) maps `MissingInput \| InvalidInput` → 2, clap → 2, else → 1; `display_error` exhaustive-match arm added; unit test `exit_code_returns_2_for_structural_input_errors` pins InvalidInput==2 / MissingInput==2 / Validation==1 |
| `src/batch.rs` | Pre-loop id pre-validation; all stdin-structural sites raise InvalidInput | ✓ VERIFIED | `prevalidate_item_ids` defined (line 249) + called (line 332, before dry-run block at 334); 10 `InvalidInput` occurrences (7 structural rejection sites: read_stdin_json ×2, collect_delete_ids ×3, run_batch_update ×2, plus prevalidate itself + doc comments); `BatchOutcome::finalize` still returns `Validation` → exit 1 (per-item contract preserved; its 3 unit tests unchanged and passing) |
| `src/api/mod.rs` | `send_with_retry` wrapper on all entity sends; 429-specific Api detail | ✓ VERIFIED | Wrapper at lines 134-160 with clamp/default/sleep/try_clone-once semantics; 41 `send_with_retry(` occurrences (def + 40 sites); direct `.send()` only inside wrapper (×2) + `send_ping_request` (×1, intentional); `map_request_error(e)` == 2 (both wrapper sends); 429 detail arms == 2 |
| `tests/batch_error_test.rs` | Mixed-batch zero-HTTP test, exit-2 contract tests, stub-server 429 tests | ✓ VERIFIED | 354 lines, 14 tests all green: 5 exit-code contract tests (.code(2)), 2 unreachable-server tests pinned .code(1), rewritten missing-id test + mixed zero-HTTP + non-string-id + dry-run pre-scan, 2 stub-server 429 tests (`spawn_stub_server` + `cmd_with_server` + `DEAL_SUCCESS_BODY` harness, hand-rolled on std::net — no new crates) |
| `Cargo.toml` | tokio `time` feature for the retry sleep | ✓ VERIFIED | `tokio = { version = "1", features = ["rt", "macros", "time"] }` — feature-only change, no new dependency |
| `src/commands/{deals,orgs,people,activities,pipelines,stages,workflows}/update.rs` ×7 | `--stdin` vs field-flag exclusivity raises InvalidInput | ✓ VERIFIED | grep: 7/7 files contain exactly 1 `CliError::InvalidInput` — the stdin/flag mutual-exclusivity error; all other Validation sites untouched (probe: exit 2 on conflict) |

### Key Link Verification

| From | To | Via | Status | Details |
| ---- | -- | --- | ------ | ------- |
| `src/batch.rs` (7 structural sites + prevalidate) | `CliError::InvalidInput` | direct construction | ✓ WIRED | 10 occurrences; exit-2 behavior probe-verified at every site class (P1-P7) |
| `src/commands/*/update.rs` ×7 | `CliError::InvalidInput` | stdin/flag exclusivity | ✓ WIRED | 7/7 converted; test `batch_update_stdin_with_field_flag_exits_2` green |
| `src/api/mod.rs` 40 entity sites | `send_with_retry` | `self.send_with_retry(request).await?` | ✓ WIRED | 41 occurrences; zero entity sites retain a direct `.send()`; `send_ping_request` untouched by design |
| `tests/batch_error_test.rs` 429 tests | stub server | `PIPELITE_SERVER_URL` → `cmd_with_server(&url)` | ✓ WIRED | Both 429 tests pass against the live stub listener with count assertions |
| `prevalidate_item_ids` | `run_batch_update` | call at line 332 | ✓ WIRED | Positioned after empty-list check, before dry-run block and item loop — proven by the dry-run probe (exit 2, empty stdout) |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
| -------- | ------------- | ------ | ------------------ | ------ |
| `prevalidate_item_ids` | `items: &[serde_json::Value]` | real piped stdin parsed by `read_stdin_json` | Yes — probe with real piped JSON | ✓ FLOWING |
| `send_with_retry` | `Retry-After` header | real HTTP response from stub server | Yes — stub tests assert count==2 driven by the actual header | ✓ FLOWING |
| Batch success rendering | `succeeded: Vec<Value>` | live `update_one` responses | Yes — 429→200 stub test renders the success path (exit 0, no failure line) | ✓ FLOWING |

### Behavioral Spot-Checks

All probes run against `./target/debug/pipelite` with `PIPELITE_SERVER_URL=http://127.0.0.1:1` (unreachable → any HTTP attempt prints "Connection failed"):

| Behavior | Command | Result | Status |
| -------- | ------- | ------ | ------ |
| **Gap 1 core regression**: mixed batch missing-ID → zero HTTP, exit 2 | `echo '[{"id":"deal_1","title":"X"},{"title":"No ID"}]' \| pipelite deals update --stdin` | exit 2; `Item(s) missing a string 'id' field: 2` + hint; **0** "Connection failed" lines; **0** per-item lines | ✓ PASS (was FAIL) |
| **Gap 3**: invalid JSON → exit 2 | `echo 'not json' \| pipelite deals update --stdin` | exit 2, "Invalid JSON input" + hint | ✓ PASS (was FAIL — exit 1) |
| `[]` update → exit 2 | `echo '[]' \| pipelite deals update --stdin` | exit 2, "Empty update list" | ✓ PASS |
| `[]` delete → exit 2 | `echo '[]' \| pipelite deals delete --stdin` | exit 2, "Empty ID list" | ✓ PASS |
| stdin+positional delete conflict → exit 2 | `echo '["deal_1"]' \| pipelite deals delete deal_1 --stdin` | exit 2, "mutually exclusive" | ✓ PASS |
| Non-string id → exit 2 | `echo '[{"id":123,"title":"X"}]' \| pipelite deals update --stdin` | exit 2, "missing a string 'id' field" | ✓ PASS |
| Dry-run validates structure too | missing-id batch + `--dry-run` | exit 2, stdout empty (0 bytes) | ✓ PASS |
| Per-item contract intact | 2 valid items vs unreachable server | `[1/2] Failed` + `[2/2] Failed` + `0/2 deal updated, 2 failed`; exit **1** (not 2) | ✓ PASS |
| Quiet-surviving summary intact | `pipelite --quiet deals delete deal_1 deal_2 --force` | stdout 0 bytes; stderr carries both failure lines + `0/2 deal deleted, 2 failed`; exit 1 | ✓ PASS |
| All-ok dry-run → exit 0 | 2 valid items + `--dry-run` | exit 0 | ✓ PASS |
| Clap misuse → exit 2 | `pipelite deals delete` (no args) | exit 2 | ✓ PASS |
| **Gap 2**: 429 retry-once-then-success | stub server (429+RA:0 → 200), `batch_update_429_retries_once_then_succeeds` | exit 0, no "Failed" line, request count == 2 | ✓ PASS (was absent) |
| **Gap 2**: still-429 → failed item with detail | stub server (429+RA:0 ×2), `batch_update_still_429_after_retry_fails_item_with_detail` | exit 1, stderr has "429"/"Rate limited" detail, count == 2 (one retry, no storm) | ✓ PASS (was absent) |
| Full test suite (no-config env) | `HOME=<empty> cargo test` | exit 0; 23 test targets all "0 failed"; **256 passed** | ✓ PASS |
| Env-failure triage | `HOME=<empty> cargo test --test deals_integration deals_list_limit_zero_is_accepted` + `--test cli_skeleton` | Both pass with empty HOME → real-HOME failures are config-environmental, not code regressions | ✓ PASS |

### Probe Execution

No probe scripts declared (`scripts/` absent; no `probe-*.sh` references in any PLAN/SUMMARY). Step 7c: N/A — behavioral probes were executed directly by this verifier (table above), including the two stub-server 429 tests run via `cargo test --test batch_error_test` (14/14 green).

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
| ----------- | ---------- | ----------- | ------ | -------- |
| BATCH-01 | 07-01 (also 07-02/03/04) | Batch-update any entity by piping JSON objects to `--stdin` | ✓ SATISFIED | Unchanged from prior verification; regression checks green; NDJSON clause per D-01 override |
| BATCH-02 | 07-01 (also 07-02/03/04) | Batch-delete by multiple IDs or `--stdin` | ✓ SATISFIED | Unchanged; CR-01 gate intact |
| BATCH-03 | 07-01 (also 07-02/03/04) | Continue-on-error with per-item results and final `N ok, M failed` summary | ✓ SATISFIED | Re-probed this verification (P8/P9b) — contract unchanged |
| BATCH-04 | 07-01, 07-05 | Shared utility with consistent contract: exit 0/1/2; quiet-surviving summary; stdin fully validated pre-HTTP; 429 Retry-After retried once then classified as failure | ✓ SATISFIED | All four clauses now hold: shared utility (batch.rs, 14 handlers) ✓; exit matrix 0/1/2 probe-verified end-to-end ✓; stdin fully validated pre-HTTP (malformed JSON, empty lists, exclusivity, missing/non-string ids — all pre-loop, mixed-batch zero-HTTP proven) ✓; 429 retried exactly once per Retry-After (clamped) with rate-limit detail on still-failure ✓. REQUIREMENTS.md marks BATCH-04 `[x]` Complete — now honest |

Orphaned requirements: none — all four IDs claimed in plan frontmatters (07-01: BATCH-01..04; 07-02/03/04: BATCH-01..03; 07-05: BATCH-04).

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| ---- | ---- | ------- | -------- | ------ |
| src/commands/workflows/list.rs | 56 | Unused `total` assignment warning (pre-existing, compiler warning only) | ℹ️ Info | Not phase-07-modified logic; hygiene only |
| src/api/models.rs / src/cache.rs / src/prompt.rs | 416 / 30 / 318 | Dead-code warnings (`WorkflowRunTrigger`, `TTL_WORKFLOWS`, `get_workflows_cached`) | ℹ️ Info | Pre-existing; unrelated to gap closure |

Carried IN-04 findings from the prior report are **resolved**: the weak missing-id predicate was rewritten to exact-message assertions, and the dead `PIPELITE_URL` note is documented in the `cmd()` helper comment. No TBD/FIXME/XXX/TODO/HACK/PLACEHOLDER markers in any phase-modified file. No `unwrap()`/`expect()` in production code (the only `expect()` hits are inside `#[cfg(test)]` modules). No hardcoded-empty stubs. Overrides respected: NDJSON still rejected (D-01), no `--continue-on-error` flag (D-06), CR-01 `--force` gate intact, `finalize` still `Validation` → exit 1.

### Human Verification Required

### 1. Interactive Batch Delete Confirmation

**Test:** Run `pipelite deals delete deal_1 deal_2` from a real terminal (TTY)
**Expected:** "Delete 2 deal(s)?" prompt (default No); No aborts pre-HTTP; Yes proceeds. `--no-input` skips prompt and proceeds; `--dry-run` previews without prompting
**Why human:** dialoguer TTY rendering and keyboard interaction can't be exercised by non-TTY automated probes

### 2. Live-Server Batch Success Path

**Test:** Against a real Pipelite instance: pipe a small JSON array to `pipelite deals update --stdin`; run multi-ID delete with `--force` (or dry-run on real data)
**Expected:** Updated records render as a table (respecting `--format`); "Deleted deal <id>" lines; cache invalidates (subsequent `list` reflects changes); exit 0 with no failure summary. A real 429 from the server should visibly retry once before failing/succeeding
**Why human:** Requires a live API server and real credentials; all automated probes use the unreachable 127.0.0.1:1 endpoint or a scripted stub

### Gaps Summary

No gaps remain. All three blocking gaps from the prior verification are closed and independently re-verified against the current codebase:

1. **Missing-ID pre-HTTP validation (gap 1)** — `prevalidate_item_ids` runs before the dry-run block and any HTTP; the exact probe that failed last time (mixed valid/id-less batch) now exits 2 with zero requests fired. Locked in by the mixed-batch zero-HTTP test asserting "Connection failed" and "[1/2] Failed" are absent.
2. **429 Retry-After retry-once (gap 2)** — `send_with_retry` wraps all 40 entity send sites; Retry-After parsed/clamped (0..=60s, default 1s); exactly one retry via `try_clone`; stub-server tests prove count==2 for both retry-success (exit 0) and still-429 (exit 1 with "API error (HTTP 429)" detail on the per-item line).
3. **Exit-2 structural contract (gap 3)** — dedicated `CliError::InvalidInput` variant at all 7 batch structural sites + 7 handler exclusivity sites; `exit_code()` maps it to 2 while `finalize`'s per-item `Validation` stays 1; the full 0/1/2 matrix was probe-verified end-to-end.

The full suite is green (256/256) in the no-config environment it is designed for. Three tests fail only under this machine's real HOME because a live `~/.pipelite/config.toml` routes commands at a real server — verified environmental by empty-HOME reruns, consistent with the already-logged deferred cli_skeleton entries (the `deals_list_limit_zero_is_accepted` instance is newly observed and belongs to the same class; recommend adding it to `deferred-items.md` entry 1).

BATCH-04 — the phase's last open requirement — is fully satisfied. Status is `human_needed` solely because the two TTY/live-server items from the prior report remain outstanding; they are unchanged in scope and cannot be automated here.

---

_Verified: 2026-09-03T11:04:07Z_
_Verifier: the agent (gsd-verifier)_
