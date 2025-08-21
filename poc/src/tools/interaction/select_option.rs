// SelectOption Tool Implementation
// Full specification compliance for select/dropdown element interaction

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::browser::Browser;
use crate::tools::{Tool, ToolError, Result};

/// Parameters for the select_option tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectOptionParams {
    /// CSS selector for the select element
    pub selector: String,
    
    /// Value(s) to select
    pub value: SelectValue,
    
    /// Optional selection options
    #[serde(default)]
    pub options: SelectOptions,
}

/// Value to select - can be single or multiple
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SelectValue {
    /// Single value to select
    Single(String),
    
    /// Multiple values to select (for multi-select)
    Multiple(Vec<String>),
}

/// Advanced selection options
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SelectOptions {
    /// How to match the option
    #[serde(default)]
    pub by: SelectBy,
    
    /// Deselect all other options first
    #[serde(default)]
    pub deselect_others: bool,
    
    /// Wait for options to be loaded
    #[serde(default)]
    pub wait_for_options: bool,
    
    /// Minimum number of options to wait for
    #[serde(default = "default_min_options")]
    pub min_options: usize,
    
    /// Trigger change event after selection
    #[serde(default = "default_true")]
    pub trigger_change: bool,
    
    /// Timeout in milliseconds
    #[serde(default = "default_timeout")]
    pub timeout: u64,
    
    /// Allow selecting disabled options
    #[serde(default)]
    pub force: bool,
    
    /// Focus select element before selection
    #[serde(default = "default_true")]
    pub focus_first: bool,
    
    /// Blur element after selection
    #[serde(default)]
    pub blur_after: bool,
    
    /// Scroll element into view
    #[serde(default)]
    pub scroll_into_view: bool,
}

/// Method to use for selecting options
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SelectBy {
    /// Select by option value attribute
    Value,
    
    /// Select by visible text
    Text,
    
    /// Select by option index
    Index,
    
    /// Select by partial text match
    PartialText,
    
    /// Select by data attribute
    DataAttribute(String),
}

impl Default for SelectBy {
    fn default() -> Self {
        SelectBy::Value
    }
}

/// Result of a select_option operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectOptionResult {
    /// Whether the selection was successful
    pub success: bool,
    
    /// Information about the select element
    pub select_element: SelectElementInfo,
    
    /// Options that were selected
    pub selected: Vec<OptionInfo>,
    
    /// Options that were deselected
    pub deselected: Vec<OptionInfo>,
    
    /// All available options
    pub available_options: Vec<OptionInfo>,
    
    /// Timing information
    pub timing: SelectTiming,
    
    /// Events triggered
    pub events_triggered: Vec<String>,
    
    /// Error information if failed
    pub error: Option<String>,
}

/// Information about the select element
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectElementInfo {
    /// Tag name (should be "select")
    pub tag_name: String,
    
    /// Name attribute
    pub name: Option<String>,
    
    /// ID attribute
    pub id: Option<String>,
    
    /// Whether multiple selections are allowed
    pub multiple: bool,
    
    /// Whether element is required
    pub required: bool,
    
    /// Whether element is disabled
    pub disabled: bool,
    
    /// Number of options
    pub option_count: usize,
    
    /// Size attribute (visible rows)
    pub size: Option<i32>,
    
    /// Form ID if part of a form
    pub form: Option<String>,
}

/// Information about an option
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionInfo {
    /// Option value attribute
    pub value: String,
    
    /// Visible text
    pub text: String,
    
    /// Option index
    pub index: usize,
    
    /// Whether option is selected
    pub selected: bool,
    
    /// Whether option is disabled
    pub disabled: bool,
    
    /// Whether option is default selected
    pub default_selected: bool,
    
    /// Option label if present
    pub label: Option<String>,
}

/// Timing information for the selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectTiming {
    /// Time to find element (ms)
    pub element_found: u64,
    
    /// Time to wait for options (ms)
    pub options_loaded: u64,
    
    /// Time to perform selection (ms)
    pub selection: u64,
    
    /// Total operation time (ms)
    pub total: u64,
}

/// SelectOption tool implementation
pub struct SelectOption {
    browser: Arc<Browser>,
}

impl SelectOption {
    pub fn new(browser: Arc<Browser>) -> Self {
        Self { browser }
    }
    
    async fn wait_for_options(&self, selector: &str, min_options: usize, timeout: u64) -> Result<()> {
        let start = Instant::now();
        let timeout_duration = Duration::from_millis(timeout);
        
        loop {
            let option_count = self.browser.execute_script(
                &format!(
                    r#"
                    (() => {{
                        const select = document.querySelector('{}');
                        if (!select) return 0;
                        return select.options.length;
                    }})()
                    "#,
                    selector
                ),
                vec![]
            ).await?
            .as_u64()
            .unwrap_or(0) as usize;
            
            if option_count >= min_options {
                return Ok(());
            }
            
            if start.elapsed() > timeout_duration {
                return Err(ToolError::Timeout(format!(
                    "Select element {} did not have {} options after {}ms (found {})",
                    selector, min_options, timeout, option_count
                )));
            }
            
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }
    
    async fn get_select_info(&self, selector: &str) -> Result<SelectElementInfo> {
        let info = self.browser.execute_script(
            &format!(
                r#"
                (() => {{
                    const select = document.querySelector('{}');
                    if (!select || select.tagName.toLowerCase() !== 'select') return null;
                    
                    return {{
                        tagName: select.tagName.toLowerCase(),
                        name: select.name || null,
                        id: select.id || null,
                        multiple: select.multiple,
                        required: select.required || select.hasAttribute('required'),
                        disabled: select.disabled || select.hasAttribute('disabled'),
                        optionCount: select.options.length,
                        size: select.size > 0 ? select.size : null,
                        form: select.form ? select.form.id : null
                    }};
                }})()
                "#,
                selector
            ),
            vec![]
        ).await?;
        
        serde_json::from_value(info)
            .map_err(|e| ToolError::JavaScriptError(format!("Failed to parse select info: {}", e)))
    }
    
    async fn get_all_options(&self, selector: &str) -> Result<Vec<OptionInfo>> {
        let options = self.browser.execute_script(
            &format!(
                r#"
                (() => {{
                    const select = document.querySelector('{}');
                    if (!select) return [];
                    
                    return Array.from(select.options).map((option, index) => ({{
                        value: option.value,
                        text: option.text,
                        index: index,
                        selected: option.selected,
                        disabled: option.disabled,
                        defaultSelected: option.defaultSelected,
                        label: option.label || null
                    }}));
                }})()
                "#,
                selector
            ),
            vec![]
        ).await?;
        
        serde_json::from_value(options)
            .map_err(|e| ToolError::JavaScriptError(format!("Failed to parse options: {}", e)))
    }
    
    async fn get_selected_options(&self, selector: &str) -> Result<Vec<OptionInfo>> {
        let options = self.get_all_options(selector).await?;
        Ok(options.into_iter().filter(|o| o.selected).collect())
    }
    
    async fn select_by_value(&self, selector: &str, value: &str, force: bool) -> Result<bool> {
        self.browser.execute_script(
            &format!(
                r#"
                (() => {{
                    const select = document.querySelector('{}');
                    if (!select) return false;
                    
                    const option = Array.from(select.options).find(o => o.value === '{}');
                    if (!option) return false;
                    
                    if (!{} && option.disabled) return false;
                    
                    option.selected = true;
                    return true;
                }})()
                "#,
                selector, value, force
            ),
            vec![]
        ).await?
        .json()
        .as_bool()
        .ok_or_else(|| ToolError::JavaScriptError("Failed to select by value".into()))
    }
    
    async fn select_by_text(&self, selector: &str, text: &str, partial: bool, force: bool) -> Result<bool> {
        let script = if partial {
            format!(
                r#"
                (() => {{
                    const select = document.querySelector('{}');
                    if (!select) return false;
                    
                    const option = Array.from(select.options).find(o => 
                        o.text.toLowerCase().includes('{}')
                    );
                    if (!option) return false;
                    
                    if (!{} && option.disabled) return false;
                    
                    option.selected = true;
                    return true;
                }})()
                "#,
                selector, text.to_lowercase(), force
            )
        } else {
            format!(
                r#"
                (() => {{
                    const select = document.querySelector('{}');
                    if (!select) return false;
                    
                    const option = Array.from(select.options).find(o => o.text === '{}');
                    if (!option) return false;
                    
                    if (!{} && option.disabled) return false;
                    
                    option.selected = true;
                    return true;
                }})()
                "#,
                selector, text, force
            )
        };
        
        self.browser.execute_script(&script, vec![]).await?
            .value()
            .as_bool()
            .ok_or_else(|| ToolError::JavaScriptError("Failed to select by text".into()))
    }
    
    async fn select_by_index(&self, selector: &str, index: usize, force: bool) -> Result<bool> {
        self.browser.execute_script(
            &format!(
                r#"
                (() => {{
                    const select = document.querySelector('{}');
                    if (!select) return false;
                    
                    const option = select.options[{}];
                    if (!option) return false;
                    
                    if (!{} && option.disabled) return false;
                    
                    option.selected = true;
                    return true;
                }})()
                "#,
                selector, index, force
            ),
            vec![]
        ).await?
        .json()
        .as_bool()
        .ok_or_else(|| ToolError::JavaScriptError("Failed to select by index".into()))
    }
    
    async fn deselect_all(&self, selector: &str) -> Result<()> {
        self.browser.execute_script(
            &format!(
                r#"
                (() => {{
                    const select = document.querySelector('{}');
                    if (!select) return;
                    
                    Array.from(select.options).forEach(option => {{
                        option.selected = false;
                    }});
                }})()
                "#,
                selector
            ),
            vec![]
        ).await?;
        Ok(())
    }
}

#[async_trait]
impl Tool for SelectOption {
    type Input = SelectOptionParams;
    type Output = SelectOptionResult;
    
    fn name(&self) -> &str {
        "select_option"
    }
    
    fn description(&self) -> &str {
        "Select options in dropdown/select elements with support for multi-select"
    }
    
    async fn execute(&self, params: Self::Input) -> Result<Self::Output> {
        let start_time = Instant::now();
        let mut timing = SelectTiming {
            element_found: 0,
            options_loaded: 0,
            selection: 0,
            total: 0,
        };
        
        // Find select element
        self.browser.find_element(&params.selector).await
            .map_err(|_| ToolError::ElementNotFound(format!("Select element {} not found", params.selector)))?;
        
        timing.element_found = start_time.elapsed().as_millis() as u64;
        
        // Get select element info
        let select_info = self.get_select_info(&params.selector).await?;
        
        // Check if element is disabled
        if select_info.disabled && !params.options.force {
            return Err(ToolError::JavaScriptError("Select element is disabled".into()));
        }
        
        // Scroll into view if requested
        if params.options.scroll_into_view {
            self.browser.execute_script(
                &format!(
                    "document.querySelector('{}').scrollIntoView({{ behavior: 'smooth', block: 'center' }})",
                    params.selector
                ),
                vec![]
            ).await?;
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
        
        // Focus element if requested
        if params.options.focus_first {
            self.browser.execute_script(
                &format!("document.querySelector('{}').focus()", params.selector),
                vec![]
            ).await?;
        }
        
        // Wait for options if requested
        if params.options.wait_for_options {
            let wait_start = Instant::now();
            self.wait_for_options(&params.selector, params.options.min_options, params.options.timeout).await?;
            timing.options_loaded = wait_start.elapsed().as_millis() as u64;
        }
        
        // Get current selection before changes
        let previously_selected = self.get_selected_options(&params.selector).await?;
        
        // Deselect all if requested
        if params.options.deselect_others {
            self.deselect_all(&params.selector).await?;
        }
        
        let selection_start = Instant::now();
        let mut events_triggered = Vec::new();
        
        // Convert value to list
        let values = match params.value {
            SelectValue::Single(v) => vec![v],
            SelectValue::Multiple(v) => {
                // Check if multiple selection is allowed
                if !select_info.multiple {
                    return Err(ToolError::InvalidInput(
                        "Multiple values provided but select element does not support multiple selection".into()
                    ));
                }
                v
            }
        };
        
        // Select each value
        let mut selection_success = true;
        for value in &values {
            let success = match params.options.by {
                SelectBy::Value => {
                    self.select_by_value(&params.selector, value, params.options.force).await?
                },
                SelectBy::Text => {
                    self.select_by_text(&params.selector, value, false, params.options.force).await?
                },
                SelectBy::PartialText => {
                    self.select_by_text(&params.selector, value, true, params.options.force).await?
                },
                SelectBy::Index => {
                    let index: usize = value.parse()
                        .map_err(|_| ToolError::InvalidInput(format!("Invalid index: {}", value)))?;
                    self.select_by_index(&params.selector, index, params.options.force).await?
                },
                SelectBy::DataAttribute(ref attr) => {
                    self.browser.execute_script(
                        &format!(
                            r#"
                            (() => {{
                                const select = document.querySelector('{}');
                                if (!select) return false;
                                
                                const option = Array.from(select.options).find(o => 
                                    o.dataset['{}'] === '{}'
                                );
                                if (!option) return false;
                                
                                if (!{} && option.disabled) return false;
                                
                                option.selected = true;
                                return true;
                            }})()
                            "#,
                            params.selector, attr, value, params.options.force
                        ),
                        vec![]
                    ).await?
                    .json()
                    .as_bool()
                    .unwrap_or(false)
                }
            };
            
            if !success {
                selection_success = false;
                break;
            }
        }
        
        // Trigger change event if requested
        if params.options.trigger_change {
            self.browser.execute_script(
                &format!(
                    r#"
                    (() => {{
                        const select = document.querySelector('{}');
                        select.dispatchEvent(new Event('change', {{ bubbles: true }}));
                    }})()
                    "#,
                    params.selector
                ),
                vec![]
            ).await?;
            events_triggered.push("change".to_string());
        }
        
        // Blur element if requested
        if params.options.blur_after {
            self.browser.execute_script(
                &format!("document.querySelector('{}').blur()", params.selector),
                vec![]
            ).await?;
            events_triggered.push("blur".to_string());
        }
        
        timing.selection = selection_start.elapsed().as_millis() as u64;
        
        // Get final selection
        let currently_selected = self.get_selected_options(&params.selector).await?;
        
        // Get all available options
        let available_options = self.get_all_options(&params.selector).await?;
        
        // Calculate deselected options
        let deselected: Vec<OptionInfo> = previously_selected
            .into_iter()
            .filter(|prev| !currently_selected.iter().any(|curr| curr.value == prev.value))
            .collect();
        
        timing.total = start_time.elapsed().as_millis() as u64;
        
        Ok(SelectOptionResult {
            success: selection_success,
            select_element: select_info,
            selected: currently_selected,
            deselected,
            available_options,
            timing,
            events_triggered,
            error: if selection_success { None } else { Some("Failed to select one or more options".into()) },
        })
    }
    
    fn validate_input(&self, params: &Self::Input) -> Result<()> {
        if params.selector.is_empty() {
            return Err(ToolError::InvalidInput("Selector cannot be empty".into()));
        }
        
        // Validate that values are not empty
        match &params.value {
            SelectValue::Single(v) if v.is_empty() => {
                return Err(ToolError::InvalidInput("Value cannot be empty".into()));
            },
            SelectValue::Multiple(values) if values.is_empty() => {
                return Err(ToolError::InvalidInput("At least one value must be provided".into()));
            },
            SelectValue::Multiple(values) if values.iter().any(|v| v.is_empty()) => {
                return Err(ToolError::InvalidInput("Values cannot be empty".into()));
            },
            _ => {}
        }
        
        // Validate index if using index selection
        if let SelectBy::Index = params.options.by {
            match &params.value {
                SelectValue::Single(v) => {
                    v.parse::<usize>()
                        .map_err(|_| ToolError::InvalidInput(format!("Invalid index: {}", v)))?;
                },
                SelectValue::Multiple(ref values) => {
                    for v in values {
                        v.parse::<usize>()
                            .map_err(|_| ToolError::InvalidInput(format!("Invalid index: {}", v)))?;
                    }
                },
                _ => {}
            }
        }
        
        Ok(())
    }
    
    fn input_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "required": ["selector", "value"],
            "properties": {
                "selector": {
                    "type": "string",
                    "description": "CSS selector for the select element"
                },
                "value": {
                    "oneOf": [
                        { "type": "string" },
                        { "type": "array", "items": { "type": "string" } }
                    ],
                    "description": "Value(s) to select"
                },
                "options": {
                    "type": "object",
                    "properties": {
                        "by": {
                            "type": "string",
                            "enum": ["value", "text", "index", "partial_text"],
                            "default": "value"
                        },
                        "deselectOthers": { "type": "boolean", "default": false },
                        "waitForOptions": { "type": "boolean", "default": false },
                        "minOptions": { "type": "integer", "default": 1 },
                        "triggerChange": { "type": "boolean", "default": true },
                        "timeout": { "type": "integer", "default": 30000 },
                        "force": { "type": "boolean", "default": false },
                        "focusFirst": { "type": "boolean", "default": true },
                        "blurAfter": { "type": "boolean", "default": false },
                        "scrollIntoView": { "type": "boolean", "default": false }
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
                "selectElement": { "type": "object" },
                "selected": { "type": "array" },
                "deselected": { "type": "array" },
                "availableOptions": { "type": "array" },
                "timing": { "type": "object" },
                "eventsTriggered": { "type": "array" },
                "error": { "type": "string" }
            }
        })
    }
}

// Helper functions
fn default_min_options() -> usize { 1 }
fn default_timeout() -> u64 { 30000 }
fn default_true() -> bool { true }