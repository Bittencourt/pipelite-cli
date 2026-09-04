---
phase: 11-webhooks-trash-audit
reviewed: 2026-09-04T00:00:00Z
depth: standard
files_reviewed: 26
files_reviewed_list:
  - src/cli/webhooks.rs
  - src/cli/trash.rs
  - src/cli/audit.rs
  - src/cli/mod.rs
  - src/commands/webhooks/mod.rs
  - src/commands/webhooks/create.rs
  - src/commands/webhooks/list.rs
  - src/commands/webhooks/get.rs
  - src/commands/webhooks/update.rs
  - src/commands/webhooks/delete.rs
  - src/commands/trash/mod.rs
  - src/commands/trash/list.rs
  - src/commands/trash/restore.rs
  - src/commands/trash/purge.rs
  - src/commands/audit/mod.rs
  - src/commands/audit/list.rs
  - src/commands/mod.rs
  - src/main.rs
  - src/api/models.rs
  - src/api/mod.rs
  - src/cache.rs
  - tests/webhooks_stub_test.rs
  - tests/trash_stub_test.rs
  - tests/audit_stub_test.rs
  - tests/help_examples_test.rs
  - docs/api-reference.md
findings:
  critical: 1
  warning: 1
  info: 3
  total: 5
status: issues_found
---

# Phase 11: Code Review Report

**Reviewed:** 2026-09-04T00:00:00Z
**Depth:** standard
**Files Reviewed:** 26
**Status:** issues_found

## Summary

Phase 11 (webhooks / trash / audit) is a large, well-tested surface (~5,200 added lines, 83 stub/help tests, all green; `cargo build` clean). Adversarial verification of the locked contracts confirmed most of them hold **in code, not just in tests**: 9-alias normalization pre-HTTP, the four break conditions in both `--all`/purge fan-outs, purge's zero-HTTP InvalidInput refusal ordering, continue-on-error with `bail!` exit 1, audit empty-omit + 1..=100 clamp + no `--all`, the audit 403 hint, secret-never-cached (id/url only), and KEY_WEBHOOKS invalidation on all three webhook mutations.

However, one locked/global contract is **broken in code and untested**: `webhooks update --stdin` ignores `--dry-run` and performs the real PUT. This was confirmed empirically against an unreachable server (the CLI attempted a live connection; the flags path under the same conditions renders a preview and exits 0). One smaller validation gap was also confirmed empirically.

Build: `cargo build` OK (3 pre-existing dead-code warnings, none phase-11). Tests: webhooks 27, trash 19, audit 13, help 24 — all pass. Phase-11 code introduces no `unwrap()` in production paths.

## Critical Issues

### CR-01: `webhooks update --stdin` ignores `--dry-run` and executes the real PUT

**File:** `src/commands/webhooks/update.rs:41-44` (stdin early-return), `:102-107` (`execute()` has no dry-run guard); the dry-run intercept at `:75-82` is only reachable in flags mode
**Issue:** The stdin path returns `execute(ctx, args, &body).await` **before** the `ctx.dry_run` check, and `execute()` calls `ctx.client.update_webhook(...)` unconditionally. `pipelite webhooks update wh1 --stdin --dry-run` therefore mutates the live webhook when the user asked for a preview. This violates the global project convention ("Respect `--dry-run` (no HTTP calls)"), the command's own documented contract (`docs/api-reference.md`: "`--dry-run` previews the PUT with zero requests"), and the module doc's claim of the standard contract. **Empirically proven:** with stdin piped and `--dry-run` against `http://127.0.0.1:1`, the binary prints `error: Connection failed` and exits 1 (a live PUT was attempted), while the flags path under identical conditions renders the preview JSON and exits 0. The test suite pins dry-run zero-HTTP only for the flags path (`update_dry_run_zero_http`), so this gap is untested.

**Fix:**
```rust
if args.stdin {
    let body = read_stdin_body()?;
    if ctx.dry_run {
        let item_url = format!(
            "{}/api/v1/webhooks/{}",
            ctx.client.base_url(),
            args.webhook_id
        );
        return dry_run::render_dry_run("PUT", &item_url, &body, &ctx.output_format, ctx.color);
    }
    return execute(ctx, args, &body).await;
}
```
Add a stub test: `update_stdin_dry_run_zero_http` asserting `counter == 0` and a `PUT` preview on stdout (mirroring `update_dry_run_zero_http`).

## Warnings

### WR-01: Non-string entries in `--stdin` `events` arrays silently bypass the 13-event allow-list

**File:** `src/commands/webhooks/mod.rs:84-90` (`validate_stdin_body`, the `filter_map(|e| e.as_str() ...)`)

**Issue:** The module doc calls the allow-list "the CLI's only defense" for stdin bodies (the server accepts any strings and silently never fires unknown events). But `filter_map` **silently drops** non-string array entries from validation, so bodies like `{"events":[123]}` or `{"events":[null]}` pass the client-side gate untouched and go to the wire, relying on the server's 422 to catch them. Verified empirically: `echo '{"events":[123]}' | pipelite webhooks create --stdin --dry-run` passes validation (the raw body renders in the preview verbatim). The result is two contract inconsistencies: a structurally invalid body gets the runtime tier (server 422 → `Validation`, exit 1) instead of the locked client-side `InvalidInput` exit-2 tier, and malformed input is ignored rather than named.

**Fix:**
```rust
if let Some(events) = body.get("events").and_then(|v| v.as_array()) {
    for e in events {
        let Some(name) = e.as_str() else {
            return Err(CliError::InvalidInput {
                detail: format!("Webhook event entries must be strings (got {e})"),
                hint: format!("Valid events: {}", WEBHOOK_EVENTS.join(", ")),
            }
            .into());
        };
        validate_events(std::slice::from_ref(&name.to_string()))?;
    }
}
```
(or collect into `Vec<String>` while erroring on the first non-string), plus a stub test asserting exit 2 / zero HTTP for `{"events":[123]}`.

## Info

### IN-01: `KEY_WEBHOOKS` cache is write-only — no consumer exists

**File:** `src/cache.rs:36-41`, `src/commands/webhooks/list.rs:33-41`
**Issue:** The constant is documented as "completion candidates for webhook get/update/delete", but nothing ever reads it (`grep` shows only the definition in `cache.rs` and write/invalidate in the four webhook command files; webhooks commands take explicit IDs and have no fuzzy-select prompt). Every `webhooks list` writes an `(id, url)` JSON file to disk that no code path consumes. Presumably staged for a later phase; until then it is speculative I/O (harmless — ids/urls only, no secret, and invalidation is correctly wired).
**Fix:** Either wire the consumer (a prompt/completion path reading `cache.get::<Vec<(String, String)>>(KEY_WEBHOOKS)`), or defer the cache write until one exists.

### IN-02: Fan-out loop and constants duplicated within the trash module

**File:** `src/commands/trash/list.rs:11-18,92-108` vs `src/commands/trash/purge.rs:10-17,145-166`
**Issue:** `FANOUT_LIMIT`/`OFFSET_CAP` and the four-break-condition pagination loop are duplicated verbatim across the two files (per-entity `fetch_all` copies exist elsewhere in the codebase, so the pattern itself is established — but this is two copies of the same new logic added in the same phase, in the same module). If the cap or a break condition changes, one copy can silently drift from the other — and these loops guard against a non-terminating fan-out, so drift here is correctness-adjacent.
**Fix:** Extract a shared helper in `src/commands/trash/mod.rs`, e.g. `async fn paginate_all<T>(&self, trash_type: Option<&str>) -> Result<Vec<T>>` with a row-mapper, and reuse it from both `fetch_all` and `collect_victims`.

### IN-03: Update JSON output injects a `secret` placeholder its own doc says never exists in this flow

**File:** `src/commands/webhooks/update.rs:113-120` (vs the doc claim at `:27`)
**Issue:** `execute()` inserts `"secret": "(shown once at creation)"` into the JSON rendering of the PUT response, while the function doc states "No secret exists anywhere in this flow — PUT responses never carry it." No secret leaks (it is a placeholder), and it mirrors list/get's display convention, but the injected key is not wire-faithful for update output — scripts keying on `secret` get placeholder text for a field the server did not send, and the code contradicts its own doc. Not pinned by any test.
**Fix:** Either restrict the placeholder to list/get (where the contract locks it) and render the update response verbatim, or update the `:27` doc comment to state the placeholder is added deliberately for display consistency.

## Contract Verification (adversarial, code-level)

| Locked contract | Verdict |
|---|---|
| Webhooks: 13-event exit-2 pre-HTTP on flags **and** stdin arrays | Holds for string entries (pinned: `create_unknown_event`, `create_stdin_bad_event`, counter==0); **gap for non-string entries → WR-01** |
| Webhooks: https client-side exit 2, zero HTTP | Holds (`create_http_url`, pre-HTTP proven via no "Connection failed") |
| Secret ONLY in create response: full 64 chars, own line, warning, quiet-exempt; json body carries it with stderr warning | Holds (pinned: `secret_shown_once_full_on_own_line` incl. no-disk walk, `secret_json_format`) |
| "(shown once at creation)" in list/get; secret NEVER cached (id/url only) | Holds (`list_json_placeholder`, `get_placeholder`; cache write is `(id, url)` pairs only) |
| Update get→merge→PUT, omitted keys = no-op; flags validated before GET | Holds (pinned: `update_merges`, `update_flag_validation_precedes_get`) — **but stdin path breaks the --dry-run contract → CR-01** |
| Delete: dry-run → TTY confirm → --force; non-TTY exit 1 Validation, zero HTTP | Holds (`delete_dry_run`, `delete_non_tty_refusal` counter==0) |
| Trash: 9-alias → plural tabs on wire | Holds (all 9 pinned in unit tests; wire-level `type=deals`/`people`/`organizations`/`activities` pinned in stub tests) |
| `--all` fan-out, 4 break conditions (empty / partial / accumulated ≥ total / offset > 10,000) | Holds in both copies; empty-page and partial-page breaks pinned; cap break present in code |
| linked_parents truncated + full json; deleted_by api_key nameless | Holds (`list_linked_parents_truncation`, `list_deleted_by_variants`) |
| Restore no-confirm; 404 re-wrap preserves detail + not-in-trash hint; 403 general hint | Holds (`restore_deal_alias`, `restore_404_rewrap`, `restore_403_general_hint`) |
| Purge: scope validated pre-HTTP → strongest confirm (scope+count+"permanently destroys") → --force bypass → non-TTY no-force = InvalidInput exit 2 ZERO HTTP | Holds in code and pinned (`purge_scope_validation_first`, `purge_refusal_zero_http` on an unreachable server, `prompt_wording_names_scope_count_and_destruction`) |
| Purge fan-out continue-on-error → N ok/M failed, `bail!` exit 1 (NOT Validation) | Holds (`purge_continue_on_error` — exit 1, all victims attempted; `error.rs:87-100` maps plain bail to 1) |
| Trash never cached | Holds — no cache touch anywhere in `commands/trash/` |
| Audit: 4 filters empty-omit on wire | Holds (`empty_flag_omitted`, `filters_verbatim`; `api/mod.rs:1280-1291` guards `Some && !is_empty`) |
| Audit: limit clamp 1..=100; NO --all | Holds (`limit_clamped_low`, `limit_clamped_high`; no `--all` arg exists in `cli/audit.rs`) |
| Audit: 403 "audit log requires an admin key" before filter validation | Holds (`forbidden_empty_filter_renders_audit_hint`, `forbidden_invalid_filter_still_gate` — invalid value proven to reach the wire) |
| Audit: changes payload --json only; never cached | Holds (changes absent from `audit_table_config` columns; no cache in `commands/audit/`) |
| KEY_WEBHOOKS invalidation on webhook mutations | Holds — create (`create.rs:99-101`), update (`update.rs:109-111`, both paths via `execute`), delete (`delete.rs:61-63`); pinned by `delete_invalidates_cache` |

---

_Reviewed: 2026-09-04T00:00:00Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: standard_
