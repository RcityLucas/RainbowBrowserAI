use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;
use tracing::{debug, info};

/// Cache entry with expiration and metadata
#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub value: Value,
    pub created_at: SystemTime,
    pub expires_at: SystemTime,
    pub access_count: u64,
    pub last_accessed: SystemTime,
    pub tool_name: String,
    pub input_hash: String,
}

impl CacheEntry {
    pub fn new(value: Value, ttl: Duration, tool_name: String, input_hash: String) -> Self {
        let now = SystemTime::now();
        Self {
            value,
            created_at: now,
            expires_at: now + ttl,
            access_count: 0,
            last_accessed: now,
            tool_name,
            input_hash,
        }
    }

    pub fn is_expired(&self) -> bool {
        SystemTime::now() > self.expires_at
    }

    pub fn access(&mut self) -> &Value {
        self.access_count += 1;
        self.last_accessed = SystemTime::now();
        &self.value
    }

    pub fn age(&self) -> Duration {
        SystemTime::now()
            .duration_since(self.created_at)
            .unwrap_or(Duration::from_secs(0))
    }
}

/// Configuration for cache behavior per tool
#[derive(Debug, Clone)]
pub struct CacheConfig {
    pub ttl: Duration,
    pub max_entries: usize,
    pub enabled: bool,
    pub invalidate_on_navigation: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            ttl: Duration::from_secs(300), // 5 minutes
            max_entries: 100,
            enabled: true,
            invalidate_on_navigation: false,
        }
    }
}

/// Smart caching system for tool execution results
pub struct ToolCache {
    entries: Arc<RwLock<HashMap<String, CacheEntry>>>,
    configs: Arc<RwLock<HashMap<String, CacheConfig>>>,
    current_url: Arc<RwLock<String>>,
}

impl ToolCache {
    pub fn new() -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            configs: Arc::new(RwLock::new(HashMap::new())),
            current_url: Arc::new(RwLock::new(String::new())),
        }
    }

    /// Generate a cache key from tool name and input
    pub fn generate_key(&self, tool_name: &str, input: &Value) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        tool_name.hash(&mut hasher);
        input.to_string().hash(&mut hasher);
        format!("{}:{:x}", tool_name, hasher.finish())
    }

    /// Set cache configuration for a specific tool
    pub async fn set_tool_config(&self, tool_name: &str, config: CacheConfig) {
        let mut configs = self.configs.write().await;
        info!(
            "Cache config set for tool '{}': ttl={}s, max_entries={}, enabled={}",
            tool_name,
            config.ttl.as_secs(),
            config.max_entries,
            config.enabled
        );
        configs.insert(tool_name.to_string(), config);
    }

    /// Get cache configuration for a tool (or default)
    pub async fn get_tool_config(&self, tool_name: &str) -> CacheConfig {
        let configs = self.configs.read().await;
        configs.get(tool_name).cloned().unwrap_or_else(|| {
            // Set smart defaults based on tool type
            match tool_name {
                "screenshot" => CacheConfig {
                    ttl: Duration::from_secs(60), // Screenshots expire quickly
                    max_entries: 20,
                    enabled: true,
                    invalidate_on_navigation: true,
                },
                "extract_text" | "extract_links" | "extract_data" => CacheConfig {
                    ttl: Duration::from_secs(120), // Content extraction medium TTL
                    max_entries: 50,
                    enabled: true,
                    invalidate_on_navigation: true,
                },
                "monitor_network" | "get_performance_metrics" => CacheConfig {
                    ttl: Duration::from_secs(30), // Performance data expires fast
                    max_entries: 10,
                    enabled: true,
                    invalidate_on_navigation: false,
                },
                "wait_for_element" | "wait_for_condition" => CacheConfig {
                    ttl: Duration::from_secs(10), // Wait operations very short TTL
                    max_entries: 30,
                    enabled: false, // Usually don't cache wait operations
                    invalidate_on_navigation: true,
                },
                _ => CacheConfig::default(),
            }
        })
    }

    /// Check if a cached result exists and is valid
    pub async fn get(&self, tool_name: &str, input: &Value) -> Option<Value> {
        let config = self.get_tool_config(tool_name).await;
        if !config.enabled {
            return None;
        }

        let key = self.generate_key(tool_name, input);
        let mut entries = self.entries.write().await;

        if let Some(entry) = entries.get_mut(&key) {
            if entry.is_expired() {
                debug!("Cache entry expired for tool '{}', removing", tool_name);
                entries.remove(&key);
                return None;
            }

            let value = entry.access().clone();
            debug!(
                "Cache hit for tool '{}' (age: {}s, access_count: {})",
                tool_name,
                entry.age().as_secs(),
                entry.access_count
            );
            Some(value)
        } else {
            debug!("Cache miss for tool '{}'", tool_name);
            None
        }
    }

    /// Store a result in the cache
    pub async fn set(&self, tool_name: &str, input: &Value, result: &Value) {
        let config = self.get_tool_config(tool_name).await;
        if !config.enabled {
            return;
        }

        let key = self.generate_key(tool_name, input);
        let input_hash = self.hash_input(input);
        let entry = CacheEntry::new(
            result.clone(),
            config.ttl,
            tool_name.to_string(),
            input_hash,
        );

        let mut entries = self.entries.write().await;

        // Enforce max entries limit
        if entries.len() >= config.max_entries {
            self.evict_oldest(&mut entries, &config).await;
        }

        entries.insert(key, entry);
        debug!(
            "Cached result for tool '{}' (TTL: {}s)",
            tool_name,
            config.ttl.as_secs()
        );
    }

    /// Evict oldest entries to make room
    async fn evict_oldest(&self, entries: &mut HashMap<String, CacheEntry>, config: &CacheConfig) {
        let target_size = config.max_entries.saturating_sub(config.max_entries / 4); // Remove 25%

        if entries.len() <= target_size {
            return;
        }

        // Sort by last accessed time and collect keys to remove
        let mut sorted_entries: Vec<_> = entries
            .iter()
            .map(|(key, entry)| (key.clone(), entry.last_accessed))
            .collect();
        sorted_entries.sort_by_key(|(_, last_accessed)| *last_accessed);

        let to_remove = entries.len() - target_size;
        let keys_to_remove: Vec<String> = sorted_entries
            .into_iter()
            .take(to_remove)
            .map(|(key, _)| key)
            .collect();

        for key in keys_to_remove {
            entries.remove(&key);
        }

        debug!("Evicted {} cache entries to make room", to_remove);
    }

    /// Clear cache for a specific tool
    pub async fn clear_tool_cache(&self, tool_name: &str) {
        let mut entries = self.entries.write().await;
        let initial_count = entries.len();
        entries.retain(|_, entry| entry.tool_name != tool_name);
        let removed = initial_count - entries.len();

        if removed > 0 {
            info!("Cleared {} cache entries for tool '{}'", removed, tool_name);
        }
    }

    /// Clear all cache entries
    pub async fn clear_all(&self) {
        let mut entries = self.entries.write().await;
        let count = entries.len();
        entries.clear();
        info!("Cleared all {} cache entries", count);
    }

    /// Handle navigation - invalidate navigation-sensitive caches
    pub async fn on_navigation(&self, new_url: &str) {
        let mut current_url = self.current_url.write().await;
        if *current_url != new_url {
            *current_url = new_url.to_string();

            let mut entries = self.entries.write().await;
            let configs = self.configs.read().await;
            let initial_count = entries.len();

            entries.retain(|_, entry| {
                let config = configs.get(&entry.tool_name).cloned().unwrap_or_default();
                !config.invalidate_on_navigation
            });

            let removed = initial_count - entries.len();
            if removed > 0 {
                info!(
                    "Navigation to '{}': invalidated {} cache entries",
                    new_url, removed
                );
            }
        }
    }

    /// Cleanup expired entries
    pub async fn cleanup_expired(&self) {
        let mut entries = self.entries.write().await;
        let initial_count = entries.len();
        entries.retain(|_, entry| !entry.is_expired());
        let removed = initial_count - entries.len();

        if removed > 0 {
            debug!("Cleaned up {} expired cache entries", removed);
        }
    }

    /// Get cache statistics
    pub async fn get_stats(&self) -> CacheStats {
        let entries = self.entries.read().await;
        let mut tool_stats = HashMap::new();
        let mut total_size = 0;
        let mut expired_count = 0;

        for (_, entry) in entries.iter() {
            total_size += entry.value.to_string().len();

            if entry.is_expired() {
                expired_count += 1;
            }

            let stats = tool_stats
                .entry(entry.tool_name.clone())
                .or_insert(ToolCacheStats {
                    total_entries: 0,
                    total_accesses: 0,
                    avg_age_seconds: 0.0,
                    oldest_entry_seconds: 0,
                });

            stats.total_entries += 1;
            stats.total_accesses += entry.access_count;

            let age_seconds = entry.age().as_secs();
            stats.avg_age_seconds += age_seconds as f64;
            stats.oldest_entry_seconds = stats.oldest_entry_seconds.max(age_seconds);
        }

        // Calculate averages
        for stats in tool_stats.values_mut() {
            if stats.total_entries > 0 {
                stats.avg_age_seconds /= stats.total_entries as f64;
            }
        }

        CacheStats {
            total_entries: entries.len(),
            expired_entries: expired_count,
            estimated_size_bytes: total_size,
            tool_stats,
        }
    }

    fn hash_input(&self, input: &Value) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        input.to_string().hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
}

impl Default for ToolCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about cache usage
#[derive(Debug, Clone, serde::Serialize)]
pub struct CacheStats {
    pub total_entries: usize,
    pub expired_entries: usize,
    pub estimated_size_bytes: usize,
    pub tool_stats: HashMap<String, ToolCacheStats>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ToolCacheStats {
    pub total_entries: usize,
    pub total_accesses: u64,
    pub avg_age_seconds: f64,
    pub oldest_entry_seconds: u64,
}

/// Background task to periodically clean up expired cache entries
pub async fn start_cache_cleanup_task(cache: Arc<ToolCache>, interval: Duration) {
    let mut cleanup_interval = tokio::time::interval(interval);

    loop {
        cleanup_interval.tick().await;
        cache.cleanup_expired().await;
    }
}
