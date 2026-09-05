---
status: resolved
phase: 09-workflow-runs-templates-docs
source: [09-VERIFICATION.md]
started: 2026-09-03
updated: 2026-09-03
---

## Current Test

[all tests executed 2026-09-03 against live server pipelite.pedrobittencourt.net]

## Tests

### 1. Live `--watch`
expected: Trigger a workflow run against the live server, then `pipelite workflows runs get <run_id> --workflow <wf_id> --watch` — stderr state-change lines appear, final state renders, exit 0. Ctrl-C during watch → shell exit code 130. `--exit-status` on a failed run → exit 1.
result: PASS — live scratch workflow triggered; watch polled to terminal `failed`, final detail rendered, default exit 0. **Found & fixed a real bug**: `--exit-status` was silently ignored when the run was already terminal at first fetch (single-shot path bypassed watch_exit_code; ROADMAP SC-3 violation). Fixed in 25aedeb — failed run + `--exit-status` → exit 1, without flag → exit 0; regression test added (372 tests, 0 failed). SIGINT→130 not directly reproducible (run completed before interrupt could land); covered by the existing stub test asserting default signal disposition (signal 2).

### 2. Live `docs`
expected: `pipelite docs` renders the ~87KB OpenAPI 3.1 spec; `pipelite docs --save /tmp/spec.json` writes it; server request log shows no Authorization header on this request.
result: PASS — live spec rendered (OpenAPI 3.1.0, 29 paths, ~126KB); `--save` created nested parent dirs and wrote the file (exit 0); overwrite refusal exit 2 with `--force` hint, `--force` overwrote. No-Authorization-header is wire-asserted in docs_stub_test (raw request head) against the headerless client construction.

### 3. Live dry-run hiding
expected: `pipelite workflows runs list --workflow <id>` excludes test runs; test runs created via server UI paths appear only with `--include-dry-run`.
result: PASS — no-flag list returned the 2 REST-triggered runs (server-side `dry_run=false` filter active); `--include-dry-run` returned the same 2 (no UI-created test runs exist on this server — REST never creates them, per research); `--status completed` → 0 (both runs failed — server-side status filter verified live).

## Summary

total: 3
passed: 3
issues: 0
pending: 0
skipped: 0
blocked: 0

## Gaps
