use comfy_table::{ContentArrangement, Table};

use crate::config::AppConfig;

/// Mask an API key for display: show first 8 chars + "..." or "(not set)" if empty.
fn mask_api_key(key: &str) -> String {
    if key.is_empty() {
        "(not set)".to_string()
    } else if key.len() <= 8 {
        format!("{}...", key)
    } else {
        format!("{}...", &key[..8])
    }
}

/// Build a comfy-table Table for config display.
pub fn config_table(config: &AppConfig) -> Table {
    let mut table = Table::new();
    table
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec!["Key", "Value"]);

    table.add_row(vec!["server.url", &config.server.url]);
    table.add_row(vec!["server.api_key", &mask_api_key(&config.server.api_key)]);

    let format_val = config
        .output
        .format
        .as_deref()
        .unwrap_or("(default: table)");
    table.add_row(vec!["output.format", format_val]);

    let no_color_val = config
        .display
        .no_color
        .map(|v| if v { "true" } else { "false" })
        .unwrap_or("(default: false)");
    table.add_row(vec!["display.no_color", no_color_val]);

    table
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mask_api_key_empty() {
        assert_eq!(mask_api_key(""), "(not set)");
    }

    #[test]
    fn mask_api_key_short() {
        assert_eq!(mask_api_key("abc"), "abc...");
    }

    #[test]
    fn mask_api_key_normal() {
        assert_eq!(mask_api_key("pk_live_abc123xyz"), "pk_live_...");
    }

    #[test]
    fn config_table_has_all_rows() {
        let config = AppConfig::new(
            "https://test.example.com".to_string(),
            "pk_test_longkey123".to_string(),
        );
        let tbl = config_table(&config);
        let output = tbl.to_string();
        assert!(output.contains("server.url"));
        assert!(output.contains("test.example.com"));
        assert!(output.contains("server.api_key"));
        assert!(output.contains("pk_test_..."));
        assert!(output.contains("output.format"));
        assert!(output.contains("display.no_color"));
    }
}
