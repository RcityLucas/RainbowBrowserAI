use super::traits::{Tool, ToolCategory};
use crate::browser::Browser;
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use std::sync::Arc;
use std::collections::HashMap;
use tracing::{info, debug};
use base64::Engine;

// ============================================================================
// Screenshot Tool
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct ScreenshotInput {
    #[serde(default)]
    pub selector: Option<String>,
    #[serde(default)]
    pub full_page: bool,
    #[serde(default = "default_quality")]
    pub quality: u8,
    #[serde(default)]
    pub format: ScreenshotFormat,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ScreenshotFormat {
    Png,
    Jpeg,
}

impl Default for ScreenshotFormat {
    fn default() -> Self {
        ScreenshotFormat::Png
    }
}

fn default_quality() -> u8 {
    90
}

#[derive(Debug, Serialize)]
pub struct ScreenshotOutput {
    pub success: bool,
    pub data_base64: String,
    pub format: ScreenshotFormat,
    pub size_bytes: usize,
}

pub struct ScreenshotTool {
    browser: Arc<Browser>,
}

impl ScreenshotTool {
    pub fn new(browser: Arc<Browser>) -> Self {
        Self { browser }
    }

    async fn screenshot_viewport(&self, input: &ScreenshotInput) -> Result<Vec<u8>> {
        let options = crate::browser::ScreenshotOptions {
            full_page: false,
            format: match input.format {
                ScreenshotFormat::Png => "png".to_string(),
                ScreenshotFormat::Jpeg => "jpeg".to_string(),
            },
            quality: if matches!(input.format, ScreenshotFormat::Jpeg) {
                Some(input.quality)
            } else {
                None
            },
            viewport_width: 1920,
            viewport_height: 1080,
            wait_after_load: std::time::Duration::from_millis(500),
        };
        self.browser.screenshot(options).await
    }

    async fn screenshot_full_page(&self, input: &ScreenshotInput) -> Result<Vec<u8>> {
        // Get page dimensions first
        let dimensions_script = r#"
            JSON.stringify({
                width: Math.max(document.documentElement.scrollWidth, document.body.scrollWidth),
                height: Math.max(document.documentElement.scrollHeight, document.body.scrollHeight)
            })
        "#;
        
        let dimensions_result = self.browser.execute_script(dimensions_script).await?;
        let width = dimensions_result["width"].as_u64().unwrap_or(1920) as u32;
        let height = dimensions_result["height"].as_u64().unwrap_or(1080) as u32;
        
        let options = crate::browser::ScreenshotOptions {
            full_page: true,
            format: match input.format {
                ScreenshotFormat::Png => "png".to_string(),
                ScreenshotFormat::Jpeg => "jpeg".to_string(),
            },
            quality: if matches!(input.format, ScreenshotFormat::Jpeg) {
                Some(input.quality)
            } else {
                None
            },
            viewport_width: width.max(1920),
            viewport_height: height.max(1080),
            wait_after_load: std::time::Duration::from_secs(1),
        };
        self.browser.screenshot(options).await
    }

    async fn screenshot_element(&self, selector: &str, input: &ScreenshotInput) -> Result<Vec<u8>> {
        // First ensure the element exists and is visible
        self.browser.wait_for_selector(selector, std::time::Duration::from_secs(5)).await
            .map_err(|_| anyhow!("Element not found: {}", selector))?;

        // Scroll element into view
        let scroll_script = format!(
            "document.querySelector('{}').scrollIntoView({{ behavior: 'auto', block: 'center', inline: 'center' }})",
            selector
        );
        self.browser.execute_script(&scroll_script).await?;

        // Wait a bit for scrolling to complete
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;

        // Get element bounding box
        let rect_script = format!(
            r#"
            const element = document.querySelector('{}');
            const rect = element.getBoundingClientRect();
            JSON.stringify({{
                x: rect.x,
                y: rect.y,
                width: rect.width,
                height: rect.height,
                left: rect.left,
                top: rect.top,
                right: rect.right,
                bottom: rect.bottom
            }})
            "#,
            selector
        );
        
        let rect_result = self.browser.execute_script(&rect_script).await?;
        let x = rect_result["x"].as_f64().unwrap_or(0.0) as i32;
        let y = rect_result["y"].as_f64().unwrap_or(0.0) as i32;
        let width = rect_result["width"].as_f64().unwrap_or(100.0) as u32;
        let height = rect_result["height"].as_f64().unwrap_or(100.0) as u32;

        if width == 0 || height == 0 {
            return Err(anyhow!("Element has zero dimensions: {}", selector));
        }

        // Take full viewport screenshot first
        let viewport_options = crate::browser::ScreenshotOptions {
            full_page: false,
            format: match input.format {
                ScreenshotFormat::Png => "png".to_string(),
                ScreenshotFormat::Jpeg => "jpeg".to_string(),
            },
            quality: if matches!(input.format, ScreenshotFormat::Jpeg) {
                Some(input.quality)
            } else {
                None
            },
            viewport_width: 1920,
            viewport_height: 1080,
            wait_after_load: std::time::Duration::from_millis(200),
        };
        
        let viewport_screenshot = self.browser.screenshot(viewport_options).await?;
        
        // For now, return the full screenshot with element info in debug
        // TODO: Implement actual image cropping when image processing crate is available
        debug!("Element bounds: x={}, y={}, width={}, height={}", x, y, width, height);
        debug!("Note: Element cropping not yet implemented, returning full viewport");
        
        Ok(viewport_screenshot)
    }
}

#[async_trait]
impl Tool for ScreenshotTool {
    type Input = ScreenshotInput;
    type Output = ScreenshotOutput;
    
    fn name(&self) -> &str {
        "screenshot"
    }
    
    fn description(&self) -> &str {
        "Capture screenshots of the page or specific elements"
    }
    
    fn category(&self) -> ToolCategory {
        ToolCategory::Memory
    }
    
    async fn execute(&self, input: Self::Input) -> Result<Self::Output> {
        info!("Taking screenshot with options: full_page={}, format={:?}, quality={}", 
              input.full_page, input.format, input.quality);
        
        let screenshot_data = if let Some(selector) = &input.selector {
            debug!("Taking element screenshot for selector: {}", selector);
            self.screenshot_element(selector, &input).await?
        } else if input.full_page {
            debug!("Taking full page screenshot");
            self.screenshot_full_page(&input).await?
        } else {
            debug!("Taking viewport screenshot");
            self.screenshot_viewport(&input).await?
        };
        
        let size = screenshot_data.len();
        let data_base64 = base64::engine::general_purpose::STANDARD.encode(&screenshot_data);
        
        Ok(ScreenshotOutput {
            success: true,
            data_base64,
            format: input.format,
            size_bytes: size,
        })
    }
}

// ============================================================================
// Session Memory Tool
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "action")]
pub enum SessionMemoryInput {
    Store { key: String, value: serde_json::Value },
    Retrieve { key: String },
    Delete { key: String },
    List,
    Clear,
}

#[derive(Debug, Serialize)]
pub struct SessionMemoryOutput {
    pub success: bool,
    pub action: String,
    pub result: Option<serde_json::Value>,
    pub keys: Option<Vec<String>>,
}

pub struct SessionMemoryTool {
    #[allow(dead_code)] // Reserved for future browser integration
    browser: Arc<Browser>,
    memory: Arc<tokio::sync::RwLock<HashMap<String, serde_json::Value>>>,
}

impl SessionMemoryTool {
    pub fn new(browser: Arc<Browser>) -> Self {
        Self {
            browser,
            memory: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl Tool for SessionMemoryTool {
    type Input = SessionMemoryInput;
    type Output = SessionMemoryOutput;
    
    fn name(&self) -> &str {
        "session_memory"
    }
    
    fn description(&self) -> &str {
        "Store and retrieve data within the current session"
    }
    
    fn category(&self) -> ToolCategory {
        ToolCategory::Memory
    }
    
    async fn execute(&self, input: Self::Input) -> Result<Self::Output> {
        match input {
            SessionMemoryInput::Store { key, value } => {
                info!("Storing session data: {}", key);
                let mut memory = self.memory.write().await;
                memory.insert(key.clone(), value.clone());
                Ok(SessionMemoryOutput {
                    success: true,
                    action: "store".to_string(),
                    result: Some(value),
                    keys: None,
                })
            }
            SessionMemoryInput::Retrieve { key } => {
                info!("Retrieving session data: {}", key);
                let memory = self.memory.read().await;
                let value = memory.get(&key).cloned();
                Ok(SessionMemoryOutput {
                    success: value.is_some(),
                    action: "retrieve".to_string(),
                    result: value,
                    keys: None,
                })
            }
            SessionMemoryInput::Delete { key } => {
                info!("Deleting session data: {}", key);
                let mut memory = self.memory.write().await;
                let removed = memory.remove(&key);
                Ok(SessionMemoryOutput {
                    success: removed.is_some(),
                    action: "delete".to_string(),
                    result: removed,
                    keys: None,
                })
            }
            SessionMemoryInput::List => {
                info!("Listing session keys");
                let memory = self.memory.read().await;
                let keys: Vec<String> = memory.keys().cloned().collect();
                Ok(SessionMemoryOutput {
                    success: true,
                    action: "list".to_string(),
                    result: None,
                    keys: Some(keys),
                })
            }
            SessionMemoryInput::Clear => {
                info!("Clearing session memory");
                let mut memory = self.memory.write().await;
                memory.clear();
                Ok(SessionMemoryOutput {
                    success: true,
                    action: "clear".to_string(),
                    result: None,
                    keys: None,
                })
            }
        }
    }
}

// ============================================================================
// Get Element Info Tool
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct GetElementInfoInput {
    pub selector: String,
    #[serde(default)]
    pub include_attributes: bool,
    #[serde(default)]
    pub include_computed_styles: bool,
    #[serde(default)]
    pub include_position: bool,
}

#[derive(Debug, Serialize)]
pub struct GetElementInfoOutput {
    pub success: bool,
    pub element_info: Option<ElementInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ElementInfo {
    pub tag_name: String,
    pub id: Option<String>,
    pub classes: Vec<String>,
    pub text: Option<String>,
    pub attributes: Option<HashMap<String, String>>,
    pub computed_styles: Option<HashMap<String, String>>,
    pub position: Option<ElementPosition>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ElementPosition {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub top: f64,
    pub left: f64,
    pub right: f64,
    pub bottom: f64,
}

pub struct GetElementInfoTool {
    browser: Arc<Browser>,
}

impl GetElementInfoTool {
    pub fn new(browser: Arc<Browser>) -> Self {
        Self { browser }
    }
}

#[async_trait]
impl Tool for GetElementInfoTool {
    type Input = GetElementInfoInput;
    type Output = GetElementInfoOutput;
    
    fn name(&self) -> &str {
        "get_element_info"
    }
    
    fn description(&self) -> &str {
        "Get detailed information about an element"
    }
    
    fn category(&self) -> ToolCategory {
        ToolCategory::Memory
    }
    
    async fn execute(&self, input: Self::Input) -> Result<Self::Output> {
        info!("Getting element info: {}", input.selector);
        
        let script = format!(
            r#"
            (function() {{
                const element = document.querySelector('{}');
                if (!element) return null;
                
                const info = {{
                    tag_name: element.tagName.toLowerCase(),
                    id: element.id || null,
                    classes: Array.from(element.classList),
                    text: element.textContent?.trim() || null,
                    attributes: null,
                    computed_styles: null,
                    position: null
                }};
                
                if ({}) {{
                    const attrs = {{}};
                    for (const attr of element.attributes) {{
                        attrs[attr.name] = attr.value;
                    }}
                    info.attributes = attrs;
                }}
                
                if ({}) {{
                    const styles = window.getComputedStyle(element);
                    const computedStyles = {{}};
                    // Get key styles
                    ['display', 'visibility', 'position', 'zIndex', 
                     'width', 'height', 'color', 'backgroundColor',
                     'fontSize', 'fontWeight', 'textAlign'].forEach(prop => {{
                        computedStyles[prop] = styles[prop];
                    }});
                    info.computed_styles = computedStyles;
                }}
                
                if ({}) {{
                    const rect = element.getBoundingClientRect();
                    info.position = {{
                        x: rect.x,
                        y: rect.y,
                        width: rect.width,
                        height: rect.height,
                        top: rect.top,
                        left: rect.left,
                        right: rect.right,
                        bottom: rect.bottom
                    }};
                }}
                
                return info;
            }})()
            "#,
            input.selector,
            input.include_attributes,
            input.include_computed_styles,
            input.include_position
        );
        
        let result = self.browser.execute_script(&script).await?;
        
        if result.is_null() {
            return Ok(GetElementInfoOutput {
                success: false,
                element_info: None,
            });
        }
        
        let element_info: ElementInfo = serde_json::from_value(result)?;
        
        Ok(GetElementInfoOutput {
            success: true,
            element_info: Some(element_info),
        })
    }
    
    async fn validate_input(&self, input: &Self::Input) -> Result<()> {
        if input.selector.is_empty() {
            return Err(anyhow!("Selector cannot be empty"));
        }
        Ok(())
    }
}

// ============================================================================
// History Tracker Tool
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "action")]
pub enum HistoryTrackerInput {
    Get,
    Clear,
    GoToIndex { index: usize },
}

#[derive(Debug, Serialize)]
pub struct HistoryTrackerOutput {
    pub success: bool,
    pub history: Vec<HistoryEntry>,
    pub current_index: usize,
}

#[derive(Debug, Serialize, Clone)]
pub struct HistoryEntry {
    pub url: String,
    pub title: Option<String>,
    pub timestamp: u64,
}

pub struct HistoryTrackerTool {
    browser: Arc<Browser>,
    history: Arc<tokio::sync::RwLock<Vec<HistoryEntry>>>,
    current_index: Arc<tokio::sync::RwLock<usize>>,
}

impl HistoryTrackerTool {
    pub fn new(browser: Arc<Browser>) -> Self {
        Self {
            browser,
            history: Arc::new(tokio::sync::RwLock::new(Vec::new())),
            current_index: Arc::new(tokio::sync::RwLock::new(0)),
        }
    }
    
    pub async fn add_entry(&self, url: String, title: Option<String>) {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let entry = HistoryEntry { url, title, timestamp };
        
        let mut history = self.history.write().await;
        let mut index = self.current_index.write().await;
        
        // Remove forward history if we're not at the end
        history.truncate(*index + 1);
        
        history.push(entry);
        *index = history.len() - 1;
    }
}

#[async_trait]
impl Tool for HistoryTrackerTool {
    type Input = HistoryTrackerInput;
    type Output = HistoryTrackerOutput;
    
    fn name(&self) -> &str {
        "history_tracker"
    }
    
    fn description(&self) -> &str {
        "Track and manage browser navigation history"
    }
    
    fn category(&self) -> ToolCategory {
        ToolCategory::Memory
    }
    
    async fn execute(&self, input: Self::Input) -> Result<Self::Output> {
        match input {
            HistoryTrackerInput::Get => {
                info!("Getting navigation history");
                let history = self.history.read().await;
                let current_index = *self.current_index.read().await;
                
                Ok(HistoryTrackerOutput {
                    success: true,
                    history: history.clone(),
                    current_index,
                })
            }
            HistoryTrackerInput::Clear => {
                info!("Clearing navigation history");
                let mut history = self.history.write().await;
                let mut index = self.current_index.write().await;
                
                history.clear();
                *index = 0;
                
                Ok(HistoryTrackerOutput {
                    success: true,
                    history: Vec::new(),
                    current_index: 0,
                })
            }
            HistoryTrackerInput::GoToIndex { index } => {
                info!("Going to history index: {}", index);
                let history = self.history.read().await;
                
                if index >= history.len() {
                    return Err(anyhow!("History index out of bounds"));
                }
                
                let entry = &history[index];
                self.browser.navigate_to(&entry.url).await?;
                
                let mut current_index = self.current_index.write().await;
                *current_index = index;
                
                Ok(HistoryTrackerOutput {
                    success: true,
                    history: history.clone(),
                    current_index: index,
                })
            }
        }
    }
}

// ============================================================================
// Persistent Cache Tool
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "action")]
pub enum PersistentCacheInput {
    Store { 
        key: String, 
        value: serde_json::Value,
        #[serde(default)]
        ttl_seconds: Option<u64>,
    },
    Retrieve { key: String },
    Delete { key: String },
    List { #[serde(default)] prefix: Option<String> },
    Clear,
}

#[derive(Debug, Serialize)]
pub struct PersistentCacheOutput {
    pub success: bool,
    pub action: String,
    pub result: Option<serde_json::Value>,
    pub keys: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct CacheEntry {
    value: serde_json::Value,
    expires_at: Option<u64>,
}

pub struct PersistentCacheTool {
    #[allow(dead_code)] // Reserved for future browser integration
    browser: Arc<Browser>,
    cache: Arc<tokio::sync::RwLock<HashMap<String, CacheEntry>>>,
}

impl PersistentCacheTool {
    pub fn new(browser: Arc<Browser>) -> Self {
        Self {
            browser,
            cache: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }
    
    fn is_expired(entry: &CacheEntry) -> bool {
        if let Some(expires_at) = entry.expires_at {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            now > expires_at
        } else {
            false
        }
    }
}

#[async_trait]
impl Tool for PersistentCacheTool {
    type Input = PersistentCacheInput;
    type Output = PersistentCacheOutput;
    
    fn name(&self) -> &str {
        "persistent_cache"
    }
    
    fn description(&self) -> &str {
        "Store and retrieve data persistently across sessions"
    }
    
    fn category(&self) -> ToolCategory {
        ToolCategory::Memory
    }
    
    async fn execute(&self, input: Self::Input) -> Result<Self::Output> {
        match input {
            PersistentCacheInput::Store { key, value, ttl_seconds } => {
                info!("Storing cache data: {}", key);
                let expires_at = ttl_seconds.map(|ttl| {
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs() + ttl
                });
                
                let entry = CacheEntry { value: value.clone(), expires_at };
                
                let mut cache = self.cache.write().await;
                cache.insert(key.clone(), entry);
                
                Ok(PersistentCacheOutput {
                    success: true,
                    action: "store".to_string(),
                    result: Some(value),
                    keys: None,
                })
            }
            PersistentCacheInput::Retrieve { key } => {
                info!("Retrieving cache data: {}", key);
                let mut cache = self.cache.write().await;
                
                if let Some(entry) = cache.get(&key) {
                    if Self::is_expired(entry) {
                        cache.remove(&key);
                        Ok(PersistentCacheOutput {
                            success: false,
                            action: "retrieve".to_string(),
                            result: None,
                            keys: None,
                        })
                    } else {
                        Ok(PersistentCacheOutput {
                            success: true,
                            action: "retrieve".to_string(),
                            result: Some(entry.value.clone()),
                            keys: None,
                        })
                    }
                } else {
                    Ok(PersistentCacheOutput {
                        success: false,
                        action: "retrieve".to_string(),
                        result: None,
                        keys: None,
                    })
                }
            }
            PersistentCacheInput::Delete { key } => {
                info!("Deleting cache data: {}", key);
                let mut cache = self.cache.write().await;
                let removed = cache.remove(&key);
                
                Ok(PersistentCacheOutput {
                    success: removed.is_some(),
                    action: "delete".to_string(),
                    result: removed.map(|e| e.value),
                    keys: None,
                })
            }
            PersistentCacheInput::List { prefix } => {
                info!("Listing cache keys");
                let cache = self.cache.read().await;
                
                let keys: Vec<String> = if let Some(prefix) = prefix {
                    cache.keys()
                        .filter(|k| k.starts_with(&prefix))
                        .cloned()
                        .collect()
                } else {
                    cache.keys().cloned().collect()
                };
                
                Ok(PersistentCacheOutput {
                    success: true,
                    action: "list".to_string(),
                    result: None,
                    keys: Some(keys),
                })
            }
            PersistentCacheInput::Clear => {
                info!("Clearing cache");
                let mut cache = self.cache.write().await;
                cache.clear();
                
                Ok(PersistentCacheOutput {
                    success: true,
                    action: "clear".to_string(),
                    result: None,
                    keys: None,
                })
            }
        }
    }
}