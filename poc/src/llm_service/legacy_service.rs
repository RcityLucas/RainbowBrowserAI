use anyhow::{Result, Context};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};
use std::time::Instant;
use crate::CostTracker;

// Import enhanced task understanding module
use crate::llm_service::llm_service_enhanced::{TaskUnderstanding, MockTaskUnderstanding};

#[derive(Debug, Clone)]
pub struct LLMService {
    client: Client,
    pub api_key: String,
    model: String,
    base_url: String,
}

#[derive(Debug, Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<Message>,
    max_tokens: u32,
    temperature: f32,
}

#[derive(Debug, Serialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<Message>,
    temperature: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    choices: Vec<Choice>,
    usage: Usage,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: Message,
}

#[derive(Debug, Deserialize)]
struct Usage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[derive(Debug, Deserialize)]
struct AnthropicResponse {
    content: Vec<AnthropicContent>,
    usage: AnthropicUsage,
}

#[derive(Debug, Deserialize)]
struct AnthropicContent {
    text: String,
}

#[derive(Debug, Deserialize)]
struct AnthropicUsage {
    input_tokens: u32,
    output_tokens: u32,
}

#[derive(Debug)]
struct UnifiedUsage {
    input_tokens: u32,
    output_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedCommand {
    pub action: String,
    pub url: Option<String>,
    pub urls: Vec<String>,
    pub screenshot: bool,
    pub filename: Option<String>,
    pub viewport_width: Option<u32>,
    pub viewport_height: Option<u32>,
    pub viewport_only: bool,
    pub retries: Option<u32>,
    pub timeout: Option<u64>,
    pub confidence: f32,
    pub parameters: CommandParams,
    pub scroll_direction: Option<String>,
    pub element_selector: Option<String>,
    pub input_text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandParams {
    pub take_screenshot: bool,
    pub screenshot_filename: Option<String>,
    pub viewport_width: Option<u32>,
    pub viewport_height: Option<u32>,
    pub retries: Option<u32>,
    pub timeout_seconds: Option<u64>,
    pub show_report: bool,
}

impl LLMService {
    pub fn new(api_key: String) -> Self {
        // Check environment for API provider configuration
        let provider = std::env::var("LLM_PROVIDER").unwrap_or_default();
        let (base_url, model) = match provider.as_str() {
            "chatapi" => (
                std::env::var("CHATAPI_BASE_URL").unwrap_or_else(|_| "https://api.chatanywhere.tech/v1/chat/completions".to_string()),
                std::env::var("CHATAPI_MODEL").unwrap_or_else(|_| "gpt-3.5-turbo".to_string())
            ),
            "azure" => (
                std::env::var("AZURE_OPENAI_ENDPOINT").unwrap_or_else(|_| "https://your-resource.openai.azure.com/openai/deployments/your-deployment/chat/completions?api-version=2023-07-01-preview".to_string()),
                std::env::var("AZURE_OPENAI_MODEL").unwrap_or_else(|_| "gpt-35-turbo".to_string())
            ),
            "anthropic" => (
                "https://api.anthropic.com/v1/messages".to_string(),
                std::env::var("ANTHROPIC_MODEL").unwrap_or_else(|_| "claude-3-sonnet-20240229".to_string())
            ),
            _ => (
                "https://api.openai.com/v1/chat/completions".to_string(),
                "gpt-3.5-turbo".to_string()
            )
        };

        Self {
            client: Client::new(),
            api_key,
            model,
            base_url,
        }
    }

    pub fn with_model(mut self, model: String) -> Self {
        self.model = model;
        self
    }

    pub async fn parse_natural_command(
        &self,
        user_input: &str,
        cost_tracker: &mut CostTracker,
    ) -> Result<ParsedCommand> {
        info!("Parsing natural language command: {}", user_input);
        
        // Check if mock mode is enabled
        let mock_mode = std::env::var("RAINBOW_MOCK_MODE")
            .unwrap_or_else(|_| "false".to_string())
            .to_lowercase() == "true";
        
        if mock_mode {
            info!("🎭 Using mock mode for LLM parsing");
            return self.parse_command_mock(user_input, cost_tracker);
        }
        
        let prompt = self.create_parsing_prompt(user_input);
        let start_time = Instant::now();
        
        // Estimate cost before making the call
        let estimated_cost = cost_tracker.estimate_llm_operation_cost(prompt.len());
        if !cost_tracker.can_afford(estimated_cost) {
            return Err(anyhow::anyhow!("Cannot afford LLM operation: ${:.4}", estimated_cost));
        }

        // Make the API call
        let (response_text, usage_info) = self.call_llm_api(&prompt).await
            .context("Failed to call LLM API")?;

        // Parse the response
        let parsed_command = self.parse_llm_response(&response_text)
            .context("Failed to parse LLM response")?;

        // Calculate actual cost and record operation
        let actual_cost = self.calculate_cost_from_usage(&usage_info);
        let duration = start_time.elapsed();
        
        cost_tracker.record_operation(
            "llm_parse".to_string(),
            format!("Parse command: {}", user_input),
            actual_cost,
            true,
        )?;

        info!(
            "LLM parsing completed in {:.2}s, cost: ${:.4}, confidence: {:.2}",
            duration.as_secs_f32(),
            actual_cost,
            parsed_command.confidence
        );

        Ok(parsed_command)
    }

    fn create_parsing_prompt(&self, user_input: &str) -> String {
        format!(
            r#"Parse this browser automation command into structured JSON. Respond ONLY with valid JSON.

User command: "{}"

Extract these fields:
- action: "navigate", "test", "report", or "unknown"
- url: single URL if navigating to one site (without protocol is ok)
- urls: array of URLs if testing multiple sites
- screenshot: boolean if user wants screenshots
- filename: custom filename if specified
- viewport_width: custom width if specified (default 1920)
- viewport_height: custom height if specified (default 1080) 
- viewport_only: true if user wants viewport-only screenshot
- retries: number of retries if specified
- timeout: timeout in seconds if specified
- confidence: 0.0-1.0 confidence in parsing

Examples:
"navigate to google" -> {{"action":"navigate","url":"google.com","screenshot":false,"confidence":0.9}}
"go to github and take a screenshot" -> {{"action":"navigate","url":"github.com","screenshot":true,"confidence":0.95}}
"test google, github, stackoverflow with screenshots" -> {{"action":"test","urls":["google.com","github.com","stackoverflow.com"],"screenshot":true,"confidence":0.9}}
"show cost report" -> {{"action":"report","confidence":0.95}}

JSON:"#,
            user_input
        )
    }

    async fn call_llm_api(&self, prompt: &str) -> Result<(String, UnifiedUsage)> {
        let provider = std::env::var("LLM_PROVIDER").unwrap_or_default();
        
        match provider.as_str() {
            "anthropic" => self.call_anthropic_api(prompt).await,
            _ => self.call_openai_api(prompt).await,
        }
    }

    async fn call_openai_api(&self, prompt: &str) -> Result<(String, UnifiedUsage)> {
        let request = OpenAIRequest {
            model: self.model.clone(),
            messages: vec![Message {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            max_tokens: 500,
            temperature: 0.1, // Low temperature for consistent parsing
        };

        // Build request with appropriate headers for different providers
        let provider = std::env::var("LLM_PROVIDER").unwrap_or_default();
        let mut request_builder = self
            .client
            .post(&self.base_url)
            .header("Content-Type", "application/json");

        // Add provider-specific headers
        request_builder = match provider.as_str() {
            "chatapi" => {
                // ChatAPI typically uses OpenAI-compatible format
                request_builder.header("Authorization", format!("Bearer {}", self.api_key))
            },
            "azure" => {
                // Azure OpenAI uses api-key header
                request_builder.header("api-key", &self.api_key)
            },
            "anthropic" => {
                // Anthropic uses different headers
                request_builder
                    .header("x-api-key", &self.api_key)
                    .header("anthropic-version", "2023-06-01")
            },
            _ => {
                // Default OpenAI format
                request_builder.header("Authorization", format!("Bearer {}", self.api_key))
            }
        };

        let response = request_builder
            .json(&request)
            .send()
            .await
            .context("Failed to send request to LLM API")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "LLM API error {}: {}",
                status,
                error_text
            ));
        }

        let api_response: OpenAIResponse = response
            .json()
            .await
            .context("Failed to parse LLM API response")?;

        let unified_usage = UnifiedUsage {
            input_tokens: api_response.usage.prompt_tokens,
            output_tokens: api_response.usage.completion_tokens,
        };

        Ok((api_response.choices[0].message.content.clone(), unified_usage))
    }

    async fn call_anthropic_api(&self, prompt: &str) -> Result<(String, UnifiedUsage)> {
        let request = AnthropicRequest {
            model: self.model.clone(),
            max_tokens: 500,
            messages: vec![Message {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            temperature: 0.1,
        };

        let response = self
            .client
            .post(&self.base_url)
            .header("Content-Type", "application/json")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&request)
            .send()
            .await
            .context("Failed to send request to Anthropic API")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "Anthropic API error {}: {}",
                status,
                error_text
            ));
        }

        let api_response: AnthropicResponse = response
            .json()
            .await
            .context("Failed to parse Anthropic API response")?;

        let unified_usage = UnifiedUsage {
            input_tokens: api_response.usage.input_tokens,
            output_tokens: api_response.usage.output_tokens,
        };

        let response_text = api_response.content.get(0)
            .map(|c| c.text.clone())
            .unwrap_or_default();

        Ok((response_text, unified_usage))
    }

    fn parse_llm_response(&self, response_text: &str) -> Result<ParsedCommand> {
        // Try to extract JSON from the response
        let json_str = if let Some(start) = response_text.find('{') {
            if let Some(end) = response_text.rfind('}') {
                &response_text[start..=end]
            } else {
                response_text
            }
        } else {
            response_text
        };

        let mut parsed: ParsedCommand = serde_json::from_str(json_str)
            .context("Failed to parse JSON response from LLM")?;

        // Validate and clean up the parsed command
        self.validate_and_clean_command(&mut parsed)?;

        Ok(parsed)
    }

    fn validate_and_clean_command(&self, command: &mut ParsedCommand) -> Result<()> {
        // Validate action
        match command.action.as_str() {
            "navigate" | "test" | "report" | "unknown" => {},
            _ => {
                warn!("Unknown action '{}', setting to 'unknown'", command.action);
                command.action = "unknown".to_string();
                command.confidence *= 0.5; // Reduce confidence
            }
        }

        // Clean and validate URLs
        if let Some(ref mut url) = command.url {
            *url = self.clean_url(url);
        }

        for url in &mut command.urls {
            *url = self.clean_url(url);
        }

        // Validate viewport dimensions
        if let Some(width) = command.viewport_width {
            if width < 320 || width > 4000 {
                warn!("Invalid viewport width {}, using default", width);
                command.viewport_width = None;
            }
        }

        if let Some(height) = command.viewport_height {
            if height < 200 || height > 4000 {
                warn!("Invalid viewport height {}, using default", height);
                command.viewport_height = None;
            }
        }

        // Validate retries
        if let Some(retries) = command.retries {
            if retries > 10 {
                warn!("Too many retries {}, limiting to 10", retries);
                command.retries = Some(10);
            }
        }

        // Validate timeout
        if let Some(timeout) = command.timeout {
            if timeout > 300 {
                warn!("Timeout too high {}s, limiting to 300s", timeout);
                command.timeout = Some(300);
            }
        }

        // Ensure confidence is in valid range
        command.confidence = command.confidence.clamp(0.0, 1.0);
        
        // Populate parameters field from individual fields
        command.parameters = CommandParams {
            take_screenshot: command.screenshot,
            screenshot_filename: command.filename.clone(),
            viewport_width: command.viewport_width,
            viewport_height: command.viewport_height,
            retries: command.retries,
            timeout_seconds: command.timeout,
            show_report: false,
        };

        Ok(())
    }

    fn clean_url(&self, url: &str) -> String {
        let mut cleaned = url.trim().to_lowercase();
        
        // Remove common prefixes that users might include
        if cleaned.starts_with("www.") {
            cleaned = cleaned[4..].to_string();
        }
        
        // Remove protocols if present
        if cleaned.starts_with("https://") {
            cleaned = cleaned[8..].to_string();
        } else if cleaned.starts_with("http://") {
            cleaned = cleaned[7..].to_string();
        }
        
        // Remove trailing slashes
        cleaned = cleaned.trim_end_matches('/').to_string();
        
        // Add .com if it looks like a domain without extension
        if !cleaned.contains('.') && !cleaned.is_empty() {
            cleaned = format!("{}.com", cleaned);
        }
        
        cleaned
    }

    fn calculate_cost_from_usage(&self, usage: &UnifiedUsage) -> f64 {
        let provider = std::env::var("LLM_PROVIDER").unwrap_or_default();
        
        match provider.as_str() {
            "anthropic" => {
                // Claude 3 Haiku pricing: $0.00025 input, $0.00125 output per 1K tokens
                let input_cost = (usage.input_tokens as f64 / 1000.0) * 0.00025;
                let output_cost = (usage.output_tokens as f64 / 1000.0) * 0.00125;
                input_cost + output_cost
            },
            _ => {
                // GPT-3.5-turbo pricing: $0.0005 input, $0.0015 output per 1K tokens
                let input_cost = (usage.input_tokens as f64 / 1000.0) * 0.0005;
                let output_cost = (usage.output_tokens as f64 / 1000.0) * 0.0015;
                input_cost + output_cost
            }
        }
    }

    fn calculate_cost(&self, usage: &Usage) -> f64 {
        // Legacy method for backwards compatibility
        let unified = UnifiedUsage {
            input_tokens: usage.prompt_tokens,
            output_tokens: usage.completion_tokens,
        };
        self.calculate_cost_from_usage(&unified)
    }

    fn parse_command_mock(&self, user_input: &str, cost_tracker: &mut CostTracker) -> Result<ParsedCommand> {
        use regex::Regex;
        use std::time::Duration;
        use std::thread;
        
        // Simulate processing time
        thread::sleep(Duration::from_millis(300));
        
        let input_lower = user_input.to_lowercase();
        let mut command = ParsedCommand::default();
        
        // Detect action type with enhanced patterns
        if input_lower.contains("test") && (input_lower.contains("sites") || input_lower.contains("websites") || input_lower.contains(",") || input_lower.contains("these")) {
            command.action = "test".to_string();
            command.confidence = 0.9;
            
            // Extract URLs from comma-separated list or common sites
            let url_regex = Regex::new(r"([a-zA-Z0-9][a-zA-Z0-9-]*\.)+[a-zA-Z]{2,}").unwrap();
            for cap in url_regex.captures_iter(&input_lower) {
                command.urls.push(cap[0].to_string());
            }
            
            // Enhanced common site detection
            if command.urls.is_empty() {
                let sites = [
                    ("google", "google.com"), ("github", "github.com"), 
                    ("stackoverflow", "stackoverflow.com"), ("reddit", "reddit.com"),
                    ("youtube", "youtube.com"), ("twitter", "twitter.com"),
                    ("facebook", "facebook.com"), ("linkedin", "linkedin.com")
                ];
                
                for (name, url) in &sites {
                    if input_lower.contains(name) {
                        command.urls.push(url.to_string());
                    }
                }
            }
        } else if input_lower.contains("get") && input_lower.contains("element") && input_lower.contains("info") {
            // V8.0 Memory Tool: get_element_info
            command.action = "get_element_info".to_string();
            command.confidence = 0.9;
            
            // Extract element reference (body, header, etc.)
            if input_lower.contains("body") {
                command.element_selector = Some("body".to_string());
            } else if input_lower.contains("header") {
                command.element_selector = Some("header".to_string());
            } else if let Some(cap) = Regex::new(r"#([a-zA-Z0-9_-]+)").unwrap().captures(&input_lower) {
                command.element_selector = Some(format!("#{}", &cap[1]));
            } else if let Some(cap) = Regex::new(r"\.([a-zA-Z0-9_-]+)").unwrap().captures(&input_lower) {
                command.element_selector = Some(format!(".{}", &cap[1]));
            }
            
            // Extract URL if present
            let url_regex = Regex::new(r"([a-zA-Z0-9][a-zA-Z0-9-]*\.)+[a-zA-Z]{2,}").unwrap();
            if let Some(cap) = url_regex.captures(&input_lower) {
                command.url = Some(cap[0].to_string());
            }
        } else if input_lower.contains("take") && input_lower.contains("screenshot") {
            // V8.0 Memory Tool: take_screenshot  
            command.action = "take_screenshot".to_string();
            command.confidence = 0.95;
            command.screenshot = true;
            
            // Extract viewport preference
            if input_lower.contains("viewport") || input_lower.contains("current") {
                command.viewport_only = true;
            } else if input_lower.contains("full") || input_lower.contains("page") {
                command.viewport_only = false;
            }
            
            // Extract URL if present
            let url_regex = Regex::new(r"([a-zA-Z0-9][a-zA-Z0-9-]*\.)+[a-zA-Z]{2,}").unwrap();
            if let Some(cap) = url_regex.captures(&input_lower) {
                command.url = Some(cap[0].to_string());
            }
        } else if (input_lower.contains("get") || input_lower.contains("retrieve")) && input_lower.contains("history") {
            // V8.0 Memory Tool: retrieve_history
            command.action = "retrieve_history".to_string();
            command.confidence = 0.9;
            
            // Extract count if specified
            if let Some(cap) = Regex::new(r"(\d+)").unwrap().captures(&input_lower) {
                // Store count in retries field for mock purposes
                command.retries = cap[1].parse().ok();
            } else {
                // Default to 10 history items
                command.retries = Some(10);
            }
        } else if input_lower.contains("report") && input_lower.contains("insight") {
            // V8.0 Metacognitive Tool: report_insight
            command.action = "report_insight".to_string();
            command.confidence = 0.95;
            
            // Extract insight category
            if input_lower.contains("performance") {
                command.input_text = Some("performance".to_string());
            } else if input_lower.contains("security") {
                command.input_text = Some("security".to_string());
            } else if input_lower.contains("usability") {
                command.input_text = Some("usability".to_string());
            } else if input_lower.contains("error") {
                command.input_text = Some("error".to_string());
            } else {
                command.input_text = Some("general".to_string());
            }
        } else if (input_lower.contains("complete") || input_lower.contains("mark")) && input_lower.contains("task") {
            // V8.0 Metacognitive Tool: complete_task
            command.action = "complete_task".to_string();
            command.confidence = 0.9;
            
            // Extract task ID or name
            if let Some(cap) = Regex::new(r"task\s+([a-zA-Z0-9_-]+)").unwrap().captures(&input_lower) {
                command.input_text = Some(cap[1].to_string());
            }
            
            // Extract success status
            if input_lower.contains("successful") || input_lower.contains("completed") {
                command.element_selector = Some("success".to_string());
            } else if input_lower.contains("failed") || input_lower.contains("error") {
                command.element_selector = Some("failed".to_string());
            }
        } else if (input_lower.contains("wait") && input_lower.contains("element")) || input_lower.contains("wait for") {
            // wait_for_element tool
            command.action = "wait_for_element".to_string();
            command.confidence = 0.9;
            
            // Extract element selector or description
            if let Some(cap) = Regex::new(r#"(?:selector|element)?\s*[\"']([^\"']+)[\"']"#).unwrap().captures(&input_lower) {
                command.element_selector = Some(cap[1].to_string());
            } else if input_lower.contains("button") {
                command.element_selector = Some("button".to_string());
            } else if input_lower.contains("link") {
                command.element_selector = Some("a".to_string());
            } else if input_lower.contains("form") {
                command.element_selector = Some("form".to_string());
            }
            
            // Extract timeout if specified
            if let Some(cap) = Regex::new(r"(\d+)\s*(?:second|sec)s?").unwrap().captures(&input_lower) {
                command.timeout = cap[1].parse::<u64>().ok().map(|s| s * 1000);
            } else if let Some(cap) = Regex::new(r"(\d+)\s*(?:ms|millisecond)s?").unwrap().captures(&input_lower) {
                command.timeout = cap[1].parse().ok();
            }
        } else if input_lower.contains("select") && (input_lower.contains("option") || input_lower.contains("dropdown") || input_lower.contains("choose")) {
            // select_option tool
            command.action = "select_option".to_string();
            command.confidence = 0.9;
            
            // Extract selector
            if let Some(cap) = Regex::new(r#"(?:from|in|selector)\s*[\"']([^\"']+)[\"']"#).unwrap().captures(&input_lower) {
                command.element_selector = Some(cap[1].to_string());
            } else if input_lower.contains("dropdown") {
                command.element_selector = Some("select".to_string());
            }
            
            // Extract selection value
            if let Some(cap) = Regex::new(r#"(?:select|choose|option)\s*[\"']([^\"']+)[\"']"#).unwrap().captures(&input_lower) {
                command.input_text = Some(cap[1].to_string());
            }
        } else if (input_lower.contains("wait") && input_lower.contains("condition")) || 
                  (input_lower.contains("wait") && (input_lower.contains("until") || input_lower.contains("page") || input_lower.contains("load"))) {
            // wait_for_condition tool
            command.action = "wait_for_condition".to_string();
            command.confidence = 0.85;
            
            // Determine condition type
            if input_lower.contains("load") || input_lower.contains("ready") {
                command.input_text = Some("page_ready".to_string());
            } else if input_lower.contains("url") {
                command.input_text = Some("url_change".to_string());
            } else if input_lower.contains("text") || input_lower.contains("content") {
                command.input_text = Some("text_contains".to_string());
            } else {
                command.input_text = Some("network_idle".to_string());
            }
            
            // Extract timeout
            if let Some(cap) = Regex::new(r"(\d+)\s*(?:second|sec)s?").unwrap().captures(&input_lower) {
                command.timeout = cap[1].parse::<u64>().ok().map(|s| s * 1000);
            }
        } else if (input_lower.contains("type") && input_lower.contains("text")) || input_lower.contains("enter text") {
            // Enhanced type_text tool (different from basic input)
            command.action = "type_text".to_string();
            command.confidence = 0.9;
            
            // Extract text to type
            if let Some(cap) = Regex::new(r#"(?:type|enter)\s*[\"']([^\"']+)[\"']"#).unwrap().captures(&input_lower) {
                command.input_text = Some(cap[1].to_string());
            }
            
            // Extract element selector
            if let Some(cap) = Regex::new(r#"(?:into|in|selector)\s*[\"']([^\"']+)[\"']"#).unwrap().captures(&input_lower) {
                command.element_selector = Some(cap[1].to_string());
            } else if input_lower.contains("input") {
                command.element_selector = Some("input".to_string());
            } else if input_lower.contains("textarea") {
                command.element_selector = Some("textarea".to_string());
            }
        } else if input_lower.contains("extract") || input_lower.contains("scrape") || input_lower.contains("data") {
            command.action = "extract".to_string();
            command.confidence = 0.85;
            
            // Extract URL for data extraction
            let url_regex = Regex::new(r"([a-zA-Z0-9][a-zA-Z0-9-]*\.)+[a-zA-Z]{2,}").unwrap();
            if let Some(cap) = url_regex.captures(&input_lower) {
                command.url = Some(cap[0].to_string());
            }
        } else if input_lower.contains("monitor") || input_lower.contains("watch") || input_lower.contains("check") {
            command.action = "monitor".to_string();
            command.confidence = 0.8;
            
            // Extract URL to monitor
            let url_regex = Regex::new(r"([a-zA-Z0-9][a-zA-Z0-9-]*\.)+[a-zA-Z]{2,}").unwrap();
            if let Some(cap) = url_regex.captures(&input_lower) {
                command.url = Some(cap[0].to_string());
            }
        } else if (input_lower.contains("search for") || input_lower.contains("search ") || input_lower.contains("find ")) && !input_lower.contains("navigate") {
            // Handle search commands explicitly before navigation
            command.action = "search".to_string();
            command.confidence = 0.9;
            
            // Extract the search query
            if let Some(query_start) = input_lower.find("search for ") {
                let query = user_input[query_start + 11..].trim().to_string();
                command.input_text = Some(query);
            } else if let Some(query_start) = input_lower.find("search ") {
                let query = user_input[query_start + 7..].trim().to_string();
                command.input_text = Some(query);
            } else if let Some(query_start) = input_lower.find("find ") {
                let query = user_input[query_start + 5..].trim().to_string();
                command.input_text = Some(query);
            }
        } else if input_lower.contains("navigate") || input_lower.contains("go to") || input_lower.contains("open") || input_lower.contains("visit") || input_lower.contains("browse") {
            command.action = "navigate".to_string();
            command.confidence = 0.95;
            
            // First try to extract a complete URL with domain extension
            let complete_url_regex = Regex::new(r"([a-zA-Z0-9][a-zA-Z0-9-]*\.)+[a-zA-Z]{2,}").unwrap();
            if let Some(cap) = complete_url_regex.captures(&input_lower) {
                command.url = Some(cap[0].to_string());
            } else {
                // If no complete URL, look for known websites or domain names
                // Skip common command words when looking for domain names
                let filtered_input = input_lower
                    .replace("navigate to", "")
                    .replace("go to", "")
                    .replace("open", "")
                    .replace("visit", "")
                    .replace("browse to", "")
                    .replace(" and ", " ")
                    .replace(" take ", " ")
                    .replace(" screenshot", "")
                    .replace(" the ", " ")
                    .replace(" a ", " ")
                    .replace(" to ", " ");
                
                // Now look for domain-like words in the filtered input
                info!("Filtered input for URL extraction: '{}'", filtered_input.trim());
                
                // Look for common website names first
                let known_sites = [
                    ("stackoverflow", "stackoverflow.com"),
                    ("google", "google.com"),
                    ("github", "github.com"),
                    ("youtube", "youtube.com"),
                    ("reddit", "reddit.com"),
                    ("twitter", "twitter.com"),
                    ("facebook", "facebook.com"),
                    ("amazon", "amazon.com"),
                    ("wikipedia", "wikipedia.org"),
                    ("linkedin", "linkedin.com"),
                ];
                
                let mut found_url = None;
                for (name, full_url) in &known_sites {
                    if filtered_input.contains(name) {
                        found_url = Some(full_url.to_string());
                        info!("Found known site: {} -> {}", name, full_url);
                        break;
                    }
                }
                
                // If no known site found, try generic domain extraction
                if found_url.is_none() {
                    let domain_regex = Regex::new(r"\b([a-zA-Z][a-zA-Z0-9]{2,})\b").unwrap();
                    if let Some(cap) = domain_regex.captures(&filtered_input) {
                        let mut url = cap[1].to_string();
                        info!("Extracted domain word: '{}'", url);
                        // Add .com if it's just a word
                        if !url.contains('.') {
                            url.push_str(".com");
                        }
                        found_url = Some(url);
                    }
                }
                
                command.url = found_url;
            }
        } else if input_lower.contains("scroll") {
            command.action = "scroll".to_string();
            command.confidence = 0.9;
            
            // Detect scroll direction/type
            if input_lower.contains("top") || input_lower.contains("up") {
                if input_lower.contains("page") {
                    command.scroll_direction = Some("page_up".to_string());
                } else {
                    command.scroll_direction = Some("top".to_string());
                }
            } else if input_lower.contains("bottom") || input_lower.contains("down") {
                if input_lower.contains("page") {
                    command.scroll_direction = Some("page_down".to_string());
                } else {
                    command.scroll_direction = Some("bottom".to_string());
                }
            } else {
                // Default scroll direction
                command.scroll_direction = Some("down".to_string());
            }
        } else if input_lower.contains("click") {
            command.action = "click".to_string();
            command.confidence = 0.85;
            
            // Extract element selector if present
            if let Some(cap) = Regex::new(r#"(?:selector|element)\s*["']([^"']+)["']"#).unwrap().captures(&input_lower) {
                command.element_selector = Some(cap[1].to_string());
            } else if let Some(cap) = Regex::new(r#"["']([^"']+)["']"#).unwrap().captures(&input_lower) {
                command.element_selector = Some(cap[1].to_string());
            }
        } else if input_lower.contains("input") || input_lower.contains("type") || input_lower.contains("enter") {
            command.action = "input".to_string();
            command.confidence = 0.85;
            
            // Extract text to input
            if let Some(cap) = Regex::new(r#"(?:input|type|enter)\s*["']([^"']+)["']"#).unwrap().captures(&input_lower) {
                command.input_text = Some(cap[1].to_string());
            }
            
            // Extract element selector
            if let Some(cap) = Regex::new(r#"(?:into|selector)\s*["']([^"']+)["']"#).unwrap().captures(&input_lower) {
                command.element_selector = Some(cap[1].to_string());
            }
        } else if input_lower.contains("refresh") || input_lower.contains("reload") || input_lower == "page_refresh" {
            command.action = "refresh".to_string();
            command.confidence = 0.9;
        } else if input_lower.contains("back") || input_lower.contains("previous") || input_lower == "page_back" {
            command.action = "back".to_string();
            command.confidence = 0.9;
        } else if input_lower.contains("forward") || input_lower.contains("next") || input_lower == "page_forward" {
            command.action = "forward".to_string();
            command.confidence = 0.9;
        } else if input_lower.contains("report") || input_lower.contains("cost") || input_lower.contains("usage") {
            command.action = "report".to_string();
            command.confidence = 0.95;
        } else {
            // Try enhanced task understanding for complex commands
            info!("Trying enhanced understanding for: '{}'", input_lower);
            let task_understanding = MockTaskUnderstanding;
            match task_understanding.classify_intent(&input_lower) {
                Ok(task_type) => {
                    info!("Enhanced understanding classified as: {:?}", task_type);
                    use crate::llm_service::llm_service_enhanced::TaskType;
                    match task_type {
                        TaskType::Planning => {
                            info!("Detected planning task: will use TaskExecutor");
                            command.action = "planning".to_string();
                            command.confidence = 0.85;
                            
                            // Set parameters to indicate this is a planning task
                            command.parameters.show_report = true;
                        },
                        TaskType::Search => {
                            command.action = "search".to_string();
                            command.confidence = 0.8;
                            // Extract the search query from the input
                            if let Some(query_start) = input_lower.find("search for ") {
                                let query = user_input[query_start + 11..].trim().to_string();
                                command.input_text = Some(query);
                            } else if let Some(query_start) = input_lower.find("search ") {
                                let query = user_input[query_start + 7..].trim().to_string();
                                command.input_text = Some(query);
                            } else if let Some(query_start) = input_lower.find("find ") {
                                let query = user_input[query_start + 5..].trim().to_string();
                                command.input_text = Some(query);
                            }
                        },
                        TaskType::Analysis => {
                            command.action = "analyze".to_string();
                            command.confidence = 0.8;
                        },
                        _ => {
                            command.action = "unknown".to_string();
                            command.confidence = 0.3;
                        }
                    }
                }
                Err(e) => {
                    info!("Enhanced understanding failed with error: {}", e);
                    command.action = "unknown".to_string();
                    command.confidence = 0.3;
                }
            }
        }
        
        // Detect screenshot request
        if input_lower.contains("screenshot") || input_lower.contains("capture") || input_lower.contains("snap") {
            command.screenshot = true;
            command.confidence = command.confidence.min(0.9);
        }
        
        // Detect viewport settings
        if let Some(cap) = Regex::new(r"(\d{3,4})\s*x\s*(\d{3,4})").unwrap().captures(&input_lower) {
            command.viewport_width = cap[1].parse().ok();
            command.viewport_height = cap[2].parse().ok();
        }
        
        // Set parameters based on parsed fields
        command.parameters = CommandParams {
            take_screenshot: command.screenshot,
            screenshot_filename: command.filename.clone(),
            viewport_width: command.viewport_width,
            viewport_height: command.viewport_height,
            retries: command.retries,
            timeout_seconds: command.timeout,
            show_report: false,
        };
        
        // Record mock operation (no cost)
        cost_tracker.record_operation(
            "llm_parse_mock".to_string(),
            format!("Mock parse: {}", user_input),
            0.0,
            true,
        )?;
        
        info!("Mock parsing result: action={}, confidence={:.2}", command.action, command.confidence);
        
        Ok(command)
    }
    
    pub async fn explain_command(&self, command: &ParsedCommand) -> String {
        match command.action.as_str() {
            "navigate" => {
                if let Some(ref url) = command.url {
                    if command.screenshot {
                        format!("Navigate to {} and take a screenshot", url)
                    } else {
                        format!("Navigate to {}", url)
                    }
                } else {
                    "Navigate command with no URL specified".to_string()
                }
            },
            "test" => {
                let url_count = command.urls.len();
                if command.screenshot {
                    format!("Test {} websites with screenshots", url_count)
                } else {
                    format!("Test {} websites", url_count)
                }
            },
            "report" => "Show cost and usage report".to_string(),
            _ => format!("Unknown command (confidence: {:.1}%)", command.confidence * 100.0),
        }
    }
}

impl Default for ParsedCommand {
    fn default() -> Self {
        Self {
            action: "unknown".to_string(),
            url: None,
            urls: Vec::new(),
            screenshot: false,
            filename: None,
            viewport_width: None,
            viewport_height: None,
            viewport_only: false,
            retries: None,
            timeout: None,
            confidence: 0.0,
            parameters: CommandParams {
                take_screenshot: false,
                screenshot_filename: None,
                viewport_width: None,
                viewport_height: None,
                retries: None,
                timeout_seconds: None,
                show_report: false,
            },
            scroll_direction: None,
            element_selector: None,
            input_text: None,
        }
    }
}