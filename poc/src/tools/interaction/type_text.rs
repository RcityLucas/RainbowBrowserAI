// TypeText Tool Implementation
// Full specification compliance for text input with validation and advanced options

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::browser::Browser;
use crate::tools::{Tool, ToolError, Result};

/// Parameters for the type_text tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeTextParams {
    /// CSS selector for the input element
    pub selector: String,
    
    /// Text to type
    pub text: String,
    
    /// Optional typing options
    #[serde(default)]
    pub options: TypeTextOptions,
}

/// Advanced typing options
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TypeTextOptions {
    /// Clear the input before typing
    #[serde(default)]
    pub clear_first: bool,
    
    /// Select all text before typing (alternative to clear)
    #[serde(default)]
    pub select_all: bool,
    
    /// Delay between keystrokes in milliseconds
    #[serde(default = "default_delay")]
    pub delay: u64,
    
    /// Press Enter after typing
    #[serde(default)]
    pub press_enter: bool,
    
    /// Press Tab after typing
    #[serde(default)]
    pub press_tab: bool,
    
    /// Paste text instead of typing (faster)
    #[serde(default)]
    pub paste: bool,
    
    /// Validate input after typing
    pub validate: Option<ValidationRule>,
    
    /// Wait for element to be ready
    pub wait_for: Option<WaitOptions>,
    
    /// Timeout in milliseconds
    #[serde(default = "default_timeout")]
    pub timeout: u64,
    
    /// Focus element before typing
    #[serde(default = "default_true")]
    pub focus_first: bool,
    
    /// Blur element after typing
    #[serde(default)]
    pub blur_after: bool,
    
    /// Trigger input/change events explicitly
    #[serde(default = "default_true")]
    pub trigger_events: bool,
    
    /// Keep existing text and append
    #[serde(default)]
    pub append: bool,
    
    /// Prepend text to existing content
    #[serde(default)]
    pub prepend: bool,
}

/// Validation rules for input
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ValidationRule {
    /// Validate using regex pattern
    Pattern { regex: String },
    
    /// Validate length constraints
    Length { min: Option<usize>, max: Option<usize> },
    
    /// Validate email format
    Email,
    
    /// Validate URL format
    Url,
    
    /// Validate numeric value
    Number { min: Option<f64>, max: Option<f64> },
    
    /// Custom validation function (JavaScript)
    Custom { script: String },
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
    
    /// Wait for element to be editable
    #[serde(default = "default_true")]
    pub editable: bool,
    
    /// Maximum wait time in milliseconds
    #[serde(default = "default_wait_timeout")]
    pub timeout: u64,
}

/// Result of a type_text operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeTextResult {
    /// Whether the typing was successful
    pub success: bool,
    
    /// Information about the input element
    pub input_element: InputElementInfo,
    
    /// Validation result if validation was performed
    pub validation: Option<ValidationResult>,
    
    /// Timing information
    pub timing: TypeTextTiming,
    
    /// Events triggered
    pub events_triggered: Vec<String>,
    
    /// Error information if failed
    pub error: Option<String>,
}

/// Information about the input element
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputElementInfo {
    /// Tag name of the element
    pub tag_name: String,
    
    /// Input type attribute
    #[serde(rename = "type")]
    pub type_: Option<String>,
    
    /// Name attribute
    pub name: Option<String>,
    
    /// ID attribute
    pub id: Option<String>,
    
    /// Placeholder text
    pub placeholder: Option<String>,
    
    /// Initial value before typing
    pub initial_value: String,
    
    /// Final value after typing
    pub final_value: String,
    
    /// Max length attribute
    pub max_length: Option<i64>,
    
    /// Whether element is required
    pub required: bool,
    
    /// Whether element is readonly
    pub readonly: bool,
    
    /// Whether element is disabled
    pub disabled: bool,
    
    /// Pattern attribute for validation
    pub pattern: Option<String>,
    
    /// Autocomplete attribute
    pub autocomplete: Option<String>,
}

/// Validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Whether validation passed
    pub passed: bool,
    
    /// Validation message
    pub message: Option<String>,
    
    /// Details about validation
    pub details: Option<serde_json::Value>,
}

/// Timing information for the type_text operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeTextTiming {
    /// Time to find element (ms)
    pub element_found: u64,
    
    /// Time to clear/prepare element (ms)
    pub preparation: u64,
    
    /// Time to type/paste text (ms)
    pub typing: u64,
    
    /// Time for validation (ms)
    pub validation: u64,
    
    /// Total operation time (ms)
    pub total: u64,
}

/// TypeText tool implementation
pub struct TypeText {
    browser: Arc<Browser>,
}

impl TypeText {
    pub fn new(browser: Arc<Browser>) -> Self {
        Self { browser }
    }
    
    async fn wait_for_editable(&self, selector: &str, options: &WaitOptions) -> Result<()> {
        let start = Instant::now();
        let timeout = Duration::from_millis(options.timeout);
        
        loop {
            // Check if element exists and is editable
            let check_result = self.browser.execute_script(
                &format!(
                    r#"
                    (() => {{
                        const el = document.querySelector('{}');
                        if (!el) return {{ exists: false }};
                        
                        const isInput = ['input', 'textarea'].includes(el.tagName.toLowerCase());
                        const isContentEditable = el.contentEditable === 'true';
                        const rect = el.getBoundingClientRect();
                        const style = window.getComputedStyle(el);
                        
                        return {{
                            exists: true,
                            visible: rect.width > 0 && rect.height > 0 && 
                                    style.display !== 'none' && style.visibility !== 'hidden',
                            enabled: !el.disabled && !el.hasAttribute('disabled'),
                            editable: (isInput || isContentEditable) && !el.readOnly && !el.hasAttribute('readonly')
                        }};
                    }})()
                    "#,
                    selector
                ),
                vec![]
            ).await?;
            
            let exists = check_result["exists"].as_bool().unwrap_or(false);
            if !exists {
                if start.elapsed() > timeout {
                    return Err(ToolError::ElementNotFound(format!("Element {} not found", selector)));
                }
                tokio::time::sleep(Duration::from_millis(100)).await;
                continue;
            }
            
            let visible = !options.visible || check_result["visible"].as_bool().unwrap_or(false);
            let enabled = !options.enabled || check_result["enabled"].as_bool().unwrap_or(false);
            let editable = !options.editable || check_result["editable"].as_bool().unwrap_or(false);
            
            if visible && enabled && editable {
                return Ok(());
            }
            
            if start.elapsed() > timeout {
                return Err(ToolError::Timeout(format!(
                    "Element {} not ready after {}ms (visible: {}, enabled: {}, editable: {})",
                    selector, options.timeout, visible, enabled, editable
                )));
            }
            
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }
    
    async fn get_input_info(&self, selector: &str) -> Result<InputElementInfo> {
        let info = self.browser.execute_script(
            &format!(
                r#"
                (() => {{
                    const el = document.querySelector('{}');
                    if (!el) return null;
                    
                    return {{
                        tagName: el.tagName.toLowerCase(),
                        type: el.type || null,
                        name: el.name || null,
                        id: el.id || null,
                        placeholder: el.placeholder || null,
                        initialValue: el.value || el.textContent || '',
                        finalValue: el.value || el.textContent || '',
                        maxLength: el.maxLength > 0 ? el.maxLength : null,
                        required: el.required || el.hasAttribute('required'),
                        readonly: el.readOnly || el.hasAttribute('readonly'),
                        disabled: el.disabled || el.hasAttribute('disabled'),
                        pattern: el.pattern || null,
                        autocomplete: el.autocomplete || null
                    }};
                }})()
                "#,
                selector
            ),
            vec![]
        ).await?;
        
        serde_json::from_value(info)
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to parse input info: {}", e)))
    }
    
    async fn validate_input(&self, value: &str, rule: &ValidationRule) -> ValidationResult {
        match rule {
            ValidationRule::Pattern { regex } => {
                let valid = self.browser.execute_script(
                    &format!(
                        r#"new RegExp('{}').test('{}')"#,
                        regex, value
                    ),
                    vec![]
                ).await.ok()
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
                
                ValidationResult {
                    passed: valid,
                    message: if valid { None } else { Some(format!("Value does not match pattern: {}", regex)) },
                    details: None,
                }
            },
            ValidationRule::Length { min, max } => {
                let len = value.len();
                let valid = min.map_or(true, |m| len >= m) && max.map_or(true, |m| len <= m);
                
                ValidationResult {
                    passed: valid,
                    message: if valid { 
                        None 
                    } else { 
                        Some(format!("Length {} not in range [{:?}, {:?}]", len, min, max))
                    },
                    details: Some(json!({ "length": len, "min": min, "max": max })),
                }
            },
            ValidationRule::Email => {
                let valid = value.contains('@') && value.contains('.');
                ValidationResult {
                    passed: valid,
                    message: if valid { None } else { Some("Invalid email format".into()) },
                    details: None,
                }
            },
            ValidationRule::Url => {
                let valid = value.starts_with("http://") || value.starts_with("https://");
                ValidationResult {
                    passed: valid,
                    message: if valid { None } else { Some("Invalid URL format".into()) },
                    details: None,
                }
            },
            ValidationRule::Number { min, max } => {
                match value.parse::<f64>() {
                    Ok(num) => {
                        let valid = min.map_or(true, |m| num >= m) && max.map_or(true, |m| num <= m);
                        ValidationResult {
                            passed: valid,
                            message: if valid { 
                                None 
                            } else { 
                                Some(format!("Number {} not in range [{:?}, {:?}]", num, min, max))
                            },
                            details: Some(json!({ "value": num, "min": min, "max": max })),
                        }
                    },
                    Err(_) => ValidationResult {
                        passed: false,
                        message: Some("Value is not a valid number".into()),
                        details: None,
                    }
                }
            },
            ValidationRule::Custom { script } => {
                let result = self.browser.execute_script(
                    &format!("({})('{}')", script, value),
                    vec![]
                ).await.ok();
                
                let passed = result.as_ref()
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                    
                ValidationResult {
                    passed,
                    message: if passed { None } else { Some("Custom validation failed".into()) },
                    details: result,
                }
            }
        }
    }
}

#[async_trait]
impl Tool for TypeText {
    type Input = TypeTextParams;
    type Output = TypeTextResult;
    
    fn name(&self) -> &str {
        "type_text"
    }
    
    fn description(&self) -> &str {
        "Type or paste text into input elements with validation and event handling"
    }
    
    async fn execute(&self, params: Self::Input) -> Result<Self::Output> {
        let start_time = Instant::now();
        let mut timing = TypeTextTiming {
            element_found: 0,
            preparation: 0,
            typing: 0,
            validation: 0,
            total: 0,
        };
        
        // Wait for element if specified
        if let Some(ref wait_options) = params.options.wait_for {
            self.wait_for_editable(&params.selector, wait_options).await?;
        }
        
        timing.element_found = start_time.elapsed().as_millis() as u64;
        
        // Get initial element information
        let mut element_info = self.get_input_info(&params.selector).await?;
        let initial_value = element_info.initial_value.clone();
        
        // Focus the element if requested
        if params.options.focus_first {
            self.browser.execute_script(
                &format!("document.querySelector('{}').focus()", params.selector),
                vec![]
            ).await?;
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        
        let prep_start = Instant::now();
        
        // Handle text preparation
        let text_to_type = if params.options.append {
            format!("{}{}", initial_value, params.text)
        } else if params.options.prepend {
            format!("{}{}", params.text, initial_value)
        } else {
            params.text.clone()
        };
        
        // Clear or select text if requested
        if params.options.clear_first {
            self.browser.execute_script(
                &format!("document.querySelector('{}').value = ''", params.selector),
                vec![]
            ).await?;
        } else if params.options.select_all {
            self.browser.execute_script(
                &format!("document.querySelector('{}').select()", params.selector),
                vec![]
            ).await?;
        }
        
        timing.preparation = prep_start.elapsed().as_millis() as u64;
        
        let type_start = Instant::now();
        let mut events_triggered = Vec::new();
        
        // Type or paste the text
        if params.options.paste {
            // Paste text directly
            self.browser.execute_script(
                &format!(
                    r#"
                    (() => {{
                        const el = document.querySelector('{}');
                        const value = '{}';
                        el.value = value;
                        el.dispatchEvent(new Event('input', {{ bubbles: true }}));
                        el.dispatchEvent(new Event('change', {{ bubbles: true }}));
                        return true;
                    }})()
                    "#,
                    params.selector,
                    text_to_type.replace('\'', "\\'").replace('\n', "\\n")
                ),
                vec![]
            ).await?;
            events_triggered.extend(vec!["input".to_string(), "change".to_string()]);
        } else {
            // Type character by character
            for (i, ch) in text_to_type.chars().enumerate() {
                self.browser.execute_script(
                    &format!(
                        r#"
                        (() => {{
                            const el = document.querySelector('{}');
                            const event = new KeyboardEvent('keydown', {{
                                key: '{}',
                                code: 'Key{}',
                                bubbles: true
                            }});
                            el.dispatchEvent(event);
                            
                            el.value = el.value + '{}';
                            el.dispatchEvent(new Event('input', {{ bubbles: true }}));
                            
                            el.dispatchEvent(new KeyboardEvent('keyup', {{
                                key: '{}',
                                code: 'Key{}',
                                bubbles: true
                            }}));
                        }})()
                        "#,
                        params.selector,
                        ch, ch.to_uppercase(),
                        ch.to_string().replace('\'', "\\'"),
                        ch, ch.to_uppercase()
                    ),
                    vec![]
                ).await?;
                
                if i == 0 {
                    events_triggered.extend(vec!["keydown".to_string(), "input".to_string(), "keyup".to_string()]);
                }
                
                if params.options.delay > 0 {
                    tokio::time::sleep(Duration::from_millis(params.options.delay)).await;
                }
            }
        }
        
        // Trigger change event if requested
        if params.options.trigger_events {
            self.browser.execute_script(
                &format!(
                    r#"
                    (() => {{
                        const el = document.querySelector('{}');
                        el.dispatchEvent(new Event('change', {{ bubbles: true }}));
                    }})()
                    "#,
                    params.selector
                ),
                vec![]
            ).await?;
            events_triggered.push("change".to_string());
        }
        
        // Press Enter if requested
        if params.options.press_enter {
            self.browser.execute_script(
                &format!(
                    r#"
                    (() => {{
                        const el = document.querySelector('{}');
                        const event = new KeyboardEvent('keydown', {{
                            key: 'Enter',
                            code: 'Enter',
                            keyCode: 13,
                            bubbles: true
                        }});
                        el.dispatchEvent(event);
                    }})()
                    "#,
                    params.selector
                ),
                vec![]
            ).await?;
            events_triggered.push("keydown:Enter".to_string());
        }
        
        // Press Tab if requested
        if params.options.press_tab {
            self.browser.execute_script(
                &format!(
                    r#"
                    (() => {{
                        const el = document.querySelector('{}');
                        const event = new KeyboardEvent('keydown', {{
                            key: 'Tab',
                            code: 'Tab',
                            keyCode: 9,
                            bubbles: true
                        }});
                        el.dispatchEvent(event);
                    }})()
                    "#,
                    params.selector
                ),
                vec![]
            ).await?;
            events_triggered.push("keydown:Tab".to_string());
        }
        
        // Blur element if requested
        if params.options.blur_after {
            self.browser.execute_script(
                &format!("document.querySelector('{}').blur()", params.selector),
                vec![]
            ).await?;
            events_triggered.push("blur".to_string());
        }
        
        timing.typing = type_start.elapsed().as_millis() as u64;
        
        // Get final value
        element_info.final_value = self.browser.execute_script(
            &format!(
                "document.querySelector('{}').value || document.querySelector('{}').textContent || ''",
                params.selector, params.selector
            ),
            vec![]
        ).await?
        .json()
        .as_str()
        .unwrap_or("")
        .to_string();
        
        // Validate if requested
        let validation = if let Some(ref rule) = params.options.validate {
            let val_start = Instant::now();
            let result = self.validate_input(&element_info.final_value, rule).await;
            timing.validation = val_start.elapsed().as_millis() as u64;
            Some(result)
        } else {
            None
        };
        
        timing.total = start_time.elapsed().as_millis() as u64;
        
        let success = validation.as_ref().map_or(true, |v| v.passed);
        
        Ok(TypeTextResult {
            success,
            input_element: element_info,
            validation,
            timing,
            events_triggered,
            error: if success { None } else { Some("Validation failed".into()) },
        })
    }
    
    fn validate_input(&self, params: &Self::Input) -> Result<()> {
        if params.selector.is_empty() {
            return Err(ToolError::InvalidInput("Selector cannot be empty".into()));
        }
        
        if params.options.delay > 1000 {
            return Err(ToolError::InvalidInput("Delay cannot exceed 1000ms".into()));
        }
        
        if params.options.append && params.options.prepend {
            return Err(ToolError::InvalidInput("Cannot both append and prepend".into()));
        }
        
        if params.options.clear_first && (params.options.append || params.options.prepend) {
            return Err(ToolError::InvalidInput("Cannot clear when appending or prepending".into()));
        }
        
        Ok(())
    }
    
    fn input_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "required": ["selector", "text"],
            "properties": {
                "selector": {
                    "type": "string",
                    "description": "CSS selector for the input element"
                },
                "text": {
                    "type": "string",
                    "description": "Text to type"
                },
                "options": {
                    "type": "object",
                    "properties": {
                        "clearFirst": { "type": "boolean", "default": false },
                        "selectAll": { "type": "boolean", "default": false },
                        "delay": { "type": "integer", "default": 50 },
                        "pressEnter": { "type": "boolean", "default": false },
                        "pressTab": { "type": "boolean", "default": false },
                        "paste": { "type": "boolean", "default": false },
                        "focusFirst": { "type": "boolean", "default": true },
                        "blurAfter": { "type": "boolean", "default": false },
                        "append": { "type": "boolean", "default": false },
                        "prepend": { "type": "boolean", "default": false },
                        "timeout": { "type": "integer", "default": 30000 }
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
                "inputElement": { "type": "object" },
                "validation": { "type": "object" },
                "timing": { "type": "object" },
                "eventsTriggered": { "type": "array" },
                "error": { "type": "string" }
            }
        })
    }
}

// Helper functions
fn default_delay() -> u64 { 50 }
fn default_timeout() -> u64 { 30000 }
fn default_true() -> bool { true }
fn default_wait_timeout() -> u64 { 30000 }