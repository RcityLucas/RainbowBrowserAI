// LLM Integration Module
// Provides intelligent task planning and AI-driven automation

pub mod client;
pub mod task_planner;
pub mod cost_tracker;
pub mod prompt_engine;
pub mod providers;

pub use client::{LLMClient, LLMResponse, LLMError, TokenUsage};
pub use task_planner::{TaskPlan, TaskStep, TaskPlanExecutor};
pub use cost_tracker::{CostTracker, UsageMetrics};
pub use prompt_engine::{PromptEngine, PromptTemplate, ContextAwarePrompt};
pub use providers::{OpenAIProvider, ClaudeProvider, LLMProvider};

use anyhow::Result;
use serde::Deserialize;
use std::collections::HashMap;

/// Configuration for LLM services
#[derive(Debug, Clone, Deserialize)]
pub struct LLMConfig {
    pub default_provider: String,
    pub openai_api_key: Option<String>,
    pub claude_api_key: Option<String>,
    pub max_tokens: u32,
    pub temperature: f32,
    pub cost_limit_usd: f32,
}

impl Default for LLMConfig {
    fn default() -> Self {
        Self {
            default_provider: "openai".to_string(),
            openai_api_key: None,
            claude_api_key: None,
            max_tokens: 4000,
            temperature: 0.7,
            cost_limit_usd: 1.0,
        }
    }
}

/// Main LLM service manager
pub struct LLMService {
    config: LLMConfig,
    providers: HashMap<String, Box<dyn LLMProvider>>,
    cost_tracker: CostTracker,
    prompt_engine: PromptEngine,
}

impl LLMService {
    pub fn new(config: LLMConfig) -> Result<Self> {
        let mut providers: HashMap<String, Box<dyn LLMProvider>> = HashMap::new();
        
        // Initialize providers based on available API keys
        if config.openai_api_key.is_some() {
            providers.insert("openai".to_string(), Box::new(OpenAIProvider::new(&config)?));
        }
        if config.claude_api_key.is_some() {
            providers.insert("claude".to_string(), Box::new(ClaudeProvider::new(&config)?));
        }

        Ok(Self {
            config,
            providers,
            cost_tracker: CostTracker::new(),
            prompt_engine: PromptEngine::new(),
        })
    }

    pub async fn plan_task(&mut self, user_instruction: &str, context: &HashMap<String, serde_json::Value>) -> Result<TaskPlan> {
        let prompt = self.prompt_engine.create_task_planning_prompt(user_instruction, context)?;
        let response = self.query(&prompt.final_prompt).await?;
        TaskPlan::from_llm_response(&response).map_err(|e| anyhow::anyhow!("Task planning failed: {}", e))
    }

    pub async fn query(&mut self, prompt: &str) -> Result<LLMResponse> {
        let provider_name = &self.config.default_provider;
        if let Some(provider) = self.providers.get_mut(provider_name) {
            let response = provider.query(prompt, &self.config).await?;
            self.cost_tracker.track_usage(&response);
            Ok(response)
        } else {
            Err(anyhow::anyhow!("No LLM provider available: {}", provider_name))
        }
    }

    pub fn get_cost_metrics(&self) -> &UsageMetrics {
        self.cost_tracker.get_metrics()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_llm_service_creation() {
        let config = LLMConfig::default();
        let service = LLMService::new(config);
        assert!(service.is_ok());
    }
}