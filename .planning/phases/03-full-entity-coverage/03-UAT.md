---
status: complete
phase: 03-full-entity-coverage
source: [03-01-SUMMARY.md, 03-02-SUMMARY.md, 03-03-SUMMARY.md]
started: 2026-03-25T03:15:00Z
updated: 2026-03-25T03:20:00Z
---

## Current Test

[testing complete]

## Tests

### 1. Organizations CRUD
expected: `pipelite orgs list` returns a table with columns: id, name, owner_id, updated_at. `pipelite orgs get <id>` shows full details. `pipelite orgs create --name "Test Org"` creates and displays. `pipelite orgs update <id> --name "Renamed"` updates. `pipelite orgs delete <id>` deletes.
result: pass

### 2. People CRUD with Dual-Name Validation
expected: `pipelite people list` shows table with full_name (computed from first+last), email, organization_id. `pipelite people create --first-name "Jane" --last-name "Doe"` creates a person. Creating without both --first-name and --last-name produces a validation error. `pipelite people get <id>` shows all person fields.
result: pass

### 3. Activities List with Filters
expected: `pipelite activities list` shows all activities. `pipelite activities list --type call` filters by type. `pipelite activities list --deal <id>` filters by deal. `pipelite activities list --done` shows only completed activities (those with completed_at set).
result: pass

### 4. Activities Mark Done / Mark Undone
expected: `pipelite activities update <id> --mark-done` sets completed_at to current UTC timestamp. `pipelite activities update <id> --mark-undone` clears completed_at back to null. Using both flags together is rejected.
result: pass

### 5. Activities Batch Create via Stdin
expected: Piping JSON lines into `pipelite activities create --stdin` creates each activity individually (no batch API). Progress is printed to stderr. Partial failures are reported and exit code is non-zero if any fail.
result: pass

### 6. Pipelines CRUD
expected: `pipelite pipelines list` shows pipelines table. `pipelite pipelines get <id>` shows single pipeline. `pipelite pipelines create --name "Sales"` creates a pipeline. `pipelite pipelines update <id> --name "Renamed"` updates. `pipelite pipelines delete <id>` deletes.
result: pass

### 7. Stages --pipeline Enforcement
expected: `pipelite stages list` WITHOUT --pipeline exits with code 1 and a helpful validation error message (not clap's generic error). `pipelite stages list --pipeline <id>` works correctly and shows stages for that pipeline.
result: pass

### 8. Stages CRUD
expected: `pipelite stages create --name "Prospect" --pipeline <id>` creates a stage. `pipelite stages get <id>` shows stage details including type field. `pipelite stages update <id> --type "won"` updates the type. `pipelite stages delete <id>` deletes.
result: pass

### 9. Entity Aliases
expected: `pipelite o list` works like `pipelite orgs list`. `pipelite p list` works like `pipelite people list`. `pipelite pl list` works like `pipelite pipelines list`. `pipelite s list --pipeline <id>` works like `pipelite stages list --pipeline <id>`.
result: pass

## Summary

total: 9
passed: 9
issues: 0
pending: 0
skipped: 0

## Gaps

[none yet]
