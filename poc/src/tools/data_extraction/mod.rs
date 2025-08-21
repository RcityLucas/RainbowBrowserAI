// Data Extraction Tools Module
// Phase 2 Implementation - Tools 8-13

// Re-export all data extraction tools (will be implemented progressively)
pub use extract_text::*;
pub use extract_data::*;         // Week 6
pub use extract_table::*;        // Week 7
pub use extract_form::*;         // Week 8 - COMPLETED
pub use extract_links::*;        // Week 8 - COMPLETED
// pub use extract_images::*;   // Future Phase 3

// Module declarations
pub mod extract_text;
pub mod extract_data;            // Week 6 - Structured data extraction
pub mod extract_table;           // Week 7 - Table data extraction
pub mod extract_form;            // Week 8 - Form extraction with validation
pub mod extract_links;           // Week 8 - Link analysis and categorization
// pub mod extract_images;      // TODO: Implement in Phase 3

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Comprehensive information about a DOM element
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementInfo {
    // Basic properties
    pub tag_name: String,
    pub id: Option<String>,
    pub classes: Vec<String>,
    
    // Content
    pub text_content: String,
    pub inner_text: String,
    pub inner_html: String,
    pub outer_html: String,
    
    // Position and dimensions
    pub bounding_box: BoundingBox,
    pub is_visible: bool,
    pub is_in_viewport: bool,
    
    // Attributes
    pub attributes: HashMap<String, String>,
    pub dataset: HashMap<String, String>,
    
    // Computed styles (optional)
    pub computed_style: Option<HashMap<String, String>>,
    
    // Relationships
    pub parent_selector: Option<String>,
    pub child_count: usize,
    pub sibling_count: usize,
}

/// Bounding box information for an element
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

/// Screenshot formats supported
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    Png,
    Jpeg,
    Webp,
}

/// Screenshot types
#[derive(Debug, Clone, PartialEq)]
pub enum ScreenshotType {
    /// Current viewport only
    Viewport,
    /// Entire scrollable page
    FullPage,
    /// Specific element
    Element(String),
    /// Custom region
    Region { x: u32, y: u32, width: u32, height: u32 },
}

/// History entry types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HistoryType {
    Navigation,
    Action,
    Perception,
    Error,
}

// ====================== Phase 2 Data Extraction Framework ======================

/// Output format for data extraction tools
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    /// Plain text format
    Text,
    /// JSON format (structured)
    Json,
    /// CSV format (tabular data)
    Csv,
    /// HTML format (preserves markup)
    Html,
    /// Markdown format
    Markdown,
}

impl Default for OutputFormat {
    fn default() -> Self {
        OutputFormat::Json
    }
}

impl std::str::FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "text" => Ok(OutputFormat::Text),
            "json" => Ok(OutputFormat::Json),
            "csv" => Ok(OutputFormat::Csv),
            "html" => Ok(OutputFormat::Html),
            "markdown" | "md" => Ok(OutputFormat::Markdown),
            _ => Err(format!("Invalid output format: '{}'. Valid formats: text, json, csv, html, markdown", s))
        }
    }
}

/// Extraction scope for data extraction tools
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExtractionScope {
    /// Extract from entire page
    Page,
    /// Extract from specific element(s) using selector
    Element(String),
    /// Extract from visible viewport only
    Viewport,
    /// Extract from a specific region (x, y, width, height)
    Region { x: i32, y: i32, width: u32, height: u32 },
}

impl Default for ExtractionScope {
    fn default() -> Self {
        ExtractionScope::Page
    }
}

/// Common configuration for all data extraction tools
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionConfig {
    /// Output format for the extracted data
    pub format: OutputFormat,
    
    /// Extraction scope
    pub scope: ExtractionScope,
    
    /// Whether to include metadata in the output
    pub include_metadata: bool,
    
    /// Whether to clean/normalize extracted text
    pub clean_text: bool,
    
    /// Maximum number of items to extract (0 = no limit)
    pub max_items: usize,
    
    /// Custom extraction options
    pub options: HashMap<String, serde_json::Value>,
}

impl Default for ExtractionConfig {
    fn default() -> Self {
        Self {
            format: OutputFormat::default(),
            scope: ExtractionScope::default(),
            include_metadata: false,
            clean_text: true,
            max_items: 0, // No limit
            options: HashMap::new(),
        }
    }
}

/// Metadata for extracted data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionMetadata {
    /// URL from which data was extracted
    pub url: String,
    
    /// Timestamp of extraction
    pub timestamp: chrono::DateTime<chrono::Utc>,
    
    /// Number of items extracted
    pub item_count: usize,
    
    /// Extraction duration in milliseconds
    pub duration_ms: u64,
    
    /// Extraction scope used
    pub scope: ExtractionScope,
    
    /// Tool that performed the extraction
    pub tool_name: String,
    
    /// Additional tool-specific metadata
    pub tool_metadata: HashMap<String, serde_json::Value>,
}

/// Common result structure for data extraction tools
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionResult<T> {
    /// The extracted data
    pub data: T,
    
    /// Metadata about the extraction process
    pub metadata: Option<ExtractionMetadata>,
    
    /// Success status
    pub success: bool,
    
    /// Error message if extraction failed
    pub error_message: Option<String>,
}

impl<T> ExtractionResult<T> {
    /// Create a successful extraction result
    pub fn success(data: T, metadata: Option<ExtractionMetadata>) -> Self {
        Self {
            data,
            metadata,
            success: true,
            error_message: None,
        }
    }
    
    /// Create a failed extraction result
    pub fn failure(data: T, error: String, metadata: Option<ExtractionMetadata>) -> Self {
        Self {
            data,
            metadata,
            success: false,
            error_message: Some(error),
        }
    }
}

/// Text cleaning utilities
pub mod text_utils {
    use regex::Regex;
    use lazy_static::lazy_static;
    
    lazy_static! {
        static ref WHITESPACE_REGEX: Regex = Regex::new(r"\s+").unwrap();
        static ref HTML_TAG_REGEX: Regex = Regex::new(r"<[^>]*>").unwrap();
        static ref SCRIPT_STYLE_REGEX: Regex = Regex::new(r"(?i)<(script|style)[^>]*>.*?</\1>").unwrap();
    }
    
    /// Clean extracted text by normalizing whitespace and removing unwanted content
    pub fn clean_text(text: &str) -> String {
        // Remove script and style tags with their content
        let text = SCRIPT_STYLE_REGEX.replace_all(text, "");
        
        // Remove HTML tags
        let text = HTML_TAG_REGEX.replace_all(&text, "");
        
        // Normalize whitespace
        let text = WHITESPACE_REGEX.replace_all(&text, " ");
        
        // Trim and return
        text.trim().to_string()
    }
    
    /// Extract visible text from HTML content
    pub fn extract_visible_text(html: &str) -> String {
        clean_text(html)
    }
    
    /// Truncate text to a maximum length
    pub fn truncate_text(text: &str, max_length: usize) -> String {
        if text.len() <= max_length {
            text.to_string()
        } else {
            format!("{}...", &text[..max_length.saturating_sub(3)])
        }
    }
}

/// Output formatting utilities
pub mod format_utils {
    use super::*;
    use serde_json;
    
    /// Convert data to specified output format
    pub fn format_output<T: Serialize>(data: &T, format: &OutputFormat) -> Result<String, serde_json::Error> {
        match format {
            OutputFormat::Json => serde_json::to_string_pretty(data),
            OutputFormat::Text => {
                // For text format, try to serialize to JSON first, then extract text
                let json_value = serde_json::to_value(data)?;
                Ok(extract_text_from_json(&json_value))
            }
            OutputFormat::Csv => {
                // CSV formatting will be implemented per tool based on data structure
                serde_json::to_string(data)
            }
            OutputFormat::Html => {
                // HTML formatting will be implemented per tool
                serde_json::to_string(data)
            }
            OutputFormat::Markdown => {
                // Markdown formatting will be implemented per tool
                serde_json::to_string(data)
            }
        }
    }
    
    fn extract_text_from_json(value: &serde_json::Value) -> String {
        match value {
            serde_json::Value::String(s) => s.clone(),
            serde_json::Value::Array(arr) => {
                arr.iter()
                    .map(extract_text_from_json)
                    .collect::<Vec<_>>()
                    .join("\n")
            }
            serde_json::Value::Object(obj) => {
                obj.values()
                    .map(extract_text_from_json)
                    .collect::<Vec<_>>()
                    .join(" ")
            }
            _ => format!("{}", value),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::text_utils::*;
    
    #[test]
    fn test_output_format_parsing() {
        assert_eq!("json".parse::<OutputFormat>().unwrap(), OutputFormat::Json);
        assert_eq!("text".parse::<OutputFormat>().unwrap(), OutputFormat::Text);
        assert_eq!("csv".parse::<OutputFormat>().unwrap(), OutputFormat::Csv);
        assert_eq!("html".parse::<OutputFormat>().unwrap(), OutputFormat::Html);
        assert_eq!("markdown".parse::<OutputFormat>().unwrap(), OutputFormat::Markdown);
        assert_eq!("md".parse::<OutputFormat>().unwrap(), OutputFormat::Markdown);
    }
    
    #[test]
    fn test_text_cleaning() {
        let html = "<p>Hello <strong>world</strong>!</p><script>alert('test');</script>";
        let cleaned = clean_text(html);
        assert_eq!(cleaned, "Hello world!");
    }
    
    #[test]
    fn test_text_truncation() {
        let text = "This is a very long text that should be truncated";
        let truncated = truncate_text(text, 20);
        assert_eq!(truncated, "This is a very lo...");
    }
}