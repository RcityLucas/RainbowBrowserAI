// 彩虹城浏览器 8.0 - AI生命体的数字器官
// 六大引擎架构，实现AI在数字世界的感知、行动和记忆

// Core modules
pub mod error;
pub mod types;

// 独立浏览器模块
#[cfg(feature = "standalone")]
pub mod standalone_browser;

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

// 重新导出核心类型
pub use unified_kernel::{UnifiedKernel, Session, SessionConfig};
pub use layered_perception::{LayeredPerception, PerceptionResult, PerceptionMode};
pub use intelligent_action::{IntelligentAction, Action, ActionResult};
pub use optimized_persistence::{OptimizedPersistence, MemoryData, DataType};
pub use performance_engine::{PerformanceEngine, PerformanceReport};
pub use stability_engine::{StabilityEngine, StabilityReport};

/// 彩虹城浏览器 8.0 - 主结构
pub struct RainbowBrowserV8 {
    // 六大生命器官
    pub unified_kernel: Arc<UnifiedKernel>,
    pub layered_perception: Arc<LayeredPerception>,
    pub intelligent_action: Arc<IntelligentAction>,
    pub optimized_persistence: Arc<OptimizedPersistence>,
    pub performance_engine: Arc<PerformanceEngine>,
    pub stability_engine: Arc<StabilityEngine>,
}

impl RainbowBrowserV8 {
    /// 创建AI生命体
    pub async fn new() -> Result<Self> {
        log::info!("🌈 彩虹城浏览器 8.0 - AI生命体觉醒");
        
        // 初始化六大引擎
        let unified_kernel = Arc::new(UnifiedKernel::new().await?);
        let layered_perception = Arc::new(LayeredPerception::new().await?);
        let intelligent_action = Arc::new(IntelligentAction::new().await?);
        let optimized_persistence = Arc::new(OptimizedPersistence::new().await?);
        let performance_engine = Arc::new(PerformanceEngine::new().await?);
        let stability_engine = Arc::new(StabilityEngine::new().await?);
        
        // 启动自动恢复
        stability_engine.enable_auto_recovery().await?;
        
        Ok(Self {
            unified_kernel,
            layered_perception,
            intelligent_action,
            optimized_persistence,
            performance_engine,
            stability_engine,
        })
    }
    
    /// 处理用户请求 - 完整的生命活动
    pub async fn process_request(&self, user_request: &str) -> Result<String> {
        // 1. 创建会话（生命周期开始）
        let config = SessionConfig::new("https://www.google.com")
            .with_perception_mode(PerceptionMode::Standard);
        let session = self.unified_kernel.create_session(config).await?;
        
        // 2. 开始性能监控
        self.performance_engine.start_monitoring(&session).await?;
        
        // 3. 健康检查
        self.stability_engine.health_check(&session).await?;
        
        // 4. 感知环境
        let perception = self.layered_perception.perceive(
            &session.config.url,
            session.config.perception_mode,
        ).await?;
        
        // 5. 智能执行任务
        let action_results = self.intelligent_action.execute_smart_task(
            session.id,
            user_request,
        ).await?;
        
        // 6. 存储记忆
        let memory = MemoryData {
            id: uuid::Uuid::new_v4(),
            session_id: session.id,
            timestamp: std::time::SystemTime::now(),
            data_type: DataType::Experience,
            content: serde_json::json!({
                "request": user_request,
                "perception": perception,
                "actions": action_results,
            }),
            metadata: std::collections::HashMap::new(),
        };
        self.optimized_persistence.store(memory).await?;
        
        // 7. 生成响应
        let response = format!(
            "✨ 任务完成！\n执行了 {} 个操作\n感知模式: {:?}",
            action_results.len(),
            session.config.perception_mode
        );
        
        // 8. 销毁会话（生命周期结束）
        self.unified_kernel.destroy_session(&session.id).await?;
        
        Ok(response)
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