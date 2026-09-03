# Phase 8 Validation — Error Layer, Models & Pagination

Materialized from 08-RESEARCH.md § Validation Architecture (workflow.nyquist_validation: true).
Source of truth: 08-RESEARCH.md; this file is the executable requirement → test gate map.

## Test Framework

| Property | Value |
|----------|-------|
| Framework | cargo test (Rust 1.94, built-in) + assert_cmd integration tests |
| Config file | none needed (Cargo conventions) |
| Quick run command | `cargo test --quiet` |
| Full suite command | `cargo test` |

## Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | Test File | Gate | Plan/Task | File Exists? |
|--------|----------|-----------|-------------------|-----------|------|-----------|--------------|
| FIX-05 | 422 RFC 7807 body with errors[] renders server text (`{field}: {message} ({code})`) | stub-server integration | `cargo test --test error_layer_stub_test` | tests/error_layer_stub_test.rs | stderr contains joined errors[] text, NOT "Request validation failed"; exit 1 | 08-01 Task 3 | ❌ Wave 0 (new file) |
| FIX-05 | 403 → Forbidden variant + surface hint (deals surface default) | stub-server integration | `cargo test --test error_layer_stub_test` | tests/error_layer_stub_test.rs | stderr contains "Forbidden" + "permission", NOT "Check your API key"; server detail preserved; exit 1 | 08-01 Task 3 | ❌ Wave 0 (new file) |
| FIX-05 | 409 on workflows trigger → activation hint | stub-server integration | `cargo test --test error_layer_stub_test` | tests/error_layer_stub_test.rs | stderr contains server detail AND "pipelite workflows update" AND "--active true"; no JSON quotes; exit 1 | 08-01 Task 3 | ❌ Wave 0 (new file) |
| FIX-05 | parse_rfc7807 unit: errors[]-first, detail fallback, no JSON quotes, legacy keys | unit | `cargo test --bin pipelite` | src/api/mod.rs `#[cfg(test)]` | all fallback branches proven; degenerate `{}` and non-JSON bodies → `"HTTP {status}"` | 08-01 Task 1 | ❌ Wave 0 (new mod in api/mod.rs) |
| FIX-05 | ping-path 403 → Forbidden (check_auth_status, third mapping site) | unit/integration | `cargo test --test error_layer_stub_test` | tests/error_layer_stub_test.rs | `pipelite ping` against 403 stub reports Forbidden, NOT bad-key | 08-01 Task 2 + 3 | ❌ Wave 0 |
| FIX-03 | fractional position deserializes; whole f64 serializes as `N.0` | unit (models.rs tests) | `cargo test --bin pipelite models` | src/api/models.rs `#[cfg(test)]` | `"position": 10010.5` → `Some(10010.5)`; to_string emits `10000.0`; stage `2` → `2.0` | 08-02 Task 1 | ❌ Wave 0 (extend existing mod) |
| FIX-02 | expand payload survives round-trip; absent → no `expanded` key | unit (models.rs tests) | `cargo test --bin pipelite models` | src/api/models.rs `#[cfg(test)]` | sibling `owner` object survives in `expanded`; empty case → no `expanded` key in output | 08-02 Task 1 | ❌ Wave 0 |
| FIX-04 | stages list without --pipeline issues no pipeline_id param; rows show pipeline_id | stub-server integration | `cargo test --test stages_integration` | tests/stages_integration.rs | exit 0; rows from multiple pipeline_id values; stub receives NO pipeline_id pair; filtered case still pushes pair when Some | 08-02 Task 2 | ✅ (extend) |
| FIX-01 | removed flags → InvalidInput, exit 2, hint text, zero HTTP (5 paths) | integration | `cargo test --test dead_flags_test` | tests/dead_flags_test.rs | exit 2 + hint fragments; stderr does NOT contain "Connection failed" (unreachable 127.0.0.1:1 server); --help does NOT advertise removed flags | 08-02 Task 3 | ❌ Wave 0 (new file) |
| FIX-01 | people list --help hides --org/--owner (existing test flipped) | integration | `cargo test --test people_integration` | tests/people_integration.rs | `people_list_help_shows_filter_flags`: --org/--owner NOT advertised; --limit/--offset still advertised | 08-02 Task 3 | ✅ (extend — flip assertions) |
| FIX-01 | workflows list --active filters client-side + exactly one warning | stub-server integration | `cargo test --test workflows_integration` | tests/workflows_integration.rs | only active rows rendered; stderr contains the locked warning line exactly once; 1 request on page path; no warning without --active | 08-02 Task 2 | ✅ (extend) |
| FIX-06 | --all at 1000 ceiling prints locked warning, exit 0 | stub-server integration | `cargo test --test deals_integration` | tests/deals_integration.rs | total=1243: stderr contains `warning: --all stopped at 1000 records (server ceiling); results may be incomplete`; exit 0; exactly 10 requests | 08-02 Task 2 | ✅ (extend) |
| FIX-06 | --all UNDER the ceiling: no warning (guard stays conditional) | stub-server integration | `cargo test --test deals_integration` | tests/deals_integration.rs | total=450: exit 0; stderr does NOT contain "server ceiling" | 08-02 Task 2 | ✅ (extend — negative case) |

## Sampling Rate

- **Per task commit:** `cargo test --quiet` (full suite is fast enough at this codebase size)
- **Per wave merge:** `cargo test` + `cargo build` warning check
- **Phase gate:** full suite green before `/gsd-verify-work`

## Wave 0 Gaps

- [ ] `tests/error_layer_stub_test.rs` — RFC 7807 fixtures (422 errors[], 403, 409, 401), extends the `spawn_stub_server` pattern (body-carrying script tuples) — 08-01 Task 3
- [ ] `tests/dead_flags_test.rs` — parse-then-error integration for the 5 removed flag paths (127.0.0.1:1 unreachable pattern) — 08-02 Task 3
- [ ] models.rs test additions (fractional position, flatten round-trip) — inside existing `#[cfg(test)] mod tests` — 08-02 Task 1
- [ ] `src/api/mod.rs` `#[cfg(test)]` unit mod for parse_rfc7807 + Forbidden exit code — 08-01 Task 1
- [ ] Flip `people_list_help_shows_filter_flags` assertions in tests/people_integration.rs — 08-02 Task 3
- [ ] No framework install needed

## Gate Notes (checker-revised)

- `Workflow.active` (models.rs:378) and `WorkflowUpdate.active` (models.rs:411) MUST survive — only `WorkflowCreate.active` (models.rs:396) is removed; gates pin exactly one `pub active: bool` + one `pub active: Option<bool>` in models.rs.
- The FIX-06 ceiling warning sits INSIDE the kept `if total > max_records` guard in all 7 list.rs files; the negative under-ceiling test is part of the FIX-06 gate.
- `hide = true` gate counts total ≥5 across the 4 CLI files (people.rs legitimately carries 2: --org and --owner).
- 08-01 Task 3's 422 create test uses `deals create --title X --stage stg_1` and is TTY-hermetic (`--no-input` or `.write_stdin("")`) — `--stage` is client-side required and required-field satisfaction reaches optional-field prompts on a TTY.
