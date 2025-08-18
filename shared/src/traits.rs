use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// Common types
pub type SessionId = Uuid;
pub type Result<T> = std::result::Result<T, anyhow::Error>;

// Browser Service Trait
#[async_trait]
pub trait BrowserService: Send + Sync {
    /// Create a new browser session
    async fn create_session(&self) -> Result<SessionId>;
    
    /// Navigate to a URL
    async fn navigate(&self, session: &SessionId, url: &str) -> Result<NavigationResult>;
    
    /// Take a screenshot
    async fn screenshot(&self, session: &SessionId, options: ScreenshotOptions) -> Result<Vec<u8>>;
    
    /// Extract content from current page
    async fn extract_content(&self, session: &SessionId) -> Result<PageContent>;
    
    /// Interact with page elements
    async fn interact(&self, session: &SessionId, action: InteractionAction) -> Result<()>;
    
    /// Close a browser session
    async fn close_session(&self, session: &SessionId) -> Result<()>;
    
    /// Get current page URL
    async fn get_current_url(&self, session: &SessionId) -> Result<String>;
    
    /// Get page title
    async fn get_title(&self, session: &SessionId) -> Result<String>;
}

// LLM Service Trait
#[async_trait]
pub trait LLMService: Send + Sync {
    /// Parse natural language command
    async fn parse_command(&self, input: &str, context: &Context) -> Result<ParsedCommand>;
    
    /// Generate response based on prompt
    async fn generate_response(&self, prompt: &str, options: GenerationOptions) -> Result<LLMResponse>;
    
    /// Analyze content for insights
    async fn analyze_content(&self, content: &str, analysis_type: AnalysisType) -> Result<Analysis>;
    
    /// Estimate cost for operation
    async fn estimate_cost(&self, operation: &LLMOperation) -> Result<f64>;
    
    /// Get usage statistics
    async fn get_usage_stats(&self) -> Result<UsageStats>;
}

// Data structures for Browser Service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigationResult {
    pub url: String,
    pub title: Option<String>,
    pub status_code: Option<u16>,
    pub load_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenshotOptions {
    pub full_page: bool,
    pub viewport_width: Option<u32>,
    pub viewport_height: Option<u32>,
    pub wait_after_load: Option<std::time::Duration>,
    pub element_selector: Option<String>,
}

impl Default for ScreenshotOptions {
    fn default() -> Self {
        Self {
            full_page: true,
            viewport_width: Some(1920),
            viewport_height: Some(1080),
            wait_after_load: Some(std::time::Duration::from_secs(2)),
            element_selector: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageContent {
    pub url: String,
    pub title: Option<String>,
    pub meta_description: Option<String>,
    pub headings: Vec<String>,
    pub text_content: Vec<String>,
    pub links: Vec<LinkInfo>,
    pub images: Vec<ImageInfo>,
    pub forms: Vec<FormInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkInfo {
    pub text: String,
    pub href: String,
    pub title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageInfo {
    pub src: String,
    pub alt: Option<String>,
    pub title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormInfo {
    pub action: Option<String>,
    pub method: String,
    pub fields: Vec<FormField>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormField {
    pub name: Option<String>,
    pub field_type: String,
    pub placeholder: Option<String>,
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InteractionAction {
    Click { selector: String },
    Type { selector: String, text: String },
    Clear { selector: String },
    Submit { selector: String },
    Scroll { direction: ScrollDirection, amount: Option<i32> },
    Wait { duration_ms: u64 },
    WaitForElement { selector: String, timeout_ms: u64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScrollDirection {
    Up,
    Down,
    Left,
    Right,
    ToElement { selector: String },
}

// Data structures for LLM Service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Context {
    pub conversation_history: Vec<HistoryEntry>,
    pub current_page_info: Option<PageInfo>,
    pub user_preferences: HashMap<String, serde_json::Value>,
    pub session_data: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub user_input: String,
    pub ai_response: String,
    pub actions_taken: Vec<String>,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageInfo {
    pub url: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub content_summary: Option<String>,
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
    pub parameters: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationOptions {
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub frequency_penalty: Option<f32>,
    pub presence_penalty: Option<f32>,
    pub stop_sequences: Option<Vec<String>>,
}

impl Default for GenerationOptions {
    fn default() -> Self {
        Self {
            max_tokens: Some(500),
            temperature: Some(0.1),
            top_p: None,
            frequency_penalty: None,
            presence_penalty: None,
            stop_sequences: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMResponse {
    pub content: String,
    pub tokens_used: u32,
    pub cost: f64,
    pub provider: String,
    pub model: String,
    pub finish_reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnalysisType {
    Sentiment,
    KeyPhrases,
    Summary,
    Classification { categories: Vec<String> },
    Custom { prompt: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Analysis {
    pub analysis_type: String,
    pub results: HashMap<String, serde_json::Value>,
    pub confidence: f32,
    pub tokens_used: u32,
    pub cost: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LLMOperation {
    ParseCommand { input_length: usize },
    GenerateResponse { prompt_length: usize, max_tokens: u32 },
    AnalyzeContent { content_length: usize, analysis_type: AnalysisType },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageStats {
    pub total_requests: u64,
    pub total_tokens: u64,
    pub total_cost: f64,
    pub requests_today: u64,
    pub tokens_today: u64,
    pub cost_today: f64,
    pub average_response_time_ms: f64,
    pub success_rate: f64,
}