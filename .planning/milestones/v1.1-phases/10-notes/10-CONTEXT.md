# Phase 10: Notes - Context

**Gathered:** 2026-09-03
**Status:** Ready for planning

<domain>
## Phase Boundary

Users can attach, revise, and remove notes on deals, organizations, people, and activities — the first parent-scoped sub-resource. Covers NOTE-01..04: list, add (flag/@file/stdin/prompt), edit by ID, delete by ID with confirmation — plus rejection surfaces for non-capable parents and a `notes get` attempt. Depends on Phase 8 (author-or-admin 403 → Forbidden hint).

</domain>

<decisions>
## Implementation Decisions

### Command Shape & Input Sources
- **Grammar**: `pipelite notes <list|add|edit|delete> <entity-type> <parent-id> ...` — top-level command group (STATE.md decision), entity-type ∈ {deals, orgs, people, activities} (plural CLI names).
- **Body input precedence** (add/edit): `--body` flag > `@file` path > `--stdin` > interactive prompt (TTY only; `--no-input` → MissingInput exit 2). Multiple explicit sources → pre-HTTP InvalidInput exit 2.
- **`@file` semantics**: `--body @path/to.md` reads the file; missing/unreadable → InvalidInput exit 2 + hint; `@-` reads stdin (equivalent to `--stdin`).
- **No client-side body length cap** — server validates; 422 errors[] renders via Phase 8 layer.

### List & Edit UX
- **List default columns**: id, created_at, body truncated (~80 chars); full text via `--json`/plain; `--json` documented as the view-before-edit path.
- **List ordering/pagination**: server order + `--limit/--offset` passthrough; no `--all` unless research shows a 1000-cap like other lists.
- **Edit flow**: `notes edit <type> <parent-id> <note-id> --body ...`; prompt text explains "the server offers no single-note GET — run `notes list <type> <id> --json` to view existing content first" (criterion 3).
- **Edit confirmation**: none (non-destructive, idempotent); `--dry-run` previews the PUT.

### Delete & Rejection Surfaces
- **Delete**: confirm_destructive flow (dry-run preview → TTY prompt → `--force` bypass; non-TTY without `--force` → exit 1 Validation) — exact Phase 7/9 delete contract, run_batch_delete-style single delete is fine (delete is by note ID, one at a time).
- **Non-capable parents** (`notes list pipelines <id>`): pre-HTTP InvalidInput exit 2 — "notes are only available on deals, orgs, people, activities" + valid types listed; zero HTTP.
- **`notes get` attempt**: hidden subcommand → exit 2 with hint "the server has no single-note GET — use `notes list <type> <id> --json`" (Phase 8/9 hidden-variant pattern).
- **403 on foreign notes**: Forbidden via Phase 8 per-surface hint — "deleting a note requires its author or an admin" (register a notes surface key in forbidden_hint).

### Models, Cache & Structure
- **Plan structure**: 2 plans — (1) notes surface end-to-end; (2) tests/wiring if needed; planner decides with real file counts (single plan acceptable if small).
- **No cache for notes** — append-heavy parent-scoped resource, no invalidation keys; direct reads.
- **Note model**: follow the server serializer exactly (research verifies fields); `#[serde(flatten)] expanded` only if notes support --expand.
- **Docs**: update `docs/api-reference.md` + after_help examples in the same phase (not deferred to Phase 13).

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- Phase 8 error layer: parse_rfc7807, forbidden_hint surface table (add notes key), InvalidInput exit 2
- Phase 7/9 delete contract: confirm prompt, --force, non-TTY refusal exit 1 (src/commands/workflows/delete.rs, src/batch.rs)
- Hidden-variant parse-then-error pattern (templates update from 09-02)
- Head-capturing stub server helpers (tests/common/mod.rs) — head + body capture for stdin/body tests
- prompt::require_text/check_missing for interactive body input; `#[serde(flatten)] expanded` pattern

### Established Patterns
- Top-level command groups: templates (09-02) is the closest analog — cli/<group>.rs + commands/<group>/
- after_help examples on every subcommand; hints on every error; exit 2 = InvalidInput
- TTY-hermetic tests (--no-input / write_stdin); server-contract facts verified against server source

### Integration Points
- forbidden_hint table in src/api/mod.rs — new "notes" surface key
- cli/mod.rs + commands/mod.rs + main.rs wiring
- docs/api-reference.md gains a Notes section

</code_context>

<specifics>
## Specific Ideas

- STATE.md: "Notes is a top-level command group with entity-type positional, not nested ×4 under entities" — locked
- Criterion 3's "prompt text explains why existing content can't be shown" → edit flow wording above

</specifics>

<deferred>
## Deferred Ideas

- Note search/filtering by author or date — not in requirements
- Bulk note operations — not in requirements

</deferred>
