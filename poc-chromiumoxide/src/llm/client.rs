// LLM Client implementation
// Provides unified interface for different LLM providers

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// Response from LLM API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMResponse {
    pub content: String,
    pub model: String,
    pub usage: TokenUsage,
    pub finish_reason: String,
    pub timestamp: DateTime<Utc>,
}

/// Token usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// LLM-specific error types
#[derive(Debug, thiserror::Error)]
pub enum LLMError {
    #[error("API request failed: {0}")]
    ApiError(String),
    
    #[error("Rate limit exceeded: {0}")]
    RateLimit(String),
    
    #[error("Invalid response format: {0}")]
    InvalidResponse(String),
    
    #[error("Authentication failed: {0}")]
    AuthError(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
}

/// Main LLM client interface
pub struct LLMClient {
    provider: Box<dyn crate::llm::providers::LLMProvider>,
    config: super::LLMConfig,
}

impl LLMClient {
    /// Create new client with specified provider
    pub fn new(provider: Box<dyn crate::llm::providers::LLMProvider>, config: super::LLMConfig) -> Self {
        Self { provider, config }
    }
    
    /// Send a query to the LLM
    pub async fn query(&mut self, prompt: &str) -> Result<LLMResponse, LLMError> {
        self.provider.query(prompt, &self.config).await
    }
    
    /// Send a query with context
    pub async fn query_with_context(
        &mut self, 
        prompt: &str, 
        context: &HashMap<String, serde_json::Value>
    ) -> Result<LLMResponse, LLMError> {
        let contextualized_prompt = self.build_contextualized_prompt(prompt, context)?;
        self.query(&contextualized_prompt).await
    }
    
    /// Build prompt with context information
    fn build_contextualized_prompt(
        &self, 
        prompt: &str, 
        context: &HashMap<String, serde_json::Value>
    ) -> Result<String, LLMError> {
        let mut contextualized = String::new();
        
        // Add context information
        if !context.is_empty() {
            contextualized.push_str("Context Information:\n");
            for (key, value) in context {
                contextualized.push_str(&format!("{}: {}\n", key, value));
            }
            contextualized.push_str("\n");
        }
        
        // Add the main prompt
        contextualized.push_str("Task: ");
        contextualized.push_str(prompt);
        
        Ok(contextualized)
    }
    
    /// Check if client is properly configured
    pub fn is_configured(&self) -> bool {
        match self.config.default_provider.as_str() {
            "openai" => self.config.openai_api_key.is_some(),
            "claude" => self.config.claude_api_key.is_some(),
            _ => false,
        }
    }
    
    /// Get current provider name
    pub fn provider_name(&self) -> &str {
        &self.config.default_provider
    }
}

/// Builder for LLM client
pub struct LLMClientBuilder {
    config: super::LLMConfig,
}

impl LLMClientBuilder {
    pub fn new() -> Self {
        Self {
            config: super::LLMConfig::default(),
        }
    }
    
    pub fn provider(mut self, provider: String) -> Self {
        self.config.default_provider = provider;
        self
    }
    
    pub fn openai_key(mut self, key: String) -> Self {
        self.config.openai_api_key = Some(key);
        self
    }
    
    pub fn claude_key(mut self, key: String) -> Self {
        self.config.claude_api_key = Some(key);
        self
    }
    
    pub fn max_tokens(mut self, tokens: u32) -> Self {
        self.config.max_tokens = tokens;
        self
    }
    
    pub fn temperature(mut self, temp: f32) -> Self {
        self.config.temperature = temp;
        self
    }
    
    pub fn cost_limit(mut self, limit: f32) -> Self {
        self.config.cost_limit_usd = limit;
        self
    }
    
    pub fn build(self) -> Result<LLMClient, LLMError> {
        let provider: Box<dyn crate::llm::providers::LLMProvider> = match self.config.default_provider.as_str() {
            "openai" => {
                if self.config.openai_api_key.is_none() {
                    return Err(LLMError::ConfigError("OpenAI API key required".to_string()));
                }
                Box::new(crate::llm::providers::OpenAIProvider::new(&self.config)?)
            }
            "claude" => {
                if self.config.claude_api_key.is_none() {
                    return Err(LLMError::ConfigError("Claude API key required".to_string()));
                }
                Box::new(crate::llm::providers::ClaudeProvider::new(&self.config)?)
            }
            _ => return Err(LLMError::ConfigError(format!("Unsupported provider: {}", self.config.default_provider))),
        };
        
        Ok(LLMClient::new(provider, self.config))
    }
}

impl Default for LLMClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_client_builder() {
        let builder = LLMClientBuilder::new()
            .provider("openai".to_string())
            .max_tokens(2000)
            .temperature(0.5);
            
        assert_eq!(builder.config.default_provider, "openai");
        assert_eq!(builder.config.max_tokens, 2000);
        assert_eq!(builder.config.temperature, 0.5);
    }
    
    #[test]
    fn test_contextualized_prompt() {
        let config = super::super::LLMConfig::default();
        let provider = Box::new(crate::llm::providers::MockProvider::new());
        let client = LLMClient::new(provider, config);
        
        let mut context = HashMap::new();
        context.insert("page_url".to_string(), serde_json::Value::String("https://example.com".to_string()));
        context.insert("element_count".to_string(), serde_json::Value::Number(serde_json::Number::from(5)));
        
        let prompt = client.build_contextualized_prompt("Find the login button", &context).unwrap();
        
        assert!(prompt.contains("Context Information:"));
        assert!(prompt.contains("page_url: \"https://example.com\""));
        assert!(prompt.contains("element_count: 5"));
        assert!(prompt.contains("Task: Find the login button"));
    }
}