# Phase 8: Foundations — Error Layer, Models & Pagination - Context

**Gathered:** 2026-09-03
**Status:** Ready for planning

<domain>
## Phase Boundary

The CLI tells the truth — errors carry the server's real reason with actionable hints, permission tiers are distinguishable, models match server output, and no list truncates silently. Covers FIX-01 through FIX-06: RFC 7807 error parsing with a per-surface Forbidden variant, 409 inactive-trigger mapping, fractional `position` deserialization, `stages list` all-mode, `--expand` output passthrough, dead-flag removals (breaking, with changelog callout), and `--all` pagination ceiling warning.

</domain>

<decisions>
## Implementation Decisions

### Error Layer Shape
- **Forbidden is a separate CliError variant** from Auth — 401 (bad key) ≠ 403 (insufficient permission); enables per-surface hints and distinct messaging.
- **Central `parse_rfc7807` helper** in `api/mod.rs`, called by `handle_response`/`handle_delete_response` — one place, benefits all surfaces. Parses `detail` and `errors[]` keys.
- **Per-surface Forbidden hints keyed by surface param** on API calls (e.g. audit → "audit log requires an admin key", trash purge → admin, notes → author-or-admin, webhooks → foreign webhook 403).
- **409 inactive-trigger hint**: "Workflow trigger is inactive — activate it before firing (`pipelite workflows update <id> --active true`)".

### Models & Expansion
- **`Option<f64>` / `f64`** for `Deal.position`, `Stage.position`, and other position fields — server emits fractional values (FIX-03 root cause).
- **Typed models gain `Option<serde_json::Value>` expand field** (`skip_serializing_if` none) so output renders `--expand` payloads; request-only models untouched (FIX-02).
- **No client-side validation of `--expand` values** — pass through, server validates (v1.0 behavior).
- **`stages list` without `--pipeline`**: single unfiltered call to the stages endpoint; add `pipeline_id` to default columns so rows are distinguishable (FIX-04).

### Dead-Flag Removal UX (Breaking)
- **Parse-then-error for removed flags** (`people list --org/--owner`, `workflows create --active`, dead `--custom-field` on pipelines/stages): keep flags defined (hidden), command layer rejects with `CliError::InvalidInput` (exit 2) + hint naming the replacement.
- **`workflows list --active`**: client-side filter retained; one stderr warning line per invocation: `warning: --active filters client-side after fetching all records`.
- **CHANGELOG.md "v1.1 Breaking Changes" section** written in this phase — removed flag → what to use instead.
- **`--all` ceiling**: stderr warning `warning: --all stopped at 1000 records (server ceiling); results may be incomplete`; exit stays 0 (FIX-06).

### Phase Structure & Process
- **2 plans**: (1) error layer — RFC 7807 parser, Forbidden variant, 409 hint; (2) models/pagination/dead-flags — position floats, expand passthrough, stages all-mode, flag removals + changelog.
- **Keep `api/mod.rs` single-file this phase** (981 lines; split rule is ~2,000) — revisit at Phase 13.
- **Stub-server tests** (TcpListener pattern from 07-05) for 403 Forbidden hints, 409 hint, 422 detail parsing, fractional-position fixture; unit tests for flag removals.
- **Error layer first** — research marks it a hard gate for Phases 9–12 (403-heavy surfaces). Then models, then flags/pagination.

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/error.rs` (238 lines): CliError enum with detail+hint fields, `exit_code()` with InvalidInput→2 contract from Phase 7
- `src/api/mod.rs` (981 lines): `send_with_retry` (429 Retry-After from 07-05), `handle_response`/`handle_delete_response` status mapping — the central error chokepoints
- `src/api/models.rs` (829 lines): per-entity Base/Create/Update structs with `skip_serializing_if` patterns
- Stub-server TcpListener test pattern with `PIPELITE_SERVER_URL` (tests/batch_error_test.rs from 07-05)
- `output/` renderers keyed by `--format`; `serde_json::Value` already used for workflow triggers/nodes

### Established Patterns
- Every error carries actionable `hint` text
- Position fields currently `Option<i64>`/`i64` (models.rs:40, 327) — the fractional-position break
- `deals list fetch_all` paginates in batches of 100 to a 1000-record ceiling (list.rs:53-65)
- `--expand` is passed to the API but payloads discarded on deserialization

### Integration Points
- Phases 9-12 consume the Forbidden/RFC-7807 layer (audit, purge, notes, webhooks, inactive-trigger 409)
- Phase 11 needs `confirm_destructive`-style flows and Forbidden hints
- CHANGELOG.md (new file) becomes the breaking-changes record for the milestone

</code_context>

<specifics>
## Specific Ideas

- STATE.md notes dead-flag removals are breaking — changelog callout required (captured above)
- STATE.md api/mod.rs size watch item — decision: keep single file this phase, revisit Phase 13
- Success criterion 1 example hint: "audit log requires an admin key" — use as the template style for per-surface hints

</specifics>

<deferred>
## Deferred Ideas

- `api/mod.rs` split into `api/client.rs` + `api/methods/` — only if it crosses ~2,000 lines (check at Phase 13)
- NDJSON stdin support (descoped in Phase 7, D-01) — unchanged
- Client-side `--expand` validation whitelist — rejected this phase, pass-through stands

</deferred>
