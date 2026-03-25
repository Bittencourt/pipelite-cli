---
phase: 2
slug: core-crud-and-output
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-24
---

# Phase 2 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (built-in Rust test framework) |
| **Config file** | Cargo.toml (already configured) |
| **Quick run command** | `cargo test --lib` |
| **Full suite command** | `cargo test` |
| **Estimated runtime** | ~15 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --lib`
- **After every plan wave:** Run `cargo test`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 15 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 02-01-01 | 01 | 1 | OUTP-01, OUTP-02, OUTP-03, OUTP-04 | unit | `cargo test output` | ❌ W0 | ⬜ pending |
| 02-01-02 | 01 | 1 | OUTP-05 | unit | `cargo test output::fields` | ❌ W0 | ⬜ pending |
| 02-01-03 | 01 | 1 | OUTP-06 | unit | `cargo test output::detect` | ✅ | ⬜ pending |
| 02-01-04 | 01 | 1 | OUTP-07 | unit | `cargo test output::color` | ❌ W0 | ⬜ pending |
| 02-02-01 | 02 | 2 | DEAL-01, DEAL-02 | integration | `cargo test deals` | ❌ W0 | ⬜ pending |
| 02-02-02 | 02 | 2 | DEAL-03, DEAL-04 | integration | `cargo test deals::create deals::update` | ❌ W0 | ⬜ pending |
| 02-02-03 | 02 | 2 | DEAL-05 | integration | `cargo test deals::delete` | ❌ W0 | ⬜ pending |
| 02-02-04 | 02 | 2 | FILT-01, FILT-02 | integration | `cargo test deals::filter` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `tests/deals_integration.rs` — integration test stubs for deal CRUD endpoints
- [ ] `src/output/tests.rs` — unit test stubs for output formatting (table, csv, json, plain)
- [ ] Test helpers for mock API responses (serde_json fixtures)

*Existing test infrastructure from Phase 1 covers basic CLI integration tests.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| TTY auto-detection | OUTP-06 | Requires real TTY vs pipe environment | Run `pipelite deals list` in terminal (table) and `pipelite deals list \| cat` (JSON) |
| Colored output display | OUTP-07 | Visual verification of ANSI colors | Run `pipelite deals list` and verify colored headers; run with `NO_COLOR=1` and verify no colors |
| Pagination footer | FILT-02 | Visual verification of "Showing X-Y of Z" | Run `pipelite deals list --limit 2` against server with >2 deals |

*If none: "All phase behaviors have automated verification."*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 15s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
