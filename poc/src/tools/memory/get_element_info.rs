//! Get Element Info tool implementation
//! 
//! Retrieves detailed information about page elements including attributes,
//! styles, position, state, and optionally screenshots.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use anyhow::{Result, Context};
use tracing::{info, debug, warn};
use serde_json::{json, Value};

use crate::tools::{Tool, ToolError, DynamicTool};
use crate::tools::types::*;
use crate::browser::Browser;

/// Parameters for get_element_info tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetElementInfoParams {
    /// CSS selector or element ID
    pub selector: String,
    
    /// Optional configuration options
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<ElementInfoOptions>,
}

/// Options for element info retrieval
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ElementInfoOptions {
    /// Include computed CSS styles
    #[serde(default)]
    pub include_computed_style: bool,
    
    /// Include all HTML attributes
    #[serde(default)]
    pub include_attributes: bool,
    
    /// Include data-* attributes
    #[serde(default)]
    pub include_dataset: bool,
    
    /// Include screenshot of the element
    #[serde(default)]
    pub include_screenshot: bool,
    
    /// Traverse child elements
    #[serde(default)]
    pub traverse_children: bool,
    
    /// Maximum depth for child traversal
    #[serde(default = "default_max_depth")]
    pub max_depth: u32,
}

fn default_max_depth() -> u32 {
    3
}

/// Bounding box information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundingBox {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
    pub left: f64,
}

/// Complete element information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementInfo {
    // Basic information
    pub tag_name: String,
    pub text_content: String,
    pub inner_text: String,
    pub inner_html: String,
    pub outer_html: String,
    
    // Identification
    pub id: Option<String>,
    pub class_list: Vec<String>,
    pub unique_id: String,
    
    // Position information
    pub bounding_box: BoundingBox,
    
    // State information
    pub is_visible: bool,
    pub is_enabled: bool,
    pub is_selected: bool,
    pub is_focused: bool,
    pub is_in_viewport: bool,
    
    // Optional attributes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attributes: Option<HashMap<String, String>>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dataset: Option<HashMap<String, String>>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub computed_style: Option<HashMap<String, String>>,
    
    // Relationships
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<Box<ElementInfo>>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<ElementInfo>>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub siblings: Option<Vec<ElementInfo>>,
    
    // Element-specific information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_type: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_value: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub href: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub src: Option<String>,
    
    // Screenshot
    #[serde(skip_serializing_if = "Option::is_none")]
    pub screenshot: Option<ElementScreenshot>,
}

/// Element screenshot data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementScreenshot {
    pub format: String,
    pub data: String, // Base64 encoded
}

/// Get Element Info tool
pub struct GetElementInfo {
    browser: Arc<Browser>,
}

impl GetElementInfo {
    /// Create a new GetElementInfo tool
    pub fn new(browser: Arc<Browser>) -> Self {
        Self { browser }
    }
    
    /// Get element information from the page
    async fn fetch_element_info(
        &self,
        selector: &str,
        options: &ElementInfoOptions,
    ) -> Result<ElementInfo> {
        // In mock mode, return simulated element info
        if std::env::var("RAINBOW_MOCK_MODE").unwrap_or_default() == "true" {
            return Ok(self.create_mock_element_info(selector, options));
        }
        
        // Real implementation would interact with browser
        // For now, return mock data
        Ok(self.create_mock_element_info(selector, options))
    }
    
    /// Create mock element information for testing
    fn create_mock_element_info(
        &self,
        selector: &str,
        options: &ElementInfoOptions,
    ) -> ElementInfo {
        let mut info = ElementInfo {
            tag_name: "div".to_string(),
            text_content: format!("Mock content for {}", selector),
            inner_text: format!("Mock text for {}", selector),
            inner_html: format!("<span>Mock HTML for {}</span>", selector),
            outer_html: format!("<div id=\"mock\">{}</div>", selector),
            
            id: Some("mock-element".to_string()),
            class_list: vec!["mock-class".to_string(), "test-element".to_string()],
            unique_id: format!("unique-{}", uuid::Uuid::new_v4()),
            
            bounding_box: BoundingBox {
                x: 100.0,
                y: 200.0,
                width: 300.0,
                height: 150.0,
                top: 200.0,
                right: 400.0,
                bottom: 350.0,
                left: 100.0,
            },
            
            is_visible: true,
            is_enabled: true,
            is_selected: false,
            is_focused: false,
            is_in_viewport: true,
            
            attributes: None,
            dataset: None,
            computed_style: None,
            parent: None,
            children: None,
            siblings: None,
            input_type: None,
            input_value: None,
            href: None,
            src: None,
            screenshot: None,
        };
        
        // Add optional data based on options
        if options.include_attributes {
            info.attributes = Some(HashMap::from([
                ("class".to_string(), "mock-class test-element".to_string()),
                ("id".to_string(), "mock-element".to_string()),
                ("data-testid".to_string(), "test-123".to_string()),
            ]));
        }
        
        if options.include_dataset {
            info.dataset = Some(HashMap::from([
                ("testid".to_string(), "test-123".to_string()),
                ("value".to_string(), "mock-value".to_string()),
            ]));
        }
        
        if options.include_computed_style {
            info.computed_style = Some(HashMap::from([
                ("display".to_string(), "block".to_string()),
                ("position".to_string(), "relative".to_string()),
                ("color".to_string(), "rgb(0, 0, 0)".to_string()),
                ("background-color".to_string(), "rgba(255, 255, 255, 1)".to_string()),
                ("font-size".to_string(), "14px".to_string()),
                ("opacity".to_string(), "1".to_string()),
            ]));
        }
        
        if options.include_screenshot {
            info.screenshot = Some(ElementScreenshot {
                format: "png".to_string(),
                data: "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNkYPhfDwAChwGA60e6kgAAAABJRU5ErkJggg==".to_string(),
            });
        }
        
        if options.traverse_children && options.max_depth > 0 {
            info.children = Some(vec![
                ElementInfo {
                    tag_name: "span".to_string(),
                    text_content: "Child element 1".to_string(),
                    inner_text: "Child element 1".to_string(),
                    inner_html: "Child element 1".to_string(),
                    outer_html: "<span>Child element 1</span>".to_string(),
                    id: None,
                    class_list: vec!["child".to_string()],
                    unique_id: format!("child-{}", uuid::Uuid::new_v4()),
                    bounding_box: BoundingBox {
                        x: 110.0,
                        y: 210.0,
                        width: 100.0,
                        height: 30.0,
                        top: 210.0,
                        right: 210.0,
                        bottom: 240.0,
                        left: 110.0,
                    },
                    is_visible: true,
                    is_enabled: true,
                    is_selected: false,
                    is_focused: false,
                    is_in_viewport: true,
                    attributes: None,
                    dataset: None,
                    computed_style: None,
                    parent: None,
                    children: None,
                    siblings: None,
                    input_type: None,
                    input_value: None,
                    href: None,
                    src: None,
                    screenshot: None,
                },
            ]);
        }
        
        info
    }
}

#[async_trait]
impl Tool for GetElementInfo {
    type Input = GetElementInfoParams;
    type Output = ElementInfo;
    
    fn name(&self) -> &str {
        "get_element_info"
    }
    
    fn description(&self) -> &str {
        "Get detailed information about a page element including attributes, styles, position, and state"
    }
    
    async fn execute(&self, params: Self::Input) -> Result<Self::Output> {
        debug!("Getting element info for selector: {}", params.selector);
        
        let options = params.options.unwrap_or_default();
        let start = std::time::Instant::now();
        
        let info = self.fetch_element_info(&params.selector, &options).await?;
        
        let duration = start.elapsed();
        info!("Retrieved element info in {:?}", duration);
        
        Ok(info)
    }
    
    fn validate_input(&self, params: &Self::Input) -> Result<()> {
        if params.selector.trim().is_empty() {
            return Err(anyhow::anyhow!("Selector cannot be empty"));
        }
        
        if let Some(options) = &params.options {
            if options.max_depth > 10 {
                return Err(anyhow::anyhow!("Max depth cannot exceed 10"));
            }
        }
        
        Ok(())
    }
    
    fn input_schema(&self) -> serde_json::Value {
        json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object",
            "properties": {
                "selector": {
                    "type": "string",
                    "description": "CSS selector or element ID"
                },
                "options": {
                    "type": "object",
                    "properties": {
                        "include_computed_style": {
                            "type": "boolean",
                            "default": false
                        },
                        "include_attributes": {
                            "type": "boolean",
                            "default": false
                        },
                        "include_dataset": {
                            "type": "boolean",
                            "default": false
                        },
                        "include_screenshot": {
                            "type": "boolean",
                            "default": false
                        },
                        "traverse_children": {
                            "type": "boolean",
                            "default": false
                        },
                        "max_depth": {
                            "type": "integer",
                            "default": 3,
                            "minimum": 0,
                            "maximum": 10
                        }
                    }
                }
            },
            "required": ["selector"]
        })
    }
    
    fn output_schema(&self) -> serde_json::Value {
        json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object",
            "properties": {
                "tag_name": {"type": "string"},
                "text_content": {"type": "string"},
                "inner_text": {"type": "string"},
                "inner_html": {"type": "string"},
                "outer_html": {"type": "string"},
                "id": {"type": ["string", "null"]},
                "class_list": {
                    "type": "array",
                    "items": {"type": "string"}
                },
                "unique_id": {"type": "string"},
                "bounding_box": {
                    "type": "object",
                    "properties": {
                        "x": {"type": "number"},
                        "y": {"type": "number"},
                        "width": {"type": "number"},
                        "height": {"type": "number"},
                        "top": {"type": "number"},
                        "right": {"type": "number"},
                        "bottom": {"type": "number"},
                        "left": {"type": "number"}
                    }
                },
                "is_visible": {"type": "boolean"},
                "is_enabled": {"type": "boolean"},
                "is_selected": {"type": "boolean"},
                "is_focused": {"type": "boolean"},
                "is_in_viewport": {"type": "boolean"}
            },
            "required": [
                "tag_name", "text_content", "unique_id", "bounding_box",
                "is_visible", "is_enabled", "is_in_viewport"
            ]
        })
    }
}

// Implement DynamicTool for runtime dispatch
#[async_trait]
impl DynamicTool for GetElementInfo {
    fn name(&self) -> &str {
        Tool::name(self)
    }
    
    async fn execute_json(&self, params: serde_json::Value) -> Result<serde_json::Value> {
        let input: GetElementInfoParams = serde_json::from_value(params)
            .context("Failed to parse GetElementInfo parameters")?;
        let output = self.execute(input).await?;
        Ok(serde_json::to_value(output)?)
    }
    
    fn input_schema(&self) -> serde_json::Value {
        Tool::input_schema(self)
    }
    
    fn output_schema(&self) -> serde_json::Value {
        Tool::output_schema(self)
    }
}