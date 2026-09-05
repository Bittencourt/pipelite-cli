# Phase 13: Integration Hardening & Docs Refresh - Context

**Gathered:** 2026-09-04
**Status:** Ready for planning

<domain>
## Phase Boundary

The milestone holds together — every new surface honors the v1.0 global contract, cross-cutting behaviors are verified end-to-end, and documentation is current. Depends on Phases 7-12 (all prior phases). This is the final feature phase: contract audit + fixes, docs refresh, E2E validation.

</domain>

<decisions>
## Implementation Decisions

### Contract Audit Mechanics
- **Table-driven contract-matrix test file**: one assertion per new command (phases 7-12 surfaces) × global flag (`--dry-run` zero-HTTP, `--quiet`, `--no-input`, `--no-color`), run against stubs; violations found get FIXED in-phase, each with a pinning test.
- **Format coverage**: every new group renders through the existing output layer — stub tests assert csv headers + plain output per group; gaps fixed here.
- **v1.0 baseline**: full suite green + explicit v1.0-surface smoke (CRUD paths of the 7 original entities unchanged).

### Docs Refresh
- **docs/SKILL.md**: add sections per new surface (batch, workflow runs, templates, notes, webhooks, trash, audit, custom-fields, docs) following the existing per-entity structure + exit-code contract and hint conventions.
- **docs/api-reference.md**: gap audit against all 9 new groups (most documented in-phase); add missing endpoints/flags; verify examples against `--help`.
- **README**: update only if it enumerates commands.
- **CHANGELOG**: finalize `## v1.1` — feature list + breaking changes + the custom-fields data-correctness callout, released-ready.

### E2E Verification & Loose Ends
- **Live-server scripted E2E** (deferred from Phase 12): batch exit codes under `--quiet`, show-once secret, trash list→restore round-trip, typed custom-field write verified server-side. Non-admin-key Forbidden → stub-covered (no non-admin key available), noted in the report.
- **2 plans**: (1) contract-matrix tests + fixes; (2) docs refresh + E2E validation report.
- **api/mod.rs size watch**: check current size; split into api/client.rs + api/methods/ ONLY if over ~2,000 lines — else record the decision.
- **deferred-items.md audit**: mark resolved items (env-dependent tests went hermetic in Phase 8), carry still-open ones to the milestone audit.

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- Head+body capturing stub helpers; hermetic cmd() env — the contract matrix rides these
- Output layer (table/csv/json/plain) shared by all groups
- All 9 new groups have in-phase api-reference sections (gaps expected to be small)
- Live-server credentials available for the E2E (user-provided)

### Established Patterns
- v1.0 global contract: --dry-run (zero HTTP), --no-input, --quiet, --no-color, 4 output formats
- Exit-code contract: 0 ok / 1 item-failure or user-refusal / 2 structural
- CHANGELOG v1.1 Breaking Changes section exists (Phase 8)

### Integration Points
- Milestone audit follows this phase — this phase's artifacts feed it
- api/mod.rs size check feeds the split decision (STATE.md watch item)

</code_context>

<specifics>
## Specific Ideas

- ROADMAP criterion 4 names the v1.0 baseline: 98 unit + 13 integration tests — the suite is now ~589; the criterion means "no regressions against v1.0 behavior", not the literal count
- STATE.md watch item: api/mod.rs split at ~2,000 lines
- Phase 12 verifier deferred live E2E to this phase explicitly

</specifics>

<deferred>
## Deferred Ideas

- comfy-table 7→8 upgrade — post-milestone per STATE.md
- Any new feature work — this phase is hardening only

</deferred>
