# Phase 13 Validation Map — Integration Hardening & Docs Refresh

**Requirement → test → gate mapping.** Phase 13 has no exclusive requirement IDs (ROADMAP: "None exclusively — cross-cutting verification of every requirement delivered in Phases 7-12"); both plans carry `requirements: []` (verified passing `frontmatter.validate --schema plan`). The map below traces each ROADMAP Phase 13 success criterion to the tests and typed grep gates that prove it. The decisive planning fact: the contract matrix encodes the LOCKED contracts (STATE.md Decisions), so any red row is a candidate violation to fix-and-pin — never an expectation to relax.

## Requirement → Test → Gate Map

| Criterion | Behavior under test | Test file (created in) | Pinning tests | Automated gate |
|-----|--------------------|------------------------|---------------|----------------|
| SC-1 (--dry-run) | Every Phase 7-12 MUTATION surfaces a zero-HTTP preview: batch deals update/delete --stdin, templates create/delete, notes add/edit/delete, webhooks create/update, trash restore, custom-fields create/update/delete → exit 0, no "Connection failed" (unreachable server), method+URL preview on stdout. Typed-writing row: cold cache + --dry-run → zero HTTP + fallback note (quiet-suppressible). Read-only rows documented exempt (side-effect-free; v1.0 precedent) | `tests/contract_matrix_test.rs` (13-01 T1) | dry_run_zero_http (per-mutation rows), dry_run_typed_writing_cold_cache | `cargo test --test contract_matrix_test` |
| SC-1 (--no-input) | Locked refusals fire pre-HTTP with flag-naming hints: batch delete stdin → exit 1 "Re-run with --force"; webhooks/templates/notes/custom-fields delete → exit 1 "--force"; trash purge → exit 2 (locked stricter outlier); notes add missing body → exit 2 (prompt suppressed under piped stdin) | same (13-01 T1) | no_input_refusals rows (per Destructive case) | same |
| SC-1 (--quiet) | Data survives, chatter suppressed: empty-hints + success confirmations absent under --quiet; batch "N ok, M failed" summary PRESENT (Phase 7 SC-3); webhook show-once secret PRESENT under --quiet (it is DATA) | same (13-01 T2) | quiet_suppression rows + batch anchor + webhook-secret row | same |
| SC-1 (--no-color) | Zero "\x1b[" bytes in stdout AND stderr on success (webhooks/trash table renders, batch run) and error paths (404 get) | same (13-01 T2) | no_color_zero_ansi rows | same |
| SC-1 (formats) | csv header row (contains "id", ≥2 lines) + plain non-empty output for all 7 list-bearing new groups (runs, templates, notes, webhooks, trash, audit, custom-fields); batch/docs exempt with table comments | same (13-01 T2) | csv_and_plain rows (7×2) | same |
| SC-2 (stub anchors) | Batch exit-code contract + show-once secret proven at matrix level; the LIVE half is 13-02's human-run E2E (batch exit codes --quiet, secret shown once, trash round-trip, typed write NUMBER server-side — the Phase 12 deferred item) | `tests/contract_matrix_test.rs` (13-01) + `scripts/e2e-v1.1.sh` / `docs/e2e-v1.1-report.md` (13-02 T3/T4) | quiet batch anchor, webhook-secret-under-quiet; E2E scenario functions + filled report | 13-01: matrix green · 13-02: `bash -n scripts/e2e-v1.1.sh` + grep gates + human-run report (checkpoint) |
| SC-3 (docs) | SKILL.md: 9 new surfaces + Exit Code Contract (0/1/2 + hint conventions) + Batch/Cache/Typed-Writing patterns; stale facts corrected (line counts, cache keys, endpoint table). api-reference.md: complete sections for all 9 groups (batch, runs, templates, docs ADDED; 5 existing audited); every documented flag verified against real --help. README enumerates new groups (it has a Commands section — 13-CONTEXT condition met) | docs edits (13-02 T1/T2) | — (docs; pins are greps + the --help checklist in the summary) | grep gates per plan (≥14 SKILL `##` sections, ≥15 api-reference `pipelite` sections, zero "unreleased" in CHANGELOG, README enumerations) |
| SC-4 (no regressions) | Full `cargo test` green; explicit v1.0-surface smoke rows in the matrix (deals list/get json, orgs create --dry-run zero-HTTP, ping, dashboard, completions); any newly-failing pre-existing test proven pre-existing via throwaway-worktree check before classification | `tests/contract_matrix_test.rs` (13-01 T3) | v1.0 smoke tests | `cargo test --test contract_matrix_test` + full `cargo test` |
| Loose ends | api/mod.rs = 1805 < ~2000 → keep single file (decision + measurement in 13-02 summary); deferred-items (3 files) annotated RESOLVED-with-evidence or CARRIED per item | summary + deferred-items edits (13-02 T3) | — | grep gates: RESOLVED/CARRIED present in all three files |

## Cross-Cutting Gates

| Concern | Gate |
|---------|------|
| Full-suite phase gate | `cargo test` green after 13-01 T3 (matrix + fixes) — the 13-02 docs/E2E tasks add no production code |
| Zero-HTTP proof pattern | Unreachable-server rows assert stderr LACKS "Connection failed"; stub rows assert counter == 0 (established batch_error_test precedent) |
| Locked exit codes | 1 = standard delete refusals + batch partial failure; 2 = trash purge non-TTY refusal + structural/missing-input — the matrix pins these verbatim; an honest fix that would change a locked code escalates to a deviation-log entry for orchestrator decision, never a silent relaxation |
| Fix-and-pin discipline | Violations fixed in the surface's src (conventions: hints, no unwrap, ctx.quiet/ctx.color gating), pinned in the matrix AND the surface's own test file, deviation-logged in 13-01-SUMMARY.md ("No violations found" is also an entry) |
| Credential hygiene | `grep -ciE 'fake-test-key|pk_[a-z0-9]{8}|whsec_[a-z0-9]{8}'` over scripts/e2e-v1.1.sh + docs/e2e-v1.1-report.md == 0; creds env-var-only with unset-guard; webhook secret recorded as length + shown-once, redacted |
| Live-data safety | E2E touches only `e2e-`-prefixed throwaway records; `trash purge` never invoked (grep gate `pipelite .+trash purge` == 0); trap-based cleanup; no purge of the round-trip record |

## Sampling Rate

- **Per task commit (13-01):** `cargo test --test contract_matrix_test` (axis under construction) + targeted greps
- **Per wave:** full `cargo test` at 13-01 T3 (production-code changes possible there; 13-02 is docs/scripts only)
- **Phase gate:** matrix green + full suite green + docs greps + E2E report filled (human checkpoint) before the milestone audit

## Wave-0 Gap Status

| Test file | Criterion coverage | Created in |
|-----------|---------------------|------------|
| tests/contract_matrix_test.rs | SC-1 (all four flags + formats), SC-2 stub anchors, SC-4 smoke | 13-01 Task 1 (scaffold + behavioral axes), Task 2 (presentation axes), Task 3 (smoke + fixes) |
| scripts/e2e-v1.1.sh + docs/e2e-v1.1-report.md | SC-2 live half + Phase 12 deferred typed-write item | 13-02 Task 3 (authored), Task 4 (human-run checkpoint) |

No framework installs needed — assert_cmd + tests/common/mod.rs (cmd / cmd_with_server / spawn_head_capturing_stub_server) cover every matrix shape: unreachable-server zero-HTTP probes, head+body-capturing stubs, per-child HOME for cache hermeticity. jq is system-provided for the E2E's type assertion.

## Broken/Affected Existing Tests (inventory — verified in-repo)

| Test | Current state | What happens |
|------|--------------|--------------|
| All existing suites (589+ tests incl. 13 per-surface stub files, batch suites, help_examples, dead_flags, headless, quiet_mode) | green | 13-01 matrix is ADDITIVE — no existing test asserts on the matrix surface. If a fix-and-pin change corrects pinned-but-wrong behavior, the affected surface tests are UPDATED in the same commit and ENUMERATED in the 13-01 deviation log (violations are unknown until execution — the protocol, not a list, is the binding plan content) |
| tests/cli_skeleton + deals_integration env-dependent trio (deferred-items 07/08) | status re-verified in 13-02 T3 | Annotated RESOLVED/CARRIED with evidence — never silently absorbed into the 13-01 suite gate |
| docs/help truthfulness tests (help_examples_test.rs) | green | 13-02 docs edits must not contradict them; the --help validation pass uses the same source of truth |

## Coverage Audit (source → plan)

| Source item | Plan |
|-------------|------|
| Table-driven contract-matrix test file (13-CONTEXT) | 13-01 T1/T2 |
| Violations FIXED in-phase + pinning tests + deviation log (13-CONTEXT) | 13-01 T3 + summary requirement |
| Format coverage csv/plain per new group (13-CONTEXT) | 13-01 T2 |
| v1.0 baseline: full suite + explicit smoke (13-CONTEXT, ROADMAP SC-4) | 13-01 T3 |
| SKILL.md sections per new surface + exit-code contract + hint conventions (13-CONTEXT) | 13-02 T1 |
| api-reference.md gap audit vs --help (13-CONTEXT) | 13-02 T2 |
| README only-if-enumerates (13-CONTEXT) → it enumerates (verified) | 13-02 T2 |
| CHANGELOG v1.1 finalize: features + breaking + data-correctness, released-ready (13-CONTEXT) | 13-02 T2 |
| Live E2E: batch exit codes --quiet, show-once secret, trash round-trip, typed write server-side; non-admin Forbidden noted (13-CONTEXT, Phase 12 deferral) | 13-02 T3/T4 |
| 2 plans (13-CONTEXT) | 13-01 + 13-02 |
| api/mod.rs size decision (13-CONTEXT, STATE watch item) — 1805 < 2000 → keep single file | 13-02 T3 |
| deferred-items.md audit: mark resolved, carry open (13-CONTEXT) | 13-02 T3 |
| Deferred (excluded per 13-CONTEXT): comfy-table 7→8 upgrade, any new feature work | — (correctly absent) |
