# Phase 1: Batch Operations for All Entities - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-29
**Phase:** 01-batch-operations-for-all-entities
**Areas discussed:** Input formats, Batch update design, Batch delete design, Error handling

---

## Input Formats

| Option | Description | Selected |
|--------|-------------|----------|
| JSON only | Keep the existing pattern — JSON arrays via --stdin. Consistent, already implemented for create, works well with jq pipelines. | ✓ |
| JSON + CSV | Add --format csv support for stdin input. Useful for spreadsheet imports but adds parsing complexity and type ambiguity. | |
| JSON + NDJSON | Add newline-delimited JSON support. Good for streaming large datasets, one object per line. Common in data pipelines. | |

**User's choice:** JSON only (Recommended)
**Notes:** Keeps consistency with existing --stdin pattern across all entities.

---

## Batch Update Design

### Target identification

| Option | Description | Selected |
|--------|-------------|----------|
| ID in each object | Each JSON object includes an "id" field plus the fields to patch. Simple, self-contained, mirrors single-update pattern. | ✓ |
| Separate ID list + patch | Provide IDs via --ids flag and a single patch body via stdin. All entities get the same update. Good for bulk status changes. | |
| Both modes | Support both: --stdin for per-entity patches and --ids + --stdin for uniform bulk updates. More flexible but more complex. | |

**User's choice:** ID in each object (Recommended)
**Notes:** None

### Subcommand design

| Option | Description | Selected |
|--------|-------------|----------|
| --stdin on update | Same pattern as create. Consistent, no new subcommands. --stdin switches from single to batch mode. | ✓ |
| New batch subcommand | Explicit separation with `batch-update` subcommands. Adds 7 new subcommands, breaks CRUD symmetry. | |

**User's choice:** --stdin on update (Recommended)
**Notes:** None

---

## Batch Delete Design

### ID input method

| Option | Description | Selected |
|--------|-------------|----------|
| Multiple args | Pass multiple IDs as positional args. Also support --stdin for large lists. | ✓ |
| --stdin only | JSON array of IDs from stdin. Consistent with create/update but less ergonomic for small batches. | |
| Both args and --stdin | Multiple positional args for small batches + --stdin for large lists. Most flexible. | |

**User's choice:** Multiple args (Recommended)
**Notes:** None

### Confirmation behavior

| Option | Description | Selected |
|--------|-------------|----------|
| Confirm with count | Show "Delete N items? [y/N]" prompt. Respects --no-input and --dry-run. | ✓ |
| No confirmation | Delete immediately like single delete. Rely on --dry-run for safety. | |
| Confirm above threshold | Only prompt when deleting more than N items. Small batches proceed silently. | |

**User's choice:** Confirm with count (Recommended)
**Notes:** None

---

## Error Handling

### Failure strategy

| Option | Description | Selected |
|--------|-------------|----------|
| Continue + summary | Process all items, collect errors, report summary. Best for large batches with partial success. | ✓ |
| Fail fast | Stop on first error, report what succeeded so far. Safer for all-or-nothing operations. | |
| Flag-controlled | Default continue-on-error, add --fail-fast flag. Most flexible but adds CLI surface. | |

**User's choice:** Continue + summary (Recommended)
**Notes:** None

### Error output

| Option | Description | Selected |
|--------|-------------|----------|
| Inline + exit code | Successful items to stdout, error summary to stderr. Exit code 0 if all succeed, non-zero on any failure. | ✓ |
| Structured report | JSON/table report with per-item status. More parseable but changes output shape. | |
| You decide | Claude picks based on existing output patterns. | |

**User's choice:** Inline + exit code (Recommended)
**Notes:** None

---

## Claude's Discretion

- Specific exit code value for partial failure
- Whether to retrofit continue-on-error to existing batch create operations
- Internal implementation patterns (shared batch utilities, trait abstractions)

## Deferred Ideas

None — discussion stayed within phase scope
