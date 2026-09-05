---
phase: 11-webhooks-trash-audit
verified: 2026-09-04T04:11:56Z
status: passed
score: 5/5 must-haves verified
overrides_applied: 0
human_verification:
  - test: "Run `pipelite trash purge` interactively in a real terminal on a throwaway record — check the dialoguer prompt renders the strongest-confirm wording (scope + count + 'permanently destroys'), defaults to No on bare Enter, and prints 'Aborted' on decline"
    expected: "Prompt appears with scope/count wording; declining exits 0 with 'Aborted' and issues zero DELETEs"
    why_human: "dialoguer requires a real TTY; the wording is unit-gated but the interactive flow (render, default, abort path) cannot be driven programmatically without a pty harness"
  - test: "Run `pipelite webhooks delete <id>` interactively in a real terminal and decline the prompt"
    expected: "Confirm prompt (default false), 'Aborted' on decline, zero HTTP"
    why_human: "Same TTY-only dialoguer path"
  - test: "Against a live Pipelite server with a real admin key: run `audit list`, `trash list`, and `trash purge --force <throwaway-record>`"
    expected: "Real audit entries render (actor/action/entity cells); purge destroys the record permanently and prints '1 permanently destroyed'; non-admin key run of audit list shows the admin hint"
    why_human: "Stub servers approximate the wire; live-server auth, real RFC 7807 shapes, and an irreversible destruction require human judgment on a real environment"
  - test: "Visually inspect table output (webhooks list, trash list with long linked_parents, audit list) in a color terminal at typical widths"
    expected: "Truncated cells end in '...' and remain readable; colors/wrapping look correct"
    why_human: "Terminal rendering quality (wrap points, color, readability) is visual"
---

# Phase 11: Webhooks, Trash & Audit Log Verification Report

**Phase Goal:** Users can manage automation integrations and recover from mistakes — with admin-gated and irreversible operations failing safe
**Verified:** 2026-09-04T04:11:56Z
**Status:** human_needed (all machine-verifiable truths VERIFIED; 4 TTY/live-only items remain)
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths (ROADMAP success criteria are the contract)

| #  | Truth                                                                                                                                                                                                                                       | Status     | Evidence |
|----|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|-----------|----------|
| 1  | Webhooks list/get/create/update(PUT)/delete; unknown event rejected client-side exit 2 listing all 13 events                                                                                                                                | ✓ VERIFIED | `Commands::Webhooks` dispatch (main.rs:106); 6 client methods (api/mod.rs:1102–1180); `WEBHOOK_EVENTS: [&str; 13]` (commands/webhooks/mod.rs:21–35); live probe: `--events deal.created,deal.archived` → exit 2, stderr names `deal.archived` + lists `deal.stage_changed`, zero "Connection failed"; update = get→merge→PUT (update.rs:98 `get_webhook` → merged PUT, wire-tested in `update_merges`); foreign 403 renders "This webhook belongs to another user." (live probe, exit 1) |
| 2  | Signing secret shown exactly once on create — full, untruncated, own line, save-it-now warning — never in list/get output or the cache                                                                                                       | ✓ VERIFIED | **Live stub probe**: exit 0; exactly 1 stdout line equals the full 64-char secret; line before it is `Signing secret (save it now — shown only once):`; length exactly 64 (no 120-col truncation — secret bypasses table cells via raw println, create.rs render_created); secret occurs exactly once; 0 files under redirected HOME contain it; list --json shows "(shown once at creation)" placeholder and zero 64-hex; cache file `webhooks.json` contains only `["wh1","https://example.com/hook"]` pairs; "secret" appears nowhere in cache.rs (grep = 0) |
| 3  | Trash list with `--type` filter + restore by type + ID; piped round-trips work via singular/plural normalization                                                                                                                             | ✓ VERIFIED | `normalize_trash_type` (9 aliases → 4 plural tabs) called in ALL three subcommands (list.rs:38, restore.rs:24, purge.rs:51) BEFORE any HTTP; live probes: `restore bogus` → exit 2 "Unknown trash type", zero HTTP; `--type bogus` → exit 2 zero HTTP; TrashRow carries plural `type` tab (the URL token) + singular entity_type; stub tests prove restore/purge URLs contain only plural tabs (`post /api/v1/trash/deals/t1/restore`) |
| 4  | Purge is permanent, admin-only, strongest confirmation in the codebase (--force bypass); --no-input without --force refuses exit 2 with zero HTTP                                                                                            | ✓ VERIFIED | Live probes: `trash purge` non-TTY → exit 2, stderr has "permanently" + "--force", zero "Connection failed" (no server existed); `--no-input` → "Refusing to permanently destroy…" exit 2; `purge --type bogus` → exit 2 zero HTTP (normalization precedes the gate); `purge_prompt` pure fn contains scope + count + "permanently destroys" (unit-gated, purge.rs:24–26); `CliError::InvalidInput` (exit 2) fires BEFORE the fan-out list (purge.rs:74–80); word "Validation" absent from purge.rs; admin 403 → "Permanent purge requires an admin API key." (stub test); fan-out continue-on-error with "N permanently destroyed / M failed" (stub tests) |
| 5  | Audit list with all four filters verbatim; non-admin keys get first-class Forbidden hint on every admin-gated surface                                                                                                                        | ✓ VERIFIED | `list_audit` (api/mod.rs:1266–1295) appends a filter pair ONLY when Some AND non-empty (`is_empty` check — server 422s empty values); limit clamped 1..=100 (list.rs:36, wire-tested 0→1 and 150→100); snake_case server keys (entity_type/entity_id/actor_kind/workflow_run_id — stub-tested on the wire); no `--all` flag (only the after_help note "(no --all)"); **live 403 probe**: `audit list` → exit 1 + "The audit log requires an admin API key."; purge 403 → purge hint; foreign webhook 403 → ownership hint (all three surfaces probed) |

**Score:** 5/5 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
| -------- | -------- | ------ | ------- |
| `src/api/models.rs` | Webhook (no secret/description) + WebhookCreated (flatten+secret) + WebhookCreate {url,events} + TrashRow (dual type tokens) + DeletedBy (tag="kind", 7 variants) + AuditEntry (verbatim changes) + 3 table configs | ✓ VERIFIED | All 6 structs + 3 configs at models.rs:788–932; serializer-exact per unit tests; TrashRow.tab renamed from wire "type"; DeletedBy closed union rejects unknown kinds |
| `src/api/mod.rs` | 6 webhook + 3 trash + 1 audit client methods with correct surfaces | ✓ VERIFIED | methods at 1102–1180 (webhooks, all surface "webhooks"), 1199–1244 (trash: list/restore "general", purge "trash"), 1266–1295 (audit, surface "audit"); forbidden_hint table (94–101) has all four pre-registered keys |
| `src/cache.rs` | KEY_WEBHOOKS + TTL_WEBHOOKS; word "secret" absent | ✓ VERIFIED | lines 39–41; grep -ci secret = 0; live probe: cache file holds (id, url) pairs only |
| `src/cli/webhooks.rs` | 5 subcommands, --stdin flags, 13 events in create after_help, no "description" | ✓ VERIFIED | 111 lines; grep -ci description = 0; create --help contains deal.stage_changed + save-it-now wording, no --description (probe P1) |
| `src/cli/trash.rs` | List/Restore/Purge, plain-String types (not ValueEnum), per-subcommand after_help | ✓ VERIFIED | 72 lines; group help says "permanently destroys"; restore help says "no confirmation"; purge help has --force; list help has jq round-trip + 10,000 cap (probe) |
| `src/cli/audit.rs` | List with 4 filter flags, clamped limit, enums in after_help, NO --all flag | ✓ VERIFIED | 46 lines; after_help lists 6 entity types, 5 actor kinds, 4 actions, "iterate --offset (no --all)"; no --all flag definition (probe P9) |
| `src/commands/webhooks/*` | dispatch + validators + 5 handlers with pre-HTTP validation, show-once render, cache invalidation | ✓ VERIFIED | mod.rs:210 (13-event list, validate_events/validate_https_url/validate_stdin_body + WR-01 non-string rejection); create.rs:100 + update.rs:122 + delete.rs:62 all invalidate KEY_WEBHOOKS; delete runs the standard exit-1 Validation contract (distinct from purge's exit 2) |
| `src/commands/trash/*` | normalize + list --all fan-out (10,000 cap) + restore (no confirm, 404 re-wrap) + purge (ordered contract) | ✓ VERIFIED | list.rs:18 OFFSET_CAP 10_000, zero cache references; restore.rs:13 not-in-trash hint, dry-run preview; purge.rs order: normalize → dry-run → zero-HTTP refusal → collect → confirm → continue-on-error → bail summary |
| `src/commands/audit/*` | dispatch + list (clamp, composed cells, empty-omit passthrough) | ✓ VERIFIED | list.rs:36 clamp; truncate_with_ellipsis cells; no cache; no changes column |
| `tests/webhooks_stub_test.rs` | ≥27 tests, secret tests first | ✓ VERIFIED | 29 tests green (27 planned + 2 review-fix: CR-01 dry-run-stdin, WR-01 non-string events); first test = secret_shown_once_full_on_own_line |
| `tests/trash_stub_test.rs` | ≥24 tests, purge refusal pinned | ✓ VERIFIED | 24 tests green |
| `tests/audit_stub_test.rs` | ≥12 tests, 403 probes pinned | ✓ VERIFIED | 13 tests green |
| `docs/api-reference.md` | `pipelite webhooks` / `pipelite trash` / `pipelite audit` sections | ✓ VERIFIED | all three `##` sections present before Error Codes; "permanently destroys", "admin API key", 13 events, exit-2 refusal documented |

### Key Link Verification

| From | To | Via | Status | Details |
| ---- | -- | --- | ------ | ------- |
| main.rs | webhooks/trash/audit handlers | `Commands::{Webhooks,Trash,Audit}` arms | ✓ WIRED | main.rs:106/110/114 |
| cli/mod.rs + commands/mod.rs | module trees | `pub mod` + variant wiring | ✓ WIRED | cli/mod.rs:2,16,17,179,186,193; commands/mod.rs:2,17,18 |
| create.rs/update.rs | validators | `validate_events` / `validate_https_url` / `validate_stdin_body` | ✓ WIRED | imported + called pre-HTTP (create.rs:71–72, update.rs:72–75); counter==0 stub-proven |
| update.rs | get_webhook → PUT merge | get→merge→PUT | ✓ WIRED | update.rs:98 fetch, merged body wire-tested |
| webhooks list.rs | cache | `cache.set(KEY_WEBHOOKS, …)` (id,url) pairs ONLY | ✓ WIRED | list.rs:33–41 — built exclusively from list response id/url; create/update/delete invalidate (3/3) |
| trash list/restore/purge | normalize_trash_type | pre-URL normalization | ✓ WIRED | all three call it before building any URL |
| audit list.rs | list_audit | Options passthrough | ✓ WIRED | list.rs:40; empty-omit + clamp live in the client (api/mod.rs:1281–1291) |
| all three groups | forbidden_hint surfaces | "webhooks"/"general"/"trash"/"audit" | ✓ WIRED | handle_response/handle_delete_response calls verified per method; live 403 probes rendered the exact registered hints |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
| -------- | ------------- | ------ | ------------------ | ------ |
| webhooks list/get | rendered rows | list_webhooks/get_webhook wire response | Yes (live stub: real id/url rendered) | ✓ FLOWING |
| webhooks create output | secret | server 201 create response | Yes (live stub: real 64-char secret rendered exactly once) | ✓ FLOWING |
| trash list | rows | list_trash wire response | Yes (stub tests render fixture rows) | ✓ FLOWING |
| audit list | rows | list_audit wire response | Yes (stub tests: changes.value.from==100 via --json) | ✓ FLOWING |
| "(shown once at creation)" | secret display in list/get | locked display contract (server never re-sends the secret) | N/A — deliberate CONTEXT-locked value, not a missing data source | ✓ (not a stub) |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
| -------- | ------- | -------- | ------ |
| Full test suite (3 runs) | `cargo test` | 493 tests; phase suites green every run; 2/3 runs fully green | ✓ PASS (see flake note below) |
| webhooks stub suite | `cargo test --test webhooks_stub_test` | 29 passed / 0 failed | ✓ PASS |
| trash stub suite | `cargo test --test trash_stub_test` | 24 passed / 0 failed | ✓ PASS |
| audit stub suite | `cargo test --test audit_stub_test` | 13 passed / 0 failed | ✓ PASS |
| help truthfulness | `cargo test --test help_examples_test` | 19 passed / 0 failed | ✓ PASS |
| Unknown event pre-HTTP | `pipelite webhooks create --url https://x/h --events deal.archived` | exit 2, names event + lists 13, no "Connection failed" | ✓ PASS |
| Non-https URL | `pipelite webhooks create --url http://x/h --events deal.created` | exit 2, "https" hint, zero HTTP | ✓ PASS |
| Show-once secret (live stub) | stub 201 + `webhooks create --format table`, piped | exit 0; one 64-char own-line secret behind the warning; occurs once; absent from redirected HOME | ✓ PASS |
| List placeholder (live stub) | stub 200 + `webhooks list --format json` | "(shown once at creation)" present; zero 64-hex in stdout; cache = id/url pairs only | ✓ PASS |
| Purge refusal | `pipelite trash purge` (non-TTY) | exit 2, "permanently" + "--force", zero HTTP; `--no-input` identical | ✓ PASS |
| Purge scope-first | `pipelite trash purge --type bogus` | exit 2 "Unknown trash type", zero HTTP | ✓ PASS |
| Restore scope-first | `pipelite trash restore bogus x1` | exit 2, zero HTTP | ✓ PASS |
| Audit admin hint (live stub 403) | `pipelite audit list` vs 403 stub | exit 1 + "The audit log requires an admin API key." | ✓ PASS |
| Webhook ownership hint (live stub 403) | `pipelite webhooks get wh1` vs 403 stub | exit 1 + "This webhook belongs to another user." | ✓ PASS |
| Help honesty | webhooks/trash/audit --help probes | 13 events in create help; no --description; no --all flag on audit; purge warning at group level; round-trip + cap notes | ✓ PASS |

### Probe Execution

No `scripts/*/tests/probe-*.sh` probes declared by PLANs or found conventionally; behavioral probes were executed directly against the built binary and head-capturing stub servers (table above). All PASS.

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
| ----------- | ----------- | ----------- | ------ | -------- |
| WHOK-01 | 11-01 | List, get, create, update (PUT), delete webhooks | ✓ SATISFIED | Truth 1; all five subcommands wire-proven |
| WHOK-02 | 11-01 | 13-event client-side validation with actionable error | ✓ SATISFIED | Truth 1; exit 2 pre-HTTP on flags AND --stdin arrays (WR-01 hardening: non-string entries also rejected) |
| WHOK-03 | 11-01 | Secret shown exactly once, full, own line, save-it-now; never cached / in list-get | ✓ SATISFIED | Truth 2; live probes at every layer |
| TRSH-01 | 11-02 | List trashed records with --type filter | ✓ SATISFIED | Truth 3; 9-alias normalization, bounded --all |
| TRSH-02 | 11-02 | Restore by type and ID | ✓ SATISFIED | Truth 3; plural-tab URLs, no confirm, 404 re-wrap |
| TRSH-03 | 11-02 | Permanent purge — admin-only, strongest confirm, --force, exit-2 zero-HTTP refusal under --no-input, round-trip normalization | ✓ SATISFIED | Truth 4 |
| AUDT-01 | 11-03 | Audit list filtered by 4 flags | ✓ SATISFIED | Truth 5; verbatim passthrough (snake_case server keys), empty-omit, clamp 1..=100 |
| AUDT-02 | 11-03 | First-class Forbidden hint on admin-gated surfaces | ✓ SATISFIED | Truth 5; all three gated surfaces probed (audit/purge/foreign-webhook) |

**Orphaned requirements: NONE.** REQUIREMENTS.md traceability maps exactly these 8 IDs to Phase 11; plan frontmatter declares exactly these 8 (11-01: WHOK-01..03; 11-02: TRSH-01..03; 11-03: AUDT-01..02). AUDT-03 and WHOK-04 are "Future Requirements" (explicitly out of milestone scope — 11-03's success criteria require NOT building them; not gaps, not milestone-phase-deferred).

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| ---- | ---- | ------- | -------- | ------ |
| src/config.rs (pre-existing, Phase 1 commit 7621978) | 315 | Flaky test `config::tests::env_var_precedence_server_url` under full-suite parallelism (env-var mutation race; passes 6/6 in isolation; failed 1 of 3 full runs) | ⚠️ Warning | NOT Phase 11 code — zero Phase 11 commits touch config.rs. Muddies the "full suite green" signal only; no phase goal impacted |
| tests/audit_stub_test.rs | 21 | Unused import `std::sync::atomic::Ordering` (compile warning) | ℹ️ Info | Cosmetic |
| src/api/models.rs:451, src/cache.rs:30, src/prompt.rs:318 | — | Pre-existing dead_code warnings (WorkflowRunTrigger, TTL_WORKFLOWS, get_workflows_cached) | ℹ️ Info | Pre-date Phase 11; untouched by it |

Debt-marker scan (TBD/FIXME/XXX/PLACEHOLDER/coming-soon) over all 21 phase files: **zero matches**. Production-code unwrap/expect scan over all new files: **zero matches** (all `.expect()` hits are inside `#[cfg(test)] mod tests`). No stubs — every rendered value flows from the wire or a locked display contract.

### Post-Summary Review-Fix Commits (verified consistent)

Three review-driven fixes landed after the SUMMARYs (review status: clean / all_fixed):
- `f3dba8e` CR-01: `--dry-run` now honored on `webhooks update --stdin` (was executing a real PUT)
- `3978c34` WR-01: non-string `events` entries in `--stdin` bodies rejected exit 2 (were silently skipped past the allow-list)
- `4cac5e5` IN-03: update `--json` renders verbatim (no injected fake secret placeholder — PUT responses never carry a secret)

All three strengthen the phase contract; none violate a must-have. Accepted-carrying review notes (IN-01 write-only cache, IN-02 fan-out duplication, IN-04) are quality observations, not goal gaps.

### Human Verification Required

See `human_verification` frontmatter: (1) interactive `trash purge` TTY confirm flow, (2) interactive `webhooks delete` TTY confirm flow, (3) live-server admin-key smoke test including an irreversible `purge --force` on a throwaway record, (4) visual table rendering inspection. Everything else was machine-verified, including live stub-server probes of the secret contract, the 403 hints, and the zero-HTTP refusals.

### Gaps Summary

None. All 5 roadmap success criteria are observably true in the codebase and in behavior. The only non-green signal is the pre-existing, out-of-scope `config::tests::env_var_precedence_server_url` parallelism flake (documented above) — it does not touch Phase 11 code and passes in isolation.

---

_Verified: 2026-09-04T04:11:56Z_
_Verifier: the agent (gsd-verifier)_
