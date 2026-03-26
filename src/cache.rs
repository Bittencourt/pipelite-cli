use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

// ── TTL constants (seconds) ────────────────────────────────────────
/// Pipelines change rarely -- 1 hour cache.
pub const TTL_PIPELINES: u64 = 3600;
/// Stages change rarely -- 1 hour cache.
pub const TTL_STAGES: u64 = 3600;
/// Users change very rarely -- 2 hour cache.
pub const TTL_USERS: u64 = 7200;
/// Entity lists (deals, orgs, people, activities) change frequently -- 5 min cache.
pub const TTL_ENTITY_LIST: u64 = 300;

// ── Key constants ──────────────────────────────────────────────────
pub const KEY_PIPELINES: &str = "pipelines";
pub const KEY_STAGES: &str = "stages";
pub const KEY_USERS: &str = "users";
pub const KEY_DEALS: &str = "deals";
pub const KEY_ORGS: &str = "orgs";
pub const KEY_PEOPLE: &str = "people";
pub const KEY_ACTIVITIES: &str = "activities";

/// A cached entry with embedded timestamp and TTL for expiry checks.
#[derive(Debug, Serialize, Deserialize)]
struct CacheEntry<T> {
    data: T,
    cached_at: DateTime<Utc>,
    ttl_seconds: u64,
}

impl<T> CacheEntry<T> {
    fn is_expired(&self) -> bool {
        let elapsed = Utc::now()
            .signed_duration_since(self.cached_at)
            .num_seconds();
        elapsed < 0 || elapsed as u64 >= self.ttl_seconds
    }
}

/// TTL-based JSON file cache stored at ~/.pipelite/cache/.
///
/// Each key maps to a single JSON file in the cache directory.
/// Writes use atomic temp-file + rename to prevent corruption.
pub struct CacheStore {
    cache_dir: PathBuf,
}

impl CacheStore {
    /// Create a new CacheStore, creating the cache directory if needed.
    pub fn new() -> Result<Self> {
        let home = std::env::var("HOME")
            .context("Could not determine home directory")?;
        let cache_dir = PathBuf::from(home).join(".pipelite").join("cache");
        fs::create_dir_all(&cache_dir)
            .with_context(|| format!("Failed to create cache directory: {}", cache_dir.display()))?;
        Ok(Self { cache_dir })
    }

    /// Create a CacheStore with a custom directory (for testing).
    #[cfg(test)]
    fn with_dir(dir: PathBuf) -> Result<Self> {
        fs::create_dir_all(&dir)?;
        Ok(Self { cache_dir: dir })
    }

    /// Get a cached value. Returns None if expired, missing, or corrupted.
    ///
    /// On corrupted JSON, the file is silently deleted.
    pub fn get<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
        let path = self.key_path(key);
        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => return None,
        };

        let entry: CacheEntry<T> = match serde_json::from_str(&content) {
            Ok(e) => e,
            Err(_) => {
                // Corrupted file -- delete it silently
                let _ = fs::remove_file(&path);
                return None;
            }
        };

        if entry.is_expired() {
            return None;
        }

        Some(entry.data)
    }

    /// Store a value in the cache with a given TTL.
    ///
    /// Uses atomic write (temp file + rename) to prevent corruption.
    pub fn set<T: Serialize>(&self, key: &str, data: &T, ttl_seconds: u64) -> Result<()> {
        let entry = CacheEntry {
            data,
            cached_at: Utc::now(),
            ttl_seconds,
        };

        let json = serde_json::to_string_pretty(&entry)
            .context("Failed to serialize cache entry")?;

        let target = self.key_path(key);
        let temp = self.cache_dir.join(format!(".{}.tmp", key));

        fs::write(&temp, json.as_bytes())
            .with_context(|| format!("Failed to write cache temp file: {}", temp.display()))?;

        fs::rename(&temp, &target)
            .with_context(|| format!("Failed to rename cache file: {}", target.display()))?;

        Ok(())
    }

    /// Remove all cached files.
    pub fn clear(&self) -> Result<()> {
        fs::remove_dir_all(&self.cache_dir)
            .with_context(|| format!("Failed to clear cache: {}", self.cache_dir.display()))?;
        fs::create_dir_all(&self.cache_dir)
            .with_context(|| format!("Failed to recreate cache dir: {}", self.cache_dir.display()))?;
        Ok(())
    }

    /// Remove a specific cache entry by key. Ignores errors.
    pub fn invalidate(&self, key: &str) {
        let _ = fs::remove_file(self.key_path(key));
    }

    /// Remove all cache entries whose key starts with the given prefix.
    ///
    /// Used for invalidating e.g. "stages_*" when a pipeline changes.
    pub fn invalidate_prefix(&self, prefix: &str) {
        if let Ok(entries) = fs::read_dir(&self.cache_dir) {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    // Strip .json suffix to get the key
                    let key = name.strip_suffix(".json").unwrap_or(name);
                    if key.starts_with(prefix) {
                        let _ = fs::remove_file(entry.path());
                    }
                }
            }
        }
    }

    /// Build the file path for a cache key.
    fn key_path(&self, key: &str) -> PathBuf {
        self.cache_dir.join(format!("{}.json", key))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn test_store() -> (CacheStore, TempDir) {
        let dir = TempDir::new().unwrap();
        let store = CacheStore::with_dir(dir.path().join("cache")).unwrap();
        (store, dir)
    }

    #[test]
    fn set_and_get_returns_data() {
        let (store, _dir) = test_store();
        store.set("test_key", &vec!["hello", "world"], TTL_PIPELINES).unwrap();
        let result: Option<Vec<String>> = store.get("test_key");
        assert!(result.is_some());
        assert_eq!(result.unwrap(), vec!["hello", "world"]);
    }

    #[test]
    fn get_missing_key_returns_none() {
        let (store, _dir) = test_store();
        let result: Option<String> = store.get("nonexistent");
        assert!(result.is_none());
    }

    #[test]
    fn get_expired_entry_returns_none() {
        let (store, _dir) = test_store();
        // Write an entry with 0-second TTL (immediately expired)
        let entry = CacheEntry {
            data: "test",
            cached_at: Utc::now() - chrono::Duration::seconds(10),
            ttl_seconds: 1,
        };
        let path = store.key_path("expired");
        fs::write(&path, serde_json::to_string(&entry).unwrap()).unwrap();

        let result: Option<String> = store.get("expired");
        assert!(result.is_none());
    }

    #[test]
    fn get_corrupted_json_returns_none_and_deletes_file() {
        let (store, _dir) = test_store();
        let path = store.key_path("bad");
        fs::write(&path, "not valid json{{{").unwrap();

        let result: Option<String> = store.get("bad");
        assert!(result.is_none());
        // File should have been deleted
        assert!(!path.exists());
    }

    #[test]
    fn clear_removes_all_files() {
        let (store, _dir) = test_store();
        store.set("a", &"data_a", 3600).unwrap();
        store.set("b", &"data_b", 3600).unwrap();

        store.clear().unwrap();

        let result_a: Option<String> = store.get("a");
        let result_b: Option<String> = store.get("b");
        assert!(result_a.is_none());
        assert!(result_b.is_none());
        // Cache dir still exists
        assert!(store.cache_dir.exists());
    }

    #[test]
    fn invalidate_removes_specific_key() {
        let (store, _dir) = test_store();
        store.set("keep", &"keep_data", 3600).unwrap();
        store.set("remove", &"remove_data", 3600).unwrap();

        store.invalidate("remove");

        let keep: Option<String> = store.get("keep");
        let remove: Option<String> = store.get("remove");
        assert!(keep.is_some());
        assert!(remove.is_none());
    }

    #[test]
    fn invalidate_prefix_removes_matching_keys() {
        let (store, _dir) = test_store();
        store.set("stages_pl_001", &"s1", 3600).unwrap();
        store.set("stages_pl_002", &"s2", 3600).unwrap();
        store.set("pipelines", &"p1", 3600).unwrap();

        store.invalidate_prefix("stages_");

        let s1: Option<String> = store.get("stages_pl_001");
        let s2: Option<String> = store.get("stages_pl_002");
        let p1: Option<String> = store.get("pipelines");
        assert!(s1.is_none());
        assert!(s2.is_none());
        assert!(p1.is_some());
    }

    #[test]
    fn ttl_constants_are_reasonable() {
        assert_eq!(TTL_PIPELINES, 3600);
        assert_eq!(TTL_STAGES, 3600);
        assert_eq!(TTL_USERS, 7200);
        assert_eq!(TTL_ENTITY_LIST, 300);
    }

    #[test]
    fn set_overwrites_existing_entry() {
        let (store, _dir) = test_store();
        store.set("key", &"old_value", 3600).unwrap();
        store.set("key", &"new_value", 3600).unwrap();

        let result: Option<String> = store.get("key");
        assert_eq!(result.unwrap(), "new_value");
    }

    #[test]
    fn invalidate_nonexistent_key_does_not_error() {
        let (store, _dir) = test_store();
        // Should not panic or error
        store.invalidate("does_not_exist");
    }
}
