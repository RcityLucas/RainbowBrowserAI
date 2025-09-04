use super::traits::{Tool, ToolCategory};
use crate::browser::{Browser, BrowserOps};
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use std::sync::Arc;
use tracing::{info, debug, warn};

// ============================================================================
// Click Tool
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct ClickInput {
    pub selector: String,
    #[serde(default)]
    pub wait_for_element: bool,
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,
    #[serde(default)]
    pub offset_x: Option<i32>,
    #[serde(default)]
    pub offset_y: Option<i32>,
}

fn default_timeout() -> u64 {
    5000
}

#[derive(Debug, Serialize)]
pub struct ClickOutput {
    pub success: bool,
    pub element_found: bool,
    pub click_position: Option<ClickPosition>,
}

#[derive(Debug, Serialize)]
pub struct ClickPosition {
    pub x: f64,
    pub y: f64,
}

pub struct ClickTool {
    browser: Arc<Browser>,
}

impl ClickTool {
    pub fn new(browser: Arc<Browser>) -> Self {
        Self { browser }
    }
}

#[async_trait]
impl Tool for ClickTool {
    type Input = ClickInput;
    type Output = ClickOutput;
    
    fn name(&self) -> &str {
        "click"
    }
    
    fn description(&self) -> &str {
        "Click on an element specified by selector"
    }
    
    fn category(&self) -> ToolCategory {
        ToolCategory::Interaction
    }
    
    async fn execute(&self, input: Self::Input) -> Result<Self::Output> {
        info!("Clicking element: {}", input.selector);
        
        // Wait for element if requested
        if input.wait_for_element {
            let timeout = std::time::Duration::from_millis(input.timeout_ms);
            self.browser.wait_for_selector(&input.selector, timeout).await?;
        }
        
        // Get element position before clicking (optional)
        let click_position = match self.browser.find_element(&input.selector).await {
            Ok(element_info) => {
                element_info.rect.map(|rect| ClickPosition {
                    x: rect.x + rect.width / 2.0,
                    y: rect.y + rect.height / 2.0,
                })
            }
            Err(_) => None,
        };
        
        // Perform the click
        match self.browser.click(&input.selector).await {
            Ok(_) => {
                Ok(ClickOutput {
                    success: true,
                    element_found: true,
                    click_position,
                })
            }
            Err(e) => {
                warn!("Click failed: {}", e);
                Err(e)
            }
        }
    }
    
    async fn validate_input(&self, input: &Self::Input) -> Result<()> {
        if input.selector.is_empty() {
            return Err(anyhow!("Selector cannot be empty"));
        }
        Ok(())
    }
}

// ============================================================================
// Type Text Tool
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct TypeTextInput {
    pub selector: String,
    pub text: String,
    #[serde(default)]
    pub clear_first: bool,
    #[serde(default)]
    pub delay_ms: Option<u64>,
    #[serde(default)]
    pub wait_for_element: bool,
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,
}

#[derive(Debug, Serialize)]
pub struct TypeTextOutput {
    pub success: bool,
    pub characters_typed: usize,
    pub final_value: Option<String>,
}

pub struct TypeTextTool {
    browser: Arc<Browser>,
}

impl TypeTextTool {
    pub fn new(browser: Arc<Browser>) -> Self {
        Self { browser }
    }
}

#[async_trait]
impl Tool for TypeTextTool {
    type Input = TypeTextInput;
    type Output = TypeTextOutput;
    
    fn name(&self) -> &str {
        "type_text"
    }
    
    fn description(&self) -> &str {
        "Type text into an input field"
    }
    
    fn category(&self) -> ToolCategory {
        ToolCategory::Interaction
    }
    
    async fn execute(&self, input: Self::Input) -> Result<Self::Output> {
        info!("Typing text into: {}", input.selector);
        
        // Wait for element if requested
        if input.wait_for_element {
            let timeout = std::time::Duration::from_millis(input.timeout_ms);
            self.browser.wait_for_selector(&input.selector, timeout).await?;
        }
        
        // Clear field first if requested
        if input.clear_first {
            debug!("Clearing field first");
            let clear_script = format!(
                "document.querySelector('{}').value = ''",
                input.selector
            );
            self.browser.execute_script(&clear_script).await?;
        }
        
        // Type the text
        self.browser.type_text(&input.selector, &input.text).await?;
        
        // Add delay if specified
        if let Some(delay) = input.delay_ms {
            tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
        }
        
        // Get the final value
        let value_script = format!(
            "document.querySelector('{}').value",
            input.selector
        );
        let final_value = self.browser.execute_script(&value_script).await
            .ok()
            .and_then(|v| v.as_str().map(|s| s.to_string()));
        
        Ok(TypeTextOutput {
            success: true,
            characters_typed: input.text.len(),
            final_value,
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
// Select Option Tool
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct SelectOptionInput {
    pub selector: String,
    #[serde(flatten)]
    pub option: SelectOption,
    #[serde(default)]
    pub wait_for_element: bool,
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SelectOption {
    Value(String),
    Text(String),
    Index(usize),
}

#[derive(Debug, Serialize)]
pub struct SelectOptionOutput {
    pub success: bool,
    pub selected_value: Option<String>,
    pub selected_text: Option<String>,
    pub selected_index: Option<usize>,
}

pub struct SelectOptionTool {
    browser: Arc<Browser>,
}

impl SelectOptionTool {
    pub fn new(browser: Arc<Browser>) -> Self {
        Self { browser }
    }
}

#[async_trait]
impl Tool for SelectOptionTool {
    type Input = SelectOptionInput;
    type Output = SelectOptionOutput;
    
    fn name(&self) -> &str {
        "select_option"
    }
    
    fn description(&self) -> &str {
        "Select an option from a dropdown/select element"
    }
    
    fn category(&self) -> ToolCategory {
        ToolCategory::Interaction
    }
    
    async fn execute(&self, input: Self::Input) -> Result<Self::Output> {
        info!("Selecting option in: {}", input.selector);
        
        // Wait for element if requested
        if input.wait_for_element {
            let timeout = std::time::Duration::from_millis(input.timeout_ms);
            self.browser.wait_for_selector(&input.selector, timeout).await?;
        }
        
        // Select based on option type
        let script = match &input.option {
            SelectOption::Value(value) => {
                format!(
                    r#"
                    (function() {{
                        const select = document.querySelector('{}');
                        select.value = '{}';
                        select.dispatchEvent(new Event('change'));
                        return {{
                            value: select.value,
                            text: select.options[select.selectedIndex]?.text,
                            index: select.selectedIndex
                        }};
                    }})()"#,
                    input.selector, value
                )
            }
            SelectOption::Text(text) => {
                format!(
                    r#"
                    (function() {{
                        const select = document.querySelector('{}');
                        const option = Array.from(select.options).find(o => o.text === '{}');
                        if (option) {{
                            select.selectedIndex = option.index;
                            select.dispatchEvent(new Event('change'));
                        }}
                        return {{
                            value: select.value,
                            text: select.options[select.selectedIndex]?.text,
                            index: select.selectedIndex
                        }};
                    }})()"#,
                    input.selector, text
                )
            }
            SelectOption::Index(index) => {
                format!(
                    r#"
                    (function() {{
                        const select = document.querySelector('{}');
                        select.selectedIndex = {};
                        select.dispatchEvent(new Event('change'));
                        return {{
                            value: select.value,
                            text: select.options[select.selectedIndex]?.text,
                            index: select.selectedIndex
                        }};
                    }})()"#,
                    input.selector, index
                )
            }
        };
        
        let result = self.browser.execute_script(&script).await?;
        
        Ok(SelectOptionOutput {
            success: true,
            selected_value: result["value"].as_str().map(|s| s.to_string()),
            selected_text: result["text"].as_str().map(|s| s.to_string()),
            selected_index: result["index"].as_u64().map(|i| i as usize),
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
// Hover Tool
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct HoverInput {
    pub selector: String,
    #[serde(default = "default_hover_duration")]
    pub duration_ms: u64,
    #[serde(default)]
    pub wait_for_element: bool,
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,
}

fn default_hover_duration() -> u64 {
    1000
}

#[derive(Debug, Serialize)]
pub struct HoverOutput {
    pub success: bool,
    pub element_found: bool,
    pub hover_position: Option<HoverPosition>,
}

#[derive(Debug, Serialize)]
pub struct HoverPosition {
    pub x: f64,
    pub y: f64,
}

pub struct HoverTool {
    browser: Arc<Browser>,
}

impl HoverTool {
    pub fn new(browser: Arc<Browser>) -> Self {
        Self { browser }
    }
}

#[async_trait]
impl Tool for HoverTool {
    type Input = HoverInput;
    type Output = HoverOutput;
    
    fn name(&self) -> &str {
        "hover"
    }
    
    fn description(&self) -> &str {
        "Hover over an element"
    }
    
    fn category(&self) -> ToolCategory {
        ToolCategory::Interaction
    }
    
    async fn execute(&self, input: Self::Input) -> Result<Self::Output> {
        info!("Hovering over element: {}", input.selector);
        
        // Wait for element if requested
        if input.wait_for_element {
            let timeout = std::time::Duration::from_millis(input.timeout_ms);
            self.browser.wait_for_selector(&input.selector, timeout).await?;
        }
        
        // Get element position
        let element_info = self.browser.find_element(&input.selector).await?;
        let hover_position = element_info.rect.map(|rect| HoverPosition {
            x: rect.x + rect.width / 2.0,
            y: rect.y + rect.height / 2.0,
        });
        
        // Perform hover using JavaScript
        let script = format!(
            r#"
            (function() {{
                const element = document.querySelector('{}');
                if (element) {{
                    const event = new MouseEvent('mouseover', {{
                        view: window,
                        bubbles: true,
                        cancelable: true
                    }});
                    element.dispatchEvent(event);
                    
                    const enterEvent = new MouseEvent('mouseenter', {{
                        view: window,
                        bubbles: false,
                        cancelable: false
                    }});
                    element.dispatchEvent(enterEvent);
                    return true;
                }}
                return false;
            }})()"#,
            input.selector
        );
        
        let element_found = self.browser.execute_script(&script).await?
            .as_bool()
            .unwrap_or(false);
        
        // Hold the hover for specified duration
        tokio::time::sleep(std::time::Duration::from_millis(input.duration_ms)).await;
        
        // Trigger mouseout event
        let mouseout_script = format!(
            r#"
            (function() {{
                const element = document.querySelector('{}');
                if (element) {{
                    const event = new MouseEvent('mouseout', {{
                        view: window,
                        bubbles: true,
                        cancelable: true
                    }});
                    element.dispatchEvent(event);
                    
                    const leaveEvent = new MouseEvent('mouseleave', {{
                        view: window,
                        bubbles: false,
                        cancelable: false
                    }});
                    element.dispatchEvent(leaveEvent);
                }}
            }})()"#,
            input.selector
        );
        
        self.browser.execute_script(&mouseout_script).await?;
        
        Ok(HoverOutput {
            success: true,
            element_found,
            hover_position,
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
// Focus Tool
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct FocusInput {
    pub selector: String,
    #[serde(default)]
    pub wait_for_element: bool,
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,
}

#[derive(Debug, Serialize)]
pub struct FocusOutput {
    pub success: bool,
    pub element_focused: bool,
    pub previous_active_element: Option<String>,
}

pub struct FocusTool {
    browser: Arc<Browser>,
}

impl FocusTool {
    pub fn new(browser: Arc<Browser>) -> Self {
        Self { browser }
    }
}

#[async_trait]
impl Tool for FocusTool {
    type Input = FocusInput;
    type Output = FocusOutput;
    
    fn name(&self) -> &str {
        "focus"
    }
    
    fn description(&self) -> &str {
        "Focus on an element"
    }
    
    fn category(&self) -> ToolCategory {
        ToolCategory::Interaction
    }
    
    async fn execute(&self, input: Self::Input) -> Result<Self::Output> {
        info!("Focusing element: {}", input.selector);
        
        // Wait for element if requested
        if input.wait_for_element {
            let timeout = std::time::Duration::from_millis(input.timeout_ms);
            self.browser.wait_for_selector(&input.selector, timeout).await?;
        }
        
        // Get current active element
        let previous_script = r#"
            document.activeElement ? 
            (document.activeElement.id || document.activeElement.tagName) : null
        "#;
        let previous_active = self.browser.execute_script(previous_script).await
            .ok()
            .and_then(|v| v.as_str().map(|s| s.to_string()));
        
        // Focus the element
        let focus_script = format!(
            r#"
            (function() {{
                const element = document.querySelector('{}');
                if (element) {{
                    element.focus();
                    return document.activeElement === element;
                }}
                return false;
            }})()"#,
            input.selector
        );
        
        let element_focused = self.browser.execute_script(&focus_script).await?
            .as_bool()
            .unwrap_or(false);
        
        Ok(FocusOutput {
            success: true,
            element_focused,
            previous_active_element: previous_active,
        })
    }
    
    async fn validate_input(&self, input: &Self::Input) -> Result<()> {
        if input.selector.is_empty() {
            return Err(anyhow!("Selector cannot be empty"));
        }
        Ok(())
    }
}