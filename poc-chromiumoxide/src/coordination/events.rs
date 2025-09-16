// Event System for Module Coordination
// Provides event-driven communication between modules

use anyhow::Result;
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::Instant;
use tracing::{debug, error, info};
use uuid::Uuid;

/// Core event types in the system
#[derive(Debug, Clone)]
pub enum Event {
    // Browser Events
    NavigationStarted { 
        session_id: String, 
        url: String,
        timestamp: Instant,
    },
    NavigationCompleted { 
        session_id: String, 
        url: String, 
        load_time_ms: u64,
        timestamp: Instant,
    },
    PageContentChanged { 
        session_id: String, 
        change_type: ContentChangeType,
        timestamp: Instant,
    },
    BrowserError {
        session_id: String,
        error: String,
        timestamp: Instant,
    },
    
    // Perception Events
    PerceptionAnalysisStarted {
        session_id: String,
        analysis_type: String,
        timestamp: Instant,
    },
    PerceptionAnalysisCompleted {
        session_id: String,
        analysis_type: String,
        duration_ms: u64,
        result_summary: String,
        timestamp: Instant,
    },
    ElementFound {
        session_id: String,
        selector: String,
        confidence: f64,
        element_type: String,
        timestamp: Instant,
    },
    PageClassified {
        session_id: String,
        page_type: String,
        confidence: f64,
        timestamp: Instant,
    },
    AnalysisCompleted {
        session_id: String,
        analysis_type: String,
        element_count: usize,
        duration_ms: u64,
        timestamp: Instant,
    },
    
    // Intelligence Events
    PlanningCompleted {
        session_id: String,
        action_id: String,
        confidence: f64,
        duration_ms: u64,
        timestamp: Instant,
    },
    LearningCompleted {
        session_id: String,
        success: bool,
        patterns_updated: usize,
        timestamp: Instant,
    },
    
    // Tool Events
    ToolExecutionStarted {
        session_id: String,
        tool_name: String,
        execution_id: String,
        timestamp: Instant,
    },
    ToolExecutionCompleted {
        session_id: String,
        tool_name: String,
        execution_id: String,
        success: bool,
        duration_ms: u64,
        timestamp: Instant,
    },
    ToolExecutionFailed {
        session_id: String,
        tool_name: String,
        execution_id: String,
        error: String,
        timestamp: Instant,
    },
    
    // Session Events
    SessionCreated {
        session_id: String,
        timestamp: Instant,
    },
    SessionClosed {
        session_id: String,
        reason: String,
        timestamp: Instant,
    },
    SessionTimeout {
        session_id: String,
        idle_duration_ms: u64,
        timestamp: Instant,
    },
    
    // Cache Events
    CacheInvalidated {
        cache_type: String,
        reason: String,
        keys_affected: Vec<String>,
        timestamp: Instant,
    },
    CacheHit {
        cache_type: String,
        key: String,
        timestamp: Instant,
    },
    CacheMiss {
        cache_type: String,
        key: String,
        timestamp: Instant,
    },
    
    // System Events
    ResourceWarning {
        resource_type: String,
        usage_percent: f64,
        threshold: f64,
        timestamp: Instant,
    },
    ModuleInitialized {
        module_type: String,
        session_id: String,
        timestamp: Instant,
    },
    ModuleShutdown {
        module_type: String,
        session_id: String,
        timestamp: Instant,
    },
    ModuleError {
        module_type: String,
        error: String,
        timestamp: Instant,
    },
    SessionContextCreated {
        session_id: String,
        timestamp: Instant,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventType {
    NavigationStarted,
    NavigationCompleted,
    PageContentChanged,
    BrowserError,
    PerceptionAnalysisStarted,
    PerceptionAnalysisCompleted,
    ElementFound,
    PageClassified,
    AnalysisCompleted,
    PlanningCompleted,
    LearningCompleted,
    ToolExecutionStarted,
    ToolExecutionCompleted,
    ToolExecutionFailed,
    SessionCreated,
    SessionClosed,
    SessionTimeout,
    CacheInvalidated,
    CacheHit,
    CacheMiss,
    ResourceWarning,
    ModuleInitialized,
    ModuleShutdown,
    ModuleError,
    SessionContextCreated,
}

impl Event {
    pub fn event_type(&self) -> EventType {
        match self {
            Event::NavigationStarted { .. } => EventType::NavigationStarted,
            Event::NavigationCompleted { .. } => EventType::NavigationCompleted,
            Event::PageContentChanged { .. } => EventType::PageContentChanged,
            Event::BrowserError { .. } => EventType::BrowserError,
            Event::PerceptionAnalysisStarted { .. } => EventType::PerceptionAnalysisStarted,
            Event::PerceptionAnalysisCompleted { .. } => EventType::PerceptionAnalysisCompleted,
            Event::ElementFound { .. } => EventType::ElementFound,
            Event::PageClassified { .. } => EventType::PageClassified,
            Event::AnalysisCompleted { .. } => EventType::AnalysisCompleted,
            Event::PlanningCompleted { .. } => EventType::PlanningCompleted,
            Event::LearningCompleted { .. } => EventType::LearningCompleted,
            Event::ToolExecutionStarted { .. } => EventType::ToolExecutionStarted,
            Event::ToolExecutionCompleted { .. } => EventType::ToolExecutionCompleted,
            Event::ToolExecutionFailed { .. } => EventType::ToolExecutionFailed,
            Event::SessionCreated { .. } => EventType::SessionCreated,
            Event::SessionClosed { .. } => EventType::SessionClosed,
            Event::SessionTimeout { .. } => EventType::SessionTimeout,
            Event::CacheInvalidated { .. } => EventType::CacheInvalidated,
            Event::CacheHit { .. } => EventType::CacheHit,
            Event::CacheMiss { .. } => EventType::CacheMiss,
            Event::ResourceWarning { .. } => EventType::ResourceWarning,
            Event::ModuleInitialized { .. } => EventType::ModuleInitialized,
            Event::ModuleShutdown { .. } => EventType::ModuleShutdown,
            Event::ModuleError { .. } => EventType::ModuleError,
            Event::SessionContextCreated { .. } => EventType::SessionContextCreated,
        }
    }
    
    pub fn session_id(&self) -> Option<&str> {
        match self {
            Event::NavigationStarted { session_id, .. } |
            Event::NavigationCompleted { session_id, .. } |
            Event::PageContentChanged { session_id, .. } |
            Event::BrowserError { session_id, .. } |
            Event::PerceptionAnalysisStarted { session_id, .. } |
            Event::PerceptionAnalysisCompleted { session_id, .. } |
            Event::ElementFound { session_id, .. } |
            Event::PageClassified { session_id, .. } |
            Event::AnalysisCompleted { session_id, .. } |
            Event::PlanningCompleted { session_id, .. } |
            Event::LearningCompleted { session_id, .. } |
            Event::ToolExecutionStarted { session_id, .. } |
            Event::ToolExecutionCompleted { session_id, .. } |
            Event::ToolExecutionFailed { session_id, .. } |
            Event::SessionCreated { session_id, .. } |
            Event::SessionClosed { session_id, .. } |
            Event::SessionTimeout { session_id, .. } |
            Event::ModuleInitialized { session_id, .. } |
            Event::ModuleShutdown { session_id, .. } |
            Event::SessionContextCreated { session_id, .. } => Some(session_id),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ContentChangeType {
    DomMutation,
    FormInput,
    ScrollPosition,
    WindowResize,
    AjaxComplete,
    Unknown,
}

/// Event handler trait
#[async_trait::async_trait]
pub trait EventHandler: Send + Sync {
    async fn handle(&self, event: &Event) -> Result<()>;
}

/// Event subscriber wrapper
pub struct EventSubscriber {
    id: String,
    handler: Box<dyn EventHandler>,
    filter: Option<Box<dyn Fn(&Event) -> bool + Send + Sync>>,
}

/// Timestamped event for history
#[derive(Debug, Clone)]
pub struct TimestampedEvent {
    pub event: Event,
    pub timestamp: Instant,
    pub sequence_id: u64,
}

/// Event bus metrics
#[derive(Debug, Default, Clone)]
pub struct EventMetrics {
    pub total_events: u64,
    pub events_by_type: HashMap<EventType, u64>,
    pub failed_handlers: u64,
    pub average_handling_time_ms: f64,
}

/// Central event bus for the system
pub struct EventBus {
    subscribers: Arc<RwLock<HashMap<EventType, Vec<Arc<EventSubscriber>>>>>,
    event_history: Arc<RwLock<VecDeque<TimestampedEvent>>>,
    metrics: Arc<RwLock<EventMetrics>>,
    sequence_counter: Arc<RwLock<u64>>,
    max_history_size: usize,
}

impl EventBus {
    pub fn new() -> Self {
        Self::with_config(EventBusConfig::default())
    }
    
    pub fn with_config(config: EventBusConfig) -> Self {
        Self {
            subscribers: Arc::new(RwLock::new(HashMap::new())),
            event_history: Arc::new(RwLock::new(VecDeque::with_capacity(config.max_history_size))),
            metrics: Arc::new(RwLock::new(EventMetrics::default())),
            sequence_counter: Arc::new(RwLock::new(0)),
            max_history_size: config.max_history_size,
        }
    }
    
    /// Emit an event to all subscribers
    pub async fn emit(&self, event: Event) -> Result<()> {
        let start_time = Instant::now();
        let event_type = event.event_type();
        
        // Get next sequence ID
        let sequence_id = {
            let mut counter = self.sequence_counter.write().await;
            *counter += 1;
            *counter
        };
        
        // Store in history
        {
            let mut history = self.event_history.write().await;
            if history.len() >= self.max_history_size {
                history.pop_front();
            }
            history.push_back(TimestampedEvent {
                event: event.clone(),
                timestamp: Instant::now(),
                sequence_id,
            });
        }
        
        // Update metrics
        {
            let mut metrics = self.metrics.write().await;
            metrics.total_events += 1;
            *metrics.events_by_type.entry(event_type).or_insert(0) += 1;
        }
        
        // Notify subscribers
        let subscribers = self.subscribers.read().await;
        if let Some(handlers) = subscribers.get(&event_type) {
            for subscriber in handlers {
                // Apply filter if present
                if let Some(ref filter) = subscriber.filter {
                    if !filter(&event) {
                        continue;
                    }
                }
                
                // Handle event
                if let Err(e) = subscriber.handler.handle(&event).await {
                    error!("Event handler {} failed: {}", subscriber.id, e);
                    let mut metrics = self.metrics.write().await;
                    metrics.failed_handlers += 1;
                }
            }
        }
        
        // Update average handling time
        let duration = start_time.elapsed();
        {
            let mut metrics = self.metrics.write().await;
            let total = metrics.total_events as f64;
            let current_avg = metrics.average_handling_time_ms;
            metrics.average_handling_time_ms = 
                (current_avg * (total - 1.0) + duration.as_millis() as f64) / total;
        }
        
        debug!("Event {:?} emitted to {} subscribers in {:?}", 
               event_type, 
               subscribers.get(&event_type).map(|s| s.len()).unwrap_or(0),
               duration);
        
        Ok(())
    }
    
    /// Subscribe to events of a specific type
    pub async fn subscribe<H>(&self, event_type: EventType, handler: H) -> String
    where
        H: EventHandler + 'static,
    {
        self.subscribe_with_filter(event_type, handler, None::<fn(&Event) -> bool>).await
    }
    
    /// Subscribe with a filter function
    pub async fn subscribe_with_filter<H, F>(&self, event_type: EventType, handler: H, filter: Option<F>) -> String
    where
        H: EventHandler + 'static,
        F: Fn(&Event) -> bool + Send + Sync + 'static,
    {
        let subscriber_id = Uuid::new_v4().to_string();
        let subscriber = Arc::new(EventSubscriber {
            id: subscriber_id.clone(),
            handler: Box::new(handler),
            filter: filter.map(|f| Box::new(f) as Box<dyn Fn(&Event) -> bool + Send + Sync>),
        });
        
        let mut subscribers = self.subscribers.write().await;
        subscribers.entry(event_type).or_default().push(subscriber);
        
        info!("New subscriber {} registered for {:?} events", subscriber_id, event_type);
        subscriber_id
    }
    
    /// Unsubscribe from events
    pub async fn unsubscribe(&self, subscriber_id: &str) -> Result<()> {
        let mut subscribers = self.subscribers.write().await;
        for handlers in subscribers.values_mut() {
            handlers.retain(|s| s.id != subscriber_id);
        }
        Ok(())
    }
    
    /// Get event history
    pub async fn get_history(&self, limit: Option<usize>) -> Vec<TimestampedEvent> {
        let history = self.event_history.read().await;
        match limit {
            Some(n) => history.iter().rev().take(n).cloned().collect(),
            None => history.iter().cloned().collect(),
        }
    }
    
    /// Get metrics
    pub async fn get_metrics(&self) -> EventMetrics {
        self.metrics.read().await.clone()
    }
    
    /// Clear event history
    pub async fn clear_history(&self) {
        self.event_history.write().await.clear();
    }
}

/// Configuration for event bus
#[derive(Debug, Clone)]
pub struct EventBusConfig {
    pub max_history_size: usize,
    pub enable_metrics: bool,
}

impl Default for EventBusConfig {
    fn default() -> Self {
        Self {
            max_history_size: 1000,
            enable_metrics: true,
        }
    }
}

// Helper implementation for closures as event handlers
#[async_trait::async_trait]
impl<F> EventHandler for F
where
    F: Fn(&Event) -> Result<()> + Send + Sync,
{
    async fn handle(&self, event: &Event) -> Result<()> {
        self(event)
    }
}