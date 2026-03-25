/// JSON output rendering.
use anyhow::Result;

use super::fields::filter_fields;

/// Render a list of items as a pretty-printed JSON array to stdout.
///
/// If `fields` is provided, each item is filtered to only include those fields.
pub fn render_list(items: &[serde_json::Value], fields: &Option<Vec<String>>) -> Result<()> {
    let output = format_list(items, fields)?;
    println!("{output}");
    Ok(())
}

/// Render a single item as a pretty-printed JSON object to stdout.
///
/// If `fields` is provided, the item is filtered to only include those fields.
pub fn render_single(item: &serde_json::Value, fields: &Option<Vec<String>>) -> Result<()> {
    let output = format_single(item, fields)?;
    println!("{output}");
    Ok(())
}

/// Format a list of items as a pretty-printed JSON string (for testability).
pub fn format_list(items: &[serde_json::Value], fields: &Option<Vec<String>>) -> Result<String> {
    let output: Vec<serde_json::Value> = match fields {
        Some(f) => items.iter().map(|item| filter_fields(item, f)).collect(),
        None => items.to_vec(),
    };
    Ok(serde_json::to_string_pretty(&output)?)
}

/// Format a single item as a pretty-printed JSON string (for testability).
pub fn format_single(
    item: &serde_json::Value,
    fields: &Option<Vec<String>>,
) -> Result<String> {
    let output = match fields {
        Some(f) => filter_fields(item, f),
        None => item.clone(),
    };
    Ok(serde_json::to_string_pretty(&output)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn format_list_valid_json() {
        let items = vec![
            json!({"id": "1", "title": "Deal A"}),
            json!({"id": "2", "title": "Deal B"}),
        ];
        let result = format_list(&items, &None).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert!(parsed.is_array());
        assert_eq!(parsed.as_array().unwrap().len(), 2);
    }

    #[test]
    fn format_list_with_field_filtering() {
        let items = vec![json!({"id": "1", "title": "Deal A", "notes": "secret"})];
        let fields = Some(vec!["id".to_string(), "title".to_string()]);
        let result = format_list(&items, &fields).unwrap();
        let parsed: Vec<serde_json::Value> = serde_json::from_str(&result).unwrap();
        let obj = parsed[0].as_object().unwrap();
        assert!(obj.contains_key("id"));
        assert!(obj.contains_key("title"));
        assert!(!obj.contains_key("notes"));
    }

    #[test]
    fn format_single_valid_json() {
        let item = json!({"id": "1", "title": "Deal A"});
        let result = format_single(&item, &None).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert!(parsed.is_object());
        assert_eq!(parsed["id"], "1");
    }

    #[test]
    fn format_single_with_field_filtering() {
        let item = json!({"id": "1", "title": "Deal A", "notes": "secret"});
        let fields = Some(vec!["id".to_string()]);
        let result = format_single(&item, &fields).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        let obj = parsed.as_object().unwrap();
        assert_eq!(obj.len(), 1);
        assert!(obj.contains_key("id"));
    }
}
