// Core modules required for start.sh
pub mod browser;
pub mod config;
pub mod llm_service;
pub mod api;
pub mod api_client;
pub mod mock_llm_provider;

// Supporting modules used by core
pub mod context;
pub mod workflow;
pub mod browser_pool;
pub mod metrics;
pub mod security;
pub mod cost_tracker;

// Plugin system (optional but lightweight)
pub mod plugins;

// Simple implementations kept for basic functionality
pub mod cache;
pub mod task_executor;
pub mod health_monitor;
pub mod error_recovery;

// Required dependencies for mock_llm_provider and health_monitor
pub mod llm_integration;
pub mod contextual_awareness;
pub mod simple_memory;

// New enhanced modules for instruction parsing and extraction
pub mod instruction_parser;
pub mod semantic_analyzer;
pub mod action_mapper;
pub mod enhanced_executor;  // Re-enabled after fixing perception_mvp
// pub mod api_v2;  // Depends on enhanced_executor

// Enhanced perception system - MVP and complete implementation
// pub mod perception_mvp;  // Temporarily disabled for core action testing
// pub mod final_integration; // Temporarily disabled due to compilation issues

// Simple working perception module  
// pub mod perception_simple; // Temporarily disabled for core action testing

// Continuous improvement pipeline for automated learning and optimization
pub mod continuous_improvement_pipeline;

// A/B testing framework for systematic improvement validation
// pub mod ab_testing_framework; // Module file doesn't exist

// Tool orchestration system - depends on tools module
// pub mod tool_orchestrator;

// V8 Perception system
pub mod v8_perception;

// Tools module - re-enabling for perception integration
// pub mod tools; // Temporarily disabled for core action testing

// Commented out non-essential advanced modules
// These can be re-enabled when needed for advanced features
// pub mod organic_perception_simple;
// pub mod organic_perception_enhanced;
// pub mod simple_memory;
// pub mod contextual_perception;
// pub mod creative_engine;
// pub mod command_registry;
// pub mod enhanced_browser;
// pub mod smart_actions;
// pub mod intent_translator;
// pub mod adaptive_pipeline;
// pub mod config_manager;
// pub mod advanced_learning;
// pub mod multi_model_orchestration;
// pub mod self_healing;
// pub mod advanced_analytics;
// Tools module - temporarily disabled due to existing compilation issues
// pub mod tools;

// Core exports required for start.sh
pub use browser::{SimpleBrowser, ScreenshotOptions};
pub use config::{Config, ApiConfig};
pub use llm_service::{LLMService, ParsedCommand, CommandParams};
pub use llm_service::llm_service_enhanced::{TaskPlan, ActionStep, TaskType, TaskUnderstanding, MockTaskUnderstanding};
pub use api::{create_router, start_server, ApiState};
pub use mock_llm_provider::{MockLLMProvider, create_mock_provider, create_custom_mock_provider};

// Supporting exports
pub use context::{ConversationContext, HistoryEntry, ExecutionResult};
pub use workflow::{Workflow, WorkflowEngine, WorkflowResult, WorkflowStep, ActionType};
pub use browser_pool::{BrowserPool, PooledBrowserHandle};
pub use metrics::{MetricsCollector, Metrics, MetricsSummary};
pub use security::{SecurityConfig, SecurityMiddleware, RateLimiter, InputValidator};
pub use cost_tracker::CostTracker;
pub use plugins::{PluginManager, init_plugin_system};
pub use cache::{Cache, LLMCache, WorkflowCache};
pub use task_executor::{TaskExecutor, TaskExecutionResult, ExecutionProgress, AggregatedResults};
pub use health_monitor::{HealthMonitor, HealthMonitorConfig, HealthStatus, SystemHealthMetrics, HealthReport, create_health_monitor, create_custom_health_monitor};
pub use error_recovery::{ErrorRecoveryManager, ErrorRecoveryConfig, ErrorCategory, ErrorSeverity, RecoveryResult, create_error_recovery_manager, create_custom_error_recovery_manager};
pub use llm_integration::{LLMIntegrationManager, LLMConfig, LLMProvider, ModelSelectionStrategy, LLMMetrics, LLMRequest, LLMResponse, IntentUnderstanding, Entity, CreativeSolution as LLMCreativeSolution, ProviderHealth, create_llm_integration_manager, create_custom_llm_integration_manager};
pub use contextual_awareness::{ContextualAwareness, ContextSnapshot, ContextualRecommendations, TemporalContext, EnvironmentalContext, UserContext, SystemContext, create_contextual_awareness, create_contextual_awareness_with_memory};
pub use simple_memory::{SimpleMemory, SimpleMemoryConfig, InteractionRecord, LearnedPattern, SimpleMemoryStats, create_simple_memory};

// New enhanced instruction parsing and extraction exports
pub use instruction_parser::{InstructionParser, UserInstruction, ContextHints, Feedback, PageType};
pub use semantic_analyzer::{SemanticAnalyzer, SemanticPageModel, PageRegion, SemanticElement};
pub use action_mapper::{ActionMapper, ExecutableAction, ActionExecutor, ActionResult};
// pub use enhanced_executor::{EnhancedExecutor, EnhancedCommandProcessor, demo_enhanced_execution};
// pub use api_v2::{ApiV2State, create_v2_routes};

// Enhanced perception system exports
// pub use perception_mvp::{PerceptionEngineMVP, PerceivedElement, PageType as PerceptionPageType};
// pub use final_integration::{UnifiedBrowserSystem, UnifiedCommand, UnifiedCommandResult};

// Simple perception module exports
// pub use perception_simple::{SimplePerception, FoundElement, PageType as SimplePageType, PerceptionExecutor};

// Continuous improvement pipeline exports
pub use continuous_improvement_pipeline::{
    ContinuousImprovementPipeline, PipelineConfig, MetricsCollector as ImprovementMetricsCollector,
    PerformanceMetric, ImprovementReport, UserFeedback, ImprovementStatus
};

// Tools system exports with perception integration
// pub use tools::{
//     Tool, DynamicTool, ToolRegistry, ToolError,
//     PerceptionAnalyzer, PerceptionAnalyzerInput, PerceptionAnalyzerOutput,
//     PerceptionEngine, AnalysisType, PerceptionElement
// };

// Commented out non-essential exports
// pub use organic_perception_simple::{SimpleOrganicPerception, IntelligenceMode, IntelligenceStats as SimpleIntelligenceStats};
// pub use organic_perception_enhanced::{EnhancedOrganicPerception, EnhancedIntelligenceStats, create_enhanced_perception};
// pub use simple_memory::{SimpleMemory, SimpleMemoryConfig, InteractionRecord, LearnedPattern, SimpleMemoryStats, create_simple_memory};
// pub use contextual_awareness::{ContextualAwareness, ContextSnapshot, ContextualRecommendations, TemporalContext, EnvironmentalContext, UserContext, SystemContext, create_contextual_awareness, create_contextual_awareness_with_memory};
// pub use contextual_perception::{ContextualPerception, ContextualTaskUnderstanding, ContextualTaskPlan, ContextualIntelligenceStats, ContextualIntelligenceMode, ExecutionPriority, create_contextual_perception};
// pub use creative_engine::{CreativeEngine, CreativeSolution, CreativeTaskPlan, CreativeStrategy, ThinkingMode, ProblemType, ComplexityLevel, CreativeEngineStats, SolutionFeedback, create_creative_engine, create_creative_engine_with_memory};
// pub use command_registry::{IntelligentCommandRegistry, CommandDefinition, CommandCategory, CommandSelection, ExecutionRecord, CommandStatistics, CommandBuilder, create_command_registry, create_advanced_registry};
// pub use smart_actions::{SmartActionOrchestrator, SmartActionConfig, ActionType as SmartActionType, SmartActionExecution, SmartActionResult, ActionRecommendation, RecommendationType, Priority, create_smart_action_orchestrator, create_smart_action_orchestrator_with_memory};
// pub use intent_translator::{IntentTranslator, TranslatorConfig, TranslationResult, ActionPlan, ActionPlanStep, ExecutionStrategy, TranslationExecution, TranslationStatistics, create_intent_translator, create_advanced_intent_translator, create_intent_translator_with_memory};
// pub use adaptive_pipeline::{AdaptivePipeline, PipelineConfig, PipelineResult, PipelineExecution, ExecutionPhase, PipelineMetrics, create_adaptive_pipeline, create_creative_adaptive_pipeline, create_memory_adaptive_pipeline};
// pub use llm_integration::{LLMIntegrationManager, LLMConfig, LLMProvider, ModelSelectionStrategy, LLMMetrics, LLMRequest, LLMResponse, IntentUnderstanding, Entity, CreativeSolution as LLMCreativeSolution, ProviderHealth, create_llm_integration_manager, create_custom_llm_integration_manager};
// pub use config_manager::{ConfigManager, ConfigManagerConfig, MasterConfiguration, ConfigSource, ValidationResult as ConfigValidationResult, create_config_manager, create_custom_config_manager};
// pub use advanced_learning::{AdvancedLearningEngine, AdvancedLearningConfig, LearningAlgorithm, LearningObjective, LearnedPattern as AdvancedLearnedPattern, LearningMetrics, OptimizationRecommendation, create_advanced_learning_engine, create_custom_learning_engine};
// pub use multi_model_orchestration::{MultiModelOrchestrator, OrchestrationConfig, OrchestrationStrategy, OrchestrationRequest, OrchestrationResponse, TaskSpecialty, create_multi_model_orchestrator, create_custom_orchestrator};
// pub use self_healing::{SelfHealingSystem, SelfHealingConfig, HealingStrategy, OptimizationArea, DetectedIssue, SelfHealingMetrics, create_self_healing_system, create_custom_self_healing_system};
// pub use advanced_analytics::{AdvancedAnalyticsEngine, AnalyticsConfig, AnalyticsInsight, AnalyticsReport, TimePeriod, InsightType, create_analytics_engine, create_custom_analytics_engine};