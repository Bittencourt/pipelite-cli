/// Value formatting for table display: relative dates, currency, truncation.
use chrono::{DateTime, Utc};
use chrono_humanize::HumanTime;

/// Format a JSON value for table display based on field name heuristics.
///
/// - Fields ending in `_at` or `_date`: relative time (e.g., "2h ago")
/// - `value` field: currency with commas (e.g., "50,000")
/// - All others: plain string representation
pub fn format_value(key: &str, value: &serde_json::Value, _color: bool) -> String {
    if value.is_null() {
        return String::new();
    }

    if key.ends_with("_at") || key.ends_with("_date") {
        if let Some(s) = value.as_str() {
            return format_relative_date(s);
        }
    }

    if key == "value" {
        if let Some(n) = value.as_f64() {
            return format_currency(n);
        }
    }

    match value {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Null => String::new(),
        _ => serde_json::to_string(value).unwrap_or_default(),
    }
}

/// Format an ISO 8601 date string as a relative time (e.g., "3 days ago").
/// Falls back to the raw string if parsing fails.
fn format_relative_date(s: &str) -> String {
    if let Ok(dt) = s.parse::<DateTime<Utc>>() {
        let ht = HumanTime::from(dt);
        ht.to_string()
    } else {
        s.to_string()
    }
}

/// Format a number as currency with comma separators (no currency symbol).
fn format_currency(n: f64) -> String {
    let is_negative = n < 0.0;
    let abs = n.abs();

    // Format integer part with comma separators
    let int_part = abs as u64;
    let frac_part = abs - int_part as f64;

    let int_str = format_with_commas(int_part);

    let result = if frac_part > 0.001 {
        // Keep up to 2 decimal places
        let decimals = format!("{:.2}", frac_part);
        // decimals is like "0.50"
        format!("{}{}", int_str, &decimals[1..])
    } else {
        int_str
    };

    if is_negative {
        format!("-{result}")
    } else {
        result
    }
}

/// Format a u64 with comma thousand separators.
fn format_with_commas(n: u64) -> String {
    let s = n.to_string();
    let len = s.len();
    if len <= 3 {
        return s;
    }

    let mut result = String::with_capacity(len + len / 3);
    for (i, ch) in s.chars().enumerate() {
        if i > 0 && (len - i) % 3 == 0 {
            result.push(',');
        }
        result.push(ch);
    }
    result
}

#[allow(dead_code)]
/// Truncate a string to max_width, appending "..." if it exceeds the limit.
///
/// Returns the original string if it fits within max_width.
pub fn truncate_with_ellipsis(s: &str, max_width: usize) -> String {
    if max_width < 4 {
        // Not enough room for any content plus "..."
        return s.chars().take(max_width).collect();
    }

    let char_count: usize = s.chars().count();
    if char_count <= max_width {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max_width - 3).collect();
        format!("{truncated}...")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn format_value_relative_date() {
        // Use a known past date
        let val = json!("2020-01-01T00:00:00Z");
        let result = format_value("created_at", &val, false);
        // Should contain "ago" since it's in the past
        assert!(result.contains("ago"), "Expected relative date, got: {result}");
    }

    #[test]
    fn format_value_date_field_ending_with_date() {
        let val = json!("2020-06-15T00:00:00Z");
        let result = format_value("expected_close_date", &val, false);
        assert!(
            result.contains("ago") || result.contains("year"),
            "Expected relative date, got: {result}"
        );
    }

    #[test]
    fn format_value_unparseable_date_returns_raw() {
        let val = json!("not-a-date");
        let result = format_value("created_at", &val, false);
        assert_eq!(result, "not-a-date");
    }

    #[test]
    fn format_value_currency() {
        let val = json!(50000.0);
        let result = format_value("value", &val, false);
        assert_eq!(result, "50,000");
    }

    #[test]
    fn format_value_currency_with_decimals() {
        let val = json!(1234.56);
        let result = format_value("value", &val, false);
        assert_eq!(result, "1,234.56");
    }

    #[test]
    fn format_value_currency_small() {
        let val = json!(42.0);
        let result = format_value("value", &val, false);
        assert_eq!(result, "42");
    }

    #[test]
    fn format_value_plain_string() {
        let val = json!("hello");
        let result = format_value("title", &val, false);
        assert_eq!(result, "hello");
    }

    #[test]
    fn format_value_null_returns_empty() {
        let val = json!(null);
        let result = format_value("notes", &val, false);
        assert_eq!(result, "");
    }

    #[test]
    fn truncate_short_string_unchanged() {
        assert_eq!(truncate_with_ellipsis("hello", 10), "hello");
    }

    #[test]
    fn truncate_exact_fit() {
        assert_eq!(truncate_with_ellipsis("hello", 5), "hello");
    }

    #[test]
    fn truncate_with_ellipsis_applied() {
        assert_eq!(
            truncate_with_ellipsis("This is a very long title", 15),
            "This is a ve..."
        );
    }

    #[test]
    fn truncate_very_small_width() {
        assert_eq!(truncate_with_ellipsis("hello", 2), "he");
    }

    #[test]
    fn format_currency_millions() {
        let val = json!(1500000.0);
        let result = format_value("value", &val, false);
        assert_eq!(result, "1,500,000");
    }
}
