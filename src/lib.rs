// 彩虹城浏览器 8.0 - AI生命体的数字器官
// 六大引擎架构，实现AI在数字世界的感知、行动和记忆

// Core modules
pub mod error;
pub mod types;
pub mod traits;
pub mod factory;
pub mod orchestrator;
pub mod events;
pub mod trait_impls;
pub mod features;
pub mod simplified_traits;
pub mod user_api;

// Feature-based module loading
pub use features::Features;

// Conditional standalone browser module
#[cfg(feature = "standalone")]
pub mod standalone_browser;

// Initialize features on library load
pub fn init() {
    features::initialize_features();
}

// Core layers
pub mod base;
pub mod core;
pub mod apps;

// Six engines architecture
pub mod unified_kernel;
pub mod layered_perception;
pub mod intelligent_action;
pub mod optimized_persistence;
pub mod performance_engine;
pub mod stability_engine;

use anyhow::Result;
use std::sync::Arc;
use crate::factory::EngineFactory;
use crate::events::EventPublisher;

// 重新导出核心类型
pub use unified_kernel::{UnifiedKernel, Session, SessionConfig};
pub use layered_perception::{LayeredPerception, PerceptionResult, PerceptionMode};
pub use intelligent_action::{IntelligentAction, Action, ActionResult};
pub use optimized_persistence::{OptimizedPersistence, MemoryData, DataType};
pub use performance_engine::{PerformanceEngine, PerformanceReport};
pub use stability_engine::{StabilityEngine, StabilityReport};

/// 彩虹城浏览器 8.0 - 主结构 (Refactored with DIP)
pub struct RainbowBrowserV8 {
    // Using trait objects for dependency inversion
    pub unified_kernel: Arc<dyn traits::KernelEngine>,
    pub layered_perception: Arc<dyn traits::PerceptionEngine>,
    pub intelligent_action: Arc<dyn traits::ActionEngine>,
    pub optimized_persistence: Arc<dyn traits::PersistenceEngine>,
    pub performance_engine: Arc<dyn traits::PerformanceMonitor>,
    pub stability_engine: Arc<dyn traits::StabilityMonitor>,
    
    // Event bus for decoupled communication
    pub event_bus: Arc<events::EventBus>,
    
    // Workflow orchestrator to avoid God Object pattern
    pub orchestrator: Arc<orchestrator::WorkflowOrchestrator>,
}

impl RainbowBrowserV8 {
    /// 创建AI生命体 (Using Factory Pattern)
    pub async fn new() -> Result<Self> {
        log::info!("🌈 彩虹城浏览器 8.0 - AI生命体觉醒");
        
        // Use factory pattern for engine creation
        let factory = factory::DefaultEngineFactory;
        let config = factory::EngineConfigBuilder::new()
            .with_caching(true)
            .with_auto_recovery(true)
            .build();
        
        // Create engines using factory
        let unified_kernel = factory.create_kernel(&config).await?;
        let layered_perception = factory.create_perception(&config).await?;
        let intelligent_action = factory.create_action(&config).await?;
        let optimized_persistence = factory.create_persistence(&config).await?;
        let performance_engine = factory.create_performance(&config).await?;
        let stability_engine = factory.create_stability(&config).await?;
        
        // Create event bus for decoupled communication
        let event_bus = events::EventSystemBuilder::new()
            .with_max_history(1000)
            .add_logging("system".to_string())
            .add_metrics("performance".to_string())
            .build()
            .await;
        
        // Build workflow orchestrator
        let orchestrator = Arc::new(
            orchestrator::WorkflowBuilder::new()
                .with_session_management(unified_kernel.clone())
                .with_monitoring(performance_engine.clone(), stability_engine.clone())
                .with_perception(layered_perception.clone())
                .with_action_execution(intelligent_action.clone())
                .with_memory_storage(optimized_persistence.clone())
                .with_cleanup(unified_kernel.clone())
                .build()
        );
        
        Ok(Self {
            unified_kernel,
            layered_perception,
            intelligent_action,
            optimized_persistence,
            performance_engine,
            stability_engine,
            event_bus,
            orchestrator,
        })
    }
    
    /// 处理用户请求 - Delegated to Workflow Orchestrator
    pub async fn process_request(&self, user_request: &str) -> Result<String> {
        // Publish event for request start (using strongly-typed event)
        self.event_bus.publish(events::Event::new(
            events::EventType::RequestStarted,
            "RainbowBrowserV8".to_string(),
        ).with_data(serde_json::json!({ "request": user_request }))).await;
        
        // Delegate to orchestrator (no more God Object!)
        let result = self.orchestrator.execute_with_recovery(user_request).await;
        
        // Publish event for request completion (using strongly-typed event)
        let event_type = if result.is_ok() {
            events::EventType::RequestCompleted
        } else {
            events::EventType::RequestFailed
        };
        
        self.event_bus.publish(events::Event::new(
            event_type,
            "RainbowBrowserV8".to_string(),
        ).with_data(serde_json::json!({ 
            "request": user_request,
            "success": result.is_ok() 
        }))).await;
        
        result
    }
    
    /// 获取系统状态
    pub async fn get_system_status(&self) -> Result<SystemStatus> {
        let health = self.unified_kernel.get_health_status().await?;
        let performance = self.performance_engine.get_performance_report().await?;
        let stability = self.stability_engine.get_stability_report().await?;
        let memory_stats = self.optimized_persistence.get_statistics().await?;
        
        Ok(SystemStatus {
            health,
            performance,
            stability,
            memory_stats,
        })
    }
    
    /// 优化系统
    pub async fn optimize(&self) -> Result<()> {
        log::info!("开始系统优化");
        
        // 性能优化
        self.performance_engine.optimize().await?;
        
        // 存储优化
        self.optimized_persistence.optimize().await?;
        
        log::info!("系统优化完成");
        Ok(())
    }
}

/// 系统状态
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SystemStatus {
    pub health: unified_kernel::HealthStatus,
    pub performance: PerformanceReport,
    pub stability: StabilityReport,
    pub memory_stats: optimized_persistence::MemoryStatistics,
}

/// 预设导出
pub mod prelude {
    pub use super::{
        RainbowBrowserV8,
        Session, SessionConfig, PerceptionMode,
        Action, ActionResult,
        MemoryData, DataType,
        SystemStatus,
    };
}