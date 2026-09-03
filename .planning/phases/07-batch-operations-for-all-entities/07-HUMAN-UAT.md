---
status: partial
phase: 07-batch-operations-for-all-entities
source: [07-VERIFICATION.md]
started: 2026-09-03
updated: 2026-09-03
---

## Current Test

[awaiting human testing]

## Tests

### 1. Interactive batch delete confirmation prompt
expected: Running `pipelite deals delete deal_1 deal_2` from a real terminal shows the "Delete 2 deal(s)?" confirmation prompt (default No). Answering No aborts before any HTTP call. Answering Yes proceeds. `--no-input` refuses without `--force`; `--dry-run` previews without prompting.
result: [pending]

### 2. Live-server batch success path
expected: Piping a JSON array to `pipelite deals update --stdin` and running a multi-ID `pipelite deals delete <id1> <id2> --force` against a real Pipelite instance renders the results table / "Deleted" lines, invalidates the relevant cache entries (subsequent `deals list` / `stages list` reflect changes), and exits 0.
result: [pending]

## Summary

total: 2
passed: 0
issues: 0
pending: 2
skipped: 0
blocked: 0

## Gaps
