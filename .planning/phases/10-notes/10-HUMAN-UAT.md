---
status: resolved
phase: 10-notes
source: [10-VERIFICATION.md]
started: 2026-09-03
updated: 2026-09-03
---

## Current Test

[all tests executed 2026-09-03 against live server pipelite.pedrobittencourt.net]

## Tests

### 1. Interactive add/edit prompt
expected: `pipelite notes add deals <deal-id>` with no body source in a real terminal shows the body prompt (edit carries the "no single-note GET" wording); typed text posts to the server as `{"content": ...}`; `--no-input` without a source → exit 2 MissingInput.
result: PASS — pty-driven live run: "Add note: Note content:" prompt rendered, typed body posted, live `notes list --json` shows exact content, exit 0. Edit prompt confirmed to carry the "no single-note GET" wording; edited content verified server-side. (`--no-input` missing-source → exit 2 is stub-test-locked.)

### 2. Interactive delete confirmation
expected: `pipelite notes delete <type> <parent-id> <note-id>` without `--force` in a real terminal shows the confirm prompt (default No); declining prints "Aborted" with zero HTTP; accepting deletes (subsequent list no longer shows the note).
result: PASS — pty-driven live run: decline → "Aborted", note intact (list still 1); accept → deleted (list shows 0 + the quiet-suppressible empty-list hint rendered).

## Summary

total: 2
passed: 2
issues: 0
pending: 0
skipped: 0
blocked: 0

## Gaps
