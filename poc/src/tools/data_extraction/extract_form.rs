// Extract Form Tool - Phase 2 Week 8 Implementation
// 
// This tool specializes in extracting comprehensive form information from web pages with
// intelligent input type detection, form validation analysis, and detailed field metadata.

use crate::tools::{Tool, ToolError};
use super::{OutputFormat, ExtractionScope, ExtractionConfig, ExtractionResult, ExtractionMetadata, text_utils, format_utils};
use std::sync::Arc;
use std::collections::HashMap;
use thirtyfour::{WebDriver, By, WebElement};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::time::{Duration, Instant};
use chrono::Utc;

/// Form field information with comprehensive metadata
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FormField {
    /// Field name/identifier
    pub name: String,
    
    /// Field ID attribute
    pub id: Option<String>,
    
    /// Input type (text, password, email, etc.)
    pub input_type: FormInputType,
    
    /// Field label text
    pub label: Option<String>,
    
    /// Placeholder text
    pub placeholder: Option<String>,
    
    /// Default/current value
    pub value: Option<String>,
    
    /// Whether field is required
    pub required: bool,
    
    /// Whether field is disabled
    pub disabled: bool,
    
    /// Whether field is readonly
    pub readonly: bool,
    
    /// Field validation attributes
    pub validation: FormValidation,
    
    /// Field position in form
    pub position: usize,
    
    /// CSS classes
    pub css_classes: Vec<String>,
    
    /// Field options (for select, radio, checkbox)
    pub options: Vec<FormOption>,
    
    /// Field dimensions and position
    pub bounding_box: Option<FormFieldBounds>,
}

/// Form input types with intelligent detection
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FormInputType {
    // Text inputs
    Text,
    Password,
    Email,
    Tel,
    Url,
    Search,
    
    // Numeric inputs
    Number,
    Range,
    
    // Date/time inputs
    Date,
    Time,
    DateTime,
    DateTimeLocal,
    Month,
    Week,
    
    // Selection inputs
    Select,
    Radio,
    Checkbox,
    
    // File input
    File,
    
    // Button types
    Submit,
    Button,
    Reset,
    
    // Other inputs
    Hidden,
    Color,
    Image,
    
    // Form elements
    Textarea,
    
    // Unknown type
    Unknown(String),
}

impl Default for FormInputType {
    fn default() -> Self {
        FormInputType::Text
    }
}

impl std::str::FromStr for FormInputType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "text" => Ok(FormInputType::Text),
            "password" => Ok(FormInputType::Password),
            "email" => Ok(FormInputType::Email),
            "tel" | "phone" => Ok(FormInputType::Tel),
            "url" => Ok(FormInputType::Url),
            "search" => Ok(FormInputType::Search),
            "number" => Ok(FormInputType::Number),
            "range" => Ok(FormInputType::Range),
            "date" => Ok(FormInputType::Date),
            "time" => Ok(FormInputType::Time),
            "datetime" => Ok(FormInputType::DateTime),
            "datetime-local" => Ok(FormInputType::DateTimeLocal),
            "month" => Ok(FormInputType::Month),
            "week" => Ok(FormInputType::Week),
            "select" | "select-one" | "select-multiple" => Ok(FormInputType::Select),
            "radio" => Ok(FormInputType::Radio),
            "checkbox" => Ok(FormInputType::Checkbox),
            "file" => Ok(FormInputType::File),
            "submit" => Ok(FormInputType::Submit),
            "button" => Ok(FormInputType::Button),
            "reset" => Ok(FormInputType::Reset),
            "hidden" => Ok(FormInputType::Hidden),
            "color" => Ok(FormInputType::Color),
            "image" => Ok(FormInputType::Image),
            "textarea" => Ok(FormInputType::Textarea),
            _ => Ok(FormInputType::Unknown(s.to_string())),
        }
    }
}

/// Form field validation information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FormValidation {
    /// Minimum length
    pub min_length: Option<u32>,
    
    /// Maximum length
    pub max_length: Option<u32>,
    
    /// Minimum value (for numeric inputs)
    pub min: Option<f64>,
    
    /// Maximum value (for numeric inputs)
    pub max: Option<f64>,
    
    /// Step value (for numeric inputs)
    pub step: Option<f64>,
    
    /// Pattern regex
    pub pattern: Option<String>,
    
    /// Custom validation messages
    pub validation_message: Option<String>,
    
    /// Current validation state
    pub validation_state: ValidationState,
}

impl Default for FormValidation {
    fn default() -> Self {
        Self {
            min_length: None,
            max_length: None,
            min: None,
            max: None,
            step: None,
            pattern: None,
            validation_message: None,
            validation_state: ValidationState::Unknown,
        }
    }
}

/// Field validation state
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ValidationState {
    Valid,
    Invalid,
    Unknown,
}

/// Form field option (for select, radio, checkbox)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FormOption {
    /// Option value
    pub value: String,
    
    /// Option display text
    pub text: String,
    
    /// Whether option is selected/checked
    pub selected: bool,
    
    /// Whether option is disabled
    pub disabled: bool,
}

/// Form field bounding box
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FormFieldBounds {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// Complete form information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormInfo {
    /// Form name/ID
    pub name: Option<String>,
    
    /// Form ID
    pub id: Option<String>,
    
    /// Form action URL
    pub action: Option<String>,
    
    /// Form method (GET, POST, etc.)
    pub method: String,
    
    /// Form encoding type
    pub enctype: Option<String>,
    
    /// Form fields
    pub fields: Vec<FormField>,
    
    /// Form CSS classes
    pub css_classes: Vec<String>,
    
    /// Form position on page
    pub form_index: usize,
    
    /// Whether form has validation
    pub has_validation: bool,
    
    /// Form submit buttons
    pub submit_buttons: Vec<FormField>,
}

/// Form extraction configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormExtractionConfig {
    /// Whether to include hidden fields
    pub include_hidden: bool,
    
    /// Whether to include disabled fields
    pub include_disabled: bool,
    
    /// Whether to extract field options (select, radio, checkbox)
    pub extract_options: bool,
    
    /// Whether to analyze validation state
    pub analyze_validation: bool,
    
    /// Whether to include field positioning
    pub include_positioning: bool,
    
    /// Maximum number of options per field to extract
    pub max_options_per_field: usize,
}

impl Default for FormExtractionConfig {
    fn default() -> Self {
        Self {
            include_hidden: false,
            include_disabled: true,
            extract_options: true,
            analyze_validation: true,
            include_positioning: false,
            max_options_per_field: 100,
        }
    }
}

/// Input parameters for extract_form tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractFormInput {
    /// CSS selector for form elements (optional, defaults to "form")
    pub form_selector: Option<String>,
    
    /// Extraction configuration
    pub config: ExtractionConfig,
    
    /// Form-specific configuration
    pub form_config: FormExtractionConfig,
    
    /// Whether to extract from multiple forms or just the first
    pub extract_multiple: bool,
}

impl Default for ExtractFormInput {
    fn default() -> Self {
        Self {
            form_selector: None,
            config: ExtractionConfig::default(),
            form_config: FormExtractionConfig::default(),
            extract_multiple: false,
        }
    }
}

/// Output from extract_form tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractFormOutput {
    /// Extracted form data
    pub forms: Vec<FormInfo>,
    
    /// Formatted output in requested format
    pub formatted_output: String,
    
    /// Total number of forms extracted
    pub form_count: usize,
    
    /// Total number of fields across all forms
    pub total_fields: usize,
    
    /// Extraction configuration used
    pub config: ExtractionConfig,
    
    /// Form configuration used
    pub form_config: FormExtractionConfig,
}

/// Extract form tool implementation
pub struct ExtractForm {
    driver: Arc<WebDriver>,
}

impl ExtractForm {
    /// Create a new extract form tool
    pub fn new(driver: Arc<WebDriver>) -> Self {
        Self { driver }
    }
    
    /// Find form elements using selector
    async fn find_forms(&self, selector: &str) -> anyhow::Result<Vec<WebElement>> {
        Ok(self.driver.find_all(By::Css(selector)).await?)
    }
    
    /// Extract information from a single form
    async fn extract_form_info(&self, form: &WebElement, form_index: usize, input: &ExtractFormInput) -> anyhow::Result<FormInfo> {
        // Extract form metadata
        let name = form.attr("name").await?;
        let id = form.attr("id").await?;
        let action = form.attr("action").await?;
        let method = form.attr("method").await?.unwrap_or_else(|| "GET".to_string()).to_uppercase();
        let enctype = form.attr("enctype").await?;
        let css_classes = self.get_css_classes(form).await?;
        
        // Find all form fields
        let field_elements = form.find_all(By::Css("input, select, textarea, button")).await?;
        let mut fields = Vec::new();
        let mut submit_buttons = Vec::new();
        
        for (position, field_element) in field_elements.iter().enumerate() {
            let field = self.extract_field_info(field_element, position, input).await?;
            
            // Check if this is a submit button
            if matches!(field.input_type, FormInputType::Submit) {
                submit_buttons.push(field.clone());
            }
            
            // Include field based on configuration
            if self.should_include_field(&field, input) {
                fields.push(field);
            }
        }
        
        // Check if form has validation
        let has_validation = fields.iter().any(|f| f.required || f.validation.pattern.is_some() || 
                                                 f.validation.min_length.is_some() || 
                                                 f.validation.max_length.is_some());
        
        Ok(FormInfo {
            name,
            id,
            action,
            method,
            enctype,
            fields,
            css_classes,
            form_index,
            has_validation,
            submit_buttons,
        })
    }
    
    /// Extract information from a single form field
    async fn extract_field_info(&self, element: &WebElement, position: usize, input: &ExtractFormInput) -> anyhow::Result<FormField> {
        let tag_name = element.tag_name().await?;
        let name = element.attr("name").await?.unwrap_or_default();
        let id = element.attr("id").await?;
        let placeholder = element.attr("placeholder").await?;
        let value = element.attr("value").await?;
        let required = element.attr("required").await?.is_some();
        let disabled = element.attr("disabled").await?.is_some();
        let readonly = element.attr("readonly").await?.is_some();
        let css_classes = self.get_css_classes(element).await?;
        
        // Determine input type
        let input_type = self.determine_input_type(element, &tag_name).await?;
        
        // Find associated label
        let label = self.find_field_label(element, &id, &name).await?;
        
        // Extract validation information
        let validation = if input.form_config.analyze_validation {
            self.extract_validation_info(element).await?
        } else {
            FormValidation::default()
        };
        
        // Extract options for select/radio/checkbox
        let options = if input.form_config.extract_options && 
                         matches!(input_type, FormInputType::Select | FormInputType::Radio | FormInputType::Checkbox) {
            self.extract_field_options(element, &input_type, input).await?
        } else {
            Vec::new()
        };
        
        // Get bounding box if requested
        let bounding_box = if input.form_config.include_positioning {
            self.get_field_bounds(element).await?
        } else {
            None
        };
        
        Ok(FormField {
            name,
            id,
            input_type,
            label,
            placeholder,
            value,
            required,
            disabled,
            readonly,
            validation,
            position,
            css_classes,
            options,
            bounding_box,
        })
    }
    
    /// Determine the input type of a form field
    async fn determine_input_type(&self, element: &WebElement, tag_name: &str) -> anyhow::Result<FormInputType> {
        match tag_name.to_lowercase().as_str() {
            "input" => {
                let input_type = element.attr("type").await?.unwrap_or_else(|| "text".to_string());
                Ok(input_type.parse().unwrap_or(FormInputType::Unknown(input_type)))
            }
            "select" => Ok(FormInputType::Select),
            "textarea" => Ok(FormInputType::Textarea),
            "button" => {
                // Check button type
                let button_type = element.attr("type").await?.unwrap_or_else(|| "button".to_string());
                match button_type.as_str() {
                    "submit" => Ok(FormInputType::Submit),
                    "reset" => Ok(FormInputType::Reset),
                    _ => Ok(FormInputType::Button),
                }
            }
            _ => Ok(FormInputType::Unknown(tag_name.to_string())),
        }
    }
    
    /// Find the label associated with a form field
    async fn find_field_label(&self, element: &WebElement, id: &Option<String>, name: &str) -> anyhow::Result<Option<String>> {
        // Try to find label by 'for' attribute
        if let Some(field_id) = id {
            if let Ok(label) = self.driver.find(By::Css(&format!("label[for='{}']", field_id))).await {
                let label_text = text_utils::clean_text(&label.text().await?);
                if !label_text.is_empty() {
                    return Ok(Some(label_text));
                }
            }
        }
        
        // Try to find parent label
        if let Ok(parent) = element.find(By::XPath("ancestor::label[1]")).await {
            let label_text = text_utils::clean_text(&parent.text().await?);
            if !label_text.is_empty() {
                return Ok(Some(label_text));
            }
        }
        
        // Try to find nearby label by proximity (previous sibling)
        if let Ok(sibling) = element.find(By::XPath("preceding-sibling::label[1]")).await {
            let label_text = text_utils::clean_text(&sibling.text().await?);
            if !label_text.is_empty() {
                return Ok(Some(label_text));
            }
        }
        
        // Try to find aria-label
        if let Ok(Some(aria_label)) = element.attr("aria-label").await {
            if !aria_label.trim().is_empty() {
                return Ok(Some(aria_label));
            }
        }
        
        Ok(None)
    }
    
    /// Extract validation information from form field
    async fn extract_validation_info(&self, element: &WebElement) -> anyhow::Result<FormValidation> {
        let min_length = element.attr("minlength").await?.and_then(|s| s.parse().ok());
        let max_length = element.attr("maxlength").await?.and_then(|s| s.parse().ok());
        let min = element.attr("min").await?.and_then(|s| s.parse().ok());
        let max = element.attr("max").await?.and_then(|s| s.parse().ok());
        let step = element.attr("step").await?.and_then(|s| s.parse().ok());
        let pattern = element.attr("pattern").await?;
        
        // Try to get custom validation message
        let validation_message = element.attr("title").await?.or_else(|| {
            // Could also check data-* attributes for custom validation messages
            None
        });
        
        // Determine validation state (basic check)
        let validation_state = if element.attr("aria-invalid").await?.as_deref() == Some("true") {
            ValidationState::Invalid
        } else if element.attr("aria-invalid").await?.as_deref() == Some("false") {
            ValidationState::Valid
        } else {
            ValidationState::Unknown
        };
        
        Ok(FormValidation {
            min_length,
            max_length,
            min,
            max,
            step,
            pattern,
            validation_message,
            validation_state,
        })
    }
    
    /// Extract options from select, radio, or checkbox fields
    async fn extract_field_options(&self, element: &WebElement, input_type: &FormInputType, input: &ExtractFormInput) -> anyhow::Result<Vec<FormOption>> {
        let mut options = Vec::new();
        
        match input_type {
            FormInputType::Select => {
                // Extract options from select element
                let option_elements = element.find_all(By::Css("option")).await?;
                for option_element in option_elements.iter().take(input.form_config.max_options_per_field) {
                    let value = option_element.attr("value").await?.unwrap_or_default();
                    let text = text_utils::clean_text(&option_element.text().await?);
                    let selected = option_element.attr("selected").await?.is_some();
                    let disabled = option_element.attr("disabled").await?.is_some();
                    
                    options.push(FormOption {
                        value,
                        text,
                        selected,
                        disabled,
                    });
                }
            }
            FormInputType::Radio => {
                // Find all radio buttons with the same name
                let name = element.attr("name").await?.unwrap_or_default();
                if !name.is_empty() {
                    let radio_elements = self.driver.find_all(By::Css(&format!("input[type='radio'][name='{}']", name))).await?;
                    for radio_element in radio_elements.iter().take(input.form_config.max_options_per_field) {
                        let value = radio_element.attr("value").await?.unwrap_or_default();
                        let text = self.find_field_label(&radio_element, &radio_element.attr("id").await?, &name).await?
                                       .unwrap_or_else(|| value.clone());
                        let selected = radio_element.is_selected().await.unwrap_or(false);
                        let disabled = radio_element.attr("disabled").await?.is_some();
                        
                        options.push(FormOption {
                            value,
                            text,
                            selected,
                            disabled,
                        });
                    }
                }
            }
            FormInputType::Checkbox => {
                // For checkbox, create a single option representing its state
                let value = element.attr("value").await?.unwrap_or_else(|| "on".to_string());
                let text = self.find_field_label(element, &element.attr("id").await?, &element.attr("name").await?.unwrap_or_default()).await?
                               .unwrap_or_else(|| "Checkbox".to_string());
                let selected = element.is_selected().await.unwrap_or(false);
                let disabled = element.attr("disabled").await?.is_some();
                
                options.push(FormOption {
                    value,
                    text,
                    selected,
                    disabled,
                });
            }
            _ => {} // No options for other input types
        }
        
        Ok(options)
    }
    
    /// Get field bounding box
    async fn get_field_bounds(&self, element: &WebElement) -> anyhow::Result<Option<FormFieldBounds>> {
        if let Ok(rect) = element.rect().await {
            Ok(Some(FormFieldBounds {
                x: rect.x,
                y: rect.y,
                width: rect.width,
                height: rect.height,
            }))
        } else {
            Ok(None)
        }
    }
    
    /// Get CSS classes from element
    async fn get_css_classes(&self, element: &WebElement) -> anyhow::Result<Vec<String>> {
        let class_attr = element.attr("class").await?.unwrap_or_default();
        if class_attr.trim().is_empty() {
            Ok(Vec::new())
        } else {
            Ok(class_attr.split_whitespace().map(|s| s.to_string()).collect())
        }
    }
    
    /// Check if field should be included based on configuration
    fn should_include_field(&self, field: &FormField, input: &ExtractFormInput) -> bool {
        // Check hidden fields
        if !input.form_config.include_hidden && matches!(field.input_type, FormInputType::Hidden) {
            return false;
        }
        
        // Check disabled fields
        if !input.form_config.include_disabled && field.disabled {
            return false;
        }
        
        true
    }
}

#[async_trait]
impl Tool for ExtractForm {
    type Input = ExtractFormInput;
    type Output = ExtractFormOutput;

    fn name(&self) -> &str {
        "extract_form"
    }

    fn description(&self) -> &str {
        "Extract comprehensive form information from web pages with intelligent input type detection and validation analysis"
    }

    async fn execute(&self, input: Self::Input) -> anyhow::Result<Self::Output> {
        let start_time = Instant::now();
        
        // Determine form selector
        let form_selector = input.form_selector
            .as_deref()
            .unwrap_or("form");
        
        // Find forms
        let form_elements = self.find_forms(form_selector).await?;
        
        if form_elements.is_empty() {
            return Err(anyhow::anyhow!("No forms found with selector '{}'", form_selector));
        }
        
        let mut forms = Vec::new();
        let mut total_fields = 0;
        
        // Extract data from forms
        let extract_count = if input.extract_multiple { 
            form_elements.len() 
        } else { 
            1 
        };
        
        for (index, form_element) in form_elements.iter().enumerate().take(extract_count) {
            let form_info = self.extract_form_info(form_element, index, &input).await?;
            total_fields += form_info.fields.len();
            forms.push(form_info);
        }
        
        // Create extraction metadata
        let metadata = if input.config.include_metadata {
            let mut tool_metadata = HashMap::new();
            tool_metadata.insert("form_selector".to_string(), serde_json::Value::String(form_selector.to_string()));
            tool_metadata.insert("extract_multiple".to_string(), serde_json::Value::Bool(input.extract_multiple));
            tool_metadata.insert("include_hidden".to_string(), serde_json::Value::Bool(input.form_config.include_hidden));
            tool_metadata.insert("analyze_validation".to_string(), serde_json::Value::Bool(input.form_config.analyze_validation));
            
            Some(ExtractionMetadata {
                url: self.driver.current_url().await?.to_string(),
                timestamp: Utc::now(),
                item_count: forms.len(),
                duration_ms: start_time.elapsed().as_millis() as u64,
                scope: input.config.scope.clone(),
                tool_name: self.name().to_string(),
                tool_metadata,
            })
        } else {
            None
        };
        
        // Create result structure for formatting
        let result_data = ExtractionResult::success(forms.clone(), metadata.clone());
        
        // Format output
        let formatted_output = format_utils::format_output(&result_data, &input.config.format)
            .map_err(|e| anyhow::anyhow!("Failed to format output: {}", e))?;
        
        Ok(ExtractFormOutput {
            forms,
            formatted_output,
            form_count: result_data.data.len(),
            total_fields,
            config: input.config,
            form_config: input.form_config,
        })
    }
}

// Implementation checklist for extract_form tool:
// [x] Define comprehensive form data structures (FormInfo, FormField, FormValidation)
// [x] Implement form input type detection with intelligent parsing
// [x] Add form field validation analysis and state detection  
// [x] Implement field label detection with multiple strategies
// [x] Create option extraction for select/radio/checkbox fields
// [x] Add form positioning and bounding box information
// [x] Implement comprehensive field filtering based on configuration
// [x] Add form metadata extraction (action, method, enctype)
// [x] Create validation state analysis (valid/invalid/unknown)
// [ ] Add CLI integration in main.rs
// [ ] Create specialized form analysis report formatting
// [ ] Add form security analysis (CSRF tokens, etc.)
// [ ] Create unit tests and integration tests
// [ ] Add form submission preview/validation

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_form_input_type_parsing() {
        assert_eq!("text".parse::<FormInputType>().unwrap(), FormInputType::Text);
        assert_eq!("password".parse::<FormInputType>().unwrap(), FormInputType::Password);
        assert_eq!("email".parse::<FormInputType>().unwrap(), FormInputType::Email);
        assert_eq!("select".parse::<FormInputType>().unwrap(), FormInputType::Select);
        assert_eq!("submit".parse::<FormInputType>().unwrap(), FormInputType::Submit);
    }
    
    #[test]
    fn test_form_validation_default() {
        let validation = FormValidation::default();
        assert_eq!(validation.validation_state, ValidationState::Unknown);
        assert!(validation.pattern.is_none());
        assert!(validation.min_length.is_none());
    }
    
    #[test]
    fn test_form_extraction_config_default() {
        let config = FormExtractionConfig::default();
        assert!(!config.include_hidden);
        assert!(config.include_disabled);
        assert!(config.extract_options);
        assert!(config.analyze_validation);
        assert_eq!(config.max_options_per_field, 100);
    }
    
    #[test]
    fn test_should_include_field_logic() {
        let tool = ExtractForm { driver: Arc::new(unsafe { std::mem::zeroed() }) };
        let input = ExtractFormInput::default();
        
        // Test hidden field exclusion
        let hidden_field = FormField {
            name: "hidden".to_string(),
            input_type: FormInputType::Hidden,
            disabled: false,
            ..Default::default()
        };
        assert!(!tool.should_include_field(&hidden_field, &input));
        
        // Test normal field inclusion
        let text_field = FormField {
            name: "username".to_string(),
            input_type: FormInputType::Text,
            disabled: false,
            ..Default::default()
        };
        assert!(tool.should_include_field(&text_field, &input));
    }
}

impl Default for FormField {
    fn default() -> Self {
        Self {
            name: String::new(),
            id: None,
            input_type: FormInputType::default(),
            label: None,
            placeholder: None,
            value: None,
            required: false,
            disabled: false,
            readonly: false,
            validation: FormValidation::default(),
            position: 0,
            css_classes: Vec::new(),
            options: Vec::new(),
            bounding_box: None,
        }
    }
}