// Trait abstractions for major engines - Dependency Inversion Principle
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::sync::Arc;

// Import necessary types
use crate::{
    unified_kernel::{Session, SessionConfig, HealthStatus},
    layered_perception::{PerceptionMode, PerceptionResult},
    intelligent_action::{Action, ActionResult},
    optimized_persistence::{MemoryData, MemoryStatistics},
    performance_engine::PerformanceReport,
    stability_engine::StabilityReport,
};

/// Trait for the Unified Kernel engine
#[async_trait]
pub trait KernelEngine: Send + Sync {
    async fn create_session(&self, config: SessionConfig) -> Result<Session>;
    async fn destroy_session(&self, session_id: &Uuid) -> Result<()>;
    async fn get_health_status(&self) -> Result<HealthStatus>;
}

/// Trait for the Perception engine
#[async_trait]
pub trait PerceptionEngine: Send + Sync {
    async fn perceive(&self, url: &str, mode: PerceptionMode) -> Result<PerceptionResult>;
    async fn adaptive_perceive(&self, url: &str, context: &crate::layered_perception::Context) -> Result<PerceptionResult>;
}

/// Trait for the Action engine
#[async_trait]
pub trait ActionEngine: Send + Sync {
    async fn execute_action(&self, session_id: Uuid, action: Action) -> Result<ActionResult>;
    async fn execute_smart_task(&self, session_id: Uuid, user_request: &str) -> Result<Vec<ActionResult>>;
    async fn execute_batch(&self, session_id: Uuid, actions: Vec<Action>) -> Result<Vec<ActionResult>>;
}

/// Trait for the Persistence engine
#[async_trait]
pub trait PersistenceEngine: Send + Sync {
    async fn store(&self, data: MemoryData) -> Result<()>;
    async fn retrieve(&self, id: Uuid) -> Result<Option<MemoryData>>;
    async fn get_statistics(&self) -> Result<MemoryStatistics>;
    async fn optimize(&self) -> Result<()>;
}

/// Trait for the Performance engine
#[async_trait]
pub trait PerformanceMonitor: Send + Sync {
    async fn start_monitoring(&self, session: &Session) -> Result<()>;
    async fn get_performance_report(&self) -> Result<PerformanceReport>;
    async fn optimize(&self) -> Result<()>;
}

/// Trait for the Stability engine
#[async_trait]
pub trait StabilityMonitor: Send + Sync {
    async fn health_check(&self, session: &Session) -> Result<()>;
    async fn get_stability_report(&self) -> Result<StabilityReport>;
    async fn enable_auto_recovery(&self) -> Result<()>;
}