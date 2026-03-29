---
status: complete
phase: 06-update-the-cli-tools-to-include-the-new-workflow-api
source: [06-01-SUMMARY.md, 06-02-SUMMARY.md]
started: 2026-03-29T21:10:00Z
updated: 2026-03-29T21:20:00Z
---

## Current Test

[testing complete]

## Tests

### 1. Workflows Help Shows All Subcommands
expected: Running `pipelite workflows --help` shows all 6 subcommands: list, get, create, update, delete, trigger. The alias `w` also works (`pipelite w --help`).
result: pass

### 2. Workflows List
expected: Running `pipelite workflows list` connects to the API and returns workflows. `--active` filters to active-only. `--format json` outputs JSON. `--fields id,name` limits columns.
result: blocked
blocked_by: server
reason: "API returns 401 -- no valid API key configured for testing"

### 3. Workflows Get
expected: Running `pipelite workflows get <id>` returns a single workflow with full details including triggers and nodes arrays.
result: blocked
blocked_by: server
reason: "API returns 401 -- no valid API key configured for testing"

### 4. Workflows Create (Dry Run)
expected: Running `pipelite workflows create --name "Test Workflow" --dry-run` shows the POST request that would be sent to `/api/v1/workflows` without actually creating anything. Shows the JSON body with name field.
result: pass

### 5. Workflows Create Interactive Prompts
expected: Running `pipelite workflows create` in a TTY without flags prompts for name (required), description (optional), and active toggle (yes/no via dialoguer). `--triggers` and `--nodes` accept JSON strings for complex fields.
result: blocked
blocked_by: server
reason: "Interactive prompts require TTY + live API for full flow verification"

### 6. Workflows Update (Dry Run)
expected: Running `pipelite workflows update <id> --name "Updated" --dry-run` shows the PUT request that would be sent without executing. Fields not provided are omitted from the payload.
result: pass

### 7. Workflows Delete with Confirmation
expected: Running `pipelite workflows delete <id>` in a TTY shows a confirmation prompt. Answering no cancels. `--force` skips confirmation. In headless mode (`--no-input`) without `--force`, it errors with a clear message.
result: pass

### 8. Workflows Trigger (Fire-and-Forget)
expected: Running `pipelite workflows trigger <id>` sends POST to `/workflows/{id}/run` and immediately displays the run_id and "pending" status. No polling or waiting. `--data '{"key":"val"}'` passes JSON payload. `--data @file.json` loads from file.
result: blocked
blocked_by: server
reason: "Trigger live execution requires API access. Dry-run verified: POST /api/v1/workflows/wf_123/run with correct URL and empty body"

### 9. Dashboard Shows Workflow Summary
expected: Running `pipelite dashboard` shows a "Workflows: N active of M total" line in the output. JSON format wraps data in `{"pipelines": [...], "workflows": {"active": N, "total": M}}`.
result: blocked
blocked_by: server
reason: "Dashboard requires live API for data. Code review confirms render_display prints workflow summary and render_json wraps in object"

### 10. Cache Invalidation on Mutations
expected: After creating/updating/deleting a workflow, the workflow cache is invalidated. Subsequent `workflows list` fetches fresh data from the API.
result: blocked
blocked_by: server
reason: "Cache invalidation requires live API roundtrips to observe. Code review confirms invalidate(KEY_WORKFLOWS) on create/update/delete"

### 11. Shell Completions Include Workflow IDs
expected: Shell completions (if configured) suggest workflow IDs with name hints when completing workflow subcommand arguments.
result: blocked
blocked_by: server
reason: "Shell completions require cache populated from live API. Code confirms workflow_id_candidates() function exists"

### 12. Headless Mode Works
expected: Running `pipelite workflows create --no-input --name "Bot Workflow"` works without prompts. Running `pipelite workflows create --no-input` (missing required --name) exits with error code 2 and mentions --name in the error.
result: pass

## Summary

total: 12
passed: 5
issues: 0
pending: 0
skipped: 0
blocked: 7

## Gaps

[none yet]
