# Phase 6: Update the CLI tools to include the new Workflow API - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-29
**Phase:** 06-update-the-cli-tools-to-include-the-new-workflow-api
**Areas discussed:** Workflow entity model, Command structure, Dashboard integration, Workflow execution

---

## Workflow Entity Model

| Option | Description | Selected |
|--------|-------------|----------|
| Automation rules | If-this-then-that automation rules with triggers + actions | |
| Multi-step processes | Defined sequences of steps attached to deals/pipelines | |
| State machines | Allowed transitions between states | |
| Read API docs | Read the API doc directly to derive the model | ✓ |

**User's choice:** Read the API documentation directly to conclude the best entity model
**Notes:** User directed to read `../pipelite/docs/user/reference/workflows.md` and `../pipelite/public/openapi.yaml`. API reveals workflows are server-side automation rules with triggers (crm_event, schedule, webhook, manual) and nodes (action, condition, delay) forming an execution graph.

---

## Command Structure

### Run command verb

| Option | Description | Selected |
|--------|-------------|----------|
| pipelite workflows run <id> | Simple 'run' subcommand | |
| pipelite workflows trigger <id> | 'trigger' verb matching API terminology | ✓ |
| You decide | Claude picks | |

**User's choice:** `pipelite workflows trigger <id>`

### Alias

| Option | Description | Selected |
|--------|-------------|----------|
| wf | Short and unique two-letter alias | |
| w | Single letter like other entities | ✓ |
| No alias | Full name only | |

**User's choice:** `w` (single letter)

### Complex field input

| Option | Description | Selected |
|--------|-------------|----------|
| JSON-only via --stdin/flags | All input via JSON | |
| Interactive for basics + JSON | Prompts for name/desc/active, JSON for triggers/nodes | ✓ |
| Full interactive wizard | Step-by-step guided prompts for everything | |

**User's choice:** Interactive for basics + JSON for complex fields

---

## Dashboard Integration

| Option | Description | Selected |
|--------|-------------|----------|
| No change to dashboard | Workflows separate from dashboard | |
| Add workflow summary | Add active count + recent runs to existing dashboard | ✓ |
| Separate workflows dashboard | New `pipelite workflows status` command | |

**User's choice:** Add workflow summary section to existing dashboard

---

## Workflow Execution

### Entity context

| Option | Description | Selected |
|--------|-------------|----------|
| Entity flags | --entity-type and --entity-id flags | |
| Just --data JSON | Single --data flag for arbitrary JSON | ✓ |
| You decide | Claude picks | |

**User's choice:** Just `--data` JSON flag — simpler and more flexible

### Run output

| Option | Description | Selected |
|--------|-------------|----------|
| Confirm only | Show run_id and pending status immediately | ✓ |
| Wait + poll | Optionally wait with --wait flag | |

**User's choice:** Confirm only — fire and forget

---

## Claude's Discretion

- Table columns for list vs get display
- Trigger/node rendering in table output
- Dashboard summary formatting
- Cache key structure
- --from-file flag design
- --data flag format

## Deferred Ideas

None — discussion stayed within phase scope
