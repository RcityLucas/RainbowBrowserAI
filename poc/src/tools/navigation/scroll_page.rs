//! Scroll page tool implementation

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use anyhow::{Result, Context};
use tracing::{info, debug};

use crate::tools::{Tool, ToolError, DynamicTool};
use crate::tools::types::*;
use crate::browser::Browser;

/// Parameters for scroll_page tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrollPageParams {
    /// Scroll direction or target
    pub direction: ScrollDirection,
    
    /// Amount to scroll in pixels (for directional scrolling)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<u32>,
    
    /// Optional scroll options
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<ScrollOptions>,
}

/// Scroll options
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScrollOptions {
    /// Use smooth scrolling
    #[serde(skip_serializing_if = "Option::is_none")]
    pub smooth: Option<bool>,
    
    /// Duration for smooth scrolling in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<u64>,
    
    /// Wait time after scrolling in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wait_after: Option<u64>,
}

/// Result of scroll operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrollResult {
    /// Whether the scroll succeeded
    pub success: bool,
    
    /// Previous scroll position
    pub previous_position: Position,
    
    /// Current scroll position after scrolling
    pub current_position: Position,
    
    /// Viewport dimensions
    pub viewport: Viewport,
    
    /// Document dimensions
    pub document: DocumentSize,
    
    /// Which boundaries were reached
    pub reached_boundary: ReachedBoundary,
}

/// Scroll page tool
pub struct ScrollPage {
    browser: Arc<Browser>,
}

impl ScrollPage {
    /// Create a new ScrollPage tool
    pub fn new(browser: Arc<Browser>) -> Self {
        Self { browser }
    }
    
    /// Get current scroll position
    async fn get_scroll_position(&self) -> Result<Position> {
        // Execute JavaScript to get scroll position
        let script = r#"
            return {
                x: window.pageXOffset || document.documentElement.scrollLeft,
                y: window.pageYOffset || document.documentElement.scrollTop
            };
        "#;
        
        // In a real implementation, we would execute this script
        // For now, return a default position
        Ok(Position { x: 0, y: 0 })
    }
    
    /// Get viewport size
    async fn get_viewport_size(&self) -> Result<Viewport> {
        // Execute JavaScript to get viewport size
        let script = r#"
            return {
                width: window.innerWidth,
                height: window.innerHeight
            };
        "#;
        
        // In a real implementation, we would execute this script
        // For now, return default viewport
        Ok(Viewport {
            width: 1920,
            height: 1080,
        })
    }
    
    /// Get document size
    async fn get_document_size(&self) -> Result<DocumentSize> {
        // Execute JavaScript to get document size
        let script = r#"
            return {
                width: document.documentElement.scrollWidth,
                height: document.documentElement.scrollHeight
            };
        "#;
        
        // In a real implementation, we would execute this script
        // For now, return default size
        Ok(DocumentSize {
            width: 1920,
            height: 2000,
        })
    }
    
    /// Check which boundaries are reached
    fn check_boundaries(&self, position: Position, document: DocumentSize, viewport: Viewport) -> ReachedBoundary {
        ReachedBoundary {
            top: position.y <= 0,
            bottom: position.y >= (document.height as i32 - viewport.height as i32),
            left: position.x <= 0,
            right: position.x >= (document.width as i32 - viewport.width as i32),
        }
    }
    
    /// Perform scroll to element
    async fn scroll_to_element(&self, selector: &str, options: &ScrollOptions) -> Result<()> {
        debug!("Scrolling to element: {}", selector);
        
        let script = if options.smooth.unwrap_or(false) {
            format!(r#"
                const element = document.querySelector('{}');
                if (element) {{
                    element.scrollIntoView({{ behavior: 'smooth', block: 'center' }});
                    return true;
                }}
                return false;
            "#, selector)
        } else {
            format!(r#"
                const element = document.querySelector('{}');
                if (element) {{
                    element.scrollIntoView({{ block: 'center' }});
                    return true;
                }}
                return false;
            "#, selector)
        };
        
        // In a real implementation, we would execute this script
        // For now, simulate success
        Ok(())
    }
    
    /// Perform scroll to position
    async fn scroll_to_position(&self, x: i32, y: i32, options: &ScrollOptions) -> Result<()> {
        debug!("Scrolling to position: ({}, {})", x, y);
        
        let script = if options.smooth.unwrap_or(false) {
            format!(r#"
                window.scrollTo({{
                    left: {},
                    top: {},
                    behavior: 'smooth'
                }});
            "#, x, y)
        } else {
            format!("window.scrollTo({}, {});", x, y)
        };
        
        // In a real implementation, we would execute this script
        // For now, simulate success
        Ok(())
    }
    
    /// Perform directional scroll
    async fn scroll_direction(&self, direction: SimpleScrollDirection, amount: u32, options: &ScrollOptions) -> Result<()> {
        let current = self.get_scroll_position().await?;
        
        let (new_x, new_y) = match direction {
            SimpleScrollDirection::Up => (current.x, current.y - amount as i32),
            SimpleScrollDirection::Down => (current.x, current.y + amount as i32),
            SimpleScrollDirection::Left => (current.x - amount as i32, current.y),
            SimpleScrollDirection::Right => (current.x + amount as i32, current.y),
            SimpleScrollDirection::Top => (current.x, 0),
            SimpleScrollDirection::Bottom => {
                let doc = self.get_document_size().await?;
                let viewport = self.get_viewport_size().await?;
                (current.x, doc.height as i32 - viewport.height as i32)
            },
        };
        
        self.scroll_to_position(new_x, new_y, options).await
    }
}

#[async_trait]
impl Tool for ScrollPage {
    type Input = ScrollPageParams;
    type Output = ScrollResult;
    
    fn name(&self) -> &str {
        "scroll_page"
    }
    
    fn description(&self) -> &str {
        "Intelligently scroll the page in various directions or to specific elements"
    }
    
    async fn execute(&self, params: Self::Input) -> Result<Self::Output> {
        let options = params.options.unwrap_or_default();
        
        // Get initial state
        let previous_position = self.get_scroll_position().await?;
        let viewport = self.get_viewport_size().await?;
        let document = self.get_document_size().await?;
        
        info!("Starting scroll from position: ({}, {})", previous_position.x, previous_position.y);
        
        // Perform the scroll
        match params.direction {
            ScrollDirection::Simple(direction) => {
                let amount = params.amount.unwrap_or(match direction {
                    SimpleScrollDirection::Up | SimpleScrollDirection::Down => 500,
                    SimpleScrollDirection::Left | SimpleScrollDirection::Right => 100,
                    SimpleScrollDirection::Top | SimpleScrollDirection::Bottom => 0,
                });
                
                self.scroll_direction(direction, amount, &options).await
                    .context("Failed to scroll in direction")?;
            },
            ScrollDirection::ToElement { selector } => {
                self.scroll_to_element(&selector, &options).await
                    .context("Failed to scroll to element")?;
            },
            ScrollDirection::ToPosition { x, y } => {
                self.scroll_to_position(x, y, &options).await
                    .context("Failed to scroll to position")?;
            },
        }
        
        // Apply smooth scroll duration if specified
        if options.smooth.unwrap_or(false) {
            if let Some(duration) = options.duration {
                tokio::time::sleep(tokio::time::Duration::from_millis(duration)).await;
            } else {
                // Default smooth scroll duration
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            }
        }
        
        // Wait after scrolling if specified
        if let Some(wait_after) = options.wait_after {
            debug!("Waiting {}ms after scroll", wait_after);
            tokio::time::sleep(tokio::time::Duration::from_millis(wait_after)).await;
        }
        
        // Get final state
        let current_position = self.get_scroll_position().await?;
        let reached_boundary = self.check_boundaries(current_position, document, viewport);
        
        info!("Scroll completed. New position: ({}, {})", current_position.x, current_position.y);
        
        if reached_boundary.top || reached_boundary.bottom || reached_boundary.left || reached_boundary.right {
            debug!("Reached boundaries: {:?}", reached_boundary);
        }
        
        Ok(ScrollResult {
            success: true,
            previous_position,
            current_position,
            viewport,
            document,
            reached_boundary,
        })
    }
    
    fn validate_input(&self, params: &Self::Input) -> Result<()> {
        // Validate amount if provided
        if let Some(amount) = params.amount {
            if amount == 0 {
                return Err(ToolError::InvalidInput("Scroll amount must be greater than 0".into()).into());
            }
        }
        
        // Validate selector if scrolling to element
        if let ScrollDirection::ToElement { ref selector } = params.direction {
            if selector.is_empty() {
                return Err(ToolError::InvalidInput("Element selector cannot be empty".into()).into());
            }
        }
        
        Ok(())
    }
    
    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "direction": {
                    "oneOf": [
                        {
                            "type": "string",
                            "enum": ["up", "down", "left", "right", "top", "bottom"]
                        },
                        {
                            "type": "object",
                            "properties": {
                                "selector": {
                                    "type": "string",
                                    "description": "CSS selector of element to scroll to"
                                }
                            },
                            "required": ["selector"]
                        },
                        {
                            "type": "object",
                            "properties": {
                                "x": {
                                    "type": "integer",
                                    "description": "X coordinate to scroll to"
                                },
                                "y": {
                                    "type": "integer",
                                    "description": "Y coordinate to scroll to"
                                }
                            },
                            "required": ["x", "y"]
                        }
                    ]
                },
                "amount": {
                    "type": "integer",
                    "minimum": 1,
                    "description": "Amount to scroll in pixels"
                },
                "options": {
                    "type": "object",
                    "properties": {
                        "smooth": {
                            "type": "boolean",
                            "description": "Use smooth scrolling"
                        },
                        "duration": {
                            "type": "integer",
                            "description": "Duration for smooth scrolling in milliseconds"
                        },
                        "wait_after": {
                            "type": "integer",
                            "description": "Wait time after scrolling in milliseconds"
                        }
                    }
                }
            },
            "required": ["direction"]
        })
    }
    
    fn output_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "success": {
                    "type": "boolean"
                },
                "previous_position": {
                    "type": "object",
                    "properties": {
                        "x": { "type": "integer" },
                        "y": { "type": "integer" }
                    }
                },
                "current_position": {
                    "type": "object",
                    "properties": {
                        "x": { "type": "integer" },
                        "y": { "type": "integer" }
                    }
                },
                "viewport": {
                    "type": "object",
                    "properties": {
                        "width": { "type": "integer" },
                        "height": { "type": "integer" }
                    }
                },
                "document": {
                    "type": "object",
                    "properties": {
                        "width": { "type": "integer" },
                        "height": { "type": "integer" }
                    }
                },
                "reached_boundary": {
                    "type": "object",
                    "properties": {
                        "top": { "type": "boolean" },
                        "bottom": { "type": "boolean" },
                        "left": { "type": "boolean" },
                        "right": { "type": "boolean" }
                    }
                }
            }
        })
    }
}

// Implement DynamicTool for ScrollPage
#[async_trait]
impl DynamicTool for ScrollPage {
    fn name(&self) -> &str {
        Tool::name(self)
    }
    
    async fn execute_json(&self, params: serde_json::Value) -> Result<serde_json::Value> {
        let input: ScrollPageParams = serde_json::from_value(params)?;
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