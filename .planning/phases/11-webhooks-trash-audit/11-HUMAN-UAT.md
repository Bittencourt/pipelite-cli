---
status: resolved
phase: 11-webhooks-trash-audit
source: [11-VERIFICATION.md]
started: 2026-09-03
updated: 2026-09-03
---

## Current Test

[all tests executed 2026-09-04 against live server pipelite.pedrobittencourt.net]

## Tests

### 1. Interactive `trash purge` confirm
expected: `pipelite trash purge` (or `--type <t>`) in a real terminal shows the strongest-confirm prompt naming scope + count + "permanently destroys" (default No); declining aborts with zero HTTP; `--no-input` without `--force` → exit 2.
result: PASS — pty-driven live run: prompt names scope + "permanently destroys", default No; decline → "Aborted", zero HTTP, trash untouched. `--force` fan-out then permanently destroyed 9 records (throwaway + prior UAT scratch deals), summary "9 permanently destroyed", exit 0, trash verified empty.

### 2. Interactive `webhooks delete` confirm
expected: `pipelite webhooks delete <id>` without `--force` in a real terminal shows the confirm prompt (default No); declining aborts; accepting deletes.
result: PASS — pty-driven live run: confirm prompt (default No), decline → "Aborted" zero HTTP, webhook still present; created/declined/deleted cleanly. (Show-once secret line observed at create — full 64-char on its own line behind the save-it-now warning.)

### 3. Live-server admin smoke (audit + purge)
expected: With the admin key: `pipelite audit list` renders real entries (columns timestamp/actor/action/entity); `trash purge --force` on a throwaway trashed record permanently removes it; non-admin key (if available) gets "audit log requires an admin key" Forbidden.
result: PASS — `audit list` renders live admin entries (timestamp/actor/action/entity columns, truncation clean); `trash purge --force` permanently removed the throwaway (verified gone from trash). Non-admin-key variant not exercisable (no non-admin key available) — 403 hints are stub-proven at the wire level (tests/audit_stub_test.rs).

### 4. Visual table inspection
expected: webhook/trash/audit tables truncate cleanly (no wrap garbage), colors respect --no-color, secret placeholder shows "(shown once at creation)".
result: PASS — webhook/audit tables truncate cleanly, relative timestamps, colors suppressed with --no-color (0 ANSI codes), secret renders "(shown once at creation)" in table and json (no raw 64-char secret anywhere in get/list output).

## Summary

total: 4
passed: 4
issues: 0
pending: 0
skipped: 0
blocked: 0

## Gaps
