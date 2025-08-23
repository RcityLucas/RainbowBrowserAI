use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;
use regex::Regex;

use crate::traits::*;
use crate::utils::{Timer, cost};

/// Mock LLM Service implementation for development
pub struct MockLLMService {
    usage_stats: Arc<RwLock<UsageStats>>,
    known_sites: HashMap<String, String>,
}

impl MockLLMService {
    pub fn new() -> Self {
        let known_sites = vec![
            ("stackoverflow", "stackoverflow.com"),
            ("google", "google.com"),
            ("github", "github.com"),
            ("youtube", "youtube.com"),
            ("twitter", "twitter.com"),
            ("facebook", "facebook.com"),
            ("reddit", "reddit.com"),
            ("wikipedia", "wikipedia.org"),
            ("amazon", "amazon.com"),
            ("netflix", "netflix.com"),
        ]
        .into_iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();

        Self {
            usage_stats: Arc::new(RwLock::new(UsageStats {
                total_requests: 0,
                total_tokens: 0,
                total_cost: 0.0,
                requests_today: 0,
                tokens_today: 0,
                cost_today: 0.0,
                average_response_time_ms: 0.0,
                success_rate: 1.0,
            })),
            known_sites,
        }
    }

    /// Parse command in mock mode (extracted from POC)
    fn parse_command_mock(&self, input: &str) -> ParsedCommand {
        let input_lower = input.to_lowercase();
        
        // Enhanced URL extraction with better filtering
        let url_pattern = Regex::new(r"\b(?:https?://)?(?:www\.)?([a-zA-Z0-9](?:[a-zA-Z0-9\-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9\-]{0,61}[a-zA-Z0-9])?)*(?:\.[a-zA-Z]{2,}))(?:/[^\s]*)?\b").unwrap();
        
        let mut urls = Vec::new();
        
        // First, check for known sites
        for (site_name, site_url) in &self.known_sites {
            if input_lower.contains(site_name) {
                // Make sure it's not part of another word using word boundaries
                let word_pattern = format!(r"\b{}\b", regex::escape(site_name));
                if Regex::new(&word_pattern).unwrap().is_match(&input_lower) {
                    urls.push(format!("https://{}", site_url));
                    info!("Matched known site: {} -> {}", site_name, site_url);
                    break; // Take first match to avoid duplicates
                }
            }
        }
        
        // If no known sites found, extract URLs from text
        if urls.is_empty() {
            for cap in url_pattern.captures_iter(&input) {
                if let Some(domain) = cap.get(1) {
                    let domain_str = domain.as_str();
                    
                    // Filter out command words and short strings
                    if domain_str.len() > 2 && 
                       !domain_str.starts_with("to") &&
                       !domain_str.starts_with("go") &&
                       domain_str.contains('.') {
                        
                        let url = if input.contains("http") {
                            cap.get(0).unwrap().as_str().to_string()
                        } else {
                            format!("https://{}", domain_str)
                        };
                        
                        urls.push(url);
                        info!("Extracted URL: {}", urls.last().unwrap());
                    }
                }
            }
        }
        
        // Determine action
        let action = if input_lower.contains("screenshot") || input_lower.contains("capture") {
            "screenshot".to_string()
        } else if input_lower.contains("navigate") || input_lower.contains("go to") || input_lower.contains("visit") {
            "navigate".to_string()
        } else if !urls.is_empty() {
            "navigate".to_string() // Default action when URL is found
        } else {
            "unknown".to_string()
        };
        
        // Parse screenshot options
        let screenshot = input_lower.contains("screenshot") || input_lower.contains("capture");
        let viewport_only = input_lower.contains("viewport") && !input_lower.contains("full");
        
        // Extract filename if specified
        let filename = if let Some(cap) = Regex::new(r"(?:save|filename|name)(?:\s+as)?\s+([a-zA-Z0-9_\-\.]+)").unwrap().captures(&input_lower) {
            cap.get(1).map(|m| m.as_str().to_string())
        } else {
            None
        };
        
        let confidence = if !urls.is_empty() { 0.9 } else { 0.3 };
        
        ParsedCommand {
            action,
            url: urls.first().cloned(),
            urls,
            screenshot,
            filename,
            viewport_width: Some(1920),
            viewport_height: Some(1080),
            viewport_only: !screenshot || viewport_only, // Default to viewport for non-screenshot actions
            retries: Some(3),
            timeout: Some(30000),
            confidence,
            parameters: HashMap::new(),
        }
    }

    async fn update_stats(&self, tokens_used: u32, cost: f64, success: bool) {
        let mut stats = self.usage_stats.write().await;
        stats.total_requests += 1;
        stats.total_tokens += tokens_used as u64;
        stats.total_cost += cost;
        stats.requests_today += 1;
        stats.tokens_today += tokens_used as u64;
        stats.cost_today += cost;
        
        // Update success rate (simple moving average)
        let total_success = (stats.success_rate * (stats.total_requests - 1) as f64) + if success { 1.0 } else { 0.0 };
        stats.success_rate = total_success / stats.total_requests as f64;
    }
}

impl Default for MockLLMService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LLMService for MockLLMService {
    async fn parse_command(&self, input: &str, _context: &Context) -> Result<ParsedCommand> {
        let _timer = Timer::new("parse_command");
        
        let parsed = self.parse_command_mock(input);
        let tokens_used = (input.len() / 4) as u32; // Rough token estimation
        let cost = cost::calculate_openai_cost(tokens_used, 50, "gpt-3.5-turbo");
        
        self.update_stats(tokens_used + 50, cost, parsed.confidence > 0.5).await;
        
        info!("Parsed command: {} -> {} (confidence: {:.2})", 
              input, parsed.action, parsed.confidence);
        
        Ok(parsed)
    }

    async fn generate_response(&self, prompt: &str, options: GenerationOptions) -> Result<LLMResponse> {
        let _timer = Timer::new("generate_response");
        
        let input_tokens = (prompt.len() / 4) as u32;
        let output_tokens = options.max_tokens.unwrap_or(500);
        let cost = cost::calculate_openai_cost(input_tokens, output_tokens, "gpt-3.5-turbo");
        
        // Simple mock response generation
        let content = if prompt.contains("error") {
            "I encountered an issue while processing your request. Please try again.".to_string()
        } else if prompt.contains("success") {
            "Operation completed successfully!".to_string()
        } else {
            format!("Mock response for: {}", prompt.chars().take(50).collect::<String>())
        };
        
        self.update_stats(input_tokens + output_tokens, cost, true).await;
        
        Ok(LLMResponse {
            content,
            tokens_used: input_tokens + output_tokens,
            cost,
            provider: "mock".to_string(),
            model: "gpt-3.5-turbo-mock".to_string(),
            finish_reason: "stop".to_string(),
        })
    }

    async fn analyze_content(&self, content: &str, analysis_type: AnalysisType) -> Result<Analysis> {
        let _timer = Timer::new("analyze_content");
        
        let tokens_used = (content.len() / 4) as u32 + 200; // Content + analysis tokens
        let cost = cost::calculate_openai_cost(tokens_used, 200, "gpt-3.5-turbo");
        
        let mut results = HashMap::new();
        let analysis_type_str = format!("{:?}", analysis_type);
        
        match analysis_type {
            AnalysisType::Sentiment => {
                results.insert("sentiment".to_string(), serde_json::Value::String("neutral".to_string()));
                results.insert("score".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(0.5).unwrap()));
            }
            AnalysisType::KeyPhrases => {
                let phrases = vec!["mock phrase 1", "mock phrase 2", "mock phrase 3"];
                results.insert("phrases".to_string(), serde_json::Value::Array(
                    phrases.into_iter().map(|p| serde_json::Value::String(p.to_string())).collect()
                ));
            }
            AnalysisType::Summary => {
                results.insert("summary".to_string(), serde_json::Value::String(
                    format!("Mock summary of content ({}chars)", content.len())
                ));
            }
            AnalysisType::Classification { ref categories } => {
                let category = categories.first().unwrap_or(&"general".to_string()).clone();
                results.insert("category".to_string(), serde_json::Value::String(category));
                results.insert("confidence".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(0.8).unwrap()));
            }
            AnalysisType::Custom { prompt: _ } => {
                results.insert("result".to_string(), serde_json::Value::String("Mock custom analysis result".to_string()));
            }
        }
        
        self.update_stats(tokens_used, cost, true).await;
        
        Ok(Analysis {
            analysis_type: analysis_type_str,
            results,
            confidence: 0.8,
            tokens_used,
            cost,
        })
    }

    async fn estimate_cost(&self, operation: &LLMOperation) -> Result<f64> {
        let cost = match operation {
            LLMOperation::ParseCommand { input_length } => {
                let tokens = (*input_length / 4) as u32 + 50;
                cost::calculate_openai_cost(tokens, 50, "gpt-3.5-turbo")
            }
            LLMOperation::GenerateResponse { prompt_length, max_tokens } => {
                let input_tokens = (*prompt_length / 4) as u32;
                cost::calculate_openai_cost(input_tokens, *max_tokens, "gpt-3.5-turbo")
            }
            LLMOperation::AnalyzeContent { content_length, analysis_type: _ } => {
                let tokens = (*content_length / 4) as u32 + 200;
                cost::calculate_openai_cost(tokens, 200, "gpt-3.5-turbo")
            }
        };
        
        Ok(cost)
    }

    async fn get_usage_stats(&self) -> Result<UsageStats> {
        let stats = self.usage_stats.read().await;
        Ok(stats.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_parse_command_stackoverflow() {
        let service = MockLLMService::new();
        let context = Context {
            conversation_history: vec![],
            current_page_info: None,
            user_preferences: HashMap::new(),
            session_data: HashMap::new(),
        };

        let result = service.parse_command("go to stackoverflow and take screenshot", &context).await.unwrap();
        
        assert_eq!(result.action, "screenshot");
        assert!(result.screenshot);
        assert_eq!(result.url.unwrap(), "https://stackoverflow.com");
        assert!(result.confidence > 0.8);
    }

    #[tokio::test]
    async fn test_parse_command_url_extraction() {
        let service = MockLLMService::new();
        let context = Context {
            conversation_history: vec![],
            current_page_info: None,
            user_preferences: HashMap::new(),
            session_data: HashMap::new(),
        };

        let result = service.parse_command("navigate to https://example.com", &context).await.unwrap();
        
        assert_eq!(result.action, "navigate");
        assert_eq!(result.url.unwrap(), "https://example.com");
    }

    #[tokio::test]
    async fn test_generate_response() {
        let service = MockLLMService::new();
        let options = GenerationOptions::default();

        let result = service.generate_response("Test prompt", options).await.unwrap();
        
        assert!(!result.content.is_empty());
        assert_eq!(result.provider, "mock");
        assert!(result.tokens_used > 0);
    }

    #[tokio::test]
    async fn test_usage_stats() {
        let service = MockLLMService::new();
        
        // Perform some operations
        let context = Context {
            conversation_history: vec![],
            current_page_info: None,
            user_preferences: HashMap::new(),
            session_data: HashMap::new(),
        };
        
        service.parse_command("test", &context).await.unwrap();
        
        let stats = service.get_usage_stats().await.unwrap();
        assert_eq!(stats.total_requests, 1);
        assert!(stats.total_tokens > 0);
    }
}