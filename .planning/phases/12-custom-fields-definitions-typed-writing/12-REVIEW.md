---
phase: 12-custom-fields-definitions-typed-writing
reviewed: 2026-09-04T00:00:00Z
depth: standard
files_reviewed: 24
files_reviewed_list:
  - src/custom_fields.rs
  - src/cli/custom_fields.rs
  - src/cli/mod.rs
  - src/cli/deals.rs
  - src/cli/orgs.rs
  - src/cli/people.rs
  - src/cli/activities.rs
  - src/commands/custom_fields/mod.rs
  - src/commands/custom_fields/create.rs
  - src/commands/custom_fields/get.rs
  - src/commands/custom_fields/list.rs
  - src/commands/custom_fields/update.rs
  - src/commands/custom_fields/delete.rs
  - src/commands/deals/create.rs
  - src/commands/deals/update.rs
  - src/commands/orgs/create.rs
  - src/commands/orgs/update.rs
  - src/commands/people/create.rs
  - src/commands/people/update.rs
  - src/commands/activities/create.rs
  - src/commands/activities/update.rs
  - src/api/mod.rs
  - src/api/models.rs
  - src/cache.rs
  - src/batch.rs
  - src/main.rs
  - src/error.rs
  - tests/typed_writing_stub_test.rs
  - tests/custom_fields_stub_test.rs
  - tests/headless_test.rs
  - tests/deals_integration.rs
  - tests/orgs_integration.rs
  - docs/api-reference.md
  - CHANGELOG.md
findings:
  critical: 1
  warning: 1
  info: 7
  total: 9
status: issues_found
---

# Phase 12: Code Review Report

**Reviewed:** 2026-09-04
**Depth:** standard
**Files Reviewed:** 24 source files (+8 supporting tests/docs)
**Status:** issues_found

## Summary

Phase 12's locked contracts are overwhelmingly met and provable in-repo: the canonical `price=4` bug is dead (raw-body wire assertions in `tests/typed_writing_stub_test.rs:21-74`), one shared resolver serves exactly 8 handler files (grep-proven), `batch::parse_custom_fields` has zero remaining hits in `src/`, all 12 `has_flags` sites carry `custom_field_json.is_some()`, the 4 create handlers' stdin-exclusivity errors are now `InvalidInput` (exit 2), POST bodies carry no `position`, PUT carries `Option<f64>` position, `--description` exists nowhere, all three definition mutations invalidate `custom_fields_`, and `--dry-run` is zero-HTTP by construction (`resolve_cache_only` has no client call path). 585 tests pass; full suite green.

However, the review found **one critical defect in the cache contract between `custom-fields list` (writer) and the resolver (reader)**: the list command caches a single possibly-partial page under the per-entity key that the resolver treats as the COMPLETE definition set. This resurrects the exact type-confusion failure this phase exists to kill, triggered by ordinary read-only usage. One warning covers a malformed-options edge that contradicts the module's own documented never-hard-block stance.

Full test suite: 585 passed, 0 failed. Clippy: 5 new warnings introduced by phase-12 code (Info).

## Critical Issues

### CR-01: `custom-fields list` caches a partial page as the COMPLETE definition set — typed writing silently degrades to strings

**File:** `src/commands/custom_fields/list.rs:35-45` (writer) + `src/custom_fields.rs:136-140` (reader)

**Issue:** `custom-fields list --entity-type <t>` writes `response.data` — whatever single page was fetched with the user's `--limit`/`--offset` — under `KEY_CUSTOM_FIELDS_*` with TTL 3600, with **no completeness guard**. The resolver's `fetch_definitions_cached` reads that key and treats it as the full definition universe (its own fetch path paginates to a partial page precisely so it CAN treat the cache as complete). Consequences within the 1-hour TTL:

- Entity has 60 deal definitions, user runs the default `custom-fields list --entity-type deals` (limit 50) → fields 51-60 are unknown to the resolver → `deals create --custom-field field51=4` sends the STRING `"4"` (the exact pre-phase-12 bug) with a misleading "not a defined field(s)" warning — for a field the user just saw in the list output.
- `custom-fields list --entity-type deals --offset 50` caches ONLY the second page → fields 1-50 lose typing and option validation.
- `--limit 10` caches 10 definitions; even the docs' own suggested `--limit 100 --format json` poisons the key for entities with >100 definitions.

The warning path (`--quiet`-suppressible) is the only signal, and the wire value is still type-confused. This defeats SC-2/CFLD-02 under realistic, documented usage — and it is caused by a *read-only* command.

**Fix:** Only cache when the fetched page is a COMPLETE first page, e.g. in `list.rs:35-45`:

```rust
if let Some(ref cache) = ctx.cache {
    if let Some(token) = normalized {
        let complete = args.offset == 0
            && response.data.len() as u64 == response.meta.total;
        if complete {
            let key = match token { /* unchanged */ };
            let _ = cache.set(key, &response.data, TTL_CUSTOM_FIELDS);
        }
    }
}
```

(Alternatively, persist a completeness marker alongside the entry, but the offset/total check is sufficient and touches one file.) Add a stub test: `custom-fields list --entity-type deals --limit 1` on a multi-definition fixture, then `deals create --custom-field <second-page-field>=4` must still GET definitions (counter proves no poisoned hit) and store a JSON number.

## Warnings

### WR-01: Non-string `config.options` array hard-blocks ALL values with an empty "Valid options:" hint — contradicts the documented never-hard-block stance

**File:** `src/custom_fields.rs:365-368` (`options_of`) and `src/custom_fields.rs:374-388` (`validate_option`)

**Issue:** `options_of` filters with `filter_map(|o| o.as_str())`, so a definition whose `config.options` is an array of NON-strings (e.g. `[{"value":"a"}]` — the server never validates config shape, exactly the scenario the module documents) yields `Some(vec![])` instead of `None`. `validate_option` then rejects **every** value with detail "Value 'x' is not a valid option…" and hint `Valid options: ` (empty list). This contradicts the function's own doc contract (lines 370-373: "A definition whose options are missing or **malformed** is NOT hard-blocked — the caller notes it and sends") and the locked CONTEXT amendment ("membership check impossible → send the string with a one-line note"). It also skips the `no_options_note` because `is_none()` is false (`build_typed_map:244-248`), so the user gets zero explanation.

**Fix:** Make `options_of` return `None` when the raw array is non-empty but contains no strings:

```rust
fn options_of(config: Option<&Value>) -> Option<Vec<&str>> {
    let arr = config?.get("options")?.as_array()?;
    if arr.is_empty() {
        return Some(Vec::new()); // genuinely empty: strict rejection is correct
    }
    let strings: Option<Vec<&str>> = arr.iter().map(|o| o.as_str()).collect();
    strings // None when any element is a non-string → malformed → note-and-send
}
```

## Info

### IN-01: Unused import in list.rs

**File:** `src/commands/custom_fields/list.rs:3`
**Issue:** `CustomFieldDefinition` is imported but never named (the type is inferred at `cache.set`). Clippy-confirmed `unused_imports`.
**Fix:** Drop it from the `use` statement.

### IN-02: Clippy style lints in new phase-12 code

**File:** `src/commands/custom_fields/list.rs:35-44, 56`; `src/custom_fields.rs:136-140, 378`
**Issue:** Collapsible `if let` nests (two sites), redundant closure `|d| serde_json::to_value(d)` (list.rs:56), `iter().any()` where `contains()` applies (custom_fields.rs:378).
**Fix:** Apply `cargo clippy --fix` suggestions for these four sites.

### IN-03: TOMBSTONE_NOTE "single source" claim vs four hardcoded help copies

**File:** `src/commands/custom_fields/mod.rs:13-19`; hardcoded at `src/cli/custom_fields.rs:9,15,33` and `src/cli/mod.rs` (group after_help)
**Issue:** The const's doc comment claims "List help, delete help, the group help, and the delete confirmation all render this exact sentence," but only `delete.rs:80` uses the const — the three help strings are separate inline copies. Wording drift between help output and the delete-time reminder is one careless edit away.
**Fix:** Either build the clap `after_help` strings with `concat!`/`const` interpolation (e.g. `const LIST_AFTER_HELP: &str = concat!("Examples:...\n", TOMBSTONE_NOTE);`) or soften the doc comment to acknowledge the duplication.

### IN-04: Definitions pagination silently caps at 1000

**File:** `src/custom_fields.rs:156-158`
**Issue:** The `offset >= 1000` guard stops fetching without any signal; definitions beyond 1000 silently fall into the unknown-key string path. Acceptable as a runaway guard, but silent.
**Fix:** Emit one quiet-suppressible stderr note when breaking early (mirrors the cache-miss note pattern).

### IN-05: `--options ,` creates a select definition that rejects every value

**File:** `src/commands/custom_fields/create.rs:72-81` + `src/commands/custom_fields/mod.rs:87-94`
**Issue:** `--type single_select --options ,` passes the require-check (`options = [""]`, non-empty) and stores `config: {"options": []}` after empty-segment filtering. Every future `--custom-field <key>=<anything>` on it exits 2 with hint "Valid options: " (empty).
**Fix:** After filtering, if select-family and the filtered list is empty, reject at create time: `"--options resolved to an empty list"` + usage hint.

### IN-06: batch.rs lost its trailing newline

**File:** `src/batch.rs` (EOF)
**Issue:** The parse_custom_fields removal left `}` as the last byte with no trailing newline (`\ No newline at end of file` in the diff) — noisy for future diffs and some tooling.
**Fix:** Append a newline.

### IN-07: Pre-existing `unwrap()` calls remain in heavily-edited handler files

**File:** `src/commands/deals/create.rs:78-79`, `src/commands/orgs/create.rs:63`, `src/commands/people/create.rs:74-75`, `src/commands/activities/create.rs:74-75`
**Issue:** CLAUDE.md forbids `unwrap()` in production code. These lines are provably safe (gated by `prompt::check_missing` erroring first) and were NOT touched by this phase (verified against the diff), so they are out of scope for blocking — noted because the phase heavily edited the surrounding functions and the convention is absolute.
**Fix:** Follow-up cleanup: replace with `ok_or_else(|| CliError::Validation { .. })?` matching the idiom used for `id` extraction in the update handlers.

## Contract Verification (locked items)

| Contract | Verdict | Evidence |
|---|---|---|
| `--key` → wire `name` | PASS | `commands/custom_fields/create.rs:93-97`; test `create_body_shape` |
| `select` → `single_select` normalization | PASS | `commands/custom_fields/mod.rs:68` + alias test |
| POST body has NO position / NO description | PASS | `api/models.rs:874-884` (struct + doc); test `..._without_position_or_description` |
| PUT carries `Option<f64>` position | PASS | `cli/custom_fields.rs:129`, `commands/custom_fields/update.rs:98-99`, test `update_position_f64` |
| Delete confirm: dry-run → TTY → `--force`; non-TTY exit 1 Validation | PASS | `commands/custom_fields/delete.rs:31-68` (Validation variant, exit 1 per `error.rs:87-99`) |
| KEY_CUSTOM_FIELDS_* invalidation on all definition mutations | PASS | `invalidate_prefix("custom_fields_")` at create.rs:125, update.rs:122, delete.rs:75 |
| Tombstone note in help + delete output | PASS | `TOMBSTONE_NOTE` (delete.rs:80) + inline copies in group/list/delete after_help (see IN-03) |
| No `--description` anywhere | PASS | grep clean across cli/commands/docs |
| `price=4` → JSON number 4 (wire-level) | PASS | `typed_writing_stub_test.rs:21-74` — raw body `"price":4`, `as_i64()==4`, string/float forms asserted dead |
| Number non-numeric / boolean non-strict / unknown option / formula / missing `=` → exit 2 pre-HTTP | PASS | resolver inference table + warm-cache `counter==0` pins (boolean_wire, select tests, formula_refused, missing_equals_cold_zero_http) |
| Unknown definition name → raw string + quiet-suppressible aggregated warning | PASS | `custom_fields.rs:252-264`; test `unknown_key_warning` incl. `--quiet` |
| `--dry-run` NEVER fetches (cache-only, zero HTTP by construction) | PASS | `resolve_cache_only` takes no client path; `dry_run_cold_cache_zero_http` scripts NO defs response, pins counter==0 |
| `--custom-field-json` verbatim + mutually exclusive (exit 2, pre-HTTP) | PASS | `custom_fields.rs:84-105`; tests `json_verbatim_nested`, `json_non_object`, `exclusivity_json_stdin`, `both_flags_zero_http_warm` |
| ONE shared resolver across exactly 8 handlers | PASS | grep: `resolve_custom_fields` in exactly 8 command files; 8 arg structs carry `custom_field_json` |
| `parse_custom_fields` fully removed | PASS | zero grep hits in `src/`; batch diff removes fn + 2 tests only |
| `--mark-undone` raw path carries typed values exactly once | PASS | resolve at `activities/update.rs:81` before the branch; single insert at :194-196; test pins `"custom_fields"` occurs exactly once alongside `"completed_at":null` |
| batch.rs Phase-7 contracts untouched | PASS | diff touches only the deleted parser + its tests; batch-path Validation errors intact |
| Exclusivity normalization (12 sites, creates exit 2) | PASS | all 4 create handlers now `InvalidInput`; headless pin `.code(2)`; 12 `custom_field_json.is_some()` sites counted |

---

_Reviewed: 2026-09-04_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: standard_
