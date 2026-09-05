---
phase: 12-custom-fields-definitions-typed-writing
plan: 02
subsystem: cli-handlers
tags: [custom-fields, typed-writing, resolver, wire-tests, exit-codes, cache]

# Dependency graph
requires:
  - phase: 12-custom-fields-definitions-typed-writing
    plan: 01
    provides: KEY_CUSTOM_FIELDS_DEAL/ORG/PEOPLE/ACTIVITY + TTL_CUSTOM_FIELDS cache interface, CustomFieldDefinition model, list_custom_field_definitions client method, custom_fields_ prefix invalidation
  - phase: 08-foundations-error-layer-models-pagination
    provides: CliError::InvalidInput (exit 2) + hint conventions, Option<f64> position lesson
  - phase: 07-batch-operations
    provides: batch::parse_custom_fields (deleted here), BATCH-04 structural-exit-2 precedent, batch_error_test contract (kept green)
provides:
  - src/custom_fields.rs: pub async fn resolve_custom_fields — ONE shared entry point for all 8 create/update handlers (json bypass funnel, dry-run cache-only branch, cache-through auto-paginated definitions fetch, name-keyed inference table)
  - CfEntityType enum (Deal/Organization/Person/Activity → api_token + cache_key)
  - --custom-field-json verbatim bypass on all 8 arg structs; mutual exclusivity with --custom-field and --stdin at exit 2 pre-HTTP
  - Client-side validation surface: number i64-first, strict boolean, option-validated select/multi_select (exit 2 listing valid options), formula refusal, aggregated unknown-key warning
  - Exit-code normalization: all 12 has_flags sites + 4 create-handler exclusivity errors → InvalidInput exit 2
affects: [completions, CFLD-04 (--custom-field-string, future), docs]

# Tech tracking
tech-stack:
  added: [] # zero new crates — serde_json/clap/anyhow as in tree
  patterns:
    - "One shared async resolver with the dry-run branch INSIDE (8 identical handler call shapes)"
    - "Wire-pinned typed writing: raw-string body asserts (\"price\":4 present, \"price\":\"4\"/4.0 absent) + as_i64 — Value comparison alone can mask 4.0"
    - "Two-run stub pattern: warm via `custom-fields list --entity-type` (exactly 1 request), then assert warm-path deltas"
    - "Zero-HTTP validation pins: structural/semantic refusals assert counter unchanged on a warm cache"
    - "Compiler-enforced honesty: resolve_cache_only has no client call path — dry-run cannot fetch by construction"

key-files:
  created:
    - src/custom_fields.rs
    - tests/typed_writing_stub_test.rs
  modified:
    - src/commands/deals/create.rs
    - src/commands/deals/update.rs
    - src/commands/orgs/create.rs
    - src/commands/orgs/update.rs
    - src/commands/people/create.rs
    - src/commands/people/update.rs
    - src/commands/activities/create.rs
    - src/commands/activities/update.rs
    - src/cli/deals.rs
    - src/cli/orgs.rs
    - src/cli/people.rs
    - src/cli/activities.rs
    - src/batch.rs
    - src/main.rs
    - tests/headless_test.rs
    - tests/deals_integration.rs
    - tests/orgs_integration.rs
    - docs/api-reference.md
    - CHANGELOG.md

key-decisions:
  - "Single resolver entry point: resolve_custom_fields(ctx, entity, pairs, dry_run, json_passthrough) — the json funnel, dry-run branch, and inference live inside so all 8 handlers share one identical call shape"
  - "Number inference i64-first (parse::<i64> before f64) so price=4 emits exactly 4 — wire-pinned by raw-string assertions, not just parsed Value comparison"
  - "Strict select/multi_select option validation at exit 2 listing valid options (webhooks 13-event precedent); missing/malformed config.options is NOT hard-blocked — sent with a quiet-suppressible note"
  - "Formula writes REFUSED exit 2 (server strips formula keys — a silent no-op would be a lie); --custom-field-json remains the documented unvalidated bypass"
  - "Unknown definition names sent as raw strings with ONE aggregated quiet-suppressible warning; dry-run cold-cache fallback sends strings with its own note"
  - "Structural k=v split of ALL pairs precedes the definitions fetch — missing '=' is zero-HTTP even cold (BATCH-04 spirit); old batch parser's Validation shape replaced by InvalidInput exit 2"
  - "activities update resolves ONCE at the top; the typed value flows through both the normal PUT and --mark-undone's raw payload (Pitfall 6 pinned: inserted exactly once)"

patterns-established:
  - "Pattern: shared resolver module consumed by N handlers with per-entity enum parameterization (CfEntityType)"
  - "Pattern: exit-code normalization sweeps with grep-count gates (12 has_flags sites, 4 create-handler variants)"
  - "Pattern: warm-cache delta assertions (counter before/after) proving pre-HTTP refusal"

requirements-completed: [CFLD-02, CFLD-03]

# Metrics
duration: 32min
completed: 2026-09-04
---

# Phase 12 Plan 02: Typed Custom-Field Writing Resolver Summary

**Shared definitions-aware resolver (src/custom_fields.rs) rewiring all 8 create/update handlers so `--custom-field price=4` stores JSON number 4 — raw-body wire-pinned — with a `--custom-field-json` verbatim bypass at exit-2 mutual exclusivity, honest cache-only dry-run, client-side option/boolean/formula validation (the server validates nothing), and `batch::parse_custom_fields` deleted — proven by 26 wire tests + 21 resolver unit tests, full suite 585 green.**

## Performance

- **Duration:** 32 min
- **Started:** 2026-09-04T10:50:25Z
- **Completed:** 2026-09-04T11:22:06Z
- **Tasks:** 3 (TDD tasks 1-2 each landed RED+GREEN commits)
- **Files modified:** 19 (2 created, 17 modified)

## Accomplishments
- The milestone's data-correctness bug is dead: `--custom-field price=4` on a number definition stores JSON number 4 — the canonical test asserts the RAW captured body contains `"price":4`, never `"price":"4"` nor `4.0`, plus `as_i64() == Some(4)`, with the definitions GET + POST both counted
- One shared resolver (`resolve_custom_fields`) consumed by exactly 8 handler files — no per-entity copies; `--custom-field-json` verbatim bypass (nested objects/arrays/null/floats untouched) mutually exclusive with `--custom-field` AND `--stdin` at exit 2 pre-HTTP across all 8 handlers (12 has_flags sites, 4 create-handler errors normalized Validation→InvalidInput)
- `--dry-run` honesty: cold cache → ZERO HTTP (compiler-enforced: no client call path), strings + visible quiet-suppressible warm-the-cache note; warm cache → typed values (cache-only ≠ strings-only)
- Client-side validation (the only kind in existence): strict boolean, option-validated select/multi_select with valid-options hints, formula refusal, aggregated unknown-key warning — every refusal exit 2 with zero HTTP on a warm cache
- `batch::parse_custom_fields` + its 2 unit tests deleted (zero grep hits); batch stdin typed paths untouched (batch_error_test green)
- Full suite: 585 passed / 0 failed across 34 binaries (was 543/33 at 12-01)

## Task Commits

Each task was committed atomically (TDD tasks 1-2 carry RED + GREEN):

1. **Task 1: Canonical price=4 wire test RED + resolver + deals create/update reference rewiring** — `a52791d` (test, RED) + `4f0336d` (feat, GREEN)
2. **Task 2: Fan-out — remaining 6 handlers, 8 arg structs, 12 has_flags sites, exclusivity normalization, parse_custom_fields removal** — `0a61a00` (test, RED) + `5ab48e7` (feat, GREEN)
3. **Task 3: Full inference wire matrix + docs + CHANGELOG + full-suite gate** — `ecbfac5` (feat)

**Plan metadata:** committed separately after SUMMARY (docs commit).

## Files Created/Modified
- `src/custom_fields.rs` (NEW) — CfEntityType (api_token + KEY_CUSTOM_FIELDS_* cache_key), fetch_definitions_cached (cache-first, auto-pagination limit 100 → partial page, TTL_CUSTOM_FIELDS write), resolve_cache_only (no client path), split_pairs (missing '=' → InvalidInput exit 2), build_typed_map (aggregated unknown warning + no-options note), infer_typed_value (pure inference table), validate_option — 21 unit tests
- `src/commands/{deals,orgs,people,activities}/{create,update}.rs` (8 files) — resolver call replacing batch::parse_custom_fields; has_flags gains `custom_field_json.is_some()` (12 sites); 4 create handlers' stdin-exclusivity error Validation→InvalidInput; activities update resolves ONCE before the --mark-undone branch
- `src/cli/{deals,orgs,people,activities}.rs` — `--custom-field-json` (value_name JSON) on all 8 Create/Update args structs; deals after_help gains typed-write examples
- `src/batch.rs` — parse_custom_fields + 2 unit tests deleted (zero callers remain)
- `src/main.rs` — `mod custom_fields;`
- `tests/typed_writing_stub_test.rs` (NEW) — 26 wire tests: canonical price=4, float, dry-run cold/quiet/warm, warm zero-POST validation, update path, orgs/people/activities typed, mark-undone raw path (8th site), json+stdin exclusivity, both-flags zero-HTTP, no-flags-gate-counts-json, boolean/select/multi_select matrices, no-options note, date/text passthrough, unknown-key warning + quiet, formula refusal, missing-= cold zero-HTTP, json verbatim nested + non-object, cache round trip
- `tests/headless_test.rs` — stdin_and_flags_mutually_exclusive pins `.code(2)` explicitly
- `tests/deals_integration.rs`, `tests/orgs_integration.rs` — help asserts extended with `--custom-field-json` (3 sites)
- `docs/api-reference.md` — 4 flag tables: typed `--custom-field` rows + `--custom-field-json` rows; "Typed custom-field writing" note (inference rules, client-side validation, dry-run cache behavior, formula refusal)
- `CHANGELOG.md` — v1.1 Changed/Fixed entry: type-correct values, client-side validation, `--custom-field-json`, exit-2 exclusivity consistency, dry-run cache fallback

## Decisions Made
- Resolver owns the json funnel FIRST (before any fetch/lookup), so both-flags and json-only paths are zero-HTTP by construction — pinned by both_flags_zero_http_warm and exclusivity_json_stdin
- Warm-cache delta assertions (counter before/after) instead of absolute counter==0 for two-run tests — proves the refusal added zero requests while keeping the warm-up request counted
- Doc-comment references to the deleted batch parser removed so `grep -rn parse_custom_fields src/` returns literally zero hits (the plan's own gate)
- select-family definitions with missing/malformed config.options emit a quiet-suppressible "no options configured" note rather than blocking (CONTEXT amendment 1 edge — never hard-block on server-side config sloppiness)
- No #[allow(dead_code)] on CfEntityType variants: the transient Task-1 warning is resolved by Task-2's fan-out minutes later; the repo already tolerates pre-existing warnings

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Tooling] Verify-gate greps that could never match verbatim**
- **Found during:** Task 1/Task 2/Task 3 verification
- **Issue:** (a) `grep -c 'pattern' file1 file2` emits filename-prefixed per-file counts, breaking the gates' `test "$(…)" -eq N` string comparison (Task 1's resolve/parse counts, Task 3's server-computed count); (b) `grep -c '#[test]'` treats `[test]` as a BRE character class (same as 12-01 deviation #3a)
- **Fix:** (a) summed per-file counts with `awk -F: '{s+=$NF} END {print s}'`; (b) used `grep -cF` for the fixed-string test count. All gate expressions otherwise pass verbatim
- **Files modified:** none (verification invocation only)
- **Verification:** all gates green
- **Committed in:** n/a

**2. [Rule 1 - Bug] Task 2 fixture helper wrapped a single definition object as `data`**
- **Found during:** Task 2 (GREEN)
- **Issue:** `definitions_list(definition(…))` serialized `"data": {…}` (an object) where the client deserializes `Vec<CustomFieldDefinition>` — the resolver's definitions GET failed to parse, so the 4 fan-out tests kept failing after wiring
- **Fix:** helper takes `Vec<serde_json::Value>`; call sites wrap in `vec![…]`
- **Files modified:** tests/typed_writing_stub_test.rs
- **Verification:** 14/14 fan-out tests green
- **Committed in:** 5ab48e7

**3. [Rule 1 - Bug] Unit tests asserted on anyhow Display, which shows only the variant title**
- **Found during:** Task 1 (GREEN)
- **Issue:** `format!("{}", anyhow_err)` renders "Invalid input" — the detail/hint fields (the actionable text the behavior block asserts on: "number", "true or false", valid options, "key=value") are struct fields, not the Display chain
- **Fix:** tests downcast to CliError and assert on `detail | hint`; also reworded the boolean error so the literal "expects true or false" appears in detail (the plan's pinned hint text)
- **Files modified:** src/custom_fields.rs (error message + test helper only)
- **Verification:** 21/21 resolver unit tests green; 196→194 bin tests green after batch-parser test deletion
- **Committed in:** 4f0336d

---

**Total deviations:** 3 auto-fixed (2 bugs, 1 tooling) — all inside this plan's own files, no scope creep.
**Impact on plan:** Requirements surface unchanged; all fixes were required to satisfy the plan's own gates.

## Issues Encountered
- RED phases failed for exactly the right reasons: Task 1's 3 tests failed on the string-typed body/missing note; Task 2's 4 fan-out tests failed on the string-typed orgs/people/activities handlers while the 6 already-green deals-side pins held
- The full suite passed first try after Task 3 (585/0); no flaky stub behavior observed

## User Setup Required
None — no new crates, no config changes; stub-server tests are fully hermetic (HOME-redirected, scripted TCP).

## Known Stubs
None — the resolver performs real HTTP against `/api/v1/custom-field-definitions` on the live path (wire-proven); `--custom-field-json` verbatim passthrough is real serde deserialization, not a mock.

## Threat Mitigations Applied
- T-12-06 (type confusion): resolver inference table + raw-body wire pins (price=4 canonical test)
- T-12-07 (formula spoofing): hard refusal exit 2 with server-stripping hint
- T-12-08 (silent merge/stringification): both-flags exit 2 pre-HTTP (counter pinned); unknown keys announced by one aggregated warning
- T-12-09 (dishonest dry-run): cache-only branch with no client call path; cold→strings+note, warm→typed, quiet suppression pinned
- T-12-10 (partial pagination): auto-pagination to partial page at limit 100 (prompt.rs pattern)

## Next Phase Readiness
- Phase 12 complete: definitions CRUD (12-01) + typed writing (12-02) close CFLD-01..03; the `custom_fields` blob is now type-correct end-to-end (CLI → server → filters/workflows)
- Watch item carried forward: `api/mod.rs` ~1,845 lines — approaching the ~2,000-line split threshold noted in STATE.md
- Future: CFLD-04 (`--custom-field-string` force-string) intentionally deferred; `--custom-field-json` covers the escape hatch today

---
*Phase: 12-custom-fields-definitions-typed-writing*
*Completed: 2026-09-04*

## Self-Check: PASSED

- All key files exist on disk (11/11 verified, including both new files)
- All 5 task commits present in history: a52791d, 4f0336d, 0a61a00, 5ab48e7, ecbfac5
- Full suite at execution end: 585 passed / 0 failed across 34 test binaries
- Structure gates: 8 handler files on one resolver, parse_custom_fields zero grep hits, 12 has_flags sites, 4 create-handler exclusivity errors InvalidInput
