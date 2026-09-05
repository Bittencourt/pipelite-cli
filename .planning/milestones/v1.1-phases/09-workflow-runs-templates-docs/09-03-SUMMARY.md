---
phase: 09-workflow-runs-templates-docs
plan: 03
subsystem: docs-command
tags: [docs, openapi, unauthenticated, api-client, cli, stub-tests]
requires:
  - 09-01/09-02 seam: tests/common/mod.rs 4-tuple stub helper (heads + full bodies)
  - Phase 8 error layer (handle_response, parse_rfc7807 legacy error-key fallback)
  - PipeliteClient::from_credentials default_headers (the leak vector the dedicated client avoids)
provides:
  - "pipelite docs: fetch the server's OpenAPI 3.1 spec from the public /api/v1/docs route with NO Authorization header on the wire (ROADMAP SC-5)"
  - "PipeliteClient::get_docs building a dedicated headerless reqwest::Client (same 5s/30s timeouts, 429 retry via send_with_retry)"
  - "--save FILE with pre-HTTP overwrite refusal unless --force (exit 2), parent-dir creation, quiet-aware byte-count confirmation"
  - "Docs-endpoint 404/Api errors re-wrapped with the locked hint 'the server may not expose the docs endpoint — check server version', server detail preserved"
  - "--format accepted but ignored, documented in help and after_help examples"
affects:
  - scripts/agents reading the API contract from the terminal (DOCS-01)
  - Phase 13 cross-verification of shared conventions (after_help examples, hint-bearing errors)
tech-stack:
  added: []
  patterns:
    - dedicated headerless reqwest::Client inside a client method (default_headers cannot be removed per-request)
    - command-level error re-wrap (NotFound/Api detail-preserving hint substitution) instead of touching the shared mapper
    - pre-HTTP filesystem refusal pinned by a 0-request stub test against a reachable server
key-files:
  created:
    - src/cli/docs.rs
    - src/commands/docs.rs
    - tests/docs_stub_test.rs
  modified:
    - src/api/mod.rs
    - src/cli/mod.rs
    - src/commands/mod.rs
    - src/main.rs
    - tests/help_examples_test.rs
decisions:
  - "get_docs builds a local bare reqwest::Client — the shared authenticated client installs the API key in default_headers which reqwest attaches to EVERY request with no per-request removal; gate greps prove zero references to the shared instance inside the method body"
  - "Overwrite refusal fires BEFORE the fetch (zero HTTP on refusal) — tested against a reachable server via the stub request counter, stronger than the unreachable-server trick"
  - "Docs errors re-wrap at the command layer (NotFound/Api keep the server's detail, substitute the CONTEXT-locked hint) — handle_response stays untouched for every other surface"
  - "fs::create_dir_all/write failures map through CliError::Validation (runtime condition, exit 1) with actionable hints per the all-errors-carry-hints convention"
metrics:
  duration: 12 min
  completed: 2026-09-03T20:08:00Z
  tasks: 2
  files: 8
---

# Phase 9 Plan 03: Docs Command Summary

**One-liner:** `pipelite docs [--save FILE]` fetches the OpenAPI 3.1 spec over a deliberately headerless HTTP client (no API key on the wire — proven by a raw request-head assertion), pretty-prints or saves it with pre-HTTP overwrite refusal, and re-wraps docs errors with the locked server-version hint.

## What Was Built

- **Task 1 — Unauthenticated get_docs + Docs command** (9cda7f2): `PipeliteClient::get_docs()` builds a LOCAL `reqwest::Client::builder()` (5s connect / 30s total timeouts, no `default_headers`) because the shared authenticated client installs the API key in `default_headers`, which ride along on every request from that instance. The GET goes through `send_with_retry` (429 retry free) and maps through the standard `handle_response` arms with surface `"docs"`. New `DocsArgs {--save, --force}` with after_help examples and the format-ignored note; `Docs(DocsArgs)` wired into `Commands` + dispatch. The handler refuses to overwrite an existing file BEFORE any HTTP (exit 2 `InvalidInput` with the `--force` hint), creates missing parent dirs, writes the pretty string, and prints `Wrote OpenAPI spec to {FILE} ({N} bytes)` unless quiet. 404/Api errors are re-wrapped preserving the server's detail and substituting the CONTEXT-locked hint; Connection/Auth pass through. No cache interaction; nothing else on stdout.
- **Task 2 — Integration proof** (9ba2361): 8 `docs_stub_test.rs` tests + 1 help test: the wire request head contains NO `authorization:` line (exactly 1 request), compact stub JSON renders pretty (`"openapi": "3.1.0"` with the space), `--save` creates nested parent dirs and confirms, refusal without `--force` exits 2 with sentinel content intact and a 0-request counter, `--force` overwrites, `-q` suppresses the confirmation while still writing, 404 renders the locked hint with `"Not Found"` detail preserved and the generic 404 hint absent, legacy 500 body renders `OpenAPI specification not available` unquoted (exit 1 Api arm), and `--format csv` is inert.

## Tasks Completed

| Task | Name | Commit(s) | Files |
| ---- | ---- | --------- | ----- |
| 1 | Unauthenticated get_docs + Docs command | 9cda7f2 | src/api/mod.rs, src/cli/{docs.rs,mod.rs}, src/commands/{docs.rs,mod.rs}, src/main.rs |
| 2 | Docs integration tests + help examples | 9ba2361 | tests/docs_stub_test.rs, tests/help_examples_test.rs |

## Verification Results

- Full suite: **367 passed, 0 failed** (28 test binaries; baseline 358 after 09-02 → +9)
- Task gates: both `<verify><automated>` gates printed GATE-OK (grep-proven: one `get_docs` definition, zero references to the shared client instance inside its body, one local `reqwest::Client::builder`, wiring complete, hint + create_dir_all + format-note present)
- No-auth proof is wire-level: the stub records the raw lowercased request head and asserts zero `authorization:` lines — the contract is tested on the wire, not assumed from code inspection
- `docs --help` smoke-tested on the built binary (Examples + format-ignored note render; exit 0)
- No `unwrap()`/`expect()` in new production code; every new error carries a hint; zero new crates; docs never touches the cache

## Deviations from Plan

**1. [Rule 3 - Gate/naming] Help-gate grep needed the literal `docs --help` in help_examples_test.rs**
- **Found during:** Task 2 (verify gate)
- **Issue:** The gate `grep -c 'docs --help' tests/help_examples_test.rs >= 1` returned 0 — the test uses the repo's `.args(["docs", "--help"])` array form, so the literal string never appears
- **Fix:** Phrased the test's doc comment to carry the literal invocation (`The \`pipelite docs --help\` page ...`), matching the file's existing comment convention of listing exact help invocations
- **Files modified:** tests/help_examples_test.rs
- **Commit:** 9ba2361

**2. [Note - expected TDD behavior] Task 2 tests passed at first run**
- The docs integration tests compiled and passed immediately because the functionality landed in Task 1 (the plan sequences implementation before integration proof, as 09-01/09-02 did). No RED gate was possible without artificially breaking landed code; committed as a single `test(09-03)` commit per the wave-2 precedent.

## Auth Gates

None.

## Known Stubs

None. The docs command is fully wired to `get_docs`; `--format` being inert is the locked, help-documented design (not a stub); docs responses are never cached by decision.

## Threat Flags

None — no trust-boundary surface beyond the plan's threat model. All three mitigate-disposition threats are implemented and test-covered: T-09-08 (dedicated headerless client + wire-level no-Authorization assertion), T-09-09 (pre-HTTP overwrite refusal tested with sentinel file + 0-request counter), T-09-10 (spec rendered via `to_string_pretty` on the parsed Value; error details via `parse_rfc7807` as_str — never raw byte dumps).

## Self-Check: PASSED

- FOUND: src/cli/docs.rs, src/commands/docs.rs, tests/docs_stub_test.rs
- FOUND modified: src/api/mod.rs, src/cli/mod.rs, src/commands/mod.rs, src/main.rs, tests/help_examples_test.rs
- FOUND commits: 9cda7f2, 9ba2361 (both in `git log`)
