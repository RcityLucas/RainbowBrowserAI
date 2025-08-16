// Factory pattern for engine creation - reduces coupling
use anyhow::Result;
use std::sync::Arc;
use async_trait::async_trait;

use crate::{
    traits::*,
    unified_kernel::UnifiedKernel,
    layered_perception::LayeredPerception,
    intelligent_action::IntelligentAction,
    optimized_persistence::OptimizedPersistence,
    performance_engine::PerformanceEngine,
    stability_engine::StabilityEngine,
};

/// Configuration for engine creation
#[derive(Debug, Clone, Default)]
pub struct EngineConfig {
    pub enable_caching: bool,
    pub enable_auto_recovery: bool,
    pub max_memory_size: Option<usize>,
    pub performance_monitoring_interval: Option<u64>,
}

/// Abstract factory trait for creating engines
#[async_trait]
pub trait EngineFactory: Send + Sync {
    type KernelType: KernelEngine;
    type PerceptionType: PerceptionEngine;
    type ActionType: ActionEngine;
    type PersistenceType: PersistenceEngine;
    type PerformanceType: PerformanceMonitor;
    type StabilityType: StabilityMonitor;

    async fn create_kernel(&self, config: &EngineConfig) -> Result<Arc<Self::KernelType>>;
    async fn create_perception(&self, config: &EngineConfig) -> Result<Arc<Self::PerceptionType>>;
    async fn create_action(&self, config: &EngineConfig) -> Result<Arc<Self::ActionType>>;
    async fn create_persistence(&self, config: &EngineConfig) -> Result<Arc<Self::PersistenceType>>;
    async fn create_performance(&self, config: &EngineConfig) -> Result<Arc<Self::PerformanceType>>;
    async fn create_stability(&self, config: &EngineConfig) -> Result<Arc<Self::StabilityType>>;
}

/// Default engine factory implementation
pub struct DefaultEngineFactory;

#[async_trait]
impl EngineFactory for DefaultEngineFactory {
    type KernelType = UnifiedKernel;
    type PerceptionType = LayeredPerception;
    type ActionType = IntelligentAction;
    type PersistenceType = OptimizedPersistence;
    type PerformanceType = PerformanceEngine;
    type StabilityType = StabilityEngine;

    async fn create_kernel(&self, _config: &EngineConfig) -> Result<Arc<Self::KernelType>> {
        Ok(Arc::new(UnifiedKernel::new().await?))
    }

    async fn create_perception(&self, _config: &EngineConfig) -> Result<Arc<Self::PerceptionType>> {
        Ok(Arc::new(LayeredPerception::new().await?))
    }

    async fn create_action(&self, _config: &EngineConfig) -> Result<Arc<Self::ActionType>> {
        Ok(Arc::new(IntelligentAction::new().await?))
    }

    async fn create_persistence(&self, _config: &EngineConfig) -> Result<Arc<Self::PersistenceType>> {
        Ok(Arc::new(OptimizedPersistence::new().await?))
    }

    async fn create_performance(&self, _config: &EngineConfig) -> Result<Arc<Self::PerformanceType>> {
        Ok(Arc::new(PerformanceEngine::new().await?))
    }

    async fn create_stability(&self, config: &EngineConfig) -> Result<Arc<Self::StabilityType>> {
        let engine = Arc::new(StabilityEngine::new().await?);
        if config.enable_auto_recovery {
            engine.enable_auto_recovery().await?;
        }
        Ok(engine)
    }
}

/// Test factory for unit testing with mock implementations
#[cfg(test)]
pub struct TestEngineFactory;

#[cfg(test)]
mod test_implementations {
    use super::*;
    use crate::{
        unified_kernel::{Session, SessionConfig, HealthStatus},
        layered_perception::{PerceptionMode, PerceptionResult, Context},
        intelligent_action::{Action, ActionResult},
        optimized_persistence::{MemoryData, MemoryStatistics},
        performance_engine::PerformanceReport,
        stability_engine::StabilityReport,
    };
    use uuid::Uuid;

    pub struct MockKernel;
    pub struct MockPerception;
    pub struct MockAction;
    pub struct MockPersistence;
    pub struct MockPerformance;
    pub struct MockStability;

    #[async_trait]
    impl KernelEngine for MockKernel {
        async fn create_session(&self, config: SessionConfig) -> Result<Session> {
            Ok(Session::new(config))
        }
        async fn destroy_session(&self, _session_id: &Uuid) -> Result<()> {
            Ok(())
        }
        async fn get_health_status(&self) -> Result<HealthStatus> {
            Ok(HealthStatus::default())
        }
    }

    #[async_trait]
    impl PerceptionEngine for MockPerception {
        async fn perceive(&self, _url: &str, _mode: PerceptionMode) -> Result<PerceptionResult> {
            unimplemented!("Mock implementation")
        }
        async fn adaptive_perceive(&self, _url: &str, _context: &Context) -> Result<PerceptionResult> {
            unimplemented!("Mock implementation")
        }
    }

    #[async_trait]
    impl ActionEngine for MockAction {
        async fn execute_action(&self, _session_id: Uuid, _action: Action) -> Result<ActionResult> {
            unimplemented!("Mock implementation")
        }
        async fn execute_smart_task(&self, _session_id: Uuid, _user_request: &str) -> Result<Vec<ActionResult>> {
            Ok(Vec::new())
        }
        async fn execute_batch(&self, _session_id: Uuid, _actions: Vec<Action>) -> Result<Vec<ActionResult>> {
            Ok(Vec::new())
        }
    }

    #[async_trait]
    impl PersistenceEngine for MockPersistence {
        async fn store(&self, _data: MemoryData) -> Result<()> {
            Ok(())
        }
        async fn retrieve(&self, _id: Uuid) -> Result<Option<MemoryData>> {
            Ok(None)
        }
        async fn get_statistics(&self) -> Result<MemoryStatistics> {
            Ok(MemoryStatistics::default())
        }
        async fn optimize(&self) -> Result<()> {
            Ok(())
        }
    }

    #[async_trait]
    impl PerformanceMonitor for MockPerformance {
        async fn start_monitoring(&self, _session: &Session) -> Result<()> {
            Ok(())
        }
        async fn get_performance_report(&self) -> Result<PerformanceReport> {
            Ok(PerformanceReport::default())
        }
        async fn optimize(&self) -> Result<()> {
            Ok(())
        }
    }

    #[async_trait]
    impl StabilityMonitor for MockStability {
        async fn health_check(&self, _session: &Session) -> Result<()> {
            Ok(())
        }
        async fn get_stability_report(&self) -> Result<StabilityReport> {
            Ok(StabilityReport::default())
        }
        async fn enable_auto_recovery(&self) -> Result<()> {
            Ok(())
        }
    }
}

/// Builder pattern for creating engine configurations
pub struct EngineConfigBuilder {
    config: EngineConfig,
}

impl EngineConfigBuilder {
    pub fn new() -> Self {
        Self {
            config: EngineConfig::default(),
        }
    }

    pub fn with_caching(mut self, enable: bool) -> Self {
        self.config.enable_caching = enable;
        self
    }

    pub fn with_auto_recovery(mut self, enable: bool) -> Self {
        self.config.enable_auto_recovery = enable;
        self
    }

    pub fn with_max_memory(mut self, size: usize) -> Self {
        self.config.max_memory_size = Some(size);
        self
    }

    pub fn with_monitoring_interval(mut self, interval_ms: u64) -> Self {
        self.config.performance_monitoring_interval = Some(interval_ms);
        self
    }

    pub fn build(self) -> EngineConfig {
        self.config
    }
}