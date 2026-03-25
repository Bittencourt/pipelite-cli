/// Field selection and extraction from serde_json::Value using dot-notation paths.

/// Extract a single field value from a JSON value using dot-notation path.
///
/// For example, `extract_field(value, "custom_fields.industry")` traverses
/// into nested objects. Returns empty string for missing or null fields.
pub fn extract_field(value: &serde_json::Value, path: &str) -> String {
    let mut current = value;
    for part in path.split('.') {
        match current.get(part) {
            Some(v) => current = v,
            None => return String::new(),
        }
    }
    value_to_string(current)
}

/// Convert a serde_json::Value to a display string.
fn value_to_string(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Null => String::new(),
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Array(_) | serde_json::Value::Object(_) => {
            serde_json::to_string(value).unwrap_or_default()
        }
    }
}

/// Filter a JSON value to include only the requested fields.
///
/// Returns a new Value::Object containing only the specified fields.
/// Supports dot notation for nested field access.
pub fn filter_fields(value: &serde_json::Value, fields: &[String]) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    for field in fields {
        let extracted = extract_field_value(value, field);
        // Use the leaf key name for the output
        let key = field.split('.').last().unwrap_or(field.as_str());
        map.insert(key.to_string(), extracted);
    }
    serde_json::Value::Object(map)
}

/// Extract a field value preserving its JSON type (not converting to string).
fn extract_field_value(value: &serde_json::Value, path: &str) -> serde_json::Value {
    let mut current = value;
    for part in path.split('.') {
        match current.get(part) {
            Some(v) => current = v,
            None => return serde_json::Value::Null,
        }
    }
    current.clone()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn extract_top_level_string() {
        let val = json!({"title": "Big Deal"});
        assert_eq!(extract_field(&val, "title"), "Big Deal");
    }

    #[test]
    fn extract_nested_dot_notation() {
        let val = json!({"custom_fields": {"industry": "Tech"}});
        assert_eq!(extract_field(&val, "custom_fields.industry"), "Tech");
    }

    #[test]
    fn extract_missing_field_returns_empty() {
        let val = json!({"title": "Deal"});
        assert_eq!(extract_field(&val, "nonexistent"), "");
    }

    #[test]
    fn extract_null_field_returns_empty() {
        let val = json!({"notes": null});
        assert_eq!(extract_field(&val, "notes"), "");
    }

    #[test]
    fn extract_number_field() {
        let val = json!({"value": 50000.5});
        assert_eq!(extract_field(&val, "value"), "50000.5");
    }

    #[test]
    fn extract_bool_field() {
        let val = json!({"active": true});
        assert_eq!(extract_field(&val, "active"), "true");
    }

    #[test]
    fn extract_deeply_nested() {
        let val = json!({"a": {"b": {"c": "deep"}}});
        assert_eq!(extract_field(&val, "a.b.c"), "deep");
    }

    #[test]
    fn extract_missing_intermediate_returns_empty() {
        let val = json!({"a": {"b": 1}});
        assert_eq!(extract_field(&val, "a.x.y"), "");
    }

    #[test]
    fn filter_fields_selects_requested_fields() {
        let val = json!({
            "id": "deal_001",
            "title": "Big Deal",
            "value": 50000,
            "notes": "secret"
        });
        let filtered = filter_fields(&val, &["id".to_string(), "title".to_string()]);
        let obj = filtered.as_object().unwrap();
        assert_eq!(obj.len(), 2);
        assert_eq!(obj["id"], "deal_001");
        assert_eq!(obj["title"], "Big Deal");
    }

    #[test]
    fn filter_fields_with_dot_notation() {
        let val = json!({
            "id": "deal_001",
            "custom_fields": {"industry": "Tech"}
        });
        let filtered = filter_fields(
            &val,
            &["id".to_string(), "custom_fields.industry".to_string()],
        );
        let obj = filtered.as_object().unwrap();
        assert_eq!(obj["id"], "deal_001");
        assert_eq!(obj["industry"], "Tech");
    }
}
