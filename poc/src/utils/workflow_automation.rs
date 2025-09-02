// Workflow Automation System
// Orchestrates complex multi-step browser workflows with AI-driven adaptation

use anyhow::{Result, Context};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tracing::{debug, info, warn, error};

use crate::perception_mvp::{
    browser_connection::BrowserConnection,
    perception_orchestrator::{PerceptionOrchestrator, UnifiedPerceptionResult, PerceptionLevel},
    natural_language::NaturalLanguageFinder,
};
use crate::intelligence::core::ai_decision_engine::{AIDecisionEngine, DecisionContext, QualityRequirements};

/// Workflow automation engine
pub struct WorkflowEngine {
    perception_orchestrator: PerceptionOrchestrator,
    ai_decision_engine: AIDecisionEngine,
    nl_finder: NaturalLanguageFinder,
    execution_context: ExecutionContext,
    workflow_registry: HashMap<String, WorkflowTemplate>,
    execution_stats: ExecutionStats,
}

/// Execution context for workflows
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    pub variables: HashMap<String, String>,
    pub session_data: HashMap<String, String>,
    pub error_recovery_enabled: bool,
    pub max_retries: u32,
    pub timeout_duration: Duration,
    pub quality_threshold: f32,
}

/// Statistics about workflow executions
#[derive(Debug, Clone, Default)]
pub struct ExecutionStats {
    pub total_workflows: u64,
    pub successful_workflows: u64,
    pub failed_workflows: u64,
    pub average_execution_time_ms: f64,
    pub total_steps_executed: u64,
    pub error_recovery_invocations: u64,
}

/// A complete workflow definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub author: String,
    pub tags: Vec<String>,
    pub parameters: Vec<WorkflowParameter>,
    pub steps: Vec<WorkflowStep>,
    pub error_handling: ErrorHandlingStrategy,
    pub success_criteria: Vec<SuccessCriterion>,
    pub metadata: WorkflowMetadata,
}

/// Parameter definition for workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowParameter {
    pub name: String,
    pub parameter_type: ParameterType,
    pub required: bool,
    pub default_value: Option<String>,
    pub description: String,
    pub validation_rules: Vec<ValidationRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParameterType {
    String,
    Number,
    Boolean,
    Url,
    Selector,
    FilePath,
    Email,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    pub rule_type: ValidationType,
    pub value: String,
    pub error_message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationType {
    Regex,
    MinLength,
    MaxLength,
    Range,
    Required,
    Url,
    Email,
}

/// Individual step in a workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    pub id: String,
    pub name: String,
    pub step_type: StepType,
    pub action: ActionDefinition,
    pub conditions: Vec<StepCondition>,
    pub retry_policy: RetryPolicy,
    pub timeout: Option<Duration>,
    pub on_success: Vec<String>, // Next step IDs
    pub on_failure: Vec<String>, // Failure handling step IDs
    pub variables: HashMap<String, String>, // Step-specific variables
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StepType {
    Navigation,
    Perception,
    Interaction,
    Validation,
    DataExtraction,
    Decision,
    Loop,
    Conditional,
    Parallel,
    Wait,
    Custom(String),
}

/// Action definition for a workflow step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionDefinition {
    pub action_type: ActionType,
    pub target: Option<String>,
    pub parameters: HashMap<String, String>,
    pub expected_outcome: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionType {
    // Navigation actions
    NavigateTo,
    GoBack,
    GoForward,
    Refresh,
    
    // Perception actions
    PerceivePage,
    AnalyzeContent,
    FindElement,
    
    // Interaction actions
    Click,
    Type,
    Select,
    Submit,
    Upload,
    Download,
    
    // Validation actions
    AssertExists,
    AssertText,
    AssertUrl,
    AssertValue,
    
    // Data actions
    ExtractText,
    ExtractAttribute,
    ExtractData,
    SaveData,
    
    // Control flow actions
    If,
    Loop,
    Wait,
    Break,
    Continue,
    
    // AI actions
    MakeDecision,
    LearnPattern,
    AdaptStrategy,
    
    // Custom actions
    ExecuteScript,
    CallApi,
    SendNotification,
    Custom(String),
}

/// Condition for step execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepCondition {
    pub condition_type: ConditionType,
    pub operator: ComparisonOperator,
    pub expected_value: String,
    pub actual_value_source: ValueSource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConditionType {
    ElementExists,
    TextContains,
    UrlMatches,
    VariableEquals,
    PerceptionConfidence,
    TimeElapsed,
    StepResult,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComparisonOperator {
    Equals,
    NotEquals,
    Contains,
    NotContains,
    GreaterThan,
    LessThan,
    Matches, // Regex
    Exists,
    NotExists,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValueSource {
    Constant(String),
    Variable(String),
    ElementText(String),
    ElementAttribute(String, String),
    PerceptionResult(String),
    PreviousStepResult,
}

/// Retry policy for failed steps
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub delay_ms: u64,
    pub exponential_backoff: bool,
    pub retry_conditions: Vec<RetryCondition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryCondition {
    pub error_type: String,
    pub should_retry: bool,
}

/// Error handling strategy for workflows
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorHandlingStrategy {
    pub strategy_type: ErrorStrategyType,
    pub recovery_steps: Vec<String>, // Step IDs for error recovery
    pub notification_settings: NotificationSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorStrategyType {
    FailFast,
    ContinueOnError,
    RetryWithRecovery,
    AdaptiveRecovery,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationSettings {
    pub notify_on_error: bool,
    pub notify_on_success: bool,
    pub notification_channels: Vec<String>,
}

/// Success criteria for workflow completion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuccessCriterion {
    pub criterion_type: SuccessCriterionType,
    pub description: String,
    pub validation_method: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SuccessCriterionType {
    AllStepsCompleted,
    SpecificStepsCompleted(Vec<String>),
    DataExtracted,
    TargetReached,
    ConditionMet(StepCondition),
    TimeThresholdMet,
    Custom(String),
}

/// Metadata about the workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowMetadata {
    pub created_at: String,
    pub updated_at: String,
    pub execution_count: u64,
    pub success_rate: f32,
    pub average_duration_ms: f64,
    pub complexity_score: f32,
    pub dependencies: Vec<String>,
}

/// Result of workflow execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowExecutionResult {
    pub workflow_id: String,
    pub execution_id: String,
    pub status: ExecutionStatus,
    pub start_time: String,
    pub end_time: Option<String>,
    pub duration_ms: u64,
    pub steps_executed: Vec<StepExecutionResult>,
    pub extracted_data: HashMap<String, String>,
    pub error_messages: Vec<String>,
    pub performance_metrics: WorkflowPerformanceMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionStatus {
    Running,
    Completed,
    Failed,
    Paused,
    Cancelled,
    PartiallyCompleted,
}

/// Result of individual step execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepExecutionResult {
    pub step_id: String,
    pub status: StepStatus,
    pub start_time: String,
    pub duration_ms: u64,
    pub result_data: HashMap<String, String>,
    pub error_message: Option<String>,
    pub retry_count: u32,
    pub perception_data: Option<String>, // Serialized perception result
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StepStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Skipped,
    Retrying,
}

/// Performance metrics for workflow execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowPerformanceMetrics {
    pub total_perception_time_ms: u64,
    pub total_interaction_time_ms: u64,
    pub total_wait_time_ms: u64,
    pub cache_hit_rate: f32,
    pub ai_decision_count: u32,
    pub error_recovery_count: u32,
    pub adaptive_changes_count: u32,
}

impl WorkflowEngine {
    pub fn new(
        perception_orchestrator: PerceptionOrchestrator,
        ai_decision_engine: AIDecisionEngine,
    ) -> Self {
        Self {
            perception_orchestrator,
            ai_decision_engine,
            nl_finder: NaturalLanguageFinder::new(),
            execution_context: ExecutionContext::default(),
            workflow_registry: HashMap::new(),
            execution_stats: ExecutionStats::default(),
        }
    }
    
    /// Register a workflow template
    pub fn register_workflow(&mut self, workflow: WorkflowTemplate) -> Result<()> {
        info!("Registering workflow: {} ({})", workflow.name, workflow.id);
        
        // Validate workflow
        self.validate_workflow(&workflow)?;
        
        self.workflow_registry.insert(workflow.id.clone(), workflow);
        Ok(())
    }
    
    /// Execute a workflow by ID
    pub async fn execute_workflow(
        &mut self,
        workflow_id: &str,
        parameters: HashMap<String, String>,
        browser: &BrowserConnection,
    ) -> Result<WorkflowExecutionResult> {
        let workflow = self.workflow_registry
            .get(workflow_id)
            .ok_or_else(|| anyhow::anyhow!("Workflow not found: {}", workflow_id))?
            .clone();
        
        self.execute_workflow_template(&workflow, parameters, browser).await
    }
    
    /// Execute a workflow template directly
    pub async fn execute_workflow_template(
        &mut self,
        workflow: &WorkflowTemplate,
        parameters: HashMap<String, String>,
        browser: &BrowserConnection,
    ) -> Result<WorkflowExecutionResult> {
        let execution_id = format!("exec_{}", self.generate_id());
        let start_time = Instant::now();
        
        info!("Starting workflow execution: {} ({})", workflow.name, execution_id);
        
        // Initialize execution context
        self.execution_context.variables = parameters;
        
        // Create result structure
        let mut result = WorkflowExecutionResult {
            workflow_id: workflow.id.clone(),
            execution_id: execution_id.clone(),
            status: ExecutionStatus::Running,
            start_time: chrono::Utc::now().to_rfc3339(),
            end_time: None,
            duration_ms: 0,
            steps_executed: Vec::new(),
            extracted_data: HashMap::new(),
            error_messages: Vec::new(),
            performance_metrics: WorkflowPerformanceMetrics {
                total_perception_time_ms: 0,
                total_interaction_time_ms: 0,
                total_wait_time_ms: 0,
                cache_hit_rate: 0.0,
                ai_decision_count: 0,
                error_recovery_count: 0,
                adaptive_changes_count: 0,
            },
        };
        
        // Execute workflow steps
        match self.execute_steps(&workflow.steps, browser, &mut result).await {
            Ok(_) => {
                result.status = ExecutionStatus::Completed;
                info!("Workflow completed successfully: {}", execution_id);
            }
            Err(e) => {
                result.status = ExecutionStatus::Failed;
                result.error_messages.push(e.to_string());
                error!("Workflow failed: {} - {}", execution_id, e);
                
                // Attempt error recovery if enabled
                if self.execution_context.error_recovery_enabled {
                    if let Ok(_) = self.attempt_error_recovery(&workflow, browser, &mut result).await {
                        result.status = ExecutionStatus::PartiallyCompleted;
                    }
                }
            }
        }
        
        // Finalize result
        result.duration_ms = start_time.elapsed().as_millis() as u64;
        result.end_time = Some(chrono::Utc::now().to_rfc3339());
        
        // Update statistics
        self.update_execution_stats(&result);
        
        Ok(result)
    }
    
    /// Execute a sequence of workflow steps
    async fn execute_steps(
        &mut self,
        steps: &[WorkflowStep],
        browser: &BrowserConnection,
        result: &mut WorkflowExecutionResult,
    ) -> Result<()> {
        let mut step_index = 0;
        
        while step_index < steps.len() {
            let step = &steps[step_index];
            let step_start = Instant::now();
            
            info!("Executing step: {} ({})", step.name, step.id);
            
            // Check step conditions
            if !self.evaluate_step_conditions(&step.conditions, browser).await? {
                debug!("Step conditions not met, skipping: {}", step.id);
                result.steps_executed.push(StepExecutionResult {
                    step_id: step.id.clone(),
                    status: StepStatus::Skipped,
                    start_time: chrono::Utc::now().to_rfc3339(),
                    duration_ms: 0,
                    result_data: HashMap::new(),
                    error_message: Some("Conditions not met".to_string()),
                    retry_count: 0,
                    perception_data: None,
                });
                step_index += 1;
                continue;
            }
            
            // Execute step with retry policy
            let step_result = self.execute_step_with_retry(step, browser).await;
            let step_duration = step_start.elapsed().as_millis() as u64;
            
            match step_result {
                Ok(step_data) => {
                    // Successful step execution
                    result.steps_executed.push(StepExecutionResult {
                        step_id: step.id.clone(),
                        status: StepStatus::Completed,
                        start_time: chrono::Utc::now().to_rfc3339(),
                        duration_ms: step_duration,
                        result_data: step_data.clone(),
                        error_message: None,
                        retry_count: 0,
                        perception_data: None,
                    });
                    
                    // Update context variables with step results
                    for (key, value) in step_data {
                        self.execution_context.variables.insert(key, value);
                    }
                    
                    // Determine next step
                    if step.on_success.is_empty() {
                        step_index += 1; // Continue to next step
                    } else {
                        // Find next step by ID
                        let next_step_id = &step.on_success[0];
                        if let Some(next_index) = steps.iter().position(|s| &s.id == next_step_id) {
                            step_index = next_index;
                        } else {
                            warn!("Next step not found: {}", next_step_id);
                            step_index += 1;
                        }
                    }
                }
                Err(e) => {
                    // Failed step execution
                    error!("Step failed: {} - {}", step.id, e);
                    
                    result.steps_executed.push(StepExecutionResult {
                        step_id: step.id.clone(),
                        status: StepStatus::Failed,
                        start_time: chrono::Utc::now().to_rfc3339(),
                        duration_ms: step_duration,
                        result_data: HashMap::new(),
                        error_message: Some(e.to_string()),
                        retry_count: 0,
                        perception_data: None,
                    });
                    
                    // Handle failure based on step configuration
                    if !step.on_failure.is_empty() {
                        let failure_step_id = &step.on_failure[0];
                        if let Some(failure_index) = steps.iter().position(|s| &s.id == failure_step_id) {
                            step_index = failure_index;
                            continue;
                        }
                    }
                    
                    // If no failure handling, propagate error
                    return Err(e);
                }
            }
        }
        
        Ok(())
    }
    
    /// Execute a single step with retry logic
    async fn execute_step_with_retry(
        &mut self,
        step: &WorkflowStep,
        browser: &BrowserConnection,
    ) -> Result<HashMap<String, String>> {
        let mut attempts = 0;
        let mut last_error = None;
        
        while attempts < step.retry_policy.max_attempts {
            match self.execute_single_step(step, browser).await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    attempts += 1;
                    last_error = Some(e);
                    
                    if attempts < step.retry_policy.max_attempts {
                        let delay = if step.retry_policy.exponential_backoff {
                            step.retry_policy.delay_ms * (2_u64.pow(attempts - 1))
                        } else {
                            step.retry_policy.delay_ms
                        };
                        
                        warn!("Step failed, retrying in {}ms (attempt {}/{})", 
                              delay, attempts, step.retry_policy.max_attempts);
                        sleep(Duration::from_millis(delay)).await;
                    }
                }
            }
        }
        
        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("Step failed after {} attempts", attempts)))
    }
    
    /// Execute a single step
    async fn execute_single_step(
        &mut self,
        step: &WorkflowStep,
        browser: &BrowserConnection,
    ) -> Result<HashMap<String, String>> {
        let mut result_data = HashMap::new();
        
        match &step.action.action_type {
            ActionType::NavigateTo => {
                let url = self.resolve_parameter(&step.action.target)?;
                browser.navigate(&url).await?;
                result_data.insert("navigated_to".to_string(), url);
            }
            
            ActionType::PerceivePage => {
                let level = step.action.parameters.get("level")
                    .unwrap_or(&"adaptive".to_string()).clone();
                
                let perception_level = match level.as_str() {
                    "lightning" => PerceptionLevel::Lightning,
                    "quick" => PerceptionLevel::Quick,
                    "standard" => PerceptionLevel::Standard,
                    "deep" => PerceptionLevel::Deep,
                    _ => PerceptionLevel::Quick, // Default
                };
                
                let perception_result = self.perception_orchestrator
                    .perceive(browser, perception_level).await?;
                
                result_data.insert("perception_confidence".to_string(), 
                                 perception_result.execution_info.confidence_score.to_string());
                result_data.insert("perception_quality".to_string(), 
                                 perception_result.execution_info.quality_score.to_string());
            }
            
            ActionType::FindElement => {
                let description = self.resolve_parameter(&step.action.target)?;
                let element_result = self.nl_finder
                    .find_element(&description, browser, None).await?;
                
                result_data.insert("element_selector".to_string(), element_result.selector);
                result_data.insert("element_confidence".to_string(), element_result.confidence.to_string());
            }
            
            ActionType::Click => {
                let target = self.resolve_parameter(&step.action.target)?;
                
                // Use natural language finding if target looks like a description
                let selector = if target.starts_with('#') || target.starts_with('.') || target.contains('=') {
                    target // Already a CSS selector
                } else {
                    // Use natural language finding
                    let element_result = self.nl_finder
                        .find_element(&target, browser, None).await?;
                    element_result.selector
                };
                
                browser.click_element(&selector).await?;
                result_data.insert("clicked_element".to_string(), selector);
            }
            
            ActionType::Type => {
                let target = self.resolve_parameter(&step.action.target)?;
                let text = step.action.parameters.get("text")
                    .ok_or_else(|| anyhow::anyhow!("Text parameter required for Type action"))?;
                let resolved_text = self.resolve_parameter(text)?;
                
                browser.type_text(&target, &resolved_text).await?;
                result_data.insert("typed_text".to_string(), resolved_text);
            }
            
            ActionType::ExtractText => {
                let target = self.resolve_parameter(&step.action.target)?;
                let text = browser.get_element_text(&target).await?;
                
                result_data.insert("extracted_text".to_string(), text);
            }
            
            ActionType::Wait => {
                let duration_str = step.action.parameters.get("duration")
                    .ok_or_else(|| anyhow::anyhow!("Duration parameter required for Wait action"))?;
                let duration_ms: u64 = duration_str.parse()?;
                
                sleep(Duration::from_millis(duration_ms)).await;
                result_data.insert("waited_ms".to_string(), duration_ms.to_string());
            }
            
            ActionType::MakeDecision => {
                let perception_result = self.perception_orchestrator
                    .perceive_adaptive(browser).await?;
                
                let context = DecisionContext {
                    page_url: browser.current_url().await?,
                    user_goal: step.action.parameters.get("goal").cloned(),
                    time_constraints: None,
                    quality_requirements: QualityRequirements::default(),
                    business_context: None,
                };
                
                let decision = self.ai_decision_engine
                    .make_decision(&perception_result, &context).await?;
                
                result_data.insert("decision_confidence".to_string(), decision.confidence.to_string());
                result_data.insert("decision_reasoning".to_string(), decision.reasoning);
            }
            
            ActionType::Custom(action_name) => {
                warn!("Custom action not implemented: {}", action_name);
                result_data.insert("custom_action".to_string(), action_name.clone());
            }
            
            _ => {
                warn!("Action type not implemented: {:?}", step.action.action_type);
                result_data.insert("action_type".to_string(), format!("{:?}", step.action.action_type));
            }
        }
        
        Ok(result_data)
    }
    
    /// Resolve parameter values (variables, etc.)
    fn resolve_parameter(&self, parameter: &Option<String>) -> Result<String> {
        match parameter {
            Some(value) => {
                if value.starts_with("${") && value.ends_with("}") {
                    let var_name = &value[2..value.len()-1];
                    self.execution_context.variables.get(var_name)
                        .cloned()
                        .ok_or_else(|| anyhow::anyhow!("Variable not found: {}", var_name))
                } else {
                    Ok(value.clone())
                }
            }
            None => Err(anyhow::anyhow!("Parameter is required")),
        }
    }
    
    /// Evaluate step conditions
    async fn evaluate_step_conditions(
        &self,
        conditions: &[StepCondition],
        browser: &BrowserConnection,
    ) -> Result<bool> {
        for condition in conditions {
            let actual_value = self.get_actual_value(&condition.actual_value_source, browser).await?;
            let condition_met = self.compare_values(&actual_value, &condition.expected_value, &condition.operator)?;
            
            if !condition_met {
                return Ok(false);
            }
        }
        
        Ok(true)
    }
    
    /// Get actual value from value source
    async fn get_actual_value(
        &self,
        source: &ValueSource,
        browser: &BrowserConnection,
    ) -> Result<String> {
        match source {
            ValueSource::Constant(value) => Ok(value.clone()),
            ValueSource::Variable(var_name) => {
                self.execution_context.variables.get(var_name)
                    .cloned()
                    .ok_or_else(|| anyhow::anyhow!("Variable not found: {}", var_name))
            }
            ValueSource::ElementText(selector) => {
                browser.get_element_text(selector).await
            }
            ValueSource::ElementAttribute(selector, attr) => {
                browser.get_element_attribute(selector, attr).await
            }
            _ => {
                warn!("Value source not implemented: {:?}", source);
                Ok(String::new())
            }
        }
    }
    
    /// Compare values using operator
    fn compare_values(&self, actual: &str, expected: &str, operator: &ComparisonOperator) -> Result<bool> {
        Ok(match operator {
            ComparisonOperator::Equals => actual == expected,
            ComparisonOperator::NotEquals => actual != expected,
            ComparisonOperator::Contains => actual.contains(expected),
            ComparisonOperator::NotContains => !actual.contains(expected),
            ComparisonOperator::Exists => !actual.is_empty(),
            ComparisonOperator::NotExists => actual.is_empty(),
            ComparisonOperator::Matches => {
                regex::Regex::new(expected)?.is_match(actual)
            }
            _ => false,
        })
    }
    
    /// Attempt error recovery
    async fn attempt_error_recovery(
        &mut self,
        workflow: &WorkflowTemplate,
        browser: &BrowserConnection,
        result: &mut WorkflowExecutionResult,
    ) -> Result<()> {
        info!("Attempting error recovery for workflow: {}", workflow.id);
        
        // Execute recovery steps if defined
        if !workflow.error_handling.recovery_steps.is_empty() {
            let recovery_steps: Vec<WorkflowStep> = workflow.steps.iter()
                .filter(|step| workflow.error_handling.recovery_steps.contains(&step.id))
                .cloned()
                .collect();
            
            if !recovery_steps.is_empty() {
                self.execute_steps(&recovery_steps, browser, result).await?;
                result.performance_metrics.error_recovery_count += 1;
            }
        }
        
        Ok(())
    }
    
    /// Validate workflow template
    fn validate_workflow(&self, workflow: &WorkflowTemplate) -> Result<()> {
        // Check required fields
        if workflow.id.is_empty() {
            return Err(anyhow::anyhow!("Workflow ID cannot be empty"));
        }
        
        if workflow.steps.is_empty() {
            return Err(anyhow::anyhow!("Workflow must have at least one step"));
        }
        
        // Validate step IDs are unique
        let mut step_ids = std::collections::HashSet::new();
        for step in &workflow.steps {
            if !step_ids.insert(&step.id) {
                return Err(anyhow::anyhow!("Duplicate step ID: {}", step.id));
            }
        }
        
        // Validate step references
        for step in &workflow.steps {
            for next_step_id in &step.on_success {
                if !step_ids.contains(next_step_id) {
                    return Err(anyhow::anyhow!("Invalid step reference: {}", next_step_id));
                }
            }
            for failure_step_id in &step.on_failure {
                if !step_ids.contains(failure_step_id) {
                    return Err(anyhow::anyhow!("Invalid failure step reference: {}", failure_step_id));
                }
            }
        }
        
        Ok(())
    }
    
    /// Update execution statistics
    fn update_execution_stats(&mut self, result: &WorkflowExecutionResult) {
        self.execution_stats.total_workflows += 1;
        
        match result.status {
            ExecutionStatus::Completed | ExecutionStatus::PartiallyCompleted => {
                self.execution_stats.successful_workflows += 1;
            }
            ExecutionStatus::Failed => {
                self.execution_stats.failed_workflows += 1;
            }
            _ => {}
        }
        
        self.execution_stats.total_steps_executed += result.steps_executed.len() as u64;
        self.execution_stats.error_recovery_invocations += result.performance_metrics.error_recovery_count as u64;
        
        // Update average execution time
        let total_time = self.execution_stats.average_execution_time_ms * (self.execution_stats.total_workflows as f64 - 1.0) + result.duration_ms as f64;
        self.execution_stats.average_execution_time_ms = total_time / self.execution_stats.total_workflows as f64;
    }
    
    /// Generate unique ID
    fn generate_id(&self) -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        format!("{:x}", timestamp)
    }
    
    /// Get execution statistics
    pub fn get_stats(&self) -> &ExecutionStats {
        &self.execution_stats
    }
    
    /// List registered workflows
    pub fn list_workflows(&self) -> Vec<&WorkflowTemplate> {
        self.workflow_registry.values().collect()
    }
}

impl Default for ExecutionContext {
    fn default() -> Self {
        Self {
            variables: HashMap::new(),
            session_data: HashMap::new(),
            error_recovery_enabled: true,
            max_retries: 3,
            timeout_duration: Duration::from_secs(300), // 5 minutes
            quality_threshold: 0.7,
        }
    }
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            delay_ms: 1000,
            exponential_backoff: true,
            retry_conditions: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::perception_mvp::perception_orchestrator::{PerceptionOrchestrator, OrchestratorConfig};

    #[test]
    fn test_workflow_engine_creation() {
        let orchestrator = PerceptionOrchestrator::new(OrchestratorConfig::default());
        let ai_engine = AIDecisionEngine::new();
        let engine = WorkflowEngine::new(orchestrator, ai_engine);
        
        assert_eq!(engine.workflow_registry.len(), 0);
        assert_eq!(engine.execution_stats.total_workflows, 0);
    }
    
    #[test]
    fn test_workflow_validation() {
        let orchestrator = PerceptionOrchestrator::new(OrchestratorConfig::default());
        let ai_engine = AIDecisionEngine::new();
        let engine = WorkflowEngine::new(orchestrator, ai_engine);
        
        let mut workflow = WorkflowTemplate {
            id: "test_workflow".to_string(),
            name: "Test Workflow".to_string(),
            description: "A test workflow".to_string(),
            version: "1.0.0".to_string(),
            author: "Test Author".to_string(),
            tags: vec!["test".to_string()],
            parameters: Vec::new(),
            steps: vec![
                WorkflowStep {
                    id: "step1".to_string(),
                    name: "Step 1".to_string(),
                    step_type: StepType::Navigation,
                    action: ActionDefinition {
                        action_type: ActionType::NavigateTo,
                        target: Some("https://example.com".to_string()),
                        parameters: HashMap::new(),
                        expected_outcome: "Navigate to example.com".to_string(),
                    },
                    conditions: Vec::new(),
                    retry_policy: RetryPolicy::default(),
                    timeout: None,
                    on_success: Vec::new(),
                    on_failure: Vec::new(),
                    variables: HashMap::new(),
                }
            ],
            error_handling: ErrorHandlingStrategy {
                strategy_type: ErrorStrategyType::FailFast,
                recovery_steps: Vec::new(),
                notification_settings: NotificationSettings {
                    notify_on_error: false,
                    notify_on_success: false,
                    notification_channels: Vec::new(),
                },
            },
            success_criteria: Vec::new(),
            metadata: WorkflowMetadata {
                created_at: chrono::Utc::now().to_rfc3339(),
                updated_at: chrono::Utc::now().to_rfc3339(),
                execution_count: 0,
                success_rate: 0.0,
                average_duration_ms: 0.0,
                complexity_score: 1.0,
                dependencies: Vec::new(),
            },
        };
        
        // Valid workflow should pass
        assert!(engine.validate_workflow(&workflow).is_ok());
        
        // Empty ID should fail
        workflow.id = String::new();
        assert!(engine.validate_workflow(&workflow).is_err());
    }
}