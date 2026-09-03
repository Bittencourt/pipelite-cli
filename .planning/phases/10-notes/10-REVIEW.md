---
phase: 10-notes
reviewed: 2026-09-03T00:00:00Z
depth: standard
files_reviewed: 16
files_reviewed_list:
  - src/cli/notes.rs
  - src/cli/mod.rs
  - src/commands/mod.rs
  - src/commands/notes/add.rs
  - src/commands/notes/body.rs
  - src/commands/notes/delete.rs
  - src/commands/notes/edit.rs
  - src/commands/notes/list.rs
  - src/commands/notes/mod.rs
  - src/api/mod.rs
  - src/api/models.rs
  - src/main.rs
  - src/output/format.rs
  - tests/notes_stub_test.rs
  - tests/help_examples_test.rs
  - docs/api-reference.md
findings:
  critical: 0
  warning: 2
  info: 4
  total: 6
status: issues_found
---

# Phase 10: Code Review Report

**Reviewed:** 2026-09-03
**Depth:** standard
**Files Reviewed:** 16
**Status:** issues_found

## Summary

Reviewed all phase 10 (notes surface) changes at standard depth: the `Note` model, four client methods, the body resolver, four command handlers, CLI wiring, output truncation, tests, and docs.

**Locked contracts — all verified in code, not just tests:**
- **Body precedence** (`body.rs:37-65`): explicit-source XOR check runs before ANY read (`--body @-` + `--stdin` → `InvalidInput` exit 2, stdin never consumed); `--body` literal > `@file` > `@-` → `--stdin` > TTY prompt; unreadable `@file` → exit 2 with check-the-path hint. ✔
- **Non-capable entity types** (`commands/notes/mod.rs:45-59`): `resolve_entity_type` is the first statement in all four handlers — exit 2, zero HTTP, valid-types hint. ✔
- **Hidden `notes get`** (`commands/notes/mod.rs:25-30`): parse-then-error, exit 2 with the no-single-GET hint; `hide = true` keeps it out of help (asserted by both test files). ✔
- **Edit** (`edit.rs:40-55`): PATCH URL carries only the note ID; no confirmation; `--dry-run` previews the exact PATCH body after body resolution (prompting suppressed under `--dry-run` via `no_input || dry_run`). ✔
- **Delete** (`delete.rs:34-59`): dry-run intercept → TTY confirm (default false) → `--force`; non-TTY without `--force` → `CliError::Validation` (exit 1, zero HTTP). Matches `templates/delete.rs` / `workflows/delete.rs` verbatim. ✔
- **List** (`list.rs`, `format.rs:95-108`): `--limit`/`--offset` pass through untouched (server clamps to [1,100]); table-only truncation flattens `\n`/`\r` first, then truncates to 80 chars with ASCII `...` via char-safe `truncate_with_ellipsis`; json/plain/csv keep full raw text (csv crate quotes embedded newlines — RFC-4180 safe); empty-page stderr hint is `--quiet`-suppressed, exit 0. ✔
- **Note model** (`models.rs:726-749`): serializer-exact — singular `entity_type`, `content`, nullable/`#[serde(default)]` `author_id`, no `deleted_at`, no expand map; `NoteCreate`/`NoteUpdate` serialize to exactly one `content` key (unit-tested). ✔

**Verification:** `cargo test` passes in full (149 unit + all integration suites, including the 22 new notes stub tests); no `unwrap()` in new production code; every new error carries an actionable hint; `--dry-run`/`--no-input`/`--quiet` respected throughout; `send_with_retry` only retries on 429 (never processes the first request), so the retried POST cannot duplicate a note.

No critical issues. Two warnings and four info items below.

## Warnings

### WR-01: Empty-page hint is factually wrong when paging past the end

**File:** `src/commands/notes/list.rs:39-45`
**Issue:** The hint fires on any empty `response.data` and says "No notes on {type} {id} **yet** — add one with: …". With `--offset` beyond `meta.total` (e.g., 5 notes exist, `--offset 10`), the page is empty but notes DO exist — the message tells the user the parent has no notes and nudges them to create a duplicate. `PaginationMeta.total` is already deserialized (`models.rs:36-40`) and passed to `render_list` but is not used to gate the hint.
**Fix:** Only claim "no notes yet" when the collection is truly empty; otherwise stay silent (or emit a page-specific message):

```rust
if response.data.is_empty() && response.meta.total == 0 && !ctx.quiet {
    eprintln!("No notes on {entity_type} {parent_id} yet — add one with: ...");
}
```

### WR-02: Brittle bare-substring assertion in help-truthfulness tests

**File:** `tests/help_examples_test.rs:146-149`, `tests/notes_stub_test.rs:266-269`
**Issue:** Both files assert `!stdout.contains("get")` on the full `notes --help` output. Any future innocuous copy containing the substring "get" ("gets", "target", "widget", a `--budget` flag example) fails CI with a misleading "must not advertise a get subcommand" panic. The sibling templates test does this correctly with a per-line check (`help_examples_test.rs:118-121`).
**Fix:** Match the templates-test pattern — assert no help line *starts with* the subcommand name:

```rust
assert!(
    stdout.lines().all(|l| !l.trim().starts_with("get")),
    "notes --help must not advertise a get subcommand:\n{stdout}"
);
```

## Info

### IN-01: `--body @` (bare `@`) yields a confusing empty-path error

**File:** `src/commands/notes/body.rs:48-57`
**Issue:** `"@".strip_prefix('@')` returns `Some("")`, so `read_to_string("")` fails and the user sees `Failed to read file '': …`. Correctly rejected exit 2 pre-HTTP, but the message doesn't explain the problem.
**Fix:** Add an explicit arm for the empty path: `Some("") => Err(InvalidInput { detail: "`--body @` has no file path".into(), hint: "Use `--body @-` to read stdin or `--body @<file>`.".into() })`.

### IN-02: Mutation output diverges from the entity convention

**File:** `src/commands/notes/add.rs:56-72`, `src/commands/notes/edit.rs:57-68`
**Issue:** Every other entity's create/update (`deals/create.rs:120`, `orgs/create.rs:100`, …) calls `output::render_single` unconditionally, so table/csv/plain users see the full created/updated record. Notes render entity data only for `--format json` and print a one-liner otherwise — a table-mode `notes add` never shows the stored content. This is deliberate (pinned by `notes_stub_test.rs` and the after_help), but it is a UX inconsistency across entities worth a conscious decision.
**Fix:** Either render the note via `render_single` in non-JSON modes (matching other entities) or document the divergence in `docs/architecture.md`.

### IN-03: New clippy warnings introduced in new code

**File:** `src/commands/notes/body.rs:68-69`, `src/commands/notes/list.rs:70-71`
**Issue:** Two `clippy::collapsible_if` warnings fire on phase-10 code (`if !no_input && is_terminal() { if let Some(...) }` and `if truncate { if let Object(map) }`). The repo carries pre-existing clippy noise, but new code shouldn't add to it.
**Fix:** Collapse the conditions, e.g. `if !no_input && std::io::stdin().is_terminal() { if let Some((action, label)) = prompt_context { ... } }` → `if let (false, true, Some((action, label))) = (no_input, std::io::stdin().is_terminal(), prompt_context)` or nest-free `&&` chain with an early structure.

### IN-04: Path IDs interpolated into URLs without percent-encoding

**File:** `src/api/mod.rs:1013-1016` (list_notes), `1040-1043` (create_note), `1072-1073` (update_note), `1087-1088` (delete_note)
**Issue:** `parent_id`/`note_id` are formatted directly into URL paths; an ID containing `/`, `?`, or `#` silently produces a different request (e.g., `notes list deals "../x/notes"` normalizes elsewhere, `d1?limit=999` injects query params). This mirrors the pre-existing pattern used by every other entity, so phase 10 is consistent — but it now exists at 4 more call sites. Security impact is minimal (the user attacks only their own authenticated session); correctness impact is a wrong request with a confusing server error.
**Fix:** In a future hardening pass, encode path segments codebase-wide, e.g. build URLs with `reqwest::Url::path_segments_mut().push(id)` or `urlencoding::encode`.

---

_Reviewed: 2026-09-03_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: standard_
