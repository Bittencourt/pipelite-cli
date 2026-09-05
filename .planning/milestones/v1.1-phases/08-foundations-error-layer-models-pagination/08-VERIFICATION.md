---
phase: 08-foundations-error-layer-models-pagination
verified: 2026-09-03T18:40:00Z
status: passed
score: 14/14
overrides_applied: 0
---

# Phase 8: Foundations — Error Layer, Models, Pagination — Verification Report

**Phase Goal:** The CLI tells the truth — errors carry the server's real reason with actionable hints, permission tiers are distinguishable, models match server output, and no list truncates silently
**Verified:** 2026-09-03T18:40:00Z
**Status:** passed
**Re-verification:** No — initial verification

## Goal Achievement

Verification was performed goal-backward against the actual codebase, not SUMMARY claims. Evidence includes direct binary probes run by the verifier against hand-rolled stub servers (live-server credentials unavailable; suite is hermetic) plus the full test suite re-run in this session.

### Observable Truths

Merged from ROADMAP success criteria (SC-1..SC-5) and both PLAN frontmatter must-have lists. 14 truths total.

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | 403 on any API surface reports Forbidden with a surface-specific hint, not the init hint (SC-1) | ✓ VERIFIED | Probe: `deals get` vs 403 stub → `error: Forbidden (HTTP 403)` + server detail "You don't have access to this resource" + hint "Your API key doesn't have permission for this action."; no "Check your API key". `forbidden_hint` table at src/api/mod.rs:91-99 with audit/trash/notes/webhooks entries ready for Phases 10-12 |
| 2 | 401 still reports Authentication failed with init hint — tiers distinguishable (SC-1) | ✓ VERIFIED | src/api/mod.rs 401 arms keep `CliError::Auth` + "Check your API key. Run `pipelite init`..."; suite 401 regression test passes (error_layer_stub_test) |
| 3 | 422 renders server's errors[] as `{field}: {message} ({code})`, never bare status or generic detail (SC-2) | ✓ VERIFIED | `parse_rfc7807` (src/api/mod.rs:45-83): errors[]-first, joined "; ", `(invalid)` default code, all-invalid degrades to detail chain; suite tests assert joined text and absence of "Request validation failed" |
| 4 | 409 inactive-trigger maps to actionable activation hint with server detail preserved (SC-2) | ✓ VERIFIED | Probe: `workflows trigger wf_1` vs 409 stub → verbatim "Workflow is not active. Activate the workflow before triggering a run." + hint naming `pipelite workflows update <id> --active true`; exit 1; quote-free |
| 5 | All three 401/403 mapping sites split: handle_response, handle_delete_response, check_auth_status | ✓ VERIFIED | 3 × `CliError::Forbidden` mappings (src/api/mod.rs:273, 327, 394); 0 combined `401 \| 403` arms; 33+7 surface-keyed call sites; 2 `parse_rfc7807(&error_body` calls. Probe: `ping` vs 403 stub → Forbidden + permission hint (third site proven through CLI) |
| 6 | Rendered detail never carries JSON quotes; 6-step fallback errors[] → detail → title → legacy → "HTTP {status}" | ✓ VERIFIED | Code read (lines 45-83): every extraction via `as_str()`; total function with `.unwrap_or_else` fallthrough; 14 unit tests pass |
| 7 | `deals list` succeeds against fractional `position`; whole numbers render `10000.0` (SC-3) | ✓ VERIFIED | `Deal.position: Option<f64>` (models.rs:56), `Stage.position: f64` (models.rs:355). Probe: deal with `"position":10010.5` listed fine → `position: 10010.5`; unit fixtures pin int-token → 10000.0 + serialization nuance |
| 8 | `stages list` without `--pipeline` issues one unfiltered call listing all stages; rows distinguishable (SC-3) | ✓ VERIFIED | Probe: stub serving stages from pl_a + pl_b → request line `GET /api/v1/stages?limit=50&offset=0` (NO pipeline_id param), both rows rendered, exit 0; `stages_table_config` default_columns include pipeline_id |
| 9 | `--expand` payloads appear in output, not silently discarded (SC-4) | ✓ VERIFIED | Probe: `deals list --expand owner` vs stub with sibling `owner` object → `owner` key present in JSON output; 7 × `#[serde(flatten, default, skip_serializing_if = "Map::is_empty")] pub expanded` fields (grep count = 7) |
| 10 | Dead flags gone: `people list --org/--owner`, `workflows create --active`, pipelines/stages `--custom-field` reject exit 2 + hint BEFORE HTTP, hidden from help (SC-5) | ✓ VERIFIED | Probes: all three paths → exit 2, removal detail + replacement hint, no "Connection failed" (zero HTTP against unreachable server). Help probes: 0 occurrences of --org/--owner, --active (create), --custom-field in respective --help. 5 × `hide = true` (people.rs:75,81; workflows.rs:116; pipelines.rs:106; stages.rs:137). Guards are first statements of handlers (people/list.rs:17-25, workflows/create.rs:25-32, pipelines/create.rs:24, stages/create.rs:25) |
| 11 | `workflows create` no longer prompts and never sends `active` (SC-5) | ✓ VERIFIED | 0 `dialoguer` references in src/commands/workflows/create.rs; `WorkflowCreate` has no `active` field (models.rs: exactly one `pub active: bool` = Workflow:408, one `pub active: Option<bool>` = WorkflowUpdate:446); suite stdin-path test proves unknown `active` key dropped |
| 12 | `workflows list --active` filters client-side with exactly one stderr warning, fetching all records (CR-01 fix, SC-5) | ✓ VERIFIED | src/commands/workflows/list.rs:27-30: any `--active` routes through `fetch_all` (auto-paginates all records before filtering); warning at line 28; `apply_active_filter` (112-130) rebuilds PaginationMeta from filtered rows; `WorkflowsListParams` has no `active` field (grep = 0). Suite: mixed-rows filter test, multi-page coverage test, --active-false fetches-all regression test — all pass |
| 13 | `--all` warns loudly at the record ceiling on stderr; exit stays 0 (FIX-06, SC-5) | ✓ VERIFIED | Locked message `warning: --all stopped at 1000 records (server ceiling); results may be incomplete` in all 7 list.rs files (grep = 7); conditional `if total > max_records` guard preserved; old "Showing X of Y" message gone. Suite: total=1243 → warning + exit 0 + 10 requests; total=450 → NO warning + 5 requests (negative case) |
| 14 | CHANGELOG.md carries v1.1 Breaking Changes: removed flags → replacements, --active behavior, f64 nuance, orgs --owner known-dead note | ✓ VERIFIED | CHANGELOG.md read in full: all 5 removed flags with replacements, `--active` client-side semantics with exact warning line, ceiling warning (exit unchanged), f64 `10000.0` rendering nuance, orgs `--owner` known-limitation note, plus Added (RFC 7807/Forbidden/409) and Fixed (stages all-mode, expand rendering) sections |

**Score:** 14/14 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
| -------- | -------- | ------ | ------- |
| `src/error.rs` | CliError::Forbidden variant, display arm, exit 1 | ✓ VERIFIED | Variant at :15-16 with "Forbidden (HTTP 403)" title; display arm :58; unit test asserts exit 1 |
| `src/api/mod.rs` | parse_rfc7807, surface param, hint table, 409 hint, split check_auth_status | ✓ VERIFIED | All present and substantive (lines 45-99, 265-420); 40 call sites threaded |
| `src/api/models.rs` | f64 positions + flatten expanded ×7 + fixtures | ✓ VERIFIED | grep: flatten = 7, Option<f64> = 1, f64 = 1, 10010.5 fixture present |
| `src/commands/workflows/list.rs` | --active fetch_all routing + filter + warning | ✓ VERIFIED | Full file read; CR-01 fix structurally present |
| `src/commands/stages/list.rs` | Optional --pipeline, Option<String> threading | ✓ VERIFIED | Required-flag Validation deleted; StagesListParams.pipeline_id pushed only when Some (api/mod.rs:1046-1075) |
| `src/commands/workflows/create.rs` | No prompt, no active, parse-then-error guard | ✓ VERIFIED | 0 dialoguer refs; guard first statement with activation hint |
| `tests/error_layer_stub_test.rs` | 7 stub tests: 403/401/409/422/404 paths | ✓ VERIFIED | 7 tests, all pass; PIPELITE_SERVER_URL hermetic |
| `tests/dead_flags_test.rs` | 11 tests: 5 removals + help hiding + stub success | ✓ VERIFIED | 11 tests, all pass |
| `CHANGELOG.md` | v1.1 Breaking Changes section | ✓ VERIFIED | Complete coverage of all SC-5 items |
| All 7 `src/commands/*/list.rs` | Locked ceiling warning | ✓ VERIFIED | grep = 7 files, conditional guard intact |

### Key Link Verification

| From | To | Via | Status | Details |
| ---- | --- | --- | ------ | ------- |
| handle_response + handle_delete_response | parse_rfc7807 | both extract detail through helper | ✓ WIRED | 2 call sites (`parse_rfc7807(&error_body`, lines 324, 391) |
| 403 arms + check_auth_status | CliError::Forbidden | forbidden_hint(surface) | ✓ WIRED | 3 mappings; 4 forbidden_hint refs (def + 3 calls) |
| tests/error_layer_stub_test.rs | app server URL override | PIPELITE_SERVER_URL → TcpListener stub | ✓ WIRED | 10 test files use PIPELITE_SERVER_URL; stubs receive real requests (probes confirmed request lines) |
| people/list.rs run() | CliError::InvalidInput | dead-flag guard before HTTP | ✓ WIRED | First statement; probe proved zero HTTP |
| 7 × list.rs fetch_all | ceiling warning | eprintln inside conditional guard | ✓ WIRED | Message present ×7; suite proves conditional firing |
| dead_flags + integration tests | stub servers | capturing TcpListener helpers | ✓ WIRED | Request-count and captured-body assertions pass |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
| -------- | ------------- | ------ | ------------------ | ------ |
| parse_rfc7807 → error detail | error_detail | Server response body text | Yes (probe: server detail verbatim in output) | ✓ FLOWING |
| expanded map | expanded | Server-emitted sibling keys (--expand payloads) | Yes (probe: owner object rendered) | ✓ FLOWING |
| Deal/Stage position | position | Server records (numeric) | Yes (probe: 10010.5, 1.0, 2.0) | ✓ FLOWING |
| fetch_all ceiling warning | total | Server meta.total per page | Yes (suite: 1243-total stub fires warning; 450 does not) | ✓ FLOWING |

### Behavioral Spot-Checks

All run by the verifier in this session against the freshly built binary (not SUMMARY claims).

| Behavior | Command | Result | Status |
| -------- | ------- | ------ | ------ |
| Dead flag rejects pre-HTTP | `people list --org org_x` (unreachable server) | exit 2; removal detail + jq hint; no "Connection failed" | ✓ PASS |
| workflows create --active removed | `workflows create --name W --active true` | exit 2; hint names `workflows update <id> --active true` | ✓ PASS |
| stages create --custom-field removed | `stages create --name S --pipeline pl_1 --custom-field k=v` | exit 2; unaffected-surfaces hint | ✓ PASS |
| Removed flags hidden from help | 3 × `--help` grep | 0 matches for --org/--owner, --active (create), --custom-field | ✓ PASS |
| 403 → Forbidden, entity path | deals get vs 403 stub | "Forbidden (HTTP 403)" + server voice + permission hint, exit 1 | ✓ PASS |
| 403 → Forbidden, ping path (site 3) | ping vs 403 stub | Forbidden + permission hint, NOT bad-key, exit 1 | ✓ PASS |
| 409 → activation hint | workflows trigger vs 409 stub | Verbatim server detail + `pipelite workflows update <id> --active true` hint, quote-free, exit 1 | ✓ PASS |
| stages list all-mode | stages list vs 2-pipeline stub | Request has NO pipeline_id param; rows from pl_a + pl_b; exit 0 | ✓ PASS |
| --expand passthrough + fractional position | deals list --expand owner vs stub | `owner` key in output; `position: 10010.5` | ✓ PASS |
| Full test suite | `cargo test --no-fail-fast` | 306 passed / 0 failed across 25 binaries | ✓ PASS |

### Probe Execution

| Probe | Command | Result | Status |
| ----- | ------- | ------ | ------ |
| error_layer_stub_test | `cargo test --test error_layer_stub_test` | 7 passed; 0 failed | PASS |
| dead_flags_test | `cargo test --test dead_flags_test` | 11 passed; 0 failed | PASS |
| stages/workflows/deals integration | included in full suite re-run | all green | PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
| ----------- | ---------- | ----------- | ------ | -------- |
| FIX-01 | 08-02 | Misleading/ignored filter flags fixed or removed with hints | ✓ SATISFIED | Truths 10-12: 5 flags removed (exit 2 + hints + hidden), --active honest client-side filter + warning, prompt gone |
| FIX-02 | 08-02 | --expand payloads render in output | ✓ SATISFIED | Truth 9: flatten ×7 + probe |
| FIX-03 | 08-02 | Position fields deserialize server floats (Option<f64>) | ✓ SATISFIED | Truth 7: f64 fields + probe + fixtures |
| FIX-04 | 08-02 | stages list allows omitting --pipeline | ✓ SATISFIED | Truth 8: optional param, single unfiltered call, pipeline_id column |
| FIX-05 | 08-01 | RFC 7807 parsing; Forbidden variant with per-surface hints; 409 hint | ✓ SATISFIED | Truths 1-6: all three mapping sites, hint table, probes |
| FIX-06 | 08-02 | --all warns loudly on stderr at record ceiling | ✓ SATISFIED | Truth 13: locked message ×7, conditional guard, exit 0 |

Orphaned requirements: none — REQUIREMENTS.md maps FIX-01..06 to Phase 8, and both plans' frontmatter claim exactly these IDs (08-01: FIX-05; 08-02: FIX-01..04 + FIX-06). No requirement left unclaimed.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| ---- | ---- | ------- | -------- | ------ |
| (none) | — | Zero TBD/FIXME/XXX/TODO/HACK/PLACEHOLDER markers across all phase-modified files | — | — |

Code review (08-REVIEW.md, iteration 2) is clean: 0 Critical, 0 Warning, 5 Info (IN-01 indentation cosmetic; IN-02 pre-existing short-server guard suggestion; IN-03 clippy style; IN-04 ceiling message names --all on --active-only runs — truncation still truthfully disclosed; IN-05 --limit/--offset ignored on --active path, matching the pre-existing --all convention and disclosed by the warning). None of the Info findings contradict any success criterion.

### Deferred Items

None as gaps. Informational notes for downstream phases (already tracked in plan summaries, no action this phase):
- `docs/api-reference.md:381` ("401, 403 → Auth") and the removed-flags/stages-optional documentation are stale — scheduled for the Phase 13 docs pass.
- Pre-existing dead-code warnings (WorkflowRunTrigger, TTL_WORKFLOWS, get_workflows_cached) may re-activate in Phases 10-12 — re-check before deleting (deferred-items.md).
- Pre-existing intermittent config-test flake (~1-in-20) documented in deferred-items.md; the previously-red env-dependent trio was fixed hermetically in commit 3d8233e — the full suite now passes 306/0 deterministically (confirmed in this session's run).

### Human Verification Required

None. Every observable behavior in the phase goal was machine-verified: exit codes, stderr warning text, error titles/details/hints, JSON output shape, and query-string semantics were probed directly through the real binary against stub servers, and the 306-test hermetic suite passes. Live-server conformance was established at research time from server source (verified ProblemDetail shapes); admin-gated surfaces (audit/trash/notes/webhooks) arrive in Phases 10-12 and only need to pass a surface key.

### Gaps Summary

No gaps. All 14 must-have truths verified, all artifacts exist and are substantive and wired, all key links connected, all 6 requirement IDs satisfied with direct evidence, full suite green (306/0), code review clean, no debt markers. The CLI now tells the truth on every surface this phase touched: 403 ≠ 401 with actionable per-surface hints, server error reasons render quote-free, fractional positions and --expand payloads survive to output, dead flags reject loudly before any HTTP, workflows --active filters honestly with a warning, and no list truncates silently at the 1000-record ceiling.

---

_Verified: 2026-09-03T18:40:00Z_
_Verifier: the agent (gsd-verifier)_
