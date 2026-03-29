---
phase: 02-core-crud-and-output
plan: 01
subsystem: output
tags: [serde, comfy-table, csv, chrono, chrono-humanize, json, table, field-selection]

requires:
  - phase: 01-foundation
    provides: OutputFormat enum, detect_format, AppContext, CliError, PipeliteClient

provides:
  - Deal, DealCreate, DealUpdate data model structs with serde attributes
  - ApiListResponse<T> and PaginationMeta for paginated API responses
  - Generic output rendering layer (table, JSON, CSV, plain) taking serde_json::Value
  - Field selection with dot-notation across all output formats
  - Value formatting (relative dates, currency with commas, truncation)
  - Validation and Api error variants in CliError
  - TableConfig and deals_table_config() for default column configuration

affects: [02-02-deal-crud-commands, 03-full-entities]

tech-stack:
  added: [csv 1.3, chrono 0.4, chrono-humanize 0.2]
  patterns: [generic-output-via-serde-json-value, dot-notation-field-extraction, format-value-heuristics]

key-files:
  created:
    - src/output/fields.rs
    - src/output/format.rs
    - src/output/json.rs
    - src/output/csv.rs
    - src/output/plain.rs
  modified:
    - Cargo.toml
    - src/api/models.rs
    - src/error.rs
    - src/output/mod.rs
    - src/output/table.rs

key-decisions:
  - "Output modules are fully generic -- take serde_json::Value, not Deal-specific types"
  - "comfy-table ColumnConstraint::UpperBoundary(Width::Fixed(40)) for title column max width"
  - "format_value uses field name heuristics (_at/_date for relative time, 'value' for currency)"
  - "Internal format_* helpers return String for testability; public render_* writes to stdout"

patterns-established:
  - "Output testability: internal format_* functions return strings, public render_* prints to stdout"
  - "Field extraction via dot-notation paths on serde_json::Value for all output formats"
  - "Currency formatting with comma separators, no currency symbol"
  - "Relative date display via chrono-humanize in table output, raw ISO in JSON"

requirements-completed: [OUTP-01, OUTP-02, OUTP-03, OUTP-04, OUTP-05, OUTP-06, OUTP-07]

duration: 5min
completed: 2026-03-24
---

# Phase 2 Plan 1: Output Layer and Deal Models Summary

**Generic output rendering (table/JSON/CSV/plain) with dot-notation field selection, relative dates, currency formatting, and Deal data models with serde skip_serializing_if**

## Performance

- **Duration:** 5 min
- **Started:** 2026-03-25T01:22:13Z
- **Completed:** 2026-03-25T01:27:08Z
- **Tasks:** 2
- **Files modified:** 12

## Accomplishments
- Deal/DealCreate/DealUpdate structs with skip_serializing_if on optional fields
- ApiListResponse<T> and PaginationMeta for paginated API responses
- All 4 output formats (JSON, table, CSV, plain) rendering from generic serde_json::Value
- Dot-notation field selection and filtering across all formats
- Value formatting: relative dates via chrono-humanize, currency with comma separators, string truncation with ellipsis
- Table rendering with comfy-table: colored dimmed headers, right-aligned value column, pagination footer
- Vertical key-value single-item display (gh issue view style)
- Validation and Api error variants added to CliError

## Task Commits

Each task was committed atomically:

1. **Task 1: Add dependencies and create Deal data models** - `b8eed8b` (feat)
2. **Task 2: Build generic output rendering layer** - `3f442d6` (feat)

## Files Created/Modified
- `Cargo.toml` - Added csv, chrono, chrono-humanize dependencies
- `src/api/models.rs` - Deal, DealCreate, DealUpdate, ApiListResponse, PaginationMeta, TableConfig
- `src/error.rs` - Added Validation and Api error variants with display_error handling
- `src/output/mod.rs` - Dispatch render_list/render_single across all 4 formats
- `src/output/table.rs` - comfy-table rendering with pagination footer and key-value single display
- `src/output/json.rs` - Pretty-printed JSON output with field filtering
- `src/output/csv.rs` - CSV output with header row and proper escaping
- `src/output/plain.rs` - Tab-separated output with no headers
- `src/output/fields.rs` - Dot-notation field extraction and filtering
- `src/output/format.rs` - Relative date formatting, currency formatting, truncation

## Decisions Made
- Output modules are fully generic (serde_json::Value) -- no Deal-specific types, enabling reuse for all 5 remaining entities
- Used ColumnConstraint::UpperBoundary(Width::Fixed(40)) for title column (comfy-table API requires wrapping Width in ColumnConstraint)
- format_value uses field name heuristics: _at/_date suffix for relative time, "value" for currency
- Internal format_* helpers return String for testability; public render_* writes to stdout

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] comfy-table Width::UpperBoundary does not exist**
- **Found during:** Task 2 (table.rs compilation)
- **Issue:** Plan specified `Width::UpperBoundary(40)` but comfy-table 7.2 uses `ColumnConstraint::UpperBoundary(Width::Fixed(40))`
- **Fix:** Wrapped Width::Fixed(40) in ColumnConstraint::UpperBoundary as the API requires
- **Files modified:** src/output/table.rs
- **Verification:** cargo build succeeds, table tests pass
- **Committed in:** 3f442d6

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Minor API correction. No scope creep.

## Issues Encountered
None beyond the comfy-table API deviation above.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Output infrastructure complete and generic -- ready for deal CRUD commands in plan 02-02
- All 4 output formats tested with 43 output tests + 4 model tests
- Patterns proven here (generic Value-based rendering, field selection) will be reused for all entities

---
*Phase: 02-core-crud-and-output*
*Completed: 2026-03-24*
