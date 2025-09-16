// Unified Cache System for Cross-Module Coordination
// Provides intelligent caching with event-driven invalidation

use anyhow::Result;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{Duration, Instant};
use tracing::{debug, info};

use super::events::{EventBus, Event, EventType};

/// Unified cache for all modules
pub struct UnifiedCache {
    browser_cache: Arc<BrowserCache>,
    perception_cache: Arc<PerceptionCache>,
    tool_cache: Arc<ToolCache>,
    coordinator: Arc<CacheCoordinator>,
}

impl UnifiedCache {
    pub async fn new(event_bus: Arc<EventBus>) -> Result<Self> {
        let coordinator = Arc::new(CacheCoordinator::new(event_bus).await);
        
        Ok(Self {
            browser_cache: Arc::new(BrowserCache::new()),
            perception_cache: Arc::new(PerceptionCache::new()),
            tool_cache: Arc::new(ToolCache::new()),
            coordinator,
        })
    }
    
    pub async fn get_stats(&self) -> CacheStats {
        CacheStats {
            browser_stats: self.browser_cache.get_stats().await,
            perception_stats: self.perception_cache.get_stats().await,
            tool_stats: self.tool_cache.get_stats().await,
        }
    }
    
    /// Invalidate cache entries matching a pattern
    pub async fn invalidate_by_pattern(&self, pattern: &str) {
        // Invalidate matching entries in each cache
        self.browser_cache.invalidate_by_pattern(pattern).await;
        self.perception_cache.invalidate_by_pattern(pattern).await;
        self.tool_cache.invalidate_by_pattern(pattern).await;
    }
}

/// Cache coordinator for intelligent invalidation
pub struct CacheCoordinator {
    invalidation_rules: HashMap<EventType, Vec<InvalidationRule>>,
    event_bus: Arc<EventBus>,
}

impl CacheCoordinator {
    pub async fn new(event_bus: Arc<EventBus>) -> Self {
        let mut coordinator = Self {
            invalidation_rules: Self::build_invalidation_rules(),
            event_bus: event_bus.clone(),
        };
        
        // Subscribe to events that require cache invalidation
        coordinator.setup_event_subscriptions(event_bus).await;
        
        coordinator
    }
    
    fn build_invalidation_rules() -> HashMap<EventType, Vec<InvalidationRule>> {
        let mut rules = HashMap::new();
        
        // Navigation invalidates perception and some tool caches
        rules.insert(EventType::NavigationCompleted, vec![
            InvalidationRule::InvalidateAll(CacheType::PerceptionElements),
            InvalidationRule::InvalidateAll(CacheType::BrowserScreenshots),
            InvalidationRule::Selective(CacheType::ToolResults, |key| {
                key.contains("element_") || key.contains("page_")
            }),
        ]);
        
        // Page content changes invalidate element caches
        rules.insert(EventType::PageContentChanged, vec![
            InvalidationRule::InvalidateAll(CacheType::PerceptionElements),
            InvalidationRule::Selective(CacheType::ToolResults, |key| {
                key.contains("extract_") || key.contains("analyze_")
            }),
        ]);
        
        rules
    }
    
    async fn setup_event_subscriptions(&mut self, event_bus: Arc<EventBus>) {
        // For now, we'll handle events through the event bus emit mechanism
        // The subscription system needs proper async handler implementation
        // which is complex with the current trait design
        
        // TODO: Implement proper event subscription with async handlers
        // This would require creating handler structs that implement EventHandler trait
    }
    
    pub async fn invalidate_navigation_sensitive(&self, _new_url: &str) -> Result<()> {
        // Emit cache invalidation event
        self.event_bus.emit(Event::CacheInvalidated {
            cache_type: "navigation_sensitive".to_string(),
            reason: "navigation".to_string(),
            keys_affected: vec![],
            timestamp: Instant::now(),
        }).await?;
        
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum InvalidationRule {
    InvalidateAll(CacheType),
    Selective(CacheType, fn(&str) -> bool),
}

#[derive(Debug, Clone, Copy)]
pub enum CacheType {
    BrowserScreenshots,
    PerceptionElements,
    ToolResults,
}

/// Browser-specific cache
pub struct BrowserCache {
    screenshots: Arc<RwLock<HashMap<String, CachedScreenshot>>>,
    page_data: Arc<RwLock<HashMap<String, CachedPageData>>>,
}

impl BrowserCache {
    pub fn new() -> Self {
        Self {
            screenshots: Arc::new(RwLock::new(HashMap::new())),
            page_data: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    pub async fn get_stats(&self) -> ComponentCacheStats {
        ComponentCacheStats {
            entries: self.screenshots.read().await.len() + self.page_data.read().await.len(),
            hits: 0,
            misses: 0,
            size_bytes: 0,
        }
    }
    
    pub async fn invalidate_by_pattern(&self, pattern: &str) {
        // Simple pattern matching - in real implementation would be more sophisticated
        if pattern.contains("*") {
            // Clear all if pattern is wildcard
            self.screenshots.write().await.clear();
            self.page_data.write().await.clear();
        }
    }
}

/// Perception-specific cache
pub struct PerceptionCache {
    elements: Arc<RwLock<HashMap<String, CachedElement>>>,
    analyses: Arc<RwLock<HashMap<String, CachedAnalysis>>>,
}

impl PerceptionCache {
    pub fn new() -> Self {
        Self {
            elements: Arc::new(RwLock::new(HashMap::new())),
            analyses: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    pub async fn get_stats(&self) -> ComponentCacheStats {
        ComponentCacheStats {
            entries: self.elements.read().await.len() + self.analyses.read().await.len(),
            hits: 0,
            misses: 0,
            size_bytes: 0,
        }
    }
    
    pub async fn invalidate_by_pattern(&self, pattern: &str) {
        if pattern.contains("*") {
            self.elements.write().await.clear();
            self.analyses.write().await.clear();
        }
    }
}

/// Tool-specific cache
pub struct ToolCache {
    results: Arc<RwLock<HashMap<String, CachedToolResult>>>,
    metrics: Arc<RwLock<CacheMetrics>>,
}

impl ToolCache {
    pub fn new() -> Self {
        Self {
            results: Arc::new(RwLock::new(HashMap::new())),
            metrics: Arc::new(RwLock::new(CacheMetrics::default())),
        }
    }
    
    pub async fn get_stats(&self) -> ComponentCacheStats {
        let metrics = self.metrics.read().await;
        ComponentCacheStats {
            entries: self.results.read().await.len(),
            hits: metrics.hits,
            misses: metrics.misses,
            size_bytes: 0,
        }
    }
    
    pub async fn invalidate_by_pattern(&self, pattern: &str) {
        if pattern.contains("*") {
            self.results.write().await.clear();
        }
    }
}

// Cache entry types

#[derive(Debug, Clone)]
struct CachedScreenshot {
    data: Vec<u8>,
    format: String,
    cached_at: Instant,
    url: String,
}

#[derive(Debug, Clone)]
struct CachedPageData {
    title: String,
    html: String,
    cached_at: Instant,
}

#[derive(Debug, Clone)]
struct CachedElement {
    selector: String,
    element_type: String,
    cached_at: Instant,
    confidence: f64,
}

#[derive(Debug, Clone)]
struct CachedAnalysis {
    analysis_type: String,
    result: serde_json::Value,
    cached_at: Instant,
}

#[derive(Debug, Clone)]
struct CachedToolResult {
    tool_name: String,
    input_hash: String,
    result: serde_json::Value,
    cached_at: Instant,
}

#[derive(Debug, Default)]
struct CacheMetrics {
    hits: u64,
    misses: u64,
}

// Public cache statistics

#[derive(Debug, Clone)]
pub struct CacheStats {
    pub browser_stats: ComponentCacheStats,
    pub perception_stats: ComponentCacheStats,
    pub tool_stats: ComponentCacheStats,
}

#[derive(Debug, Clone)]
pub struct ComponentCacheStats {
    pub entries: usize,
    pub hits: u64,
    pub misses: u64,
    pub size_bytes: usize,
}