// Core Coordination Module for RainbowBrowserAI
// Provides centralized coordination, event-driven communication, and unified state management

pub mod events;
pub mod state;
pub mod session;
pub mod coordinator;
pub mod cache;
pub mod monitoring;
pub mod perception_impl;
pub mod tools_impl;
pub mod intelligence_impl;
pub mod browser_context;
pub mod error_handler;

// Re-export main types
pub use events::{Event, EventBus, EventType, EventHandler};
pub use state::{UnifiedStateManager, BrowserState, PerceptionState, ToolState};
pub use session::{SessionContext, SessionBundle};
pub use coordinator::RainbowCoordinator;
pub use cache::{UnifiedCache, CacheCoordinator};
pub use monitoring::{UnifiedMonitoring, ModuleHealth};

use anyhow::Result;
use std::sync::Arc;

/// Trait for modules that participate in coordinated operations
#[async_trait::async_trait]
pub trait CoordinatedModule: Send + Sync {
    /// Initialize the module with session context
    async fn initialize(&mut self, context: &SessionContext) -> Result<()>;
    
    /// Handle events from the event bus
    async fn handle_event(&self, event: &Event) -> Result<()>;
    
    /// Perform cleanup when module is shutting down
    async fn cleanup(&mut self) -> Result<()>;
    
    /// Get the list of module dependencies
    fn dependencies(&self) -> Vec<ModuleType>;
    
    /// Check the health of the module
    fn health_check(&self) -> ModuleHealth;
    
    /// Get module metrics
    fn get_metrics(&self) -> serde_json::Value;
}

/// Types of modules in the system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModuleType {
    Browser,
    Perception,
    Tools,
    Intelligence,
    LLM,
    Cache,
    Monitoring,
}

/// Module initialization configuration
#[derive(Debug, Clone)]
pub struct ModuleConfig {
    pub enable_caching: bool,
    pub enable_monitoring: bool,
    pub event_subscriptions: Vec<EventType>,
    pub performance_mode: PerformanceMode,
}

#[derive(Debug, Clone, Copy)]
pub enum PerformanceMode {
    /// Optimize for low latency
    LowLatency,
    /// Optimize for high throughput
    HighThroughput,
    /// Balanced performance
    Balanced,
    /// Power saving mode
    PowerSaving,
}

impl Default for ModuleConfig {
    fn default() -> Self {
        Self {
            enable_caching: true,
            enable_monitoring: true,
            event_subscriptions: Vec::new(),
            performance_mode: PerformanceMode::Balanced,
        }
    }
}