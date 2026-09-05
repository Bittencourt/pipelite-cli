# E2E v1.1 — Live-Server Validation Report

> **Status: EXECUTED — 21 PASS / 0 FAIL.** Live run completed 2026-09-05 with
> the user's admin credentials (session environment only, never committed).
> This report is the SC-2 evidence for the v1.1 milestone audit.
>
> **The API key and webhook signing secret are NOT in this report.** The
> secret is recorded as "shown once: yes (64 chars, redacted)".

- **Date:** 2026-09-05
- **Server URL:** https://pipelite.pedrobittencourt.net
- **Key is ADMIN:** yes
- **Script:** `scripts/e2e-v1.1.sh` (release build, env-gated, e2e- prefixed throwaway records only)

## Results

| # | Scenario | Result | Evidence (one line) |
|---|----------|--------|---------------------|
| 1 | batch_exit_codes_quiet | **PASS** (7 asserts) | all-ok `--quiet` exit 0; malformed stdin exit 2 pre-HTTP; mixed batch exit 1 with "1/2 deal updated, 1 failed" summary |
| 2 | webhook_show_once_secret | **PASS** (5 asserts) | secret shown once: yes (64 chars, REDACTED); absent from list and get output |
| 3 | trash_round_trip | **PASS** (4 asserts) | org e2e-trash-* soft-deleted → found in `trash list --type orgs` → restored → fetchable; purge never invoked (script-guarded) |
| 4 | typed_write_server_side | **PASS** (3 asserts) | orgs get --format json: `.custom_fields.e2e_price == 4` with jq type `number` — Phase 12 deferred item closed server-side |

## Scenario Details

### 1. batch_exit_codes_quiet

- **Result:** PASS
- All-ok batch update under `--quiet`: exit 0 (silent by the locked Phase 7
  contract — the "N ok, M failed" summary prints on the failure path only).
- Malformed stdin: exit 2 before any HTTP.
- Mixed batch (one bogus ID): exit 1, summary "1/2 deal updated, 1 failed"
  present on stderr.
- Observed vs expected (if FAIL): n/a — all asserts passed.

### 2. webhook_show_once_secret

- **Result:** PASS
- Secret shown exactly once at create: yes — length 64 chars, value **REDACTED**.
- Secret absent from `webhooks list`: yes; absent from `webhooks get`: yes.
- Webhook deleted after the check: yes.
- Observed vs expected (if FAIL): n/a — all asserts passed.

### 3. trash_round_trip

- **Result:** PASS
- Round-trip: create → delete → `trash list --type orgs` contains the record:
  yes → `trash restore organizations <id>` exit 0 → `orgs get` returns
  the record: yes.
- `trash purge` invoked: no (grep-guarded in the script).
- **Documented leftover:** the cleanup re-delete leaves trash entries
  (`e2e-trash-<timestamp>` + scenario-A soft-deleted deals) — harmless,
  expected, documented here per plan; purge is available if desired.
- Observed vs expected (if FAIL): n/a — all asserts passed.

### 4. typed_write_server_side

- **Result:** PASS
- Definition `e2e_price` (number, orgs): created (6c3f0b06-a7ba-442d-abad-950d6d8f1a2a).
- Org created with `--custom-field e2e_price=4`; server-side read-back:
  `.custom_fields.e2e_price` = 4 with JSON type `number` (never `"4"`/string).
- This closes the Phase 12 deferred item (typed writing verified server-side).
- Cleanup: org deleted yes; definition deleted yes.
- Observed vs expected (if FAIL): n/a — all asserts passed.

## Forbidden (non-admin) checks

**SKIPPED live — no non-admin key available** (per 13-CONTEXT). The 403
permission paths (audit/audit-grade surfaces with a member key, webhook
ownership exclusivity, non-admin trash purge) are covered by stub tests:
`tests/error_layer_stub_test.rs` (plus the per-surface stub suites). The
milestone audit reads this note as the disposition for the live non-admin
matrix.

## Cleanup Accounting

- Throwaway records created: all prefixed `e2e-` (`e2e-batch-a-*`, `e2e-batch-b-*`,
  `e2e-hook-*` webhook, `e2e-trash-*` org, `e2e-typed-write-*` org).
- Leftovers after the run: exactly ONE trash entry (`e2e-trash-<timestamp>`),
  documented above. Everything else deleted by the script (trap + explicit cleanup).
- `trash purge` invocations: **0** (enforced by the `pl()` guard — exit 70 if attempted).
