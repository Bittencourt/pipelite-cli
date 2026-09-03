---
phase: 10-notes
fixed_at: 2026-09-03T00:00:00Z
review_path: .planning/phases/10-notes/10-REVIEW.md
iteration: 1
findings_in_scope: 4
fixed: 4
skipped: 0
status: all_fixed
---

# Phase 10: Code Review Fix Report

**Fixed at:** 2026-09-03
**Source review:** .planning/phases/10-notes/10-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 4 (WR-01, WR-02, IN-01, IN-03 — IN-02/IN-04 out of scope per fix directive)
- Fixed: 4
- Skipped: 0

## Fixed Issues

### WR-01: Empty-page hint is factually wrong when paging past the end

**Files modified:** `src/commands/notes/list.rs`, `tests/notes_stub_test.rs`
**Commit:** a2e8b8f
**Applied fix:** The stderr "No notes yet" hint now fires only when `response.data.is_empty() && response.meta.total == 0` — an empty page with `total > 0` (i.e. `--offset` paged past existing notes) renders silently with exit 0. Doc comment updated to state the total==0 gate. Added stub test `list_empty_page_with_total_above_zero_stays_silent` (empty page, `total: 5`, `offset: 10` → stderr contains no "No notes"); the existing `total == 0` hint test and `--quiet` suppression test still pass.

### WR-02: Brittle bare-substring assertion in help-truthfulness tests

**Files modified:** `tests/help_examples_test.rs`, `tests/notes_stub_test.rs`
**Commit:** 6ce3214
**Applied fix:** Both `!stdout.contains("get")` assertions (innocuous future copy like "gets"/"target" would break CI) replaced with the per-line check pattern the templates test uses: `stdout.lines().all(|l| !l.trim().starts_with("get"))`. Both affected tests pass — hidden `get` subcommand remains undetected in `notes --help` lines.

### IN-01: `--body @` (bare `@`) yields a confusing empty-path error

**Files modified:** `src/commands/notes/body.rs`, `tests/notes_stub_test.rs`
**Commit:** bcc22ec
**Applied fix:** Added an explicit `Some("")` arm in `resolve_body` before the generic `@file` read: `InvalidInput` (exit 2) with detail `` `--body @` has no file path `` and hint "@ must be followed by a file path (`--body @note.md`), or use `--body @-` for stdin." — no more `Failed to read file ''`. Doc comment updated. Added pre-HTTP stub test `add_bare_at_rejects_with_clear_hint_pre_http` (exit 2, hint present, no `Failed to read file ''`, zero HTTP).

### IN-03: New clippy warnings introduced in new code

**Files modified:** `src/commands/notes/body.rs`, `src/commands/notes/list.rs`
**Commit:** ffb9242
**Applied fix:** Collapsed both `clippy::collapsible_if` sites using let-chains (edition 2024, rustc 1.94): `if !no_input && is_terminal() && let Some((action, label)) = prompt_context` in `body.rs` and `if truncate && let Value::Object(ref mut map) = value` in `list.rs`. Clippy warning count dropped 37 → 35 (exactly the two phase-10 sites); the repo's other pre-existing clippy noise is untouched and out of scope.

## Skipped Issues

None — all in-scope findings were fixed.

## Verification

- `cargo check`: no new warnings (same 3 pre-existing dead-code warnings before and after)
- `cargo clippy`: 37 → 35 warnings (only the two fixed collapsible_if removed); 0 collapsible_if in `src/commands/notes/`
- Full suite: `cargo test --no-fail-fast` — **404 passed, 0 failed** (149 unit + 255 integration across 28 suites, including 2 new stub tests and 1 adjusted assertion set)

---

_Fixed: 2026-09-03_
_Fixer: the agent (gsd-code-fixer)_
_Iteration: 1_
