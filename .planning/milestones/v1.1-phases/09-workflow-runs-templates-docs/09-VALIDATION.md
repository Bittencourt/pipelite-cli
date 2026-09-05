---
phase: 9
slug: workflow-runs-templates-docs
status: draft
nyquist_compliant: true
wave_0_complete: false
created: 2026-09-03
---

# Phase 9 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | `cargo test` + `assert_cmd` CLI integration tests (existing) + `#[cfg(test)]` unit modules in src |
| **Config file** | none — tests are hermetic via env (PIPELITE_URL/PIPELITE_SERVER_URL/PIPELITE_API_KEY) |
| **Quick run command** | `cargo test --quiet --test workflow_runs_stub_test --test templates_stub_test --test docs_stub_test` |
| **Full suite command** | `cargo test` |
| **Estimated runtime** | ~90 s full suite; ~30 s targeted stub files |

---

## Sampling Rate

- **After every task commit:** Run the targeted test file(s) for that task (see map below)
- **After every plan wave:** Run `cargo test` (full suite)
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 90 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 09-01-T1 | 01 | 1 | WRUN-01, WRUN-02 | T-09-02 | dry_run param sent only on opt-in; enum fallback never breaks parse | unit (models) | `cargo test --bin pipelite` | ❌ in-task | ⬜ pending |
| 09-01-T2 | 01 | 1 | WRUN-01, WRUN-02 | T-09-01, T-09-03 | hint wording locked; ≤1 probe; required --workflow | build + grep gates | `cargo build` + grep assertions in plan | ✅ (source) | ⬜ pending |
| 09-01-T3 | 01 | 1 | WRUN-01, WRUN-02 | T-09-02 | query params proven on the wire; probe count; quiet suppression | integration (stub) | `cargo test --test workflow_runs_stub_test` | ❌ in-task (Wave-0 gap: tests/common/mod.rs + test file) | ⬜ pending |
| 09-02-T1 | 02 | 2 | WRUN-03 | T-09-06 | terminal set {completed,failed}; waiting polls; exit mapping; SIGINT | integration (stub, scripted) | `cargo test --test workflow_runs_stub_test` | ✅ (extended) | ⬜ pending |
| 09-02-T2 | 02 | 2 | TPL-01 | T-09-04, T-09-05 | triggers[0] mapping + warning; single-source pre-HTTP; confirmation flow | build + grep gates | `cargo build` + grep assertions in plan | ✅ (source) | ⬜ pending |
| 09-02-T3 | 02 | 2 | TPL-01 | T-09-04, T-09-07 | delete zero-HTTP refusal; hidden update exit 2; 422 passthrough | integration (stub) | `cargo test --test templates_stub_test` | ❌ in-task | ⬜ pending |
| 09-03-T1 | 03 | 3 | DOCS-01 | T-09-08, T-09-09 | headerless client (grep: no self.client in get_docs); --save refusal pre-HTTP | build + grep gates | `cargo build` + grep assertions in plan | ✅ (source) | ⬜ pending |
| 09-03-T2 | 03 | 3 | DOCS-01 | T-09-08 | NO authorization header on the wire; error hints; save matrix | integration (stub) | `cargo test --test docs_stub_test` | ❌ in-task | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

**Wave-0 note:** The three stub test files + tests/common/mod.rs do not exist yet (research Wave 0 gaps). Each plan creates its test infrastructure in the same plan that implements the behavior (09-01-T3 creates common/mod.rs + workflow_runs_stub_test.rs; 09-02-T3 creates templates_stub_test.rs; 09-03-T2 creates docs_stub_test.rs), so no `<automated>MISSING` references remain — every verify gate is executable at its task's completion.

---

## Wave 0 Requirements

- [ ] `tests/common/mod.rs` — shared hermetic helpers: cmd_with_server, unreachable cmd(), spawn_head_capturing_stub_server (scripted responses + recorded request heads) — created by 09-01-T3
- [ ] `tests/workflow_runs_stub_test.rs` — WRUN-01/02/03 coverage — created 09-01-T3, extended 09-02-T1
- [ ] `tests/templates_stub_test.rs` — TPL-01 coverage — created 09-02-T3
- [ ] `tests/docs_stub_test.rs` — DOCS-01 coverage — created 09-03-T2
- [ ] No framework install needed — assert_cmd/predicates/tempfile already in dev-dependencies

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Shell-visible exit 130 on Ctrl-C during --watch | WRUN-03 | ExitStatus::signal()==2 is asserted automated (`sigint_terminates_watch`); the literal `130?` shell report needs a human at a terminal | Start a run in the server UI, `pipelite workflows runs get <run> --workflow <wf> --watch`, press Ctrl-C, observe `echo $?` → 130 |
| Live end-to-end watch against a real server run | WRUN-03 | Requires a running Pipelite server + active workflow + triggered run | `docker-compose up` in /home/pedro/programming/pipelite, trigger a workflow, `pipelite workflows runs get <run> --workflow <wf> --watch --exit-status`; observe transition lines and exit code |
| docs --save against the live server (87 KB spec) | DOCS-01 | Stub uses a tiny fixture; live proves real payload handling | `pipelite docs --save /tmp/spec.json` and inspect the file |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or in-task Wave-0 dependencies (no MISSING references)
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references (created within the implementing plans)
- [x] No watch-mode flags
- [x] Feedback latency < 90 s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
