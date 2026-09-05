# E2E v1.1 — Live-Server Validation Report

> **Status: TEMPLATE — not yet executed.** The live run requires the user's
> admin credentials (session environment only: `PIPELITE_SERVER_URL` +
> `PIPELITE_API_KEY`, never committed). Run `bash scripts/e2e-v1.1.sh`, then
> fill every `Result:` and evidence placeholder below from its output. The
> filled report is the SC-2 evidence for the v1.1 milestone audit.
>
> **Never paste the API key or the webhook signing secret into this report.**
> Record the secret as "shown once: yes/no (64 chars, redacted)".

- **Date:** `<YYYY-MM-DD>`
- **Server URL:** `<PIPELITE_SERVER_URL — same value, no key>`
- **Key is ADMIN:** `<yes/no>`
- **Script:** `scripts/e2e-v1.1.sh` (release build, env-gated, e2e- prefixed throwaway records only)

## Results

| # | Scenario | Result | Evidence (one line) |
|---|----------|--------|---------------------|
| 1 | batch_exit_codes_quiet | `Result: PASS/FAIL/SKIPPED` | `<all-ok --quiet exit 0; malformed stdin exit 2 pre-HTTP; mixed batch exit 1 with "1/2 deal updated, 1 failed" summary>` |
| 2 | webhook_show_once_secret | `Result: PASS/FAIL/SKIPPED` | `<secret shown once: yes/no (64 chars, REDACTED); absent from list and get>` |
| 3 | trash_round_trip | `Result: PASS/FAIL/SKIPPED` | `<org e2e-trash-* soft-deleted -> found in trash list --type orgs -> restored -> fetchable; purge never invoked>` |
| 4 | typed_write_server_side | `Result: PASS/FAIL/SKIPPED` | `<orgs get --format json: .custom_fields.e2e_price == 4 with jq type "number" — Phase 12 deferred item closed server-side>` |

## Scenario Details

### 1. batch_exit_codes_quiet

- **Result:** `PASS/FAIL/SKIPPED`
- All-ok batch update under `--quiet`: exit `<0>` (silent by the locked Phase 7
  contract — the "N ok, M failed" summary prints on the failure path only).
- Malformed stdin: exit `<2>` before any HTTP.
- Mixed batch (one bogus ID): exit `<1>`, summary `<1/2 deal updated, 1 failed>`
  present on stderr.
- Observed vs expected (if FAIL): `<paste the script's line>`

### 2. webhook_show_once_secret

- **Result:** `PASS/FAIL/SKIPPED`
- Secret shown exactly once at create: `<yes/no>` — length `<64>` chars, value **REDACTED**.
- Secret absent from `webhooks list`: `<yes/no>`; absent from `webhooks get`: `<yes/no>`.
- Webhook deleted after the check: `<yes/no>`.
- Observed vs expected (if FAIL): `<paste the script's line>`

### 3. trash_round_trip

- **Result:** `PASS/FAIL/SKIPPED`
- Round-trip: create → delete → `trash list --type orgs` contains the record:
  `<yes/no>` → `trash restore organizations <id>` exit `<0>` → `orgs get` returns
  the record: `<yes/no>`.
- `trash purge` invoked: `<no — must always be no>` (grep-guarded in the script).
- **Documented leftover:** the cleanup re-delete leaves ONE trash entry
  (`e2e-trash-<timestamp>`) — harmless, expected, documented here per plan.
- Observed vs expected (if FAIL): `<paste the script's line>`

### 4. typed_write_server_side

- **Result:** `PASS/FAIL/SKIPPED`
- Definition `e2e_price` (number, orgs): `<created/reused id>`.
- Org created with `--custom-field e2e_price=4`; server-side read-back:
  `.custom_fields.e2e_price` = `<4>` with JSON type `<number>` (never `"4"`/string).
- This closes the Phase 12 deferred item (typed writing verified server-side).
- Cleanup: org deleted `<yes/no>`; definition deleted `<yes/no>`.
- Observed vs expected (if FAIL): `<paste the script's line>`

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
