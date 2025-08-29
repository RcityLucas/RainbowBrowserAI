//! Tool Orchestrator - Intelligent Tool Coordination System
//! 
//! This module implements an intelligent system that:
//! 1. Analyzes user commands to determine required tools
//! 2. Plans the execution sequence based on dependencies
//! 3. Executes tools in the optimal order
//! 4. Handles errors and retries
//! 5. Aggregates results for the user

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};
use anyhow::{Result, Context as AnyhowContext};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::sync::RwLock;
use tracing::{info, debug, warn, error, instrument};
use async_trait::async_trait;

use crate::{
    tools::{Tool, DynamicTool, ToolRegistry},
    instruction_parser::{UserInstruction, Intent},
    browser::SimpleBrowser,
    v8_perception::{PerceptionMode, PerceptionEngine},
};

/// Tool execution plan - defines what tools to run and in what order
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExecutionPlan {
    /// The original user command
    pub user_command: String,
    
    /// Parsed intent from the command
    pub intent: String,
    
    /// List of tool executions to perform
    pub steps: Vec<ToolExecutionStep>,
    
    /// Dependencies between steps (step_id -> depends_on)
    pub dependencies: HashMap<String, Vec<String>>,
    
    /// Estimated total execution time
    pub estimated_duration_ms: u64,
    
    /// Confidence in the plan (0.0 - 1.0)
    pub confidence: f32,
}

/// Individual tool execution step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExecutionStep {
    /// Unique ID for this step
    pub id: String,
    
    /// Tool name to execute
    pub tool_name: String,
    
    /// Input parameters for the tool
    pub parameters: Value,
    
    /// Whether this step is critical (failure stops execution)
    pub critical: bool,
    
    /// Maximum retries if the step fails
    pub max_retries: u32,
    
    /// Timeout for this step
    pub timeout_ms: u64,
    
    /// Description of what this step does
    pub description: String,
}

/// Result of executing a tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExecutionResult {
    /// Step ID that was executed
    pub step_id: String,
    
    /// Tool that was executed
    pub tool_name: String,
    
    /// Whether execution was successful
    pub success: bool,
    
    /// Output from the tool
    pub output: Option<Value>,
    
    /// Error message if failed
    pub error: Option<String>,
    
    /// Execution duration
    pub duration_ms: u64,
    
    /// Number of retries needed
    pub retries_used: u32,
}

/// Overall orchestration result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestrationResult {
    /// Whether the overall orchestration succeeded
    pub success: bool,
    
    /// The execution plan that was followed
    pub plan: ToolExecutionPlan,
    
    /// Results from each tool execution
    pub step_results: Vec<ToolExecutionResult>,
    
    /// Total execution time
    pub total_duration_ms: u64,
    
    /// Final aggregated output for the user
    pub final_output: Value,
    
    /// Human-readable summary
    pub summary: String,
}

/// Tool Orchestrator - coordinates multiple tool executions
pub struct ToolOrchestrator {
    /// Registry of available tools
    tool_registry: Arc<RwLock<ToolRegistry>>,
    
    /// Browser instance for web operations
    browser: Arc<SimpleBrowser>,
    
    /// Perception engine for page analysis
    perception_engine: Arc<PerceptionEngine>,
    
    /// Execution history for learning
    execution_history: Arc<RwLock<Vec<OrchestrationResult>>>,
    
    /// Tool capability mapping
    tool_capabilities: HashMap<String, ToolCapability>,
}

/// Describes what a tool can do
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCapability {
    /// Tool name
    pub name: String,
    
    /// What intents this tool can handle
    pub handles_intents: Vec<String>,
    
    /// What this tool requires as input
    pub requires: Vec<String>,
    
    /// What this tool produces as output
    pub produces: Vec<String>,
    
    /// Average execution time in ms
    pub avg_execution_time: u64,
    
    /// Success rate (0.0 - 1.0)
    pub success_rate: f32,
}

impl ToolOrchestrator {
    /// Create a new Tool Orchestrator
    pub fn new(
        browser: Arc<SimpleBrowser>,
        perception_engine: Arc<PerceptionEngine>,
    ) -> Self {
        let tool_registry = Arc::new(RwLock::new(ToolRegistry::new()));
        
        // Initialize tool capabilities (this would be loaded from config in production)
        let mut tool_capabilities = HashMap::new();
        
        // Navigation tools
        tool_capabilities.insert("navigate_to_url".to_string(), ToolCapability {
            name: "navigate_to_url".to_string(),
            handles_intents: vec!["navigate".to_string(), "open".to_string(), "visit".to_string()],
            requires: vec!["url".to_string()],
            produces: vec!["page_loaded".to_string(), "page_title".to_string()],
            avg_execution_time: 2000,
            success_rate: 0.95,
        });
        
        tool_capabilities.insert("go_back".to_string(), ToolCapability {
            name: "go_back".to_string(),
            handles_intents: vec!["back".to_string(), "previous".to_string()],
            requires: vec!["page_loaded".to_string()],
            produces: vec!["page_loaded".to_string()],
            avg_execution_time: 500,
            success_rate: 0.98,
        });
        
        // Interaction tools
        tool_capabilities.insert("click".to_string(), ToolCapability {
            name: "click".to_string(),
            handles_intents: vec!["click".to_string(), "tap".to_string(), "press".to_string()],
            requires: vec!["page_loaded".to_string(), "element_selector".to_string()],
            produces: vec!["action_performed".to_string()],
            avg_execution_time: 300,
            success_rate: 0.90,
        });
        
        tool_capabilities.insert("type_text".to_string(), ToolCapability {
            name: "type_text".to_string(),
            handles_intents: vec!["type".to_string(), "input".to_string(), "fill".to_string()],
            requires: vec!["page_loaded".to_string(), "element_selector".to_string(), "text".to_string()],
            produces: vec!["text_entered".to_string()],
            avg_execution_time: 500,
            success_rate: 0.92,
        });
        
        // Synchronization tools
        tool_capabilities.insert("wait_for_element".to_string(), ToolCapability {
            name: "wait_for_element".to_string(),
            handles_intents: vec!["wait".to_string()],
            requires: vec!["page_loaded".to_string(), "element_selector".to_string()],
            produces: vec!["element_ready".to_string()],
            avg_execution_time: 1000,
            success_rate: 0.88,
        });
        
        // Data extraction tools
        tool_capabilities.insert("extract_text".to_string(), ToolCapability {
            name: "extract_text".to_string(),
            handles_intents: vec!["extract".to_string(), "get".to_string(), "read".to_string()],
            requires: vec!["page_loaded".to_string()],
            produces: vec!["extracted_text".to_string()],
            avg_execution_time: 200,
            success_rate: 0.95,
        });
        
        tool_capabilities.insert("extract_links".to_string(), ToolCapability {
            name: "extract_links".to_string(),
            handles_intents: vec!["links".to_string(), "hrefs".to_string()],
            requires: vec!["page_loaded".to_string()],
            produces: vec!["link_list".to_string()],
            avg_execution_time: 300,
            success_rate: 0.96,
        });
        
        // Memory tools
        tool_capabilities.insert("take_screenshot".to_string(), ToolCapability {
            name: "take_screenshot".to_string(),
            handles_intents: vec!["screenshot".to_string(), "capture".to_string(), "snap".to_string()],
            requires: vec!["page_loaded".to_string()],
            produces: vec!["screenshot_path".to_string()],
            avg_execution_time: 500,
            success_rate: 0.97,
        });
        
        Self {
            tool_registry,
            browser,
            perception_engine,
            execution_history: Arc::new(RwLock::new(Vec::new())),
            tool_capabilities,
        }
    }
    
    /// Main orchestration method - analyzes command and executes tools
    #[instrument(skip(self))]
    pub async fn orchestrate(&self, instruction: &UserInstruction) -> Result<OrchestrationResult> {
        let start_time = Instant::now();
        info!("Starting orchestration for command: {}", instruction.raw_text);
        
        // Step 1: Create execution plan
        let plan = self.create_execution_plan(instruction).await?;
        info!("Created execution plan with {} steps", plan.steps.len());
        debug!("Execution plan: {:?}", plan);
        
        // Step 2: Validate plan
        self.validate_plan(&plan)?;
        
        // Step 3: Execute plan
        let step_results = self.execute_plan(&plan).await?;
        
        // Step 4: Aggregate results
        let final_output = self.aggregate_results(&plan, &step_results)?;
        
        // Step 5: Create summary
        let summary = self.create_summary(&plan, &step_results, &final_output);
        
        let total_duration_ms = start_time.elapsed().as_millis() as u64;
        
        let result = OrchestrationResult {
            success: step_results.iter().all(|r| r.success || !self.is_critical_step(&plan, &r.step_id)),
            plan: plan.clone(),
            step_results,
            total_duration_ms,
            final_output,
            summary,
        };
        
        // Store in history for learning
        self.execution_history.write().await.push(result.clone());
        
        info!("Orchestration completed in {}ms", total_duration_ms);
        Ok(result)
    }
    
    /// Create an execution plan based on the user instruction
    async fn create_execution_plan(&self, instruction: &UserInstruction) -> Result<ToolExecutionPlan> {
        let mut steps = Vec::new();
        let mut dependencies = HashMap::new();
        let mut current_context = HashSet::new();
        current_context.insert("browser_ready".to_string());
        
        // Analyze the intent and required tools
        match &instruction.intent {
            Intent::Navigate { url, .. } => {
                // Navigation requires just the navigate tool
                steps.push(ToolExecutionStep {
                    id: "nav_1".to_string(),
                    tool_name: "navigate_to_url".to_string(),
                    parameters: json!({
                        "url": url,
                    }),
                    critical: true,
                    max_retries: 3,
                    timeout_ms: 10000,
                    description: format!("Navigate to {}", url),
                });
                current_context.insert("page_loaded".to_string());
            }
            
            Intent::Click { target, .. } => {
                // Click requires the page to be analyzed first
                steps.push(ToolExecutionStep {
                    id: "perceive_1".to_string(),
                    tool_name: "analyze_page".to_string(),
                    parameters: json!({
                        "mode": "quick",
                    }),
                    critical: false,
                    max_retries: 1,
                    timeout_ms: 2000,
                    description: "Analyze page structure".to_string(),
                });
                
                steps.push(ToolExecutionStep {
                    id: "click_1".to_string(),
                    tool_name: "click".to_string(),
                    parameters: json!({
                        "target": target,
                    }),
                    critical: true,
                    max_retries: 2,
                    timeout_ms: 3000,
                    description: format!("Click on {}", target),
                });
                
                dependencies.insert("click_1".to_string(), vec!["perceive_1".to_string()]);
            }
            
            Intent::Type { text, target, clear_first } => {
                // Type requires finding the input field first
                steps.push(ToolExecutionStep {
                    id: "wait_1".to_string(),
                    tool_name: "wait_for_element".to_string(),
                    parameters: json!({
                        "selector": target,
                        "timeout": 5000,
                    }),
                    critical: true,
                    max_retries: 2,
                    timeout_ms: 6000,
                    description: format!("Wait for input field {}", target),
                });
                
                if *clear_first {
                    steps.push(ToolExecutionStep {
                        id: "clear_1".to_string(),
                        tool_name: "clear_input".to_string(),
                        parameters: json!({
                            "selector": target,
                        }),
                        critical: false,
                        max_retries: 1,
                        timeout_ms: 1000,
                        description: "Clear input field".to_string(),
                    });
                    dependencies.insert("clear_1".to_string(), vec!["wait_1".to_string()]);
                }
                
                steps.push(ToolExecutionStep {
                    id: "type_1".to_string(),
                    tool_name: "type_text".to_string(),
                    parameters: json!({
                        "selector": target,
                        "text": text,
                    }),
                    critical: true,
                    max_retries: 2,
                    timeout_ms: 3000,
                    description: format!("Type '{}' into field", text),
                });
                
                let deps = if *clear_first {
                    vec!["clear_1".to_string()]
                } else {
                    vec!["wait_1".to_string()]
                };
                dependencies.insert("type_1".to_string(), deps);
            }
            
            Intent::Search { query, .. } => {
                // Search is a complex multi-step operation
                // 1. Find search box
                // 2. Click on it
                // 3. Type the query
                // 4. Press enter or click search button
                
                steps.push(ToolExecutionStep {
                    id: "analyze_1".to_string(),
                    tool_name: "analyze_page".to_string(),
                    parameters: json!({
                        "mode": "standard",
                        "find_search": true,
                    }),
                    critical: true,
                    max_retries: 2,
                    timeout_ms: 5000,
                    description: "Find search elements on page".to_string(),
                });
                
                steps.push(ToolExecutionStep {
                    id: "click_search".to_string(),
                    tool_name: "click".to_string(),
                    parameters: json!({
                        "target": "search_input",
                        "smart_select": true,
                    }),
                    critical: true,
                    max_retries: 2,
                    timeout_ms: 2000,
                    description: "Click on search box".to_string(),
                });
                dependencies.insert("click_search".to_string(), vec!["analyze_1".to_string()]);
                
                steps.push(ToolExecutionStep {
                    id: "type_query".to_string(),
                    tool_name: "type_text".to_string(),
                    parameters: json!({
                        "text": query,
                        "target": "search_input",
                    }),
                    critical: true,
                    max_retries: 2,
                    timeout_ms: 3000,
                    description: format!("Type search query: {}", query),
                });
                dependencies.insert("type_query".to_string(), vec!["click_search".to_string()]);
                
                steps.push(ToolExecutionStep {
                    id: "submit_search".to_string(),
                    tool_name: "press_key".to_string(),
                    parameters: json!({
                        "key": "Enter",
                    }),
                    critical: true,
                    max_retries: 2,
                    timeout_ms: 1000,
                    description: "Submit search".to_string(),
                });
                dependencies.insert("submit_search".to_string(), vec!["type_query".to_string()]);
            }
            
            Intent::Extract { data_type, .. } => {
                // Extract requires analyzing the page first
                steps.push(ToolExecutionStep {
                    id: "analyze_deep".to_string(),
                    tool_name: "analyze_page".to_string(),
                    parameters: json!({
                        "mode": "deep",
                    }),
                    critical: true,
                    max_retries: 1,
                    timeout_ms: 10000,
                    description: "Deep analysis of page content".to_string(),
                });
                
                let extract_tool = match data_type.as_deref() {
                    Some("links") => "extract_links",
                    Some("images") => "extract_images",
                    Some("tables") => "extract_tables",
                    _ => "extract_text",
                };
                
                steps.push(ToolExecutionStep {
                    id: "extract_1".to_string(),
                    tool_name: extract_tool.to_string(),
                    parameters: json!({}),
                    critical: true,
                    max_retries: 1,
                    timeout_ms: 5000,
                    description: format!("Extract {} from page", data_type.as_deref().unwrap_or("text")),
                });
                dependencies.insert("extract_1".to_string(), vec!["analyze_deep".to_string()]);
            }
            
            Intent::Screenshot { .. } => {
                // Screenshot is simple
                steps.push(ToolExecutionStep {
                    id: "screenshot_1".to_string(),
                    tool_name: "take_screenshot".to_string(),
                    parameters: json!({
                        "full_page": true,
                    }),
                    critical: true,
                    max_retries: 2,
                    timeout_ms: 3000,
                    description: "Take screenshot of page".to_string(),
                });
            }
            
            _ => {
                // For unknown intents, do a page analysis
                steps.push(ToolExecutionStep {
                    id: "analyze_default".to_string(),
                    tool_name: "analyze_page".to_string(),
                    parameters: json!({
                        "mode": "standard",
                    }),
                    critical: false,
                    max_retries: 1,
                    timeout_ms: 5000,
                    description: "Analyze page for context".to_string(),
                });
            }
        }
        
        // Calculate estimated duration
        let estimated_duration_ms = steps.iter()
            .map(|s| s.timeout_ms)
            .sum::<u64>() / 2; // Assume average execution is half the timeout
        
        Ok(ToolExecutionPlan {
            user_command: instruction.raw_text.clone(),
            intent: format!("{:?}", instruction.intent),
            steps,
            dependencies,
            estimated_duration_ms,
            confidence: instruction.confidence,
        })
    }
    
    /// Validate that the execution plan is feasible
    fn validate_plan(&self, plan: &ToolExecutionPlan) -> Result<()> {
        // Check for circular dependencies
        for (step_id, deps) in &plan.dependencies {
            if deps.contains(step_id) {
                return Err(anyhow::anyhow!("Circular dependency detected for step {}", step_id));
            }
        }
        
        // Check that all dependencies exist
        let step_ids: HashSet<_> = plan.steps.iter().map(|s| &s.id).collect();
        for deps in plan.dependencies.values() {
            for dep in deps {
                if !step_ids.contains(&dep) {
                    return Err(anyhow::anyhow!("Dependency {} not found in plan", dep));
                }
            }
        }
        
        // Check that required tools exist
        for step in &plan.steps {
            if !self.tool_capabilities.contains_key(&step.tool_name) {
                warn!("Tool {} not found in capabilities, will check registry", step.tool_name);
            }
        }
        
        Ok(())
    }
    
    /// Execute the plan step by step
    async fn execute_plan(&self, plan: &ToolExecutionPlan) -> Result<Vec<ToolExecutionResult>> {
        let mut results = Vec::new();
        let mut completed = HashSet::new();
        let mut context = HashMap::new();
        
        info!("Executing plan with {} steps", plan.steps.len());
        
        // Execute steps in dependency order
        while completed.len() < plan.steps.len() {
            let mut made_progress = false;
            
            for step in &plan.steps {
                if completed.contains(&step.id) {
                    continue;
                }
                
                // Check if dependencies are satisfied
                let deps_satisfied = plan.dependencies
                    .get(&step.id)
                    .map(|deps| deps.iter().all(|d| completed.contains(d)))
                    .unwrap_or(true);
                
                if deps_satisfied {
                    info!("Executing step {}: {}", step.id, step.description);
                    let result = self.execute_step(step, &context).await;
                    
                    // Update context with output
                    if let Ok(ref res) = result {
                        if let Some(ref output) = res.output {
                            context.insert(step.id.clone(), output.clone());
                        }
                    }
                    
                    results.push(result?);
                    completed.insert(step.id.clone());
                    made_progress = true;
                }
            }
            
            if !made_progress {
                return Err(anyhow::anyhow!("Could not make progress in execution plan - possible deadlock"));
            }
        }
        
        Ok(results)
    }
    
    /// Execute a single step with retries
    async fn execute_step(
        &self,
        step: &ToolExecutionStep,
        context: &HashMap<String, Value>,
    ) -> Result<ToolExecutionResult> {
        let start_time = Instant::now();
        let mut last_error = None;
        let mut retries_used = 0;
        
        for attempt in 0..=step.max_retries {
            if attempt > 0 {
                info!("Retry {} for step {}", attempt, step.id);
                tokio::time::sleep(Duration::from_millis(500 * attempt as u64)).await;
            }
            
            // Prepare parameters with context
            let mut params = step.parameters.clone();
            if let Value::Object(ref mut map) = params {
                map.insert("_context".to_string(), json!(context));
            }
            
            // Execute the tool
            let result = match step.tool_name.as_str() {
                "navigate_to_url" => self.execute_navigate(params).await,
                "click" => self.execute_click(params).await,
                "type_text" => self.execute_type(params).await,
                "wait_for_element" => self.execute_wait(params).await,
                "analyze_page" => self.execute_analyze(params).await,
                "extract_text" => self.execute_extract_text(params).await,
                "extract_links" => self.execute_extract_links(params).await,
                "take_screenshot" => self.execute_screenshot(params).await,
                "press_key" => self.execute_press_key(params).await,
                "clear_input" => self.execute_clear_input(params).await,
                _ => {
                    // Try to find in registry
                    if let Ok(registry) = self.tool_registry.try_read() {
                        if let Some(tool) = registry.get(&step.tool_name) {
                            tool.execute_json(params).await
                        } else {
                            Err(anyhow::anyhow!("Unknown tool: {}", step.tool_name))
                        }
                    } else {
                        Err(anyhow::anyhow!("Could not access tool registry"))
                    }
                }
            };
            
            match result {
                Ok(output) => {
                    return Ok(ToolExecutionResult {
                        step_id: step.id.clone(),
                        tool_name: step.tool_name.clone(),
                        success: true,
                        output: Some(output),
                        error: None,
                        duration_ms: start_time.elapsed().as_millis() as u64,
                        retries_used,
                    });
                }
                Err(e) => {
                    warn!("Step {} failed on attempt {}: {}", step.id, attempt + 1, e);
                    last_error = Some(e.to_string());
                    retries_used = attempt;
                }
            }
        }
        
        // All retries exhausted
        error!("Step {} failed after {} retries", step.id, step.max_retries);
        Ok(ToolExecutionResult {
            step_id: step.id.clone(),
            tool_name: step.tool_name.clone(),
            success: false,
            output: None,
            error: last_error,
            duration_ms: start_time.elapsed().as_millis() as u64,
            retries_used,
        })
    }
    
    // Tool execution implementations
    
    async fn execute_navigate(&self, params: Value) -> Result<Value> {
        let url = params["url"].as_str()
            .ok_or_else(|| anyhow::anyhow!("URL parameter required"))?;
        
        self.browser.navigate_to(url).await?;
        let title = self.browser.get_title().await?;
        
        Ok(json!({
            "success": true,
            "url": url,
            "title": title,
        }))
    }
    
    async fn execute_click(&self, params: Value) -> Result<Value> {
        let target = params["target"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Target parameter required"))?;
        
        // Use perception to find the element
        let perception_result = self.perception_engine
            .analyze(PerceptionMode::Quick, &self.browser)
            .await?;
        
        // Find matching element
        let element = perception_result.key_elements.iter()
            .find(|e| e.text.as_ref().map(|t| t.contains(target)).unwrap_or(false) ||
                      e.selector.contains(target))
            .ok_or_else(|| anyhow::anyhow!("Element not found: {}", target))?;
        
        // Click the element
        self.browser.click(&element.selector).await?;
        
        Ok(json!({
            "success": true,
            "clicked": element.selector.clone(),
        }))
    }
    
    async fn execute_type(&self, params: Value) -> Result<Value> {
        let text = params["text"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Text parameter required"))?;
        let selector = params["selector"].as_str()
            .or_else(|| params["target"].as_str())
            .unwrap_or("input");
        
        self.browser.type_text(selector, text).await?;
        
        Ok(json!({
            "success": true,
            "typed": text,
            "into": selector,
        }))
    }
    
    async fn execute_wait(&self, params: Value) -> Result<Value> {
        let selector = params["selector"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Selector parameter required"))?;
        let timeout = params["timeout"].as_u64().unwrap_or(5000);
        
        self.browser.wait_for_element(selector, Duration::from_millis(timeout)).await?;
        
        Ok(json!({
            "success": true,
            "found": selector,
        }))
    }
    
    async fn execute_analyze(&self, params: Value) -> Result<Value> {
        let mode_str = params["mode"].as_str().unwrap_or("standard");
        let mode = match mode_str {
            "lightning" => PerceptionMode::Lightning,
            "quick" => PerceptionMode::Quick,
            "deep" => PerceptionMode::Deep,
            _ => PerceptionMode::Standard,
        };
        
        let result = self.perception_engine.analyze(mode, &self.browser).await?;
        
        Ok(json!({
            "success": true,
            "mode": mode_str,
            "elements_found": result.elements_found,
            "page_status": result.page_status,
            "confidence": result.confidence,
        }))
    }
    
    async fn execute_extract_text(&self, params: Value) -> Result<Value> {
        let page_text = self.browser.extract_text().await?;
        
        Ok(json!({
            "success": true,
            "text": page_text,
            "length": page_text.len(),
        }))
    }
    
    async fn execute_extract_links(&self, params: Value) -> Result<Value> {
        let links = self.browser.extract_links().await?;
        
        Ok(json!({
            "success": true,
            "links": links,
            "count": links.len(),
        }))
    }
    
    async fn execute_screenshot(&self, params: Value) -> Result<Value> {
        let full_page = params["full_page"].as_bool().unwrap_or(false);
        let filename = format!("screenshot_{}.png", chrono::Utc::now().timestamp());
        
        self.browser.take_screenshot(&filename).await?;
        
        Ok(json!({
            "success": true,
            "filename": filename,
            "full_page": full_page,
        }))
    }
    
    async fn execute_press_key(&self, params: Value) -> Result<Value> {
        let key = params["key"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Key parameter required"))?;
        
        self.browser.press_key(key).await?;
        
        Ok(json!({
            "success": true,
            "key_pressed": key,
        }))
    }
    
    async fn execute_clear_input(&self, params: Value) -> Result<Value> {
        let selector = params["selector"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Selector parameter required"))?;
        
        self.browser.clear_input(selector).await?;
        
        Ok(json!({
            "success": true,
            "cleared": selector,
        }))
    }
    
    /// Aggregate results from all steps
    fn aggregate_results(
        &self,
        plan: &ToolExecutionPlan,
        results: &[ToolExecutionResult],
    ) -> Result<Value> {
        let mut aggregated = json!({
            "command": plan.user_command,
            "intent": plan.intent,
            "total_steps": plan.steps.len(),
            "successful_steps": results.iter().filter(|r| r.success).count(),
            "failed_steps": results.iter().filter(|r| !r.success).count(),
        });
        
        // Collect key outputs
        let mut outputs = json!({});
        for result in results {
            if let Some(ref output) = result.output {
                outputs[&result.step_id] = output.clone();
            }
        }
        aggregated["outputs"] = outputs;
        
        // Add any extracted data
        for result in results {
            if result.tool_name.starts_with("extract_") {
                if let Some(ref output) = result.output {
                    aggregated["extracted_data"] = output.clone();
                }
            }
        }
        
        // Add screenshot paths
        for result in results {
            if result.tool_name == "take_screenshot" {
                if let Some(ref output) = result.output {
                    aggregated["screenshot"] = output["filename"].clone();
                }
            }
        }
        
        Ok(aggregated)
    }
    
    /// Create a human-readable summary
    fn create_summary(
        &self,
        plan: &ToolExecutionPlan,
        results: &[ToolExecutionResult],
        final_output: &Value,
    ) -> String {
        let successful = results.iter().filter(|r| r.success).count();
        let total = results.len();
        
        let mut summary = format!(
            "Executed {} for command: '{}'\n",
            if successful == total { "successfully" } else { "with some failures" },
            plan.user_command
        );
        
        summary.push_str(&format!("Completed {}/{} steps\n", successful, total));
        
        // Add key achievements
        for result in results {
            if result.success {
                match result.tool_name.as_str() {
                    "navigate_to_url" => {
                        if let Some(output) = &result.output {
                            summary.push_str(&format!("✓ Navigated to {}\n", output["url"]));
                        }
                    }
                    "click" => summary.push_str("✓ Clicked on element\n"),
                    "type_text" => summary.push_str("✓ Typed text into field\n"),
                    "take_screenshot" => {
                        if let Some(output) = &result.output {
                            summary.push_str(&format!("✓ Screenshot saved as {}\n", output["filename"]));
                        }
                    }
                    "extract_text" => summary.push_str("✓ Extracted text from page\n"),
                    "extract_links" => {
                        if let Some(output) = &result.output {
                            summary.push_str(&format!("✓ Found {} links\n", output["count"]));
                        }
                    }
                    _ => {}
                }
            } else {
                summary.push_str(&format!("✗ Failed: {} - {}\n", 
                    result.tool_name,
                    result.error.as_ref().unwrap_or(&"Unknown error".to_string())
                ));
            }
        }
        
        summary
    }
    
    /// Check if a step is critical
    fn is_critical_step(&self, plan: &ToolExecutionPlan, step_id: &str) -> bool {
        plan.steps.iter()
            .find(|s| s.id == step_id)
            .map(|s| s.critical)
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_plan_creation() {
        // Test plan creation for navigation
        let instruction = UserInstruction {
            raw_text: "Go to google.com".to_string(),
            normalized_text: "go to google.com".to_string(),
            intent: Intent::Navigate { 
                url: "google.com".to_string(),
                new_tab: false,
            },
            confidence: 0.95,
            entities: vec![],
            context: Default::default(),
            suggestions: vec![],
        };
        
        // This would need a mock browser and perception engine
        // let orchestrator = ToolOrchestrator::new(browser, perception);
        // let plan = orchestrator.create_execution_plan(&instruction).await.unwrap();
        // assert_eq!(plan.steps.len(), 1);
        // assert_eq!(plan.steps[0].tool_name, "navigate_to_url");
    }
}