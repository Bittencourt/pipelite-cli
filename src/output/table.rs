/// Table rendering with comfy-table for list and single-item display.
use anyhow::Result;
use colored::Colorize;
use comfy_table::{CellAlignment, ColumnConstraint, ContentArrangement, Table, Width};

use crate::api::models::PaginationMeta;


use super::format::format_value;

/// Render a list of items as a table to stdout.
///
/// Uses comfy-table with dynamic content arrangement. Supports colored headers,
/// right-aligned value columns, fixed-width title columns, and pagination footer.
pub fn render_list(
    items: &[serde_json::Value],
    columns: &[String],
    color: bool,
    meta: Option<&PaginationMeta>,
) -> Result<()> {
    let output = format_list(items, columns, color, meta);
    println!("{output}");
    Ok(())
}

/// Render a single item as a vertical key-value display to stdout.
///
/// Similar to `gh issue view` -- field name on the left, value on the right.
pub fn render_single(
    item: &serde_json::Value,
    columns: &[String],
    color: bool,
) -> Result<()> {
    let output = format_single(item, columns, color);
    println!("{output}");
    Ok(())
}

/// Format a list of items as a table string (for testability).
pub fn format_list(
    items: &[serde_json::Value],
    columns: &[String],
    color: bool,
    meta: Option<&PaginationMeta>,
) -> String {
    let mut table = Table::new();
    table.set_content_arrangement(ContentArrangement::Dynamic);

    // When stdout is not a terminal, use a fixed width
    if !std::io::IsTerminal::is_terminal(&std::io::stdout()) {
        table.set_width(120);
    }

    // Set headers
    let headers: Vec<comfy_table::Cell> = columns
        .iter()
        .enumerate()
        .map(|(i, col)| {
            let header_text = if color {
                col.dimmed().to_string()
            } else {
                col.to_string()
            };
            let mut cell = comfy_table::Cell::new(header_text);

            // Right-align value column
            if col == "value" {
                cell = cell.set_alignment(CellAlignment::Right);
            }

            // Set column constraints
            if col == "title" {
                // Set max width on title column via column constraint
                if let Some(column) = table.column_mut(i) {
                    column.set_constraint(ColumnConstraint::UpperBoundary(Width::Fixed(40)));
                }
            }

            cell
        })
        .collect();
    table.set_header(headers);

    // Set title column constraint (after headers are set so column exists)
    for (i, col) in columns.iter().enumerate() {
        if col == "title" {
            if let Some(column) = table.column_mut(i) {
                column.set_constraint(ColumnConstraint::UpperBoundary(Width::Fixed(40)));
            }
        }
        if col == "value" {
            if let Some(column) = table.column_mut(i) {
                column.set_cell_alignment(CellAlignment::Right);
            }
        }
    }

    // Add data rows
    for item in items {
        let row: Vec<String> = columns
            .iter()
            .map(|col| format_value(col, &item[col.as_str()], color))
            .collect();
        table.add_row(row);
    }

    let mut output = table.to_string();

    // Add pagination footer
    if let Some(m) = meta {
        let count = items.len() as u64;
        let start = m.offset + 1;
        let end = m.offset + count;
        output.push_str(&format!("\nShowing {start}-{end} of {}", m.total));
    }

    output
}

/// Format a single item as a vertical key-value string (for testability).
pub fn format_single(
    item: &serde_json::Value,
    columns: &[String],
    color: bool,
) -> String {
    let max_key_width = columns.iter().map(|c| c.len()).max().unwrap_or(0);
    let mut lines = Vec::new();

    for col in columns {
        let value = format_value(col, &item[col.as_str()], color);
        let label = if color {
            format!("{:>width$}", col.dimmed(), width = max_key_width)
        } else {
            format!("{:>width$}", col, width = max_key_width)
        };
        lines.push(format!("{label}  {value}"));
    }

    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn sample_items() -> Vec<serde_json::Value> {
        vec![
            json!({"id": "deal_001", "title": "First Deal", "value": 10000, "stage_id": "s1"}),
            json!({"id": "deal_002", "title": "Second Deal", "value": 25000, "stage_id": "s2"}),
        ]
    }

    fn default_columns() -> Vec<String> {
        vec![
            "id".to_string(),
            "title".to_string(),
            "value".to_string(),
            "stage_id".to_string(),
        ]
    }

    #[test]
    fn table_has_correct_row_count() {
        let items = sample_items();
        let cols = default_columns();
        let output = format_list(&items, &cols, false, None);
        // Header separator + header + 2 data rows = at least 4 lines
        let lines: Vec<&str> = output.lines().collect();
        // Should have header line, separator lines, and 2 data lines
        assert!(lines.len() >= 4, "Expected at least 4 lines, got: {}", lines.len());
    }

    #[test]
    fn table_contains_values() {
        let items = sample_items();
        let cols = default_columns();
        let output = format_list(&items, &cols, false, None);
        assert!(output.contains("deal_001"));
        assert!(output.contains("First Deal"));
        assert!(output.contains("deal_002"));
    }

    #[test]
    fn table_pagination_footer() {
        let items = sample_items();
        let cols = default_columns();
        let meta = PaginationMeta {
            total: 42,
            offset: 0,
            limit: 50,
        };
        let output = format_list(&items, &cols, false, Some(&meta));
        assert!(output.contains("Showing 1-2 of 42"));
    }

    #[test]
    fn table_pagination_with_offset() {
        let items = sample_items();
        let cols = default_columns();
        let meta = PaginationMeta {
            total: 100,
            offset: 10,
            limit: 50,
        };
        let output = format_list(&items, &cols, false, Some(&meta));
        assert!(output.contains("Showing 11-12 of 100"));
    }

    #[test]
    fn single_renders_key_value_pairs() {
        let item = json!({"id": "deal_001", "title": "Big Deal", "value": 50000});
        let cols = vec![
            "id".to_string(),
            "title".to_string(),
            "value".to_string(),
        ];
        let output = format_single(&item, &cols, false);
        assert!(output.contains("id"));
        assert!(output.contains("deal_001"));
        assert!(output.contains("title"));
        assert!(output.contains("Big Deal"));
        assert!(output.contains("value"));
        assert!(output.contains("50,000"));
    }

    #[test]
    fn single_aligns_labels() {
        let item = json!({"id": "1", "title": "Deal"});
        let cols = vec!["id".to_string(), "title".to_string()];
        let output = format_single(&item, &cols, false);
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines.len(), 2);
        // "title" is 5 chars, "id" should be padded to 5
        assert!(lines[0].starts_with("   id"), "Expected padded id, got: '{}'", lines[0]);
    }
}
