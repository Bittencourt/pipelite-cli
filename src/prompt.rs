use std::io::IsTerminal;

use anyhow::Result;
use dialoguer::{FuzzySelect, Input};

use crate::api::{OrgsListParams, PipelinesListParams, PipeliteClient, StagesListParams};
use crate::cache::{CacheStore, KEY_ORGS, KEY_PIPELINES, TTL_ENTITY_LIST, TTL_PIPELINES, TTL_STAGES};
use crate::error::CliError;

/// Prompt for a required text field interactively, or collect it from flag value.
///
/// If `flag_value` is Some, returns it immediately. Otherwise, if stdin is a TTY
/// and `no_input` is false, prompts the user interactively. If neither, pushes
/// `--{field_name}` to the missing list for batch error reporting.
pub fn require_text(
    flag_value: &Option<String>,
    field_name: &str,
    prompt_label: &str,
    missing: &mut Vec<String>,
    no_input: bool,
) -> Result<Option<String>> {
    if let Some(val) = flag_value {
        return Ok(Some(val.clone()));
    }

    if std::io::stdin().is_terminal() && !no_input {
        let input: String = Input::new()
            .with_prompt(prompt_label)
            .interact_text()?;
        if input.is_empty() {
            missing.push(format!("--{}", field_name));
            return Ok(None);
        }
        return Ok(Some(input));
    }

    missing.push(format!("--{}", field_name));
    Ok(None)
}

/// Prompt for a required selection from a list of options via FuzzySelect.
///
/// `options` is a slice of `(id, display_name)` tuples. The display format is
/// "name (id)". Returns the selected ID.
pub fn require_select(
    flag_value: &Option<String>,
    field_name: &str,
    prompt_label: &str,
    options: &[(String, String)],
    missing: &mut Vec<String>,
    no_input: bool,
) -> Result<Option<String>> {
    if let Some(val) = flag_value {
        return Ok(Some(val.clone()));
    }

    if std::io::stdin().is_terminal() && !no_input {
        if options.is_empty() {
            return Err(CliError::Validation {
                detail: format!("No options available for {}", field_name),
                hint: format!("Create a {} first, then retry.", field_name),
            }
            .into());
        }

        let display: Vec<String> = options
            .iter()
            .map(|(id, name)| format!("{} ({})", name, id))
            .collect();

        let selection = FuzzySelect::new()
            .with_prompt(prompt_label)
            .items(&display)
            .default(0)
            .interact()?;

        return Ok(Some(options[selection].0.clone()));
    }

    missing.push(format!("--{}", field_name));
    Ok(None)
}

/// Prompt for an optional text field interactively.
///
/// Returns flag value if present, prompts if TTY and not no_input,
/// or returns None in headless mode.
pub fn optional_text(
    flag_value: &Option<String>,
    prompt_label: &str,
    no_input: bool,
) -> Result<Option<String>> {
    if let Some(val) = flag_value {
        return Ok(Some(val.clone()));
    }

    if std::io::stdin().is_terminal() && !no_input {
        let input: String = Input::new()
            .with_prompt(format!("{} (optional, press Enter to skip)", prompt_label))
            .allow_empty(true)
            .interact_text()?;
        if input.is_empty() {
            return Ok(None);
        }
        return Ok(Some(input));
    }

    Ok(None)
}

/// Prompt for an optional numeric field interactively.
///
/// Same pattern as optional_text but parses the input as f64.
pub fn optional_number(
    flag_value: &Option<f64>,
    prompt_label: &str,
    no_input: bool,
) -> Result<Option<f64>> {
    if let Some(val) = flag_value {
        return Ok(Some(*val));
    }

    if std::io::stdin().is_terminal() && !no_input {
        let input: String = Input::new()
            .with_prompt(format!("{} (optional, press Enter to skip)", prompt_label))
            .allow_empty(true)
            .interact_text()?;
        if input.is_empty() {
            return Ok(None);
        }
        let num: f64 = input.parse().map_err(|_| CliError::Validation {
            detail: format!("Invalid number: '{}'", input),
            hint: "Enter a valid number (e.g., 50000.00).".to_string(),
        })?;
        return Ok(Some(num));
    }

    Ok(None)
}

/// Check if any required fields are missing and return a batch error.
///
/// If `missing` is non-empty, returns a `CliError::MissingInput` listing all
/// missing flags at once, so the user can fix them in a single retry.
pub fn check_missing(missing: &[String], usage_hint: &str) -> Result<(), CliError> {
    if missing.is_empty() {
        return Ok(());
    }

    Err(CliError::MissingInput {
        detail: format!("Missing required flags: {}", missing.join(", ")),
        hint: usage_hint.to_string(),
    })
}

// ── Cache-through helpers ─────────────────────────────────────────

/// Fetch pipelines with cache-through: try cache first, fall back to API on miss.
///
/// Returns `Vec<(id, name)>` suitable for FuzzySelect options.
/// When cache is `None`, always fetches from API.
pub async fn get_pipelines_cached(
    cache: Option<&CacheStore>,
    client: &PipeliteClient,
) -> Result<Vec<(String, String)>> {
    // Try cache first
    if let Some(store) = cache {
        if let Some(cached) = store.get::<Vec<(String, String)>>(KEY_PIPELINES) {
            return Ok(cached);
        }
    }

    // Cache miss -- fetch from API with auto-paginate
    let mut all_items: Vec<(String, String)> = Vec::new();
    let mut offset: u64 = 0;
    let limit: u64 = 100;

    loop {
        let resp = client
            .list_pipelines(&PipelinesListParams {
                limit,
                offset,
                expand: None,
            })
            .await?;

        let batch_len = resp.data.len() as u64;
        for p in &resp.data {
            all_items.push((p.id.clone(), p.name.clone()));
        }

        if batch_len < limit {
            break;
        }
        offset += limit;
        if offset >= 1000 {
            break;
        }
    }

    // Populate cache on successful fetch
    if let Some(store) = cache {
        let _ = store.set(KEY_PIPELINES, &all_items, TTL_PIPELINES);
    }

    Ok(all_items)
}

/// Fetch stages for a pipeline with cache-through: try cache first, fall back to API on miss.
///
/// Cache key: `stages_{pipeline_id}`. Returns `Vec<(id, name)>`.
pub async fn get_stages_cached(
    cache: Option<&CacheStore>,
    client: &PipeliteClient,
    pipeline_id: &str,
) -> Result<Vec<(String, String)>> {
    let cache_key = format!("stages_{}", pipeline_id);

    // Try cache first
    if let Some(store) = cache {
        if let Some(cached) = store.get::<Vec<(String, String)>>(&cache_key) {
            return Ok(cached);
        }
    }

    // Cache miss -- fetch from API with auto-paginate
    let mut all_items: Vec<(String, String)> = Vec::new();
    let mut offset: u64 = 0;
    let limit: u64 = 100;

    loop {
        let resp = client
            .list_stages(&StagesListParams {
                pipeline_id: pipeline_id.to_string(),
                limit,
                offset,
                expand: None,
            })
            .await?;

        let batch_len = resp.data.len() as u64;
        for s in &resp.data {
            all_items.push((s.id.clone(), s.name.clone()));
        }

        if batch_len < limit {
            break;
        }
        offset += limit;
        if offset >= 1000 {
            break;
        }
    }

    // Populate cache
    if let Some(store) = cache {
        let _ = store.set(&cache_key, &all_items, TTL_STAGES);
    }

    Ok(all_items)
}

/// Fetch organizations with cache-through: try cache first, fall back to API on miss.
///
/// Uses TTL_ENTITY_LIST (5 min) since org lists change more frequently.
/// Returns `Vec<(id, name)>`.
pub async fn get_orgs_cached(
    cache: Option<&CacheStore>,
    client: &PipeliteClient,
) -> Result<Vec<(String, String)>> {
    // Try cache first
    if let Some(store) = cache {
        if let Some(cached) = store.get::<Vec<(String, String)>>(KEY_ORGS) {
            return Ok(cached);
        }
    }

    // Cache miss -- fetch from API with auto-paginate
    let mut all_items: Vec<(String, String)> = Vec::new();
    let mut offset: u64 = 0;
    let limit: u64 = 100;

    loop {
        let resp = client
            .list_orgs(&OrgsListParams {
                owner: None,
                limit,
                offset,
                expand: None,
            })
            .await?;

        let batch_len = resp.data.len() as u64;
        for o in &resp.data {
            all_items.push((o.id.clone(), o.name.clone()));
        }

        if batch_len < limit {
            break;
        }
        offset += limit;
        if offset >= 1000 {
            break;
        }
    }

    // Populate cache
    if let Some(store) = cache {
        let _ = store.set(KEY_ORGS, &all_items, TTL_ENTITY_LIST);
    }

    Ok(all_items)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::CacheStore;
    use tempfile::TempDir;

    fn test_cache() -> (CacheStore, TempDir) {
        let dir = TempDir::new().unwrap();
        let store = CacheStore::with_dir(dir.path().join("cache")).unwrap();
        (store, dir)
    }

    #[tokio::test]
    async fn get_pipelines_cached_returns_cached_data_on_hit() {
        let (store, _dir) = test_cache();
        let items = vec![
            ("pl_001".to_string(), "Sales Pipeline".to_string()),
            ("pl_002".to_string(), "Support Pipeline".to_string()),
        ];
        store.set(KEY_PIPELINES, &items, TTL_PIPELINES).unwrap();

        // Create a client pointing to unreachable server -- should never be called
        let client = PipeliteClient::from_credentials(
            "http://127.0.0.1:1",
            "fake-key",
        ).unwrap();

        let result = get_pipelines_cached(Some(&store), &client).await.unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], ("pl_001".to_string(), "Sales Pipeline".to_string()));
        assert_eq!(result[1], ("pl_002".to_string(), "Support Pipeline".to_string()));
    }

    #[tokio::test]
    async fn get_pipelines_cached_falls_back_to_api_on_miss() {
        let (store, _dir) = test_cache();

        // Client points to unreachable server -- should get connection error (proving API was attempted)
        let client = PipeliteClient::from_credentials(
            "http://127.0.0.1:1",
            "fake-key",
        ).unwrap();

        let result = get_pipelines_cached(Some(&store), &client).await;
        assert!(result.is_err(), "Expected error from unreachable API (proves fallback path triggered)");
    }

    #[tokio::test]
    async fn get_pipelines_cached_works_with_none_cache() {
        // None cache should attempt API -- gets connection error (proves no panic)
        let client = PipeliteClient::from_credentials(
            "http://127.0.0.1:1",
            "fake-key",
        ).unwrap();

        let result = get_pipelines_cached(None, &client).await;
        assert!(result.is_err(), "Expected error from unreachable API with None cache");
    }

    #[tokio::test]
    async fn get_stages_cached_uses_pipeline_specific_key() {
        let (store, _dir) = test_cache();
        let items = vec![
            ("st_001".to_string(), "Qualification".to_string()),
        ];
        // Store under the pipeline-specific key
        store.set("stages_pl_abc", &items, TTL_STAGES).unwrap();

        let client = PipeliteClient::from_credentials(
            "http://127.0.0.1:1",
            "fake-key",
        ).unwrap();

        // Should find the cached data using stages_{pipeline_id}
        let result = get_stages_cached(Some(&store), &client, "pl_abc").await.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "st_001");

        // Different pipeline ID should miss cache and try API
        let result2 = get_stages_cached(Some(&store), &client, "pl_other").await;
        assert!(result2.is_err(), "Different pipeline should miss cache and hit unreachable API");
    }

    #[tokio::test]
    async fn get_orgs_cached_returns_cached_data() {
        let (store, _dir) = test_cache();
        let items = vec![
            ("org_001".to_string(), "Acme Corp".to_string()),
        ];
        store.set(KEY_ORGS, &items, TTL_ENTITY_LIST).unwrap();

        let client = PipeliteClient::from_credentials(
            "http://127.0.0.1:1",
            "fake-key",
        ).unwrap();

        let result = get_orgs_cached(Some(&store), &client).await.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], ("org_001".to_string(), "Acme Corp".to_string()));
    }
}
