---
status: partial
phase: 11-webhooks-trash-audit
source: [11-VERIFICATION.md]
started: 2026-09-03
updated: 2026-09-03
---

## Current Test

[awaiting human testing — TTY/live items]

## Tests

### 1. Interactive `trash purge` confirm
expected: `pipelite trash purge` (or `--type <t>`) in a real terminal shows the strongest-confirm prompt naming scope + count + "permanently destroys" (default No); declining aborts with zero HTTP; `--no-input` without `--force` → exit 2.
result: [pending]

### 2. Interactive `webhooks delete` confirm
expected: `pipelite webhooks delete <id>` without `--force` in a real terminal shows the confirm prompt (default No); declining aborts; accepting deletes.
result: [pending]

### 3. Live-server admin smoke (audit + purge)
expected: With the admin key: `pipelite audit list` renders real entries (columns timestamp/actor/action/entity); `trash purge --force` on a throwaway trashed record permanently removes it; non-admin key (if available) gets "audit log requires an admin key" Forbidden.
result: [pending]

### 4. Visual table inspection
expected: webhook/trash/audit tables truncate cleanly (no wrap garbage), colors respect --no-color, secret placeholder shows "(shown once at creation)".
result: [pending]

## Summary

total: 4
passed: 0
issues: 0
pending: 4
skipped: 0
blocked: 0

## Gaps
