// Workflow Orchestrator - Removes God Object anti-pattern from RainbowBrowserV8
use anyhow::Result;
use std::sync::Arc;
use uuid::Uuid;
use async_trait::async_trait;

use crate::{
    traits::*,
    unified_kernel::{Session, SessionConfig},
    layered_perception::PerceptionMode,
    optimized_persistence::{MemoryData, DataType},
    SystemStatus,
};

/// Workflow context for passing state between workflow steps
pub struct WorkflowContext {
    pub session: Session,
    pub user_request: String,
    pub perception_result: Option<crate::layered_perception::PerceptionResult>,
    pub action_results: Vec<crate::intelligent_action::ActionResult>,
}

/// Trait for workflow steps
#[async_trait]
pub trait WorkflowStep: Send + Sync {
    async fn execute(&self, context: &mut WorkflowContext) -> Result<()>;
    fn name(&self) -> &str;
}

/// Session creation step
pub struct CreateSessionStep {
    kernel: Arc<dyn KernelEngine>,
}

impl CreateSessionStep {
    pub fn new(kernel: Arc<dyn KernelEngine>) -> Self {
        Self { kernel }
    }
}

#[async_trait]
impl WorkflowStep for CreateSessionStep {
    async fn execute(&self, context: &mut WorkflowContext) -> Result<()> {
        let config = SessionConfig::new("https://www.google.com")
            .with_perception_mode(PerceptionMode::Standard);
        context.session = self.kernel.create_session(config).await?;
        Ok(())
    }

    fn name(&self) -> &str {
        "CreateSession"
    }
}

/// Monitoring step
pub struct StartMonitoringStep {
    performance_monitor: Arc<dyn PerformanceMonitor>,
    stability_monitor: Arc<dyn StabilityMonitor>,
}

impl StartMonitoringStep {
    pub fn new(
        performance_monitor: Arc<dyn PerformanceMonitor>,
        stability_monitor: Arc<dyn StabilityMonitor>,
    ) -> Self {
        Self {
            performance_monitor,
            stability_monitor,
        }
    }
}

#[async_trait]
impl WorkflowStep for StartMonitoringStep {
    async fn execute(&self, context: &mut WorkflowContext) -> Result<()> {
        self.performance_monitor.start_monitoring(&context.session).await?;
        self.stability_monitor.health_check(&context.session).await?;
        Ok(())
    }

    fn name(&self) -> &str {
        "StartMonitoring"
    }
}

/// Perception step
pub struct PerceptionStep {
    perception_engine: Arc<dyn PerceptionEngine>,
}

impl PerceptionStep {
    pub fn new(perception_engine: Arc<dyn PerceptionEngine>) -> Self {
        Self { perception_engine }
    }
}

#[async_trait]
impl WorkflowStep for PerceptionStep {
    async fn execute(&self, context: &mut WorkflowContext) -> Result<()> {
        let result = self.perception_engine.perceive(
            &context.session.config.url,
            context.session.config.perception_mode,
        ).await?;
        context.perception_result = Some(result);
        Ok(())
    }

    fn name(&self) -> &str {
        "Perception"
    }
}

/// Action execution step
pub struct ActionExecutionStep {
    action_engine: Arc<dyn ActionEngine>,
}

impl ActionExecutionStep {
    pub fn new(action_engine: Arc<dyn ActionEngine>) -> Self {
        Self { action_engine }
    }
}

#[async_trait]
impl WorkflowStep for ActionExecutionStep {
    async fn execute(&self, context: &mut WorkflowContext) -> Result<()> {
        let results = self.action_engine.execute_smart_task(
            context.session.id,
            &context.user_request,
        ).await?;
        context.action_results = results;
        Ok(())
    }

    fn name(&self) -> &str {
        "ActionExecution"
    }
}

/// Memory storage step
pub struct MemoryStorageStep {
    persistence_engine: Arc<dyn PersistenceEngine>,
}

impl MemoryStorageStep {
    pub fn new(persistence_engine: Arc<dyn PersistenceEngine>) -> Self {
        Self { persistence_engine }
    }
}

#[async_trait]
impl WorkflowStep for MemoryStorageStep {
    async fn execute(&self, context: &mut WorkflowContext) -> Result<()> {
        let memory = MemoryData {
            id: Uuid::new_v4(),
            session_id: context.session.id,
            timestamp: std::time::SystemTime::now(),
            data_type: DataType::Experience,
            content: serde_json::json!({
                "request": context.user_request,
                "perception": context.perception_result,
                "actions": context.action_results,
            }),
            metadata: std::collections::HashMap::new(),
        };
        self.persistence_engine.store(memory).await?;
        Ok(())
    }

    fn name(&self) -> &str {
        "MemoryStorage"
    }
}

/// Session cleanup step
pub struct CleanupSessionStep {
    kernel: Arc<dyn KernelEngine>,
}

impl CleanupSessionStep {
    pub fn new(kernel: Arc<dyn KernelEngine>) -> Self {
        Self { kernel }
    }
}

#[async_trait]
impl WorkflowStep for CleanupSessionStep {
    async fn execute(&self, context: &mut WorkflowContext) -> Result<()> {
        self.kernel.destroy_session(&context.session.id).await?;
        Ok(())
    }

    fn name(&self) -> &str {
        "CleanupSession"
    }
}

/// Main workflow orchestrator
pub struct WorkflowOrchestrator {
    steps: Vec<Arc<dyn WorkflowStep>>,
}

impl WorkflowOrchestrator {
    pub fn new() -> Self {
        Self { steps: Vec::new() }
    }

    /// Add a workflow step
    pub fn add_step(&mut self, step: Arc<dyn WorkflowStep>) {
        self.steps.push(step);
    }

    /// Execute the complete workflow
    pub async fn execute(&self, user_request: &str) -> Result<String> {
        let mut context = WorkflowContext {
            session: Session::default(),
            user_request: user_request.to_string(),
            perception_result: None,
            action_results: Vec::new(),
        };

        // Execute each step in sequence
        for step in &self.steps {
            log::debug!("Executing workflow step: {}", step.name());
            step.execute(&mut context).await?;
        }

        // Generate response
        let response = format!(
            "✨ 任务完成！\n执行了 {} 个操作\n感知模式: {:?}",
            context.action_results.len(),
            context.session.config.perception_mode
        );

        Ok(response)
    }

    /// Execute with error recovery
    pub async fn execute_with_recovery(&self, user_request: &str) -> Result<String> {
        match self.execute(user_request).await {
            Ok(response) => Ok(response),
            Err(e) => {
                log::error!("Workflow failed: {}", e);
                // Attempt recovery or return graceful error
                Err(e)
            }
        }
    }
}

/// Builder for creating workflow orchestrators
pub struct WorkflowBuilder {
    orchestrator: WorkflowOrchestrator,
}

impl WorkflowBuilder {
    pub fn new() -> Self {
        Self {
            orchestrator: WorkflowOrchestrator::new(),
        }
    }

    pub fn with_session_management(mut self, kernel: Arc<dyn KernelEngine>) -> Self {
        self.orchestrator.add_step(Arc::new(CreateSessionStep::new(kernel.clone())));
        // Cleanup will be added at the end
        self
    }

    pub fn with_monitoring(
        mut self,
        performance: Arc<dyn PerformanceMonitor>,
        stability: Arc<dyn StabilityMonitor>,
    ) -> Self {
        self.orchestrator.add_step(Arc::new(StartMonitoringStep::new(performance, stability)));
        self
    }

    pub fn with_perception(mut self, perception: Arc<dyn PerceptionEngine>) -> Self {
        self.orchestrator.add_step(Arc::new(PerceptionStep::new(perception)));
        self
    }

    pub fn with_action_execution(mut self, action: Arc<dyn ActionEngine>) -> Self {
        self.orchestrator.add_step(Arc::new(ActionExecutionStep::new(action)));
        self
    }

    pub fn with_memory_storage(mut self, persistence: Arc<dyn PersistenceEngine>) -> Self {
        self.orchestrator.add_step(Arc::new(MemoryStorageStep::new(persistence)));
        self
    }

    pub fn with_cleanup(mut self, kernel: Arc<dyn KernelEngine>) -> Self {
        self.orchestrator.add_step(Arc::new(CleanupSessionStep::new(kernel)));
        self
    }

    pub fn build(self) -> WorkflowOrchestrator {
        self.orchestrator
    }
}