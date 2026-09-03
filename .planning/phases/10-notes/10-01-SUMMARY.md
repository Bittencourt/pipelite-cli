---
phase: 10-notes
plan: 01
subsystem: cli
tags: [notes, clap, rest, subresource, stub-testing, tdd]

# Dependency graph
requires:
  - phase: 08-foundations-error-layer-models-pagination
    provides: parse_rfc7807 + handle_response/handle_delete_response with per-surface forbidden_hint (\"notes\" pre-registered), InvalidInput/MissingInput exit-2 error variants
  - phase: 07-batch / 09-templates
    provides: templates single_delete confirmation contract, head+body-capturing stub-server helpers (tests/common/mod.rs), top-level command-group pattern
provides:
  - Top-level `notes` command group (list/add/edit/delete + hidden get rejection) on deals/orgs/people/activities
  - Note/NoteCreate/NoteUpdate models (serializer-exact: singular entity_type, content-only wire payload, no deleted_at)
  - 4 PipeliteClient methods (list_notes, create_note, update_note, delete_note) on surface \"notes\"
  - resolve_body source resolver (--body > @file/@- > --stdin > prompt) with pre-HTTP XOR source guard
  - resolve_entity_type gate (4 capable types, plural→segment map) reused by every notes handler
  - truncate_with_ellipsis activated (dead_code attribute removed) for table-mode content cells
  - tests/notes_stub_test.rs — 25 wire-level integration tests
affects: [11-webhooks, 12-custom-fields, docs]

# Tech tracking
tech-stack:
  added: []  # no new crates — clap/reqwest/serde/dialoguer/tempfile already pinned
  patterns: [parent-scoped sub-resource (collection POST vs item PATCH/DELETE by note ID), body-source resolver with XOR guard, table-only truncation/flattening in the value builder]

key-files:
  created:
    - src/cli/notes.rs
    - src/commands/notes/mod.rs
    - src/commands/notes/list.rs
    - src/commands/notes/body.rs
    - src/commands/notes/add.rs
    - src/commands/notes/edit.rs
    - src/commands/notes/delete.rs
    - tests/notes_stub_test.rs
  modified:
    - src/api/models.rs
    - src/api/mod.rs
    - src/output/format.rs
    - src/cli/mod.rs
    - src/commands/mod.rs
    - src/main.rs
    - tests/help_examples_test.rs
    - docs/api-reference.md

key-decisions:
  - "Table-only content truncation: newline flattening + ~80-char truncate_with_ellipsis applied in the table value builder ONLY — json/plain/csv keep full raw text (plain treated as machine-readable like json)"
  - "Mutation success output keys on the RESOLVED output format: json renders the note via render_single, other formats print the quiet-suppressible confirmation line (Created/Updated/Deleted note)"
  - "Body-source XOR guard counts --body and --stdin BEFORE any read, so the stdin-backed pair --body @- + --stdin can never consume each other's data (open question 3, exit 2)"
  - "Prompting suppressed under --dry-run by passing no_input || dry_run into resolve_body — a dry-run with no explicit source previews the MissingInput rejection instead of blocking on stdin"

patterns-established:
  - "Parent-scoped sub-resource: collection route gets the parent segment, item routes carry ONLY the item ID (parent args validated then unused)"
  - "Entity-type positional as plain String + handler-side resolve_entity_type gate — never clap ValueEnum, so the locked rejection message survives"
  - "resolve_body resolver: reusable precedence chain for any future free-text input (flag > @file/@- > stdin > prompt)"

requirements-completed: [NOTE-01, NOTE-02, NOTE-03, NOTE-04]

# Metrics
duration: 23min
completed: 2026-09-03
---

# Phase 10 Plan 01: Notes Surface Summary

**Top-level `notes` group (list/add/edit/delete + hidden-get rejection) with serializer-exact Note model, note-ID-only PATCH/DELETE paths, locked body-source precedence, and 25 wire-level stub tests — full suite 402 green**

## Performance

- **Duration:** 23 min
- **Started:** 2026-09-03T21:58:58Z
- **Completed:** 2026-09-03T22:22:25Z
- **Tasks:** 3
- **Files modified:** 16

## Accomplishments

- NOTE-01: `notes list <type> <parent-id>` on all four capable types — newest-first server order, `--limit/--offset` wire-passthrough (no `--all`; server page cap 100), id/created_at/truncated-flattened content columns, full text via `--format json`, quiet-suppressible empty-page hint, parent 404 → NotFound exit 1
- NOTE-02: `notes add` posts exactly `{"content": …}` sourced by the locked precedence (flag > @file > @-/--stdin > TTY prompt); every ambiguity rejected pre-HTTP with exit 2
- NOTE-03: `notes edit` PATCHes `/api/v1/notes/{noteId}` with no confirmation; the no-single-GET wording lives in the prompt label, after_help, and api-reference
- NOTE-04: `notes delete` runs the exact templates delete contract (dry-run first → TTY Confirm → `--force` → non-TTY exit-1 refusal); sequential re-delete 404s as NotFound
- SC-5: `notes list pipelines <id>` and hidden `notes get` reject exit 2 with locked hints and zero HTTP; 403 on foreign-note edit renders the pre-registered notes Forbidden hint

## Task Commits

Each task was committed atomically (TDD: RED test commit before GREEN feat commit):

1. **Task 1: Note model, list_notes client, notes group wiring, list handler + rejection surfaces** — `a2c1330` (test, RED) + `0fcdc16` (feat, GREEN)
2. **Task 2: Body resolver + add/edit/delete handlers** — `594c2e9` (test, RED) + `31bbc86` (feat, GREEN)
3. **Task 3: Help truthfulness tests + api-reference Notes section + full-suite gate** — `ce96597` (test/docs)

## Files Created/Modified

- `src/cli/notes.rs` — NotesCommands enum (List/Add/Edit/Delete + hidden Get), per-subcommand after_help examples
- `src/commands/notes/mod.rs` — dispatch, hidden-Get parse-then-error rejection, resolve_entity_type gate
- `src/commands/notes/list.rs` — list handler with table-only flattening/truncation and empty-page hint
- `src/commands/notes/body.rs` — resolve_body source precedence with pre-read XOR guard
- `src/commands/notes/add.rs` — add handler (body resolves before dry-run intercept; never prompts under --dry-run)
- `src/commands/notes/edit.rs` — edit handler (PATCH by note ID, no confirmation, locked prompt wording)
- `src/commands/notes/delete.rs` — delete handler (verbatim templates single_delete contract)
- `src/api/models.rs` — Note/NoteCreate/NoteUpdate + notes_table_config + 3 unit tests
- `src/api/mod.rs` — list_notes/create_note/update_note/delete_note, all surface "notes"
- `src/output/format.rs` — truncate_with_ellipsis activated (`#[allow(dead_code)]` removed)
- `src/cli/mod.rs`, `src/commands/mod.rs`, `src/main.rs` — wiring (1–3-line touches)
- `tests/notes_stub_test.rs` — 25 stub-server integration tests (wire shapes, source matrix, zero-HTTP rejections, delete contract, truncation, 422/403/404 passthrough)
- `tests/help_examples_test.rs` — notes_help_truthful + notes_subcommands_have_examples
- `docs/api-reference.md` — complete `pipelite notes` section

## Decisions Made

- Table-mode truncation lives in the value builder only; plain/csv/json keep the full raw text (plain is machine-readable like json — agent's-discretion call per plan)
- Success output for add/edit branches on the resolved output format: json → render_single of the note, otherwise the quiet-suppressible confirmation line (delete always prints its line, matching templates delete)
- Two-source guard counts explicit sources before ANY read; `--body @-` + `--stdin` = two sources → exit 2 (open question 3)
- No entity_id mismatch warning on edit/delete (open question 2 deferred — server 404 covers it)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Two Task-2 RED tests passed accidentally (weak exit-code-only assertions)**
- **Found during:** Task 2 (RED phase)
- **Issue:** `add --body x --stdin` and `add --body @- --stdin` expected exit 2; clap's own "unrecognized subcommand" also exits 2 pre-implementation, so the tests passed for the wrong reason (TDD fail-fast rule tripped)
- **Fix:** Strengthened both to assert the locked rejection text ("Multiple note body sources" / "exactly one"), restoring true RED
- **Files modified:** tests/notes_stub_test.rs
- **Verification:** 17/17 new tests fail pre-implementation, pass post-implementation
- **Committed in:** 594c2e9

**2. [Rule 1 - Bug] Edit success-line assertion conflicted with piped-default JSON format**
- **Found during:** Task 2 (GREEN phase)
- **Issue:** The plan's edit test asserted stdout contains "Updated note n1" under a default invocation, but assert_cmd pipes stdout → detect_format resolves JSON → the handler (per the plan's own action spec: json → render_single, else confirmation line) printed the note JSON, not the line
- **Fix:** Test now covers both paths deterministically — default piped run asserts the note data (id), `--format table` run asserts "Updated note n1". Implementation unchanged (matches the plan's action text verbatim)
- **Files modified:** tests/notes_stub_test.rs
- **Verification:** 25/25 stub tests pass
- **Committed in:** 31bbc86

---

**Total deviations:** 2 auto-fixed (2 × Rule 1 test-level corrections; no production-code deviation)
**Impact on plan:** Both fixes align tests with environment semantics (piped default format, clap exit codes) without changing locked behavior. No scope creep.

## Issues Encountered

- `std::io::stdin().is_terminal()` needed an explicit `use std::io::IsTerminal;` in body.rs (compile fix during GREEN; trivial)

## TDD Gate Compliance

- Task 1: `test(...)` commit `a2c1330` precedes `feat(...)` commit `0fcdc16` ✓
- Task 2: `test(...)` commit `594c2e9` precedes `feat(...)` commit `31bbc86` ✓
- Task 3 is a non-TDD task (help tests + docs) — no gate required

## User Setup Required

None — no external service configuration required.

## Next Phase Readiness

- Notes surface end-to-end; the body-source resolver (`resolve_body`) and the templates delete contract are reusable patterns for Phase 11 (webhooks/trash) free-text inputs
- forbidden_hint("notes") was pre-registered and consumed as-is — the 403 path is wire-proven for future author-or-admin surfaces
- No blockers; full suite 402 passing (149 unit + 253 integration across all suites)

## Threat Surface Scan

No new security-relevant surface outside the plan's threat model: @file reads (T-10-02), delete confirmation UX (T-10-03), 403 rendering (T-10-04), and table flattening (T-10-01) are all implemented and stub-tested as planned.

## Self-Check: PASSED

- All 8 created files exist on disk (verified)
- All 5 task commits present in git log (a2c1330, 0fcdc16, 594c2e9, 31bbc86, ce96597)
- Full `cargo test` suite: 402 passed, 0 failed

---
*Phase: 10-notes*
*Completed: 2026-09-03*
