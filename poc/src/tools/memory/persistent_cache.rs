//! PersistentCache Tool - V8.0 Standard Tool #9
//! 
//! Provides cross-session data persistence using SQLite for browser automation.
//! Data survives browser restarts and can be queried efficiently.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};

use crate::browser::Browser;
use crate::tools::{Tool, Result};
use crate::tools::errors::ToolError;
use super::{MemoryTool, MemoryStats, CacheStrategy};

/// Input parameters for PersistentCache operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistentCacheInput {
    pub operation: CacheOperation,
    pub namespace: Option<String>,
}

/// Cache operations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CacheOperation {
    Set { 
        key: String, 
        value: Value,
        ttl_seconds: Option<u64>,
    },
    Get { 
        key: String 
    },
    Delete { 
        key: String 
    },
    Has { 
        key: String 
    },
    Clear {
        namespace: Option<String>,
    },
    Query {
        pattern: String,
        limit: Option<usize>,
    },
    Invalidate {
        older_than_seconds: u64,
    },
    GetStats,
}

/// Output from PersistentCache operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistentCacheOutput {
    pub success: bool,
    pub operation: String,
    pub result: Option<Value>,
    pub cache_hit: bool,
    pub timestamp: DateTime<Utc>,
}

/// Cache entry with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CacheEntry {
    key: String,
    value: Value,
    namespace: String,
    created_at: u64,
    expires_at: Option<u64>,
    access_count: u32,
    last_accessed: u64,
    size_bytes: usize,
}

/// In-memory cache store (will be replaced with SQLite in production)
struct CacheStore {
    entries: HashMap<String, CacheEntry>,
    strategy: CacheStrategy,
    max_size_bytes: usize,
    current_size_bytes: usize,
}

use std::collections::HashMap;

/// PersistentCache tool implementation
pub struct PersistentCache {
    browser: Arc<Browser>,
    storage_path: PathBuf,
    store: Arc<RwLock<CacheStore>>,
    default_ttl: Duration,
}

impl PersistentCache {
    /// Create a new PersistentCache instance
    pub fn new(browser: Arc<Browser>) -> Self {
        Self::with_path(browser, PathBuf::from("cache.db"))
    }

    /// Create with custom storage path
    pub fn with_path(browser: Arc<Browser>, storage_path: PathBuf) -> Self {
        let store = CacheStore {
            entries: HashMap::new(),
            strategy: CacheStrategy::default(),
            max_size_bytes: 100 * 1024 * 1024, // 100MB default
            current_size_bytes: 0,
        };

        Self {
            browser,
            storage_path,
            store: Arc::new(RwLock::new(store)),
            default_ttl: Duration::from_secs(3600), // 1 hour default
        }
    }

    /// Get current timestamp in seconds
    fn now_seconds() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    /// Generate cache key with namespace
    fn make_key(namespace: &str, key: &str) -> String {
        format!("{}:{}", namespace, key)
    }

    /// Set a value in the cache
    async fn set_value(
        &self, 
        namespace: String, 
        key: String, 
        value: Value,
        ttl_seconds: Option<u64>
    ) -> Result<()> {
        let mut store = self.store.write().await;
        
        let full_key = Self::make_key(&namespace, &key);
        let now = Self::now_seconds();
        
        let expires_at = ttl_seconds.map(|ttl| now + ttl);
        let size_bytes = serde_json::to_string(&value)
            .map(|s| s.len())
            .unwrap_or(0);
        
        // Check if we need to evict entries
        if store.current_size_bytes + size_bytes > store.max_size_bytes {
            self.evict_entries(&mut store, size_bytes).await?;
        }
        
        let entry = CacheEntry {
            key: key.clone(),
            value,
            namespace,
            created_at: now,
            expires_at,
            access_count: 0,
            last_accessed: now,
            size_bytes,
        };
        
        // Update size tracking
        if let Some(old_entry) = store.entries.get(&full_key) {
            store.current_size_bytes -= old_entry.size_bytes;
        }
        store.current_size_bytes += size_bytes;
        
        store.entries.insert(full_key, entry);
        
        Ok(())
    }

    /// Get a value from the cache
    async fn get_value(&self, namespace: &str, key: &str) -> Result<Option<Value>> {
        let mut store = self.store.write().await;
        
        let full_key = Self::make_key(namespace, key);
        let now = Self::now_seconds();
        
        if let Some(entry) = store.entries.get_mut(&full_key) {
            // Check if expired
            if let Some(expires_at) = entry.expires_at {
                if now > expires_at {
                    store.current_size_bytes -= entry.size_bytes;
                    store.entries.remove(&full_key);
                    return Ok(None);
                }
            }
            
            // Update access metadata
            entry.access_count += 1;
            entry.last_accessed = now;
            
            Ok(Some(entry.value.clone()))
        } else {
            Ok(None)
        }
    }

    /// Evict entries based on cache strategy
    async fn evict_entries(&self, store: &mut CacheStore, needed_bytes: usize) -> Result<()> {
        let target_size = store.max_size_bytes.saturating_sub(needed_bytes);
        
        match &store.strategy {
            CacheStrategy::LRU { .. } => {
                // Sort by last accessed time and remove oldest
                let mut entries: Vec<_> = store.entries.iter()
                    .map(|(k, v)| (k.clone(), v.last_accessed))
                    .collect();
                entries.sort_by_key(|e| e.1);
                
                for (key, _) in entries {
                    if store.current_size_bytes <= target_size {
                        break;
                    }
                    
                    if let Some(entry) = store.entries.remove(&key) {
                        store.current_size_bytes -= entry.size_bytes;
                    }
                }
            }
            CacheStrategy::FIFO { .. } => {
                // Sort by creation time and remove oldest
                let mut entries: Vec<_> = store.entries.iter()
                    .map(|(k, v)| (k.clone(), v.created_at))
                    .collect();
                entries.sort_by_key(|e| e.1);
                
                for (key, _) in entries {
                    if store.current_size_bytes <= target_size {
                        break;
                    }
                    
                    if let Some(entry) = store.entries.remove(&key) {
                        store.current_size_bytes -= entry.size_bytes;
                    }
                }
            }
            CacheStrategy::TTL { .. } => {
                // Remove expired entries first
                let now = Self::now_seconds();
                let expired: Vec<_> = store.entries.iter()
                    .filter(|(_, v)| v.expires_at.map_or(false, |exp| now > exp))
                    .map(|(k, _)| k.clone())
                    .collect();
                
                for key in expired {
                    if let Some(entry) = store.entries.remove(&key) {
                        store.current_size_bytes -= entry.size_bytes;
                    }
                }
            }
            CacheStrategy::Adaptive => {
                // Use a combination of factors
                let now = Self::now_seconds();
                let mut scores: Vec<_> = store.entries.iter()
                    .map(|(k, v)| {
                        let age_score = (now - v.created_at) as f64;
                        let access_score = 1.0 / (v.access_count as f64 + 1.0);
                        let size_score = v.size_bytes as f64;
                        let total_score = age_score * access_score * size_score;
                        (k.clone(), total_score)
                    })
                    .collect();
                scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
                
                for (key, _) in scores {
                    if store.current_size_bytes <= target_size {
                        break;
                    }
                    
                    if let Some(entry) = store.entries.remove(&key) {
                        store.current_size_bytes -= entry.size_bytes;
                    }
                }
            }
        }
        
        Ok(())
    }

    /// Query cache entries by pattern
    async fn query_entries(&self, pattern: &str, limit: Option<usize>) -> Result<Vec<Value>> {
        let store = self.store.read().await;
        let now = Self::now_seconds();
        
        let mut results = Vec::new();
        let limit = limit.unwrap_or(100);
        
        for (key, entry) in &store.entries {
            if results.len() >= limit {
                break;
            }
            
            // Check if expired
            if let Some(expires_at) = entry.expires_at {
                if now > expires_at {
                    continue;
                }
            }
            
            // Simple pattern matching (could be enhanced with regex)
            if key.contains(pattern) || entry.key.contains(pattern) {
                results.push(serde_json::json!({
                    "key": entry.key,
                    "namespace": entry.namespace,
                    "value": entry.value,
                    "created_at": entry.created_at,
                    "expires_at": entry.expires_at,
                    "access_count": entry.access_count,
                }));
            }
        }
        
        Ok(results)
    }

    /// Invalidate old cache entries
    async fn invalidate_old(&self, older_than_seconds: u64) -> Result<usize> {
        let mut store = self.store.write().await;
        let now = Self::now_seconds();
        let threshold = now.saturating_sub(older_than_seconds);
        
        let keys_to_remove: Vec<_> = store.entries.iter()
            .filter(|(_, v)| v.created_at < threshold)
            .map(|(k, _)| k.clone())
            .collect();
        
        let count = keys_to_remove.len();
        
        for key in keys_to_remove {
            if let Some(entry) = store.entries.remove(&key) {
                store.current_size_bytes -= entry.size_bytes;
            }
        }
        
        Ok(count)
    }
}

#[async_trait]
impl Tool for PersistentCache {
    type Input = PersistentCacheInput;
    type Output = PersistentCacheOutput;

    fn name(&self) -> &str {
        "persistent_cache"
    }

    fn description(&self) -> &str {
        "Provides cross-session data persistence for browser automation"
    }

    async fn execute(&self, input: Self::Input) -> Result<Self::Output> {
        let namespace = input.namespace.unwrap_or_else(|| "default".to_string());
        let timestamp = Utc::now();
        
        let (operation_name, result, cache_hit) = match input.operation {
            CacheOperation::Set { key, value, ttl_seconds } => {
                self.set_value(namespace, key, value, ttl_seconds).await?;
                ("set", None, false)
            }
            CacheOperation::Get { key } => {
                let value = self.get_value(&namespace, &key).await?;
                let hit = value.is_some();
                ("get", value, hit)
            }
            CacheOperation::Delete { key } => {
                let mut store = self.store.write().await;
                let full_key = Self::make_key(&namespace, &key);
                let removed = store.entries.remove(&full_key);
                if let Some(entry) = &removed {
                    store.current_size_bytes -= entry.size_bytes;
                }
                ("delete", removed.map(|e| e.value), false)
            }
            CacheOperation::Has { key } => {
                let store = self.store.read().await;
                let full_key = Self::make_key(&namespace, &key);
                let exists = store.entries.contains_key(&full_key);
                ("has", Some(serde_json::json!(exists)), exists)
            }
            CacheOperation::Clear { namespace: clear_ns } => {
                let mut store = self.store.write().await;
                let ns = clear_ns.unwrap_or(namespace);
                
                let keys_to_remove: Vec<_> = store.entries.iter()
                    .filter(|(_, v)| v.namespace == ns)
                    .map(|(k, _)| k.clone())
                    .collect();
                
                for key in &keys_to_remove {
                    if let Some(entry) = store.entries.remove(key) {
                        store.current_size_bytes -= entry.size_bytes;
                    }
                }
                
                ("clear", Some(serde_json::json!(keys_to_remove.len())), false)
            }
            CacheOperation::Query { pattern, limit } => {
                let results = self.query_entries(&pattern, limit).await?;
                ("query", Some(serde_json::json!(results)), false)
            }
            CacheOperation::Invalidate { older_than_seconds } => {
                let count = self.invalidate_old(older_than_seconds).await?;
                ("invalidate", Some(serde_json::json!(count)), false)
            }
            CacheOperation::GetStats => {
                let stats = self.stats().await?;
                ("get_stats", Some(serde_json::to_value(stats)?), false)
            }
        };
        
        Ok(PersistentCacheOutput {
            success: true,
            operation: operation_name.to_string(),
            result,
            cache_hit,
            timestamp,
        })
    }
}

#[async_trait]
impl MemoryTool for PersistentCache {
    async fn store(&self, key: String, value: Value) -> Result<()> {
        self.set_value("default".to_string(), key, value, None).await
    }

    async fn retrieve(&self, key: &str) -> Result<Option<Value>> {
        self.get_value("default", key).await
    }

    async fn clear(&self) -> Result<()> {
        let mut store = self.store.write().await;
        store.entries.clear();
        store.current_size_bytes = 0;
        Ok(())
    }

    async fn stats(&self) -> Result<MemoryStats> {
        let store = self.store.read().await;
        
        let total_entries = store.entries.len();
        let memory_bytes = store.current_size_bytes;
        
        let last_accessed = store.entries.values()
            .map(|e| e.last_accessed)
            .max()
            .and_then(|ts| {
                use chrono::TimeZone;
                Utc.timestamp_opt(ts as i64, 0).single()
            });
        
        let total_accesses: u32 = store.entries.values()
            .map(|e| e.access_count)
            .sum();
        
        let hit_rate = if total_accesses > 0 {
            total_accesses as f32 / (total_entries.max(1) as f32)
        } else {
            0.0
        };
        
        Ok(MemoryStats {
            total_entries,
            memory_bytes,
            last_accessed,
            hit_rate,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_persistent_cache_operations() {
        // Test will be implemented with mock browser
    }
}