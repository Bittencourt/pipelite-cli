# Phase 12: Custom Fields — Definitions & Typed Writing - Context

**Gathered:** 2026-09-04
**Status:** Ready for planning

<domain>
## Phase Boundary

Users can define custom fields per entity and write correctly-typed values to them — fixes the data-correctness bug where `--custom-field price=4` stores string `"4"` instead of number `4`. Covers CFLD-01..03: definitions CRUD, typed writing resolved from cached definitions, raw JSON bypass. Depends on Phase 7 (confirm_destructive for definition deletes), Phase 8 (foundations). Internal order: definitions CRUD before the typed parser (definitions are the type source). Touches 8 existing handlers (create+update × deals/orgs/people/activities) — widest blast radius of the milestone, hence last feature phase.

</domain>

<decisions>
## Implementation Decisions

### Definitions CRUD
- **Top-level `custom-fields` group** (list/get/create/update/delete); `--entity-type` filter on list.
- **Create input**: `--entity-type` + `--key` + `--type` (text/number/boolean/date/select/multi_select) + type-specific flags (`--options a,b,c` for select/multi_select, `--description`, `--position`); `--stdin` JSON for the full config shape.
- **Soft-delete marker**: researcher/planner pin the server behavior from source first; if soft-deleted definitions linger, list shows them with a `deleted` column rather than hiding.
- **Delete**: confirm_destructive contract (dry-run → TTY → `--force`; non-TTY refusal exit 1 Validation).

### Typed Writing
- **Definition resolution**: per-entity-type cache (`KEY_CUSTOM_FIELDS_<type>`), fetched on miss; `--dry-run` NEVER fetches — cache-only fallback with a visible stderr note (criterion 4; quiet-suppressible).
- **Inference rules**: number → JSON number (non-numeric input → exit 2 + hint naming the field's type); boolean → strict `true`/`false`; date → ISO string passthrough; select → string (server validates options); multi_select → comma-split to JSON array (`tags=a,b` → `["a","b"]`); text → string.
- **Unknown keys** (not in cached definitions): send as raw string + one stderr warning "not a defined field — sent as string" (quiet-suppressible).
- **`--custom-field-json`**: raw passthrough, verbatim object (criterion 3); BOTH it and `--custom-field` k=v entries → exit 2 pre-HTTP (no silent merge).

### Blast Radius & Structure
- **2 plans**: (1) definitions CRUD group; (2) typed-writing resolver rewiring the 8 existing create/update handlers (isolated for review/verification).
- **Shared resolver**: one module consumed by all 8 handlers (per-entity cache keys parameterized) — no duplication (WR-05 lesson).
- **Wire format unchanged**: `custom_fields` object in create/update bodies; only VALUE types become correct.
- **Docs in-phase**: api-reference Custom Fields section + CHANGELOG behavior note ("custom-field values are now type-correct").

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- batch::parse_custom_fields (Phase 7/9 centralization) — the k=v parser the resolver replaces/extends
- confirm_destructive delete contract (webhooks delete is the freshest analog)
- ApiListResponse + table configs; head+body capturing stub helpers
- Phase 8 f64 lesson: custom_field_definitions.position is numeric(20,10) — model position as f64 FROM BIRTH (research note)
- Cache conventions (KEY_* constants, invalidation on mutation)

### Established Patterns
- Top-level command groups (templates/notes/webhooks/trash/audit precedents)
- Exit codes: 0/1/2; hidden-variant pattern; --stdin raw passthrough (post_workflow_template_raw precedent)
- Server-contract facts verified against server source with citations

### Integration Points
- 8 handlers: src/commands/{deals,orgs,people,activities}/{create,update}.rs
- cache.rs: new KEY_CUSTOM_FIELDS_* constants + invalidation when definitions mutate
- CHANGELOG.md + docs/api-reference.md

</code_context>

<specifics>
## Specific Ideas

- ROADMAP note: "fixes the stores \"4\" not 4 data-correctness bug" — the typed-writing tests must assert the JSON BODY (wire-level), not just absence of quotes
- Research flags to resolve during planning: definition soft-delete marker + multi_select config shape (pin from server source)
- Definition delete is destructive (data using the field remains) — confirm contract applies

</specifics>

<deferred>
## Deferred Ideas

- CFLD-04 (--custom-field-string force-string flag) — future milestone per REQUIREMENTS.md
- Custom field definition reordering UI / bulk operations — not in requirements

</deferred>
