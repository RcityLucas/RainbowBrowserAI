// Advanced Caching System for Perception Module
// Implements multi-layer caching with TTL, LRU eviction, and intelligent invalidation

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use anyhow::{Result, Context};
use tracing::{debug, info, warn};

/// Multi-layer cache for perception data
pub struct PerceptionCache {
    /// Lightning cache - very short TTL for immediate re-scans
    lightning_cache: Arc<RwLock<LayerCache<LightningCacheEntry>>>,
    
    /// Quick cache - moderate TTL for interactive sessions
    quick_cache: Arc<RwLock<LayerCache<QuickCacheEntry>>>,
    
    /// Element cache - longer TTL for stable elements
    element_cache: Arc<RwLock<LayerCache<ElementCacheEntry>>>,
    
    /// Global stats for monitoring
    stats: Arc<RwLock<CacheStats>>,
}

/// Individual cache layer with LRU eviction
struct LayerCache<T> {
    entries: HashMap<String, CacheEntry<T>>,
    access_order: VecDeque<String>,
    max_entries: usize,
    max_memory_mb: usize,
    default_ttl: Duration,
}

/// Generic cache entry with metadata
#[derive(Clone)]
struct CacheEntry<T> {
    data: T,
    created_at: Instant,
    last_accessed: Instant,
    access_count: u32,
    ttl: Duration,
    size_bytes: usize,
}

/// Lightning perception cache entry
#[derive(Clone, Serialize, Deserialize)]
pub struct LightningCacheEntry {
    pub key_elements: Vec<(String, String)>, // (selector, text)
    pub page_status: PageStatus,
    pub urgent_signals: Vec<String>,
    pub url: String,
    pub timestamp: u64,
}

/// Quick perception cache entry
#[derive(Clone, Serialize, Deserialize)]
pub struct QuickCacheEntry {
    pub lightning_data: LightningCacheEntry,
    pub interactions: Vec<InteractionData>,
    pub layout: LayoutData,
    pub navigation: Vec<NavItem>,
    pub forms: Vec<FormData>,
}

/// Element-specific cache entry
#[derive(Clone, Serialize, Deserialize)]
pub struct ElementCacheEntry {
    pub selector: String,
    pub element_type: String,
    pub text: String,
    pub attributes: HashMap<String, String>,
    pub bounding_box: BoundingBox,
    pub interaction_hints: Vec<String>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PageStatus {
    pub is_loading: bool,
    pub has_errors: bool,
    pub ready_state: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InteractionData {
    pub selector: String,
    pub interaction_type: String,
    pub confidence: f32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct LayoutData {
    pub has_header: bool,
    pub has_navigation: bool,
    pub has_sidebar: bool,
    pub layout_type: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct NavItem {
    pub text: String,
    pub url: String,
    pub is_active: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct FormData {
    pub selector: String,
    pub field_count: usize,
    pub has_submit: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct BoundingBox {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// Cache statistics for monitoring
#[derive(Default, Debug, Clone)]
pub struct CacheStats {
    pub total_hits: u64,
    pub total_misses: u64,
    pub total_evictions: u64,
    pub total_invalidations: u64,
    pub current_memory_mb: usize,
    pub peak_memory_mb: usize,
}

impl PerceptionCache {
    /// Create a new perception cache with default settings
    pub fn new() -> Self {
        Self {
            lightning_cache: Arc::new(RwLock::new(LayerCache::new(
                100,  // max entries
                10,   // max memory MB
                Duration::from_millis(500), // 500ms TTL
            ))),
            quick_cache: Arc::new(RwLock::new(LayerCache::new(
                50,   // max entries
                20,   // max memory MB
                Duration::from_secs(5), // 5 second TTL
            ))),
            element_cache: Arc::new(RwLock::new(LayerCache::new(
                200,  // max entries
                30,   // max memory MB
                Duration::from_secs(30), // 30 second TTL
            ))),
            stats: Arc::new(RwLock::new(CacheStats::default())),
        }
    }

    /// Create cache with custom configuration
    pub fn with_config(config: CacheConfig) -> Self {
        Self {
            lightning_cache: Arc::new(RwLock::new(LayerCache::new(
                config.lightning_max_entries,
                config.lightning_max_memory_mb,
                config.lightning_ttl,
            ))),
            quick_cache: Arc::new(RwLock::new(LayerCache::new(
                config.quick_max_entries,
                config.quick_max_memory_mb,
                config.quick_ttl,
            ))),
            element_cache: Arc::new(RwLock::new(LayerCache::new(
                config.element_max_entries,
                config.element_max_memory_mb,
                config.element_ttl,
            ))),
            stats: Arc::new(RwLock::new(CacheStats::default())),
        }
    }

    /// Get Lightning perception data from cache
    pub async fn get_lightning(&self, key: &str) -> Option<LightningCacheEntry> {
        let mut cache = self.lightning_cache.write().await;
        let mut stats = self.stats.write().await;
        
        match cache.get(key) {
            Some(entry) => {
                stats.total_hits += 1;
                debug!("Lightning cache hit for key: {}", key);
                Some(entry)
            }
            None => {
                stats.total_misses += 1;
                debug!("Lightning cache miss for key: {}", key);
                None
            }
        }
    }

    /// Store Lightning perception data in cache
    pub async fn set_lightning(&self, key: String, data: LightningCacheEntry) {
        let size = estimate_size(&data);
        let mut cache = self.lightning_cache.write().await;
        let mut stats = self.stats.write().await;
        
        let evicted = cache.set(key.clone(), data, size);
        if evicted > 0 {
            stats.total_evictions += evicted as u64;
        }
        
        stats.current_memory_mb = cache.current_memory_mb();
        stats.peak_memory_mb = stats.peak_memory_mb.max(stats.current_memory_mb);
        
        info!("Cached Lightning data for key: {} (size: {} bytes)", key, size);
    }

    /// Get Quick perception data from cache
    pub async fn get_quick(&self, key: &str) -> Option<QuickCacheEntry> {
        let mut cache = self.quick_cache.write().await;
        let mut stats = self.stats.write().await;
        
        match cache.get(key) {
            Some(entry) => {
                stats.total_hits += 1;
                debug!("Quick cache hit for key: {}", key);
                Some(entry)
            }
            None => {
                stats.total_misses += 1;
                debug!("Quick cache miss for key: {}", key);
                None
            }
        }
    }

    /// Store Quick perception data in cache
    pub async fn set_quick(&self, key: String, data: QuickCacheEntry) {
        let size = estimate_size(&data);
        let mut cache = self.quick_cache.write().await;
        let mut stats = self.stats.write().await;
        
        let evicted = cache.set(key.clone(), data, size);
        if evicted > 0 {
            stats.total_evictions += evicted as u64;
        }
        
        stats.current_memory_mb = cache.current_memory_mb();
        stats.peak_memory_mb = stats.peak_memory_mb.max(stats.current_memory_mb);
        
        info!("Cached Quick data for key: {} (size: {} bytes)", key, size);
    }

    /// Get element data from cache
    pub async fn get_element(&self, key: &str) -> Option<ElementCacheEntry> {
        let mut cache = self.element_cache.write().await;
        let mut stats = self.stats.write().await;
        
        match cache.get(key) {
            Some(entry) => {
                stats.total_hits += 1;
                debug!("Element cache hit for key: {}", key);
                Some(entry)
            }
            None => {
                stats.total_misses += 1;
                debug!("Element cache miss for key: {}", key);
                None
            }
        }
    }

    /// Store element data in cache
    pub async fn set_element(&self, key: String, data: ElementCacheEntry) {
        let size = estimate_size(&data);
        let mut cache = self.element_cache.write().await;
        let mut stats = self.stats.write().await;
        
        let evicted = cache.set(key.clone(), data, size);
        if evicted > 0 {
            stats.total_evictions += evicted as u64;
        }
        
        stats.current_memory_mb = cache.current_memory_mb();
        stats.peak_memory_mb = stats.peak_memory_mb.max(stats.current_memory_mb);
        
        debug!("Cached element data for key: {} (size: {} bytes)", key, size);
    }

    /// Invalidate all caches for a specific URL
    pub async fn invalidate_url(&self, url: &str) {
        let mut lightning = self.lightning_cache.write().await;
        let mut quick = self.quick_cache.write().await;
        let mut element = self.element_cache.write().await;
        let mut stats = self.stats.write().await;
        
        let count = lightning.invalidate_prefix(url) + 
                   quick.invalidate_prefix(url) + 
                   element.invalidate_prefix(url);
        
        stats.total_invalidations += count as u64;
        info!("Invalidated {} cache entries for URL: {}", count, url);
    }

    /// Clear all expired entries across all caches
    pub async fn cleanup_expired(&self) {
        let mut lightning = self.lightning_cache.write().await;
        let mut quick = self.quick_cache.write().await;
        let mut element = self.element_cache.write().await;
        
        let expired = lightning.cleanup_expired() + 
                     quick.cleanup_expired() + 
                     element.cleanup_expired();
        
        if expired > 0 {
            info!("Cleaned up {} expired cache entries", expired);
        }
    }

    /// Get current cache statistics
    pub async fn get_stats(&self) -> CacheStats {
        self.stats.read().await.clone()
    }

    /// Clear all caches
    pub async fn clear_all(&self) {
        let mut lightning = self.lightning_cache.write().await;
        let mut quick = self.quick_cache.write().await;
        let mut element = self.element_cache.write().await;
        
        lightning.clear();
        quick.clear();
        element.clear();
        
        info!("Cleared all perception caches");
    }
}

impl<T: Clone> LayerCache<T> {
    fn new(max_entries: usize, max_memory_mb: usize, default_ttl: Duration) -> Self {
        Self {
            entries: HashMap::new(),
            access_order: VecDeque::new(),
            max_entries,
            max_memory_mb,
            default_ttl,
        }
    }

    fn get(&mut self, key: &str) -> Option<T> {
        if let Some(entry) = self.entries.get_mut(key) {
            // Check if expired
            if entry.created_at.elapsed() > entry.ttl {
                self.entries.remove(key);
                self.access_order.retain(|k| k != key);
                return None;
            }
            
            // Update access metadata
            entry.last_accessed = Instant::now();
            entry.access_count += 1;
            
            // Move to front of access order
            self.access_order.retain(|k| k != key);
            self.access_order.push_front(key.to_string());
            
            Some(entry.data.clone())
        } else {
            None
        }
    }

    fn set(&mut self, key: String, data: T, size_bytes: usize) -> usize {
        let mut evicted = 0;
        
        // Check memory limit
        while self.current_memory_bytes() + size_bytes > self.max_memory_mb * 1024 * 1024 {
            if let Some(oldest_key) = self.access_order.pop_back() {
                self.entries.remove(&oldest_key);
                evicted += 1;
            } else {
                break;
            }
        }
        
        // Check entry limit
        while self.entries.len() >= self.max_entries {
            if let Some(oldest_key) = self.access_order.pop_back() {
                self.entries.remove(&oldest_key);
                evicted += 1;
            } else {
                break;
            }
        }
        
        // Insert new entry
        let entry = CacheEntry {
            data,
            created_at: Instant::now(),
            last_accessed: Instant::now(),
            access_count: 1,
            ttl: self.default_ttl,
            size_bytes,
        };
        
        self.entries.insert(key.clone(), entry);
        self.access_order.push_front(key);
        
        evicted
    }

    fn invalidate_prefix(&mut self, prefix: &str) -> usize {
        let keys_to_remove: Vec<String> = self.entries
            .keys()
            .filter(|k| k.starts_with(prefix))
            .cloned()
            .collect();
        
        let count = keys_to_remove.len();
        for key in keys_to_remove {
            self.entries.remove(&key);
            self.access_order.retain(|k| k != &key);
        }
        
        count
    }

    fn cleanup_expired(&mut self) -> usize {
        let now = Instant::now();
        let expired_keys: Vec<String> = self.entries
            .iter()
            .filter(|(_, entry)| now.duration_since(entry.created_at) > entry.ttl)
            .map(|(key, _)| key.clone())
            .collect();
        
        let count = expired_keys.len();
        for key in expired_keys {
            self.entries.remove(&key);
            self.access_order.retain(|k| k != &key);
        }
        
        count
    }

    fn current_memory_bytes(&self) -> usize {
        self.entries.values().map(|e| e.size_bytes).sum()
    }

    fn current_memory_mb(&self) -> usize {
        self.current_memory_bytes() / (1024 * 1024)
    }

    fn clear(&mut self) {
        self.entries.clear();
        self.access_order.clear();
    }
}

/// Cache configuration
#[derive(Clone)]
pub struct CacheConfig {
    pub lightning_max_entries: usize,
    pub lightning_max_memory_mb: usize,
    pub lightning_ttl: Duration,
    
    pub quick_max_entries: usize,
    pub quick_max_memory_mb: usize,
    pub quick_ttl: Duration,
    
    pub element_max_entries: usize,
    pub element_max_memory_mb: usize,
    pub element_ttl: Duration,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            lightning_max_entries: 100,
            lightning_max_memory_mb: 10,
            lightning_ttl: Duration::from_millis(500),
            
            quick_max_entries: 50,
            quick_max_memory_mb: 20,
            quick_ttl: Duration::from_secs(5),
            
            element_max_entries: 200,
            element_max_memory_mb: 30,
            element_ttl: Duration::from_secs(30),
        }
    }
}

/// Estimate size of data in bytes (simplified)
fn estimate_size<T: Serialize>(data: &T) -> usize {
    serde_json::to_vec(data).map(|v| v.len()).unwrap_or(1024)
}

/// Background task for periodic cache maintenance
pub async fn cache_maintenance_task(cache: Arc<PerceptionCache>) {
    let mut interval = tokio::time::interval(Duration::from_secs(60));
    
    loop {
        interval.tick().await;
        
        // Cleanup expired entries
        cache.cleanup_expired().await;
        
        // Log statistics
        let stats = cache.get_stats().await;
        let hit_rate = if stats.total_hits + stats.total_misses > 0 {
            (stats.total_hits as f64) / ((stats.total_hits + stats.total_misses) as f64) * 100.0
        } else {
            0.0
        };
        
        info!(
            "Cache stats - Hit rate: {:.1}%, Memory: {}MB, Evictions: {}",
            hit_rate,
            stats.current_memory_mb,
            stats.total_evictions
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_basic_operations() {
        let cache = PerceptionCache::new();
        
        // Test Lightning cache
        let lightning_data = LightningCacheEntry {
            key_elements: vec![("button".to_string(), "Submit".to_string())],
            page_status: PageStatus {
                is_loading: false,
                has_errors: false,
                ready_state: "complete".to_string(),
            },
            urgent_signals: vec![],
            url: "https://example.com".to_string(),
            timestamp: 123456,
        };
        
        cache.set_lightning("test_key".to_string(), lightning_data.clone()).await;
        let retrieved = cache.get_lightning("test_key").await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().url, "https://example.com");
    }

    #[tokio::test]
    async fn test_cache_expiration() {
        let config = CacheConfig {
            lightning_ttl: Duration::from_millis(100),
            ..Default::default()
        };
        let cache = PerceptionCache::with_config(config);
        
        let lightning_data = LightningCacheEntry {
            key_elements: vec![],
            page_status: PageStatus {
                is_loading: false,
                has_errors: false,
                ready_state: "complete".to_string(),
            },
            urgent_signals: vec![],
            url: "https://example.com".to_string(),
            timestamp: 123456,
        };
        
        cache.set_lightning("expire_test".to_string(), lightning_data).await;
        
        // Should exist immediately
        assert!(cache.get_lightning("expire_test").await.is_some());
        
        // Wait for expiration
        tokio::time::sleep(Duration::from_millis(150)).await;
        
        // Should be expired
        assert!(cache.get_lightning("expire_test").await.is_none());
    }

    #[tokio::test]
    async fn test_cache_invalidation() {
        let cache = PerceptionCache::new();
        
        // Add multiple entries
        for i in 0..5 {
            let data = LightningCacheEntry {
                key_elements: vec![],
                page_status: PageStatus {
                    is_loading: false,
                    has_errors: false,
                    ready_state: "complete".to_string(),
                },
                urgent_signals: vec![],
                url: format!("https://example.com/page{}", i),
                timestamp: i as u64,
            };
            cache.set_lightning(format!("https://example.com/page{}", i), data).await;
        }
        
        // Invalidate by prefix
        cache.invalidate_url("https://example.com").await;
        
        // All should be gone
        for i in 0..5 {
            assert!(cache.get_lightning(&format!("https://example.com/page{}", i)).await.is_none());
        }
    }
}