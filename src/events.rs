// Observer pattern for event-driven communication between engines
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use serde::{Deserialize, Serialize};

/// Event types in the system - Strongly typed
#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub enum EventType {
    // Session events
    SessionCreated,
    SessionDestroyed,
    SessionError,
    
    // Perception events
    PerceptionStarted,
    PerceptionCompleted,
    PerceptionFailed,
    
    // Action events
    ActionStarted,
    ActionCompleted,
    ActionFailed,
    
    // Memory events
    MemoryStored,
    MemoryRetrieved,
    MemoryOptimized,
    
    // Performance events
    PerformanceThresholdExceeded,
    PerformanceOptimized,
    
    // Stability events
    HealthCheckPassed,
    HealthCheckFailed,
    RecoveryInitiated,
    RecoveryCompleted,
    
    // Request lifecycle events (replacing string-based custom events)
    RequestStarted,
    RequestCompleted,
    RequestFailed,
    RequestCancelled,
    
    // Workflow events
    WorkflowStepStarted(WorkflowStep),
    WorkflowStepCompleted(WorkflowStep),
    WorkflowStepFailed(WorkflowStep),
    
    // User interaction events
    UserInputReceived,
    UserFeedbackProvided,
    UserChoiceMade,
    
    // System events
    SystemInitialized,
    SystemShutdown,
    ConfigurationChanged,
    
    // Monitoring events
    MetricCollected(MetricType),
    AlertTriggered(AlertLevel),
    
    // For truly custom events (discouraged)
    Custom(String),
}

/// Workflow steps as strongly-typed enum
#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub enum WorkflowStep {
    SessionCreation,
    Monitoring,
    Perception,
    ActionExecution,
    MemoryStorage,
    Cleanup,
}

/// Metric types for monitoring events
#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub enum MetricType {
    CpuUsage,
    MemoryUsage,
    ResponseTime,
    ErrorRate,
    Throughput,
}

/// Alert levels for monitoring
#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub enum AlertLevel {
    Info,
    Warning,
    Error,
    Critical,
}

/// System event with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: Uuid,
    pub event_type: EventType,
    pub timestamp: std::time::SystemTime,
    pub session_id: Option<Uuid>,
    pub data: serde_json::Value,
    pub source: String,
}

impl Event {
    pub fn new(event_type: EventType, source: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            event_type,
            timestamp: std::time::SystemTime::now(),
            session_id: None,
            data: serde_json::Value::Null,
            source,
        }
    }

    pub fn with_session(mut self, session_id: Uuid) -> Self {
        self.session_id = Some(session_id);
        self
    }

    pub fn with_data(mut self, data: serde_json::Value) -> Self {
        self.data = data;
        self
    }
}

/// Event observer trait
#[async_trait]
pub trait EventObserver: Send + Sync {
    async fn on_event(&self, event: &Event);
    fn name(&self) -> &str;
}

/// Event publisher trait
#[async_trait]
pub trait EventPublisher: Send + Sync {
    async fn publish(&self, event: Event);
    async fn subscribe(&self, event_type: EventType, observer: Arc<dyn EventObserver>);
    async fn unsubscribe(&self, event_type: &EventType, observer_name: &str);
}

/// Default event bus implementation
pub struct EventBus {
    observers: Arc<RwLock<HashMap<EventType, Vec<Arc<dyn EventObserver>>>>>,
    event_history: Arc<RwLock<Vec<Event>>>,
    max_history_size: usize,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            observers: Arc::new(RwLock::new(HashMap::new())),
            event_history: Arc::new(RwLock::new(Vec::new())),
            max_history_size: 1000,
        }
    }

    pub fn with_max_history(mut self, size: usize) -> Self {
        self.max_history_size = size;
        self
    }

    /// Get event history
    pub async fn get_history(&self, event_type: Option<EventType>) -> Vec<Event> {
        let history = self.event_history.read().await;
        if let Some(et) = event_type {
            history.iter()
                .filter(|e| e.event_type == et)
                .cloned()
                .collect()
        } else {
            history.clone()
        }
    }

    /// Clear event history
    pub async fn clear_history(&self) {
        let mut history = self.event_history.write().await;
        history.clear();
    }
}

#[async_trait]
impl EventPublisher for EventBus {
    async fn publish(&self, event: Event) {
        // Store in history
        {
            let mut history = self.event_history.write().await;
            history.push(event.clone());
            
            // Trim history if needed
            if history.len() > self.max_history_size {
                let drain_count = history.len() - self.max_history_size;
                history.drain(0..drain_count);
            }
        }

        // Notify observers
        let observers = self.observers.read().await;
        if let Some(observer_list) = observers.get(&event.event_type) {
            for observer in observer_list {
                observer.on_event(&event).await;
            }
        }

        // Also notify observers subscribed to all events
        if let Some(observer_list) = observers.get(&EventType::Custom("*".to_string())) {
            for observer in observer_list {
                observer.on_event(&event).await;
            }
        }
    }

    async fn subscribe(&self, event_type: EventType, observer: Arc<dyn EventObserver>) {
        let mut observers = self.observers.write().await;
        observers.entry(event_type)
            .or_insert_with(Vec::new)
            .push(observer);
    }

    async fn unsubscribe(&self, event_type: &EventType, observer_name: &str) {
        let mut observers = self.observers.write().await;
        if let Some(observer_list) = observers.get_mut(event_type) {
            observer_list.retain(|o| o.name() != observer_name);
        }
    }
}

/// Logging observer for debugging
pub struct LoggingObserver {
    name: String,
    log_level: log::Level,
}

impl LoggingObserver {
    pub fn new(name: String) -> Self {
        Self {
            name,
            log_level: log::Level::Info,
        }
    }

    pub fn with_level(mut self, level: log::Level) -> Self {
        self.log_level = level;
        self
    }
}

#[async_trait]
impl EventObserver for LoggingObserver {
    async fn on_event(&self, event: &Event) {
        log::log!(
            self.log_level,
            "[{}] Event: {:?} from {} at {:?}",
            self.name,
            event.event_type,
            event.source,
            event.timestamp
        );
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// Metrics observer for performance tracking
pub struct MetricsObserver {
    name: String,
    event_counts: Arc<RwLock<HashMap<EventType, u64>>>,
}

impl MetricsObserver {
    pub fn new(name: String) -> Self {
        Self {
            name,
            event_counts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn get_metrics(&self) -> HashMap<EventType, u64> {
        self.event_counts.read().await.clone()
    }
}

#[async_trait]
impl EventObserver for MetricsObserver {
    async fn on_event(&self, event: &Event) {
        let mut counts = self.event_counts.write().await;
        *counts.entry(event.event_type.clone()).or_insert(0) += 1;
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// Alert observer for critical events
pub struct AlertObserver {
    name: String,
    alert_threshold: u32,
    alert_count: Arc<RwLock<u32>>,
    alert_callback: Arc<dyn Fn(Event) + Send + Sync>,
}

impl AlertObserver {
    pub fn new(
        name: String,
        threshold: u32,
        callback: Arc<dyn Fn(Event) + Send + Sync>,
    ) -> Self {
        Self {
            name,
            alert_threshold: threshold,
            alert_count: Arc::new(RwLock::new(0)),
            alert_callback: callback,
        }
    }
}

#[async_trait]
impl EventObserver for AlertObserver {
    async fn on_event(&self, event: &Event) {
        // Check if this is a critical event
        match &event.event_type {
            EventType::SessionError |
            EventType::PerceptionFailed |
            EventType::ActionFailed |
            EventType::HealthCheckFailed => {
                let mut count = self.alert_count.write().await;
                *count += 1;
                
                if *count >= self.alert_threshold {
                    (self.alert_callback)(event.clone());
                    *count = 0; // Reset counter
                }
            }
            _ => {}
        }
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// Chain of responsibility observer for event processing
pub struct ChainObserver {
    name: String,
    handlers: Vec<Arc<dyn EventObserver>>,
}

impl ChainObserver {
    pub fn new(name: String) -> Self {
        Self {
            name,
            handlers: Vec::new(),
        }
    }

    pub fn add_handler(&mut self, handler: Arc<dyn EventObserver>) {
        self.handlers.push(handler);
    }
}

#[async_trait]
impl EventObserver for ChainObserver {
    async fn on_event(&self, event: &Event) {
        for handler in &self.handlers {
            handler.on_event(event).await;
        }
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// Builder for setting up event system
pub struct EventSystemBuilder {
    event_bus: EventBus,
    observers: Vec<(EventType, Arc<dyn EventObserver>)>,
}

impl EventSystemBuilder {
    pub fn new() -> Self {
        Self {
            event_bus: EventBus::new(),
            observers: Vec::new(),
        }
    }

    pub fn with_max_history(mut self, size: usize) -> Self {
        self.event_bus = self.event_bus.with_max_history(size);
        self
    }

    pub fn add_observer(mut self, event_type: EventType, observer: Arc<dyn EventObserver>) -> Self {
        self.observers.push((event_type, observer));
        self
    }

    pub fn add_logging(mut self, name: String) -> Self {
        let observer = Arc::new(LoggingObserver::new(name));
        self.observers.push((EventType::Custom("*".to_string()), observer));
        self
    }

    pub fn add_metrics(mut self, name: String) -> Self {
        let observer = Arc::new(MetricsObserver::new(name));
        self.observers.push((EventType::Custom("*".to_string()), observer));
        self
    }

    pub async fn build(self) -> Arc<EventBus> {
        let bus = Arc::new(self.event_bus);
        
        for (event_type, observer) in self.observers {
            bus.subscribe(event_type, observer).await;
        }
        
        bus
    }
}