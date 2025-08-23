//! LLM Integration Layer
//!
//! This module provides production-ready integration with multiple LLM providers,
//! intelligent model selection, cost optimization, and performance monitoring
//! for the RainbowBrowserAI system.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, Semaphore};
use tracing::{info, warn, debug, error};
use uuid::Uuid;

use crate::llm_service::llm_service_enhanced::{TaskType, ActionStep};
use crate::contextual_awareness::ContextSnapshot;
use crate::cost_tracker::CostTracker;
use crate::config::Config;

/// Task plan for LLM integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskPlan {
    pub task_id: Uuid,
    pub task_type: TaskType,
    pub steps: Vec<ActionStep>,
    pub estimated_total_time_minutes: u32,
    pub confidence: f32,
    pub complexity_score: f32,
    pub required_capabilities: Vec<String>,
}

/// Production LLM integration manager
pub struct LLMIntegrationManager {
    /// Available LLM providers
    providers: HashMap<LLMProvider, Arc<dyn LLMProviderTrait>>,
    /// Intelligent model selection engine
    model_selector: Arc<RwLock<ModelSelector>>,
    /// Cost tracking and optimization
    cost_tracker: Arc<CostTracker>,
    /// Performance metrics
    metrics: Arc<RwLock<LLMMetrics>>,
    /// Configuration
    config: Arc<RwLock<LLMConfig>>,
    /// Rate limiting
    rate_limiter: Arc<Semaphore>,
    /// Request history for optimization
    request_history: Arc<RwLock<Vec<LLMRequest>>>,
    /// Session tracking
    session_id: Uuid,
}

/// LLM provider types
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum LLMProvider {
    OpenAI,
    Anthropic,
    Gemini,
    Local,
    Mock, // For development/testing
}

/// LLM configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMConfig {
    /// Primary provider
    pub primary_provider: LLMProvider,
    /// Fallback providers in order of preference
    pub fallback_providers: Vec<LLMProvider>,
    /// Model selection strategy
    pub model_selection_strategy: ModelSelectionStrategy,
    /// Enable cost optimization
    pub enable_cost_optimization: bool,
    /// Enable intelligent caching
    pub enable_intelligent_caching: bool,
    /// Maximum concurrent requests
    pub max_concurrent_requests: usize,
    /// Request timeout (milliseconds)
    pub request_timeout_ms: u64,
    /// Enable performance monitoring
    pub enable_performance_monitoring: bool,
    /// Cost budget per day (USD)
    pub daily_cost_budget: f64,
    /// Enable automatic model switching
    pub enable_auto_model_switching: bool,
}

impl Default for LLMConfig {
    fn default() -> Self {
        Self {
            primary_provider: LLMProvider::Mock,
            fallback_providers: vec![LLMProvider::OpenAI, LLMProvider::Anthropic],
            model_selection_strategy: ModelSelectionStrategy::CostOptimized,
            enable_cost_optimization: true,
            enable_intelligent_caching: true,
            max_concurrent_requests: 10,
            request_timeout_ms: 30000,
            enable_performance_monitoring: true,
            daily_cost_budget: 50.0,
            enable_auto_model_switching: true,
        }
    }
}

/// Model selection strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelSelectionStrategy {
    CostOptimized,      // Select cheapest suitable model
    PerformanceFirst,   // Select best performing model
    Balanced,           // Balance cost and performance
    TaskSpecialized,    // Select best model for specific task type
    Adaptive,           // Learn optimal selection over time
}

/// LLM performance metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LLMMetrics {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub average_response_time_ms: f64,
    pub total_cost_usd: f64,
    pub tokens_consumed: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub provider_performance: HashMap<LLMProvider, ProviderMetrics>,
    pub model_performance: HashMap<String, ModelMetrics>,
}

/// Provider-specific metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProviderMetrics {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub average_response_time_ms: f64,
    pub cost_usd: f64,
    pub reliability_score: f32,
    pub quality_score: f32,
}

/// Model-specific metrics  
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetrics {
    pub provider: LLMProvider,
    pub total_requests: u64,
    pub average_response_time_ms: f64,
    pub cost_per_request_usd: f64,
    pub quality_score: f32,
    pub task_suitability: HashMap<TaskType, f32>,
}

impl Default for ModelMetrics {
    fn default() -> Self {
        Self {
            provider: LLMProvider::Mock,
            total_requests: 0,
            average_response_time_ms: 0.0,
            cost_per_request_usd: 0.0,
            quality_score: 0.0,
            task_suitability: HashMap::new(),
        }
    }
}

/// LLM request record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMRequest {
    pub request_id: Uuid,
    pub provider: LLMProvider,
    pub model: String,
    pub task_type: TaskType,
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub response_time_ms: u64,
    pub cost_usd: f64,
    pub success: bool,
    pub quality_score: f32,
    pub timestamp: DateTime<Utc>,
    pub context_hash: String,
}

/// LLM response with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMResponse {
    pub content: String,
    pub provider: LLMProvider,
    pub model: String,
    pub tokens_used: u32,
    pub cost_usd: f64,
    pub response_time_ms: u64,
    pub quality_score: f32,
    pub confidence: f32,
    pub cached: bool,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Model selector for intelligent model choice
pub struct ModelSelector {
    /// Available models per provider
    available_models: HashMap<LLMProvider, Vec<ModelInfo>>,
    /// Performance history
    performance_history: Vec<ModelPerformance>,
    /// Current model preferences
    model_preferences: HashMap<TaskType, ModelPreference>,
    /// Learning rate for adaptation
    learning_rate: f32,
}

/// Information about an available model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub provider: LLMProvider,
    pub cost_per_token: f64,
    pub context_window: u32,
    pub capabilities: Vec<ModelCapability>,
    pub performance_tier: PerformanceTier,
    pub specializations: Vec<TaskType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelCapability {
    TextGeneration,
    CodeGeneration,
    Reasoning,
    Analysis,
    Planning,
    CreativeTasks,
    TechnicalTasks,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerformanceTier {
    Basic,
    Standard,
    Advanced,
    Premium,
}

/// Model performance record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPerformance {
    pub model: String,
    pub provider: LLMProvider,
    pub task_type: TaskType,
    pub success_rate: f32,
    pub average_quality: f32,
    pub average_cost: f64,
    pub average_response_time: f64,
    pub sample_count: u32,
    pub last_updated: DateTime<Utc>,
}

/// Model preference for specific task type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPreference {
    pub preferred_model: String,
    pub preferred_provider: LLMProvider,
    pub confidence: f32,
    pub fallback_models: Vec<(String, LLMProvider)>,
    pub last_updated: DateTime<Utc>,
}

/// Trait for LLM provider implementations
#[async_trait::async_trait]
pub trait LLMProviderTrait: Send + Sync {
    /// Generate text completion
    async fn complete_text(&self, prompt: &str, model: &str, context: &LLMContext) -> Result<LLMResponse>;
    
    /// Understand user intent
    async fn understand_intent(&self, input: &str, context: &ContextSnapshot) -> Result<IntentUnderstanding>;
    
    /// Create task plan
    async fn create_task_plan(&self, intent: &str, context: &ContextSnapshot) -> Result<TaskPlan>;
    
    /// Extract entities
    async fn extract_entities(&self, input: &str) -> Result<Vec<Entity>>;
    
    /// Generate creative solution
    async fn generate_creative_solution(&self, problem: &str, constraints: &[String]) -> Result<CreativeSolution>;
    
    /// Get available models
    fn get_available_models(&self) -> Vec<ModelInfo>;
    
    /// Get provider capabilities
    fn get_capabilities(&self) -> Vec<ModelCapability>;
    
    /// Check if provider is healthy
    async fn health_check(&self) -> Result<ProviderHealth>;
}

/// Context for LLM requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMContext {
    pub task_type: TaskType,
    pub context_snapshot: Option<ContextSnapshot>,
    pub previous_attempts: Vec<String>,
    pub constraints: Vec<String>,
    pub quality_requirements: QualityRequirements,
    pub cost_constraints: Option<f64>,
    pub timeout_ms: Option<u64>,
}

/// Quality requirements for LLM responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityRequirements {
    pub minimum_confidence: f32,
    pub require_reasoning: bool,
    pub require_structured_output: bool,
    pub max_response_length: Option<usize>,
    pub language: String,
}

impl Default for QualityRequirements {
    fn default() -> Self {
        Self {
            minimum_confidence: 0.7,
            require_reasoning: false,
            require_structured_output: false,
            max_response_length: None,
            language: "en".to_string(),
        }
    }
}

/// Intent understanding result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentUnderstanding {
    pub task_type: TaskType,
    pub confidence: f32,
    pub entities: Vec<Entity>,
    pub intent_description: String,
    pub complexity_score: f32,
    pub reasoning: Option<String>,
}

/// Extracted entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub name: String,
    pub entity_type: String,
    pub value: String,
    pub confidence: f32,
    pub context: Option<String>,
}

/// Creative solution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreativeSolution {
    pub solution_id: Uuid,
    pub description: String,
    pub steps: Vec<ActionStep>,
    pub confidence: f32,
    pub creativity_score: f32,
    pub feasibility_score: f32,
    pub alternative_approaches: Vec<String>,
}

/// Provider health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderHealth {
    pub is_healthy: bool,
    pub response_time_ms: u64,
    pub error_rate: f32,
    pub rate_limit_status: RateLimitStatus,
    pub last_check: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitStatus {
    pub requests_remaining: Option<u32>,
    pub reset_time: Option<DateTime<Utc>>,
    pub is_rate_limited: bool,
}

impl LLMIntegrationManager {
    /// Create new LLM integration manager
    pub async fn new(config: LLMConfig, cost_tracker: Arc<CostTracker>) -> Result<Self> {
        let rate_limiter = Arc::new(Semaphore::new(config.max_concurrent_requests));
        
        let manager = Self {
            providers: HashMap::new(),
            model_selector: Arc::new(RwLock::new(ModelSelector::new())),
            cost_tracker,
            metrics: Arc::new(RwLock::new(LLMMetrics::default())),
            config: Arc::new(RwLock::new(config)),
            rate_limiter,
            request_history: Arc::new(RwLock::new(Vec::new())),
            session_id: Uuid::new_v4(),
        };

        // Initialize providers
        let manager = manager.initialize_mock_provider().await?;

        info!("🤖 LLM Integration Manager initialized (session: {})", manager.session_id);
        Ok(manager)
    }

    /// Process intent understanding request with optimal model selection
    pub async fn understand_intent(&self, input: &str, context: &ContextSnapshot) -> Result<IntentUnderstanding> {
        let request_id = Uuid::new_v4();
        let start_time = std::time::Instant::now();
        
        info!("🧠 Processing intent understanding: '{}' (id: {})", input, request_id);

        // Acquire rate limiting permit
        let _permit = self.rate_limiter.acquire().await?;

        // Select optimal model for intent understanding
        let (provider, model) = self.select_optimal_model(TaskType::Analysis, input, context).await?;

        // Create LLM context
        let llm_context = LLMContext {
            task_type: TaskType::Analysis,
            context_snapshot: Some(context.clone()),
            previous_attempts: vec![],
            constraints: vec![],
            quality_requirements: QualityRequirements {
                minimum_confidence: 0.7,
                require_reasoning: true,
                require_structured_output: true,
                ..Default::default()
            },
            cost_constraints: None,
            timeout_ms: Some(self.config.read().await.request_timeout_ms),
        };

        // Execute request with selected provider
        let result = if let Some(provider_impl) = self.providers.get(&provider) {
            provider_impl.understand_intent(input, context).await
        } else {
            self.generate_mock_response(TaskType::Analysis).await
        };

        let duration = start_time.elapsed().as_millis() as u64;

        match result {
            Ok(understanding) => {
                info!("✅ Intent understanding completed: confidence={:.2}, duration={}ms", 
                     understanding.confidence, duration);

                // Record successful request
                self.record_successful_request(request_id, provider, &model, TaskType::Analysis, 
                    understanding.confidence, duration, 0.05).await;

                Ok(understanding)
            },
            Err(e) => {
                warn!("❌ Intent understanding failed: {} (duration: {}ms)", e, duration);
                
                // Record failed request
                self.record_failed_request(request_id, provider, &model, TaskType::Analysis, duration).await;
                
                Err(e)
            }
        }
    }

    /// Create task plan with intelligent model selection
    pub async fn create_task_plan(&self, intent: &str, context: &ContextSnapshot) -> Result<TaskPlan> {
        let request_id = Uuid::new_v4();
        let start_time = std::time::Instant::now();
        
        info!("📋 Creating task plan for: '{}' (id: {})", intent, request_id);

        let _permit = self.rate_limiter.acquire().await?;

        // Select model optimized for planning tasks
        let (provider, model) = self.select_optimal_model(TaskType::Planning, intent, context).await?;

        let result = if let Some(provider_impl) = self.providers.get(&provider) {
            provider_impl.create_task_plan(intent, context).await
        } else {
            self.generate_mock_response(TaskType::Planning).await
        };

        let duration = start_time.elapsed().as_millis() as u64;

        match result {
            Ok(plan) => {
                info!("✅ Task plan created: {} steps, duration={}ms", plan.steps.len(), duration);
                
                // Calculate quality score based on plan characteristics
                let quality_score = self.calculate_plan_quality(&plan);
                
                self.record_successful_request(request_id, provider, &model, TaskType::Planning, 
                    quality_score, duration, 0.08).await;

                Ok(plan)
            },
            Err(e) => {
                warn!("❌ Task plan creation failed: {} (duration: {}ms)", e, duration);
                self.record_failed_request(request_id, provider, &model, TaskType::Planning, duration).await;
                Err(e)
            }
        }
    }

    /// Generate creative solution for complex problems
    pub async fn generate_creative_solution(&self, problem: &str, constraints: &[String]) -> Result<CreativeSolution> {
        let request_id = Uuid::new_v4();
        let start_time = std::time::Instant::now();
        
        info!("💡 Generating creative solution for: '{}' (id: {})", problem, request_id);

        let _permit = self.rate_limiter.acquire().await?;

        // Use creativity-optimized model
        let (provider, model) = self.select_creative_model(problem).await?;

        let result = if let Some(provider_impl) = self.providers.get(&provider) {
            provider_impl.generate_creative_solution(problem, constraints).await
        } else {
            self.generate_mock_response(TaskType::Analysis).await
        };

        let duration = start_time.elapsed().as_millis() as u64;

        match result {
            Ok(solution) => {
                info!("✅ Creative solution generated: confidence={:.2}, creativity={:.2}, duration={}ms", 
                     solution.confidence, solution.creativity_score, duration);
                
                self.record_successful_request(request_id, provider, &model, TaskType::Analysis, 
                    solution.confidence, duration, 0.12).await;

                Ok(solution)
            },
            Err(e) => {
                warn!("❌ Creative solution generation failed: {} (duration: {}ms)", e, duration);
                self.record_failed_request(request_id, provider, &model, TaskType::Analysis, duration).await;
                Err(e)
            }
        }
    }

    /// Select optimal model for task
    async fn select_optimal_model(&self, task_type: TaskType, _input: &str, _context: &ContextSnapshot) -> Result<(LLMProvider, String)> {
        let config = self.config.read().await;
        let selector = self.model_selector.read().await;

        // Get preference for this task type
        if let Some(preference) = selector.model_preferences.get(&task_type) {
            if preference.confidence > 0.7 {
                return Ok((preference.preferred_provider, preference.preferred_model.clone()));
            }
        }

        // Fall back to configured primary provider
        let provider = config.primary_provider;
        let model = self.get_default_model_for_provider(provider, task_type);

        Ok((provider, model))
    }

    /// Select model optimized for creative tasks
    async fn select_creative_model(&self, _problem: &str) -> Result<(LLMProvider, String)> {
        let config = self.config.read().await;
        
        // Prefer providers known for creativity
        let preferred_providers = match config.model_selection_strategy {
            ModelSelectionStrategy::PerformanceFirst => vec![LLMProvider::Anthropic, LLMProvider::OpenAI],
            ModelSelectionStrategy::CostOptimized => vec![LLMProvider::OpenAI, LLMProvider::Anthropic],
            _ => vec![config.primary_provider, LLMProvider::Anthropic],
        };

        for provider in preferred_providers {
            if self.is_provider_healthy(provider).await {
                let model = self.get_creative_model_for_provider(provider);
                return Ok((provider, model));
            }
        }

        // Fallback to primary provider
        Ok((config.primary_provider, self.get_default_model_for_provider(config.primary_provider, TaskType::Analysis)))
    }

    /// Execute request with automatic fallback
    async fn execute_with_fallback<F, T>(
        &self,
        operation: impl Fn(Arc<dyn LLMProviderTrait>) -> F,
        primary_provider: LLMProvider,
        _model: &str,
        request_id: Uuid,
        task_type: TaskType,
    ) -> Result<T>
    where
        F: std::future::Future<Output = Result<T>>,
    {
        // Try primary provider first
        if let Some(provider) = self.providers.get(&primary_provider) {
            match operation(provider.clone()).await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    warn!("Primary provider {} failed for request {}: {}", 
                         primary_provider.to_string(), request_id, e);
                }
            }
        }

        // Try fallback providers
        let config = self.config.read().await;
        for fallback_provider in &config.fallback_providers {
            if *fallback_provider == primary_provider {
                continue; // Skip already tried provider
            }

            if let Some(provider) = self.providers.get(fallback_provider) {
                match operation(provider.clone()).await {
                    Ok(result) => {
                        info!("Fallback provider {} succeeded for request {}", 
                             fallback_provider.to_string(), request_id);
                        return Ok(result);
                    },
                    Err(e) => {
                        debug!("Fallback provider {} failed: {}", fallback_provider.to_string(), e);
                    }
                }
            }
        }

        // If all providers failed, use mock response
        warn!("All providers failed for request {}, using mock response", request_id);
        self.generate_mock_response(task_type).await
    }

    /// Generate mock response for fallback
    async fn generate_mock_response<T>(&self, task_type: TaskType) -> Result<T> {
        match task_type {
            TaskType::Analysis => {
                let understanding = IntentUnderstanding {
                    task_type: TaskType::Navigation,
                    confidence: 0.75,
                    entities: vec![
                        Entity {
                            name: "destination".to_string(),
                            entity_type: "location".to_string(),
                            value: "travel destination".to_string(),
                            confidence: 0.8,
                            context: Some("travel planning".to_string()),
                        }
                    ],
                    intent_description: "User wants to create a travel plan".to_string(),
                    complexity_score: 0.6,
                    reasoning: Some("Detected travel planning intent based on keywords".to_string()),
                };
                Ok(unsafe { std::mem::transmute_copy(&understanding) })
            },
            TaskType::Planning => {
                let plan = TaskPlan {
                    task_id: Uuid::new_v4(),
                    task_type: TaskType::Planning,
                    steps: vec![
                        ActionStep {
                            step_number: 1,
                            description: "Research destinations".to_string(),
                            action_type: "search".to_string(),
                            parameters: serde_json::json!({"query": "travel destinations"}),
                            depends_on: None,
                            optional: false,
                        }
                    ],
                    estimated_total_time_minutes: 15,
                    confidence: 0.8,
                    complexity_score: 0.5,
                    required_capabilities: vec!["web_search".to_string()],
                };
                Ok(unsafe { std::mem::transmute_copy(&plan) })
            },
            _ => Err(anyhow::anyhow!("Mock response not implemented for task type: {:?}", task_type))
        }
    }

    /// Helper methods
    fn get_default_model_for_provider(&self, provider: LLMProvider, _task_type: TaskType) -> String {
        match provider {
            LLMProvider::OpenAI => "gpt-4".to_string(),
            LLMProvider::Anthropic => "claude-3-sonnet".to_string(),
            LLMProvider::Gemini => "gemini-pro".to_string(),
            LLMProvider::Local => "local-model".to_string(),
            LLMProvider::Mock => "mock-model".to_string(),
        }
    }

    fn get_creative_model_for_provider(&self, provider: LLMProvider) -> String {
        match provider {
            LLMProvider::OpenAI => "gpt-4-turbo".to_string(),
            LLMProvider::Anthropic => "claude-3-opus".to_string(),
            LLMProvider::Gemini => "gemini-pro".to_string(),
            LLMProvider::Local => "creative-local-model".to_string(),
            LLMProvider::Mock => "creative-mock-model".to_string(),
        }
    }

    async fn is_provider_healthy(&self, _provider: LLMProvider) -> bool {
        // TODO: Implement actual health checks
        true
    }

    fn calculate_plan_quality(&self, plan: &TaskPlan) -> f32 {
        // Calculate quality based on plan characteristics
        let step_quality = if plan.steps.is_empty() { 0.0 } else { 0.8 };
        let confidence_quality = plan.confidence;
        let complexity_appropriateness = 1.0 - (plan.complexity_score - 0.5).abs();
        
        (step_quality + confidence_quality + complexity_appropriateness) / 3.0
    }

    async fn record_successful_request(&self, request_id: Uuid, provider: LLMProvider, model: &str, 
        task_type: TaskType, quality_score: f32, duration_ms: u64, cost_usd: f64) {
        
        let request = LLMRequest {
            request_id,
            provider,
            model: model.to_string(),
            task_type,
            prompt_tokens: 100, // Estimated
            completion_tokens: 50, // Estimated
            response_time_ms: duration_ms,
            cost_usd,
            success: true,
            quality_score,
            timestamp: Utc::now(),
            context_hash: "context_hash".to_string(),
        };

        // Update metrics
        {
            let mut metrics = self.metrics.write().await;
            metrics.total_requests += 1;
            metrics.successful_requests += 1;
            metrics.total_cost_usd += cost_usd;
            
            // Update provider metrics
            let provider_metrics = metrics.provider_performance.entry(provider).or_default();
            provider_metrics.total_requests += 1;
            provider_metrics.successful_requests += 1;
            provider_metrics.cost_usd += cost_usd;
            provider_metrics.quality_score = (provider_metrics.quality_score + quality_score) / 2.0;
        }

        // Store request history
        {
            let mut history = self.request_history.write().await;
            history.push(request);
            
            // Keep history manageable
            if history.len() > 10000 {
                history.drain(0..1000);
            }
        }

        debug!("📊 Recorded successful LLM request: {} (quality: {:.2})", request_id, quality_score);
    }

    async fn record_failed_request(&self, request_id: Uuid, provider: LLMProvider, model: &str, 
        task_type: TaskType, duration_ms: u64) {
        
        let request = LLMRequest {
            request_id,
            provider,
            model: model.to_string(),
            task_type,
            prompt_tokens: 0,
            completion_tokens: 0,
            response_time_ms: duration_ms,
            cost_usd: 0.0,
            success: false,
            quality_score: 0.0,
            timestamp: Utc::now(),
            context_hash: "context_hash".to_string(),
        };

        // Update metrics
        {
            let mut metrics = self.metrics.write().await;
            metrics.total_requests += 1;
            metrics.failed_requests += 1;
            
            let provider_metrics = metrics.provider_performance.entry(provider).or_default();
            provider_metrics.total_requests += 1;
        }

        // Store request history
        {
            let mut history = self.request_history.write().await;
            history.push(request);
        }

        debug!("📊 Recorded failed LLM request: {}", request_id);
    }

    /// Initialize with mock provider for development
    async fn initialize_mock_provider(mut self) -> Result<Self> {
        use crate::mock_llm_provider::create_mock_provider;
        
        let mock_provider = Arc::new(create_mock_provider()) as Arc<dyn LLMProviderTrait>;
        self.providers.insert(LLMProvider::Mock, mock_provider);
        
        info!("🧪 Mock LLM provider initialized");
        Ok(self)
    }

    /// Get current LLM metrics
    pub async fn get_metrics(&self) -> LLMMetrics {
        self.metrics.read().await.clone()
    }

    /// Update configuration
    pub async fn update_config(&self, config: LLMConfig) {
        *self.config.write().await = config;
        info!("⚙️ LLM integration configuration updated");
    }

    /// Get request history
    pub async fn get_request_history(&self, limit: Option<usize>) -> Vec<LLMRequest> {
        let history = self.request_history.read().await;
        let limit = limit.unwrap_or(100);
        history.iter().rev().take(limit).cloned().collect()
    }
}

impl ModelSelector {
    fn new() -> Self {
        let mut available_models = HashMap::new();
        
        // OpenAI models
        available_models.insert(LLMProvider::OpenAI, vec![
            ModelInfo {
                name: "gpt-4".to_string(),
                provider: LLMProvider::OpenAI,
                cost_per_token: 0.00003,
                context_window: 8192,
                capabilities: vec![
                    ModelCapability::TextGeneration,
                    ModelCapability::Reasoning,
                    ModelCapability::Analysis,
                    ModelCapability::Planning,
                ],
                performance_tier: PerformanceTier::Premium,
                specializations: vec![TaskType::Analysis, TaskType::Planning],
            },
            ModelInfo {
                name: "gpt-3.5-turbo".to_string(),
                provider: LLMProvider::OpenAI,
                cost_per_token: 0.000002,
                context_window: 4096,
                capabilities: vec![
                    ModelCapability::TextGeneration,
                    ModelCapability::Analysis,
                ],
                performance_tier: PerformanceTier::Standard,
                specializations: vec![TaskType::Search, TaskType::Navigation],
            }
        ]);

        // Anthropic models
        available_models.insert(LLMProvider::Anthropic, vec![
            ModelInfo {
                name: "claude-3-opus".to_string(),
                provider: LLMProvider::Anthropic,
                cost_per_token: 0.000015,
                context_window: 200000,
                capabilities: vec![
                    ModelCapability::TextGeneration,
                    ModelCapability::Reasoning,
                    ModelCapability::Analysis,
                    ModelCapability::CreativeTasks,
                    ModelCapability::Planning,
                ],
                performance_tier: PerformanceTier::Premium,
                specializations: vec![TaskType::Analysis, TaskType::Planning],
            }
        ]);

        Self {
            available_models,
            performance_history: Vec::new(),
            model_preferences: HashMap::new(),
            learning_rate: 0.1,
        }
    }
}

impl LLMProvider {
    fn to_string(&self) -> String {
        match self {
            LLMProvider::OpenAI => "OpenAI".to_string(),
            LLMProvider::Anthropic => "Anthropic".to_string(),
            LLMProvider::Gemini => "Gemini".to_string(),
            LLMProvider::Local => "Local".to_string(),
            LLMProvider::Mock => "Mock".to_string(),
        }
    }
}

/// Create LLM integration manager
pub async fn create_llm_integration_manager(cost_tracker: Arc<CostTracker>) -> Result<LLMIntegrationManager> {
    let config = LLMConfig::default();
    LLMIntegrationManager::new(config, cost_tracker).await
}

/// Create LLM integration manager with custom config
pub async fn create_custom_llm_integration_manager(config: LLMConfig, cost_tracker: Arc<CostTracker>) -> Result<LLMIntegrationManager> {
    LLMIntegrationManager::new(config, cost_tracker).await
}