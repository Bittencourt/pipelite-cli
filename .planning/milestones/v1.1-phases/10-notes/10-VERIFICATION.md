---
phase: 10-notes
verified: 2026-09-03T23:59:00Z
status: passed
score: 5/5 must-haves verified
overrides_applied: 0
human_verification:
  - test: "In a real terminal, run `pipelite notes add deals <real-parent-id>` with no --body/--stdin"
    expected: "An interactive prompt appears ('Add note: Note content'); typed text is posted as exactly {\"content\": <typed text>} and the success line prints"
    why_human: "The dialoguer prompt path requires a real TTY (is_terminal() gate); non-TTY branches are stub-proven but the interactive read cannot be driven by pipes"
  - test: "In a real terminal, run `pipelite notes delete deals <parent-id> <note-id>` without --force"
    expected: "'Delete note <id>?' Confirm appears (default N); declining prints 'Aborted' with zero HTTP; accepting soft-deletes the note; the edit prompt (no source) shows the locked no-single-GET wording"
    why_human: "TTY Confirm interaction is real-time terminal behavior; the wording is grep-verified in source but the dialoguer UX (default, abort message, read path) needs eyes on a terminal"
---

# Phase 10: Notes Verification Report

**Phase Goal:** Users can attach, revise, and remove notes on deals, organizations, people, and activities — the first parent-scoped sub-resource
**Verified:** 2026-09-03T23:59:00Z
**Status:** human_needed (5/5 truths machine-verified; 2 interactive-TTY items deferred to human)
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
| --- | --- | --- | --- |
| 1 | User can list notes on any deal, org, person, or activity (SC-1) | ✓ VERIFIED | Live probe: GET `/api/v1/deals/d1/notes?limit=10&offset=5` on the wire (limit/offset passthrough proven); table renders id + created_at column + content; `--format json` returns FULL raw text (newline intact, 120-char body untruncated); `orgs` maps to `/organizations/` segment; empty page (meta.total==0) prints one stderr hint, suppressed by `--quiet`; parent 404 renders "Deal not found" exit 1 |
| 2 | User can add a note via flag, @file, stdin, or prompt on any of the four entities (SC-2) | ✓ VERIFIED | Wire probe: POST body is EXACTLY `{"content":"from flag"}` (one key); `--stdin` posts stdin text; `orgs`→`/organizations/o1/notes`; two sources (`--body x --stdin`) exit 2 pre-HTTP; unreadable `@file` exit 2 with check-the-path hint; `--no-input` without source exit 2 MissingInput; `--dry-run` previews POST with zero HTTP. Prompt leg: code verified (body.rs:74-83, locked wording), TTY interaction → human item #1 |
| 3 | User can edit a note by ID, view-before-edit documented, prompt explains why content can't be shown (SC-3) | ✓ VERIFIED | Wire probe: PATCH to `/api/v1/notes/n1` ONLY (no `/deals/` in URL), body `{"content":"new text"}`; no-single-GET wording present in 4 places: hidden-get hint (commands/notes/mod.rs:27), edit prompt label (body.rs context via edit.rs:34-37), edit after_help (cli/notes.rs:20), group after_help (cli/mod.rs:164), docs § notes edit; 422 errors[] renders "Note content is required"; 403 renders registered "You can only modify your own notes" hint exit 1 |
| 4 | User can delete a note by ID with confirmation, `--force` bypass (SC-4) | ✓ VERIFIED | Wire probe: `--force` → DELETE `/api/v1/notes/n1` against 204 → exit 0 "Deleted note n1"; non-TTY without `--force` → exit 1, `--force` hint, ZERO requests issued; `--dry-run` previews the URL exit 0 zero HTTP; sequential delete→re-delete 404 "Note not found" covered by test `delete_then_redelete_404s_as_not_found` (passes); dry-run intercept precedes prompt (delete.rs:34 before :40) |
| 5 | Notes on pipelines/stages/workflows parents and `notes get` rejected with actionable hint, zero HTTP (SC-5) | ✓ VERIFIED | Live probe: `notes list pipelines 123` → exit 2, "notes are only available on", NO connection attempt (unreachable server, no "Connection failed"); `notes get deals 123` → exit 2, "no single-note GET — use `notes list <type> <id> --json`"; `notes --help` contains no `get` subcommand line and no `--all` |

**Score:** 5/5 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
| -------- | -------- | ------ | ------- |
| `src/api/models.rs` (Note) | Serializer-exact Note + NoteCreate/NoteUpdate + notes_table_config | ✓ VERIFIED | Note at :731 — id, singular entity_type String, entity_id, content, `#[serde(default)]` nullable author_id, source, optional created_at/updated_at; NO deleted_at, NO expand map. NoteCreate/NoteUpdate content-only. 3 unit tests incl. exact-one-key serialization |
| `src/api/mod.rs` (notes methods) | list/create/update/delete on surface "notes" | ✓ VERIFIED | :1018 list_notes (query limit/offset), :1046 create_note, :1071 update_note (note-ID-only URL), :1086 delete_note — all pass surface "notes"; forbidden_hint pre-registered at :96 (untouched) |
| `src/cli/notes.rs` | NotesCommands List/Add/Edit/Delete + hidden Get, plain-String positional, after_helps | ✓ VERIFIED | All 5 variants; per-subcommand after_help with Examples; hidden Get with permissive args; entity_type plain String (not ValueEnum) so the locked rejection survives |
| `src/commands/notes/mod.rs` | dispatch + resolve_entity_type gate + hidden-Get parse-then-error | ✓ VERIFIED | resolve_entity_type maps 4 types→segments, else InvalidInput exit 2 with locked message; Get(_) rejects pre-HTTP |
| `src/commands/notes/body.rs` | resolve_body precedence + XOR guard before any read | ✓ VERIFIED | Source count at :38 precedes every read (@- + --stdin can't consume each other); @file→InvalidInput exit 2; bare `@` rejected (review fix IN-01); prompt fallback gated on is_terminal && !no_input |
| `src/commands/notes/delete.rs` | templates single_delete contract | ✓ VERIFIED | Dry-run FIRST (:34) → TTY Confirm default false (:42-45) → non-TTY CliError::Validation refusal (:51) → delete_note (:61); single delete only |
| `src/output/format.rs` | truncate_with_ellipsis activated | ✓ VERIFIED | `#[allow(dead_code)]` removed; function live, used at list.rs:78 |
| `tests/notes_stub_test.rs` | ≥25 wire-level tests | ✓ VERIFIED | 27 test functions, all passing (25 planned + 2 review-fix regression locks) |
| `tests/help_examples_test.rs` | notes help truthfulness tests | ✓ VERIFIED | notes_help_truthful + notes_subcommands_have_examples, passing |
| `docs/api-reference.md` | `## pipelite notes` section | ✓ VERIFIED | :369 — all 4 subcommands, 4 valid types, body-source precedence, --force, no-single-GET, rejection surfaces, 403 semantics, copy-pasteable examples per subcommand |

### Key Link Verification

| From | To | Via | Status | Details |
| ---- | --- | --- | ------ | ------- |
| commands/notes/add.rs | api create_note | `create_note(` | ✓ WIRED | add.rs:53 call with NoteCreate; wire-proven POST |
| commands/notes/body.rs | fs/stdin reads | `read_to_string` | ✓ WIRED | body.rs:58 file read; read_stdin_to_end for @- and --stdin; XOR guard precedes both |
| edit.rs / delete.rs | /api/v1/notes/{noteId} | `api/v1/notes/` | ✓ WIRED | edit.rs:40, delete.rs:35 (dry-run) + api/mod.rs:1072/1087; wire probe confirmed PATCH/DELETE carry only note ID |
| api notes methods | forbidden_hint table | surface `"notes"` | ✓ WIRED | handle_response/handle_delete_response with "notes" (4 sites); 403 probe rendered the registered hint |
| commands/notes/list.rs | truncate_with_ellipsis | import + use | ✓ WIRED | list.rs:8 import, :78 usage in table-only value builder; probe proved table truncated/flattened, json full |
| cli/mod.rs + main.rs | notes group | `pub mod notes` / `Commands::Notes` | ✓ WIRED | cli/mod.rs:9,161-164; commands/mod.rs:9; main.rs:104 dispatch |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
| -------- | ------------- | ------ | ------------------ | ------ |
| list.rs rendering | response.data | ctx.client.list_notes → live stub server | Yes — stub Note rendered to table/json stdout | ✓ FLOWING |
| add.rs success output | note (created) | create_note response unwrap .data | Yes — created id echoed | ✓ FLOWING |
| edit.rs success output | note (updated) | update_note response | Yes | ✓ FLOWING |

No hardcoded-empty props; no static returns in the notes path (4 client methods all consume server responses).

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
| -------- | -------- | ------ | ------ |
| Help truthfulness | `pipelite notes --help` | exit 0; "Examples:" + "no single-note GET" + "--body" present; no "--all"; no `get` subcommand line | ✓ PASS |
| Non-capable parent, zero HTTP | `notes list pipelines 123` (unreachable server) | exit 2; locked message; no "Connection failed" | ✓ PASS |
| Hidden get rejection | `notes get deals 123` | exit 2; "The server exposes no single-note GET" + hint | ✓ PASS |
| List wire passthrough | stub GET | `/api/v1/deals/d1/notes?limit=10&offset=5` on the wire | ✓ PASS |
| Table flatten + truncate | stub 2 notes (newline + 120-char) | "line1 line2" in table; 78+ consecutive chars absent; "..." present; json keeps full raw text | ✓ PASS |
| Empty-page hint + quiet | stub total:0 | stderr "No notes … add one with: … --body"; exit 0; `--quiet` suppresses | ✓ PASS |
| Parent 404 passthrough | stub 404 | exit 1; "Deal not found" | ✓ PASS |
| Add wire shape | stub POST | body exactly `{"content":"from flag"}`; stdin and orgs-segment variants also exact | ✓ PASS |
| Edit wire shape | stub PATCH | PATCH `/api/v1/notes/n1` (parent args absent from URL); body `{"content":"new text"}` | ✓ PASS |
| Delete contract | stub 204 + no-server | --force exit 0; non-TTY refusal exit 1 zero requests with --force hint; dry-run previews zero HTTP | ✓ PASS |
| 403 hint rendering | stub 403 on PATCH | exit 1; "You can only modify your own notes (or use an admin key)." | ✓ PASS |
| 422 passthrough | stub errors[] fixture | exit 1; "Note content is required" (whitespace body sent, no client trim) | ✓ PASS |
| Source-matrix rejections | flag+stdin, @-+stdin, unreadable @file, missing source --no-input | all exit 2 pre-HTTP with locked hints | ✓ PASS |
| Dry-run previews (add/edit/delete) | --dry-run | POST/PATCH/DELETE + URL (+content) in stdout; zero HTTP | ✓ PASS |

### Probe Execution

| Probe | Command | Result | Status |
| ----- | ------- | ------ | ------ |
| Full test suite | `cargo test` | 0 failed across all suites (149 unit + 27 notes stub + all pre-existing) | PASS |
| Notes stub suite | `cargo test --test notes_stub_test` | 27 passed; 0 failed | PASS |
| Build | `cargo build` | success (3 pre-existing warnings, none in notes code per review) | PASS |
| Release commits | `git log` a2c1330, 0fcdc16, 594c2e9, 31bbc86, ce96597 + fixes a2e8b8f/6ce3214/bcc22ec/ffb9242 | all present | PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
| ----------- | ---------- | ----------- | ------ | -------- |
| NOTE-01 | 10-01 | List notes on a deal, organization, person, or activity | ✓ SATISFIED | Truth 1; wire probe + tests 1-8 |
| NOTE-02 | 10-01 | Add a note via flag, @file, stdin, or interactive prompt | ✓ SATISFIED | Truth 2; wire probe + tests 10-19 (prompt leg human item) |
| NOTE-03 | 10-01 | Edit a note by ID; document `notes list --json` as view-before-edit | ✓ SATISFIED | Truth 3; wording in 4 surfaces + docs |
| NOTE-04 | 10-01 | Delete a note by ID with confirmation and --force bypass | ✓ SATISFIED | Truth 4; delete contract wire-proven |

No orphaned requirements — all Phase-10 IDs in REQUIREMENTS.md (NOTE-01..04) are claimed by plan 10-01 and satisfied.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| ---- | ---- | ------- | -------- | ------ |
| (none) | — | Zero TBD/FIXME/XXX/TODO/HACK/PLACEHOLDER in all 11 phase-touched source files; zero unwrap()/expect() in notes handlers; dead_code attribute removed | — | Clean |

### Deferred Items (informational — accepted review findings, not gaps)

| # | Item | Addressed In | Evidence |
|---|------|-------------|----------|
| 1 | URL path-ID percent-encoding (IN-04, accepted info) | Phase 13 | Phase 13 goal: "Integration Hardening & Docs Refresh — every new surface honors the v1.0 global contract"; review documents it as a codebase-wide hardening item, mirroring all pre-existing entities |

### Human Verification Required

### 1. Interactive add/edit prompt (TTY)

**Test:** In a real terminal run `pipelite notes add deals <real-parent-id>` (no --body/--stdin), then `pipelite notes edit deals <parent> <note-id>` with no source
**Expected:** Prompt appears ("Add note: Note content"; edit shows the locked no-single-GET wording); typed text posts as exactly `{"content": <text>}`
**Why human:** `is_terminal()` gate means the prompt path only activates on a real TTY; pipes/stubs cannot exercise dialoguer's interactive read

### 2. Interactive delete confirmation (TTY)

**Test:** In a real terminal run `pipelite notes delete deals <parent> <note-id>` without --force; try both declining and accepting
**Expected:** "Delete note <id>?" Confirm (default N); declining prints "Aborted" with zero HTTP; accepting soft-deletes and prints the confirmation line
**Why human:** Real-time TTY confirm UX (default value, abort message) can't be driven by the stub harness

### Gaps Summary

No gaps. All five roadmap success criteria are machine-verified against the real binary with wire-level stub probes: correct REST routes (collection POST on the parent segment vs item PATCH/DELETE by note ID), exact `{"content": …}` wire bodies, locked pre-HTTP rejection surfaces with zero HTTP, the full templates delete contract, table-only truncation/flattening with full-fidelity json, and truthful help/docs. Full suite green (149 unit + 27 notes stub tests + all pre-existing suites). Code review is final/clean (0 critical, 0 warning). Status is human_needed solely because the two interactive TTY prompt paths (add/edit prompt fallback, delete Confirm) cannot be exercised by the stub harness; their wording and gating are source-verified.

---

_Verified: 2026-09-03T23:59:00Z_
_Verifier: the agent (gsd-verifier)_
