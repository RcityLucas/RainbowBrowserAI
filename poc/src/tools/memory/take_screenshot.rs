//! Take Screenshot tool implementation
//! 
//! Captures screenshots of the page, viewport, or specific elements,
//! with support for various formats and quality settings.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use anyhow::{Result, Context};
use tracing::{info, debug, warn};
use serde_json::json;
use base64::{Engine as _, engine::general_purpose};

use crate::tools::{Tool, DynamicTool};
use crate::browser::Browser;

/// Parameters for take_screenshot tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TakeScreenshotParams {
    /// Optional screenshot options
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<ScreenshotOptions>,
}

/// Screenshot configuration options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenshotOptions {
    /// Type of screenshot to capture
    #[serde(default = "default_screenshot_type")]
    pub screenshot_type: ScreenshotType,
    
    /// Image format
    #[serde(default = "default_image_format")]
    pub format: ImageFormat,
    
    /// JPEG quality (0-100, only for JPEG format)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality: Option<u8>,
    
    /// Capture full page instead of viewport
    #[serde(default)]
    pub full_page: bool,
    
    /// Clip rectangle for partial screenshot
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clip: Option<Rectangle>,
    
    /// Element selector for element screenshot
    #[serde(skip_serializing_if = "Option::is_none")]
    pub element: Option<String>,
    
    /// Omit background (transparent)
    #[serde(default)]
    pub omit_background: bool,
    
    /// Highlight areas on the screenshot
    #[serde(default)]
    pub highlights: Vec<Highlight>,
}

fn default_screenshot_type() -> ScreenshotType {
    ScreenshotType::Viewport
}

fn default_image_format() -> ImageFormat {
    ImageFormat::Png
}

/// Type of screenshot
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ScreenshotType {
    /// Only visible viewport
    Viewport,
    /// Full page
    FullPage,
    /// Specific element
    Element,
}

/// Image format for screenshot
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImageFormat {
    Png,
    Jpeg,
    Webp,
}

/// Rectangle for clipping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rectangle {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// Highlight area on screenshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Highlight {
    pub selector: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<HighlightStyle>,
}

/// Style for highlighting elements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighlightStyle {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub border: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opacity: Option<f32>,
}

/// Screenshot result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Screenshot {
    /// Image format
    pub format: ImageFormat,
    
    /// Binary data (base64 encoded for JSON serialization)
    pub data: Vec<u8>,
    
    /// Data URL (data:image/png;base64,...)
    pub data_url: String,
    
    /// Image dimensions
    pub dimensions: ImageDimensions,
    
    /// File size in bytes
    pub file_size: usize,
    
    /// Metadata
    pub metadata: ScreenshotMetadata,
}

/// Image dimensions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageDimensions {
    pub width: u32,
    pub height: u32,
}

/// Screenshot metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenshotMetadata {
    pub timestamp: String,
    pub url: String,
    pub device_pixel_ratio: f32,
    pub color_space: String,
}

/// Take Screenshot tool
pub struct TakeScreenshot {
    browser: Arc<Browser>,
}

impl TakeScreenshot {
    /// Create a new TakeScreenshot tool
    pub fn new(browser: Arc<Browser>) -> Self {
        Self { browser }
    }
    
    /// Capture screenshot with given options
    async fn capture_screenshot(&self, options: &ScreenshotOptions) -> Result<Screenshot> {
        // In mock mode, return simulated screenshot
        if std::env::var("RAINBOW_MOCK_MODE").unwrap_or_default() == "true" {
            return Ok(self.create_mock_screenshot(options));
        }
        
        // Real implementation would interact with browser
        // For now, return mock data
        Ok(self.create_mock_screenshot(options))
    }
    
    /// Create mock screenshot for testing
    fn create_mock_screenshot(&self, options: &ScreenshotOptions) -> Screenshot {
        // Create a simple 1x1 pixel image based on format
        let (data, mime_type) = match &options.format {
            ImageFormat::Png => {
                // 1x1 transparent PNG
                let png_data = vec![
                    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A,
                    0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
                    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01,
                    0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4,
                    0x89, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x44, 0x41,
                    0x54, 0x08, 0x5B, 0x63, 0xF8, 0x0F, 0x00, 0x00,
                    0x01, 0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x8D,
                    0xA7, 0x93, 0x24, 0x00, 0x00, 0x00, 0x00, 0x49,
                    0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
                ];
                (png_data, "image/png")
            },
            ImageFormat::Jpeg => {
                // Minimal JPEG
                let jpeg_data = vec![
                    0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46,
                    0x49, 0x46, 0x00, 0x01, 0x01, 0x00, 0x00, 0x01,
                    0x00, 0x01, 0x00, 0x00, 0xFF, 0xDB, 0x00, 0x43,
                    // ... simplified JPEG data
                    0xFF, 0xD9,
                ];
                (jpeg_data, "image/jpeg")
            },
            ImageFormat::Webp => {
                // Minimal WebP
                let webp_data = vec![
                    0x52, 0x49, 0x46, 0x46, 0x24, 0x00, 0x00, 0x00,
                    0x57, 0x45, 0x42, 0x50, 0x56, 0x50, 0x38, 0x20,
                    // ... simplified WebP data
                ];
                (webp_data, "image/webp")
            },
        };
        
        // Create base64 encoded data URL
        let base64_data = general_purpose::STANDARD.encode(&data);
        let data_url = format!("data:{};base64,{}", mime_type, base64_data);
        
        // Determine dimensions based on screenshot type
        let (width, height) = match options.screenshot_type {
            ScreenshotType::Viewport => (1920, 1080),
            ScreenshotType::FullPage => {
                if options.full_page {
                    (1920, 3000)  // Simulate longer page
                } else {
                    (1920, 1080)
                }
            },
            ScreenshotType::Element => {
                // Simulate element dimensions
                if let Some(element) = &options.element {
                    if element.contains("button") {
                        (120, 40)
                    } else if element.contains("image") || element.contains("img") {
                        (800, 600)
                    } else {
                        (300, 200)
                    }
                } else {
                    (300, 200)
                }
            },
        };
        
        Screenshot {
            format: options.format.clone(),
            data: data.clone(),
            data_url,
            dimensions: ImageDimensions { width, height },
            file_size: data.len(),
            metadata: ScreenshotMetadata {
                timestamp: chrono::Utc::now().to_rfc3339(),
                url: "https://example.com".to_string(),
                device_pixel_ratio: 2.0,
                color_space: "srgb".to_string(),
            },
        }
    }
}

impl Default for ScreenshotOptions {
    fn default() -> Self {
        Self {
            screenshot_type: default_screenshot_type(),
            format: default_image_format(),
            quality: None,
            full_page: false,
            clip: None,
            element: None,
            omit_background: false,
            highlights: Vec::new(),
        }
    }
}

#[async_trait]
impl Tool for TakeScreenshot {
    type Input = TakeScreenshotParams;
    type Output = Screenshot;
    
    fn name(&self) -> &str {
        "take_screenshot"
    }
    
    fn description(&self) -> &str {
        "Capture screenshots of the page, viewport, or specific elements"
    }
    
    async fn execute(&self, params: Self::Input) -> Result<Self::Output> {
        let options = params.options.unwrap_or_default();
        
        debug!("Taking screenshot with options: {:?}", options);
        let start = std::time::Instant::now();
        
        // Apply highlights if specified
        if !options.highlights.is_empty() {
            debug!("Applying {} highlights", options.highlights.len());
            // In real implementation, would inject CSS to highlight elements
        }
        
        let screenshot = self.capture_screenshot(&options).await?;
        
        let duration = start.elapsed();
        info!(
            "Captured {} screenshot in {:?} ({} bytes)",
            match options.format {
                ImageFormat::Png => "PNG",
                ImageFormat::Jpeg => "JPEG",
                ImageFormat::Webp => "WebP",
            },
            duration,
            screenshot.file_size
        );
        
        Ok(screenshot)
    }
    
    fn validate_input(&self, params: &Self::Input) -> Result<()> {
        if let Some(options) = &params.options {
            // Validate quality for JPEG
            if let Some(quality) = options.quality {
                if quality > 100 {
                    return Err(anyhow::anyhow!("JPEG quality must be between 0 and 100"));
                }
                if !matches!(options.format, ImageFormat::Jpeg) && quality > 0 {
                    warn!("Quality setting is only used for JPEG format");
                }
            }
            
            // Validate element selector if element type is specified
            if matches!(options.screenshot_type, ScreenshotType::Element) && options.element.is_none() {
                return Err(anyhow::anyhow!("Element selector required for element screenshot type"));
            }
            
            // Validate clip rectangle
            if let Some(clip) = &options.clip {
                if clip.width <= 0.0 || clip.height <= 0.0 {
                    return Err(anyhow::anyhow!("Clip width and height must be positive"));
                }
                if clip.x < 0.0 || clip.y < 0.0 {
                    return Err(anyhow::anyhow!("Clip x and y must be non-negative"));
                }
            }
        }
        
        Ok(())
    }
    
    fn input_schema(&self) -> serde_json::Value {
        json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object",
            "properties": {
                "options": {
                    "type": "object",
                    "properties": {
                        "type": {
                            "type": "string",
                            "enum": ["viewport", "full_page", "element"],
                            "default": "viewport"
                        },
                        "format": {
                            "type": "string",
                            "enum": ["png", "jpeg", "webp"],
                            "default": "png"
                        },
                        "quality": {
                            "type": "integer",
                            "minimum": 0,
                            "maximum": 100,
                            "description": "JPEG quality (0-100)"
                        },
                        "full_page": {
                            "type": "boolean",
                            "default": false
                        },
                        "clip": {
                            "type": "object",
                            "properties": {
                                "x": {"type": "number"},
                                "y": {"type": "number"},
                                "width": {"type": "number", "minimum": 0},
                                "height": {"type": "number", "minimum": 0}
                            },
                            "required": ["x", "y", "width", "height"]
                        },
                        "element": {
                            "type": "string",
                            "description": "CSS selector for element screenshot"
                        },
                        "omit_background": {
                            "type": "boolean",
                            "default": false
                        },
                        "highlights": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "properties": {
                                    "selector": {"type": "string"},
                                    "style": {
                                        "type": "object",
                                        "properties": {
                                            "border": {"type": "string"},
                                            "background": {"type": "string"},
                                            "opacity": {"type": "number"}
                                        }
                                    }
                                },
                                "required": ["selector"]
                            }
                        }
                    }
                }
            }
        })
    }
    
    fn output_schema(&self) -> serde_json::Value {
        json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object",
            "properties": {
                "format": {
                    "type": "string",
                    "enum": ["png", "jpeg", "webp"]
                },
                "data": {
                    "type": "array",
                    "items": {"type": "integer"}
                },
                "data_url": {
                    "type": "string",
                    "format": "uri"
                },
                "dimensions": {
                    "type": "object",
                    "properties": {
                        "width": {"type": "integer"},
                        "height": {"type": "integer"}
                    },
                    "required": ["width", "height"]
                },
                "file_size": {"type": "integer"},
                "metadata": {
                    "type": "object",
                    "properties": {
                        "timestamp": {"type": "string"},
                        "url": {"type": "string"},
                        "device_pixel_ratio": {"type": "number"},
                        "color_space": {"type": "string"}
                    }
                }
            },
            "required": ["format", "data", "data_url", "dimensions", "file_size", "metadata"]
        })
    }
}

// Implement DynamicTool for runtime dispatch
#[async_trait]
impl DynamicTool for TakeScreenshot {
    fn name(&self) -> &str {
        Tool::name(self)
    }
    
    async fn execute_json(&self, params: serde_json::Value) -> Result<serde_json::Value> {
        let input: TakeScreenshotParams = serde_json::from_value(params)
            .context("Failed to parse TakeScreenshot parameters")?;
        let output = self.execute(input).await?;
        
        // Convert binary data to base64 for JSON serialization
        let mut json_output = serde_json::to_value(&output)?;
        if let Some(data) = json_output.get_mut("data") {
            let base64_data = general_purpose::STANDARD.encode(&output.data);
            *data = serde_json::Value::String(base64_data);
        }
        
        Ok(json_output)
    }
    
    fn input_schema(&self) -> serde_json::Value {
        Tool::input_schema(self)
    }
    
    fn output_schema(&self) -> serde_json::Value {
        Tool::output_schema(self)
    }
}