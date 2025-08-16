use anyhow::{Result, Context};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};
use std::time::Instant;
use crate::CostTracker;

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
        Self {
            client: Client::new(),
            api_key,
            model: "gpt-3.5-turbo".to_string(),
            base_url: "https://api.openai.com/v1/chat/completions".to_string(),
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
        
        let prompt = self.create_parsing_prompt(user_input);
        let start_time = Instant::now();
        
        // Estimate cost before making the call
        let estimated_cost = cost_tracker.estimate_llm_operation_cost(prompt.len());
        if !cost_tracker.can_afford(estimated_cost) {
            return Err(anyhow::anyhow!("Cannot afford LLM operation: ${:.4}", estimated_cost));
        }

        // Make the API call
        let response = self.call_openai_api(&prompt).await
            .context("Failed to call OpenAI API")?;

        // Parse the response
        let parsed_command = self.parse_llm_response(&response.choices[0].message.content)
            .context("Failed to parse LLM response")?;

        // Calculate actual cost and record operation
        let actual_cost = self.calculate_cost(&response.usage);
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

    async fn call_openai_api(&self, prompt: &str) -> Result<OpenAIResponse> {
        let request = OpenAIRequest {
            model: self.model.clone(),
            messages: vec![Message {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            max_tokens: 500,
            temperature: 0.1, // Low temperature for consistent parsing
        };

        let response = self
            .client
            .post(&self.base_url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .context("Failed to send request to OpenAI")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "OpenAI API error {}: {}",
                status,
                error_text
            ));
        }

        let api_response: OpenAIResponse = response
            .json()
            .await
            .context("Failed to parse OpenAI response")?;

        Ok(api_response)
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

    fn calculate_cost(&self, usage: &Usage) -> f64 {
        // GPT-3.5-turbo pricing: $0.0005 input, $0.0015 output per 1K tokens
        let input_cost = (usage.prompt_tokens as f64 / 1000.0) * 0.0005;
        let output_cost = (usage.completion_tokens as f64 / 1000.0) * 0.0015;
        input_cost + output_cost
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
        }
    }
}