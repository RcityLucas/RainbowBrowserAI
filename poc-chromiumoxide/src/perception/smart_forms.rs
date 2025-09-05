// Smart form handling with intelligent field detection and auto-fill capabilities

use anyhow::Result;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// Smart form handler that can intelligently fill forms
pub struct SmartFormHandler {
    field_patterns: HashMap<String, FieldPattern>,
    user_profiles: HashMap<String, UserProfile>,
}

/// Analysis of a form structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartFormAnalysis {
    pub form_type: FormType,
    pub fields: Vec<FormField>,
    pub required_fields: Vec<String>,
    pub submit_elements: Vec<String>,
    pub validation_rules: HashMap<String, ValidationRule>,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FormType {
    Login,
    Registration,
    Contact,
    Checkout,
    Search,
    Profile,
    Payment,
    Address,
    Survey,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormField {
    pub selector: String,
    pub field_type: FieldType,
    pub label: Option<String>,
    pub placeholder: Option<String>,
    pub required: bool,
    pub current_value: Option<String>,
    pub validation: Option<ValidationRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FieldType {
    Text,
    Email,
    Password,
    Phone,
    Name,
    Address,
    City,
    State,
    ZipCode,
    Country,
    CreditCard,
    CVV,
    ExpiryDate,
    Checkbox,
    Radio,
    Select,
    TextArea,
    Date,
    Number,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    pub pattern: Option<String>,
    pub min_length: Option<u32>,
    pub max_length: Option<u32>,
    pub required: bool,
    pub custom_rules: Vec<String>,
}

/// User profile for auto-fill
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub name: String,
    pub personal_info: PersonalInfo,
    pub contact_info: ContactInfo,
    pub address_info: AddressInfo,
    pub preferences: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalInfo {
    pub first_name: String,
    pub last_name: String,
    pub middle_name: Option<String>,
    pub title: Option<String>,
    pub date_of_birth: Option<String>,
    pub gender: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactInfo {
    pub email: String,
    pub phone: String,
    pub alternate_email: Option<String>,
    pub alternate_phone: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressInfo {
    pub street_address: String,
    pub street_address_2: Option<String>,
    pub city: String,
    pub state: String,
    pub zip_code: String,
    pub country: String,
}

/// Result of form filling operation
#[derive(Debug, Serialize)]
pub struct FillResult {
    pub success: bool,
    pub filled_fields: Vec<String>,
    pub failed_fields: Vec<String>,
    pub warnings: Vec<String>,
    pub next_steps: Vec<String>,
}

/// Pattern for recognizing field types
struct FieldPattern {
    labels: Vec<String>,
    names: Vec<String>,
    placeholders: Vec<String>,
    types: Vec<String>,
    field_type: FieldType,
}

impl SmartFormHandler {
    pub fn new() -> Self {
        let mut handler = Self {
            field_patterns: HashMap::new(),
            user_profiles: HashMap::new(),
        };
        
        handler.initialize_field_patterns();
        handler
    }

    /// Analyze a form on the current page
    pub async fn analyze_form(&self, browser: &crate::browser::Browser, form_selector: Option<&str>) -> Result<SmartFormAnalysis> {
        let selector = form_selector.unwrap_or("form");
        
        // Get form structure using JavaScript
        let form_analysis_script = format!(r#"
            const form = document.querySelector('{}');
            if (!form) return null;
            
            const fields = [];
            const inputs = form.querySelectorAll('input, select, textarea');
            
            inputs.forEach(input => {{
                const label = form.querySelector(`label[for="${{input.id}}]`) || 
                             input.closest('label') ||
                             input.previousElementSibling?.tagName === 'LABEL' ? input.previousElementSibling : null;
                
                fields.push({{
                    selector: input.tagName.toLowerCase() + (input.id ? '#' + input.id : ''),
                    type: input.type || input.tagName.toLowerCase(),
                    name: input.name || '',
                    placeholder: input.placeholder || '',
                    label: label?.textContent?.trim() || '',
                    required: input.required || false,
                    value: input.value || ''
                }});
            }});
            
            const submitButtons = form.querySelectorAll('input[type="submit"], button[type="submit"], button:not([type])');
            
            return {{
                fields: fields,
                submitButtons: Array.from(submitButtons).map(btn => btn.tagName.toLowerCase() + (btn.id ? '#' + btn.id : '')),
                action: form.action || '',
                method: form.method || 'get'
            }};
        "#, selector);

        let result = browser.execute_script(&form_analysis_script).await?;
        
        if result.is_null() {
            return Err(anyhow::anyhow!("No form found with selector: {}", selector));
        }

        // Parse the form analysis result
        let form_data: serde_json::Value = result;
        let empty_vec = vec![];
        let fields_data = form_data.get("fields").and_then(|f| f.as_array()).unwrap_or(&empty_vec);
        
        let mut fields = Vec::new();
        let mut required_fields = Vec::new();
        
        for field_data in fields_data {
            let field_type = self.classify_field_type(
                field_data.get("type").and_then(|t| t.as_str()).unwrap_or(""),
                field_data.get("name").and_then(|n| n.as_str()).unwrap_or(""),
                field_data.get("label").and_then(|l| l.as_str()).unwrap_or(""),
                field_data.get("placeholder").and_then(|p| p.as_str()).unwrap_or(""),
            );
            
            let selector = field_data.get("selector").and_then(|s| s.as_str()).unwrap_or("").to_string();
            let required = field_data.get("required").and_then(|r| r.as_bool()).unwrap_or(false);
            
            if required {
                required_fields.push(selector.clone());
            }
            
            fields.push(FormField {
                selector,
                field_type,
                label: field_data.get("label").and_then(|l| l.as_str()).map(|s| s.to_string()),
                placeholder: field_data.get("placeholder").and_then(|p| p.as_str()).map(|s| s.to_string()),
                required,
                current_value: field_data.get("value").and_then(|v| v.as_str()).map(|s| s.to_string()),
                validation: None, // TODO: Implement validation rule detection
            });
        }

        let submit_elements = form_data.get("submitButtons")
            .and_then(|s| s.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str()).map(|s| s.to_string()).collect())
            .unwrap_or_default();

        // Classify form type based on fields
        let form_type = self.classify_form_type(&fields);

        Ok(SmartFormAnalysis {
            form_type,
            fields,
            required_fields,
            submit_elements,
            validation_rules: HashMap::new(),
            confidence: 0.8,
        })
    }

    /// Automatically fill a form using user profile
    pub async fn auto_fill_form(
        &self,
        browser: &crate::browser::Browser,
        form_analysis: &SmartFormAnalysis,
        profile_name: &str,
    ) -> Result<FillResult> {
        let profile = self.user_profiles.get(profile_name)
            .ok_or_else(|| anyhow::anyhow!("User profile not found: {}", profile_name))?;

        let mut filled_fields = Vec::new();
        let mut failed_fields = Vec::new();
        let mut warnings = Vec::new();

        for field in &form_analysis.fields {
            match self.get_fill_value(&field.field_type, profile) {
                Some(value) => {
                    match self.fill_field(browser, &field.selector, &value).await {
                        Ok(_) => {
                            filled_fields.push(field.selector.clone());
                        }
                        Err(e) => {
                            failed_fields.push(field.selector.clone());
                            warnings.push(format!("Failed to fill {}: {}", field.selector, e));
                        }
                    }
                }
                None => {
                    warnings.push(format!("No suitable value found for field type: {:?}", field.field_type));
                }
            }
        }

        let next_steps = if !form_analysis.submit_elements.is_empty() {
            vec!["Click submit button to complete form".to_string()]
        } else {
            vec!["Form filled, but no submit button detected".to_string()]
        };

        Ok(FillResult {
            success: failed_fields.is_empty(),
            filled_fields,
            failed_fields,
            warnings,
            next_steps,
        })
    }

    /// Add or update user profile
    pub fn add_user_profile(&mut self, profile: UserProfile) {
        self.user_profiles.insert(profile.name.clone(), profile);
    }

    /// Get available user profiles
    pub fn get_user_profiles(&self) -> Vec<String> {
        self.user_profiles.keys().cloned().collect()
    }

    // Private helper methods

    fn initialize_field_patterns(&mut self) {
        // Email patterns
        self.field_patterns.insert("email".to_string(), FieldPattern {
            labels: vec!["email", "e-mail", "email address"].iter().map(|s| s.to_string()).collect(),
            names: vec!["email", "emailAddress", "user_email", "mail"].iter().map(|s| s.to_string()).collect(),
            placeholders: vec!["email", "your email", "email address"].iter().map(|s| s.to_string()).collect(),
            types: vec!["email"].iter().map(|s| s.to_string()).collect(),
            field_type: FieldType::Email,
        });

        // Password patterns
        self.field_patterns.insert("password".to_string(), FieldPattern {
            labels: vec!["password", "pass", "pwd"].iter().map(|s| s.to_string()).collect(),
            names: vec!["password", "pass", "pwd", "passwd"].iter().map(|s| s.to_string()).collect(),
            placeholders: vec!["password", "enter password"].iter().map(|s| s.to_string()).collect(),
            types: vec!["password"].iter().map(|s| s.to_string()).collect(),
            field_type: FieldType::Password,
        });

        // Name patterns
        self.field_patterns.insert("name".to_string(), FieldPattern {
            labels: vec!["name", "full name", "your name", "first name", "last name"].iter().map(|s| s.to_string()).collect(),
            names: vec!["name", "fullName", "firstName", "lastName", "fname", "lname"].iter().map(|s| s.to_string()).collect(),
            placeholders: vec!["name", "your name", "full name"].iter().map(|s| s.to_string()).collect(),
            types: vec!["text"].iter().map(|s| s.to_string()).collect(),
            field_type: FieldType::Name,
        });

        // Phone patterns
        self.field_patterns.insert("phone".to_string(), FieldPattern {
            labels: vec!["phone", "telephone", "phone number", "mobile"].iter().map(|s| s.to_string()).collect(),
            names: vec!["phone", "phoneNumber", "telephone", "mobile", "tel"].iter().map(|s| s.to_string()).collect(),
            placeholders: vec!["phone", "phone number", "mobile number"].iter().map(|s| s.to_string()).collect(),
            types: vec!["tel", "phone"].iter().map(|s| s.to_string()).collect(),
            field_type: FieldType::Phone,
        });

        // Add more patterns as needed...
    }

    fn classify_field_type(&self, input_type: &str, name: &str, label: &str, placeholder: &str) -> FieldType {
        let combined_text = format!("{} {} {} {}", input_type, name, label, placeholder).to_lowercase();

        // Check against known patterns
        for (_, pattern) in &self.field_patterns {
            // Check type match
            if pattern.types.iter().any(|t| t == input_type) {
                return pattern.field_type.clone();
            }

            // Check text matches
            if pattern.labels.iter().any(|l| combined_text.contains(l)) ||
               pattern.names.iter().any(|n| combined_text.contains(n)) ||
               pattern.placeholders.iter().any(|p| combined_text.contains(p)) {
                return pattern.field_type.clone();
            }
        }

        // Default classification based on input type
        match input_type {
            "email" => FieldType::Email,
            "password" => FieldType::Password,
            "tel" | "phone" => FieldType::Phone,
            "number" => FieldType::Number,
            "date" => FieldType::Date,
            "checkbox" => FieldType::Checkbox,
            "radio" => FieldType::Radio,
            "select" => FieldType::Select,
            "textarea" => FieldType::TextArea,
            _ => FieldType::Text,
        }
    }

    fn classify_form_type(&self, fields: &[FormField]) -> FormType {
        let has_password = fields.iter().any(|f| matches!(f.field_type, FieldType::Password));
        let has_email = fields.iter().any(|f| matches!(f.field_type, FieldType::Email));
        let has_credit_card = fields.iter().any(|f| matches!(f.field_type, FieldType::CreditCard));
        let has_address = fields.iter().any(|f| matches!(f.field_type, FieldType::Address));

        if has_password && has_email && fields.len() == 2 {
            FormType::Login
        } else if has_password && has_email && fields.len() > 3 {
            FormType::Registration
        } else if has_credit_card {
            FormType::Payment
        } else if has_address {
            FormType::Address
        } else if fields.len() > 5 {
            FormType::Survey
        } else if has_email && !has_password {
            FormType::Contact
        } else {
            FormType::Unknown
        }
    }

    fn get_fill_value(&self, field_type: &FieldType, profile: &UserProfile) -> Option<String> {
        match field_type {
            FieldType::Email => Some(profile.contact_info.email.clone()),
            FieldType::Phone => Some(profile.contact_info.phone.clone()),
            FieldType::Name => Some(format!("{} {}", profile.personal_info.first_name, profile.personal_info.last_name)),
            FieldType::Address => Some(profile.address_info.street_address.clone()),
            FieldType::City => Some(profile.address_info.city.clone()),
            FieldType::State => Some(profile.address_info.state.clone()),
            FieldType::ZipCode => Some(profile.address_info.zip_code.clone()),
            FieldType::Country => Some(profile.address_info.country.clone()),
            _ => None,
        }
    }

    async fn fill_field(&self, browser: &crate::browser::Browser, selector: &str, value: &str) -> Result<()> {
        // Click to focus the field
        browser.click(selector).await?;
        
        // Clear existing content
        let clear_script = format!(r#"
            const element = document.querySelector('{}');
            if (element) {{
                element.select();
                element.value = '';
                element.dispatchEvent(new Event('input', {{ bubbles: true }}));
            }}
        "#, selector);
        browser.execute_script(&clear_script).await?;
        
        // Type the new value
        browser.type_text(selector, value).await?;
        
        Ok(())
    }
}