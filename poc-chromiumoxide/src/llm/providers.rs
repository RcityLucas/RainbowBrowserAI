// LLM Provider implementations
// Supports OpenAI GPT and Claude API integration

use anyhow::Result;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{error, info};

use super::{LLMConfig, LLMError, LLMResponse, TokenUsage};

/// Trait for LLM providers
#[async_trait]
pub trait LLMProvider: Send + Sync {
    async fn query(&mut self, prompt: &str, config: &LLMConfig) -> Result<LLMResponse, LLMError>;
    fn provider_name(&self) -> &str;
    fn is_available(&self) -> bool;
}

/// OpenAI GPT provider
pub struct OpenAIProvider {
    client: Client,
    api_key: String,
    base_url: String,
}

impl OpenAIProvider {
    pub fn new(config: &LLMConfig) -> Result<Self, LLMError> {
        let api_key = config
            .openai_api_key
            .as_ref()
            .ok_or_else(|| LLMError::ConfigError("OpenAI API key required".to_string()))?
            .clone();

        let client = Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .map_err(|e| LLMError::NetworkError(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            client,
            api_key,
            base_url: "https://api.openai.com/v1".to_string(),
        })
    }
}

#[async_trait]
impl LLMProvider for OpenAIProvider {
    async fn query(&mut self, prompt: &str, config: &LLMConfig) -> Result<LLMResponse, LLMError> {
        let request = OpenAIRequest {
            model: "gpt-4".to_string(),
            messages: vec![OpenAIMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            max_tokens: Some(config.max_tokens),
            temperature: Some(config.temperature),
        };

        info!("Sending request to OpenAI API");
        let response = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| LLMError::NetworkError(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            error!("OpenAI API error {}: {}", status, error_text);

            return match status.as_u16() {
                429 => Err(LLMError::RateLimit(error_text)),
                401 | 403 => Err(LLMError::AuthError(error_text)),
                _ => Err(LLMError::ApiError(format!("{}: {}", status, error_text))),
            };
        }

        let openai_response: OpenAIResponse = response
            .json()
            .await
            .map_err(|e| LLMError::InvalidResponse(format!("Failed to parse response: {}", e)))?;

        let choice = openai_response
            .choices
            .first()
            .ok_or_else(|| LLMError::InvalidResponse("No choices in response".to_string()))?;

        Ok(LLMResponse {
            content: choice.message.content.clone(),
            model: openai_response.model,
            usage: TokenUsage {
                prompt_tokens: openai_response.usage.prompt_tokens,
                completion_tokens: openai_response.usage.completion_tokens,
                total_tokens: openai_response.usage.total_tokens,
            },
            finish_reason: choice
                .finish_reason
                .clone()
                .unwrap_or_else(|| "unknown".to_string()),
            timestamp: chrono::Utc::now(),
        })
    }

    fn provider_name(&self) -> &str {
        "openai"
    }

    fn is_available(&self) -> bool {
        !self.api_key.is_empty()
    }
}

/// Claude provider (Anthropic)
pub struct ClaudeProvider {
    client: Client,
    api_key: String,
    base_url: String,
}

impl ClaudeProvider {
    pub fn new(config: &LLMConfig) -> Result<Self, LLMError> {
        let api_key = config
            .claude_api_key
            .as_ref()
            .ok_or_else(|| LLMError::ConfigError("Claude API key required".to_string()))?
            .clone();

        let client = Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .map_err(|e| LLMError::NetworkError(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            client,
            api_key,
            base_url: "https://api.anthropic.com/v1".to_string(),
        })
    }
}

#[async_trait]
impl LLMProvider for ClaudeProvider {
    async fn query(&mut self, prompt: &str, config: &LLMConfig) -> Result<LLMResponse, LLMError> {
        let request = ClaudeRequest {
            model: "claude-3-sonnet-20240229".to_string(),
            max_tokens: config.max_tokens,
            temperature: Some(config.temperature),
            messages: vec![ClaudeMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
        };

        info!("Sending request to Claude API");
        let response = self
            .client
            .post(format!("{}/messages", self.base_url))
            .header("x-api-key", &self.api_key)
            .header("Content-Type", "application/json")
            .header("anthropic-version", "2023-06-01")
            .json(&request)
            .send()
            .await
            .map_err(|e| LLMError::NetworkError(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            error!("Claude API error {}: {}", status, error_text);

            return match status.as_u16() {
                429 => Err(LLMError::RateLimit(error_text)),
                401 | 403 => Err(LLMError::AuthError(error_text)),
                _ => Err(LLMError::ApiError(format!("{}: {}", status, error_text))),
            };
        }

        let claude_response: ClaudeResponse = response
            .json()
            .await
            .map_err(|e| LLMError::InvalidResponse(format!("Failed to parse response: {}", e)))?;

        let content = claude_response
            .content
            .first()
            .ok_or_else(|| LLMError::InvalidResponse("No content in response".to_string()))?;

        Ok(LLMResponse {
            content: content.text.clone(),
            model: claude_response.model,
            usage: TokenUsage {
                prompt_tokens: claude_response.usage.input_tokens,
                completion_tokens: claude_response.usage.output_tokens,
                total_tokens: claude_response.usage.input_tokens
                    + claude_response.usage.output_tokens,
            },
            finish_reason: claude_response
                .stop_reason
                .unwrap_or_else(|| "unknown".to_string()),
            timestamp: chrono::Utc::now(),
        })
    }

    fn provider_name(&self) -> &str {
        "claude"
    }

    fn is_available(&self) -> bool {
        !self.api_key.is_empty()
    }
}

/// Mock provider for testing
pub struct MockProvider {
    responses: Vec<String>,
    current_index: usize,
}

impl MockProvider {
    pub fn new() -> Self {
        Self {
            responses: vec![
                "Mock response 1".to_string(),
                "Mock response 2".to_string(),
                "Mock response 3".to_string(),
            ],
            current_index: 0,
        }
    }

    pub fn with_responses(responses: Vec<String>) -> Self {
        Self {
            responses,
            current_index: 0,
        }
    }
}

impl Default for MockProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LLMProvider for MockProvider {
    async fn query(&mut self, _prompt: &str, _config: &LLMConfig) -> Result<LLMResponse, LLMError> {
        let response = self
            .responses
            .get(self.current_index)
            .unwrap_or(&"Default mock response".to_string())
            .clone();

        self.current_index = (self.current_index + 1) % self.responses.len();

        Ok(LLMResponse {
            content: response,
            model: "mock-model".to_string(),
            usage: TokenUsage {
                prompt_tokens: 10,
                completion_tokens: 20,
                total_tokens: 30,
            },
            finish_reason: "stop".to_string(),
            timestamp: chrono::Utc::now(),
        })
    }

    fn provider_name(&self) -> &str {
        "mock"
    }

    fn is_available(&self) -> bool {
        true
    }
}

// OpenAI API request/response structures
#[derive(Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    max_tokens: Option<u32>,
    temperature: Option<f32>,
}

#[derive(Serialize, Deserialize)]
struct OpenAIMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct OpenAIResponse {
    model: String,
    choices: Vec<OpenAIChoice>,
    usage: OpenAIUsage,
}

#[derive(Deserialize)]
struct OpenAIChoice {
    message: OpenAIMessage,
    finish_reason: Option<String>,
}

#[derive(Deserialize)]
struct OpenAIUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

// Claude API request/response structures
#[derive(Serialize)]
struct ClaudeRequest {
    model: String,
    max_tokens: u32,
    temperature: Option<f32>,
    messages: Vec<ClaudeMessage>,
}

#[derive(Serialize)]
struct ClaudeMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ClaudeResponse {
    model: String,
    content: Vec<ClaudeContent>,
    usage: ClaudeUsage,
    stop_reason: Option<String>,
}

#[derive(Deserialize)]
struct ClaudeContent {
    text: String,
}

#[derive(Deserialize)]
struct ClaudeUsage {
    input_tokens: u32,
    output_tokens: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_provider() {
        let provider = MockProvider::new();
        assert_eq!(provider.provider_name(), "mock");
        assert!(provider.is_available());
    }

    #[tokio::test]
    async fn test_mock_provider_responses() {
        let mut provider =
            MockProvider::with_responses(vec!["Response 1".to_string(), "Response 2".to_string()]);

        let config = LLMConfig::default();

        let response1 = provider.query("test prompt", &config).await.unwrap();
        assert_eq!(response1.content, "Response 1");

        let response2 = provider.query("test prompt", &config).await.unwrap();
        assert_eq!(response2.content, "Response 2");

        // Should cycle back to first response
        let response3 = provider.query("test prompt", &config).await.unwrap();
        assert_eq!(response3.content, "Response 1");
    }
}
