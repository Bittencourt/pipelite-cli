---
phase: 11-webhooks-trash-audit
plan: 02
subsystem: api
tags: [trash, restore, purge, rust, clap, reqwest, stub-server-tests, confirmation-hierarchy, fan-out]

# Dependency graph
requires:
  - phase: 08-foundations-error-layer-models-pagination
    provides: forbidden_hint table ("trash" purge hint pre-registered, "general" fallback), InvalidInput exit-2 mechanism, ApiListResponse envelope
  - phase: 09-workflow-runs-templates-docs
    provides: tests/common/mod.rs head+body-capturing stub server, cmd()/cmd_with_server() hermetic helpers, docs-command 404 re-wrap precedent
  - phase: 10-notes
    provides: resolve_entity_type normalization precedent (deliberately not a ValueEnum), table-only truncation pattern
  - phase: 11-01
    provides: top-level group wiring pattern (cli/commands/main), webhooks delete as the exit-1 Validation refusal counterpart, sequential shared-wiring discipline
provides:
  - top-level `trash` command group (list/restore/purge) against /api/v1/trash
  - TrashRow (dual type tokens: entity_type singular + tab plural) + DeletedBy closed union (tag="kind", nameless api_key)
  - 3 PipeliteClient methods (list_trash/restore_trash/purge_trash) — surfaces: list/restore "general", purge "trash"
  - normalize_trash_type: 9 singular/plural aliases → 4 plural tabs, pre-HTTP exit 2 (shared by all three subcommands)
  - trash list --all bounded fan-out (limit=100, four break conditions incl. offset 10,000 cap)
  - purge zero-HTTP exit-2 non-TTY refusal (InvalidInput) + strongest-confirm prompt (scope + count + "permanently destroys", unit-gated) + continue-on-error N ok / M failed summary
affects: [11-03 audit (sequential shared wiring), CHANGELOG surface notes]

# Tech tracking
tech-stack:
  added: [] # zero new crates — clap/reqwest/serde/dialoguer/comfy-table already pinned
  patterns: [alias-normalization pre-HTTP, bounded CLI fan-out with offset cap, zero-HTTP refusal gate (InvalidInput exit 2), continue-on-error with pinned bail! summary, table-only truncation with full json passthrough]

key-files:
  created:
    - src/cli/trash.rs
    - src/commands/trash/mod.rs
    - src/commands/trash/list.rs
    - src/commands/trash/restore.rs
    - src/commands/trash/purge.rs
    - tests/trash_stub_test.rs
  modified:
    - src/api/models.rs
    - src/api/mod.rs
    - src/cli/mod.rs
    - src/commands/mod.rs
    - src/main.rs
    - tests/help_examples_test.rs
    - docs/api-reference.md

key-decisions:
  - "TrashRow carries BOTH type tokens — entity_type (singular, display) and tab (wire key \"type\", plural, URL round-trip token); only normalized plural tabs ever reach a URL (server 422s singular, P5)"
  - "Purge non-TTY no---force refusal is InvalidInput exit 2 firing BEFORE the fan-out list (zero HTTP for the whole command, pinned on an unreachable server) — deliberately stricter than webhooks delete's Validation exit-1; the word Validation never appears in purge.rs"
  - "Restore 403 keeps surface \"general\" (owner-or-admin wording, never the purge hint — P6 pin, stub-tested for both presence and absence); restore 404 re-wraps preserving server detail + not-in-trash hint (docs-command precedent)"
  - "--all and purge victim fan-out both break on ALL FOUR conditions (empty page, partial page, accumulated >= meta.total, offset > 10,000) — the loop trusts page emptiness, never total reachability (P3/S6)"
  - "purge_prompt is a pure function unit-gated to contain scope + count + 'permanently destroys' (TTY-only path); per-item failures are continue-on-error with the admin trash hint on 403s, summary '{ok} permanently destroyed[, M failed]' + pinned anyhow::bail! for exit 1"
  - "Trash is never cached — no cache write/read anywhere in the trash module (grep-pinned: the word 'cache' cannot appear in list.rs)"

requirements-completed: [TRSH-01, TRSH-02, TRSH-03]

# Metrics
duration: 15min
completed: 2026-09-04
---

# Phase 11 Plan 02: Trash List/Restore/Purge Summary

**Top-level `trash` group with 9-alias plural-tab normalization feeding every URL, bounded `--all` fan-out at the server's 10,000-offset cap, restore-as-recovery (no confirm), and purge as a zero-HTTP exit-2-refused, strongest-confirmed, continue-on-error per-record DELETE fan-out — 474 tests green.**

## Performance

- **Duration:** 15 min
- **Started:** 2026-09-04T02:40:39Z
- **Completed:** 2026-09-04T02:55:41Z
- **Tasks:** 3
- **Files modified:** 13

## Accomplishments

- TrashRow matches the serializer exactly (both type tokens, `linked_parents` defaults to [], DeletedBy closed union on `kind` with the deliberately nameless api_key) — model unit tests pin the closed union and unknown-kind rejection
- All three subcommands normalize 9 singular/plural aliases to the 4 plural tabs BEFORE any HTTP; unknown types exit 2 with zero requests (stub-proven on an unreachable server) — the round-trip contract holds: list rows carry the plural `type` token that pipes via jq straight into restore
- `trash list --all` fans out at limit=100 with all four break conditions (partial-page and empty-page breaks stub-proven); table cells truncate linked_parents (~80 chars) and collapse deleted_by to kind labels while `--json` keeps the full array/objects (T-11-09); trash is never cached
- Restore is the recovery act — no confirmation; plural-tab POST URL stub-proven (P5); 404 re-wraps preserving the server detail with the not-in-trash hint; foreign-record 403 renders the general wording and never the purge hint (P6 pin)
- Purge honors the full locked contract: scope validated pre-HTTP, non-TTY without `--force` refuses InvalidInput exit 2 with zero HTTP (PINNED unreachable-server test), `--dry-run` previews victims with one list GET and zero DELETEs, TTY confirm names scope + count + "permanently destroys" (unit-gated), fan-out DELETEs continue past per-item failures (403s render the admin trash hint), and the "N permanently destroyed, M failed" summary exits 1 via the pinned `anyhow::bail!`
- Help pages are truthful (purge warning at group + subcommand level, `--all` 10,000-cap note, jq round-trip example, restore no-confirm semantics) and docs/api-reference.md gained a complete `pipelite trash` section

## Task Commits

Each task was committed atomically (TDD: RED then GREEN for tasks 1-2):

1. **Task 1: TrashRow/DeletedBy models, 3 client methods, normalize_trash_type, trash list + --all fan-out + wiring** — `72bd6d6` (test, RED: 11 failing list tests) + `1a2c2ca` (feat, GREEN: 11 stub tests + 5 unit tests)
2. **Task 2: restore (no confirm) + purge (zero-HTTP exit-2 refusal, strongest confirm, fan-out, N ok / M failed)** — `6dc0eec` (test, RED: 13 failing) + `c3dfabd` (feat, GREEN: 24 stub tests total)
3. **Task 3: Help truthfulness tests + docs/api-reference.md Trash section + full-suite gate** — `5056795` (feat)

## Files Created/Modified

- `src/api/models.rs` — TrashRow (dual tokens), DeletedBy (tag="kind", 7 variants), trash_table_config + 2 unit tests
- `src/api/mod.rs` — list_trash (type param only when Some, surface "general"), restore_trash (plural-tab POST, surface "general"), purge_trash (DELETE, surface "trash") with contract doc comments
- `src/cli/trash.rs` — TrashCommands List/Restore/Purge; plain-String type args (NOT ValueEnum — the locked exit-2 hint fires); per-subcommand after_help incl. round-trip + cap notes
- `src/commands/trash/mod.rs` — normalize_trash_type (9→4, case-sensitive, exit 2), deleted_by_label, dispatch + 3 unit tests
- `src/commands/trash/list.rs` — --type/--limit/--offset/--all/--fields; fetch_all four-condition fan-out; table-only truncation; empty hint; zero persistence
- `src/commands/trash/restore.rs` — normalize → dry-run → restore; 404 re-wrap preserving detail; general 403 passthrough
- `src/commands/trash/purge.rs` — order-of-operations contract (normalize → dry-run → zero-HTTP refusal → collect → confirm → continue-on-error execute → summary + pinned bail); purge_prompt pure fn + wording unit test
- `src/cli/mod.rs`, `src/commands/mod.rs`, `src/main.rs` — Trash wiring after Webhooks with group after_help ("restore needs no confirmation; purge permanently destroys")
- `tests/trash_stub_test.rs` — 24 stub tests (11 list + 13 restore/purge; the purge refusal test pinned)
- `tests/help_examples_test.rs` — 3 new truthfulness tests
- `docs/api-reference.md` — `## \`pipelite trash\`` section before Error Codes

## Decisions Made

- Query-param order is limit, offset, then type (type last) — heads assert `limit=50&offset=0` and `type=deals` independently
- Long-cell table assertions flatten newlines/spaces before comparing (comfy-table wraps long cells at the 120-col piped width; flattening keeps the assertion about CONTENT, not wrap layout)
- Multi-invocation stub tests script one response per invocation (the stub serves exactly one scripted response per connection)
- Purge prints the summary line to stdout in both success and failure cases, then bails with the pinned message (which lands on stderr via error display) so scripts capture "N ok / M failed" on stdout while exit 1 stays item-failure-tier

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] restore_unknown_type hardened to stay non-vacuous in RED**
- **Found during:** Task 2 (RED authoring)
- **Issue:** The spec'd assertions (exit 2 + counter 0) would pass vacuously while `trash` restore was unimplemented — clap's unrecognized-subcommand error also exits 2 with zero requests
- **Fix:** Added stderr assertions ("Unknown trash type" present, "Connection failed" absent) mirroring the plan's own purge_scope_validation_first pattern; RED then failed for the right reason
- **Files modified:** tests/trash_stub_test.rs
- **Verification:** RED showed 13/13 failing; GREEN passes
- **Committed in:** 6dc0eec

**2. [Rule 3 - Blocking] Two-iteration tests crashed on their second invocation**
- **Found during:** Task 1 (GREEN)
- **Issue:** list_linked_parents_truncation and list_deleted_by_variants invoke the CLI twice (table + json) but scripted one response; the stub serves one response per connection, so the second run hit "Connection failed"
- **Fix:** Scripted a second identical page for both tests
- **Files modified:** tests/trash_stub_test.rs
- **Verification:** cargo test --test trash_stub_test 11/11 green
- **Committed in:** 1a2c2ca

**3. [Rule 3 - Blocking] Task 2 gate `grep -c 'fn purge_prompt' == 1` counted the unit test name**
- **Found during:** Task 2 gate run
- **Issue:** The unit test `fn purge_prompt_names_...` contains the substring, making the count 2
- **Fix:** Renamed the test to `prompt_wording_names_scope_count_and_destruction` — same intent, gate now satisfiable as written (same class of fix as 11-01 deviation #2)
- **Files modified:** src/commands/trash/purge.rs (test name only)
- **Verification:** full Task 2 gate prints GATE-OK
- **Committed in:** c3dfabd

**4. [Rule 3 - Blocking] purge help wording was all-caps, failing the case-sensitive help assertion**
- **Found during:** Task 3
- **Issue:** after_help said "PERMANENTLY DESTROYS RECORDS." — `contains("permanently")` is case-sensitive
- **Fix:** Reworded to "purge permanently destroys records — this cannot be undone." (also gains the exact "permanently destroys" phrase)
- **Files modified:** src/cli/trash.rs
- **Verification:** cargo test --test help_examples_test 16/16 green
- **Committed in:** 5056795

---

**Total deviations:** 4 auto-fixed (1 test-hardening for RED integrity, 3 gate/test-mechanics corrections)
**Impact on plan:** All four were mechanical test/mechanics corrections — no behavior or contract changes. Every must_have, pinned gate, and success criterion met as written.

## Issues Encountered

None beyond the deviations above. Full suite green on the first post-Task-3 run (474 passed / 0 failed across 31 suites).

## Threat Surface Scan

No new trust boundaries beyond the plan's threat model: no endpoints beyond /api/v1/trash (in model), no auth paths added, no schema changes, no new crates (T-11-SC holds). T-11-05 (mass purge) mitigated and test-proven at every layer (wording gate, exit-2 zero-HTTP refusal, --force bypass only, empty-scope short-circuit); T-11-07 (URL token allow-list) proven by counter==0 unknown-type tests; T-11-08 (misleading 403) proven by hint-presence + "purge"-absence assertions; T-11-09 (display truncation) proven table-vs-json in both directions.

## User Setup Required

None — no external service configuration required.

## Known Stubs

None — no placeholder data paths; every rendered value comes from the wire or a locked display contract.

## Next Phase Readiness

- Shared wiring proven for the third group in the phase — 11-03 (audit) can follow the identical pattern: client method with passthrough filters + surface "audit", bare/group command, 403 hint probe, no --all
- The confirmation hierarchy now has all three reference implementations coexisting: purge (InvalidInput exit-2 zero-HTTP refusal + strongest confirm) > webhooks delete (Validation exit-1, untouched) > restore (none)
- api/mod.rs at ~1,640 lines — the ~2,000-line split watch item remains open but untriggered

---
*Phase: 11-webhooks-trash-audit*
*Completed: 2026-09-04*

## Self-Check: PASSED

All 6 created files exist on disk; all 5 commits (72bd6d6, 1a2c2ca, 6dc0eec, c3dfabd, 5056795) present in git log. Full suite 474 passed / 0 failed; trash_stub_test 24/24; help_examples_test 16/16.
