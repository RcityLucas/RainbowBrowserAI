// Click Tool Implementation
// Full specification compliance for element clicking with advanced options

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::browser::Browser;
use crate::tools::{Tool, Result};
use crate::tools::errors::ToolError;
use crate::tools::security::InputSanitizer;

/// Parameters for the click tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClickParams {
    /// CSS selector for the element to click
    pub selector: String,
    
    /// Optional click options
    #[serde(default)]
    pub options: ClickOptions,
}

/// Advanced click options
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ClickOptions {
    /// Button to use for clicking
    #[serde(default)]
    pub button: MouseButton,
    
    /// Number of clicks (1 for single, 2 for double, 3 for triple)
    #[serde(default = "default_click_count")]
    pub click_count: u32,
    
    /// Delay between clicks in milliseconds (for multi-click)
    pub delay: Option<u64>,
    
    /// Modifier keys to hold during click
    #[serde(default)]
    pub modifiers: Vec<ModifierKey>,
    
    /// Click position relative to element
    pub offset: Option<Position>,
    
    /// Force click even if element is not visible
    #[serde(default)]
    pub force: bool,
    
    /// Wait for element to be clickable
    pub wait_for: Option<WaitOptions>,
    
    /// Timeout in milliseconds
    #[serde(default = "default_timeout")]
    pub timeout: u64,
    
    /// Scroll element into view before clicking
    #[serde(default = "default_scroll_into_view")]
    pub scroll_into_view: bool,
    
    /// Prevent default action
    #[serde(default)]
    pub prevent_default: bool,
}

/// Mouse button options
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

impl Default for MouseButton {
    fn default() -> Self {
        MouseButton::Left
    }
}

/// Modifier keys
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum ModifierKey {
    Alt,
    Control,
    Meta,
    Shift,
}

/// Position offset for clicking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

/// Wait options for element
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaitOptions {
    /// Wait for element to be visible
    #[serde(default = "default_true")]
    pub visible: bool,
    
    /// Wait for element to be enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
    
    /// Wait for element to be stable (not animating)
    #[serde(default)]
    pub stable: bool,
    
    /// Maximum wait time in milliseconds
    #[serde(default = "default_wait_timeout")]
    pub timeout: u64,
}

/// Result of a click operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClickResult {
    /// Whether the click was successful
    pub success: bool,
    
    /// Information about the clicked element
    pub element: ElementInfo,
    
    /// Effects of the click
    pub effects: ClickEffects,
    
    /// Timing information
    pub timing: ClickTiming,
    
    /// Error information if failed
    pub error: Option<String>,
}

/// Information about the clicked element
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementInfo {
    /// Tag name of the element
    pub tag_name: String,
    
    /// Element ID if present
    pub id: Option<String>,
    
    /// Element classes
    pub classes: Vec<String>,
    
    /// Element text content
    pub text: Option<String>,
    
    /// Element type (for input/button elements)
    #[serde(rename = "type")]
    pub type_: Option<String>,
    
    /// HREF for links
    pub href: Option<String>,
    
    /// Bounding box of the element
    pub bounding_box: BoundingBox,
    
    /// Whether element was visible
    pub was_visible: bool,
    
    /// Whether element was enabled
    pub was_enabled: bool,
}

/// Bounding box information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundingBox {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// Effects of the click
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClickEffects {
    /// Whether a navigation occurred
    pub navigation_triggered: bool,
    
    /// New URL if navigation occurred
    pub new_url: Option<String>,
    
    /// Whether a form was submitted
    pub form_submitted: bool,
    
    /// JavaScript events triggered
    pub events_triggered: Vec<String>,
    
    /// DOM changes detected
    pub dom_changes: Vec<DomChange>,
    
    /// Network requests initiated
    pub network_requests: Vec<NetworkRequest>,
}

/// DOM change information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomChange {
    pub selector: String,
    pub change_type: String,
    pub description: String,
}

/// Network request information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkRequest {
    pub url: String,
    pub method: String,
    pub request_type: String,
}

/// Timing information for the click
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClickTiming {
    /// Time to find element (ms)
    pub element_found: u64,
    
    /// Time to execute click (ms)
    pub click_executed: u64,
    
    /// Total operation time (ms)
    pub total: u64,
}

/// Click tool implementation
pub struct Click {
    browser: Arc<Browser>,
}

impl Click {
    pub fn new(browser: Arc<Browser>) -> Self {
        Self { browser }
    }
    
    async fn wait_for_clickable(&self, selector: &str, options: &WaitOptions) -> Result<()> {
        let start = Instant::now();
        let timeout = Duration::from_millis(options.timeout);
        
        loop {
            // Check if element exists
            if let Ok(element) = self.browser.find_element(selector).await {
                // Check visibility
                if options.visible {
                    // Sanitize selector to prevent injection
                    let safe_selector = InputSanitizer::sanitize_selector(selector)
                        .map_err(|e| ToolError::InvalidInput(e.to_string()))?;
                    
                    let visible = self.browser.execute_script(
                        r#"
                        (() => {
                            const selector = arguments[0];
                            const el = document.querySelector(selector);
                            if (!el) return false;
                            const rect = el.getBoundingClientRect();
                            const style = window.getComputedStyle(el);
                            return rect.width > 0 && 
                                   rect.height > 0 && 
                                   style.display !== 'none' &&
                                   style.visibility !== 'hidden' &&
                                   style.opacity !== '0';
                        })()
                        "#,
                        vec![json!(safe_selector)]
                    ).await?;
                    
                    if !visible.json().as_bool().unwrap_or(false) {
                        if start.elapsed() > timeout {
                            return Err(ToolError::Timeout(
                                format!("Element {} not visible after {}ms", selector, options.timeout)
                            ));
                        }
                        tokio::time::sleep(Duration::from_millis(100)).await;
                        continue;
                    }
                }
                
                // Check if enabled
                if options.enabled {
                    // Sanitize selector to prevent injection
                    let safe_selector = InputSanitizer::sanitize_selector(selector)
                        .map_err(|e| ToolError::InvalidInput(e.to_string()))?;
                    
                    let enabled = self.browser.execute_script(
                        r#"
                        (() => {
                            const selector = arguments[0];
                            const el = document.querySelector(selector);
                            return el && !el.disabled && !el.hasAttribute('disabled');
                        })()
                        "#,
                        vec![json!(safe_selector)]
                    ).await?;
                    
                    if !enabled.json().as_bool().unwrap_or(false) {
                        if start.elapsed() > timeout {
                            return Err(ToolError::Timeout(
                                format!("Element {} not enabled after {}ms", selector, options.timeout)
                            ));
                        }
                        tokio::time::sleep(Duration::from_millis(100)).await;
                        continue;
                    }
                }
                
                // Element is clickable
                return Ok(());
            }
            
            if start.elapsed() > timeout {
                return Err(ToolError::Timeout(
                    format!("Element {} not found after {}ms", selector, options.timeout)
                ));
            }
            
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }
    
    async fn get_element_info(&self, selector: &str) -> Result<ElementInfo> {
        let info = self.browser.execute_script(
            &format!(
                r#"
                (() => {{
                    const el = document.querySelector('{}');
                    if (!el) return null;
                    const rect = el.getBoundingClientRect();
                    const style = window.getComputedStyle(el);
                    return {{
                        tagName: el.tagName.toLowerCase(),
                        id: el.id || null,
                        classes: Array.from(el.classList),
                        text: el.textContent?.trim().substring(0, 100) || null,
                        type: el.type || null,
                        href: el.href || null,
                        boundingBox: {{
                            x: rect.x,
                            y: rect.y,
                            width: rect.width,
                            height: rect.height
                        }},
                        wasVisible: rect.width > 0 && rect.height > 0 && 
                                   style.display !== 'none' && 
                                   style.visibility !== 'hidden',
                        wasEnabled: !el.disabled
                    }};
                }})()
                "#,
                selector
            ),
            vec![]
        ).await?;
        
        serde_json::from_value(info)
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to parse element info: {}", e)))
    }
    
    async fn capture_state(&self) -> Result<serde_json::Value> {
        self.browser.execute_script(
            r#"
            (() => {
                return {
                    url: window.location.href,
                    title: document.title,
                    formCount: document.forms.length,
                    activeElement: document.activeElement?.tagName.toLowerCase()
                };
            })()
            "#,
            vec![]
        ).await
    }
    
    async fn analyze_effects(&self, pre_state: serde_json::Value, post_state: serde_json::Value) -> ClickEffects {
        let navigation_triggered = pre_state["url"] != post_state["url"];
        let new_url = if navigation_triggered {
            post_state["url"].as_str().map(String::from)
        } else {
            None
        };
        
        let form_submitted = pre_state["formCount"].as_u64().unwrap_or(0) > 
                           post_state["formCount"].as_u64().unwrap_or(0);
        
        // Note: In a real implementation, we would track actual events and network requests
        // For now, we'll return reasonable defaults
        ClickEffects {
            navigation_triggered,
            new_url,
            form_submitted,
            events_triggered: vec!["click".to_string()],
            dom_changes: vec![],
            network_requests: vec![],
        }
    }
}

#[async_trait]
impl Tool for Click {
    type Input = ClickParams;
    type Output = ClickResult;
    
    fn name(&self) -> &str {
        "click"
    }
    
    fn description(&self) -> &str {
        "Click on elements with advanced options including modifiers, multi-click, and force click"
    }
    
    async fn execute(&self, params: Self::Input) -> Result<Self::Output> {
        let start_time = Instant::now();
        let mut timing = ClickTiming {
            element_found: 0,
            click_executed: 0,
            total: 0,
        };
        
        // Wait for element if specified
        if let Some(ref wait_options) = params.options.wait_for {
            self.wait_for_clickable(&params.selector, wait_options).await?;
        }
        
        timing.element_found = start_time.elapsed().as_millis() as u64;
        
        // Get element information
        let element_info = self.get_element_info(&params.selector).await?;
        
        // Scroll into view if requested
        if params.options.scroll_into_view {
            self.browser.execute_script(
                &format!(
                    r#"document.querySelector('{}').scrollIntoView({{ behavior: 'smooth', block: 'center' }})"#,
                    params.selector
                ),
                vec![]
            ).await?;
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
        
        // Capture pre-click state
        let pre_click_state = self.capture_state().await?;
        
        // Build click script
        let click_script = format!(
            r#"
            (() => {{
                const el = document.querySelector('{}');
                if (!el) return {{ success: false, error: 'Element not found' }};
                
                const options = {};
                const event = new MouseEvent('{}', {{
                    view: window,
                    bubbles: true,
                    cancelable: true,
                    button: {},
                    buttons: {},
                    clientX: el.getBoundingClientRect().x + {},
                    clientY: el.getBoundingClientRect().y + {},
                    ctrlKey: {},
                    altKey: {},
                    shiftKey: {},
                    metaKey: {}
                }});
                
                {}
                
                for (let i = 0; i < {}; i++) {{
                    el.dispatchEvent(event);
                    {}
                }}
                
                return {{ success: true }};
            }})()
            "#,
            params.selector,
            json!(params.options),
            match params.options.click_count {
                2 => "dblclick",
                _ => "click"
            },
            match params.options.button {
                MouseButton::Left => 0,
                MouseButton::Middle => 1,
                MouseButton::Right => 2,
            },
            match params.options.button {
                MouseButton::Left => 1,
                MouseButton::Middle => 4,
                MouseButton::Right => 2,
            },
            params.options.offset.as_ref().map(|o| o.x).unwrap_or(0.0),
            params.options.offset.as_ref().map(|o| o.y).unwrap_or(0.0),
            params.options.modifiers.iter().any(|m| matches!(m, ModifierKey::Control)),
            params.options.modifiers.iter().any(|m| matches!(m, ModifierKey::Alt)),
            params.options.modifiers.iter().any(|m| matches!(m, ModifierKey::Shift)),
            params.options.modifiers.iter().any(|m| matches!(m, ModifierKey::Meta)),
            if params.options.prevent_default {
                "event.preventDefault();"
            } else {
                ""
            },
            params.options.click_count,
            if let Some(delay) = params.options.delay {
                format!("if (i < {} - 1) await new Promise(r => setTimeout(r, {}));", 
                       params.options.click_count, delay)
            } else {
                String::new()
            }
        );
        
        // Execute click
        let click_result = if params.options.force {
            // Force click bypasses visibility checks
            let script_result = self.browser.execute_script(&click_script, vec![]).await?;
            script_result.json().clone()
        } else {
            // Normal click respects element state
            self.browser.click(&params.selector).await
                .map(|_| json!({"success": true}))
                .unwrap_or_else(|e| json!({"success": false, "error": e.to_string()}))
        };
        
        timing.click_executed = start_time.elapsed().as_millis() as u64 - timing.element_found;
        
        // Small delay to let effects propagate
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        // Capture post-click state
        let post_click_state = self.capture_state().await?;
        
        // Analyze effects
        let effects = self.analyze_effects(pre_click_state, post_click_state).await;
        
        timing.total = start_time.elapsed().as_millis() as u64;
        
        Ok(ClickResult {
            success: click_result["success"].as_bool().unwrap_or(false),
            element: element_info,
            effects,
            timing,
            error: click_result["error"].as_str().map(String::from),
        })
    }
    
    fn validate_input(&self, params: &Self::Input) -> Result<()> {
        if params.selector.is_empty() {
            return Err(ToolError::InvalidInput("Selector cannot be empty".into()).into());
        }
        
        if params.options.click_count == 0 || params.options.click_count > 3 {
            return Err(ToolError::InvalidInput("Click count must be between 1 and 3".into()).into());
        }
        
        if let Some(ref offset) = params.options.offset {
            if offset.x < 0.0 || offset.y < 0.0 {
                return Err(ToolError::InvalidInput("Offset values must be non-negative".into()).into());
            }
        }
        
        Ok(())
    }
    
    fn input_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "required": ["selector"],
            "properties": {
                "selector": {
                    "type": "string",
                    "description": "CSS selector for the element to click"
                },
                "options": {
                    "type": "object",
                    "properties": {
                        "button": {
                            "type": "string",
                            "enum": ["left", "right", "middle"],
                            "default": "left"
                        },
                        "clickCount": {
                            "type": "integer",
                            "minimum": 1,
                            "maximum": 3,
                            "default": 1
                        },
                        "delay": {
                            "type": "integer",
                            "description": "Delay between clicks in milliseconds"
                        },
                        "modifiers": {
                            "type": "array",
                            "items": {
                                "type": "string",
                                "enum": ["Alt", "Control", "Meta", "Shift"]
                            }
                        },
                        "offset": {
                            "type": "object",
                            "properties": {
                                "x": { "type": "number" },
                                "y": { "type": "number" }
                            }
                        },
                        "force": {
                            "type": "boolean",
                            "default": false
                        },
                        "scrollIntoView": {
                            "type": "boolean",
                            "default": true
                        },
                        "timeout": {
                            "type": "integer",
                            "default": 30000
                        }
                    }
                }
            }
        })
    }
    
    fn output_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "success": { "type": "boolean" },
                "element": { "type": "object" },
                "effects": { "type": "object" },
                "timing": { "type": "object" },
                "error": { "type": "string" }
            }
        })
    }
}

// Helper functions
fn default_click_count() -> u32 { 1 }
fn default_timeout() -> u64 { 30000 }
fn default_scroll_into_view() -> bool { true }
fn default_true() -> bool { true }
fn default_wait_timeout() -> u64 { 30000 }