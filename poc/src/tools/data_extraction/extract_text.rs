// Extract Text Tool - Phase 2 Week 5 Implementation
// 
// This tool extracts text content from web pages with multiple output formats
// and advanced extraction capabilities including selectors, text cleaning, and metadata.

use crate::tools::{Tool, ToolError};
use super::{OutputFormat, ExtractionScope, ExtractionConfig, ExtractionResult, ExtractionMetadata, text_utils, format_utils};
use std::sync::Arc;
use std::collections::HashMap;
use thirtyfour::{WebDriver, By};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::time::{Duration, Instant};
use chrono::Utc;

/// Text extraction types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TextExtractionType {
    /// Extract inner text (visible text only)
    InnerText,
    /// Extract text content (includes hidden text)
    TextContent,
    /// Extract inner HTML (with tags)
    InnerHtml,
    /// Extract outer HTML (element with its tags)
    OuterHtml,
    /// Extract specific attribute value
    Attribute(String),
    /// Extract all text from multiple elements
    All,
}

impl Default for TextExtractionType {
    fn default() -> Self {
        TextExtractionType::InnerText
    }
}

impl std::str::FromStr for TextExtractionType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "innertext" | "inner-text" | "inner_text" => Ok(TextExtractionType::InnerText),
            "textcontent" | "text-content" | "text_content" => Ok(TextExtractionType::TextContent),
            "innerhtml" | "inner-html" | "inner_html" => Ok(TextExtractionType::InnerHtml),
            "outerhtml" | "outer-html" | "outer_html" => Ok(TextExtractionType::OuterHtml),
            "all" => Ok(TextExtractionType::All),
            _ if s.starts_with("attr:") => {
                let attr_name = s.strip_prefix("attr:").unwrap();
                Ok(TextExtractionType::Attribute(attr_name.to_string()))
            }
            _ => Err(format!("Invalid extraction type: '{}'. Valid types: innertext, textcontent, innerhtml, outerhtml, all, attr:<name>", s))
        }
    }
}

/// Input parameters for extract_text tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractTextInput {
    /// CSS selector for elements to extract text from (optional, defaults to body)
    pub selector: Option<String>,
    
    /// Type of text extraction to perform
    pub extraction_type: TextExtractionType,
    
    /// Extraction configuration
    pub config: ExtractionConfig,
    
    /// Whether to extract from multiple elements or just the first match
    pub extract_multiple: bool,
    
    /// Text filters to apply (case-insensitive contains)
    pub filters: Option<Vec<String>>,
    
    /// Minimum text length to include
    pub min_length: Option<usize>,
    
    /// Maximum text length per item
    pub max_length: Option<usize>,
}

impl Default for ExtractTextInput {
    fn default() -> Self {
        Self {
            selector: None,
            extraction_type: TextExtractionType::default(),
            config: ExtractionConfig::default(),
            extract_multiple: false,
            filters: None,
            min_length: None,
            max_length: None,
        }
    }
}

/// Individual text extraction result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextItem {
    /// Extracted text content
    pub text: String,
    
    /// CSS selector used to find this element
    pub selector: String,
    
    /// Element tag name
    pub tag_name: String,
    
    /// Element attributes (optional)
    pub attributes: Option<HashMap<String, String>>,
    
    /// Text length
    pub length: usize,
    
    /// Whether text was cleaned/processed
    pub processed: bool,
}

/// Output from extract_text tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractTextOutput {
    /// Extracted text items
    pub items: Vec<TextItem>,
    
    /// Formatted output in requested format
    pub formatted_output: String,
    
    /// Total number of items extracted
    pub total_count: usize,
    
    /// Total text length across all items
    pub total_length: usize,
    
    /// Extraction configuration used
    pub config: ExtractionConfig,
}

/// Extract text tool implementation
pub struct ExtractText {
    driver: Arc<WebDriver>,
}

impl ExtractText {
    /// Create a new extract text tool
    pub fn new(driver: Arc<WebDriver>) -> Self {
        Self { driver }
    }
    
    /// Extract text from a single element
    async fn extract_from_element(&self, element: &thirtyfour::WebElement, extraction_type: &TextExtractionType, clean_text: bool) -> anyhow::Result<String> {
        let raw_text = match extraction_type {
            TextExtractionType::InnerText => {
                element.inner_html().await?
            }
            TextExtractionType::TextContent => {
                element.text().await?
            }
            TextExtractionType::InnerHtml => {
                element.inner_html().await?
            }
            TextExtractionType::OuterHtml => {
                element.outer_html().await?
            }
            TextExtractionType::Attribute(attr_name) => {
                element.attr(attr_name).await?.unwrap_or_default()
            }
            TextExtractionType::All => {
                // For 'all', combine inner text and text content
                let inner_text = element.inner_html().await?;
                let text_content = element.text().await?;
                format!("{}\n{}", inner_text, text_content)
            }
        };
        
        if clean_text {
            Ok(text_utils::clean_text(&raw_text))
        } else {
            Ok(raw_text)
        }
    }
    
    /// Apply text filters
    fn apply_filters(&self, text: &str, filters: &Option<Vec<String>>) -> bool {
        if let Some(filter_list) = filters {
            let text_lower = text.to_lowercase();
            for filter in filter_list {
                if text_lower.contains(&filter.to_lowercase()) {
                    return true;
                }
            }
            // If filters are specified but none match, exclude this text
            false
        } else {
            // No filters, include all text
            true
        }
    }
    
    /// Apply length constraints
    fn apply_length_constraints(&self, text: &str, min_length: Option<usize>, max_length: Option<usize>) -> Option<String> {
        let text_len = text.len();
        
        // Check minimum length
        if let Some(min) = min_length {
            if text_len < min {
                return None;
            }
        }
        
        // Apply maximum length
        if let Some(max) = max_length {
            if text_len > max {
                Some(text_utils::truncate_text(text, max))
            } else {
                Some(text.to_string())
            }
        } else {
            Some(text.to_string())
        }
    }
}

#[async_trait]
impl Tool for ExtractText {
    type Input = ExtractTextInput;
    type Output = ExtractTextOutput;

    fn name(&self) -> &str {
        "extract_text"
    }

    fn description(&self) -> &str {
        "Extract text content from web page elements with multiple output formats and advanced filtering"
    }

    async fn execute(&self, input: Self::Input) -> anyhow::Result<Self::Output> {
        let start_time = Instant::now();
        
        // Determine selector - default to body if not specified
        let selector = input.selector.unwrap_or_else(|| {
            match &input.config.scope {
                ExtractionScope::Element(sel) => sel.clone(),
                ExtractionScope::Viewport => "body".to_string(),
                ExtractionScope::Page => "body".to_string(),
                ExtractionScope::Region { .. } => "body".to_string(),
            }
        });
        
        // Find elements
        let elements = if input.extract_multiple {
            self.driver.find_all(By::Css(&selector)).await?
        } else {
            vec![self.driver.find(By::Css(&selector)).await?]
        };
        
        let mut text_items = Vec::new();
        let mut total_length = 0;
        
        for (index, element) in elements.iter().enumerate() {
            // Respect max_items limit
            if input.config.max_items > 0 && text_items.len() >= input.config.max_items {
                break;
            }
            
            // Extract text from element
            let extracted_text = self.extract_from_element(element, &input.extraction_type, input.config.clean_text).await?;
            
            // Apply filters
            if !self.apply_filters(&extracted_text, &input.filters) {
                continue;
            }
            
            // Apply length constraints
            let final_text = match self.apply_length_constraints(&extracted_text, input.min_length, input.max_length) {
                Some(text) => text,
                None => continue, // Skip if doesn't meet length requirements
            };
            
            // Get element information
            let tag_name = element.tag_name().await.unwrap_or_else(|_| "unknown".to_string());
            
            // Get attributes if requested
            let attributes = if input.config.include_metadata {
                // Get common attributes
                let mut attrs = HashMap::new();
                if let Ok(Some(id)) = element.attr("id").await {
                    attrs.insert("id".to_string(), id);
                }
                if let Ok(Some(class)) = element.attr("class").await {
                    attrs.insert("class".to_string(), class);
                }
                Some(attrs)
            } else {
                None
            };
            
            let text_item = TextItem {
                text: final_text.clone(),
                selector: if input.extract_multiple {
                    format!("{}:nth-child({})", selector, index + 1)
                } else {
                    selector.clone()
                },
                tag_name,
                attributes,
                length: final_text.len(),
                processed: input.config.clean_text,
            };
            
            total_length += text_item.length;
            text_items.push(text_item);
        }
        
        // Create extraction metadata
        let metadata = if input.config.include_metadata {
            let mut tool_metadata = HashMap::new();
            tool_metadata.insert("extraction_type".to_string(), serde_json::to_value(&input.extraction_type)?);
            tool_metadata.insert("selector".to_string(), serde_json::Value::String(selector.clone()));
            tool_metadata.insert("extract_multiple".to_string(), serde_json::Value::Bool(input.extract_multiple));
            
            Some(ExtractionMetadata {
                url: self.driver.current_url().await?.to_string(),
                timestamp: Utc::now(),
                item_count: text_items.len(),
                duration_ms: start_time.elapsed().as_millis() as u64,
                scope: input.config.scope.clone(),
                tool_name: self.name().to_string(),
                tool_metadata,
            })
        } else {
            None
        };
        
        // Create result structure for formatting
        let result_data = ExtractionResult::success(text_items.clone(), metadata.clone());
        
        // Format output
        let formatted_output = format_utils::format_output(&result_data, &input.config.format)
            .map_err(|e| anyhow::anyhow!("Failed to format output: {}", e))?;
        
        Ok(ExtractTextOutput {
            items: text_items,
            formatted_output,
            total_count: result_data.data.len(),
            total_length,
            config: input.config,
        })
    }
}

// Implementation checklist for extract_text tool:
// [x] Define input/output structures with comprehensive configuration
// [x] Implement TextExtractionType enum with multiple extraction modes
// [x] Add support for CSS selectors and multiple element extraction
// [x] Implement text cleaning and processing utilities
// [x] Add filtering capabilities (text filters, length constraints)
// [x] Support multiple output formats (JSON, text, etc.)
// [x] Add extraction metadata and timing information
// [x] Implement proper error handling and validation
// [x] Add comprehensive attribute and element information extraction
// [ ] Add CLI integration in main.rs
// [ ] Create unit tests and integration tests
// [ ] Add output formatting for CSV, HTML, and Markdown formats

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_extraction_type_parsing() {
        assert_eq!("innertext".parse::<TextExtractionType>().unwrap(), TextExtractionType::InnerText);
        assert_eq!("textcontent".parse::<TextExtractionType>().unwrap(), TextExtractionType::TextContent);
        assert_eq!("attr:href".parse::<TextExtractionType>().unwrap(), TextExtractionType::Attribute("href".to_string()));
        assert!("invalid".parse::<TextExtractionType>().is_err());
    }
    
    #[test]
    fn test_default_input() {
        let input = ExtractTextInput::default();
        assert_eq!(input.extraction_type, TextExtractionType::InnerText);
        assert!(!input.extract_multiple);
        assert!(input.selector.is_none());
    }
    
    #[test]
    fn test_text_item_creation() {
        let item = TextItem {
            text: "Hello world".to_string(),
            selector: "h1".to_string(),
            tag_name: "h1".to_string(),
            attributes: None,
            length: 11,
            processed: true,
        };
        
        assert_eq!(item.text, "Hello world");
        assert_eq!(item.length, 11);
        assert!(item.processed);
    }
}