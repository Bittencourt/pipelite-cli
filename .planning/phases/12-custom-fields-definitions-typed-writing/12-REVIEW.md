---
phase: 12-custom-fields-definitions-typed-writing
reviewed: 2026-09-04T12:00:00Z
depth: standard
files_reviewed: 8
files_reviewed_list:
  - src/commands/custom_fields/list.rs
  - src/custom_fields.rs
  - src/cache.rs
  - src/api/mod.rs
  - src/api/models.rs
  - src/cli/custom_fields.rs
  - src/batch.rs
  - tests/typed_writing_stub_test.rs
findings:
  critical: 0
  warning: 0
  info: 4
  total: 4
status: clean
---

# Phase 12: Code Review Report (Final Verification, Iteration 2)

**Reviewed:** 2026-09-04
**Depth:** standard (verification re-review of fixes 7717090, ce8e15b, 8f5ef0e)
**Files Reviewed:** 8 (fix scope + supporting infrastructure)
**Status:** clean

## Summary

Both blocking findings from iteration 1 are verified fixed in the current code, each pinned by new passing tests, and the targeted edge-case hunt (complete-page-gate false negatives; fetch-on-miss with the write skipped) found no new Critical or Warning defects. Full suite: **589 passed, 0 failed** (34 test binaries summed — matches the fixer's claim). Clippy: the only remaining hits touching phase-12 scope are duplicates of pre-existing workflow-phase dead-code warnings (`WorkflowRunTrigger`, `TTL_WORKFLOWS`, `get_workflows_cached`) — phase-12 clippy warnings are 0, as claimed.

**CR-01 verdict: FIXED (verified).** The gate at `src/commands/custom_fields/list.rs:42-54` requires `ctx.cache` present AND `--entity-type` given AND `args.offset == 0` AND `response.data.len() as u64 == response.meta.total` before writing `KEY_CUSTOM_FIELDS_*`. A partial page (any `--limit < meta.total`, any non-zero `--offset`) is never cached. Writer audit: exactly two writers touch the `custom_fields_*` keys — this gated one (`list.rs:53`) and the resolver's own complete-as-obtainable write (`src/custom_fields.rs:161-163`, terminates on a partial page independent of `meta.total`); all three definition mutations still invalidate the prefix (`create.rs:125`, `update.rs:122`, `delete.rs:75`). No bypass path exists.

**Adversarial edge-cases probed on the gate (no defects found):**
- **Server returns `total: 0`:** only reachable with empty `data` (valid response for a definition-free entity) → caches `[]`, which is the genuinely complete empty set. `CacheStore::get` (`src/cache.rs:99-120`) deserializes `[]` to `Some(vec![])`, which matches the resolver's `if let Some(cached)` at `src/custom_fields.rs:136-140` → treated as a HIT returning the empty set. Correct semantics (all keys unknown → raw strings + aggregated quiet-suppressible warning), no miss/refetch loop. Staleness beyond the TTL hour for out-of-band definition creation is the pre-existing TTL design, unchanged by this fix.
- **`meta` or `total` missing:** `PaginationMeta.total` is a non-optional `u64` (`src/api/models.rs:35-40`) and `ApiListResponse.meta` is non-Option (`src/api/models.rs:29-32`) — a malformed envelope fails deserialization and errors out before any cache write. There is no silent `total = 0` default path.
- **Server over-reports `total` (total > actual rows):** gate skips the write → resolver performs its own auto-paginated fetch (`src/custom_fields.rs:142-159`), which terminates on a partial page regardless of `total` and caches the complete set from the authoritative writer. Fail-safe; worst case is one extra GET.
- **Tombstone skew:** the definitions list deliberately includes soft-deleted rows (`src/api/mod.rs:1195-1197`), so `data` and `total` count the same rows — no filtering-induced `len != total` mismatch.
- **Fetch-on-miss with the write correctly skipped:** the resolver's `fetch_definitions_cached` is fully independent of the list command; on a miss it GETs, auto-paginates, and writes its own complete set. Pinned both ways by the new stub tests: `partial_page_list_does_not_poison_cache` (1-of-2 page not cached; create re-fetches, request counter 3; second-page field `revenue` stores JSON number `4` on the wire) and `full_page_list_warms_cache` (6/6 page cached; create is POST-only, counter 2; warm `price` stores `4`). Both pass.
- **Type soundness of the gate:** `args.offset` is `u64` defaulting to 0 (`src/cli/custom_fields.rs:50`); `usize as u64` is lossless on all supported targets; the `let`-chain matches the pattern already used at `src/custom_fields.rs:136-140` and compiles clean on this toolchain.

**WR-01 verdict: FIXED (verified).** `options_of` (`src/custom_fields.rs:369-376`) now returns `Some(vec![])` for a genuinely empty array (strict validation — intentional and pinned) and `None` for any non-empty array containing a non-string, via `collect::<Option<Vec<&str>>>` (covers all-non-string and mixed arrays alike). `validate_option` (`:382-388`) sends on `None`, and `build_typed_map` (`:244-248`) fires the quiet-suppressible `no_options_note` precisely when `is_none()` — the never-hard-block contract is restored with the note the old code skipped. Both boundaries pinned by passing unit tests `options_non_string_array_is_malformed_sends_without_block` and `options_empty_array_is_genuinely_empty_strict`. Interaction with IN-05 is consistent: a select created via `--options ,` stores `{"options": []}` and still validates strictly — the documented IN-05 Info behavior, deliberately unchanged.

**Trivial fold-ins (commit 8f5ef0e) verified:** IN-01 unused import dropped (`list.rs:3`), IN-02 clippy lints applied (`list.rs:65` point-free `map(serde_json::to_value)`; `custom_fields.rs:136-140` let-chain; `:386` `contains`), IN-06 `batch.rs` trailing newline restored. Fix range `039f828..b982b59` touched exactly the five expected files — no drive-by changes.

## Info (carried open from iteration 1 — not blocking)

### IN-03: TOMBSTONE_NOTE "single source" claim vs hardcoded help copies
**File:** `src/commands/custom_fields/mod.rs:13-19`
Unchanged from iteration 1 (not in fix scope). Follow-up: build the clap `after_help` strings via `concat!` with the const, or soften the doc comment.

### IN-04: Definitions pagination silently caps at 1000
**File:** `src/custom_fields.rs:156-158`
Unchanged (not in fix scope). Follow-up: one quiet-suppressible stderr note when breaking early.

### IN-05: `--options ,` creates a select definition that rejects every value
**File:** `src/commands/custom_fields/create.rs:72-81`
Unchanged (not in fix scope). Follow-up: reject empty resolved option lists at create time.

### IN-07: Pre-existing `unwrap()` calls in heavily-edited create handlers
**File:** `src/commands/deals/create.rs:78-79` (+3 sibling create handlers)
Unchanged, provably safe, pre-dates this phase. Follow-up cleanup only.

## Fix Verification Matrix

| Prior finding | Commit | Verdict | Evidence |
|---|---|---|---|
| CR-01 partial-page cache poisoning | 7717090 | FIXED | Gate `offset == 0 && data.len() == meta.total` at `list.rs:42-54`; tests `partial_page_list_does_not_poison_cache` (counter 3, typed `revenue=4`) + `full_page_list_warms_cache` (counter 2, warm `price=4`) |
| WR-01 non-string options hard-block | ce8e15b | FIXED | `options_of` returns `None` for non-empty no-string arrays (`custom_fields.rs:369-376`); unit tests pin malformed-None and empty-strict boundaries |
| IN-01 unused import | 8f5ef0e | FIXED | `list.rs:3` |
| IN-02 clippy lints (4 sites) | 8f5ef0e | FIXED | `list.rs:65`, `custom_fields.rs:136-140`, `:386`; phase-12 clippy count 5 → 0 |
| IN-06 batch.rs newline | 8f5ef0e | FIXED | EOF newline restored |
| IN-03 / IN-04 / IN-05 / IN-07 | — | Open (Info) | Not in fix scope; carried above |

---

_Reviewed: 2026-09-04_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: standard (verification iteration 2)_
