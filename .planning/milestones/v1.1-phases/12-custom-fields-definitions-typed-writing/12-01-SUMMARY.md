---
phase: 12-custom-fields-definitions-typed-writing
plan: 01
subsystem: api
tags: [custom-fields, definitions, crud, clap, serde, wire-tests, cache]

# Dependency graph
requires:
  - phase: 11-webhooks-trash-audit
    provides: webhooks delete confirm contract (copied verbatim), head+body capturing stub helpers, group-wiring patterns
  - phase: 08-foundations-error-layer-models-pagination
    provides: f64 position lesson (numeric(20,10) + parseFloat), InvalidInput exit-2 hint conventions, render_list/render_single
provides:
  - Top-level `custom-fields` group (list/get/create/update/delete) against /api/v1/custom-field-definitions
  - CustomFieldDefinition (serializer-exact, position Option<f64>) + CustomFieldDefinitionCreate models
  - 6 client methods (list with entity_type filter, get, create, create_raw, update PUT Value, delete)
  - KEY_CUSTOM_FIELDS_DEAL/ORG/PEOPLE/ACTIVITY + TTL_CUSTOM_FIELDS=3600 cache interface for 12-02's resolver
  - normalize_entity_type (9 aliases → 4 server tokens) + normalize_definition_type (10 tokens + select→single_select), pre-HTTP exit 2
  - custom_fields_ cache-prefix invalidation on every mutation; filtered lists warm per-entity keys
affects: [12-02 typed-writing resolver, 8 entity create/update handlers, completions]

# Tech tracking
tech-stack:
  added: [] # zero new crates — serde_json/clap/dialoguer/reqwest as in tree
  patterns:
    - "Serializer-exact models with wire-pinned create bodies (raw-string absence asserts)"
    - "Pre-HTTP allow-list normalization (trash-precedent plain match, not ValueEnum)"
    - "Tri-state flags via paired --flag/--no-flag with conflicts_with"
    - "Two-run stub tests proving cache warm → mutation → prefix invalidation"

key-files:
  created:
    - src/cli/custom_fields.rs
    - src/commands/custom_fields/mod.rs
    - src/commands/custom_fields/list.rs
    - src/commands/custom_fields/get.rs
    - src/commands/custom_fields/create.rs
    - src/commands/custom_fields/update.rs
    - src/commands/custom_fields/delete.rs
    - tests/custom_fields_stub_test.rs
  modified:
    - src/api/models.rs
    - src/api/mod.rs
    - src/cache.rs
    - src/cli/mod.rs
    - src/commands/mod.rs
    - src/main.rs
    - tests/help_examples_test.rs
    - docs/api-reference.md

key-decisions:
  - "CustomFieldDefinitionCreate always serializes required/show_in_list — wire body is exactly {name, entity_type, type, required, show_in_list}(+config), matching must_haves over the plan's 4-key unit-test text (see Deviations)"
  - "--key maps to the wire 'name' (blob keys are definition NAMES per server source) — the definition name IS the --custom-field key"
  - "select→single_select aliasing; options REQUIRED for select family, rejected for all other types (dead-value bug class removed)"
  - "All three mutations invalidate the literal custom_fields_ prefix; filtered lists warm per-entity keys only (unfiltered lists would poison them)"
  - "Delete runs the STANDARD Validation exit-1 confirm contract (deliberately not trash purge's exit-2); re-delete 404s via the standard NotFound path"

patterns-established:
  - "Pattern: wire-exact partial PUT bodies built from a serde_json::Map of only-provided keys"
  - "Pattern: stdin immutable-key stripping with ONE quiet-suppressible stderr warning"
  - "Pattern: shared TOMBSTONE_NOTE const rendered in help text and command output"

requirements-completed: [CFLD-01]

# Metrics
duration: 25min
completed: 2026-09-04
---

# Phase 12 Plan 01: Custom Field Definitions CRUD Summary

**Top-level `custom-fields` group (list/get/create/update/delete) with serializer-exact f64-position models, `--key`→wire-`name` create mapping (no position/description keys ever sent), select→single_select aliasing, partial PUT updates with immutable-key guards, the standard confirm-delete contract, and custom_fields_ cache-prefix invalidation on every mutation — proven by 33 head+body-capturing stub tests with the create-body-shape test written RED first.**

## Performance

- **Duration:** 25 min
- **Started:** 2026-09-04T10:10:35Z
- **Completed:** 2026-09-04T10:36:03Z
- **Tasks:** 3 (TDD tasks 1-2 each landed RED+GREEN commits)
- **Files modified:** 16

## Accomplishments
- Definitions CRUD end-to-end: `pipelite custom-fields list/get/create/update/delete` against `/api/v1/custom-field-definitions`, all validation client-side pre-HTTP (the server validates nothing on the v1 path)
- Wire honesty pinned at the stub level: create POST body is exactly `{name, entity_type, type, required, show_in_list}`(+`config.options` array) — raw-string asserts prove no position/description key ever reaches the POST; update PUT carries ONLY provided keys and never the immutable entity_type/type
- 12-02 interface ready: KEY_CUSTOM_FIELDS_DEAL/ORG/PEOPLE/ACTIVITY + TTL_CUSTOM_FIELDS=3600; filtered lists warm per-entity cache files; create/update/delete invalidate the whole custom_fields_ prefix (two-run stub-proven)
- Full suite green with zero regressions: 543 tests passed across 33 binaries (33 new stub tests + 2 help-truthfulness tests + 12 new unit tests)

## Task Commits

Each task was committed atomically (TDD tasks carry RED + GREEN):

1. **Task 1: Create-body-shape test FIRST + model, client methods, cache constants, normalizers** — `f541417` (test, RED) + `895ae63` (feat, GREEN)
2. **Task 2: CLI group wiring + list/get/create handlers with tombstone notes + cache writes** — `d325431` (test, RED) + `e28df24` (feat, GREEN)
3. **Task 3: update (partial PUT, immutable guard) + delete contract + help truthfulness + docs + full-suite gate** — `1275e3c` (feat)

**Plan metadata:** committed separately after SUMMARY (docs commit).

## Files Created/Modified
- `src/api/models.rs` — CustomFieldDefinition (serializer-exact; position Option<f64> from birth; NO deleted_at/description) + CustomFieldDefinitionCreate + custom_fields_table_config + 6 unit tests
- `src/api/mod.rs` — 6 client methods under `// -- Custom field definitions --` (all surface "general"; entity_type query pair only when filtering)
- `src/cache.rs` — KEY_CUSTOM_FIELDS_DEAL/ORG/PEOPLE/ACTIVITY + TTL_CUSTOM_FIELDS=3600
- `src/commands/custom_fields/mod.rs` — normalize_entity_type (9 aliases), normalize_definition_type (11 tokens), build_options_config, validate_stdin_definition, TOMBSTONE_NOTE, dispatch + 6 unit tests
- `src/commands/custom_fields/create.rs` — flag/stdin body resolution, --key→name mapping, options gates, dry-run intercept, cache invalidation
- `src/commands/custom_fields/list.rs` — pre-HTTP entity normalization, per-entity cache warm (filtered only), empty-list hint, meta render
- `src/commands/custom_fields/get.rs` — serializer-exact render, standard 404 path
- `src/commands/custom_fields/update.rs` — partial PUT Map of only-provided keys, tri-state pairs, --config object gate, stdin immutable-strip warning, no-flag refusal
- `src/commands/custom_fields/delete.rs` — webhooks confirm contract verbatim + tombstone reminder + prefix invalidation
- `src/cli/custom_fields.rs` — CustomFieldsCommands {List, Get, Create, Update, Delete} + args (no --description anywhere; update has no immutable flags)
- `src/cli/mod.rs`, `src/main.rs`, `src/commands/mod.rs` — group wiring + dispatch arm
- `tests/custom_fields_stub_test.rs` — 33 wire-level tests (create-body-shape first, RED-verified)
- `tests/help_examples_test.rs` — custom_fields_help_truthful + custom_fields_subcommands_have_examples
- `docs/api-reference.md` — `## \`pipelite custom-fields\`` section (all five subcommands, aliases, mapping, immutability, delete contract, tombstone note)

## Decisions Made
- CustomFieldDefinitionCreate carries required/show_in_list (always serialized) — the plan's unit-test text ("keys exactly {name, entity_type, type, config}") contradicted its own must_haves wire truth and create-body-shape test, which demand `required:false`/`show_in_list:false` on the POST; resolved in favor of the wire contract (see Deviations #2)
- Update renders the PUT response directly in json mode (no follow-up GET) — the stdin contract test pins exactly ONE request
- Stdin-vocabulary validation lives in `commands/custom_fields/mod.rs` (`validate_stdin_definition`) beside the normalizers
- No deleted column, no probe hacks — the shared TOMBSTONE_NOTE is the only honesty surface (serializer omits deleted_at; indistinguishable by design)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Task 1 GREEN required the CLI group to exist**
- **Found during:** Task 1 (GREEN phase)
- **Issue:** The stub tests run the compiled binary, so tests 1-8 cannot pass without `custom-fields` wired into cli/mod.rs + main.rs — but src/cli/custom_fields.rs and the wiring were assigned to Task 2's file list
- **Fix:** Task 1 landed the create-path CLI + dispatch arm; the enum grew incrementally (Create → +List/Get in Task 2 → +Update/Delete in Task 3, matching the plan's own "group after_help stays honest for what exists" note). The plan's "List/Get/Create dispatch arms in Task 1" moved list/get arms to Task 2 where their handlers live
- **Files modified:** src/cli/custom_fields.rs, src/cli/mod.rs, src/main.rs
- **Verification:** Task 1 gate green (8 stub tests + all greps); final full suite green
- **Committed in:** 895ae63

**2. [Rule 1 - Bug] Create-struct key-set contradiction resolved in favor of must_haves**
- **Found during:** Task 1 (model implementation)
- **Issue:** Plan behavior text said CustomFieldDefinitionCreate serializes to "keys exactly {name, entity_type, type, config}" while must_haves truth #2 and stub test 1 require the flag-path body to always carry `required`/`show_in_list` — a 4-field struct makes the pinned wire test impossible
- **Fix:** Struct carries all six schema fields (config skip-if-none; booleans always serialize); unit test asserts the exact six-key set + absence of position/description
- **Files modified:** src/api/models.rs
- **Verification:** create_body_shape stub test passes; unit test pins the key set
- **Committed in:** 895ae63

**3. [Rule 3 - Tooling] Verify-gate greps that could never match**
- **Found during:** Task 1/Task 3 verification
- **Issue:** (a) `grep -c '#[test]'` treats `[test]` as a BRE character class and always returns 0; (b) `grep -c 'fn custom_fields_table_config' == 1` and the normalizer-fn counts also matched test-fn names sharing those prefixes; (c) Task 3's `--description == 0` whole-file docs gate is unsatisfiable — pre-existing notes/webhooks sections legitimately document `--description` flags (server-backed there)
- **Fix:** (a) used `grep -cF` for the intended fixed-string count; (b) renamed the new test fns (`entity_type_normalization_*`, `definition_type_normalization_*`, `options_config_*`, `definition_table_config_*`); (c) verified intent scoped to the new custom-fields docs section (0 occurrences) — the pre-existing matches are out of scope
- **Files modified:** src/commands/custom_fields/mod.rs, src/api/models.rs (test-fn renames only)
- **Verification:** all remaining gate expressions pass verbatim
- **Committed in:** 895ae63 / 1275e3c

---

**Total deviations:** 3 auto-fixed (1 blocking, 1 bug, 1 tooling) + 1 minor sequencing adjustment (validate_stdin_definition moved to mod.rs to keep Task 2's one-line-per-file gate; validation now sits beside the normalizers)
**Impact on plan:** All fixes were necessary to make the plan's own gates satisfiable; no scope creep. Requirements surface unchanged.

## Issues Encountered
- None beyond the deviations above — RED phases failed for exactly the right reason (unrecognized subcommand), and the full suite passed first try after Task 3.

## User Setup Required
None — no external service configuration required (no new crates; stub-server tests are fully hermetic).

## Known Stubs
None — every command path performs real HTTP against `/api/v1/custom-field-definitions` (verified by head-capturing stub tests); no placeholder data anywhere.

## Next Phase Readiness
- Plan 12-02 (typed-writing resolver) can start immediately: KEY_CUSTOM_FIELDS_* + TTL_CUSTOM_FIELDS exist, filtered lists warm `Vec<CustomFieldDefinition>` per entity, and every mutation invalidates the `custom_fields_` prefix (the resolver's stale-type defense is already wire-proven)
- The 8 entity create/update handlers are untouched (12-02's blast radius); `--custom-field` flags still write strings until the resolver lands
- Watch item: `api/mod.rs` grew by ~140 lines (now ~1,830) — approaching the ~2,000-line split threshold noted in STATE.md

---
*Phase: 12-custom-fields-definitions-typed-writing*
*Completed: 2026-09-04*

## Self-Check: PASSED

- All 8 created files exist on disk (verified)
- All 5 task commits present in history: f541417, 895ae63, d325431, e28df24, 1275e3c
- Full suite at execution end: 543 passed / 0 failed across 33 test binaries
