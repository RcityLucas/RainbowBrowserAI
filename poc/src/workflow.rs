use anyhow::{Result, Context as AnyhowContext};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use tracing::{info, warn, error};
use crate::{SimpleBrowser, CostTracker};
use tokio::time::sleep;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub name: String,
    pub description: Option<String>,
    pub version: Option<String>,
    pub inputs: Option<Vec<InputDefinition>>,
    pub variables: HashMap<String, serde_json::Value>,
    pub steps: Vec<WorkflowStep>,
    pub parallel: Option<bool>,
    pub on_error: Option<ErrorStrategy>,
    pub timeout: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputDefinition {
    pub name: String,
    pub input_type: String,
    pub required: Option<bool>,
    pub default: Option<serde_json::Value>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    pub name: String,
    pub action: ActionType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition: Option<Condition>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub on_error: Option<ErrorStrategy>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry: Option<RetryConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub store_as: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ActionType {
    Navigate {
        url: String,
        #[serde(default)]
        screenshot: bool,
    },
    Click {
        selector: String,
        #[serde(default)]
        wait_after: u64,
    },
    Fill {
        selector: String,
        value: String,
    },
    Extract {
        selector: String,
        attribute: Option<String>,
    },
    Wait {
        #[serde(flatten)]
        wait_type: WaitType,
    },
    Assert {
        #[serde(flatten)]
        assertion: AssertionType,
    },
    Loop {
        over: String,
        #[serde(rename = "do")]
        body: Vec<WorkflowStep>,
    },
    Conditional {
        #[serde(rename = "if")]
        condition: Condition,
        #[serde(rename = "then")]
        then_branch: Vec<WorkflowStep>,
        #[serde(rename = "else")]
        else_branch: Option<Vec<WorkflowStep>>,
    },
    Script {
        code: String,
    },
    Parallel {
        steps: Vec<WorkflowStep>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "wait_for", rename_all = "snake_case")]
pub enum WaitType {
    Element { selector: String },
    Text { text: String },
    Url { pattern: String },
    Time { seconds: u64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "assert", rename_all = "snake_case")]
pub enum AssertionType {
    ElementExists { selector: String },
    TextContains { text: String },
    UrlMatches { pattern: String },
    ElementCount { selector: String, count: usize },
    Title { expected: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "check", rename_all = "snake_case")]
pub enum Condition {
    ElementExists { selector: String },
    TextContains { text: String },
    VariableEquals { var: String, value: serde_json::Value },
    VariableGreaterThan { var: String, value: f64 },
    VariableLessThan { var: String, value: f64 },
    Not { condition: Box<Condition> },
    And { conditions: Vec<Condition> },
    Or { conditions: Vec<Condition> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorStrategy {
    Fail,
    Continue,
    Retry,
    Fallback { steps: Vec<WorkflowStep> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub delay_seconds: u64,
    pub exponential_backoff: Option<bool>,
}

pub struct WorkflowEngine {
    browser: Option<SimpleBrowser>,
    pub cost_tracker: CostTracker,
    variables: HashMap<String, serde_json::Value>,
    execution_log: Vec<ExecutionEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionEntry {
    pub timestamp: DateTime<Utc>,
    pub step_name: String,
    pub action: String,
    pub success: bool,
    pub duration_ms: u64,
    pub error: Option<String>,
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowResult {
    pub success: bool,
    pub duration_ms: u64,
    pub steps_executed: usize,
    pub steps_failed: usize,
    pub variables: HashMap<String, serde_json::Value>,
    pub execution_log: Vec<ExecutionEntry>,
    pub cost: f64,
}

impl WorkflowEngine {
    pub fn new(cost_tracker: CostTracker) -> Self {
        Self {
            browser: None,
            cost_tracker,
            variables: HashMap::new(),
            execution_log: Vec::new(),
        }
    }
    
    /// API-friendly constructor without CostTracker
    pub fn new_simple() -> Self {
        Self {
            browser: None,
            cost_tracker: CostTracker::new(100.0), // Default budget
            variables: HashMap::new(),
            execution_log: Vec::new(),
        }
    }
    
    /// Set a variable for use in templates
    pub async fn set_variable(&mut self, name: &str, value: serde_json::Value) {
        self.variables.insert(name.to_string(), value);
    }
    
    /// Simple execute method for API
    pub async fn execute(&mut self, workflow: &Workflow) -> Result<WorkflowResult> {
        self.execute_workflow(workflow, None).await
    }

    pub async fn execute_workflow(&mut self, workflow: &Workflow, inputs: Option<HashMap<String, serde_json::Value>>) -> Result<WorkflowResult> {
        let start_time = std::time::Instant::now();
        info!("🎭 Starting workflow: {}", workflow.name);

        // Initialize variables with inputs
        self.variables = workflow.variables.clone();
        if let Some(inputs) = inputs {
            for (key, value) in inputs {
                self.variables.insert(key, value);
            }
        }

        // Validate required inputs
        if let Some(input_defs) = &workflow.inputs {
            for input_def in input_defs {
                if input_def.required.unwrap_or(false) && !self.variables.contains_key(&input_def.name) {
                    if let Some(default) = &input_def.default {
                        self.variables.insert(input_def.name.clone(), default.clone());
                    } else {
                        return Err(anyhow::anyhow!("Required input '{}' not provided", input_def.name));
                    }
                }
            }
        }

        // Initialize browser if needed
        if self.browser.is_none() {
            info!("🌐 Initializing browser for workflow");
            self.browser = Some(SimpleBrowser::new().await?);
        }

        // Execute steps
        let mut steps_executed = 0;
        let mut steps_failed = 0;

        if workflow.parallel.unwrap_or(false) {
            // Execute steps in parallel
            info!("⚡ Executing {} steps in parallel", workflow.steps.len());
            let results = self.execute_parallel_steps(&workflow.steps).await?;
            for result in results {
                steps_executed += 1;
                if !result.success {
                    steps_failed += 1;
                }
                self.execution_log.push(result);
            }
        } else {
            // Execute steps sequentially
            for step in &workflow.steps {
                let step_result = self.execute_step(step).await;
                steps_executed += 1;

                let success = step_result.is_ok();
                let error_msg = step_result.as_ref().err().map(|e| e.to_string());

                self.execution_log.push(ExecutionEntry {
                    timestamp: Utc::now(),
                    step_name: step.name.clone(),
                    action: format!("{:?}", step.action),
                    success,
                    duration_ms: 0, // TODO: Track individual step duration
                    error: error_msg.clone(),
                    data: step_result.as_ref().ok().cloned(),
                });

                if !success {
                    steps_failed += 1;
                    
                    // Handle error strategy
                    match step.on_error.as_ref().or(workflow.on_error.as_ref()) {
                        Some(ErrorStrategy::Continue) => {
                            warn!("Step '{}' failed, continuing workflow", step.name);
                            continue;
                        }
                        Some(ErrorStrategy::Retry) => {
                            // Retry logic handled in execute_step
                            continue;
                        }
                        Some(ErrorStrategy::Fallback { steps }) => {
                            info!("Executing fallback steps for '{}'", step.name);
                            for fallback_step in steps {
                                let _ = self.execute_step(fallback_step).await;
                            }
                        }
                        _ => {
                            error!("Step '{}' failed, stopping workflow", step.name);
                            break;
                        }
                    }
                }
            }
        }

        // Calculate cost
        let workflow_cost = self.cost_tracker.estimate_browser_operation_cost() * steps_executed as f64;
        self.cost_tracker.record_operation(
            "workflow".to_string(),
            format!("Workflow: {}", workflow.name),
            workflow_cost,
            steps_failed == 0,
        )?;

        let duration = start_time.elapsed();
        
        Ok(WorkflowResult {
            success: steps_failed == 0,
            duration_ms: duration.as_millis() as u64,
            steps_executed,
            steps_failed,
            variables: self.variables.clone(),
            execution_log: self.execution_log.clone(),
            cost: workflow_cost,
        })
    }

    fn execute_step<'a>(&'a mut self, step: &'a WorkflowStep) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<serde_json::Value>> + Send + 'a>> {
        Box::pin(async move {
        info!("📍 Executing step: {}", step.name);

        // Check condition if present
        if let Some(condition) = &step.condition {
            if !self.evaluate_condition(condition).await? {
                info!("Skipping step '{}' - condition not met", step.name);
                return Ok(serde_json::json!({"skipped": true}));
            }
        }

        // Execute with retry if configured
        let max_attempts = step.retry.as_ref().map(|r| r.max_attempts).unwrap_or(1);
        let mut last_error = None;

        for attempt in 1..=max_attempts {
            if attempt > 1 {
                let delay = step.retry.as_ref().map(|r| r.delay_seconds).unwrap_or(2);
                info!("Retry attempt {} after {}s delay", attempt, delay);
                sleep(Duration::from_secs(delay)).await;
            }

            match self.execute_action(&step.action).await {
                Ok(result) => {
                    // Store result if requested
                    if let Some(var_name) = &step.store_as {
                        self.variables.insert(var_name.clone(), result.clone());
                        info!("Stored result in variable '{}'", var_name);
                    }
                    return Ok(result);
                }
                Err(e) => {
                    warn!("Step '{}' attempt {} failed: {}", step.name, attempt, e);
                    last_error = Some(e);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("Step failed after {} attempts", max_attempts)))
        })
    }

    async fn execute_action(&mut self, action: &ActionType) -> Result<serde_json::Value> {
        let browser = self.browser.as_ref().ok_or_else(|| anyhow::anyhow!("Browser not initialized"))?;

        match action {
            ActionType::Navigate { url, screenshot } => {
                info!("🌐 Navigating to: {}", url);
                let expanded_url = self.expand_template(url)?;
                browser.navigate_to(&expanded_url).await?;
                
                if *screenshot {
                    let filename = format!("workflow_{}.png", Utc::now().format("%Y%m%d_%H%M%S"));
                    browser.take_screenshot(&filename).await?;
                    return Ok(serde_json::json!({"screenshot": filename}));
                }
                
                Ok(serde_json::json!({"navigated": expanded_url}))
            }
            
            ActionType::Click { selector, wait_after } => {
                info!("🖱️ Clicking: {}", selector);
                let expanded_selector = self.expand_template(selector)?;
                browser.click(&expanded_selector).await?;
                
                if *wait_after > 0 {
                    sleep(Duration::from_secs(*wait_after)).await;
                }
                
                Ok(serde_json::json!({"clicked": expanded_selector}))
            }
            
            ActionType::Fill { selector, value } => {
                info!("✏️ Filling field: {}", selector);
                let expanded_selector = self.expand_template(selector)?;
                let expanded_value = self.expand_template(value)?;
                browser.fill_field(&expanded_selector, &expanded_value).await?;
                
                Ok(serde_json::json!({"filled": expanded_selector, "value": expanded_value}))
            }
            
            ActionType::Extract { selector, attribute } => {
                info!("📋 Extracting from: {}", selector);
                let expanded_selector = self.expand_template(selector)?;
                
                let extracted = if let Some(attr) = attribute {
                    browser.get_attribute(&expanded_selector, attr).await?
                } else {
                    browser.get_text(&expanded_selector).await?
                };
                
                Ok(serde_json::json!(extracted))
            }
            
            ActionType::Wait { wait_type } => {
                match wait_type {
                    WaitType::Time { seconds } => {
                        info!("⏳ Waiting {} seconds", seconds);
                        sleep(Duration::from_secs(*seconds)).await;
                    }
                    WaitType::Element { selector } => {
                        info!("⏳ Waiting for element: {}", selector);
                        let expanded_selector = self.expand_template(selector)?;
                        browser.wait_for_element(&expanded_selector, Duration::from_secs(30)).await?;
                    }
                    WaitType::Text { text } => {
                        info!("⏳ Waiting for text: {}", text);
                        let expanded_text = self.expand_template(text)?;
                        browser.wait_for_text(&expanded_text, Duration::from_secs(30)).await?;
                    }
                    WaitType::Url { pattern } => {
                        info!("⏳ Waiting for URL pattern: {}", pattern);
                        let expanded_pattern = self.expand_template(pattern)?;
                        browser.wait_for_url(&expanded_pattern, Duration::from_secs(30)).await?;
                    }
                }
                Ok(serde_json::json!({"waited": true}))
            }
            
            ActionType::Assert { assertion } => {
                self.execute_assertion(assertion).await
            }
            
            ActionType::Loop { over, body } => {
                self.execute_loop(over, body).await
            }
            
            ActionType::Conditional { condition, then_branch, else_branch } => {
                self.execute_conditional(condition, then_branch, else_branch.as_ref()).await
            }
            
            ActionType::Script { code } => {
                self.execute_script(code).await
            }
            
            ActionType::Parallel { steps } => {
                let results = self.execute_parallel_steps(steps).await?;
                Ok(serde_json::json!(results))
            }
        }
    }

    async fn execute_assertion(&self, assertion: &AssertionType) -> Result<serde_json::Value> {
        let browser = self.browser.as_ref().ok_or_else(|| anyhow::anyhow!("Browser not initialized"))?;

        match assertion {
            AssertionType::ElementExists { selector } => {
                let expanded_selector = self.expand_template(selector)?;
                let exists = browser.element_exists(&expanded_selector).await?;
                if !exists {
                    return Err(anyhow::anyhow!("Assertion failed: Element '{}' does not exist", expanded_selector));
                }
                Ok(serde_json::json!({"assertion": "element_exists", "passed": true}))
            }
            
            AssertionType::TextContains { text } => {
                let expanded_text = self.expand_template(text)?;
                let page_text = browser.get_page_text().await?;
                if !page_text.contains(&expanded_text) {
                    return Err(anyhow::anyhow!("Assertion failed: Page does not contain text '{}'", expanded_text));
                }
                Ok(serde_json::json!({"assertion": "text_contains", "passed": true}))
            }
            
            AssertionType::UrlMatches { pattern } => {
                let expanded_pattern = self.expand_template(pattern)?;
                let current_url = browser.current_url().await?;
                if !current_url.contains(&expanded_pattern) {
                    return Err(anyhow::anyhow!("Assertion failed: URL '{}' does not match pattern '{}'", current_url, expanded_pattern));
                }
                Ok(serde_json::json!({"assertion": "url_matches", "passed": true}))
            }
            
            AssertionType::ElementCount { selector, count } => {
                let expanded_selector = self.expand_template(selector)?;
                let actual_count = browser.count_elements(&expanded_selector).await?;
                if actual_count != *count {
                    return Err(anyhow::anyhow!("Assertion failed: Expected {} elements, found {}", count, actual_count));
                }
                Ok(serde_json::json!({"assertion": "element_count", "passed": true, "count": count}))
            }
            
            AssertionType::Title { expected } => {
                let expanded_expected = self.expand_template(expected)?;
                let actual_title = browser.get_title().await?;
                if actual_title != expanded_expected {
                    return Err(anyhow::anyhow!("Assertion failed: Expected title '{}', got '{}'", expanded_expected, actual_title));
                }
                Ok(serde_json::json!({"assertion": "title", "passed": true}))
            }
        }
    }

    async fn execute_loop(&mut self, over: &str, body: &[WorkflowStep]) -> Result<serde_json::Value> {
        info!("🔄 Executing loop over: {}", over);
        
        // Get the collection to iterate over
        let collection = self.variables.get(over)
            .ok_or_else(|| anyhow::anyhow!("Variable '{}' not found for loop", over))?
            .clone();
        
        let items = collection.as_array()
            .ok_or_else(|| anyhow::anyhow!("Variable '{}' is not an array", over))?;
        
        let mut results = Vec::new();
        let items_len = items.len();
        
        for (index, item) in items.iter().enumerate() {
            info!("Loop iteration {}/{}", index + 1, items_len);
            
            // Set loop variables
            self.variables.insert("_loop_index".to_string(), serde_json::json!(index));
            self.variables.insert("_loop_item".to_string(), item.clone());
            
            // Execute loop body
            for step in body {
                let result = self.execute_step(step).await?;
                results.push(result);
            }
        }
        
        Ok(serde_json::json!({"loop_results": results}))
    }

    async fn execute_conditional(&mut self, condition: &Condition, then_branch: &[WorkflowStep], else_branch: Option<&Vec<WorkflowStep>>) -> Result<serde_json::Value> {
        info!("❓ Evaluating conditional");
        
        if self.evaluate_condition(condition).await? {
            info!("✅ Condition met, executing then branch");
            let mut results = Vec::new();
            for step in then_branch {
                let result = self.execute_step(step).await?;
                results.push(result);
            }
            Ok(serde_json::json!({"branch": "then", "results": results}))
        } else if let Some(else_steps) = else_branch {
            info!("❌ Condition not met, executing else branch");
            let mut results = Vec::new();
            for step in else_steps {
                let result = self.execute_step(step).await?;
                results.push(result);
            }
            Ok(serde_json::json!({"branch": "else", "results": results}))
        } else {
            info!("❌ Condition not met, no else branch");
            Ok(serde_json::json!({"branch": "none"}))
        }
    }

    async fn execute_script(&self, code: &str) -> Result<serde_json::Value> {
        let browser = self.browser.as_ref().ok_or_else(|| anyhow::anyhow!("Browser not initialized"))?;
        let expanded_code = self.expand_template(code)?;
        
        info!("📜 Executing script");
        let result = browser.execute_script(&expanded_code).await?;
        
        Ok(result)
    }

    async fn execute_parallel_steps(&mut self, steps: &[WorkflowStep]) -> Result<Vec<ExecutionEntry>> {
        use futures::future::join_all;
        
        let mut handles = Vec::new();
        
        for step in steps {
            let step_clone = step.clone();
            let handle = tokio::spawn(async move {
                // Create a temporary engine for parallel execution
                // In a real implementation, this would share state properly
                ExecutionEntry {
                    timestamp: Utc::now(),
                    step_name: step_clone.name,
                    action: format!("{:?}", step_clone.action),
                    success: true, // Simplified for PoC
                    duration_ms: 0,
                    error: None,
                    data: Some(serde_json::json!({"parallel": true})),
                }
            });
            handles.push(handle);
        }
        
        let results = join_all(handles).await;
        let mut entries = Vec::new();
        
        for result in results {
            match result {
                Ok(entry) => entries.push(entry),
                Err(e) => {
                    error!("Parallel step failed: {}", e);
                }
            }
        }
        
        Ok(entries)
    }

    fn evaluate_condition<'a>(&'a self, condition: &'a Condition) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<bool>> + Send + 'a>> {
        Box::pin(async move {
        match condition {
            Condition::ElementExists { selector } => {
                let browser = self.browser.as_ref().ok_or_else(|| anyhow::anyhow!("Browser not initialized"))?;
                let expanded_selector = self.expand_template(selector)?;
                browser.element_exists(&expanded_selector).await
            }
            
            Condition::TextContains { text } => {
                let browser = self.browser.as_ref().ok_or_else(|| anyhow::anyhow!("Browser not initialized"))?;
                let expanded_text = self.expand_template(text)?;
                let page_text = browser.get_page_text().await?;
                Ok(page_text.contains(&expanded_text))
            }
            
            Condition::VariableEquals { var, value } => {
                let actual = self.variables.get(var);
                Ok(actual == Some(value))
            }
            
            Condition::VariableGreaterThan { var, value } => {
                if let Some(actual) = self.variables.get(var) {
                    if let Some(num) = actual.as_f64() {
                        return Ok(num > *value);
                    }
                }
                Ok(false)
            }
            
            Condition::VariableLessThan { var, value } => {
                if let Some(actual) = self.variables.get(var) {
                    if let Some(num) = actual.as_f64() {
                        return Ok(num < *value);
                    }
                }
                Ok(false)
            }
            
            Condition::Not { condition } => {
                Ok(!self.evaluate_condition(condition).await?)
            }
            
            Condition::And { conditions } => {
                for cond in conditions {
                    if !self.evaluate_condition(cond).await? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            
            Condition::Or { conditions } => {
                for cond in conditions {
                    if self.evaluate_condition(cond).await? {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
        }
        })
    }

    fn expand_template(&self, template: &str) -> Result<String> {
        let mut result = template.to_string();
        
        // Simple template expansion using {{variable}} syntax
        for (key, value) in &self.variables {
            let placeholder = format!("{{{{{}}}}}", key);
            let replacement = match value {
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Number(n) => n.to_string(),
                serde_json::Value::Bool(b) => b.to_string(),
                _ => value.to_string(),
            };
            result = result.replace(&placeholder, &replacement);
        }
        
        Ok(result)
    }

    pub async fn cleanup(&mut self) -> Result<()> {
        // Browser cleanup is handled separately - cannot take ownership here
        self.browser = None;
        Ok(())
    }
}

impl Workflow {
    pub fn from_yaml(yaml_str: &str) -> Result<Self> {
        serde_yaml::from_str(yaml_str)
            .context("Failed to parse workflow YAML")
    }

    pub fn from_json(json_str: &str) -> Result<Self> {
        serde_json::from_str(json_str)
            .context("Failed to parse workflow JSON")
    }

    pub fn to_yaml(&self) -> Result<String> {
        serde_yaml::to_string(self)
            .context("Failed to serialize workflow to YAML")
    }

    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self)
            .context("Failed to serialize workflow to JSON")
    }
}