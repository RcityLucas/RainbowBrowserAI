pub mod browser;
pub mod cost_tracker;
pub mod config;
pub mod llm_service;
pub mod context;
pub mod workflow;
pub mod browser_pool;
pub mod cache;
pub mod metrics;
pub mod security;
pub mod api;
pub mod plugins;
// Removed: extractor and chromedriver_manager modules
pub mod task_executor;
// pub mod intelligence; // Disabled due to compilation complexity - using simplified version instead
pub mod organic_perception_simple;
pub mod organic_perception_enhanced;
pub mod simple_memory;
pub mod contextual_awareness;
pub mod contextual_perception;
pub mod creative_engine;
pub mod command_registry;
// Removed: execution_engine module
pub mod enhanced_browser;
pub mod smart_actions;
pub mod intent_translator;
pub mod adaptive_pipeline;
pub mod llm_integration;
pub mod mock_llm_provider;
pub mod error_recovery;
pub mod health_monitor;
pub mod config_manager;
pub mod advanced_learning;
pub mod multi_model_orchestration;
pub mod self_healing;
pub mod advanced_analytics;
// Tools module - temporarily disabled due to compilation issues - will test separately
// pub mod tools;

pub use browser::{SimpleBrowser, ScreenshotOptions};
pub use cost_tracker::CostTracker;
pub use config::{Config, ApiConfig};
pub use llm_service::{LLMService, ParsedCommand, CommandParams};
pub use llm_service::llm_service_enhanced::{TaskPlan, ActionStep, TaskType, TaskUnderstanding, MockTaskUnderstanding};
pub use context::{ConversationContext, HistoryEntry, ExecutionResult};
pub use workflow::{Workflow, WorkflowEngine, WorkflowResult, WorkflowStep, ActionType};
pub use browser_pool::{BrowserPool, PooledBrowserHandle};
pub use cache::{Cache, LLMCache, WorkflowCache};
pub use metrics::{MetricsCollector, Metrics, MetricsSummary};
pub use security::{SecurityConfig, SecurityMiddleware, RateLimiter, InputValidator};
pub use api::{create_router, start_server, ApiState};
pub use plugins::{PluginManager, init_plugin_system};
// Removed: extractor exports
pub use task_executor::{TaskExecutor, TaskExecutionResult, ExecutionProgress, AggregatedResults};
// pub use intelligence::{OrganicPerception, IntentUnderstanding, Context, PerceptionMode, IntelligenceStats};
pub use organic_perception_simple::{SimpleOrganicPerception, IntelligenceMode, IntelligenceStats as SimpleIntelligenceStats};
pub use organic_perception_enhanced::{EnhancedOrganicPerception, EnhancedIntelligenceStats, create_enhanced_perception};
pub use simple_memory::{SimpleMemory, SimpleMemoryConfig, InteractionRecord, LearnedPattern, SimpleMemoryStats, create_simple_memory};
pub use contextual_awareness::{ContextualAwareness, ContextSnapshot, ContextualRecommendations, TemporalContext, EnvironmentalContext, UserContext, SystemContext, create_contextual_awareness, create_contextual_awareness_with_memory};
pub use contextual_perception::{ContextualPerception, ContextualTaskUnderstanding, ContextualTaskPlan, ContextualIntelligenceStats, ContextualIntelligenceMode, ExecutionPriority, create_contextual_perception};
pub use creative_engine::{CreativeEngine, CreativeSolution, CreativeTaskPlan, CreativeStrategy, ThinkingMode, ProblemType, ComplexityLevel, CreativeEngineStats, SolutionFeedback, create_creative_engine, create_creative_engine_with_memory};
pub use command_registry::{IntelligentCommandRegistry, CommandDefinition, CommandCategory, CommandSelection, ExecutionRecord, CommandStatistics, CommandBuilder, create_command_registry, create_advanced_registry};
// Removed: execution_engine exports
pub use smart_actions::{SmartActionOrchestrator, SmartActionConfig, ActionType as SmartActionType, SmartActionExecution, SmartActionResult, ActionRecommendation, RecommendationType, Priority, create_smart_action_orchestrator, create_smart_action_orchestrator_with_memory};
pub use intent_translator::{IntentTranslator, TranslatorConfig, TranslationResult, ActionPlan, ActionPlanStep, ExecutionStrategy, TranslationExecution, TranslationStatistics, create_intent_translator, create_advanced_intent_translator, create_intent_translator_with_memory};
pub use adaptive_pipeline::{AdaptivePipeline, PipelineConfig, PipelineResult, PipelineExecution, ExecutionPhase, PipelineMetrics, create_adaptive_pipeline, create_creative_adaptive_pipeline, create_memory_adaptive_pipeline};
pub use llm_integration::{LLMIntegrationManager, LLMConfig, LLMProvider, ModelSelectionStrategy, LLMMetrics, LLMRequest, LLMResponse, IntentUnderstanding, Entity, CreativeSolution as LLMCreativeSolution, ProviderHealth, create_llm_integration_manager, create_custom_llm_integration_manager};
pub use mock_llm_provider::{MockLLMProvider, create_mock_provider, create_custom_mock_provider};
pub use error_recovery::{ErrorRecoveryManager, ErrorRecoveryConfig, ErrorCategory, ErrorSeverity, RecoveryResult, create_error_recovery_manager, create_custom_error_recovery_manager};
pub use health_monitor::{HealthMonitor, HealthMonitorConfig, HealthStatus, SystemHealthMetrics, HealthReport, create_health_monitor, create_custom_health_monitor};
pub use config_manager::{ConfigManager, ConfigManagerConfig, MasterConfiguration, ConfigSource, ValidationResult as ConfigValidationResult, create_config_manager, create_custom_config_manager};
pub use advanced_learning::{AdvancedLearningEngine, AdvancedLearningConfig, LearningAlgorithm, LearningObjective, LearnedPattern as AdvancedLearnedPattern, LearningMetrics, OptimizationRecommendation, create_advanced_learning_engine, create_custom_learning_engine};
pub use multi_model_orchestration::{MultiModelOrchestrator, OrchestrationConfig, OrchestrationStrategy, OrchestrationRequest, OrchestrationResponse, TaskSpecialty, create_multi_model_orchestrator, create_custom_orchestrator};
pub use self_healing::{SelfHealingSystem, SelfHealingConfig, HealingStrategy, OptimizationArea, DetectedIssue, SelfHealingMetrics, create_self_healing_system, create_custom_self_healing_system};
pub use advanced_analytics::{AdvancedAnalyticsEngine, AnalyticsConfig, AnalyticsInsight, AnalyticsReport, TimePeriod, InsightType, create_analytics_engine, create_custom_analytics_engine};