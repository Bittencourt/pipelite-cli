---
phase: 3
slug: full-entity-coverage
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-24
---

# Phase 3 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in `#[test]` + assert_cmd 2.0 for integration |
| **Config file** | Cargo.toml `[dev-dependencies]` |
| **Quick run command** | `cargo test --test <test_file> -- --nocapture` |
| **Full suite command** | `cargo test` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --test <entity>_integration -- --nocapture`
- **After every plan wave:** Run `cargo test`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 03-01-01 | 01 | 1 | ORG-01 | integration | `cargo test --test orgs_integration -- --nocapture` | ❌ W0 | ⬜ pending |
| 03-01-02 | 01 | 1 | ORG-02 | integration | `cargo test --test orgs_integration -- --nocapture` | ❌ W0 | ⬜ pending |
| 03-01-03 | 01 | 1 | ORG-03 | integration | `cargo test --test orgs_integration -- --nocapture` | ❌ W0 | ⬜ pending |
| 03-01-04 | 01 | 1 | ORG-04 | integration | `cargo test --test orgs_integration -- --nocapture` | ❌ W0 | ⬜ pending |
| 03-01-05 | 01 | 1 | ORG-05 | integration | `cargo test --test orgs_integration -- --nocapture` | ❌ W0 | ⬜ pending |
| 03-01-06 | 01 | 1 | PEOP-01 | integration | `cargo test --test people_integration -- --nocapture` | ❌ W0 | ⬜ pending |
| 03-01-07 | 01 | 1 | PEOP-02 | integration | `cargo test --test people_integration -- --nocapture` | ❌ W0 | ⬜ pending |
| 03-01-08 | 01 | 1 | PEOP-03 | integration | `cargo test --test people_integration -- --nocapture` | ❌ W0 | ⬜ pending |
| 03-01-09 | 01 | 1 | PEOP-04 | integration | `cargo test --test people_integration -- --nocapture` | ❌ W0 | ⬜ pending |
| 03-01-10 | 01 | 1 | PEOP-05 | integration | `cargo test --test people_integration -- --nocapture` | ❌ W0 | ⬜ pending |
| 03-02-01 | 02 | 1 | ACTV-01 | integration | `cargo test --test activities_integration -- --nocapture` | ❌ W0 | ⬜ pending |
| 03-02-02 | 02 | 1 | ACTV-02 | integration | `cargo test --test activities_integration -- --nocapture` | ❌ W0 | ⬜ pending |
| 03-02-03 | 02 | 1 | ACTV-03 | integration | `cargo test --test activities_integration -- --nocapture` | ❌ W0 | ⬜ pending |
| 03-02-04 | 02 | 1 | ACTV-04 | integration | `cargo test --test activities_integration -- --nocapture` | ❌ W0 | ⬜ pending |
| 03-02-05 | 02 | 1 | ACTV-05 | integration | `cargo test --test activities_integration -- --nocapture` | ❌ W0 | ⬜ pending |
| 03-02-06 | 02 | 1 | PIPE-01 | integration | `cargo test --test pipelines_integration -- --nocapture` | ❌ W0 | ⬜ pending |
| 03-02-07 | 02 | 1 | PIPE-02 | integration | `cargo test --test pipelines_integration -- --nocapture` | ❌ W0 | ⬜ pending |
| 03-02-08 | 02 | 1 | PIPE-03 | integration | `cargo test --test pipelines_integration -- --nocapture` | ❌ W0 | ⬜ pending |
| 03-02-09 | 02 | 1 | PIPE-04 | integration | `cargo test --test pipelines_integration -- --nocapture` | ❌ W0 | ⬜ pending |
| 03-02-10 | 02 | 1 | PIPE-05 | integration | `cargo test --test pipelines_integration -- --nocapture` | ❌ W0 | ⬜ pending |
| 03-02-11 | 02 | 1 | STAG-01 | integration | `cargo test --test stages_integration -- --nocapture` | ❌ W0 | ⬜ pending |
| 03-02-12 | 02 | 1 | STAG-02 | integration | `cargo test --test stages_integration -- --nocapture` | ❌ W0 | ⬜ pending |
| 03-02-13 | 02 | 1 | STAG-03 | integration | `cargo test --test stages_integration -- --nocapture` | ❌ W0 | ⬜ pending |
| 03-02-14 | 02 | 1 | STAG-04 | integration | `cargo test --test stages_integration -- --nocapture` | ❌ W0 | ⬜ pending |
| 03-02-15 | 02 | 1 | STAG-05 | integration | `cargo test --test stages_integration -- --nocapture` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `tests/orgs_integration.rs` — stubs for ORG-01 through ORG-05
- [ ] `tests/people_integration.rs` — stubs for PEOP-01 through PEOP-05
- [ ] `tests/activities_integration.rs` — stubs for ACTV-01 through ACTV-05
- [ ] `tests/pipelines_integration.rs` — stubs for PIPE-01 through PIPE-05
- [ ] `tests/stages_integration.rs` — stubs for STAG-01 through STAG-05
- [ ] Unit tests in `src/api/models.rs` for Organization, Person, Activity, Pipeline, Stage deserialization

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| TTY delete confirmation prompt | ORG-05, PEOP-05, ACTV-05, PIPE-05, STAG-05 | Requires interactive TTY | Run `pipelite <entity> delete <id>` in terminal, verify prompt appears |
| Table column alignment in terminal | All list commands | Visual layout check | Run each `list` command, verify columns align properly |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
