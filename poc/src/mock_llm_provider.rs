//! Mock LLM Provider for Development and Testing
//!
//! This module provides a mock LLM provider that implements the LLMProviderTrait
//! for development, testing, and demonstration purposes without requiring actual
//! LLM API connections.

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::llm_integration::{
    LLMProviderTrait, LLMResponse, LLMContext, IntentUnderstanding, TaskPlan, Entity, 
    CreativeSolution, ProviderHealth, ModelInfo, ModelCapability, PerformanceTier,
    RateLimitStatus, LLMProvider
};
use crate::llm_service::llm_service_enhanced::{TaskType, ActionStep};
use crate::contextual_awareness::ContextSnapshot;

/// Mock LLM provider for testing and development
pub struct MockLLMProvider {
    provider_name: String,
    simulated_latency_ms: u64,
    success_rate: f32,
}

impl MockLLMProvider {
    /// Create new mock LLM provider
    pub fn new(provider_name: String, simulated_latency_ms: u64, success_rate: f32) -> Self {
        Self {
            provider_name,
            simulated_latency_ms,
            success_rate,
        }
    }

    /// Create default mock provider
    pub fn default() -> Self {
        Self::new("mock-provider".to_string(), 200, 0.95)
    }

    /// Simulate network latency
    async fn simulate_latency(&self) {
        tokio::time::sleep(tokio::time::Duration::from_millis(self.simulated_latency_ms)).await;
    }

    /// Check if request should succeed based on success rate
    fn should_succeed(&self) -> bool {
        // Simple deterministic success check instead of random for now
        true // Always succeed in mock mode
    }

    /// Generate mock task plan for travel requests
    fn generate_travel_task_plan(&self) -> TaskPlan {
        TaskPlan {
            task_id: Uuid::new_v4(),
            task_type: TaskType::Planning,
            steps: vec![
                ActionStep {
                    step_number: 1,
                    description: "Research destinations and attractions".to_string(),
                    action_type: "search".to_string(),
                    parameters: serde_json::json!({"query": "travel destinations", "sites": ["tripadvisor.com", "lonelyplanet.com"]}),
                    depends_on: None,
                    optional: false,
                },
                ActionStep {
                    step_number: 2,
                    description: "Search for flights".to_string(),
                    action_type: "navigation".to_string(),
                    parameters: serde_json::json!({"url": "https://www.google.com/flights", "screenshot": true}),
                    depends_on: Some(1),
                    optional: false,
                },
                ActionStep {
                    step_number: 3,
                    description: "Find accommodation options".to_string(),
                    action_type: "navigation".to_string(),
                    parameters: serde_json::json!({"url": "https://www.booking.com", "screenshot": true}),
                    depends_on: Some(1),
                    optional: false,
                },
                ActionStep {
                    step_number: 4,
                    description: "Check weather forecast".to_string(),
                    action_type: "navigation".to_string(),
                    parameters: serde_json::json!({"url": "https://www.weather.com", "screenshot": false}),
                    depends_on: None,
                    optional: true,
                },
                ActionStep {
                    step_number: 5,
                    description: "Compile travel plan summary".to_string(),
                    action_type: "report".to_string(),
                    parameters: serde_json::json!({"format": "markdown", "include_screenshots": true}),
                    depends_on: Some(4),
                    optional: false,
                }
            ],
            estimated_total_time_minutes: 25,
            confidence: 0.85,
            complexity_score: 0.6,
            required_capabilities: vec![
                "web_navigation".to_string(),
                "content_extraction".to_string(),
                "screenshot_capture".to_string(),
                "report_generation".to_string(),
            ],
        }
    }
}

#[async_trait]
impl LLMProviderTrait for MockLLMProvider {
    /// Generate text completion
    async fn complete_text(&self, prompt: &str, model: &str, _context: &LLMContext) -> Result<LLMResponse> {
        self.simulate_latency().await;

        if !self.should_succeed() {
            return Err(anyhow::anyhow!("Mock provider simulated failure"));
        }

        let content = if prompt.contains("travel") || prompt.contains("plan") {
            "Here's a comprehensive travel plan: Research destinations, book flights, find accommodation, check weather, and create an itinerary."
        } else if prompt.contains("analyze") {
            "Analysis complete: The content has been reviewed and key insights have been identified."
        } else {
            "Mock response: I understand your request and can help with browser automation tasks."
        };

        Ok(LLMResponse {
            content: content.to_string(),
            provider: LLMProvider::Mock,
            model: model.to_string(),
            tokens_used: 150,
            cost_usd: 0.01,
            response_time_ms: self.simulated_latency_ms,
            quality_score: 0.8,
            confidence: 0.85,
            cached: false,
            metadata: {
                let mut map = std::collections::HashMap::new();
                map.insert("provider".to_string(), serde_json::Value::String(self.provider_name.clone()));
                map.insert("simulated".to_string(), serde_json::Value::Bool(true));
                map.insert("prompt_length".to_string(), serde_json::Value::Number(prompt.len().into()));
                map
            },
        })
    }

    /// Understand user intent with enhanced mock responses
    async fn understand_intent(&self, input: &str, _context: &ContextSnapshot) -> Result<IntentUnderstanding> {
        self.simulate_latency().await;

        if !self.should_succeed() {
            return Err(anyhow::anyhow!("Mock provider simulated failure"));
        }

        let input_lower = input.to_lowercase();
        
        // Enhanced intent classification logic
        let (task_type, confidence, entities) = if input_lower.contains("travel") || input_lower.contains("trip") || input_lower.contains("plan") {
            (
                TaskType::Planning,
                0.9,
                vec![
                    Entity {
                        name: "activity".to_string(),
                        entity_type: "travel_activity".to_string(),
                        value: "travel_planning".to_string(),
                        confidence: 0.95,
                        context: Some("User wants to create a travel plan".to_string()),
                    }
                ]
            )
        } else if input_lower.contains("search") || input_lower.contains("find") {
            (
                TaskType::Search,
                0.85,
                vec![
                    Entity {
                        name: "action".to_string(),
                        entity_type: "search_action".to_string(),
                        value: "web_search".to_string(),
                        confidence: 0.8,
                        context: Some("User wants to search for information".to_string()),
                    }
                ]
            )
        } else if input_lower.contains("analyze") || input_lower.contains("review") {
            (
                TaskType::Analysis,
                0.8,
                vec![
                    Entity {
                        name: "action".to_string(),
                        entity_type: "analysis_action".to_string(),
                        value: "content_analysis".to_string(),
                        confidence: 0.75,
                        context: Some("User wants to analyze content".to_string()),
                    }
                ]
            )
        } else if input_lower.contains("navigate") || input_lower.contains("go to") {
            (
                TaskType::Navigation,
                0.75,
                vec![
                    Entity {
                        name: "action".to_string(),
                        entity_type: "navigation_action".to_string(),
                        value: "web_navigation".to_string(),
                        confidence: 0.8,
                        context: Some("User wants to navigate to a website".to_string()),
                    }
                ]
            )
        } else {
            (TaskType::Unknown, 0.5, vec![])
        };

        Ok(IntentUnderstanding {
            task_type,
            confidence,
            entities,
            intent_description: format!("Mock understanding of: '{}'", input),
            complexity_score: if task_type == TaskType::Planning { 0.8 } else { 0.4 },
            reasoning: Some(format!("Mock reasoning: Classified as {:?} based on keyword analysis", task_type)),
        })
    }

    /// Create detailed task plan
    async fn create_task_plan(&self, intent: &str, _context: &ContextSnapshot) -> Result<TaskPlan> {
        self.simulate_latency().await;

        if !self.should_succeed() {
            return Err(anyhow::anyhow!("Mock provider simulated failure"));
        }

        let intent_lower = intent.to_lowercase();

        if intent_lower.contains("travel") || intent_lower.contains("plan") || intent_lower.contains("trip") {
            Ok(self.generate_travel_task_plan())
        } else {
            // Generic task plan
            Ok(TaskPlan {
                task_id: Uuid::new_v4(),
                task_type: TaskType::Execution,
                steps: vec![
                    ActionStep {
                        step_number: 1,
                        description: format!("Execute task for: {}", intent),
                        action_type: "generic".to_string(),
                        parameters: serde_json::json!({"intent": intent}),
                        depends_on: None,
                        optional: false,
                    }
                ],
                estimated_total_time_minutes: 5,
                confidence: 0.7,
                complexity_score: 0.3,
                required_capabilities: vec!["basic_automation".to_string()],
            })
        }
    }

    /// Extract entities from input
    async fn extract_entities(&self, input: &str) -> Result<Vec<Entity>> {
        self.simulate_latency().await;

        if !self.should_succeed() {
            return Err(anyhow::anyhow!("Mock provider simulated failure"));
        }

        let mut entities = Vec::new();
        let input_lower = input.to_lowercase();

        // Extract common entity types
        if input_lower.contains("paris") {
            entities.push(Entity {
                name: "destination".to_string(),
                entity_type: "location".to_string(),
                value: "Paris, France".to_string(),
                confidence: 0.95,
                context: Some("Travel destination".to_string()),
            });
        }

        if input_lower.contains("google") {
            entities.push(Entity {
                name: "website".to_string(),
                entity_type: "url".to_string(),
                value: "google.com".to_string(),
                confidence: 0.9,
                context: Some("Website reference".to_string()),
            });
        }

        if input_lower.contains("tomorrow") {
            entities.push(Entity {
                name: "date".to_string(),
                entity_type: "temporal".to_string(),
                value: "tomorrow".to_string(),
                confidence: 0.85,
                context: Some("Date reference".to_string()),
            });
        }

        Ok(entities)
    }

    /// Generate creative solution
    async fn generate_creative_solution(&self, problem: &str, constraints: &[String]) -> Result<CreativeSolution> {
        self.simulate_latency().await;

        if !self.should_succeed() {
            return Err(anyhow::anyhow!("Mock provider simulated failure"));
        }

        Ok(CreativeSolution {
            solution_id: Uuid::new_v4(),
            description: format!("Creative solution for: {}", problem),
            steps: vec![
                ActionStep {
                    step_number: 1,
                    description: "Analyze the problem context".to_string(),
                    action_type: "analysis".to_string(),
                    parameters: serde_json::json!({"problem": problem, "constraints": constraints}),
                    depends_on: None,
                    optional: false,
                },
                ActionStep {
                    step_number: 2,
                    description: "Generate creative alternatives".to_string(),
                    action_type: "generate".to_string(),
                    parameters: serde_json::json!({"approach": "creative"}),
                    depends_on: Some(1),
                    optional: false,
                }
            ],
            confidence: 0.75,
            creativity_score: 0.8,
            feasibility_score: 0.7,
            alternative_approaches: vec![
                "Alternative approach 1: Use different tools".to_string(),
                "Alternative approach 2: Modify constraints".to_string(),
            ],
        })
    }

    /// Get available models for this mock provider
    fn get_available_models(&self) -> Vec<ModelInfo> {
        vec![
            ModelInfo {
                name: "mock-gpt-4".to_string(),
                provider: LLMProvider::Mock,
                cost_per_token: 0.0, // Mock provider is free
                context_window: 8192,
                capabilities: vec![
                    ModelCapability::TextGeneration,
                    ModelCapability::Reasoning,
                    ModelCapability::Analysis,
                    ModelCapability::Planning,
                ],
                performance_tier: PerformanceTier::Advanced,
                specializations: vec![TaskType::Planning, TaskType::Analysis],
            },
            ModelInfo {
                name: "mock-creative".to_string(),
                provider: LLMProvider::Mock,
                cost_per_token: 0.0,
                context_window: 4096,
                capabilities: vec![
                    ModelCapability::TextGeneration,
                    ModelCapability::CreativeTasks,
                ],
                performance_tier: PerformanceTier::Standard,
                specializations: vec![TaskType::Analysis],
            }
        ]
    }

    /// Get provider capabilities
    fn get_capabilities(&self) -> Vec<ModelCapability> {
        vec![
            ModelCapability::TextGeneration,
            ModelCapability::Reasoning,
            ModelCapability::Analysis,
            ModelCapability::Planning,
            ModelCapability::CreativeTasks,
        ]
    }

    /// Check provider health
    async fn health_check(&self) -> Result<ProviderHealth> {
        self.simulate_latency().await;

        Ok(ProviderHealth {
            is_healthy: true,
            response_time_ms: self.simulated_latency_ms,
            error_rate: 1.0 - self.success_rate,
            rate_limit_status: RateLimitStatus {
                requests_remaining: Some(1000),
                reset_time: None,
                is_rate_limited: false,
            },
            last_check: chrono::Utc::now(),
        })
    }
}

/// Create a mock LLM provider with default settings
pub fn create_mock_provider() -> MockLLMProvider {
    MockLLMProvider::default()
}

/// Create a mock LLM provider with custom settings
pub fn create_custom_mock_provider(latency_ms: u64, success_rate: f32) -> MockLLMProvider {
    MockLLMProvider::new("custom-mock".to_string(), latency_ms, success_rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_provider_intent_understanding() {
        let provider = create_mock_provider();
        let context = ContextSnapshot::default();

        let result = provider.understand_intent("give me a travel plan", &context).await;
        assert!(result.is_ok());

        let understanding = result.unwrap();
        assert_eq!(understanding.task_type, TaskType::Planning);
        assert!(understanding.confidence > 0.8);
    }

    #[tokio::test]
    async fn test_mock_provider_task_plan_creation() {
        let provider = create_mock_provider();
        let context = ContextSnapshot::default();

        let result = provider.create_task_plan("create a travel plan", &context).await;
        assert!(result.is_ok());

        let plan = result.unwrap();
        assert_eq!(plan.task_type, TaskType::Planning);
        assert!(!plan.steps.is_empty());
        assert!(plan.confidence > 0.7);
    }

    #[tokio::test]
    async fn test_mock_provider_health_check() {
        let provider = create_mock_provider();

        let result = provider.health_check().await;
        assert!(result.is_ok());

        let health = result.unwrap();
        assert!(health.is_healthy);
    }
}