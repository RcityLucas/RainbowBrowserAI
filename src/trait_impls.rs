// Trait implementations for engines to enable DIP compliance
use async_trait::async_trait;
use anyhow::Result;
use uuid::Uuid;

use crate::{
    traits::*,
    unified_kernel::{UnifiedKernel, Session, SessionConfig, HealthStatus},
    layered_perception::{LayeredPerception, PerceptionMode, PerceptionResult, Context},
    intelligent_action::{IntelligentAction, Action, ActionResult},
    optimized_persistence::{OptimizedPersistence, MemoryData, MemoryStatistics},
    performance_engine::{PerformanceEngine, PerformanceReport},
    stability_engine::{StabilityEngine, StabilityReport},
};

// UnifiedKernel trait implementation
#[async_trait]
impl KernelEngine for UnifiedKernel {
    async fn create_session(&self, config: SessionConfig) -> Result<Session> {
        self.create_session(config).await
    }

    async fn destroy_session(&self, session_id: &Uuid) -> Result<()> {
        self.destroy_session(session_id).await
    }

    async fn get_health_status(&self) -> Result<HealthStatus> {
        self.get_health_status().await
    }
}

// LayeredPerception trait implementation
#[async_trait]
impl PerceptionEngine for LayeredPerception {
    async fn perceive(&self, url: &str, mode: PerceptionMode) -> Result<PerceptionResult> {
        self.perceive(url, mode).await
    }

    async fn adaptive_perceive(&self, url: &str, context: &Context) -> Result<PerceptionResult> {
        self.adaptive_perceive(url, context).await
    }
}

// IntelligentAction trait implementation
#[async_trait]
impl ActionEngine for IntelligentAction {
    async fn execute_action(&self, session_id: Uuid, action: Action) -> Result<ActionResult> {
        self.execute_action(session_id, action).await
    }

    async fn execute_smart_task(&self, session_id: Uuid, user_request: &str) -> Result<Vec<ActionResult>> {
        self.execute_smart_task(session_id, user_request).await
    }

    async fn execute_batch(&self, session_id: Uuid, actions: Vec<Action>) -> Result<Vec<ActionResult>> {
        self.execute_batch(session_id, actions).await
    }
}

// OptimizedPersistence trait implementation
#[async_trait]
impl PersistenceEngine for OptimizedPersistence {
    async fn store(&self, data: MemoryData) -> Result<()> {
        self.store(data).await
    }

    async fn retrieve(&self, id: Uuid) -> Result<Option<MemoryData>> {
        self.retrieve(id).await
    }

    async fn get_statistics(&self) -> Result<MemoryStatistics> {
        self.get_statistics().await
    }

    async fn optimize(&self) -> Result<()> {
        self.optimize().await
    }
}

// PerformanceEngine trait implementation
#[async_trait]
impl PerformanceMonitor for PerformanceEngine {
    async fn start_monitoring(&self, session: &Session) -> Result<()> {
        self.start_monitoring(session).await
    }

    async fn get_performance_report(&self) -> Result<PerformanceReport> {
        self.get_performance_report().await
    }

    async fn optimize(&self) -> Result<()> {
        self.optimize().await
    }
}

// StabilityEngine trait implementation
#[async_trait]
impl StabilityMonitor for StabilityEngine {
    async fn health_check(&self, session: &Session) -> Result<()> {
        self.health_check(session).await
    }

    async fn get_stability_report(&self) -> Result<StabilityReport> {
        self.get_stability_report().await
    }

    async fn enable_auto_recovery(&self) -> Result<()> {
        self.enable_auto_recovery().await
    }
}