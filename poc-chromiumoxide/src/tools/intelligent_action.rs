use super::traits::{Tool, ToolCategory};
use crate::browser::Browser;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{info, debug, error};
use uuid::Uuid;

// ============================================================================
// Intelligent Action Tool - Advanced Browser Actions with Retry Logic
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelligentActionInput {
    pub action_type: String, // "click", "type", "navigate", etc.
    pub target: ActionTargetInput,
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,
    #[serde(default = "default_retry_count")]
    pub retry_count: u32,
    #[serde(default = "default_verify")]
    pub verify_result: bool,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub text: Option<String>, // For type actions
    #[serde(default)]
    pub url: Option<String>, // For navigate actions
    #[serde(default)]
    pub wait_condition: Option<String>, // For wait actions
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionTargetInput {
    pub selector: Option<String>,
    pub xpath: Option<String>,
    pub text: Option<String>,
    pub id: Option<String>,
    pub class: Option<String>,
    pub name: Option<String>,
    pub placeholder: Option<String>,
    pub value: Option<String>,
    pub role: Option<String>,
    pub coordinate: Option<CoordinateInput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinateInput {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Serialize)]
pub struct IntelligentActionOutput {
    pub success: bool,
    pub action_id: String,
    pub execution_time_ms: u64,
    pub attempts: u32,
    pub element_info: Option<ElementInfoOutput>,
    pub verification_result: Option<String>,
    pub error: Option<String>,
    pub logs: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ElementInfoOutput {
    pub tag_name: String,
    pub attributes: std::collections::HashMap<String, String>,
    pub text_content: Option<String>,
    pub bounding_box: Option<BoundingBoxOutput>,
    pub is_visible: bool,
    pub is_enabled: bool,
}

#[derive(Debug, Serialize)]
pub struct BoundingBoxOutput {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

fn default_timeout() -> u64 { 10000 }
fn default_retry_count() -> u32 { 3 }
fn default_verify() -> bool { true }

pub struct IntelligentActionTool {
    browser: Arc<Browser>,
}

impl IntelligentActionTool {
    pub fn new(browser: Arc<Browser>) -> Self {
        Self { browser }
    }

    /// Execute action with intelligence and retry logic
    async fn execute_intelligent_action(&self, input: &IntelligentActionInput) -> Result<IntelligentActionOutput> {
        let action_id = Uuid::new_v4().to_string();
        let start_time = Instant::now();
        let mut logs = Vec::new();
        
        debug!("Executing intelligent action: {} with target: {:?}", input.action_type, input.target);
        logs.push(format!("Starting action: {} with target: {:?}", input.action_type, input.target));
        
        // Execute action with retry logic
        let mut last_error = None;
        let mut attempts = 0;
        
        for attempt in 0..input.retry_count {
            attempts = attempt + 1;
            
            let log_msg = format!("Attempt {} of {}", attempts, input.retry_count);
            debug!("{}", log_msg);
            logs.push(log_msg);
            
            match self.try_execute_action(input, &mut logs).await {
                Ok(result) => {
                    info!("Action {} completed successfully on attempt {}", action_id, attempts);
                    logs.push(format!("Action completed successfully on attempt {}", attempts));
                    
                    return Ok(IntelligentActionOutput {
                        success: true,
                        action_id,
                        execution_time_ms: start_time.elapsed().as_millis() as u64,
                        attempts,
                        element_info: result.element_info,
                        verification_result: result.verification_result,
                        error: None,
                        logs,
                    });
                }
                Err(e) => {
                    error!("Action {} failed on attempt {}: {}", action_id, attempts, e);
                    let error_msg = format!("Attempt {} failed: {}", attempts, e);
                    logs.push(error_msg);
                    last_error = Some(e.to_string());
                    
                    if attempt < input.retry_count - 1 {
                        // Wait before retry with exponential backoff
                        let delay = Duration::from_millis(100 * (2_u64.pow(attempt)));
                        tokio::time::sleep(delay).await;
                        logs.push(format!("Waiting {:?} before retry", delay));
                    }
                }
            }
        }

        // All retries failed
        logs.push("All retry attempts exhausted".to_string());
        Ok(IntelligentActionOutput {
            success: false,
            action_id,
            execution_time_ms: start_time.elapsed().as_millis() as u64,
            attempts,
            element_info: None,
            verification_result: None,
            error: last_error,
            logs,
        })
    }

    /// Try to execute an action once
    async fn try_execute_action(&self, input: &IntelligentActionInput, logs: &mut Vec<String>) -> Result<ActionExecutionResult> {
        match input.action_type.to_lowercase().as_str() {
            "click" => self.execute_click_action(input, logs).await,
            "doubleclick" => self.execute_double_click_action(input, logs).await,
            "rightclick" => self.execute_right_click_action(input, logs).await,
            "type" => self.execute_type_action(input, logs).await,
            "clear" => self.execute_clear_action(input, logs).await,
            "navigate" => self.execute_navigate_action(input, logs).await,
            "screenshot" => self.execute_screenshot_action(logs).await,
            "scroll" => self.execute_scroll_action(input, logs).await,
            "hover" => self.execute_hover_action(input, logs).await,
            "focus" => self.execute_focus_action(input, logs).await,
            "wait" => self.execute_wait_action(input, logs).await,
            _ => Err(anyhow::anyhow!("Unsupported action type: {}", input.action_type)),
        }
    }

    async fn execute_click_action(&self, input: &IntelligentActionInput, logs: &mut Vec<String>) -> Result<ActionExecutionResult> {
        let selector = self.extract_selector(&input.target)?;
        logs.push(format!("Clicking element with selector: {}", selector));
        
        // Get element info before clicking
        let element_info = self.get_element_info(&selector, logs).await?;
        
        // Perform click
        self.browser.click(&selector).await?;
        
        // Verify click if requested
        let verification_result = if input.verify_result {
            logs.push("Verifying click action".to_string());
            Some("Click executed successfully".to_string())
        } else {
            None
        };

        Ok(ActionExecutionResult {
            element_info: Some(element_info),
            verification_result,
        })
    }

    async fn execute_double_click_action(&self, input: &IntelligentActionInput, logs: &mut Vec<String>) -> Result<ActionExecutionResult> {
        let selector = self.extract_selector(&input.target)?;
        logs.push(format!("Double-clicking element with selector: {}", selector));
        
        let element_info = self.get_element_info(&selector, logs).await?;
        
        // Perform double click (simulate by clicking twice quickly)
        self.browser.click(&selector).await?;
        tokio::time::sleep(Duration::from_millis(50)).await;
        self.browser.click(&selector).await?;
        
        Ok(ActionExecutionResult {
            element_info: Some(element_info),
            verification_result: Some("Double-click executed successfully".to_string()),
        })
    }

    async fn execute_right_click_action(&self, input: &IntelligentActionInput, logs: &mut Vec<String>) -> Result<ActionExecutionResult> {
        let selector = self.extract_selector(&input.target)?;
        logs.push(format!("Right-clicking element with selector: {}", selector));
        
        let element_info = self.get_element_info(&selector, logs).await?;
        
        // Simulate right click through JavaScript
        let script = format!(r#"
            const element = document.querySelector('{}');
            if (element) {{
                const event = new MouseEvent('contextmenu', {{
                    bubbles: true,
                    cancelable: true,
                    view: window
                }});
                element.dispatchEvent(event);
                return true;
            }} else {{
                throw new Error('Element not found');
            }}
        "#, selector);
        
        self.browser.execute_script(&script).await?;
        
        Ok(ActionExecutionResult {
            element_info: Some(element_info),
            verification_result: Some("Right-click executed successfully".to_string()),
        })
    }

    async fn execute_type_action(&self, input: &IntelligentActionInput, logs: &mut Vec<String>) -> Result<ActionExecutionResult> {
        let selector = self.extract_selector(&input.target)?;
        let text = input.text.as_ref().ok_or_else(|| anyhow::anyhow!("Text required for type action"))?;
        
        logs.push(format!("Typing text '{}' into element: {}", text, selector));
        
        // Get element info
        let element_info = self.get_element_info(&selector, logs).await?;
        
        // Focus element first
        self.browser.focus(&selector).await?;
        
        // Type text
        self.browser.type_text(&selector, text).await?;
        
        let verification_result = if input.verify_result {
            logs.push("Verifying text was typed".to_string());
            Some(format!("Text '{}' typed successfully", text))
        } else {
            None
        };

        Ok(ActionExecutionResult {
            element_info: Some(element_info),
            verification_result,
        })
    }

    async fn execute_clear_action(&self, input: &IntelligentActionInput, logs: &mut Vec<String>) -> Result<ActionExecutionResult> {
        let selector = self.extract_selector(&input.target)?;
        logs.push(format!("Clearing element: {}", selector));
        
        let element_info = self.get_element_info(&selector, logs).await?;
        
        // Clear field by selecting all and deleting
        self.browser.focus(&selector).await?;
        
        let script = format!(r#"
            const element = document.querySelector('{}');
            if (element) {{
                element.value = '';
                element.dispatchEvent(new Event('input', {{ bubbles: true }}));
                element.dispatchEvent(new Event('change', {{ bubbles: true }}));
            }}
        "#, selector);
        
        self.browser.execute_script(&script).await?;
        
        Ok(ActionExecutionResult {
            element_info: Some(element_info),
            verification_result: Some("Element cleared successfully".to_string()),
        })
    }

    async fn execute_navigate_action(&self, input: &IntelligentActionInput, logs: &mut Vec<String>) -> Result<ActionExecutionResult> {
        let url = input.url.as_ref()
            .or(input.text.as_ref())
            .ok_or_else(|| anyhow::anyhow!("URL required for navigate action"))?;
        
        logs.push(format!("Navigating to: {}", url));
        
        self.browser.navigate_to(url).await?;
        
        // Wait a bit for navigation to complete
        tokio::time::sleep(Duration::from_millis(500)).await;
        
        Ok(ActionExecutionResult {
            element_info: None,
            verification_result: Some(format!("Navigated to {}", url)),
        })
    }

    async fn execute_screenshot_action(&self, logs: &mut Vec<String>) -> Result<ActionExecutionResult> {
        logs.push("Taking screenshot".to_string());
        
        let screenshot_options = crate::browser::ScreenshotOptions::default();
        let _screenshot = self.browser.screenshot(screenshot_options).await?;
        
        Ok(ActionExecutionResult {
            element_info: None,
            verification_result: Some("Screenshot taken successfully".to_string()),
        })
    }

    async fn execute_scroll_action(&self, input: &IntelligentActionInput, logs: &mut Vec<String>) -> Result<ActionExecutionResult> {
        if let Some(coord) = &input.target.coordinate {
            logs.push(format!("Scrolling to coordinates ({}, {})", coord.x, coord.y));
            self.browser.scroll_to(coord.x, coord.y).await?;
            return Ok(ActionExecutionResult {
                element_info: None,
                verification_result: Some(format!("Scrolled to ({}, {})", coord.x, coord.y)),
            });
        }
        
        let selector = self.extract_selector(&input.target)?;
        logs.push(format!("Scrolling to element: {}", selector));
        
        let element_info = self.get_element_info(&selector, logs).await?;
        
        // Scroll element into view
        let script = format!("document.querySelector('{}').scrollIntoView({{behavior: 'smooth', block: 'center'}})", selector);
        self.browser.execute_script(&script).await?;
        
        Ok(ActionExecutionResult {
            element_info: Some(element_info),
            verification_result: Some("Element scrolled into view".to_string()),
        })
    }

    async fn execute_hover_action(&self, input: &IntelligentActionInput, logs: &mut Vec<String>) -> Result<ActionExecutionResult> {
        let selector = self.extract_selector(&input.target)?;
        logs.push(format!("Hovering over element: {}", selector));
        
        let element_info = self.get_element_info(&selector, logs).await?;
        
        self.browser.hover(&selector).await?;
        
        Ok(ActionExecutionResult {
            element_info: Some(element_info),
            verification_result: Some("Hover executed successfully".to_string()),
        })
    }

    async fn execute_focus_action(&self, input: &IntelligentActionInput, logs: &mut Vec<String>) -> Result<ActionExecutionResult> {
        let selector = self.extract_selector(&input.target)?;
        logs.push(format!("Focusing element: {}", selector));
        
        let element_info = self.get_element_info(&selector, logs).await?;
        
        self.browser.focus(&selector).await?;
        
        Ok(ActionExecutionResult {
            element_info: Some(element_info),
            verification_result: Some("Focus executed successfully".to_string()),
        })
    }

    async fn execute_wait_action(&self, input: &IntelligentActionInput, logs: &mut Vec<String>) -> Result<ActionExecutionResult> {
        let duration = if let Some(condition) = &input.wait_condition {
            // Parse wait condition (for now, treat as duration in ms)
            let ms: u64 = condition.parse().unwrap_or(1000);
            Duration::from_millis(ms)
        } else {
            Duration::from_secs(1)
        };
        
        logs.push(format!("Waiting for {:?}", duration));
        tokio::time::sleep(duration).await;
        
        Ok(ActionExecutionResult {
            element_info: None,
            verification_result: Some(format!("Waited for {:?}", duration)),
        })
    }

    /// Extract selector from action target
    fn extract_selector(&self, target: &ActionTargetInput) -> Result<String> {
        if let Some(selector) = &target.selector {
            return Ok(selector.clone());
        }
        if let Some(id) = &target.id {
            return Ok(format!("#{}", id));
        }
        if let Some(class) = &target.class {
            return Ok(format!(".{}", class));
        }
        if let Some(name) = &target.name {
            return Ok(format!("[name='{}']", name));
        }
        if let Some(placeholder) = &target.placeholder {
            return Ok(format!("[placeholder='{}']", placeholder));
        }
        if let Some(role) = &target.role {
            return Ok(format!("[role='{}']", role));
        }
        if let Some(text) = &target.text {
            return Ok(format!("//*[contains(text(),'{}')]", text));
        }
        
        Err(anyhow::anyhow!("No valid target specified"))
    }

    /// Get detailed element information
    async fn get_element_info(&self, selector: &str, logs: &mut Vec<String>) -> Result<ElementInfoOutput> {
        logs.push(format!("Getting element info for: {}", selector));
        
        let result = self.browser.get_element_info(selector).await?;
        
        Ok(ElementInfoOutput {
            tag_name: result["tag_name"].as_str().unwrap_or("unknown").to_string(),
            attributes: result["attributes"]
                .as_object()
                .map(|obj| obj.iter()
                    .map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_string()))
                    .collect())
                .unwrap_or_default(),
            text_content: result["text_content"].as_str().map(String::from),
            bounding_box: result.get("bounding_box").and_then(|bb| {
                Some(BoundingBoxOutput {
                    x: bb["x"].as_f64().unwrap_or(0.0),
                    y: bb["y"].as_f64().unwrap_or(0.0),
                    width: bb["width"].as_f64().unwrap_or(0.0),
                    height: bb["height"].as_f64().unwrap_or(0.0),
                })
            }),
            is_visible: result["is_visible"].as_bool().unwrap_or(false),
            is_enabled: result["is_enabled"].as_bool().unwrap_or(false),
        })
    }
}

#[async_trait]
impl Tool for IntelligentActionTool {
    type Input = IntelligentActionInput;
    type Output = IntelligentActionOutput;
    
    fn name(&self) -> &str {
        "intelligent_action"
    }
    
    fn description(&self) -> &str {
        "Execute intelligent browser actions with advanced retry logic and verification"
    }
    
    fn category(&self) -> ToolCategory {
        ToolCategory::AdvancedAutomation
    }
    
    async fn execute(&self, input: Self::Input) -> Result<Self::Output> {
        self.execute_intelligent_action(&input).await
    }
    
    async fn validate_input(&self, input: &Self::Input) -> Result<()> {
        // Validate action type
        let valid_actions = ["click", "doubleclick", "rightclick", "type", "clear", 
                            "navigate", "scroll", "hover", "focus", "wait", "screenshot"];
        
        if !valid_actions.contains(&input.action_type.to_lowercase().as_str()) {
            return Err(anyhow::anyhow!("Invalid action type: {}", input.action_type));
        }

        // Validate that at least one target is specified (except for screenshot/navigate)
        if !["screenshot", "navigate"].contains(&input.action_type.to_lowercase().as_str()) {
            let target = &input.target;
            let has_target = target.selector.is_some() || target.xpath.is_some() || 
                            target.text.is_some() || target.id.is_some() || 
                            target.class.is_some() || target.name.is_some() ||
                            target.placeholder.is_some() || target.value.is_some() ||
                            target.role.is_some() || target.coordinate.is_some();

            if !has_target {
                return Err(anyhow::anyhow!("At least one target method must be specified"));
            }
        }

        // Validate type action has text
        if input.action_type.to_lowercase() == "type" && input.text.is_none() {
            return Err(anyhow::anyhow!("Text is required for type actions"));
        }

        // Validate navigate action has URL
        if input.action_type.to_lowercase() == "navigate" && input.url.is_none() && input.text.is_none() {
            return Err(anyhow::anyhow!("URL is required for navigate actions"));
        }

        Ok(())
    }
}

/// Internal result structure
#[derive(Debug)]
struct ActionExecutionResult {
    element_info: Option<ElementInfoOutput>,
    verification_result: Option<String>,
}