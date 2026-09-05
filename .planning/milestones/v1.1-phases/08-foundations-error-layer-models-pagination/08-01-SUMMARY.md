---
phase: 08-foundations-error-layer-models-pagination
plan: 01
subsystem: api-client-error-layer
tags: [error-handling, rfc7807, forbidden, api-client, tdd]
requires:
  - Phase 7 batch.rs + exit-code contract (InvalidInput→2) + send_with_retry
provides:
  - CliError::Forbidden variant (403 ≠ 401, exit 1)
  - parse_rfc7807 helper (errors[]-first 6-step fallback, quote-free)
  - surface-keyed 401/403 mapping at all three sites (handle_response, handle_delete_response, check_auth_status)
  - forbidden_hint per-surface table (audit/trash/notes/webhooks ready for Phases 10-12)
  - 409 workflows inactive-trigger activation hint
  - spawn_body_stub_server test helper (body-carrying TcpListener stub)
affects:
  - Phase 9-12 (403-heavy surfaces pass a surface key; hints already defined)
  - Phase 13 (docs/api-reference.md:381 now stale: "401,403 → Auth")
tech-stack:
  added: []
  patterns:
    - parse-then-classify error bodies at one chokepoint (parse_rfc7807)
    - surface-key (&'static str) hint tables in the API client
    - body-carrying scripted TcpListener stub for HTTP error-path tests
key-files:
  created:
    - tests/error_layer_stub_test.rs
  modified:
    - src/error.rs
    - src/api/mod.rs
decisions:
  - Forbidden exits 1 (runtime condition, per CONTEXT/A2 — distinguished by variant/message)
  - surface is a plain &'static str (CONTEXT discretion — enum unnecessary)
  - errors[] non-empty but all items invalid degrades to detail chain, never renders ""
  - check_auth_status 403 detail carries the request URL; handlers carry the server body text
metrics:
  duration: 23 min
  completed: 2026-09-03
  tasks: 3
  files: 3
---

# Phase 8 Plan 1: Error Layer — RFC 7807, Forbidden, 409 Hint Summary

**RFC 7807 error layer: Forbidden variant with per-surface hints at all three 401/403 sites, errors[]-first parser replacing the never-matching `error`/`message` extraction, and the 409 inactive-trigger activation hint — proven by 14 unit tests + 7 stub-server integration tests.**

## What Was Built

### Task 1 — Forbidden variant + parse_rfc7807 (TDD)
- **src/error.rs**: `CliError::Forbidden { detail, hint }` with title `"Forbidden (HTTP 403)"` (status interpolated like the `Api` variant so batch per-item lines carry the code); display arm added; `exit_code()` untouched — Forbidden falls through to 1.
- **src/api/mod.rs**: private `parse_rfc7807(body, status)` — locked 6-step fallback: (1) non-empty `errors[]` → `"{field}: {message} ({code})"` joined `"; "` (wins over 422's generic detail; missing `code` → `(invalid)`; items missing field/message skipped), (2) `detail`, (3) `title`, (4) legacy `error`, (5) legacy `message`, (6) `"HTTP {status}"`. Total function: malformed/non-JSON bodies degrade to `"HTTP {status}"` (T-08-01). All extraction via `as_str()` — no JSON quotes (T-08-02/Pitfall 2). No credential interpolation (T-08-03).
- 14 unit tests (`#[cfg(test)] mod tests` in api/mod.rs + Forbidden test in error.rs) covering every fallback branch with the verified server fixtures.

### Task 2 — Surface-keyed mapping, all three sites
- `handle_response` / `handle_delete_response` gained `surface: &'static str`; all **40 call sites** (33 + 7) thread their entity key — entirely inside api/mod.rs, zero command-layer edits (verified: the only other modified source file is error.rs).
- Status arms rewritten in both handlers: `401 → Auth` (init hint preserved), `403 → Forbidden { forbidden_hint(surface) }`, **new `409` arm** → `Api { status: 409 }` with the workflows activation hint (`pipelite workflows update <id> --active true`) guarded by `surface == "workflows"`. 404/422/429/catch-all arms unchanged apart from now-real detail text.
- `check_auth_status` (ping/init path) split: 403 → `Forbidden` with `forbidden_hint("general")` — Pitfall 1 closed.
- `forbidden_hint` table: `"audit"`, `"trash"`, `"notes"`, `"webhooks"` entries defined now so Phases 10-12 only pass a key; `_` → generic permission hint.
- Grep gates: 0 `401 | 403` combined arms; 3 `CliError::Forbidden` mappings; 2 `parse_rfc7807(&error_body` calls; 0 inline `v.get("error").or(...)` remnants; single `StatusCode::FORBIDDEN` reference belongs to a Forbidden mapping.

### Task 3 — Stub-server integration tests (TDD)
- **tests/error_layer_stub_test.rs**: `spawn_body_stub_server(&[(u16, &str)])` (batch_error_test.rs TcpListener pattern generalized to carry full response bodies — no new crates, per locked decision) + 7 CLI-level tests: 403 deals get (Forbidden + permission hint + server's voice, never bad-key hint), 403 ping (third site proven through the CLI), 401 regression (Auth + init hint), 409 workflows trigger (activation hint + verbatim server detail, quote-free), 422 create with `--title/--stage/--no-input` (joined errors[] text, never "Request validation failed"), 422 empty errors[] (detail fallback), 404 (server detail, never "HTTP 404").

## Commits

| Commit | Type | Description |
|--------|------|-------------|
| 3c663fc | test(08-01) | RED: failing tests for Forbidden variant + parse_rfc7807 |
| 76bcc3b | feat(08-01) | GREEN: Forbidden variant + parse_rfc7807 helper |
| ae2a9ec | feat(08-01) | Surface-keyed 401/403 split, all three sites + 409 hint |
| c8fe619 | test(08-01) | Stub-server integration tests (RED gate proven vs pre-plan code) |

## TDD Gate Compliance

- **Task 1**: proper RED→GREEN — RED commit `3c663fc` failed compilation (E0432 `parse_rfc7807`, E0599 `Forbidden`), GREEN `76bcc3b` passed 119 tests.
- **Task 3**: tests were written after Tasks 1-2 implemented the layer (plan sequencing), so they passed on first local run. The RED gate was **proven retroactively**: the test file was run against the pre-plan commit `c3c2dcf` in a throwaway worktree — **6/7 tests fail there** as expected (only the 401 regression passes, correctly, since 401 handling was unchanged). The suite genuinely discriminates the new layer.

## Deviations from Plan

None — plan executed exactly as written. (Task 2's call-site threading used a scripted occurrence-order replacement; a paren-placement slip during it was caught and corrected by the plan's own grep gates before commit.)

### Out-of-Scope Discoveries (logged, not fixed — pre-existing, unrelated to this plan)

1. **3 environment-dependent test failures, proven pre-existing at `c3c2dcf`** (this dev machine has a real `~/.pipelite/config.toml`, so commands succeed where the tests assume no-config failure): `cli_skeleton::config_set_parses_positional_args`, `cli_skeleton::global_flags_parse_without_error`, `deals_integration::deals_list_limit_zero_is_accepted`. Candidate fix (deferred): make these tests hermetic via a temp `HOME`/`XDG_CONFIG_HOME`.
2. **4 pre-existing build warnings** in untouched files: `workflows/list.rs` unused `total`, `models.rs` dead `WorkflowRunTrigger`, `cache.rs` dead `TTL_WORKFLOWS`, `prompt.rs` dead `get_workflows_cached`.

Both logged in `.planning/phases/08-foundations-error-layer-models-pagination/deferred-items.md`.

## Verification Results

- `cargo test --bin pipelite`: **119 passed** (14 new unit tests included)
- `cargo test --test error_layer_stub_test`: **7/7 passed**
- Full suite `cargo test --no-fail-fast`: 22 test binaries green; only the 3 pre-existing env failures above (reproduce identically at pre-plan HEAD — zero regressions from this plan)
- `grep -cE '401 \| 403' src/api/mod.rs` → 0; no JSON-quote extraction paths remain; no `unwrap()`/`expect()` in production code of touched files; Forbidden/409 hints carry the CONTEXT-locked actionable text

## Known Stubs

None — every error path is wired to real server data through `parse_rfc7807`; no placeholder rendering.

## Threat Model Compliance

- **T-08-01** (hostile bodies): `parse_rfc7807` is total — `serde ... .ok()` fallthrough, all extraction via `as_str()`, no unwrap/panic paths. Proven by `degenerate_empty_object_yields_http_status` + `non_json_body_yields_http_status`.
- **T-08-02** (verbose 403 hints): hints are accurate but minimal; 403-vs-401 distinction is the FIX-05 goal (AUDT-02 dependency).
- **T-08-03** (key leakage): detail sourced exclusively from response body or request URL — no credential interpolation.
- **T-08-SC**: no new crates; stub is hand-rolled on `std::net::TcpListener`.

No new security-relevant surface beyond the plan's threat model.

## Notes for Downstream Phases

- **Phases 9-12**: pass the surface key (`"notes"`, `"webhooks"`, `"audit"`, `"trash"`) to `handle_response`/`handle_delete_response` — the hint entries already exist.
- **Phase 13 docs pass**: `docs/api-reference.md:381` ("401, 403 → Auth") is now stale — 403 → Forbidden.
- `spawn_body_stub_server` in tests/error_layer_stub_test.rs is the reference pattern for future body-scripted stub tests.

## Self-Check: PASSED

All created files exist; all 4 task commits verified in git log (3c663fc, 76bcc3b, ae2a9ec, c8fe619). Full suite: 22 binaries green, only the 3 documented pre-existing environment failures (proven at pre-plan HEAD).
