---
phase: 10
slug: notes
status: draft
nyquist_compliant: true
wave_0_complete: false
created: 2026-09-03
---

# Phase 10 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | `cargo test` + `assert_cmd` CLI integration tests (existing) + `#[cfg(test)]` unit modules in src |
| **Config file** | none — tests are hermetic via env (PIPELITE_URL/PIPELITE_SERVER_URL/PIPELITE_API_KEY) |
| **Quick run command** | `cargo test --quiet --test notes_stub_test` |
| **Full suite command** | `cargo test` |
| **Estimated runtime** | ~90 s full suite; ~25 s targeted stub file |

---

## Sampling Rate

- **After every task commit:** Run the targeted test file(s) for that task (see map below)
- **After the plan wave:** Run `cargo test` (full suite)
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 90 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 10-01-T1 | 01 | 1 | NOTE-01, SC-5 | T-10-01, T-10-05 | singular entity_type parse; table newline flattening + ~80 truncation; json full text; query params on the wire; empty hint quiet-suppressed; parent 404 → NotFound exit 1 | unit (models) + integration (stub) | `cargo test --bin pipelite && cargo test --test notes_stub_test` | ❌ in-task (Wave-0 gap: tests/notes_stub_test.rs created by this task) | ⬜ pending |
| 10-01-T1 | 01 | 1 | SC-5 | T-10-05 | `notes list pipelines` → exit 2 locked message, zero HTTP; hidden `notes get` → exit 2, zero HTTP, absent from help | integration (stub + unreachable cmd) | `cargo test --test notes_stub_test` | ❌ in-task | ⬜ pending |
| 10-01-T2 | 01 | 1 | NOTE-02 | T-10-02 | POST body exactly {"content": …}; @file read; unreadable @file exit 2 pre-HTTP; @-/stdin; two-source XOR (incl. @- + --stdin) exit 2 before ANY read; --no-input MissingInput exit 2; dry-run zero-HTTP | integration (stub, body capture + write_stdin) | `cargo test --test notes_stub_test` | ✅ (extended in-task) | ⬜ pending |
| 10-01-T2 | 01 | 1 | NOTE-03 | T-10-04 | PATCH /api/v1/notes/{id} only (parent args unused); 403 renders registered notes hint; whitespace-only 422 errors[] passthrough (no client trim) | integration (stub) | `cargo test --test notes_stub_test` | ✅ (extended) | ⬜ pending |
| 10-01-T2 | 01 | 1 | NOTE-04 | T-10-03 | dry-run first (never prompts); TTY confirm / --force bypass; non-TTY refusal exit 1 zero-HTTP; sequential re-delete → 404 NotFound | integration (stub) | `cargo test --test notes_stub_test` | ✅ (extended) | ⬜ pending |
| 10-01-T3 | 01 | 1 | NOTE-03 (criterion 3) | T-10-05 | help truthfulness: no-single-GET stated, no --all, hidden get unadvertised, examples on every subcommand; api-reference Notes section | help integration + grep gates | `cargo test` + grep assertions in plan | ✅ (extended: tests/help_examples_test.rs) + docs grep | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

**Wave-0 note:** `tests/notes_stub_test.rs` does not exist yet (research Wave 0 gap). Task 1 creates it with the list/rejection coverage; Task 2 extends it to ≥25 tests — no `<automated>MISSING` references remain, every verify gate is executable at its task's completion. All helpers already exist in `tests/common/mod.rs` (cmd, cmd_with_server, spawn_head_capturing_stub_server with head+body capture) — no helper changes needed this phase.

---

## Wave 0 Requirements

- [ ] `tests/notes_stub_test.rs` — NOTE-01..04 + SC-5 coverage — created by 10-01-T1, extended by 10-01-T2
- [ ] No framework install needed — assert_cmd/predicates/tempfile already in dev-dependencies (RESEARCH § Package Legitimacy Audit: no new crates)

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Interactive TTY prompt path (add/edit body prompt, delete confirm) | NOTE-02, NOTE-04 | The harness runs non-TTY; the prompt branches are covered by their guard tests (--no-input MissingInput; non-TTY delete refusal) but dialoguer rendering itself needs a human terminal | `pipelite notes add deals <id>` on a real TTY → observe the "Note content" prompt; `pipelite notes delete deals <id> <note-id>` → observe "Delete note …? [y/N]" and the Aborted path |
| Live end-to-end against the real server (incl. foreign-note 403) | NOTE-01..04 | Requires a running Pipelite server with two API keys (member + admin) and seeded notes | `docker-compose up` in /home/pedro/programming/pipelite; `pipelite notes add deals <id> --body "hi"`; `pipelite notes list deals <id> --format json`; edit/delete a note authored by another user with a member key → expect the Forbidden hint |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or in-task Wave-0 dependencies (no MISSING references)
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references (created within the implementing plan)
