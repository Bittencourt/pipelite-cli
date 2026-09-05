# Phase 12 Validation Map — Custom Fields: Definitions & Typed Writing

**Requirement → test → gate mapping.** Every requirement traces to named stub-server tests plus typed grep gates. Wave-0 gaps (test files) are closed inside the plans — the create-body-shape test (12-01) and the canonical `price=4` wire test + cold-cache dry-run test (12-02) are written RED before the corresponding handlers exist. The decisive server fact shapes everything: the v1 API validates NOTHING on custom-field values, so every assertion below guards a client-side check.

## Requirement → Test → Gate Map

| Req | Behavior under test | Test file (created in) | Pinning tests | Automated gate |
|-----|--------------------|------------------------|---------------|----------------|
| CFLD-01 | Definitions CRUD against stubs: list (entity_type alias-normalized on the wire, pagination), get (f64 + null position), create body shape (--key → wire "name", config.options array, NO position/description keys — raw-string asserts), select→single_select aliasing, stdin verbatim, update partial PUT (only provided keys, immutable keys never sent, --position f64), delete contract (dry-run/TTY/--force/non-TTY exit 1), re-delete 404 | `tests/custom_fields_stub_test.rs` (12-01 T1/T2/T3) | create_body_shape (RED first), create_select_alias_body, create_multi_select_body, update_position_f64, update_partial_body, update_stdin_strips_immutable, delete_non_tty_refusal, delete_then_redelete_404 + normalizer/option-config unit matrices | `cargo test --test custom_fields_stub_test` |
| CFLD-01 | Tombstone honesty: no deleted column possible (serializer omits deleted_at); note wording in list/delete help + delete output | same + `tests/help_examples_test.rs` (12-01 T3) | custom_fields_help_truthful ("does not mark them" present, --description absent) + `grep -A 12 'pub struct CustomFieldDefinition ' src/api/models.rs \| grep -c deleted_at` == 0 | same |
| CFLD-01 | Cache contract for 12-02: per-entity keys warmed by filtered list; ALL three mutations invalidate the custom_fields_ prefix | same (12-01 T3) | delete_invalidates_cache, create_invalidates_cache (two-run stub + cache-dir walk) + `grep -c invalidate_prefix create/update/delete == 3` | same |
| CFLD-02 | Typed writes on the wire: price=4 → JSON number 4 (is_number + as_i64 + raw "price":4, never "4"/4.0); boolean strict; date/text passthrough; multi_select comma-split array; select validated string; per-entity coverage on all 4 entities; activities --mark-undone raw path carries typed values (8th site) | `tests/typed_writing_stub_test.rs` (12-02 T1/T2/T3) | number_int_body (RED first), number_float_body, update_typed_body, boolean_wire, select_valid, multi_select_wire, date_passthrough, activities_mark_undone_raw_path + inference-table unit matrix (i64/f64 split, comma-split, strict boolean, option membership) | `cargo test --test typed_writing_stub_test` |
| CFLD-02 | Client-side validation (server validates nothing): non-numeric number → exit 2 naming field+type; bad boolean → exit 2; unknown select/multi_select option → exit 2 listing valid options (webhooks 13-event precedent); select with no configured options → note, not a block; formula write → REFUSED exit 2; missing '=' → exit 2 BEFORE the definitions fetch (zero HTTP even cold); unknown name → string + ONE aggregated warning (quiet-suppressible) | same | number_bad_warm_zero_post, select_invalid_warm, multi_select_invalid_warm, select_no_options_note, formula_refused, missing_equals, unknown_key_warning + quiet variant | same |
| CFLD-02 | Resolver cache: cold → GET defs (auto-paginated limit 100) + cache write; warm → POST only; two-run round-trip | same (12-02 T3) | cache_round_trip_two_runs (cold 2 requests → warm 1) | same |
| CFLD-03 | --custom-field-json verbatim bypass (nested object/array/null/float deep-equal); non-object → exit 2; mutual exclusivity with --custom-field AND --stdin → exit 2 pre-HTTP; counts as "has flags" in update no-flags gates | same (12-02 T1/T2/T3) | json_verbatim_nested, json_non_object, exclusivity_json_stdin, both_flags_zero_http_warm, update_no_flags_gate_counts_json | same |
| SC-4 | --dry-run NEVER fetches definitions: cold cache → counter == 0, strings + visible note (quiet-suppressible); warm cache → typed values, no note | same (12-02 T1) | dry_run_cold_cache_zero_http (RED first — NO defs response even scripted), dry_run_quiet_suppresses_note, dry_run_warm_cache_types | same |
| — | Exclusivity normalization across the 8 handlers (4 create handlers were Validation exit 1) | `tests/typed_writing_stub_test.rs` + `tests/headless_test.rs` (12-02 T2) | headless stdin_and_flags_mutually_exclusive pinned to .code(2) + grep gates: 12 × `custom_field_json.is_some()`, 4 × InvalidInput-before-"mutually exclusive", `parse_custom_fields` == 0 hits in src/ | `cargo test --test typed_writing_stub_test --test headless_test --test batch_error_test` |

## Cross-Cutting Gates

| Concern | Gate |
|---------|------|
| Full-suite phase gate | `cargo test` green after each plan's final task (12-01: new stub suite + help truthfulness + docs; 12-02: typed-writing suite + updated headless pin + untouched batch_error/dead_flags green) |
| Zero-HTTP rejections (locked codes) | 2 = unknown entity/type, options misuse, missing --key, non-object --config, update-no-flags, stdin-flag mix, bad number/boolean/option, formula write, missing '=', both-flags, json non-object · 1 = definition delete non-TTY refusal (Validation — the standard contract, deliberately distinct from trash purge's exit 2) — each proven with counter == 0 or unreachable-server runs lacking "Connection failed" |
| Amendment pins | `--description` absent from cli/custom_fields.rs, the create model, and the docs section · no `position` key possible on POST (CustomFieldDefinitionCreate has no field; raw-body assert) · `select` alias normalizes to `single_select` on the wire · config.options is a JSON array (never a comma-string) · tombstone note ("does not mark them") present in help + delete output |
| Server-fact pins | Blob keys matched BY NAME (resolver builds a name→definition map) · v1 validates nothing → every type/option check is client-side pre-POST · formula keys server-stripped → CLI refuses exit 2 · list includes tombstones unmarked → no deleted column code exists |
| Conventions | No unwrap/expect in production code · hints on every error naming field + type + expected shape · --dry-run (cache-only!)/--no-input/--quiet/--no-color respected · exactly two advisory stderr lines (dry-run note, unknown-key warning), both quiet-suppressible · one shared resolver module (grep: 8 handler files call it, zero per-entity copies) |

## Sampling Rate

- **Per task commit:** `cargo test --test custom_fields_stub_test` (12-01) / `cargo test --test typed_writing_stub_test` (12-02) + `cargo test --bin pipelite` (unit matrix)
- **Per wave:** `cargo test` full suite (waves execute sequentially: 12-01 → 12-02)
- **Phase gate:** full suite green before `/gsd-verify-work`

## Wave-0 Gap Status

| Test file | Requirement coverage | Created in |
|-----------|---------------------|------------|
| tests/custom_fields_stub_test.rs | CFLD-01 | 12-01 Task 1 (create_body_shape RED-first) |
| tests/typed_writing_stub_test.rs | CFLD-02, CFLD-03, SC-4 | 12-02 Task 1 (number_int_body + dry_run_cold_cache_zero_http RED-first) |

No framework installs needed — assert_cmd + tests/common/mod.rs cover every shape (head+body-capturing stub, unreachable-server pre-HTTP probes, per-child HOME for cache hermeticity, two-runs-one-stub for warm-cache claims). Resolves the RESEARCH Wave-0 gap list in full.

## Broken/Affected Existing Tests (inventory — verified in-repo)

| Test | Current state | What happens |
|------|--------------|--------------|
| tests/headless_test.rs::stdin_and_flags_mutually_exclusive | deals create --stdin --title → exit 1 (Validation) | 12-02 T2 normalizes to exit 2; test UPDATED to pin `.code(2)` |
| src/batch.rs::parse_custom_fields_{rejects_missing_equals,builds_object} | green | 12-02 T2 DELETES them with the function (zero callers remain); missing-equals re-covered by resolver unit + wire tests at exit 2 |
| tests/batch_error_test.rs::batch_update_stdin_with_field_flag_exits_2 | `.code(2)`, green | UNTOUCHED — update handlers were already InvalidInput; must stay green (Phase 7 contract) |
| tests/deals_integration.rs:138,203 · orgs_integration.rs:40 | assert `--custom-field` in help | Keep passing; 12-02 T3 EXTENDS them to also require `--custom-field-json` |
| tests/dead_flags_test.rs (pipelines/stages) | green | UNTOUCHED — different surface |
