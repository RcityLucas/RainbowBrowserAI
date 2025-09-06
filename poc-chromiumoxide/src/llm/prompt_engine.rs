// Prompt Engineering Module
// Creates context-aware prompts for different browser automation tasks

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, debug};

use super::LLMError;

/// Main prompt engine for generating LLM prompts
#[derive(Debug, Clone)]
pub struct PromptEngine {
    templates: HashMap<String, PromptTemplate>,
    context_builders: HashMap<String, ContextBuilder>,
}

/// Template for generating prompts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptTemplate {
    pub name: String,
    pub template: String,
    pub variables: Vec<String>,
    pub description: String,
    pub version: String,
}

/// Context-aware prompt with dynamic content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextAwarePrompt {
    pub base_template: String,
    pub context_data: HashMap<String, serde_json::Value>,
    pub final_prompt: String,
    pub estimated_tokens: Option<u32>,
}

/// Builds context information for prompts
#[derive(Debug, Clone)]
pub struct ContextBuilder {
    name: String,
    builder_fn: fn(&HashMap<String, serde_json::Value>) -> Result<String, LLMError>,
}

impl PromptEngine {
    /// Create new prompt engine with default templates
    pub fn new() -> Self {
        let mut engine = Self {
            templates: HashMap::new(),
            context_builders: HashMap::new(),
        };
        
        engine.load_default_templates();
        engine.load_default_context_builders();
        engine
    }
    
    /// Load default prompt templates
    fn load_default_templates(&mut self) {
        // Task planning template
        self.add_template(PromptTemplate {
            name: "task_planning".to_string(),
            template: r#"You are an expert browser automation assistant. Given a user instruction, create a detailed step-by-step plan for browser automation.

Context Information:
{context}

User Instruction: {instruction}

Create a JSON response with the following structure:
{{
  "id": "unique_task_id",
  "description": "Brief description of the task",
  "steps": [
    {{
      "id": "step_1",
      "step_type": "Navigate|Click|Type|Extract|Wait|Scroll|Screenshot|Validate|Custom",
      "action": "action_name",
      "parameters": {{}},
      "description": "What this step does",
      "expected_outcome": "Expected result",
      "timeout_seconds": 30,
      "retry_count": 2,
      "dependencies": []
    }}
  ],
  "estimated_duration": 60,
  "confidence": 0.9
}}

Important guidelines:
1. Break complex tasks into simple, atomic steps
2. Include proper error handling and retries
3. Use realistic timeouts and dependencies
4. Provide clear descriptions for each step
5. Only use step types from the provided list

Plan:"#.to_string(),
            variables: vec!["context".to_string(), "instruction".to_string()],
            description: "Creates detailed task plans from natural language instructions".to_string(),
            version: "1.0".to_string(),
        });
        
        // Element identification template
        self.add_template(PromptTemplate {
            name: "element_identification".to_string(),
            template: r#"You are an expert at identifying web page elements. Given page information and a target description, identify the best element to interact with.

Page Context:
{page_context}

Target Element Description: {target_description}

Available Elements:
{elements}

Respond with JSON:
{{
  "selected_element": {{
    "selector": "CSS selector or XPath",
    "element_type": "button|input|link|text|image|select|etc",
    "confidence": 0.95,
    "reasoning": "Why this element was chosen",
    "alternative_selectors": ["backup_selector1", "backup_selector2"]
  }},
  "interaction_strategy": {{
    "primary_action": "click|type|hover|select|etc",
    "pre_actions": ["wait_for_element", "scroll_into_view"],
    "post_actions": ["wait_for_response"],
    "expected_result": "What should happen after interaction"
  }}
}}

Element:"#.to_string(),
            variables: vec!["page_context".to_string(), "target_description".to_string(), "elements".to_string()],
            description: "Identifies web elements based on descriptions and context".to_string(),
            version: "1.0".to_string(),
        });
        
        // Data extraction template
        self.add_template(PromptTemplate {
            name: "data_extraction".to_string(),
            template: r#"You are an expert at extracting structured data from web pages. Analyze the page content and extract the requested information.

Page URL: {page_url}
Extraction Request: {extraction_request}

Page Content:
{page_content}

Extract the requested data and return as JSON:
{{
  "extracted_data": {{
    // Structure based on request
  }},
  "extraction_metadata": {{
    "confidence": 0.95,
    "data_quality": "high|medium|low",
    "missing_fields": [],
    "extraction_method": "dom|text_parsing|pattern_matching",
    "timestamp": "ISO timestamp"
  }},
  "alternative_data": {{
    // Backup data if primary extraction uncertain
  }}
}}

Data:"#.to_string(),
            variables: vec!["page_url".to_string(), "extraction_request".to_string(), "page_content".to_string()],
            description: "Extracts structured data from web page content".to_string(),
            version: "1.0".to_string(),
        });
        
        // Page analysis template
        self.add_template(PromptTemplate {
            name: "page_analysis".to_string(),
            template: r#"You are an expert at analyzing web pages for automation opportunities. Analyze the page and provide insights.

Page URL: {page_url}
Analysis Focus: {focus}

Page Information:
{page_info}

Provide analysis as JSON:
{{
  "page_summary": {{
    "page_type": "e-commerce|search|form|article|dashboard|etc",
    "main_purpose": "Description of page purpose",
    "key_elements": ["element1", "element2"],
    "automation_complexity": "low|medium|high"
  }},
  "actionable_elements": [
    {{
      "element_type": "button|link|input|etc",
      "description": "What this element does",
      "selector": "CSS selector",
      "automation_priority": "high|medium|low"
    }}
  ],
  "potential_issues": [
    {{
      "issue": "Dynamic content loading",
      "severity": "high|medium|low",
      "mitigation": "Wait strategies or alternative approaches"
    }}
  ],
  "automation_recommendations": [
    "Specific recommendations for automation"
  ]
}}

Analysis:"#.to_string(),
            variables: vec!["page_url".to_string(), "focus".to_string(), "page_info".to_string()],
            description: "Analyzes web pages for automation opportunities and challenges".to_string(),
            version: "1.0".to_string(),
        });
        
        // Error analysis template
        self.add_template(PromptTemplate {
            name: "error_analysis".to_string(),
            template: r#"You are an expert at diagnosing browser automation errors. Analyze the error and provide solutions.

Error Context:
- Action: {failed_action}
- Error: {error_message}
- Page State: {page_state}

Previous Actions:
{action_history}

Provide error analysis as JSON:
{{
  "error_diagnosis": {{
    "error_type": "element_not_found|timeout|navigation|javascript|network|etc",
    "root_cause": "Detailed explanation of what went wrong",
    "confidence": 0.9
  }},
  "solutions": [
    {{
      "solution": "Specific solution description",
      "implementation": "How to implement this solution",
      "success_probability": 0.8,
      "estimated_effort": "low|medium|high"
    }}
  ],
  "prevention": [
    "How to prevent this error in the future"
  ],
  "alternative_approaches": [
    "Different ways to accomplish the same goal"
  ]
}}

Analysis:"#.to_string(),
            variables: vec![
                "failed_action".to_string(),
                "error_message".to_string(), 
                "page_state".to_string(),
                "action_history".to_string()
            ],
            description: "Analyzes automation errors and provides solutions".to_string(),
            version: "1.0".to_string(),
        });
    }
    
    /// Load default context builders
    fn load_default_context_builders(&mut self) {
        self.context_builders.insert("browser".to_string(), ContextBuilder {
            name: "browser".to_string(),
            builder_fn: |context| {
                let mut browser_context = String::new();
                
                if let Some(url) = context.get("current_url") {
                    browser_context.push_str(&format!("Current URL: {}\n", url));
                }
                
                if let Some(title) = context.get("page_title") {
                    browser_context.push_str(&format!("Page Title: {}\n", title));
                }
                
                if let Some(viewport) = context.get("viewport_size") {
                    browser_context.push_str(&format!("Viewport: {}\n", viewport));
                }
                
                Ok(browser_context)
            }
        });
        
        self.context_builders.insert("page_elements".to_string(), ContextBuilder {
            name: "page_elements".to_string(),
            builder_fn: |context| {
                let mut elements_context = String::new();
                
                if let Some(elements) = context.get("visible_elements") {
                    elements_context.push_str("Visible Elements:\n");
                    if let Some(elements_array) = elements.as_array() {
                        for element in elements_array {
                            elements_context.push_str(&format!("- {}\n", element));
                        }
                    }
                }
                
                if let Some(forms) = context.get("forms") {
                    elements_context.push_str("Forms:\n");
                    if let Some(forms_array) = forms.as_array() {
                        for form in forms_array {
                            elements_context.push_str(&format!("- {}\n", form));
                        }
                    }
                }
                
                Ok(elements_context)
            }
        });
    }
    
    /// Add a new prompt template
    pub fn add_template(&mut self, template: PromptTemplate) {
        info!("Adding prompt template: {}", template.name);
        self.templates.insert(template.name.clone(), template);
    }
    
    /// Create task planning prompt
    pub fn create_task_planning_prompt(
        &self, 
        instruction: &str, 
        context: &HashMap<String, serde_json::Value>
    ) -> Result<ContextAwarePrompt, LLMError> {
        self.create_prompt("task_planning", &[
            ("instruction", serde_json::Value::String(instruction.to_string())),
            ("context", serde_json::Value::String(self.build_context_string(context)?)),
        ])
    }
    
    /// Create element identification prompt
    pub fn create_element_identification_prompt(
        &self,
        target_description: &str,
        page_context: &HashMap<String, serde_json::Value>,
        elements: &[serde_json::Value],
    ) -> Result<ContextAwarePrompt, LLMError> {
        self.create_prompt("element_identification", &[
            ("target_description", serde_json::Value::String(target_description.to_string())),
            ("page_context", serde_json::Value::String(self.build_context_string(page_context)?)),
            ("elements", serde_json::Value::Array(elements.to_vec())),
        ])
    }
    
    /// Create data extraction prompt
    pub fn create_data_extraction_prompt(
        &self,
        page_url: &str,
        extraction_request: &str,
        page_content: &str,
    ) -> Result<ContextAwarePrompt, LLMError> {
        self.create_prompt("data_extraction", &[
            ("page_url", serde_json::Value::String(page_url.to_string())),
            ("extraction_request", serde_json::Value::String(extraction_request.to_string())),
            ("page_content", serde_json::Value::String(page_content.to_string())),
        ])
    }
    
    /// Create page analysis prompt
    pub fn create_page_analysis_prompt(
        &self,
        page_url: &str,
        focus: &str,
        page_info: &HashMap<String, serde_json::Value>,
    ) -> Result<ContextAwarePrompt, LLMError> {
        self.create_prompt("page_analysis", &[
            ("page_url", serde_json::Value::String(page_url.to_string())),
            ("focus", serde_json::Value::String(focus.to_string())),
            ("page_info", serde_json::Value::String(self.build_context_string(page_info)?)),
        ])
    }
    
    /// Create error analysis prompt
    pub fn create_error_analysis_prompt(
        &self,
        failed_action: &str,
        error_message: &str,
        page_state: &str,
        action_history: &[String],
    ) -> Result<ContextAwarePrompt, LLMError> {
        let history_text = action_history.join("\n");
        
        self.create_prompt("error_analysis", &[
            ("failed_action", serde_json::Value::String(failed_action.to_string())),
            ("error_message", serde_json::Value::String(error_message.to_string())),
            ("page_state", serde_json::Value::String(page_state.to_string())),
            ("action_history", serde_json::Value::String(history_text)),
        ])
    }
    
    /// Create prompt from template
    pub fn create_prompt(
        &self, 
        template_name: &str, 
        variables: &[(&str, serde_json::Value)]
    ) -> Result<ContextAwarePrompt, LLMError> {
        let template = self.templates.get(template_name)
            .ok_or_else(|| LLMError::ConfigError(format!("Template not found: {}", template_name)))?;
        
        let mut final_prompt = template.template.clone();
        let mut context_data = HashMap::new();
        
        // Replace variables in template
        for (key, value) in variables {
            let placeholder = format!("{{{}}}", key);
            let value_str = match value {
                serde_json::Value::String(s) => s.clone(),
                v => v.to_string(),
            };
            
            final_prompt = final_prompt.replace(&placeholder, &value_str);
            context_data.insert(key.to_string(), value.clone());
        }
        
        // Estimate token count (rough approximation)
        let estimated_tokens = Some(self.estimate_tokens(&final_prompt));
        
        debug!("Created prompt from template '{}', estimated {} tokens", template_name, estimated_tokens.unwrap_or(0));
        
        Ok(ContextAwarePrompt {
            base_template: template.template.clone(),
            context_data,
            final_prompt,
            estimated_tokens,
        })
    }
    
    /// Build context string from context data
    fn build_context_string(&self, context: &HashMap<String, serde_json::Value>) -> Result<String, LLMError> {
        let mut context_str = String::new();
        
        // Apply context builders
        for (builder_name, builder) in &self.context_builders {
            match (builder.builder_fn)(context) {
                Ok(section) => {
                    if !section.trim().is_empty() {
                        context_str.push_str(&format!("{}:\n{}\n", builder_name, section));
                    }
                }
                Err(e) => {
                    debug!("Context builder '{}' failed: {}", builder_name, e);
                }
            }
        }
        
        // Add raw context data
        for (key, value) in context {
            if !self.context_builders.contains_key(key) {
                context_str.push_str(&format!("{}: {}\n", key, value));
            }
        }
        
        Ok(context_str)
    }
    
    /// Rough token count estimation (4 characters ≈ 1 token)
    fn estimate_tokens(&self, text: &str) -> u32 {
        (text.len() / 4) as u32
    }
    
    /// Get available template names
    pub fn get_template_names(&self) -> Vec<String> {
        self.templates.keys().cloned().collect()
    }
    
    /// Get template by name
    pub fn get_template(&self, name: &str) -> Option<&PromptTemplate> {
        self.templates.get(name)
    }
    
    /// Load templates from JSON file
    pub fn load_templates_from_file(&mut self, file_path: &str) -> Result<(), LLMError> {
        let content = std::fs::read_to_string(file_path)
            .map_err(|e| LLMError::ConfigError(format!("Failed to read template file: {}", e)))?;
            
        let templates: Vec<PromptTemplate> = serde_json::from_str(&content)
            .map_err(|e| LLMError::ConfigError(format!("Invalid template JSON: {}", e)))?;
            
        for template in templates {
            self.add_template(template);
        }
        
        Ok(())
    }
    
    /// Save templates to JSON file
    pub fn save_templates_to_file(&self, file_path: &str) -> Result<(), LLMError> {
        let templates: Vec<_> = self.templates.values().collect();
        let json = serde_json::to_string_pretty(&templates)
            .map_err(|e| LLMError::ConfigError(format!("Failed to serialize templates: {}", e)))?;
            
        std::fs::write(file_path, json)
            .map_err(|e| LLMError::ConfigError(format!("Failed to write template file: {}", e)))?;
            
        Ok(())
    }
}

impl Default for PromptEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_prompt_engine_creation() {
        let engine = PromptEngine::new();
        let template_names = engine.get_template_names();
        
        assert!(!template_names.is_empty());
        assert!(template_names.contains(&"task_planning".to_string()));
        assert!(template_names.contains(&"element_identification".to_string()));
    }
    
    #[test]
    fn test_task_planning_prompt() {
        let engine = PromptEngine::new();
        let mut context = HashMap::new();
        context.insert("current_url".to_string(), serde_json::Value::String("https://example.com".to_string()));
        
        let prompt = engine.create_task_planning_prompt(
            "Click the login button", 
            &context
        ).unwrap();
        
        assert!(prompt.final_prompt.contains("Click the login button"));
        assert!(prompt.final_prompt.contains("https://example.com"));
        assert!(prompt.estimated_tokens.is_some());
    }
    
    #[test]
    fn test_element_identification_prompt() {
        let engine = PromptEngine::new();
        let mut context = HashMap::new();
        context.insert("page_title".to_string(), serde_json::Value::String("Login Page".to_string()));
        
        let elements = vec![
            serde_json::Value::String("button#login".to_string()),
            serde_json::Value::String("input[type=email]".to_string()),
        ];
        
        let prompt = engine.create_element_identification_prompt(
            "login button",
            &context,
            &elements
        ).unwrap();
        
        assert!(prompt.final_prompt.contains("login button"));
        assert!(prompt.final_prompt.contains("Login Page"));
        assert!(prompt.final_prompt.contains("button#login"));
    }
    
    #[test]
    fn test_token_estimation() {
        let engine = PromptEngine::new();
        let text = "This is a test string for token estimation";
        let estimated = engine.estimate_tokens(text);
        
        // Rough estimate: 44 characters / 4 = 11 tokens
        assert!(estimated >= 10 && estimated <= 15);
    }
    
    #[test]
    fn test_custom_template() {
        let mut engine = PromptEngine::new();
        
        let custom_template = PromptTemplate {
            name: "custom_test".to_string(),
            template: "Hello {name}, welcome to {site}!".to_string(),
            variables: vec!["name".to_string(), "site".to_string()],
            description: "Test template".to_string(),
            version: "1.0".to_string(),
        };
        
        engine.add_template(custom_template);
        
        let prompt = engine.create_prompt("custom_test", &[
            ("name", serde_json::Value::String("Alice".to_string())),
            ("site", serde_json::Value::String("TestSite".to_string())),
        ]).unwrap();
        
        assert_eq!(prompt.final_prompt, "Hello Alice, welcome to TestSite!");
    }
}