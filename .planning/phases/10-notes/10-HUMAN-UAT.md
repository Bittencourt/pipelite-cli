---
status: partial
phase: 10-notes
source: [10-VERIFICATION.md]
started: 2026-09-03
updated: 2026-09-03
---

## Current Test

[awaiting human testing of interactive TTY paths]

## Tests

### 1. Interactive add/edit prompt
expected: `pipelite notes add deals <deal-id>` with no body source in a real terminal shows the body prompt (edit carries the "no single-note GET" wording); typed text posts to the server as `{"content": ...}`; `--no-input` without a source → exit 2 MissingInput.
result: [pending]

### 2. Interactive delete confirmation
expected: `pipelite notes delete <type> <parent-id> <note-id>` without `--force` in a real terminal shows the confirm prompt (default No); declining prints "Aborted" with zero HTTP; accepting deletes (subsequent list no longer shows the note).
result: [pending]

## Summary

total: 2
passed: 0
issues: 0
pending: 2
skipped: 0
blocked: 0

## Gaps
