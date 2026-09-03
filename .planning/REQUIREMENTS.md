# Requirements: Pipelite CLI — Milestone v1.1 Server v2 Parity

**Defined:** 2026-09-02
**Core Value:** Users can manage their entire Pipelite CRM from the terminal — fast, scriptable, and composable with other tools.

## v1.1 Requirements

Requirements for this milestone. Each maps to roadmap phases.

### Batch Operations

- [x] **BATCH-01**: User can batch-update any entity by piping JSON objects to `--stdin`
- [x] **BATCH-02**: User can batch-delete by passing multiple IDs or piping IDs via `--stdin`
- [x] **BATCH-03**: User can run batch operations with continue-on-error, seeing per-item results and a final `N ok, M failed` summary
- [x] **BATCH-04**: All entity batch operations share one utility with a consistent contract: exit 0 (all ok), 1 (any failed, even with continue-on-error), 2 (structural input failure before any HTTP); summary survives `--quiet`; stdin fully validated before the first HTTP call; 429 Retry-After retried once then classified as failure

### Notes

- [ ] **NOTE-01**: User can list notes on a deal, organization, person, or activity (`notes list <type> <id>`)
- [ ] **NOTE-02**: User can add a note to any of the four note-capable entities via flag, `@file`, stdin, or interactive prompt
- [ ] **NOTE-03**: User can edit a note by ID (note: no single-note GET exists — document `notes list --json` as the view-before-edit path)
- [ ] **NOTE-04**: User can delete a note by ID with confirmation and `--force` bypass

### Workflow Runs

- [ ] **WRUN-01**: User can list runs of a workflow with `--status` filter (pending/running/completed/failed/waiting) and `--dry-run` opt-in for test runs
- [ ] **WRUN-02**: User can view a run's detail including its steps (node, status, input, output, error, timing) flattened to readable rows
- [ ] **WRUN-03**: User can watch a run until completion with `--watch [--exit-status]` via honest polling

### Webhooks

- [ ] **WHOK-01**: User can list, get, create, update (PUT), and delete webhooks
- [ ] **WHOK-02**: Webhook event names are validated client-side against the 13 supported events with an actionable error on mismatch
- [ ] **WHOK-03**: The signing secret is displayed exactly once on create — full, untruncated, on its own line with a save-it-now warning; never cached, never shown in list/get output

### Trash

- [ ] **TRSH-01**: User can list trashed records with `--type` filter (deals/people/organizations/activities)
- [ ] **TRSH-02**: User can restore a trashed record by type and ID
- [ ] **TRSH-03**: User can permanently purge a trashed record — admin-only, strongest confirmation in the codebase, `--force` bypass, refuses under `--no-input` without `--force` (exit 2, zero HTTP), and normalizes singular/plural type names so piped round-trips work

### Custom Fields

- [ ] **CFLD-01**: User can list, get, create, update, and delete custom field definitions (`custom-fields` group, `--entity-type` filter on list)
- [ ] **CFLD-02**: `--custom-field key=value` writes type-correct JSON (number/boolean/date/array/select) resolved from cached definitions instead of always storing strings
- [ ] **CFLD-03**: User can bypass type inference with `--custom-field-json '{"key": ...}'`

### Workflow Templates

- [ ] **TPL-01**: User can list, get, create, and delete workflow templates (no update — server has none)

### Audit Log

- [ ] **AUDT-01**: User can list audit log entries filtered by `--entity-type`, `--entity-id`, `--actor-kind`, `--workflow-run-id`
- [ ] **AUDT-02**: Non-admin keys get a first-class Forbidden hint (not a generic auth error) on audit and other admin-gated surfaces

### Docs

- [ ] **DOCS-01**: User can fetch the server's OpenAPI 3.1 spec (`pipelite docs [--save FILE]`) without authentication

### Fixes and Foundations

- [ ] **FIX-01**: Misleading/ignored filter flags are fixed or removed: `people list --org/--owner` (fix server-side filtering client-side is impossible — remove with hint), `workflows list --active` (client-side filter with warning), `workflows create --active` (remove), dead `--custom-field` flags on pipelines/stages create (remove)
- [ ] **FIX-02**: `--expand` payloads render in output instead of being silently discarded (output-path raw-Value passthrough; typed models stay request-only)
- [ ] **FIX-03**: `Deal.position` (and other position fields) deserialize server-emitted floats (`Option<f64>`)
- [ ] **FIX-04**: `stages list` allows omitting `--pipeline` to list all stages across pipelines
- [x] **FIX-05**: Error layer speaks RFC 7807: parse `detail`/`errors[]` keys; new `Forbidden` error variant with per-surface hints (audit → admin key, purge → admin, notes → author-or-admin, webhooks → foreign 403); 409 (inactive workflow trigger) mapped with actionable hint
- [ ] **FIX-06**: Auto-pagination (`--all`) warns loudly on stderr when it hits a record ceiling instead of silently stopping at 1000

## Future Requirements

Deferred to later milestones. Tracked but not in current roadmap.

### Differentiators (deferred)

- **WRUN-04**: Batch failed-ID pipeable summary for one-line retries
- **AUDT-03**: Audit `changes` rendered as field: from → to diffs
- **WHOK-04**: Webhook event-name shell completions
- **CFLD-04**: `--custom-field-string` force-string flag

### Server-blocked (request upstream)

- **SRV-01**: Global search via API key (search is session-only server-side)
- **SRV-02**: File upload/download via API key (session-only server-side)

## Out of Scope

Explicitly excluded. Documented to prevent scope creep.

| Feature | Reason |
|---------|--------|
| Sort/date-range/custom-field list filters | No server support — request upstream, never emulate client-side |
| Live-streaming workflow run logs | No logs endpoint exists — polling `--watch` is the honest pattern |
| Local caching of webhook secrets | Security anti-pattern; secret is show-once by design |
| comfy-table 7→8 upgrade | Major bump; schedule as post-milestone maintenance |
| Interactive picker truncation fix (>100 records) | Opportunistic — only if touched during other work |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| BATCH-01 | Phase 7 | Complete |
| BATCH-02 | Phase 7 | Complete |
| BATCH-03 | Phase 7 | Complete |
| BATCH-04 | Phase 7 | Complete |
| FIX-01 | Phase 8 | Pending |
| FIX-02 | Phase 8 | Pending |
| FIX-03 | Phase 8 | Pending |
| FIX-04 | Phase 8 | Pending |
| FIX-05 | Phase 8 | Complete |
| FIX-06 | Phase 8 | Pending |
| WRUN-01 | Phase 9 | Pending |
| WRUN-02 | Phase 9 | Pending |
| WRUN-03 | Phase 9 | Pending |
| TPL-01 | Phase 9 | Pending |
| DOCS-01 | Phase 9 | Pending |
| NOTE-01 | Phase 10 | Pending |
| NOTE-02 | Phase 10 | Pending |
| NOTE-03 | Phase 10 | Pending |
| NOTE-04 | Phase 10 | Pending |
| WHOK-01 | Phase 11 | Pending |
| WHOK-02 | Phase 11 | Pending |
| WHOK-03 | Phase 11 | Pending |
| TRSH-01 | Phase 11 | Pending |
| TRSH-02 | Phase 11 | Pending |
| TRSH-03 | Phase 11 | Pending |
| AUDT-01 | Phase 11 | Pending |
| AUDT-02 | Phase 11 | Pending |
| CFLD-01 | Phase 12 | Pending |
| CFLD-02 | Phase 12 | Pending |
| CFLD-03 | Phase 12 | Pending |

Phase 13 (Integration Hardening & Docs) carries no exclusive requirements — it cross-verifies all 30 above end-to-end.

---
*Requirement quality: specific, testable, user-centric, atomic, independent.*
