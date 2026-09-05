# Phase 11: Webhooks, Trash & Audit - Context

**Gathered:** 2026-09-03
**Status:** Ready for planning

<domain>
## Phase Boundary

Users can manage automation integrations and recover from mistakes — with admin-gated and irreversible operations failing safe. Covers WHOK-01..03 (webhooks CRUD + show-once secret), TRSH-01..03 (trash list/restore/purge), AUDT-01..02 (audit viewer with four filters). Depends on Phase 7 (confirm_destructive), Phase 8 (Forbidden hints, RFC 7807 parsing). Internally parallelizable surfaces: webhooks ∥ trash ∥ audit.

</domain>

<decisions>
## Implementation Decisions

### Webhooks CRUD & Secret Handling
- **Unknown event names rejected client-side** (exit 2 pre-HTTP) with an actionable error listing all 13 valid events (research pins the exact list from server source).
- **Secret shown exactly once on create**: "Signing secret (save it now — shown only once):" + full 64-char secret on its own line. List/get output shows `(shown once at creation)`. The secret NEVER enters the cache or any other output. The 64-char truncation test is written FIRST (ROADMAP note).
- **Create input**: `--url`, `--events <e1,e2,...>` (comma-separated), `--description`; `--stdin` JSON for full control.
- **Update = get→merge→PUT**: update flags merge into the fetched current webhook; omitted keys remain unchanged (no-op) — protects against full-object replacement wiping fields. PUT sends the merged full object.

### Trash & Recovery
- **Type normalization**: accept singular and plural forms (deal/deals, organization/orgs/organizations, person/people, activity/activities) → normalize to the server's type token; enables `trash list | jq | trash restore` round-trips.
- **`linked_parents` rendering**: truncated table cell; full array via `--json` (planner pins exact shape from research).
- **Restore grammar**: `trash restore <type> <id>` (positional); no confirmation (restore IS the recovery act).
- **Purge grammar**: `trash purge [--type <t>]` (all trashed, or per type). Strongest confirmation in the codebase: prompt names the scope + says "permanently destroys"; non-TTY without `--force` → **exit 2** with zero HTTP (deliberately stricter than the standard delete's exit 1, per criterion 4).

### Audit Log
- **Four passthrough filters**: `--entity-type`, `--entity-id`, `--actor-kind`, `--workflow-run-id` (server validates values).
- **Pagination**: `--limit/--offset` passthrough; `--all` only if research shows a hard cap (planner pins from server source).
- **Default columns**: timestamp, actor (kind/id), action, entity (type/id); changes payload visible via `--json` only (AUDT-03 diffs deferred).
- **Admin gating**: Forbidden + existing per-surface hint "audit log requires an admin key" (registered in Phase 8) — verified in this phase.

### Structure, Cache & Confirmation Hierarchy
- **3 plans**: webhooks / trash / audit — sequential waves (shared wiring files).
- **Cache**: webhooks get `KEY_WEBHOOKS` with invalidation on mutations; trash and audit are never cached.
- **Confirmation hierarchy**: purge (strongest wording + exit-2 no-input refusal) > webhook delete (standard confirm + `--force`, non-TTY refusal exit 1) > restore (none).
- **Test emphasis**: secret-truncation test FIRST; per-plan stub-server suites; purge zero-HTTP refusal test pinned; Forbidden hints probed per surface.

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- confirm_destructive + run_batch_delete contract (src/batch.rs) — webhook delete; purge gets a stricter variant
- Hidden-variant parse-then-error (templates get/notes get precedents)
- forbidden_hint table with audit/trash-purge/notes/webhooks keys already registered (Phase 8) — zero registration work expected
- Head+body capturing stub helpers; hermetic cmd() env
- ApiListResponse<T> + table configs; get→merge→PUT precedent exists in entity update flows

### Established Patterns
- Top-level command groups (templates, notes) — webhooks/trash/audit follow
- Exit codes: 0 ok / 1 item-failure or user-refusal / 2 structural (InvalidInput/MissingInput)
- after_help examples; hints everywhere; server-contract facts verified against server source with citations

### Integration Points
- cli/mod.rs + commands/mod.rs + main.rs wiring ×3 groups
- cache.rs: KEY_WEBHOOKS addition; invalidation on webhook mutations
- CHANGELOG.md: no breaking changes expected this phase (all new surface)

</code_context>

<specifics>
## Specific Ideas

- ROADMAP note: "Write the 64-char-secret truncation test FIRST"
- Research flags to resolve during planning: webhook PUT semantics (resolved above: get→merge→PUT), trash linked_parents table rendering (resolved above: truncated cell + full json)
- Criterion 4's exit-2 no-input purge refusal is deliberately stricter than the Phase 7/9 delete exit-1 refusal — keep both contracts distinct

</specifics>

<deferred>
## Deferred Ideas

- AUDT-03 (changes rendered as field: from → to diffs) — future milestone per REQUIREMENTS.md
- WHOK-04 (event-name shell completions) — future milestone
- Webhook delivery log viewer / test-ping endpoint — not in requirements

</deferred>
