// Extract Data Tool - Phase 2 Week 6 Implementation
// 
// This tool extracts structured data from web pages with advanced selectors,
// data transformation, template-based extraction, and validation capabilities.

use crate::tools::{Tool, ToolError};
use super::{OutputFormat, ExtractionScope, ExtractionConfig, ExtractionResult, ExtractionMetadata, text_utils, format_utils};
use std::sync::Arc;
use std::collections::HashMap;
use thirtyfour::{WebDriver, By};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::time::{Duration, Instant};
use chrono::{Utc, DateTime, NaiveDateTime};
use regex::Regex;
use url::Url;

/// Data transformation types for extracted values
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DataType {
    /// Raw string (no transformation)
    String,
    /// Integer number
    Integer,
    /// Floating point number
    Float,
    /// Boolean value
    Boolean,
    /// Date/time value
    DateTime,
    /// URL value
    Url,
    /// Email address
    Email,
    /// Phone number
    Phone,
    /// JSON object
    Json,
    /// Auto-detect type based on content
    Auto,
}

impl Default for DataType {
    fn default() -> Self {
        DataType::String
    }
}

impl std::str::FromStr for DataType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "string" => Ok(DataType::String),
            "integer" | "int" => Ok(DataType::Integer),
            "float" | "number" => Ok(DataType::Float),
            "boolean" | "bool" => Ok(DataType::Boolean),
            "datetime" | "date" => Ok(DataType::DateTime),
            "url" => Ok(DataType::Url),
            "email" => Ok(DataType::Email),
            "phone" => Ok(DataType::Phone),
            "json" => Ok(DataType::Json),
            "auto" => Ok(DataType::Auto),
            _ => Err(format!("Invalid data type: '{}'. Valid types: string, integer, float, boolean, datetime, url, email, phone, json, auto", s))
        }
    }
}

/// Selector type for element selection
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SelectorType {
    /// CSS selector
    Css(String),
    /// XPath selector
    XPath(String),
    /// Combination of multiple selectors
    Multiple(Vec<SelectorType>),
}

impl std::str::FromStr for SelectorType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with("//") || s.starts_with("xpath:") {
            let xpath = s.strip_prefix("xpath:").unwrap_or(s);
            Ok(SelectorType::XPath(xpath.to_string()))
        } else if s.starts_with("css:") {
            let css = s.strip_prefix("css:").unwrap();
            Ok(SelectorType::Css(css.to_string()))
        } else {
            // Default to CSS selector
            Ok(SelectorType::Css(s.to_string()))
        }
    }
}

/// Data extraction field definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataField {
    /// Field name (key in output)
    pub name: String,
    
    /// Selector to find the data
    pub selector: SelectorType,
    
    /// Data transformation type
    pub data_type: DataType,
    
    /// Attribute to extract from (optional, defaults to text content)
    pub attribute: Option<String>,
    
    /// Whether this field is required (extraction fails if not found)
    pub required: bool,
    
    /// Default value if field is not found or invalid
    pub default_value: Option<serde_json::Value>,
    
    /// Validation regex pattern (optional)
    pub validation_pattern: Option<String>,
    
    /// Custom transformation function name (optional)
    pub transform_function: Option<String>,
}

impl Default for DataField {
    fn default() -> Self {
        Self {
            name: String::new(),
            selector: SelectorType::Css("body".to_string()),
            data_type: DataType::default(),
            attribute: None,
            required: false,
            default_value: None,
            validation_pattern: None,
            transform_function: None,
        }
    }
}

/// Template for structured data extraction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionTemplate {
    /// Template name
    pub name: String,
    
    /// Template description
    pub description: Option<String>,
    
    /// Fields to extract
    pub fields: Vec<DataField>,
    
    /// Root selector (optional - restricts extraction to specific elements)
    pub root_selector: Option<SelectorType>,
    
    /// Whether to extract multiple records or single record
    pub extract_multiple: bool,
    
    /// Maximum number of records to extract
    pub max_records: Option<usize>,
}

/// Input parameters for extract_data tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractDataInput {
    /// Extraction template defining the data structure
    pub template: ExtractionTemplate,
    
    /// Extraction configuration
    pub config: ExtractionConfig,
    
    /// Whether to validate extracted data against schema
    pub validate_data: bool,
    
    /// Whether to skip invalid records or fail extraction
    pub skip_invalid: bool,
}

/// Individual extracted data record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRecord {
    /// Extracted field values
    pub data: HashMap<String, serde_json::Value>,
    
    /// Record validation status
    pub valid: bool,
    
    /// Validation errors (if any)
    pub validation_errors: Vec<String>,
    
    /// Source element selector
    pub source_selector: Option<String>,
    
    /// Record index (for multiple records)
    pub index: usize,
}

/// Output from extract_data tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractDataOutput {
    /// Extracted data records
    pub records: Vec<DataRecord>,
    
    /// Formatted output in requested format
    pub formatted_output: String,
    
    /// Total number of records extracted
    pub total_count: usize,
    
    /// Number of valid records
    pub valid_count: usize,
    
    /// Number of invalid records
    pub invalid_count: usize,
    
    /// Template used for extraction
    pub template: ExtractionTemplate,
    
    /// Extraction configuration used
    pub config: ExtractionConfig,
}

/// Extract data tool implementation
pub struct ExtractData {
    driver: Arc<WebDriver>,
}

impl ExtractData {
    /// Create a new extract data tool
    pub fn new(driver: Arc<WebDriver>) -> Self {
        Self { driver }
    }
    
    /// Find elements using selector type
    async fn find_elements(&self, selector: &SelectorType) -> anyhow::Result<Vec<thirtyfour::WebElement>> {
        match selector {
            SelectorType::Css(css) => {
                Ok(self.driver.find_all(By::Css(css)).await?)
            }
            SelectorType::XPath(xpath) => {
                Ok(self.driver.find_all(By::XPath(xpath)).await?)
            }
            SelectorType::Multiple(selectors) => {
                let mut all_elements = Vec::new();
                for sel in selectors {
                    let mut elements = self.find_elements(sel).await?;
                    all_elements.append(&mut elements);
                }
                Ok(all_elements)
            }
        }
    }
    
    /// Find single element using selector type
    async fn find_element(&self, selector: &SelectorType) -> anyhow::Result<thirtyfour::WebElement> {
        match selector {
            SelectorType::Css(css) => {
                Ok(self.driver.find(By::Css(css)).await?)
            }
            SelectorType::XPath(xpath) => {
                Ok(self.driver.find(By::XPath(xpath)).await?)
            }
            SelectorType::Multiple(selectors) => {
                // Return first match from multiple selectors
                for sel in selectors {
                    if let Ok(element) = self.find_element(sel).await {
                        return Ok(element);
                    }
                }
                Err(anyhow::anyhow!("No elements found with any of the provided selectors"))
            }
        }
    }
    
    /// Extract raw value from element
    async fn extract_raw_value(&self, element: &thirtyfour::WebElement, field: &DataField) -> anyhow::Result<String> {
        if let Some(attr_name) = &field.attribute {
            Ok(element.attr(attr_name).await?.unwrap_or_default())
        } else {
            Ok(element.text().await?)
        }
    }
    
    /// Transform raw value based on data type
    fn transform_value(&self, raw_value: &str, data_type: &DataType) -> Result<serde_json::Value, String> {
        let cleaned_value = text_utils::clean_text(raw_value);
        
        match data_type {
            DataType::String => Ok(serde_json::Value::String(cleaned_value)),
            DataType::Integer => {
                cleaned_value.parse::<i64>()
                    .map(|n| serde_json::Value::Number(serde_json::Number::from(n)))
                    .map_err(|_| format!("Cannot parse '{}' as integer", cleaned_value))
            }
            DataType::Float => {
                match cleaned_value.parse::<f64>() {
                    Ok(f) => {
                        if let Some(num) = serde_json::Number::from_f64(f) {
                            Ok(serde_json::Value::Number(num))
                        } else {
                            Err(format!("Invalid float value: {}", cleaned_value))
                        }
                    }
                    Err(_) => Err(format!("Cannot parse '{}' as float", cleaned_value))
                }
            }
            DataType::Boolean => {
                match cleaned_value.to_lowercase().as_str() {
                    "true" | "yes" | "1" | "on" | "enabled" => Ok(serde_json::Value::Bool(true)),
                    "false" | "no" | "0" | "off" | "disabled" => Ok(serde_json::Value::Bool(false)),
                    _ => Err(format!("Cannot parse '{}' as boolean", cleaned_value))
                }
            }
            DataType::DateTime => {
                // Try multiple date formats
                let formats = [
                    "%Y-%m-%d %H:%M:%S",
                    "%Y-%m-%d",
                    "%m/%d/%Y",
                    "%d/%m/%Y",
                    "%Y-%m-%dT%H:%M:%S",
                    "%Y-%m-%dT%H:%M:%SZ",
                ];
                
                for format in &formats {
                    if let Ok(dt) = NaiveDateTime::parse_from_str(&cleaned_value, format) {
                        let utc_dt: DateTime<Utc> = DateTime::from_naive_utc_and_offset(dt, Utc);
                        return Ok(serde_json::Value::String(utc_dt.to_rfc3339()));
                    }
                }
                
                Err(format!("Cannot parse '{}' as datetime", cleaned_value))
            }
            DataType::Url => {
                Url::parse(&cleaned_value)
                    .map(|u| serde_json::Value::String(u.to_string()))
                    .map_err(|_| format!("Cannot parse '{}' as URL", cleaned_value))
            }
            DataType::Email => {
                let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
                if email_regex.is_match(&cleaned_value) {
                    Ok(serde_json::Value::String(cleaned_value))
                } else {
                    Err(format!("'{}' is not a valid email address", cleaned_value))
                }
            }
            DataType::Phone => {
                let phone_regex = Regex::new(r"^[\+]?[1-9][\d]{0,15}$").unwrap();
                let digits_only: String = cleaned_value.chars().filter(|c| c.is_ascii_digit() || *c == '+').collect();
                if phone_regex.is_match(&digits_only) {
                    Ok(serde_json::Value::String(digits_only))
                } else {
                    Err(format!("'{}' is not a valid phone number", cleaned_value))
                }
            }
            DataType::Json => {
                serde_json::from_str(&cleaned_value)
                    .map_err(|_| format!("Cannot parse '{}' as JSON", cleaned_value))
            }
            DataType::Auto => {
                // Auto-detect based on content
                if let Ok(value) = self.transform_value(&cleaned_value, &DataType::Integer) {
                    Ok(value)
                } else if let Ok(value) = self.transform_value(&cleaned_value, &DataType::Float) {
                    Ok(value)
                } else if let Ok(value) = self.transform_value(&cleaned_value, &DataType::Boolean) {
                    Ok(value)
                } else if let Ok(value) = self.transform_value(&cleaned_value, &DataType::Url) {
                    Ok(value)
                } else if let Ok(value) = self.transform_value(&cleaned_value, &DataType::Email) {
                    Ok(value)
                } else if let Ok(value) = self.transform_value(&cleaned_value, &DataType::DateTime) {
                    Ok(value)
                } else {
                    Ok(serde_json::Value::String(cleaned_value))
                }
            }
        }
    }
    
    /// Validate field value against pattern
    fn validate_field(&self, value: &serde_json::Value, field: &DataField) -> Vec<String> {
        let mut errors = Vec::new();
        
        if let Some(pattern) = &field.validation_pattern {
            if let Ok(regex) = Regex::new(pattern) {
                let value_str = match value {
                    serde_json::Value::String(s) => s.clone(),
                    _ => value.to_string(),
                };
                
                if !regex.is_match(&value_str) {
                    errors.push(format!("Field '{}' does not match pattern '{}'", field.name, pattern));
                }
            }
        }
        
        errors
    }
    
    /// Extract data from a single element (root)
    async fn extract_single_record(&self, root_element: Option<&thirtyfour::WebElement>, template: &ExtractionTemplate, index: usize) -> DataRecord {
        let mut data = HashMap::new();
        let mut validation_errors = Vec::new();
        let mut record_valid = true;
        
        for field in &template.fields {
            let field_result = async {
                // Find field element relative to root or document
                let field_element = if let Some(root) = root_element {
                    // Find within root element (this is more complex with thirtyfour)
                    // For now, we'll use document-level search
                    // TODO: Implement relative element search
                    match &field.selector {
                        SelectorType::Css(css) => self.find_element(&SelectorType::Css(css.clone())).await,
                        SelectorType::XPath(xpath) => self.find_element(&SelectorType::XPath(xpath.clone())).await,
                        SelectorType::Multiple(selectors) => self.find_element(&SelectorType::Multiple(selectors.clone())).await,
                    }
                } else {
                    self.find_element(&field.selector).await
                };
                
                match field_element {
                    Ok(element) => {
                        // Extract raw value
                        let raw_value = self.extract_raw_value(&element, field).await?;
                        
                        // Transform value
                        let transformed_value = self.transform_value(&raw_value, &field.data_type)
                            .map_err(|e| anyhow::anyhow!(e))?;
                        
                        // Validate value
                        let field_errors = self.validate_field(&transformed_value, field);
                        if !field_errors.is_empty() {
                            validation_errors.extend(field_errors);
                        }
                        
                        Ok(transformed_value)
                    }
                    Err(e) => {
                        if field.required {
                            record_valid = false;
                            validation_errors.push(format!("Required field '{}' not found: {}", field.name, e));
                        }
                        
                        // Use default value if available
                        if let Some(default) = &field.default_value {
                            Ok(default.clone())
                        } else {
                            Ok(serde_json::Value::Null)
                        }
                    }
                }
            }.await;
            
            match field_result {
                Ok(value) => {
                    data.insert(field.name.clone(), value);
                }
                Err(e) => {
                    validation_errors.push(format!("Error extracting field '{}': {}", field.name, e));
                    record_valid = false;
                }
            }
        }
        
        DataRecord {
            data,
            valid: record_valid && validation_errors.is_empty(),
            validation_errors,
            source_selector: None, // TODO: Implement source tracking
            index,
        }
    }
}

#[async_trait]
impl Tool for ExtractData {
    type Input = ExtractDataInput;
    type Output = ExtractDataOutput;

    fn name(&self) -> &str {
        "extract_data"
    }

    fn description(&self) -> &str {
        "Extract structured data from web pages with advanced selectors, data transformation, and validation"
    }

    async fn execute(&self, input: Self::Input) -> anyhow::Result<Self::Output> {
        let start_time = Instant::now();
        let template = &input.template;
        
        let mut records = Vec::new();
        
        if template.extract_multiple {
            // Extract multiple records
            let root_elements = if let Some(root_selector) = &template.root_selector {
                self.find_elements(root_selector).await?
            } else {
                vec![] // No root elements, extract single record
            };
            
            if root_elements.is_empty() {
                // No root selector or no elements found, extract single record
                let record = self.extract_single_record(None, template, 0).await;
                records.push(record);
            } else {
                // Extract from each root element
                let max_records = template.max_records.unwrap_or(usize::MAX);
                for (index, root_element) in root_elements.iter().enumerate() {
                    if records.len() >= max_records {
                        break;
                    }
                    
                    let record = self.extract_single_record(Some(root_element), template, index).await;
                    records.push(record);
                }
            }
        } else {
            // Extract single record
            let root_element = if let Some(root_selector) = &template.root_selector {
                Some(self.find_element(root_selector).await?)
            } else {
                None
            };
            
            let record = self.extract_single_record(root_element.as_ref(), template, 0).await;
            records.push(record);
        }
        
        // Filter invalid records if requested
        if !input.skip_invalid {
            records.retain(|r| r.valid || !input.validate_data);
        }
        
        let valid_count = records.iter().filter(|r| r.valid).count();
        let invalid_count = records.len() - valid_count;
        
        // Create extraction metadata
        let metadata = if input.config.include_metadata {
            let mut tool_metadata = HashMap::new();
            tool_metadata.insert("template_name".to_string(), serde_json::Value::String(template.name.clone()));
            tool_metadata.insert("extract_multiple".to_string(), serde_json::Value::Bool(template.extract_multiple));
            tool_metadata.insert("field_count".to_string(), serde_json::Value::Number(serde_json::Number::from(template.fields.len())));
            
            Some(ExtractionMetadata {
                url: self.driver.current_url().await?.to_string(),
                timestamp: Utc::now(),
                item_count: records.len(),
                duration_ms: start_time.elapsed().as_millis() as u64,
                scope: input.config.scope.clone(),
                tool_name: self.name().to_string(),
                tool_metadata,
            })
        } else {
            None
        };
        
        // Create result structure for formatting
        let result_data = ExtractionResult::success(records.clone(), metadata.clone());
        
        // Format output
        let formatted_output = format_utils::format_output(&result_data, &input.config.format)
            .map_err(|e| anyhow::anyhow!("Failed to format output: {}", e))?;
        
        Ok(ExtractDataOutput {
            records,
            formatted_output,
            total_count: result_data.data.len(),
            valid_count,
            invalid_count,
            template: template.clone(),
            config: input.config,
        })
    }
}

// Implementation checklist for extract_data tool:
// [x] Define comprehensive data types with transformation support
// [x] Implement SelectorType enum with CSS and XPath support
// [x] Create DataField structure with validation and defaults
// [x] Implement ExtractionTemplate for structured data definition
// [x] Add data transformation for common types (int, float, bool, date, url, email, phone)
// [x] Implement validation system with regex pattern support
// [x] Add auto-detection of data types based on content
// [x] Support both single and multiple record extraction
// [x] Implement comprehensive error handling and validation
// [ ] Add CLI integration in main.rs
// [ ] Create template loading from JSON/YAML files
// [ ] Add custom transformation functions support
// [ ] Create unit tests and integration tests
// [ ] Add relative element search within root elements

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_data_type_parsing() {
        assert_eq!("string".parse::<DataType>().unwrap(), DataType::String);
        assert_eq!("integer".parse::<DataType>().unwrap(), DataType::Integer);
        assert_eq!("float".parse::<DataType>().unwrap(), DataType::Float);
        assert_eq!("boolean".parse::<DataType>().unwrap(), DataType::Boolean);
        assert_eq!("auto".parse::<DataType>().unwrap(), DataType::Auto);
    }
    
    #[test]
    fn test_selector_type_parsing() {
        assert_eq!("div.class".parse::<SelectorType>().unwrap(), SelectorType::Css("div.class".to_string()));
        assert_eq!("//div[@class='test']".parse::<SelectorType>().unwrap(), SelectorType::XPath("//div[@class='test']".to_string()));
        assert_eq!("xpath://div".parse::<SelectorType>().unwrap(), SelectorType::XPath("//div".to_string()));
    }
    
    #[test]
    fn test_data_transformation() {
        let tool = ExtractData { driver: Arc::new(unsafe { std::mem::zeroed() }) }; // Mock for testing
        
        assert_eq!(tool.transform_value("123", &DataType::Integer).unwrap(), serde_json::Value::Number(serde_json::Number::from(123)));
        assert_eq!(tool.transform_value("true", &DataType::Boolean).unwrap(), serde_json::Value::Bool(true));
        assert_eq!(tool.transform_value("test@example.com", &DataType::Email).unwrap(), serde_json::Value::String("test@example.com".to_string()));
    }
}