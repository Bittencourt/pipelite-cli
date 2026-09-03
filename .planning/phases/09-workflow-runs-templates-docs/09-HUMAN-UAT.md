---
status: partial
phase: 09-workflow-runs-templates-docs
source: [09-VERIFICATION.md]
started: 2026-09-03
updated: 2026-09-03
---

## Current Test

[awaiting live-server human testing]

## Tests

### 1. Live `--watch`
expected: Trigger a workflow run against the live server, then `pipelite workflows runs get <run_id> --workflow <wf_id> --watch` — stderr state-change lines appear, final state renders, exit 0. Ctrl-C during watch → shell exit code 130. `--exit-status` on a failed run → exit 1.
result: [pending]

### 2. Live `docs`
expected: `pipelite docs` renders the ~87KB OpenAPI 3.1 spec; `pipelite docs --save /tmp/spec.json` writes it; server request log shows no Authorization header on this request.
result: [pending]

### 3. Live dry-run hiding
expected: `pipelite workflows runs list --workflow <id>` excludes test runs; test runs created via server UI paths appear only with `--include-dry-run`.
result: [pending]

## Summary

total: 3
passed: 0
issues: 0
pending: 3
skipped: 0
blocked: 0

## Gaps
