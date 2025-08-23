use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;
use anyhow::Result;
use chrono::{DateTime, Utc};

use crate::{WorkflowResult};
use super::types::{PluginId, ErrorSeverity};

/// Plugin event types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PluginEvent {
    #[serde(rename = "workflow_started")]
    WorkflowStarted {
        workflow_id: String,
        timestamp: DateTime<Utc>,
    },
    #[serde(rename = "workflow_completed")]
    WorkflowCompleted {
        workflow_id: String,
        result: WorkflowResult,
        timestamp: DateTime<Utc>,
    },
    #[serde(rename = "action_executed")]
    ActionExecuted {
        action: String,
        duration: std::time::Duration,
        success: bool,
        timestamp: DateTime<Utc>,
    },
    #[serde(rename = "browser_navigated")]
    BrowserNavigated {
        url: String,
        session_id: String,
        timestamp: DateTime<Utc>,
    },
    #[serde(rename = "error_occurred")]
    ErrorOccurred {
        error: String,
        severity: ErrorSeverity,
        plugin_id: Option<PluginId>,
        timestamp: DateTime<Utc>,
    },
    #[serde(rename = "metrics_updated")]
    MetricsUpdated {
        metrics: serde_json::Value,
        timestamp: DateTime<Utc>,
    },
    #[serde(rename = "configuration_changed")]
    ConfigurationChanged {
        section: String,
        timestamp: DateTime<Utc>,
    },
    #[serde(rename = "plugin_loaded")]
    PluginLoaded {
        plugin_id: PluginId,
        timestamp: DateTime<Utc>,
    },
    #[serde(rename = "plugin_unloaded")]
    PluginUnloaded {
        plugin_id: PluginId,
        timestamp: DateTime<Utc>,
    },
}

/// Event subscription types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EventType {
    WorkflowStarted,
    WorkflowCompleted,
    ActionExecuted,
    BrowserNavigated,
    ErrorOccurred,
    MetricsUpdated,
    ConfigurationChanged,
    PluginLoaded,
    PluginUnloaded,
    All,
}

impl From<&PluginEvent> for EventType {
    fn from(event: &PluginEvent) -> Self {
        match event {
            PluginEvent::WorkflowStarted { .. } => EventType::WorkflowStarted,
            PluginEvent::WorkflowCompleted { .. } => EventType::WorkflowCompleted,
            PluginEvent::ActionExecuted { .. } => EventType::ActionExecuted,
            PluginEvent::BrowserNavigated { .. } => EventType::BrowserNavigated,
            PluginEvent::ErrorOccurred { .. } => EventType::ErrorOccurred,
            PluginEvent::MetricsUpdated { .. } => EventType::MetricsUpdated,
            PluginEvent::ConfigurationChanged { .. } => EventType::ConfigurationChanged,
            PluginEvent::PluginLoaded { .. } => EventType::PluginLoaded,
            PluginEvent::PluginUnloaded { .. } => EventType::PluginUnloaded,
        }
    }
}

/// Event subscription handle
#[derive(Debug, Clone)]
pub struct EventSubscription {
    pub id: String,
    pub plugin_id: PluginId,
    pub event_types: Vec<EventType>,
    pub receiver: Arc<RwLock<broadcast::Receiver<PluginEvent>>>,
}

impl EventSubscription {
    pub fn new(
        plugin_id: PluginId,
        event_types: Vec<EventType>,
        receiver: broadcast::Receiver<PluginEvent>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            plugin_id,
            event_types,
            receiver: Arc::new(RwLock::new(receiver)),
        }
    }
}

/// Event bus trait for plugin communication
#[async_trait]
pub trait EventBus: Send + Sync {
    /// Subscribe to events
    async fn subscribe(
        &self,
        plugin_id: &PluginId,
        event_types: Vec<EventType>,
    ) -> Result<EventSubscription>;

    /// Publish an event
    async fn publish(&self, event: PluginEvent) -> Result<()>;

    /// Unsubscribe from events
    async fn unsubscribe(&self, subscription: EventSubscription) -> Result<()>;

    /// Get subscription count
    async fn subscription_count(&self) -> usize;
}

/// Default event bus implementation
#[derive(Debug)]
pub struct DefaultEventBus {
    sender: broadcast::Sender<PluginEvent>,
    subscriptions: Arc<RwLock<HashMap<String, EventSubscription>>>,
}

impl DefaultEventBus {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(1000);
        
        Self {
            sender,
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Get event statistics
    pub async fn get_stats(&self) -> EventBusStats {
        let subscriptions = self.subscriptions.read().await;
        
        let mut event_type_counts = HashMap::new();
        for subscription in subscriptions.values() {
            for event_type in &subscription.event_types {
                *event_type_counts.entry(event_type.clone()).or_insert(0) += 1;
            }
        }
        
        EventBusStats {
            total_subscriptions: subscriptions.len(),
            event_type_counts,
            receiver_count: self.sender.receiver_count(),
        }
    }
}

#[async_trait]
impl EventBus for DefaultEventBus {
    async fn subscribe(
        &self,
        plugin_id: &PluginId,
        event_types: Vec<EventType>,
    ) -> Result<EventSubscription> {
        let receiver = self.sender.subscribe();
        let subscription = EventSubscription::new(plugin_id.clone(), event_types, receiver);
        
        let mut subscriptions = self.subscriptions.write().await;
        subscriptions.insert(subscription.id.clone(), subscription.clone());
        
        tracing::info!(
            "Plugin {} subscribed to {} event types",
            plugin_id,
            subscription.event_types.len()
        );
        
        Ok(subscription)
    }

    async fn publish(&self, event: PluginEvent) -> Result<()> {
        let event_type = EventType::from(&event);
        
        match self.sender.send(event) {
            Ok(receiver_count) => {
                tracing::debug!(
                    "Published event {:?} to {} receivers",
                    event_type,
                    receiver_count
                );
                Ok(())
            }
            Err(broadcast::error::SendError(_)) => {
                tracing::warn!("Failed to publish event: no active receivers");
                Ok(()) // Not an error if no one is listening
            }
        }
    }

    async fn unsubscribe(&self, subscription: EventSubscription) -> Result<()> {
        let mut subscriptions = self.subscriptions.write().await;
        subscriptions.remove(&subscription.id);
        
        tracing::info!(
            "Plugin {} unsubscribed from events",
            subscription.plugin_id
        );
        
        Ok(())
    }

    async fn subscription_count(&self) -> usize {
        let subscriptions = self.subscriptions.read().await;
        subscriptions.len()
    }
}

/// Event bus statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventBusStats {
    pub total_subscriptions: usize,
    pub event_type_counts: HashMap<EventType, usize>,
    pub receiver_count: usize,
}

/// Event listener trait for plugins
#[async_trait]
pub trait EventListener: Send + Sync {
    /// Handle received event
    async fn handle_event(&mut self, event: &PluginEvent) -> Result<()>;
    
    /// Get event types this listener is interested in
    fn interested_events(&self) -> Vec<EventType>;
}

/// Event publisher utility
pub struct EventPublisher {
    event_bus: Arc<dyn EventBus>,
}

impl EventPublisher {
    pub fn new(event_bus: Arc<dyn EventBus>) -> Self {
        Self { event_bus }
    }
    
    pub async fn workflow_started(&self, workflow_id: String) -> Result<()> {
        self.event_bus.publish(PluginEvent::WorkflowStarted {
            workflow_id,
            timestamp: Utc::now(),
        }).await
    }
    
    pub async fn workflow_completed(&self, workflow_id: String, result: WorkflowResult) -> Result<()> {
        self.event_bus.publish(PluginEvent::WorkflowCompleted {
            workflow_id,
            result,
            timestamp: Utc::now(),
        }).await
    }
    
    pub async fn action_executed(
        &self,
        action: String,
        duration: std::time::Duration,
        success: bool,
    ) -> Result<()> {
        self.event_bus.publish(PluginEvent::ActionExecuted {
            action,
            duration,
            success,
            timestamp: Utc::now(),
        }).await
    }
    
    pub async fn browser_navigated(&self, url: String, session_id: String) -> Result<()> {
        self.event_bus.publish(PluginEvent::BrowserNavigated {
            url,
            session_id,
            timestamp: Utc::now(),
        }).await
    }
    
    pub async fn error_occurred(
        &self,
        error: String,
        severity: ErrorSeverity,
        plugin_id: Option<PluginId>,
    ) -> Result<()> {
        self.event_bus.publish(PluginEvent::ErrorOccurred {
            error,
            severity,
            plugin_id,
            timestamp: Utc::now(),
        }).await
    }
    
    pub async fn metrics_updated(&self, metrics: serde_json::Value) -> Result<()> {
        self.event_bus.publish(PluginEvent::MetricsUpdated {
            metrics,
            timestamp: Utc::now(),
        }).await
    }
    
    pub async fn configuration_changed(&self, section: String) -> Result<()> {
        self.event_bus.publish(PluginEvent::ConfigurationChanged {
            section,
            timestamp: Utc::now(),
        }).await
    }
    
    pub async fn plugin_loaded(&self, plugin_id: PluginId) -> Result<()> {
        self.event_bus.publish(PluginEvent::PluginLoaded {
            plugin_id,
            timestamp: Utc::now(),
        }).await
    }
    
    pub async fn plugin_unloaded(&self, plugin_id: PluginId) -> Result<()> {
        self.event_bus.publish(PluginEvent::PluginUnloaded {
            plugin_id,
            timestamp: Utc::now(),
        }).await
    }
}