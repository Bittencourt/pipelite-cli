---
phase: 12-custom-fields-definitions-typed-writing
verified: 2026-09-04T12:28:11Z
status: passed
score: 9/9
overrides_applied: 0
re_verification:
  previous_status: null
  previous_score: null
  gaps_closed: []
  gaps_remaining: []
  regressions: []
deferred:
  - truth: "End-to-end behavior against a live Pipelite server (server validates nothing on v1; formula-key stripping; tombstones in list output)"
    addressed_in: "Phase 13"
    evidence: "ROADMAP Phase 13 (Integration Hardening & Docs): 'carries no exclusive requirements — it cross-verifies all 30 above end-to-end'"
---

# Phase 12: Custom Fields — Definitions & Typed Writing Verification Report

**Phase Goal:** Users can define custom fields per entity and write correctly-typed values to them (fixes the "stores `\"4\"` not `4`" data-correctness bug)
**Verified:** 2026-09-04T12:28:11Z
**Status:** passed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

Merged from ROADMAP success criteria (contract) + both PLAN frontmatters.

| # | Truth | Status | Evidence |
| --- | ----- | ------ | -------- |
| 1 | SC-1/CFLD-01: User can list/get/create/update/delete custom field definitions, with `custom-fields list --entity-type` filtering (9 aliases normalized pre-HTTP) | ✓ VERIFIED | Independent probe P6: `custom-fields create --entity-type deals --key x --type text` → POST body `{name:"x", entity_type:"deal", ...}` (alias normalized, `--key`→wire `name`, no position/description key); `delete --force` → head `DELETE /api/v1/custom-field-definitions/cfx`; re-delete → exit 1 (standard NotFound); non-TTY delete without `--force` → exit 1 with `--force` hint. Probes P1/P3: defs GET carries `entity_type=deal` on the wire. 33 tests in `tests/custom_fields_stub_test.rs` green (suite 589/0). |
| 2 | SC-2/CFLD-02: `--custom-field price=4` on a number definition stores JSON number `4`, not string `"4"` — type-correct writes for number/boolean/date/array/select, resolved from cached definitions | ✓ VERIFIED | **Canonical probe P1 (run in verifier's own process):** cold cache, `deals create --title T --stage s1 --custom-field price=4` → 2 requests (defs GET + POST); captured raw body contains `"price":4`, does NOT contain `"price":"4"`, does NOT contain `4.0`; parsed `custom_fields.price === 4`. Probe P5: boolean `done=yes` → exit 2 ("true or false"); select `status=bogus` → exit 2 listing `new/won/lost`; multi_select `tags=a,c` → wire `["a","c"]`; all warm-cache refusals added zero HTTP. Probe P3: warm cache dry-run renders `"price": 4` typed. |
| 3 | SC-3/CFLD-03: User can bypass type inference entirely with `--custom-field-json '{"key": ...}'` | ✓ VERIFIED | Probe P4: `--custom-field-json '{"score":{"nested":[1,2,null]},"ratio":1.5}'` → captured body `custom_fields` deep-equals input (nested object/array/null/float untouched, key-order aside); `--custom-field a=1 --custom-field-json '{}'` → exit 2 with zero added HTTP. Suite: `json_verbatim_nested`, `json_non_object`, `exclusivity_json_stdin` green. |
| 4 | SC-4: `--dry-run` never triggers a definitions fetch — cache-only fallback with a visible note | ✓ VERIFIED | Probe P2: `--dry-run` with a COLD cache against a stub with NO defs response scripted → exit 0, request counter == 0, stderr contains "not cached"; `--quiet` suppresses the note. Probe P3: warm cache + `--dry-run` adds 0 requests AND emits typed `"price": 4`. Code: `resolve_cache_only` (src/custom_fields.rs:172-186) has no client call path — zero HTTP by construction. |
| 5 | 12-01: create sends EXACTLY {name, entity_type, type, required, show_in_list} (+config.options); `--key` is the wire NAME; no position/description key | ✓ VERIFIED | Probe P6 body asserts: `name === "x"`, no `position` key, no `description` key, `entity_type === "deal"`. `CustomFieldDefinitionCreate` (src/api/models.rs) carries no position field; `grep -ci description src/cli/custom_fields.rs` → 0. Suite `create_body_shape` (written RED first per plan) green. |
| 6 | 12-01: update sends ONLY provided keys; entity_type/type immutable (never sent, stripped from stdin with warning) | ✓ VERIFIED | Suite: `update_partial_body` (body == exactly `{"name":"newname"}`), `update_position_f64` (body position 10000.5, no entity_type/"type"), `update_stdin_strips_immutable`, `update_no_flags` (exit 2 "nothing to update") — all green in 33-test file. `grep -n 'pub position: Option<f64>' src/cli/custom_fields.rs:129` → present; no immutable flags on update args. |
| 7 | 12-01: delete runs the standard confirm contract; create/update/delete ALL invalidate the `custom_fields_` cache prefix | ✓ VERIFIED | Probe P6: non-TTY delete → exit 1 Validation with `--force` hint, counter 0; `--force` → exit 0. Probe P7 (wire-proven): `custom-fields list --entity-type deals` warms `custom_fields_deal` cache file under tmp HOME; then `custom-fields create` → file GONE. `grep -c invalidate_prefix` → 1 in each of create.rs/update.rs/delete.rs. |
| 8 | 12-02: CLI is the only validation layer — number/boolean/select/multi_select/formula refusals exit 2 (zero HTTP warm) | ✓ VERIFIED | Probe P5 (all exit 2, zero added HTTP, actionable stderr): `done=yes`, `status=bogus` (lists valid options), `calc_total=99` (formula refused, mentions server stripping); `tags=a,c` → `["a","c"]` wire. 21 resolver unit tests green in `src/custom_fields.rs` (i64-first number, strict boolean, option membership incl. malformed-options never-hard-block edge WR-01). |
| 9 | 12-02: All 8 create/update handlers consume ONE shared resolver; `batch::parse_custom_fields` deleted with zero callers; all exclusivity sites normalized to exit 2 | ✓ VERIFIED | `grep -l resolve_custom_fields` across the 8 handler files → exactly 8; `grep -rc 'custom_field_json.is_some()'` across 8 handlers → 12; `custom_field_json` on 8 arg structs across cli/{deals,orgs,people,activities}.rs → 8; `grep -rn parse_custom_fields src/ tests/` → 0 hits; 4 create handlers' exclusivity errors are `InvalidInput` (grep-verified); `tests/headless_test.rs:146` pins `.code(2)`; `activities_mark_undone_raw_path` (typed_writing_stub_test.rs:482) pins the 8th call site. |

**Score:** 9/9 truths verified

### Deferred Items

| # | Item | Addressed In | Evidence |
|---|------|-------------|----------|
| 1 | Live-server end-to-end confirmation of the pinned server contract (server validates nothing on v1, formula keys stripped, tombstones listed unmarked) | Phase 13 | ROADMAP Phase 13: "cross-verifies all 30 requirements end-to-end". All Phase-12 SCs are CLI-side behaviors, machine-verified against the wire; RESEARCH pinned the server contract from server source with file:line citations. |

### Required Artifacts

| Artifact | Expected | Status | Details |
| -------- | -------- | ------ | ------- |
| `src/api/models.rs` | CustomFieldDefinition (serializer-exact, position Option<f64>, no deleted_at/description) + Create struct + table config | ✓ VERIFIED | Substantive; 6 client methods consumable; no deleted_at/description fields (grep-pinned) |
| `src/api/mod.rs` | 6 definition client methods, surface "general" | ✓ VERIFIED | grep count == 6 for the method set; defs GET carries entity_type only when filtering (probe P1) |
| `src/cache.rs` | KEY_CUSTOM_FIELDS_DEAL/ORG/PEOPLE/ACTIVITY + TTL_CUSTOM_FIELDS=3600 | ✓ VERIFIED | Lines 46-51, exact names; consumed by resolver (`cache_key()`) and list.rs |
| `src/cli/custom_fields.rs` | CustomFieldsCommands {List,Get,Create,Update,Delete}; no --description; update position Option<f64> | ✓ VERIFIED | All 5 variants; 0 "description" hits; `position: Option<f64>` at :129 |
| `src/commands/custom_fields/` (mod/list/get/create/update/delete) | handlers + normalizers + tombstone note + cache warm/invalidate | ✓ VERIFIED | normalize_entity_type/normalize_definition_type present with unit tests; CR-01 complete-page cache gate at list.rs:42-54; invalidate_prefix in all 3 mutations |
| `src/custom_fields.rs` | resolver: CfEntityType, resolve_custom_fields, cache-first fetch w/ auto-pagination, cache-only dry-run, inference table, unit tests | ✓ VERIFIED | 597 lines; `pub async fn resolve_custom_fields` at :75; limit-100 pagination loop; 21 unit tests; 0 unwrap/expect in production section |
| `src/batch.rs` | parse_custom_fields REMOVED, zero callers; batch stdin paths untouched | ✓ VERIFIED | `grep -rn parse_custom_fields src/ tests/` → 0 hits; `read_stdin_json`/batch paths intact; batch_error_test green. NOTE: the plan frontmatter `contains: "pub fn parse_batches"` pattern was planner-metadata inaccuracy (that function never existed in this file — pre-existing API is run_batch_update/run_batch_delete/read_stdin_json); the must-have's actual intent (the deletion) is positively verified, so the SDK artifact flag is a false negative, not a gap |
| `src/cli/{deals,orgs,people,activities}.rs` | `--custom-field-json` on all 8 Create/Update arg structs | ✓ VERIFIED | grep count == 8 across the 4 files; help asserts extended (3 sites in deals/orgs integration tests) |
| `src/commands/{deals,orgs,people,activities}/{create,update}.rs` | resolver call replacing batch parser; 12 has_flags sites; 4 exclusivity errors → InvalidInput | ✓ VERIFIED | grep: 8 files, 12 sites, 4 InvalidInput; probe-verified end-to-end via deals (P1-P5) and suite via orgs/people/activities |
| `tests/custom_fields_stub_test.rs` | ≥26 CFLD-01 wire tests, create-body-shape RED first | ✓ VERIFIED | 33 tests, all green |
| `tests/typed_writing_stub_test.rs` | ≥20 CFLD-02/03 wire tests, canonical price=4 first | ✓ VERIFIED | 28 tests, all green; `number_int_body` is first (file header + fn order) |

### Key Link Verification

| From | To | Via | Status | Details |
| ---- | --- | --- | ------ | ------- |
| 8 create/update handlers | src/custom_fields.rs resolve_custom_fields | one shared resolver call | ✓ WIRED | grep -l == 8; probe P1/P5 wire-proven through deals; suite covers orgs/people/activities + mark-undone raw path |
| src/custom_fields.rs | cache.rs KEY_CUSTOM_FIELDS_* + list_custom_field_definitions | cache-first fetch (limit 100 → partial page), TTL write | ✓ WIRED | imports at :22-25; probe P1 shows defs GET with entity_type=deal; P7 shows cache file written then invalidated |
| src/custom_fields.rs infer_typed_value | CliError::InvalidInput (exit 2) | non-numeric/non-boolean/unknown-option/formula/missing-= | ✓ WIRED | probe P5: all refusals exit 2 pre-POST with actionable detail+hint |
| 12 has_flags sites | custom_field_json.is_some() | exclusivity threading | ✓ WIRED | grep count == 12; probe P4: both-flags exit 2 with zero HTTP |
| custom_fields/{create,update,delete}.rs | cache.invalidate_prefix("custom_fields_") | post-mutation invalidation | ✓ WIRED | grep 1 each; probe P7: file-level proof (seed → create → gone) |
| src/commands/deals+orgs+people+activities cli | --custom-field-json flag | arg structs | ✓ WIRED | 8/8 structs; 3 help-assert sites in integration tests |
| activities/update.rs | update_with_null_completed raw payload | typed value resolved once, carried through | ✓ WIRED | resolver call before the --mark-undone branch (:81); `activities_mark_undone_raw_path` test green (typed Bool + "completed_at":null in same raw body) |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
| -------- | ------------- | ------ | ------------------ | ------ |
| resolve_custom_fields | definitions Vec | cache.get(KEY_CUSTOM_FIELDS_*) → on miss real GET `/api/v1/custom-field-definitions?entity_type=…` auto-paginated | Yes — probe P1 (live GET consumed, typed output) + P7 (cache file materialized on disk) | ✓ FLOWING |
| custom-fields list | response.data | real GET with limit/offset/entity_type query | Yes — probe P3 warm-up consumed the GET; suite `list_table` asserts rendered rows (price/number/10000.5) | ✓ FLOWING |
| typed custom_fields in POST/PUT bodies | build_typed_map output | name-keyed definition map → infer_typed_value | Yes — probe P1 raw body `"price":4`; P5 `["a","c"]`; P4 verbatim JSON | ✓ FLOWING |

No disconnected props, no static fallbacks. The only "empty" path (cold-cache dry-run) is a deliberate, user-visible contract (SC-4), not a stub.

### Behavioral Spot-Checks

Independent probe run (verifier's own process, Node stub server + release-path debug binary): **40/40 PASS** (`/tmp/opencode/probe-phase12.mjs`).

| Behavior | Command (probe) | Result | Status |
| -------- | ------- | ------ | ------ |
| Canonical price=4 wire body | deals create --custom-field price=4 (cold) | raw body `"price":4`, no `"price":"4"`, no `4.0`, as_i64==4, counter==2 | ✓ PASS |
| Dry-run cold = zero HTTP + note | deals create --dry-run (no defs scripted) | counter==0, stderr "not cached", --quiet suppresses | ✓ PASS |
| Dry-run warm = typed, no fetch | warm via list, then dry-run | +0 requests, stdout `"price": 4` | ✓ PASS |
| --custom-field-json verbatim | nested object w/ null+float | deep-equal passthrough; both-flags exit 2, 0 HTTP | ✓ PASS |
| Client-side validation matrix | done=yes / status=bogus / calc_total=99 / tags=a,c | exit 2 ×3 with hints; array wire `["a","c"]`; 0 HTTP warm | ✓ PASS |
| Definitions CRUD contract | custom-fields create/delete (+--force, re-delete, non-TTY) | --key→name, no position/description, DELETE head, exit-1 refusal | ✓ PASS |
| Cache warm/invalidate loop | list → cache file; definition create → gone | file-level proof | ✓ PASS |
| Full test suite | cargo test | 589 passed / 0 failed across 34 binaries | ✓ PASS |

### Probe Execution

| Probe | Command | Result | Status |
| ----- | ------- | ------ | ------ |
| Verifier-built wire probe (canonical price=4 + 6 more scenarios, 40 asserts) | `node /tmp/opencode/probe-phase12.mjs` | exit 0 — "PROBE SUMMARY: 40/40 passed" | PASS |
| Project stub suites (the phase's own probes) | `cargo test --test custom_fields_stub_test --test typed_writing_stub_test` | 33 + 28 tests, 0 failed | PASS |
| Regression pins | `cargo test --test headless_test --test batch_error_test` | green (exit-2 pin; Phase 7 contract untouched) | PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
| ----------- | ---------- | ----------- | ------ | -------- |
| CFLD-01 | 12-01 | User can list, get, create, update, and delete custom field definitions (`custom-fields` group, `--entity-type` filter on list) | ✓ SATISFIED | Probe P6 (full CRUD + contracts) + P1/P3 (entity_type on wire); 33 stub tests; REQUIREMENTS.md line 44 |
| CFLD-02 | 12-02 | `--custom-field key=value` writes type-correct JSON (number/boolean/date/array/select) resolved from cached definitions instead of always storing strings | ✓ SATISFIED | Canonical probe P1 (`"price":4` raw) + P3/P5 matrix; 28 stub tests + 21 resolver unit tests; REQUIREMENTS.md line 45 |
| CFLD-03 | 12-02 | User can bypass type inference with `--custom-field-json '{"key": ...}'` | ✓ SATISFIED | Probe P4 (verbatim + exit-2 exclusivity); suite json_verbatim_nested/json_non_object; REQUIREMENTS.md line 46 |

**Orphaned requirements:** none — REQUIREMENTS.md maps exactly CFLD-01/02/03 to Phase 12; 12-01 claims CFLD-01, 12-02 claims CFLD-02+CFLD-03. CFLD-04 (`--custom-field-string`) is explicitly a future-milestone item (REQUIREMENTS.md:79, CONTEXT deferred section) and appears in no Phase-12 plan, as intended.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| ---- | ---- | ------- | -------- | ------ |
| (none) | — | No TBD/FIXME/XXX/TODO/HACK/PLACEHOLDER markers in any phase file; no empty implementations; no placeholder returns; 0 unwrap()/expect() in production code (test-module usage only) | — | — |

REVIEW.md (final, status: clean) carries 4 Info-level follow-ups (IN-03 help-string duplication, IN-04 pagination cap 1000 silent, IN-05 `--options ,` empty-select, IN-07 pre-existing unwraps) — all non-blocking, none touched by the fix scope, none are completion debt markers.

### Human Verification Required

None. All four success criteria are CLI-side behaviors fully machine-verified at the wire level. The only live-only concern — real-server end-to-end behavior — is explicitly assigned to Phase 13 ("cross-verifies all 30 requirements end-to-end") and recorded under Deferred Items.

### Gaps Summary

No gaps. The phase goal is achieved and observable in the codebase:

1. **The data-correctness bug is dead** — independently re-proven in the verifier's own process: `deals create --custom-field price=4` against a number definition puts JSON number `4` on the wire (raw body `"price":4`; the string and float forms are asserted absent).
2. **Definitions CRUD is complete and honest** — `--key` maps to the wire `name`, no position/description keys, alias normalization pre-HTTP, standard delete contract, prefix invalidation wire-proven at the file level.
3. **The bypass and the guardrails both exist** — verbatim `--custom-field-json` with exit-2 mutual exclusivity; client-side validation (the server validates nothing) covering number/boolean/select/multi_select/formula, all zero-HTTP refusals on a warm cache.
4. **Dry-run is honest by construction** — cache-only branch with no client call path; cold cache → strings + visible quiet-suppressible note and zero requests; warm cache → typed values.
5. **Blast radius contained as planned** — exactly 8 handler files on one shared resolver, 12 exclusivity sites normalized to exit 2, `parse_custom_fields` deleted with zero remnants, batch contract untouched.

Full suite 589/0 across 34 binaries (matches REVIEW.md's final count, which also verified the CR-01 partial-page cache gate and WR-01 malformed-options fix). One plan-frontmatter metadata inaccuracy (`src/batch.rs` `contains: "pub fn parse_batches"` — a function that never existed) was resolved by verifying the must-have's actual deletion intent; it is not an implementation gap.

---

_Verified: 2026-09-04T12:28:11Z_
_Verifier: the agent (gsd-verifier)_
