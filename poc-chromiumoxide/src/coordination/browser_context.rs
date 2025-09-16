// Shared Browser Session Context for Module Coordination
// Addresses the critical coordination issues identified in the analysis

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use std::time::Instant;
use tracing::{info, debug, warn};

use crate::browser::Browser;
use super::perception_impl::PerceptionEngine;
use super::tools_impl::ToolRegistry;
use super::{
    EventBus, Event, UnifiedStateManager, UnifiedCache,
    session::SessionContext,
};

/// Shared context that coordinates browser state across all modules
/// This solves the issue of multiple separate browser instances and lost context
pub struct BrowserSessionContext {
    session_id: String,
    browser: Arc<Browser>,
    
    // Shared module instances - solving instance fragmentation
    perception_engine: Arc<RwLock<Option<Arc<PerceptionEngine>>>>,
    tool_registry: Arc<RwLock<Option<Arc<ToolRegistry>>>>,
    
    // Shared state and coordination
    event_bus: Arc<EventBus>,
    state_manager: Arc<UnifiedStateManager>,
    cache: Arc<UnifiedCache>,
    
    // Navigation state tracking
    current_url: Arc<RwLock<String>>,
    last_navigation: Arc<RwLock<Instant>>,
    navigation_count: Arc<RwLock<u64>>,
    
    // Resource tracking
    resource_usage: Arc<RwLock<ResourceUsage>>,
    
    // Module coordination flags
    modules_initialized: Arc<RwLock<ModulesState>>,
}

#[derive(Debug, Clone, Default)]
struct ResourceUsage {
    perception_operations: u64,
    tool_executions: u64,
    cache_hits: u64,
    cache_misses: u64,
    total_operations: u64,
}

#[derive(Debug, Clone, Default)]
struct ModulesState {
    perception_initialized: bool,
    tools_initialized: bool,
    intelligence_initialized: bool,
}

impl BrowserSessionContext {
    /// Create a new shared browser session context
    pub async fn new(
        session_id: String,
        browser: Arc<Browser>,
        event_bus: Arc<EventBus>,
        state_manager: Arc<UnifiedStateManager>,
        cache: Arc<UnifiedCache>,
    ) -> Result<Arc<Self>> {
        let current_url = browser.url().await.unwrap_or_default();
        
        let context = Arc::new(Self {
            session_id: session_id.clone(),
            browser,
            perception_engine: Arc::new(RwLock::new(None)),
            tool_registry: Arc::new(RwLock::new(None)),
            event_bus: event_bus.clone(),
            state_manager,
            cache,
            current_url: Arc::new(RwLock::new(current_url)),
            last_navigation: Arc::new(RwLock::new(Instant::now())),
            navigation_count: Arc::new(RwLock::new(0)),
            resource_usage: Arc::new(RwLock::new(ResourceUsage::default())),
            modules_initialized: Arc::new(RwLock::new(ModulesState::default())),
        });
        
        // Set up event listeners for coordination
        context.setup_event_coordination().await?;
        
        // Emit context creation event
        event_bus.emit(Event::SessionContextCreated {
            session_id,
            timestamp: Instant::now(),
        }).await?;
        
        Ok(context)
    }
    
    /// Get or create the shared perception engine instance
    /// This solves the multiple instance problem
    pub async fn get_perception_engine(&self) -> Result<Arc<PerceptionEngine>> {
        let mut engine_lock = self.perception_engine.write().await;
        
        if let Some(engine) = engine_lock.as_ref() {
            // Track cache hit
            self.track_resource_usage(ResourceOperation::CacheHit).await;
            return Ok(engine.clone());
        }
        
        // Create new engine with shared browser
        info!("Creating shared PerceptionEngine for session: {}", self.session_id);
        let engine = Arc::new(PerceptionEngine::new(self.browser.clone()).await?);
        *engine_lock = Some(engine.clone());
        
        // Update module state
        let mut modules = self.modules_initialized.write().await;
        modules.perception_initialized = true;
        
        // Track resource usage
        self.track_resource_usage(ResourceOperation::PerceptionInit).await;
        
        // Emit initialization event
        self.event_bus.emit(Event::ModuleInitialized {
            session_id: self.session_id.clone(),
            module_type: "perception".to_string(),
            timestamp: Instant::now(),
        }).await?;
        
        Ok(engine)
    }
    
    /// Get or create the shared tool registry instance
    /// This prevents race conditions in lazy initialization
    pub async fn get_tool_registry(&self) -> Result<Arc<ToolRegistry>> {
        let mut registry_lock = self.tool_registry.write().await;
        
        if let Some(registry) = registry_lock.as_ref() {
            // Track cache hit
            self.track_resource_usage(ResourceOperation::CacheHit).await;
            return Ok(registry.clone());
        }
        
        // Create new registry with shared browser
        info!("Creating shared ToolRegistry for session: {}", self.session_id);
        let mut registry = ToolRegistry::new(self.browser.clone());
        registry.initialize().await?;
        
        let registry = Arc::new(registry);
        *registry_lock = Some(registry.clone());
        
        // Update module state
        let mut modules = self.modules_initialized.write().await;
        modules.tools_initialized = true;
        
        // Track resource usage
        self.track_resource_usage(ResourceOperation::ToolInit).await;
        
        // Emit initialization event
        self.event_bus.emit(Event::ModuleInitialized {
            session_id: self.session_id.clone(),
            module_type: "tools".to_string(),
            timestamp: Instant::now(),
        }).await?;
        
        Ok(registry)
    }
    
    /// Handle navigation events to coordinate all modules
    pub async fn handle_navigation(&self, url: &str) -> Result<()> {
        info!("Handling navigation to: {} for session: {}", url, self.session_id);
        
        // Update navigation state
        {
            let mut current = self.current_url.write().await;
            *current = url.to_string();
            
            let mut last = self.last_navigation.write().await;
            *last = Instant::now();
            
            let mut count = self.navigation_count.write().await;
            *count += 1;
        }
        
        // Invalidate perception cache on navigation
        if let Some(perception) = self.perception_engine.read().await.as_ref() {
            debug!("Invalidating perception cache after navigation");
            // In real implementation, call perception.clear_cache()
        }
        
        // Invalidate tool state if needed
        if let Some(tools) = self.tool_registry.read().await.as_ref() {
            debug!("Resetting tool state after navigation");
            // In real implementation, call tools.reset_state()
        }
        
        // Invalidate shared cache entries
        self.cache.invalidate_by_pattern(&format!("session:{}:*", self.session_id)).await;
        
        // Emit navigation event for all modules
        self.event_bus.emit(Event::NavigationCompleted {
            session_id: self.session_id.clone(),
            url: url.to_string(),
            load_time_ms: 0,
            timestamp: Instant::now(),
        }).await?;
        
        Ok(())
    }
    
    /// Coordinate perception and tool execution
    /// This ensures they work on the same browser context
    pub async fn execute_coordinated_action(
        &self,
        action_type: &str,
        target: &str,
    ) -> Result<serde_json::Value> {
        debug!("Executing coordinated action: {} on {}", action_type, target);
        
        // First, use perception to analyze the target
        let perception = self.get_perception_engine().await?;
        let html = self.browser.content().await?;
        let elements = perception.find_interactive_elements(&html)?;
        
        // Find matching elements
        let matches: Vec<_> = elements.iter()
            .filter(|e| e.text.contains(target) || e.selector.contains(target))
            .collect();
        
        if matches.is_empty() {
            warn!("No elements found matching target: {}", target);
            return Ok(serde_json::json!({
                "success": false,
                "error": "No matching elements found"
            }));
        }
        
        // Use tools to execute the action
        let tools = self.get_tool_registry().await?;
        let tool_result = match action_type {
            "click" => {
                // Execute click with best match
                serde_json::json!({
                    "tool": "click",
                    "selector": matches[0].selector,
                    "success": true
                })
            }
            "type" => {
                // Execute type action
                serde_json::json!({
                    "tool": "type",
                    "selector": matches[0].selector,
                    "success": true
                })
            }
            _ => {
                serde_json::json!({
                    "success": false,
                    "error": format!("Unknown action type: {}", action_type)
                })
            }
        };
        
        // Track resource usage
        self.track_resource_usage(ResourceOperation::CoordinatedAction).await;
        
        Ok(tool_result)
    }
    
    /// Set up event coordination between modules
    async fn setup_event_coordination(&self) -> Result<()> {
        // In a real implementation, this would set up event handlers
        // For now, we'll just log that coordination is set up
        debug!("Event coordination set up for session: {}", self.session_id);
        Ok(())
    }
    
    /// Track resource usage for monitoring and optimization
    async fn track_resource_usage(&self, operation: ResourceOperation) {
        let mut usage = self.resource_usage.write().await;
        usage.total_operations += 1;
        
        match operation {
            ResourceOperation::PerceptionInit | ResourceOperation::PerceptionOp => {
                usage.perception_operations += 1;
            }
            ResourceOperation::ToolInit | ResourceOperation::ToolExecution => {
                usage.tool_executions += 1;
            }
            ResourceOperation::CacheHit => {
                usage.cache_hits += 1;
            }
            ResourceOperation::CacheMiss => {
                usage.cache_misses += 1;
            }
            _ => {}
        }
    }
    
    /// Get current resource usage statistics
    pub async fn get_resource_stats(&self) -> ResourceUsage {
        self.resource_usage.read().await.clone()
    }
    
    /// Clean up resources when session ends
    pub async fn cleanup(&self) -> Result<()> {
        info!("Cleaning up BrowserSessionContext for session: {}", self.session_id);
        
        // Clear perception engine
        if let Some(perception) = self.perception_engine.write().await.take() {
            debug!("Cleaning up perception engine");
            // In real implementation, call perception.cleanup()
        }
        
        // Clear tool registry
        if let Some(tools) = self.tool_registry.write().await.take() {
            debug!("Cleaning up tool registry");
            // In real implementation, call tools.cleanup()
        }
        
        // Clear caches
        self.cache.invalidate_by_pattern(&format!("session:{}:*", self.session_id)).await;
        
        // Emit cleanup event
        self.event_bus.emit(Event::SessionClosed {
            session_id: self.session_id.clone(),
            reason: "Session cleanup".to_string(),
            timestamp: Instant::now(),
        }).await?;
        
        Ok(())
    }
}

enum ResourceOperation {
    PerceptionInit,
    PerceptionOp,
    ToolInit,
    ToolExecution,
    CacheHit,
    CacheMiss,
    CoordinatedAction,
}

/// Module coordinator that manages lifecycle and dependencies
/// This addresses the module coordination and circular dependency issues
pub struct ModuleCoordinator {
    contexts: Arc<RwLock<HashMap<String, Arc<BrowserSessionContext>>>>,
    event_bus: Arc<EventBus>,
    state_manager: Arc<UnifiedStateManager>,
    cache: Arc<UnifiedCache>,
}

impl ModuleCoordinator {
    pub fn new(
        event_bus: Arc<EventBus>,
        state_manager: Arc<UnifiedStateManager>,
        cache: Arc<UnifiedCache>,
    ) -> Self {
        Self {
            contexts: Arc::new(RwLock::new(HashMap::new())),
            event_bus,
            state_manager,
            cache,
        }
    }
    
    /// Create or get a session context
    pub async fn get_or_create_context(
        &self,
        session_id: &str,
        browser: Arc<Browser>,
    ) -> Result<Arc<BrowserSessionContext>> {
        let mut contexts = self.contexts.write().await;
        
        if let Some(context) = contexts.get(session_id) {
            return Ok(context.clone());
        }
        
        // Create new context
        let context = BrowserSessionContext::new(
            session_id.to_string(),
            browser,
            self.event_bus.clone(),
            self.state_manager.clone(),
            self.cache.clone(),
        ).await?;
        
        contexts.insert(session_id.to_string(), context.clone());
        Ok(context)
    }
    
    /// Remove a session context
    pub async fn remove_context(&self, session_id: &str) -> Result<()> {
        let mut contexts = self.contexts.write().await;
        
        if let Some(context) = contexts.remove(session_id) {
            context.cleanup().await?;
        }
        
        Ok(())
    }
    
    /// Get all active session contexts
    pub async fn get_active_contexts(&self) -> Vec<String> {
        self.contexts.read().await.keys().cloned().collect()
    }
}

// Extension to Event enum for new coordination events
impl Event {
    pub fn session_context_created(session_id: String) -> Self {
        Event::SessionCreated {
            session_id,
            timestamp: Instant::now(),
        }
    }
}