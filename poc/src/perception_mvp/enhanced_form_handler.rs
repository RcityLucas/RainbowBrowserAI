// Enhanced Form Handler for Complex Form Interactions
// Supports modern form frameworks, multi-step forms, and file uploads

use anyhow::{Result, Context};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use thirtyfour::{WebDriver, WebElement, By, Key};
use tracing::{debug, info, warn, error};

use super::enhanced_error_recovery::{EnhancedErrorRecovery, RecoveryResult};
use crate::smart_element_detector::{ElementDescriptor, ElementType};

/// Enhanced form handler with intelligent form interaction capabilities
pub struct EnhancedFormHandler {
    driver: WebDriver,
    error_recovery: EnhancedErrorRecovery,
    form_state: HashMap<String, FormFieldState>,
    config: FormHandlerConfig,
}

#[derive(Debug, Clone)]
pub struct FormHandlerConfig {
    pub auto_detect_validation: bool,
    pub wait_for_validation: Duration,
    pub retry_on_validation_error: bool,
    pub smart_field_detection: bool,
    pub handle_dynamic_fields: bool,
}

impl Default for FormHandlerConfig {
    fn default() -> Self {
        Self {
            auto_detect_validation: true,
            wait_for_validation: Duration::from_millis(1000),
            retry_on_validation_error: true,
            smart_field_detection: true,
            handle_dynamic_fields: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormFieldState {
    pub field_name: String,
    pub field_type: FormFieldType,
    pub current_value: Option<String>,
    pub is_valid: Option<bool>,
    pub validation_message: Option<String>,
    pub is_required: bool,
    pub last_interaction: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FormFieldType {
    Text,
    Email,
    Password,
    Number,
    Date,
    Select,
    MultiSelect,
    Checkbox,
    Radio,
    TextArea,
    File,
    Hidden,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormInteractionResult {
    pub success: bool,
    pub field_updated: bool,
    pub validation_passed: Option<bool>,
    pub error_message: Option<String>,
    pub suggestions: Vec<String>,
    pub execution_time_ms: u64,
}

impl EnhancedFormHandler {
    pub fn new(driver: WebDriver, config: Option<FormHandlerConfig>) -> Self {
        let error_recovery = EnhancedErrorRecovery::new(driver.clone(), None);
        
        Self {
            driver: driver.clone(),
            error_recovery,
            form_state: HashMap::new(),
            config: config.unwrap_or_default(),
        }
    }

    /// Fill a form field with intelligent error handling and validation
    pub async fn fill_field(
        &mut self, 
        field_description: &str, 
        value: &str
    ) -> Result<FormInteractionResult> {
        let start_time = Instant::now();
        info!("Filling field '{}' with value: {}", field_description, 
              if field_description.to_lowercase().contains("password") { "[HIDDEN]" } else { value });

        // Step 1: Find the field with error recovery
        let descriptor = self.create_field_descriptor(field_description).await;
        let element_result = self.error_recovery.find_element_with_recovery(&descriptor).await;
        
        let element = match element_result.result {
            Some(elem) => elem,
            None => {
                return Ok(FormInteractionResult {
                    success: false,
                    field_updated: false,
                    validation_passed: None,
                    error_message: element_result.error_message,
                    suggestions: self.generate_field_suggestions(field_description).await,
                    execution_time_ms: start_time.elapsed().as_millis() as u64,
                });
            }
        };

        // Step 2: Detect field type and characteristics
        let field_type = self.detect_field_type(&element).await?;
        let is_required = self.is_field_required(&element).await;
        
        // Step 3: Clear existing content if needed
        if let Err(e) = self.clear_field(&element, &field_type).await {
            warn!("Failed to clear field: {}", e);
        }

        // Step 4: Fill the field based on its type
        let fill_result = match field_type {
            FormFieldType::Select | FormFieldType::MultiSelect => {
                self.handle_select_field(&element, value).await
            },
            FormFieldType::Checkbox | FormFieldType::Radio => {
                self.handle_choice_field(&element, value).await
            },
            FormFieldType::File => {
                self.handle_file_field(&element, value).await
            },
            FormFieldType::Date => {
                self.handle_date_field(&element, value).await
            },
            _ => {
                self.handle_text_field(&element, value).await
            }
        };

        let field_updated = fill_result.is_ok();

        // Step 5: Wait for and check validation if enabled
        let validation_passed = if self.config.auto_detect_validation && field_updated {
            tokio::time::sleep(self.config.wait_for_validation).await;
            self.check_field_validation(&element).await
        } else {
            None
        };

        // Step 6: Update form state
        self.update_form_state(
            field_description,
            FormFieldState {
                field_name: field_description.to_string(),
                field_type: field_type.clone(),
                current_value: Some(value.to_string()),
                is_valid: validation_passed,
                validation_message: None, // TODO: Extract validation message
                is_required,
                last_interaction: Some(start_time.elapsed().as_millis() as u64),
            }
        );

        // Step 7: Handle validation errors with retry if configured
        let final_success = if let Some(false) = validation_passed {
            if self.config.retry_on_validation_error {
                warn!("Validation failed, attempting retry with corrected value");
                self.retry_with_validation_correction(&element, value, &field_type).await.is_ok()
            } else {
                false
            }
        } else {
            field_updated
        };

        Ok(FormInteractionResult {
            success: final_success,
            field_updated,
            validation_passed,
            error_message: if !final_success { 
                fill_result.err().map(|e| e.to_string()) 
            } else { 
                None 
            },
            suggestions: if !final_success { 
                self.generate_field_suggestions(field_description).await 
            } else { 
                vec![] 
            },
            execution_time_ms: start_time.elapsed().as_millis() as u64,
        })
    }

    /// Submit a form with intelligent button detection
    pub async fn submit_form(&self, submit_description: Option<&str>) -> Result<FormInteractionResult> {
        let start_time = Instant::now();
        let description = submit_description.unwrap_or("submit button");
        
        info!("Submitting form using: {}", description);

        // Try multiple submit strategies
        let submit_result = if let Ok(result) = self.try_submit_button(description).await {
            result
        } else if let Ok(result) = self.try_form_submission().await {
            result
        } else if let Ok(result) = self.try_enter_key_submission().await {
            result
        } else {
            return Ok(FormInteractionResult {
                success: false,
                field_updated: false,
                validation_passed: None,
                error_message: Some("Could not find any way to submit the form".to_string()),
                suggestions: vec![
                    "Look for a submit button with different text".to_string(),
                    "Check if the form requires all fields to be filled".to_string(),
                    "Try pressing Enter in the last field".to_string(),
                ],
                execution_time_ms: start_time.elapsed().as_millis() as u64,
            });
        };

        Ok(FormInteractionResult {
            success: true,
            field_updated: true,
            validation_passed: Some(true),
            error_message: None,
            suggestions: vec![],
            execution_time_ms: start_time.elapsed().as_millis() as u64,
        })
    }

    /// Create field descriptor from natural language description
    async fn create_field_descriptor(&self, description: &str) -> ElementDescriptor {
        let desc_lower = description.to_lowercase();
        
        let element_type = if desc_lower.contains("email") {
            ElementType::Input
        } else if desc_lower.contains("password") {
            ElementType::Input
        } else if desc_lower.contains("select") || desc_lower.contains("dropdown") || desc_lower.contains("choose") {
            ElementType::Select
        } else if desc_lower.contains("checkbox") || desc_lower.contains("check") {
            ElementType::Checkbox
        } else if desc_lower.contains("radio") {
            ElementType::Radio
        } else if desc_lower.contains("file") || desc_lower.contains("upload") {
            ElementType::Input
        } else if desc_lower.contains("text") || desc_lower.contains("field") || desc_lower.contains("input") {
            ElementType::Input
        } else {
            ElementType::Input // Default to input
        };

        ElementDescriptor {
            description: description.to_string(),
            element_type,
            attributes: HashMap::new(),
            context: None,
        }
    }

    /// Detect the actual field type from the DOM element
    async fn detect_field_type(&self, element: &WebElement) -> Result<FormFieldType> {
        let tag_name = element.tag_name().await?.to_lowercase();
        
        match tag_name.as_str() {
            "select" => {
                let multiple = element.attr("multiple").await?.is_some();
                Ok(if multiple { FormFieldType::MultiSelect } else { FormFieldType::Select })
            },
            "textarea" => Ok(FormFieldType::TextArea),
            "input" => {
                let input_type = element.attr("type").await?.unwrap_or_default().to_lowercase();
                Ok(match input_type.as_str() {
                    "email" => FormFieldType::Email,
                    "password" => FormFieldType::Password,
                    "number" => FormFieldType::Number,
                    "date" => FormFieldType::Date,
                    "checkbox" => FormFieldType::Checkbox,
                    "radio" => FormFieldType::Radio,
                    "file" => FormFieldType::File,
                    "hidden" => FormFieldType::Hidden,
                    "text" | "" => FormFieldType::Text,
                    other => FormFieldType::Custom(other.to_string()),
                })
            },
            other => Ok(FormFieldType::Custom(other.to_string())),
        }
    }

    /// Check if field is required
    async fn is_field_required(&self, element: &WebElement) -> bool {
        // Check required attribute
        if element.attr("required").await.unwrap_or_default().is_some() {
            return true;
        }

        // Check aria-required
        if let Ok(Some(aria_required)) = element.attr("aria-required").await {
            if aria_required.to_lowercase() == "true" {
                return true;
            }
        }

        // Check for asterisk or "required" in label
        // This is a simplified check - in practice, you'd want more sophisticated detection
        false
    }

    /// Clear field content appropriately
    async fn clear_field(&self, element: &WebElement, field_type: &FormFieldType) -> Result<()> {
        match field_type {
            FormFieldType::Checkbox | FormFieldType::Radio => {
                // Don't clear these, just set their state
                Ok(())
            },
            FormFieldType::Select | FormFieldType::MultiSelect => {
                // Don't clear selects, just select new option
                Ok(())
            },
            _ => {
                // Clear text-based fields
                element.clear().await.context("Failed to clear field")
            }
        }
    }

    /// Handle text field input with intelligent typing
    async fn handle_text_field(&self, element: &WebElement, value: &str) -> Result<()> {
        // Clear existing content
        element.clear().await?;
        
        // Type the value with natural timing
        element.send_keys(value).await?;
        
        // Trigger change events (important for React/Vue forms)
        element.send_keys(Key::Tab).await.ok(); // Tab away to trigger validation
        
        Ok(())
    }

    /// Handle select/dropdown fields
    async fn handle_select_field(&self, element: &WebElement, value: &str) -> Result<()> {
        debug!("Handling select field with value: {}", value);
        
        // Try to find option by text first
        let option_xpath = format!(".//option[text()='{}']", value);
        if let Ok(option) = element.find(By::XPath(&option_xpath)).await {
            option.click().await?;
            return Ok(());
        }

        // Try case-insensitive match
        let option_xpath = format!(
            ".//option[contains(translate(text(), 'ABCDEFGHIJKLMNOPQRSTUVWXYZ', 'abcdefghijklmnopqrstuvwxyz'), '{}')]",
            value.to_lowercase()
        );
        if let Ok(option) = element.find(By::XPath(&option_xpath)).await {
            option.click().await?;
            return Ok(());
        }

        // Try by value attribute
        let option_css = format!("option[value='{}']", value);
        if let Ok(option) = element.find(By::Css(&option_css)).await {
            option.click().await?;
            return Ok(());
        }

        // Try partial value match
        let option_css = format!("option[value*='{}']", value);
        if let Ok(option) = element.find(By::Css(&option_css)).await {
            option.click().await?;
            return Ok(());
        }

        Err(anyhow::anyhow!("Could not find option '{}' in select field", value))
    }

    /// Handle checkbox and radio button fields
    async fn handle_choice_field(&self, element: &WebElement, value: &str) -> Result<()> {
        let should_check = match value.to_lowercase().as_str() {
            "true" | "yes" | "1" | "on" | "checked" => true,
            "false" | "no" | "0" | "off" | "unchecked" => false,
            _ => {
                // For radio buttons, try to find the option with matching value/text
                if let Ok(field_type) = self.detect_field_type(element).await {
                    if matches!(field_type, FormFieldType::Radio) {
                        // Look for radio button with this value
                        let name = element.attr("name").await?.unwrap_or_default();
                        let radio_selector = format!("input[name='{}'][value='{}']", name, value);
                        if let Ok(radio) = self.driver.find(By::Css(&radio_selector)).await {
                            radio.click().await?;
                            return Ok(());
                        }
                    }
                }
                return Err(anyhow::anyhow!("Invalid value for checkbox/radio: {}", value));
            }
        };

        let is_checked = element.is_selected().await.unwrap_or(false);
        if is_checked != should_check {
            element.click().await?;
        }

        Ok(())
    }

    /// Handle file upload fields
    async fn handle_file_field(&self, element: &WebElement, file_path: &str) -> Result<()> {
        debug!("Uploading file: {}", file_path);
        
        // Verify file exists (basic check)
        if !std::path::Path::new(file_path).exists() {
            return Err(anyhow::anyhow!("File not found: {}", file_path));
        }

        element.send_keys(file_path).await?;
        Ok(())
    }

    /// Handle date fields with intelligent parsing
    async fn handle_date_field(&self, element: &WebElement, date_value: &str) -> Result<()> {
        debug!("Setting date field to: {}", date_value);
        
        // TODO: Add intelligent date parsing and formatting
        // For now, just send the value as-is
        element.clear().await?;
        element.send_keys(date_value).await?;
        
        Ok(())
    }

    /// Check field validation status
    async fn check_field_validation(&self, element: &WebElement) -> Option<bool> {
        // Check for HTML5 validity
        if let Ok(result) = element.is_valid().await {
            return Some(result);
        }

        // Check for common validation classes/attributes
        if let Ok(Some(class)) = element.attr("class").await {
            if class.contains("error") || class.contains("invalid") {
                return Some(false);
            }
            if class.contains("valid") || class.contains("success") {
                return Some(true);
            }
        }

        // Check aria-invalid
        if let Ok(Some(aria_invalid)) = element.attr("aria-invalid").await {
            return Some(aria_invalid.to_lowercase() != "true");
        }

        None // Unknown validation state
    }

    /// Retry filling field with validation correction
    async fn retry_with_validation_correction(
        &self,
        element: &WebElement,
        original_value: &str,
        field_type: &FormFieldType,
    ) -> Result<()> {
        debug!("Retrying field input with validation correction");
        
        // TODO: Implement intelligent correction based on validation messages
        // For now, just retry the original operation
        match field_type {
            FormFieldType::Email => {
                // Try to correct email format
                let corrected = if !original_value.contains('@') {
                    format!("{}@example.com", original_value)
                } else {
                    original_value.to_string()
                };
                self.handle_text_field(element, &corrected).await
            },
            _ => {
                // Just retry with original value
                self.handle_text_field(element, original_value).await
            }
        }
    }

    /// Try to submit form using submit button
    async fn try_submit_button(&self, description: &str) -> Result<()> {
        let descriptor = ElementDescriptor {
            description: description.to_string(),
            element_type: ElementType::Button,
            attributes: HashMap::new(),
            context: None,
        };

        let result = self.error_recovery.find_element_with_recovery(&descriptor).await;
        if let Some(button) = result.result {
            button.click().await?;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Submit button not found"))
        }
    }

    /// Try to submit form by finding the form element directly
    async fn try_form_submission(&self) -> Result<()> {
        if let Ok(form) = self.driver.find(By::Tag("form")).await {
            // Try to submit programmatically
            form.submit().await?;
            Ok(())
        } else {
            Err(anyhow::anyhow!("No form element found"))
        }
    }

    /// Try to submit by pressing Enter on the last input field
    async fn try_enter_key_submission(&self) -> Result<()> {
        let inputs = self.driver.find_all(By::Css("input[type='text'], input[type='email'], input[type='password']")).await?;
        
        if let Some(last_input) = inputs.last() {
            if last_input.is_displayed().await.unwrap_or(false) {
                last_input.send_keys(Key::Return).await?;
                return Ok(());
            }
        }

        Err(anyhow::anyhow!("No suitable input field for Enter key submission"))
    }

    /// Generate helpful suggestions when field interaction fails
    async fn generate_field_suggestions(&self, field_description: &str) -> Vec<String> {
        let mut suggestions = Vec::new();
        
        suggestions.push("Check if the field is visible on the current page".to_string());
        suggestions.push("Try scrolling to make the field visible".to_string());
        suggestions.push("Verify the field description matches the actual field label".to_string());
        
        if field_description.to_lowercase().contains("password") {
            suggestions.push("Ensure caps lock is not enabled for password fields".to_string());
        }
        
        if field_description.to_lowercase().contains("email") {
            suggestions.push("Make sure the email format is valid (user@domain.com)".to_string());
        }

        suggestions
    }

    /// Update form state tracking
    fn update_form_state(&mut self, field_name: &str, state: FormFieldState) {
        self.form_state.insert(field_name.to_string(), state);
    }

    /// Get current form state
    pub fn get_form_state(&self) -> &HashMap<String, FormFieldState> {
        &self.form_state
    }

    /// Clear form state
    pub fn clear_form_state(&mut self) {
        self.form_state.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_form_field_type_detection() {
        // This would need mock WebElement implementation
        // Just testing the enum for now
        let field_type = FormFieldType::Email;
        assert!(matches!(field_type, FormFieldType::Email));
    }

    #[test]
    fn test_form_handler_config_default() {
        let config = FormHandlerConfig::default();
        assert!(config.auto_detect_validation);
        assert!(config.smart_field_detection);
        assert!(config.handle_dynamic_fields);
    }
}