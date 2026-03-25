/// CSV output rendering with header row and proper escaping.
use anyhow::Result;

use super::fields::extract_field;

/// Render a list of items as CSV to stdout.
///
/// Includes a header row followed by one row per item. Values are extracted
/// using the provided column names. The csv crate handles escaping.
pub fn render_list(items: &[serde_json::Value], columns: &[String]) -> Result<()> {
    let output = format_list(items, columns)?;
    print!("{output}");
    Ok(())
}

/// Format a list of items as a CSV string (for testability).
pub fn format_list(items: &[serde_json::Value], columns: &[String]) -> Result<String> {
    let mut wtr = csv::WriterBuilder::new().from_writer(Vec::new());

    // Write header row
    wtr.write_record(columns)?;

    // Write data rows
    for item in items {
        let row: Vec<String> = columns.iter().map(|col| extract_field(item, col)).collect();
        wtr.write_record(&row)?;
    }

    wtr.flush()?;
    let bytes = wtr.into_inner()?;
    Ok(String::from_utf8(bytes)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn csv_has_header_row() {
        let items = vec![json!({"id": "1", "title": "Deal A"})];
        let columns = vec!["id".to_string(), "title".to_string()];
        let result = format_list(&items, &columns).unwrap();
        let lines: Vec<&str> = result.trim().lines().collect();
        assert_eq!(lines[0], "id,title");
    }

    #[test]
    fn csv_values_correctly_extracted() {
        let items = vec![json!({"id": "deal_001", "title": "Big Deal", "value": 50000})];
        let columns = vec![
            "id".to_string(),
            "title".to_string(),
            "value".to_string(),
        ];
        let result = format_list(&items, &columns).unwrap();
        let lines: Vec<&str> = result.trim().lines().collect();
        assert_eq!(lines[1], "deal_001,Big Deal,50000");
    }

    #[test]
    fn csv_escapes_special_characters() {
        let items = vec![json!({"id": "1", "title": "Deal with, comma"})];
        let columns = vec!["id".to_string(), "title".to_string()];
        let result = format_list(&items, &columns).unwrap();
        // csv crate wraps fields with commas in quotes
        assert!(result.contains("\"Deal with, comma\""));
    }

    #[test]
    fn csv_escapes_quotes() {
        let items = vec![json!({"id": "1", "title": "Deal \"quoted\""})];
        let columns = vec!["id".to_string(), "title".to_string()];
        let result = format_list(&items, &columns).unwrap();
        // csv crate escapes embedded quotes by doubling them
        assert!(result.contains("\"Deal \"\"quoted\"\"\""));
    }

    #[test]
    fn csv_missing_field_renders_empty() {
        let items = vec![json!({"id": "1"})];
        let columns = vec!["id".to_string(), "title".to_string()];
        let result = format_list(&items, &columns).unwrap();
        let lines: Vec<&str> = result.trim().lines().collect();
        assert_eq!(lines[1], "1,");
    }
}
