/// Plain tab-separated output with no headers or borders.
use anyhow::Result;

use super::fields::extract_field;

/// Render a list of items as tab-separated values to stdout.
///
/// No headers, no borders. One line per item.
pub fn render_list(items: &[serde_json::Value], columns: &[String]) -> Result<()> {
    let output = format_list(items, columns);
    print!("{output}");
    Ok(())
}

/// Format a list of items as tab-separated lines (for testability).
pub fn format_list(items: &[serde_json::Value], columns: &[String]) -> String {
    let mut lines = Vec::new();
    for item in items {
        let values: Vec<String> = columns.iter().map(|col| extract_field(item, col)).collect();
        lines.push(values.join("\t"));
    }
    if lines.is_empty() {
        String::new()
    } else {
        format!("{}\n", lines.join("\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn plain_no_headers() {
        let items = vec![json!({"id": "1", "title": "Deal A"})];
        let columns = vec!["id".to_string(), "title".to_string()];
        let result = format_list(&items, &columns);
        let lines: Vec<&str> = result.trim().lines().collect();
        // Only one line (no header)
        assert_eq!(lines.len(), 1);
    }

    #[test]
    fn plain_tab_separated() {
        let items = vec![json!({"id": "1", "title": "Deal A"})];
        let columns = vec!["id".to_string(), "title".to_string()];
        let result = format_list(&items, &columns);
        assert_eq!(result.trim(), "1\tDeal A");
    }

    #[test]
    fn plain_multiple_rows() {
        let items = vec![
            json!({"id": "1", "title": "A"}),
            json!({"id": "2", "title": "B"}),
        ];
        let columns = vec!["id".to_string(), "title".to_string()];
        let result = format_list(&items, &columns);
        let lines: Vec<&str> = result.trim().lines().collect();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], "1\tA");
        assert_eq!(lines[1], "2\tB");
    }

    #[test]
    fn plain_empty_list() {
        let items: Vec<serde_json::Value> = vec![];
        let columns = vec!["id".to_string()];
        let result = format_list(&items, &columns);
        assert_eq!(result, "");
    }
}
