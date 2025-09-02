// RainbowBrowserAI - Organized module structure
#![warn(clippy::all)]
#![allow(unused_imports)] // Temporary during reorganization

// Core modules
pub mod api;           // HTTP API endpoints and handlers
pub mod browser;       // Browser automation and WebDriver integration
pub mod config;        // Configuration management
pub mod intelligence;  // AI and machine learning components
pub mod utils;         // Utility functions and common code
pub mod tools;         // Tool implementations and orchestration
pub mod plugins;       // Plugin system

// Specialized modules
pub mod perception_mvp;        // Multi-layer perception system
pub mod instruction_parser;    // Natural language command parsing
pub mod semantic_analyzer;     // Semantic analysis and understanding
pub mod action_mapper;         // Action mapping and execution
pub mod final_integration;     // System integration components

// Specialized implementations
pub mod mock_llm_provider;              // Mock LLM for testing
pub mod task_executor;                  // Task execution engine
pub mod simple_memory;                  // Simple memory management
pub mod continuous_improvement_pipeline; // Learning and improvement
pub mod deployment_pipeline;            // Deployment automation
pub mod adaptive_learning;              // Adaptive learning system
pub mod v8_perception;                  // V8 perception system
pub mod v8_command_translator;          // V8 command translation
pub mod enhanced_command_parser;        // Enhanced command parsing
pub mod enhanced_executor;              // Enhanced execution engine
pub mod execution_engine;               // Execution engine
pub mod extractor;                      // Data extraction
pub mod monitoring_alerts;              // Monitoring and alerts
pub mod self_healing_selectors;         // Self-healing element selection
pub mod smart_element_detector;         // Smart element detection
pub mod visual_detection_enhanced;       // Enhanced visual detection
pub mod tool_orchestrator;              // Tool orchestration

// Re-exports for backward compatibility and convenience
pub use api::{ApiState, create_router};
pub use browser::{SimpleBrowser, BrowserPool, PooledBrowserHandle};
pub use config::Config;
pub use intelligence::{LLMService, AIDecisionEngine};
pub use utils::{
    ConversationContext, HistoryEntry, ExecutionResult,
    Workflow, WorkflowResult, WorkflowEngine, WorkflowStep,
    MetricsCollector, Metrics, MetricsSummary,
    SecurityMiddleware, RateLimiter, InputValidator,
    CostTracker
};
pub use perception_mvp::{
    PerceptionOrchestrator, UnifiedPerceptionResult, PerceptionLevel,
    ExecutionInfo, PerformanceMetrics, Recommendation, RecommendationType,
    RecommendationPriority, ImplementationEffort
};
pub use mock_llm_provider::{MockLLMProvider, create_mock_provider, create_custom_mock_provider};

// Plugin system re-exports (simplified for compilation)
pub use plugins::{PluginManager, Plugin, PluginEvent, PluginMetrics};

// Tool system re-exports (simplified for compilation)
pub use tools::{
    ToolResult, ToolError,
    NavigateToUrl, TypeText, ScrollPage,
    ExtractText, ExtractData,
    WaitForElement
};

// Type aliases for common use cases
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
pub type ApiResult<T> = std::result::Result<T, api::ApiError>;

// Common data structures
pub use serde::{Deserialize, Serialize};
pub use serde_json::Value as JsonValue;

// Re-export ParsedCommand from enhanced_command_parser
pub use enhanced_command_parser::ParsedCommand;

// Screenshot options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenshotOptions {
    pub full_page: bool,
    pub element_selector: Option<String>,
    pub format: String, // "png" or "jpeg"
    pub quality: Option<u8>,
}

impl Default for ScreenshotOptions {
    fn default() -> Self {
        Self {
            full_page: false,
            element_selector: None,
            format: "png".to_string(),
            quality: None,
        }
    }
}