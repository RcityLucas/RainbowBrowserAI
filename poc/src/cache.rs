use anyhow::Result;
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info};

/// A cached value with metadata
#[derive(Debug, Clone)]
pub struct CachedValue<V> {
    pub value: V,
    pub created_at: Instant,
    pub last_accessed: Instant,
    pub access_count: usize,
    pub ttl: Duration,
}

impl<V> CachedValue<V> {
    /// Check if the cached value has expired
    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed() > self.ttl
    }

    /// Update last accessed time and increment access count
    pub fn touch(&mut self) {
        self.last_accessed = Instant::now();
        self.access_count += 1;
    }
}

/// Thread-safe cache with TTL and size limits
pub struct Cache<K, V> 
where 
    K: Eq + Hash + Clone,
    V: Clone,
{
    store: Arc<RwLock<HashMap<K, CachedValue<V>>>>,
    default_ttl: Duration,
    max_size: usize,
    stats: Arc<RwLock<CacheStats>>,
}

#[derive(Debug, Default, Clone)]
pub struct CacheStats {
    pub hits: usize,
    pub misses: usize,
    pub evictions: usize,
    pub expirations: usize,
    pub current_size: usize,
}

impl CacheStats {
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }
}

impl<K, V> Cache<K, V>
where
    K: Eq + Hash + Clone + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    /// Create a new cache with default settings
    pub fn new() -> Self {
        Self::with_config(Duration::from_secs(300), 1000)
    }

    /// Create a new cache with custom configuration
    pub fn with_config(default_ttl: Duration, max_size: usize) -> Self {
        info!("Initializing cache (ttl: {:?}, max_size: {})", default_ttl, max_size);
        
        Self {
            store: Arc::new(RwLock::new(HashMap::new())),
            default_ttl,
            max_size,
            stats: Arc::new(RwLock::new(CacheStats::default())),
        }
    }

    /// Get a value from the cache
    pub async fn get(&self, key: &K) -> Option<V> {
        let mut store = self.store.write().await;
        
        if let Some(cached) = store.get_mut(key) {
            if cached.is_expired() {
                debug!("Cache entry expired for key");
                store.remove(key);
                
                let mut stats = self.stats.write().await;
                stats.expirations += 1;
                stats.current_size = store.len();
                stats.misses += 1;
                
                return None;
            }
            
            cached.touch();
            let value = cached.value.clone();
            
            let mut stats = self.stats.write().await;
            stats.hits += 1;
            
            debug!("Cache hit (access count: {})", cached.access_count);
            Some(value)
        } else {
            let mut stats = self.stats.write().await;
            stats.misses += 1;
            
            debug!("Cache miss");
            None
        }
    }

    /// Get a value from the cache or compute it if missing
    pub async fn get_or_insert_with<F, Fut>(&self, key: K, f: F) -> Result<V>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<V>>,
    {
        // Check cache first
        if let Some(value) = self.get(&key).await {
            return Ok(value);
        }

        // Compute the value
        let value = f().await?;

        // Insert into cache
        self.insert(key, value.clone()).await;

        Ok(value)
    }

    /// Insert a value into the cache
    pub async fn insert(&self, key: K, value: V) {
        self.insert_with_ttl(key, value, self.default_ttl).await;
    }

    /// Insert a value with custom TTL
    pub async fn insert_with_ttl(&self, key: K, value: V, ttl: Duration) {
        let mut store = self.store.write().await;
        
        // Check if we need to evict entries
        if store.len() >= self.max_size && !store.contains_key(&key) {
            self.evict_lru(&mut store).await;
        }
        
        let cached = CachedValue {
            value,
            created_at: Instant::now(),
            last_accessed: Instant::now(),
            access_count: 0,
            ttl,
        };
        
        store.insert(key, cached);
        
        let mut stats = self.stats.write().await;
        stats.current_size = store.len();
        
        debug!("Cached value inserted (size: {})", store.len());
    }

    /// Remove a value from the cache
    pub async fn remove(&self, key: &K) -> Option<V> {
        let mut store = self.store.write().await;
        let removed = store.remove(key).map(|cached| cached.value);
        
        if removed.is_some() {
            let mut stats = self.stats.write().await;
            stats.current_size = store.len();
            
            debug!("Cache entry removed");
        }
        
        removed
    }

    /// Clear all entries from the cache
    pub async fn clear(&self) {
        let mut store = self.store.write().await;
        store.clear();
        
        let mut stats = self.stats.write().await;
        stats.current_size = 0;
        
        info!("Cache cleared");
    }

    /// Clean up expired entries
    pub async fn cleanup_expired(&self) {
        let mut store = self.store.write().await;
        let before_size = store.len();
        
        store.retain(|_, cached| !cached.is_expired());
        
        let removed = before_size - store.len();
        if removed > 0 {
            let mut stats = self.stats.write().await;
            stats.expirations += removed;
            stats.current_size = store.len();
            
            info!("Cleaned up {} expired cache entries", removed);
        }
    }

    /// Evict the least recently used entry
    async fn evict_lru(&self, store: &mut HashMap<K, CachedValue<V>>) {
        if let Some((key, _)) = store
            .iter()
            .min_by_key(|(_, cached)| cached.last_accessed)
            .map(|(k, v)| (k.clone(), v.clone()))
        {
            store.remove(&key);
            
            let mut stats = self.stats.write().await;
            stats.evictions += 1;
            
            debug!("Evicted LRU cache entry");
        }
    }

    /// Get cache statistics
    pub async fn stats(&self) -> CacheStats {
        self.stats.read().await.clone()
    }

    /// Get the current size of the cache
    pub async fn size(&self) -> usize {
        self.store.read().await.len()
    }
}

/// Specialized cache for LLM responses
pub struct LLMCache {
    cache: Cache<String, serde_json::Value>,
}

impl LLMCache {
    pub fn new() -> Self {
        // LLM responses cached for 1 hour with max 500 entries
        Self {
            cache: Cache::with_config(Duration::from_secs(3600), 500),
        }
    }

    /// Generate a cache key from the prompt
    pub fn generate_key(prompt: &str, model: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        prompt.hash(&mut hasher);
        model.hash(&mut hasher);
        
        format!("llm_{}_{}", model, hasher.finish())
    }

    /// Get a cached LLM response
    pub async fn get(&self, prompt: &str, model: &str) -> Option<serde_json::Value> {
        let key = Self::generate_key(prompt, model);
        self.cache.get(&key).await
    }

    /// Cache an LLM response
    pub async fn insert(&self, prompt: &str, model: &str, response: serde_json::Value) {
        let key = Self::generate_key(prompt, model);
        self.cache.insert(key, response).await;
    }

    /// Get cache statistics
    pub async fn stats(&self) -> CacheStats {
        self.cache.stats().await
    }
}

/// Specialized cache for workflow templates
pub struct WorkflowCache {
    cache: Cache<String, crate::Workflow>,
}

impl WorkflowCache {
    pub fn new() -> Self {
        // Workflow templates cached for 5 minutes with max 50 entries
        Self {
            cache: Cache::with_config(Duration::from_secs(300), 50),
        }
    }

    /// Get a cached workflow
    pub async fn get(&self, path: &str) -> Option<crate::Workflow> {
        self.cache.get(&path.to_string()).await
    }

    /// Cache a workflow
    pub async fn insert(&self, path: &str, workflow: crate::Workflow) {
        self.cache.insert(path.to_string(), workflow).await;
    }

    /// Clear the workflow cache
    pub async fn clear(&self) {
        self.cache.clear().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_basic_operations() {
        let cache: Cache<String, String> = Cache::with_config(Duration::from_secs(60), 10);

        // Insert and get
        cache.insert("key1".to_string(), "value1".to_string()).await;
        assert_eq!(cache.get(&"key1".to_string()).await, Some("value1".to_string()));

        // Miss
        assert_eq!(cache.get(&"key2".to_string()).await, None);

        // Stats
        let stats = cache.stats().await;
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
    }

    #[tokio::test]
    async fn test_cache_expiration() {
        let cache: Cache<String, String> = Cache::with_config(Duration::from_millis(50), 10);

        cache.insert("key1".to_string(), "value1".to_string()).await;
        
        // Should be in cache
        assert_eq!(cache.get(&"key1".to_string()).await, Some("value1".to_string()));

        // Wait for expiration
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Should be expired
        assert_eq!(cache.get(&"key1".to_string()).await, None);

        let stats = cache.stats().await;
        assert_eq!(stats.expirations, 1);
    }

    #[tokio::test]
    async fn test_cache_eviction() {
        let cache: Cache<i32, i32> = Cache::with_config(Duration::from_secs(60), 3);

        // Fill cache to max
        cache.insert(1, 1).await;
        cache.insert(2, 2).await;
        cache.insert(3, 3).await;

        // Access first entry to make it more recently used
        cache.get(&1).await;

        // This should evict entry 2 or 3 (LRU)
        cache.insert(4, 4).await;

        assert_eq!(cache.size().await, 3);

        let stats = cache.stats().await;
        assert_eq!(stats.evictions, 1);
    }
}