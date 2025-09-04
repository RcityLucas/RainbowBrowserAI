//! Action Mapping Engine
//! 
//! Bridges natural language instructions to executable browser actions

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, warn};

use crate::instruction_parser::{UserInstruction, Intent};
use crate::semantic_analyzer::SemanticPageModel;
use crate::browser::SimpleBrowser;

/// Action to be executed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutableAction {
    pub action_type: ActionType,
    pub target: Option<String>,
    pub value: Option<String>,
    pub options: ActionOptions,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionType {
    Navigate,
    Click,
    Type,
    Select,
    Scroll,
    Wait,
    Extract,
    Screenshot,
    Back,
    Forward,
    Refresh,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ActionOptions {
    pub timeout_ms: u64,
    pub wait_after_ms: u64,
    pub retry_count: u32,
    pub validate_result: bool,
    pub take_screenshot: bool,
    pub scroll_into_view: bool,
}

/// Action execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionResult {
    pub success: bool,
    pub action: ExecutableAction,
    pub execution_time_ms: u64,
    pub data: Option<serde_json::Value>,
    pub error: Option<String>,
    pub screenshot_path: Option<String>,
}

/// Maps instructions to executable actions
pub struct ActionMapper {
    element_resolver: ElementResolver,
    action_validator: ActionValidator,
}

impl ActionMapper {
    pub fn new() -> Self {
        Self {
            element_resolver: ElementResolver::new(),
            action_validator: ActionValidator::new(),
        }
    }
    
    /// Map user instruction to executable action
    pub fn map_instruction(
        &self,
        instruction: &UserInstruction,
        page_model: &SemanticPageModel,
    ) -> Result<ExecutableAction> {
        let action = match &instruction.intent {
            Intent::Navigate { target, .. } => {
                self.map_navigation(target, instruction)
            }
            Intent::Click { target_description, .. } => {
                self.map_click(target_description, page_model, instruction)
            }
            Intent::Type { text, target, clear_first } => {
                self.map_type(text, target.as_deref(), *clear_first, page_model, instruction)
            }
            Intent::Select { option, dropdown } => {
                self.map_select(option, dropdown.as_deref(), page_model, instruction)
            }
            Intent::Extract { data_type, filters } => {
                self.map_extract(data_type, filters, instruction)
            }
            Intent::Search { query, scope } => {
                self.map_search(query, scope, page_model, instruction)
            }
            Intent::Wait { condition, timeout } => {
                self.map_wait(condition, *timeout, instruction)
            }
            Intent::Screenshot { area, filename } => {
                self.map_screenshot(area, filename.as_deref(), instruction)
            }
            Intent::Scroll { direction, amount } => {
                self.map_scroll(direction, *amount, instruction)
            }
            _ => {
                Err(anyhow::anyhow!("Unmapped intent type"))
            }
        }?;
        
        // Validate the action
        self.action_validator.validate(&action, page_model)?;
        
        Ok(action)
    }
    
    fn map_navigation(
        &self,
        target: &crate::instruction_parser::intent_recognizer::NavigationTarget,
        instruction: &UserInstruction,
    ) -> Result<ExecutableAction> {
        use crate::instruction_parser::intent_recognizer::NavigationTarget;
        
        match target {
            NavigationTarget::Url(url) => {
                Ok(ExecutableAction {
                    action_type: ActionType::Navigate,
                    target: Some(url.clone()),
                    value: None,
                    options: ActionOptions {
                        timeout_ms: 30000,
                        wait_after_ms: 1000,
                        retry_count: 1,
                        validate_result: true,
                        take_screenshot: false,
                        scroll_into_view: false,
                    },
                    confidence: instruction.confidence,
                })
            }
            NavigationTarget::Back => {
                Ok(ExecutableAction {
                    action_type: ActionType::Back,
                    target: None,
                    value: None,
                    options: Default::default(),
                    confidence: 0.95,
                })
            }
            NavigationTarget::Forward => {
                Ok(ExecutableAction {
                    action_type: ActionType::Forward,
                    target: None,
                    value: None,
                    options: Default::default(),
                    confidence: 0.95,
                })
            }
            NavigationTarget::Refresh => {
                Ok(ExecutableAction {
                    action_type: ActionType::Refresh,
                    target: None,
                    value: None,
                    options: Default::default(),
                    confidence: 0.95,
                })
            }
        }
    }
    
    fn map_click(
        &self,
        target_description: &str,
        page_model: &SemanticPageModel,
        instruction: &UserInstruction,
    ) -> Result<ExecutableAction> {
        // Resolve target element
        let selector = self.element_resolver.resolve_element(
            target_description,
            page_model,
            &instruction.entities,
        )?;
        
        Ok(ExecutableAction {
            action_type: ActionType::Click,
            target: Some(selector),
            value: None,
            options: ActionOptions {
                timeout_ms: 10000,
                wait_after_ms: 500,
                retry_count: 2,
                validate_result: true,
                take_screenshot: false,
                scroll_into_view: true,
            },
            confidence: instruction.confidence * 0.9, // Slightly reduce confidence for element resolution
        })
    }
    
    fn map_type(
        &self,
        text: &str,
        target: Option<&str>,
        clear_first: bool,
        page_model: &SemanticPageModel,
        instruction: &UserInstruction,
    ) -> Result<ExecutableAction> {
        let selector = if let Some(target) = target {
            self.element_resolver.resolve_element(
                target,
                page_model,
                &instruction.entities,
            )?
        } else {
            // Try to find the most likely input field
            self.element_resolver.find_active_input(page_model)?
        };
        
        Ok(ExecutableAction {
            action_type: ActionType::Type,
            target: Some(selector),
            value: Some(text.to_string()),
            options: ActionOptions {
                timeout_ms: 10000,
                wait_after_ms: 100,
                retry_count: 1,
                validate_result: true,
                take_screenshot: false,
                scroll_into_view: true,
            },
            confidence: instruction.confidence * 0.85,
        })
    }
    
    fn map_select(
        &self,
        option: &str,
        dropdown: Option<&str>,
        page_model: &SemanticPageModel,
        instruction: &UserInstruction,
    ) -> Result<ExecutableAction> {
        let selector = if let Some(dropdown) = dropdown {
            self.element_resolver.resolve_element(
                dropdown,
                page_model,
                &instruction.entities,
            )?
        } else {
            self.element_resolver.find_select_element(page_model)?
        };
        
        Ok(ExecutableAction {
            action_type: ActionType::Select,
            target: Some(selector),
            value: Some(option.to_string()),
            options: ActionOptions {
                timeout_ms: 10000,
                wait_after_ms: 200,
                retry_count: 1,
                validate_result: true,
                take_screenshot: false,
                scroll_into_view: true,
            },
            confidence: instruction.confidence * 0.85,
        })
    }
    
    fn map_extract(
        &self,
        data_type: &crate::instruction_parser::intent_recognizer::DataType,
        filters: &[crate::instruction_parser::intent_recognizer::Filter],
        instruction: &UserInstruction,
    ) -> Result<ExecutableAction> {
        let extraction_spec = serde_json::json!({
            "data_type": data_type,
            "filters": filters,
        });
        
        Ok(ExecutableAction {
            action_type: ActionType::Extract,
            target: None,
            value: Some(extraction_spec.to_string()),
            options: ActionOptions {
                timeout_ms: 30000,
                wait_after_ms: 0,
                retry_count: 1,
                validate_result: true,
                take_screenshot: false,
                scroll_into_view: false,
            },
            confidence: instruction.confidence,
        })
    }
    
    fn map_search(
        &self,
        query: &str,
        scope: &crate::instruction_parser::intent_recognizer::SearchScope,
        page_model: &SemanticPageModel,
        instruction: &UserInstruction,
    ) -> Result<ExecutableAction> {
        // Find search input
        let search_selector = self.element_resolver.find_search_input(page_model)?;
        
        // This would be a compound action - type in search box then submit
        Ok(ExecutableAction {
            action_type: ActionType::Type,
            target: Some(search_selector),
            value: Some(query.to_string()),
            options: ActionOptions {
                timeout_ms: 10000,
                wait_after_ms: 500,
                retry_count: 1,
                validate_result: true,
                take_screenshot: false,
                scroll_into_view: true,
            },
            confidence: instruction.confidence * 0.9,
        })
    }
    
    fn map_wait(
        &self,
        condition: &crate::instruction_parser::intent_recognizer::WaitCondition,
        timeout: Option<u64>,
        instruction: &UserInstruction,
    ) -> Result<ExecutableAction> {
        let wait_spec = serde_json::json!({
            "condition": condition,
        });
        
        Ok(ExecutableAction {
            action_type: ActionType::Wait,
            target: None,
            value: Some(wait_spec.to_string()),
            options: ActionOptions {
                timeout_ms: timeout.unwrap_or(30000),
                wait_after_ms: 0,
                retry_count: 0,
                validate_result: false,
                take_screenshot: false,
                scroll_into_view: false,
            },
            confidence: 0.95,
        })
    }
    
    fn map_screenshot(
        &self,
        area: &crate::instruction_parser::intent_recognizer::ScreenshotArea,
        filename: Option<&str>,
        instruction: &UserInstruction,
    ) -> Result<ExecutableAction> {
        let screenshot_spec = serde_json::json!({
            "area": area,
            "filename": filename,
        });
        
        Ok(ExecutableAction {
            action_type: ActionType::Screenshot,
            target: None,
            value: Some(screenshot_spec.to_string()),
            options: ActionOptions {
                timeout_ms: 5000,
                wait_after_ms: 0,
                retry_count: 1,
                validate_result: true,
                take_screenshot: true,
                scroll_into_view: false,
            },
            confidence: 0.99,
        })
    }
    
    fn map_scroll(
        &self,
        direction: &crate::instruction_parser::intent_recognizer::ScrollDirection,
        amount: Option<i32>,
        instruction: &UserInstruction,
    ) -> Result<ExecutableAction> {
        let scroll_spec = serde_json::json!({
            "direction": direction,
            "amount": amount.unwrap_or(500),
        });
        
        Ok(ExecutableAction {
            action_type: ActionType::Scroll,
            target: None,
            value: Some(scroll_spec.to_string()),
            options: ActionOptions {
                timeout_ms: 2000,
                wait_after_ms: 200,
                retry_count: 0,
                validate_result: false,
                take_screenshot: false,
                scroll_into_view: false,
            },
            confidence: 0.95,
        })
    }
}

/// Resolves element descriptions to CSS selectors
struct ElementResolver {
    selector_patterns: HashMap<String, String>,
}

impl ElementResolver {
    fn new() -> Self {
        let mut patterns = HashMap::new();
        
        // Common button patterns
        patterns.insert("submit".to_string(), "button[type='submit'], input[type='submit']".to_string());
        patterns.insert("cancel".to_string(), "button:contains('Cancel'), button.cancel".to_string());
        patterns.insert("close".to_string(), "button.close, [aria-label='Close']".to_string());
        
        // Common input patterns
        patterns.insert("search".to_string(), "input[type='search'], input[placeholder*='search']".to_string());
        patterns.insert("email".to_string(), "input[type='email'], input[name='email']".to_string());
        patterns.insert("password".to_string(), "input[type='password']".to_string());
        
        Self {
            selector_patterns: patterns,
        }
    }
    
    fn resolve_element(
        &self,
        description: &str,
        page_model: &SemanticPageModel,
        entities: &[crate::instruction_parser::entity_extractor::Entity],
    ) -> Result<String> {
        // Check for exact selector in entities
        for entity in entities {
            if matches!(entity.entity_type, crate::instruction_parser::entity_extractor::EntityType::Selector) {
                return Ok(entity.value.clone());
            }
        }
        
        // Check patterns
        let lower = description.to_lowercase();
        for (pattern, selector) in &self.selector_patterns {
            if lower.contains(pattern) {
                return Ok(selector.clone());
            }
        }
        
        // Search in semantic elements
        for element in &page_model.semantic_elements {
            if element.content.to_lowercase().contains(&lower) {
                return Ok(element.selector.clone());
            }
        }
        
        // Search in interaction points
        for point in &page_model.interaction_points {
            if point.selector.contains(description) {
                return Ok(point.selector.clone());
            }
        }
        
        // Fallback: try to construct selector
        Ok(self.construct_selector(description))
    }
    
    fn find_active_input(&self, page_model: &SemanticPageModel) -> Result<String> {
        // Look for focused input
        for element in &page_model.semantic_elements {
            if matches!(element.element_type, crate::semantic_analyzer::ElementType::Input) {
                return Ok(element.selector.clone());
            }
        }
        
        // Look for form inputs
        for region in &page_model.regions {
            if let crate::semantic_analyzer::PageRegion::Form { fields, .. } = region {
                if !fields.is_empty() {
                    return Ok(format!("input[name='{}']", fields[0].name));
                }
            }
        }
        
        // Fallback
        Ok("input:visible:first".to_string())
    }
    
    fn find_select_element(&self, page_model: &SemanticPageModel) -> Result<String> {
        for element in &page_model.semantic_elements {
            if matches!(element.element_type, crate::semantic_analyzer::ElementType::Dropdown) {
                return Ok(element.selector.clone());
            }
        }
        
        Ok("select:visible:first".to_string())
    }
    
    fn find_search_input(&self, page_model: &SemanticPageModel) -> Result<String> {
        // Look for search region
        for region in &page_model.regions {
            if let crate::semantic_analyzer::PageRegion::SearchBar { input_selector, .. } = region {
                return Ok(input_selector.clone());
            }
        }
        
        // Look for search-like inputs
        for element in &page_model.semantic_elements {
            if matches!(element.element_type, crate::semantic_analyzer::ElementType::Input) {
                if element.attributes.get("type").map(|t| t == "search").unwrap_or(false) {
                    return Ok(element.selector.clone());
                }
                if let Some(placeholder) = element.attributes.get("placeholder") {
                    if placeholder.to_lowercase().contains("search") {
                        return Ok(element.selector.clone());
                    }
                }
            }
        }
        
        Ok("input[type='search'], input[placeholder*='search']".to_string())
    }
    
    fn construct_selector(&self, description: &str) -> String {
        // Try to construct a selector from the description
        if description.starts_with('#') || description.starts_with('.') {
            description.to_string()
        } else {
            // Use attribute selector as fallback
            format!("[aria-label*='{}'], [title*='{}'], :contains('{}')", 
                description, description, description)
        }
    }
}

/// Validates actions before execution
struct ActionValidator {
}

impl ActionValidator {
    fn new() -> Self {
        Self {}
    }
    
    fn validate(&self, action: &ExecutableAction, page_model: &SemanticPageModel) -> Result<()> {
        match action.action_type {
            ActionType::Click | ActionType::Type | ActionType::Select => {
                if action.target.is_none() {
                    return Err(anyhow::anyhow!("Target element not specified"));
                }
                
                // Verify element exists in page model
                let target = action.target.as_ref().unwrap();
                let exists = page_model.semantic_elements.iter()
                    .any(|e| e.selector == *target || target.contains(&e.selector));
                
                if !exists && !target.starts_with('#') && !target.starts_with('.') {
                    warn!("Target element '{}' not found in page model", target);
                    // Don't fail, as it might be dynamically loaded
                }
            }
            ActionType::Navigate => {
                if let Some(url) = &action.target {
                    // Basic URL validation
                    if !url.starts_with("http") && !url.starts_with("https") {
                        return Err(anyhow::anyhow!("Invalid URL: {}", url));
                    }
                }
            }
            _ => {
                // Other action types don't need validation
            }
        }
        
        Ok(())
    }
}

/// Action executor
pub struct ActionExecutor<'a> {
    browser: &'a SimpleBrowser,
}

impl<'a> ActionExecutor<'a> {
    pub fn new(browser: &'a SimpleBrowser) -> Self {
        Self { browser }
    }
    
    pub async fn execute(&self, action: &ExecutableAction) -> Result<ActionResult> {
        let start = std::time::Instant::now();
        let mut result = ActionResult {
            success: false,
            action: action.clone(),
            execution_time_ms: 0,
            data: None,
            error: None,
            screenshot_path: None,
        };
        
        // Execute with retries
        let mut attempts = 0;
        let max_attempts = action.options.retry_count + 1;
        
        while attempts < max_attempts {
            match self.execute_action_internal(action).await {
                Ok(data) => {
                    result.success = true;
                    result.data = data;
                    break;
                }
                Err(e) => {
                    result.error = Some(e.to_string());
                    attempts += 1;
                    if attempts < max_attempts {
                        debug!("Retrying action after error: {}", e);
                        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                    }
                }
            }
        }
        
        // Take screenshot if requested
        if action.options.take_screenshot && result.success {
            if self.browser.take_screenshot("action_result").await.is_ok() {
                result.screenshot_path = Some("action_result.png".to_string());
            }
        }
        
        result.execution_time_ms = start.elapsed().as_millis() as u64;
        Ok(result)
    }
    
    async fn execute_action_internal(&self, action: &ExecutableAction) -> Result<Option<serde_json::Value>> {
        match action.action_type {
            ActionType::Navigate => {
                if let Some(url) = &action.target {
                    self.browser.navigate_to(url).await?;
                    Ok(Some(serde_json::json!({
                        "url": url,
                        "title": self.browser.get_title().await.ok()
                    })))
                } else {
                    Err(anyhow::anyhow!("No URL specified"))
                }
            }
            ActionType::Click => {
                if let Some(selector) = &action.target {
                    // Scroll into view if requested
                    if action.options.scroll_into_view {
                        self.browser.execute_script(
                            &format!("document.querySelector('{}').scrollIntoView({{behavior: 'smooth', block: 'center'}})", selector),
                            vec![]
                        ).await?;
                        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                    }
                    
                    let element = self.browser.find_element(selector).await?;
                    element.click().await?;
                    
                    // Wait after click
                    if action.options.wait_after_ms > 0 {
                        tokio::time::sleep(tokio::time::Duration::from_millis(action.options.wait_after_ms)).await;
                    }
                    
                    Ok(Some(serde_json::json!({
                        "clicked": selector
                    })))
                } else {
                    Err(anyhow::anyhow!("No selector specified"))
                }
            }
            ActionType::Type => {
                if let (Some(selector), Some(text)) = (&action.target, &action.value) {
                    // Scroll into view if requested
                    if action.options.scroll_into_view {
                        self.browser.execute_script(
                            &format!("document.querySelector('{}').scrollIntoView({{behavior: 'smooth', block: 'center'}})", selector),
                            vec![]
                        ).await?;
                        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                    }
                    
                    let element = self.browser.find_element(selector).await?;
                    
                    // Clear field first if it's an input
                    element.clear().await?;
                    
                    // Type text
                    element.send_keys(text).await?;
                    
                    // Wait after typing
                    if action.options.wait_after_ms > 0 {
                        tokio::time::sleep(tokio::time::Duration::from_millis(action.options.wait_after_ms)).await;
                    }
                    
                    Ok(Some(serde_json::json!({
                        "typed": text,
                        "in": selector
                    })))
                } else {
                    Err(anyhow::anyhow!("Missing selector or text"))
                }
            }
            ActionType::Screenshot => {
                let path = self.browser.take_screenshot("user_requested").await?;
                Ok(Some(serde_json::json!({
                    "screenshot": path
                })))
            }
            ActionType::Back => {
                self.browser.execute_script("window.history.back()", vec![]).await?;
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                Ok(Some(serde_json::json!({"navigated": "back"})))
            }
            ActionType::Forward => {
                self.browser.execute_script("window.history.forward()", vec![]).await?;
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                Ok(Some(serde_json::json!({"navigated": "forward"})))
            }
            ActionType::Refresh => {
                self.browser.execute_script("window.location.reload()", vec![]).await?;
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                Ok(Some(serde_json::json!({"refreshed": true})))
            }
            _ => {
                Err(anyhow::anyhow!("Action type not yet implemented: {:?}", action.action_type))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_action_creation() {
        let action = ExecutableAction {
            action_type: ActionType::Click,
            target: Some("#submit".to_string()),
            value: None,
            options: ActionOptions::default(),
            confidence: 0.9,
        };
        
        assert!(matches!(action.action_type, ActionType::Click));
        assert_eq!(action.target, Some("#submit".to_string()));
    }
}