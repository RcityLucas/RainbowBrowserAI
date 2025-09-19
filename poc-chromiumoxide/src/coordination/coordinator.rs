// Central Coordinator for RainbowBrowserAI
// Manages session lifecycle, module coordination, and resource management

use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{error, info, warn};
use uuid::Uuid;

use super::cache::UnifiedCache;
use super::events::{Event, EventBus};
use super::monitoring::UnifiedMonitoring;
use super::session::{SessionBundle, SessionContext};
use super::state::UnifiedStateManager;
use crate::browser::pool::BrowserPool;

/// Resource manager for browser instances
pub struct ResourceManager {
    browser_pool: Arc<BrowserPool>,
    active_browsers: Arc<RwLock<HashMap<String, BrowserLease>>>,
    resource_limits: ResourceLimits,
}

#[derive(Debug, Clone)]
pub struct ResourceLimits {
    pub max_sessions: usize,
    pub max_browsers: usize,
    pub max_memory_mb: usize,
    pub max_cpu_percent: f64,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_sessions: 10,
            max_browsers: 5,
            max_memory_mb: 2048,
            max_cpu_percent: 80.0,
        }
    }
}

#[allow(dead_code)]
struct BrowserLease {
    session_id: String,
    browser: Arc<crate::browser::Browser>,
    leased_at: Instant,
}

impl ResourceManager {
    pub fn new(browser_pool: Arc<BrowserPool>) -> Self {
        Self {
            browser_pool,
            active_browsers: Arc::new(RwLock::new(HashMap::new())),
            resource_limits: ResourceLimits::default(),
        }
    }

    pub async fn acquire_browser(&self, session_id: &str) -> Result<Arc<crate::browser::Browser>> {
        // Check resource limits
        let active_count = self.active_browsers.read().await.len();
        if active_count >= self.resource_limits.max_browsers {
            return Err(anyhow!(
                "Browser limit reached: {}/{}",
                active_count,
                self.resource_limits.max_browsers
            ));
        }

        // Acquire browser from pool
        let browser_guard = self.browser_pool.acquire().await?;
        let browser = browser_guard.browser_arc();

        // Store lease
        let lease = BrowserLease {
            session_id: session_id.to_string(),
            browser: browser.clone(),
            leased_at: Instant::now(),
        };
        self.active_browsers
            .write()
            .await
            .insert(session_id.to_string(), lease);

        Ok(browser)
    }

    pub async fn release_browser(&self, session_id: &str) -> Result<()> {
        self.active_browsers.write().await.remove(session_id);
        Ok(())
    }
}

/// Module registry for managing available modules
pub struct ModuleRegistry {
    modules: Arc<RwLock<HashMap<String, ModuleInfo>>>,
}

#[derive(Debug, Clone)]
pub struct ModuleInfo {
    pub name: String,
    pub module_type: super::ModuleType,
    pub version: String,
    pub enabled: bool,
}

impl ModuleRegistry {
    pub fn new() -> Self {
        let mut modules = HashMap::new();

        // Register core modules
        modules.insert(
            "perception".to_string(),
            ModuleInfo {
                name: "perception".to_string(),
                module_type: super::ModuleType::Perception,
                version: "1.0.0".to_string(),
                enabled: true,
            },
        );

        modules.insert(
            "tools".to_string(),
            ModuleInfo {
                name: "tools".to_string(),
                module_type: super::ModuleType::Tools,
                version: "1.0.0".to_string(),
                enabled: true,
            },
        );

        modules.insert(
            "intelligence".to_string(),
            ModuleInfo {
                name: "intelligence".to_string(),
                module_type: super::ModuleType::Intelligence,
                version: "1.0.0".to_string(),
                enabled: true,
            },
        );

        Self {
            modules: Arc::new(RwLock::new(modules)),
        }
    }

    pub async fn get_enabled_modules(&self) -> Vec<ModuleInfo> {
        self.modules
            .read()
            .await
            .values()
            .filter(|m| m.enabled)
            .cloned()
            .collect()
    }
}

impl Default for ModuleRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Central coordinator for the entire system
pub struct RainbowCoordinator {
    event_bus: Arc<EventBus>,
    state_manager: Arc<UnifiedStateManager>,
    resource_manager: Arc<ResourceManager>,
    module_registry: Arc<ModuleRegistry>,
    monitoring: Arc<UnifiedMonitoring>,
    cache: Arc<UnifiedCache>,
    session_contexts: Arc<RwLock<HashMap<String, Arc<SessionContext>>>>,
    session_bundles: Arc<RwLock<HashMap<String, Arc<SessionBundle>>>>,
    config: CoordinatorConfig,
}

#[derive(Debug, Clone)]
pub struct CoordinatorConfig {
    pub session_timeout: Duration,
    pub cleanup_interval: Duration,
    pub enable_monitoring: bool,
    pub enable_auto_cleanup: bool,
}

impl Default for CoordinatorConfig {
    fn default() -> Self {
        Self {
            session_timeout: Duration::from_secs(1800), // 30 minutes
            cleanup_interval: Duration::from_secs(60),  // 1 minute
            enable_monitoring: true,
            enable_auto_cleanup: true,
        }
    }
}

impl RainbowCoordinator {
    /// Create a new coordinator
    pub async fn new(browser_pool: Arc<BrowserPool>) -> Result<Self> {
        let event_bus = Arc::new(EventBus::new());
        let state_manager = Arc::new(UnifiedStateManager::new(event_bus.clone()).await?);
        let resource_manager = Arc::new(ResourceManager::new(browser_pool));
        let module_registry = Arc::new(ModuleRegistry::new());
        let cache = Arc::new(UnifiedCache::new(event_bus.clone()).await?);
        let monitoring = Arc::new(UnifiedMonitoring::new(event_bus.clone()).await?);

        let coordinator = Self {
            event_bus,
            state_manager,
            resource_manager,
            module_registry,
            monitoring,
            cache,
            session_contexts: Arc::new(RwLock::new(HashMap::new())),
            session_bundles: Arc::new(RwLock::new(HashMap::new())),
            config: CoordinatorConfig::default(),
        };

        // Start background tasks
        if coordinator.config.enable_auto_cleanup {
            coordinator.start_cleanup_task();
        }

        if coordinator.config.enable_monitoring {
            coordinator.start_monitoring_task();
        }

        info!("RainbowCoordinator initialized successfully");

        Ok(coordinator)
    }

    /// Create a new session with coordinated modules
    pub async fn create_session(&self) -> Result<Arc<SessionBundle>> {
        let session_id = Uuid::new_v4().to_string();
        info!("Creating new session: {}", session_id);

        // Acquire browser for session
        let browser = self.resource_manager.acquire_browser(&session_id).await?;

        // Create session context
        let context = Arc::new(
            SessionContext::new(
                session_id.clone(),
                browser,
                self.event_bus.clone(),
                self.state_manager.clone(),
                self.cache.clone(),
            )
            .await?,
        );

        // Create session bundle with all modules
        let bundle = Arc::new(SessionBundle::new(context.clone()).await?);

        // Store session
        self.session_contexts
            .write()
            .await
            .insert(session_id.clone(), context);
        self.session_bundles
            .write()
            .await
            .insert(session_id.clone(), bundle.clone());

        // Emit session created event
        self.event_bus
            .emit(Event::SessionCreated {
                session_id: session_id.clone(),
                timestamp: Instant::now(),
            })
            .await?;

        info!("Session {} created successfully", session_id);

        Ok(bundle)
    }

    /// Get an existing session bundle
    pub async fn get_session(&self, session_id: &str) -> Option<Arc<SessionBundle>> {
        self.session_bundles.read().await.get(session_id).cloned()
    }

    /// Get or create a session
    pub async fn get_or_create_session(
        &self,
        session_id: Option<String>,
    ) -> Result<Arc<SessionBundle>> {
        match session_id {
            Some(id) => {
                // Try to get existing session
                if let Some(bundle) = self.get_session(&id).await {
                    // Check if session is still valid
                    if !bundle.context.is_timed_out().await {
                        bundle.context.touch().await;
                        return Ok(bundle);
                    } else {
                        // Session timed out, clean it up
                        warn!("Session {} has timed out, removing", id);
                        self.remove_session(&id).await?;
                    }
                }
                // Session doesn't exist or timed out, create new one
                self.create_session().await
            }
            None => {
                // No session ID provided, create new session
                self.create_session().await
            }
        }
    }

    /// Remove a session and cleanup resources
    pub async fn remove_session(&self, session_id: &str) -> Result<()> {
        info!("Removing session: {}", session_id);

        // Get and cleanup bundle
        if let Some(bundle) = self.session_bundles.write().await.remove(session_id) {
            bundle.cleanup().await?;
        }

        // Remove context
        self.session_contexts.write().await.remove(session_id);

        // Release browser
        self.resource_manager.release_browser(session_id).await?;

        // Emit session closed event
        self.event_bus
            .emit(Event::SessionClosed {
                session_id: session_id.to_string(),
                reason: "removed".to_string(),
                timestamp: Instant::now(),
            })
            .await?;

        Ok(())
    }

    /// List all active sessions
    pub async fn list_sessions(&self) -> Vec<SessionInfo> {
        let bundles = self.session_bundles.read().await;
        let mut sessions = Vec::new();

        for (id, bundle) in bundles.iter() {
            let context = &bundle.context;
            let resource_usage = context.get_resource_usage().await;

            sessions.push(SessionInfo {
                session_id: id.clone(),
                created_at: context.created_at,
                last_activity: *context.last_activity.read().await,
                is_active: !context.is_timed_out().await,
                resource_usage: resource_usage.clone(),
            });
        }

        sessions
    }

    /// Get system health status
    pub async fn get_system_health(&self) -> SystemHealth {
        let sessions = self.session_bundles.read().await;
        let mut session_health = Vec::new();

        for (id, bundle) in sessions.iter() {
            session_health.push((id.clone(), bundle.health_check().await));
        }

        SystemHealth {
            total_sessions: session_health.len(),
            healthy_sessions: session_health
                .iter()
                .filter(|(_, h)| matches!(h.overall_status, super::session::HealthStatus::Healthy))
                .count(),
            resource_usage: self.get_resource_usage().await,
            cache_stats: self.cache.get_stats().await,
            event_metrics: self.event_bus.get_metrics().await,
        }
    }

    async fn get_resource_usage(&self) -> ResourceUsage {
        let browsers = self.resource_manager.active_browsers.read().await;
        ResourceUsage {
            active_browsers: browsers.len(),
            total_sessions: self.session_bundles.read().await.len(),
            memory_mb: 0,     // Would need actual memory monitoring
            cpu_percent: 0.0, // Would need actual CPU monitoring
        }
    }

    /// Start background cleanup task
    fn start_cleanup_task(&self) {
        let coordinator = Arc::new(self.clone());
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(coordinator.config.cleanup_interval);
            loop {
                interval.tick().await;
                if let Err(e) = coordinator.cleanup_timed_out_sessions().await {
                    error!("Cleanup task error: {}", e);
                }
            }
        });
    }

    /// Cleanup timed out sessions
    async fn cleanup_timed_out_sessions(&self) -> Result<()> {
        let bundles = self.session_bundles.read().await;
        let mut to_remove = Vec::new();

        for (id, bundle) in bundles.iter() {
            if bundle.context.is_timed_out().await {
                to_remove.push(id.clone());
            }
        }
        drop(bundles);

        for session_id in to_remove {
            info!("Cleaning up timed out session: {}", session_id);
            self.remove_session(&session_id).await?;
        }

        Ok(())
    }

    /// Start monitoring task
    fn start_monitoring_task(&self) {
        let monitoring = self.monitoring.clone();
        let bundles = self.session_bundles.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30));
            loop {
                interval.tick().await;

                let bundles = bundles.read().await;
                for bundle in bundles.values() {
                    monitoring.monitor_session_bundle(bundle).await;
                }
            }
        });
    }
}

impl Clone for RainbowCoordinator {
    fn clone(&self) -> Self {
        Self {
            event_bus: self.event_bus.clone(),
            state_manager: self.state_manager.clone(),
            resource_manager: self.resource_manager.clone(),
            module_registry: self.module_registry.clone(),
            monitoring: self.monitoring.clone(),
            cache: self.cache.clone(),
            session_contexts: self.session_contexts.clone(),
            session_bundles: self.session_bundles.clone(),
            config: self.config.clone(),
        }
    }
}

// Supporting types

#[derive(Debug, Clone)]
pub struct SessionInfo {
    pub session_id: String,
    pub created_at: Instant,
    pub last_activity: Instant,
    pub is_active: bool,
    pub resource_usage: super::session::ResourceUsage,
}

#[derive(Debug, Clone)]
pub struct SystemHealth {
    pub total_sessions: usize,
    pub healthy_sessions: usize,
    pub resource_usage: ResourceUsage,
    pub cache_stats: super::cache::CacheStats,
    pub event_metrics: super::events::EventMetrics,
}

#[derive(Debug, Clone)]
pub struct ResourceUsage {
    pub active_browsers: usize,
    pub total_sessions: usize,
    pub memory_mb: usize,
    pub cpu_percent: f64,
}
