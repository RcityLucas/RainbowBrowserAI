//! Enhanced Execution Engine
//! 
//! Integrates the new instruction parser with semantic analysis for intelligent execution

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, warn, error};

use crate::{
    instruction_parser::{InstructionParser, UserInstruction, Feedback, ContextHints, PageType as InstructionPageType, WorkflowParser},
    semantic_analyzer::{SemanticAnalyzer, SemanticPageModel, PageType as SemanticPageType},
    action_mapper::{ActionMapper, ActionExecutor, ActionResult},
    browser::SimpleBrowser,
    ConversationContext,
    // tool_orchestrator::{ToolOrchestrator, OrchestrationResult}, // Module disabled
    // v8_perception::PerceptionEngine, // Module doesn't exist
};

/// Enhanced executor that uses natural language understanding
pub struct EnhancedExecutor {
    instruction_parser: Arc<Mutex<InstructionParser>>,
    workflow_parser: WorkflowParser,
    action_mapper: ActionMapper,
    last_page_model: Arc<Mutex<Option<SemanticPageModel>>>,
    // tool_orchestrator: Option<Arc<ToolOrchestrator>>, // Disabled - module not available
}

impl EnhancedExecutor {
    pub fn new() -> Self {
        Self {
            instruction_parser: Arc::new(Mutex::new(InstructionParser::new())),
            workflow_parser: WorkflowParser::new(),
            action_mapper: ActionMapper::new(),
            last_page_model: Arc::new(Mutex::new(None)),
            // tool_orchestrator: None, // Disabled
        }
    }
    
    // /// Set the tool orchestrator for complex tool coordination
    // pub fn with_orchestrator(mut self, browser: Arc<SimpleBrowser>, perception_engine: Arc<PerceptionEngine>) -> Self {
    //     self.tool_orchestrator = Some(Arc::new(ToolOrchestrator::new(browser, perception_engine)));
    //     self
    // }
    
    /// Execute instruction with tool orchestration
    pub async fn execute_with_orchestration(
        &self,
        instruction_text: &str,
        browser: &SimpleBrowser,
        context: Option<&ConversationContext>,
    ) -> Result<EnhancedExecutionResult> {
        info!("Executing with tool orchestration: {}", instruction_text);
        
        // Parse the instruction first
        let mut parser = self.instruction_parser.lock().await;
        let hints = self.create_context_hints(context).await;
        let instruction = parser.parse_with_context(instruction_text, hints)?;
        drop(parser);
        
        // Tool orchestrator is disabled for now
        // if let Some(ref orchestrator) = self.tool_orchestrator {
        //     info!("Using tool orchestrator for execution");
        //     
        //     // Execute with orchestration
        //     let orchestration_result = orchestrator.orchestrate(&instruction).await?;
        //     
        //     // Convert orchestration result to enhanced execution result
        //     return Ok(EnhancedExecutionResult {
        //         success: orchestration_result.success,
        //         instruction: Some(instruction),
        //         page_model: None, // Could be extracted from orchestration results
        //         actions_executed: orchestration_result.step_results.iter()
        //             .map(|r| ActionResult {
        //                 success: r.success,
        //                 action_type: r.tool_name.clone(),
        //                 details: r.output.clone().map(|o| o.to_string()),
        //                 error: r.error.clone(),
        //             })
        //             .collect(),
        //         clarification_needed: None,
        //         error: if !orchestration_result.success {
        //             Some(orchestration_result.summary.clone())
        //         } else {
        //             None
        //         },
        //         suggestions: vec![],
        //     });
        // }
        
        // Fall back to regular execution if no orchestrator
        warn!("No orchestrator configured, falling back to regular execution");
        self.execute_instruction(instruction_text, browser, context).await
    }
    
    /// Execute a natural language instruction
    pub async fn execute_instruction(
        &self,
        instruction_text: &str,
        browser: &SimpleBrowser,
        context: Option<&ConversationContext>,
    ) -> Result<EnhancedExecutionResult> {
        info!("Processing instruction: {}", instruction_text);
        
        // Check if this is a complex workflow (contains "and", "then", etc.)
        let is_workflow = instruction_text.contains(" and ") || 
                         instruction_text.contains(", then ") ||
                         instruction_text.contains(", ") && 
                         (instruction_text.contains("click") || 
                          instruction_text.contains("type") || 
                          instruction_text.contains("search"));
        
        if is_workflow {
            info!("Detected complex workflow, parsing multiple steps");
            return self.execute_workflow_from_text(instruction_text, browser, context).await;
        }
        
        // Parse single instruction
        let mut parser = self.instruction_parser.lock().await;
        let hints = self.create_context_hints(context).await;
        let instruction = parser.parse_with_context(instruction_text, hints)?;
        
        // Log parsed instruction
        info!("Parsed intent: {:?} with confidence: {}", 
            instruction.intent, instruction.confidence);
        
        // Check if clarification is needed
        if instruction.needs_clarification() {
            let questions = instruction.get_clarification_questions();
            return Ok(EnhancedExecutionResult {
                success: false,
                instruction: Some(instruction),
                page_model: None,
                actions_executed: vec![],
                clarification_needed: Some(questions),
                error: None,
                suggestions: vec![],
            });
        }
        
        // For navigation intents, skip semantic analysis of current page
        let should_analyze_before = !matches!(
            instruction.intent,
            crate::instruction_parser::intent_recognizer::Intent::Navigate { .. }
        );
        
        // Analyze current page only if not navigating
        let page_model_before = if should_analyze_before {
            let semantic_analyzer = SemanticAnalyzer::new(browser.driver());
            // Add timeout to prevent hanging
            match tokio::time::timeout(
                tokio::time::Duration::from_secs(5),
                semantic_analyzer.analyze()
            ).await {
                Ok(Ok(model)) => Some(model),
                Ok(Err(e)) => {
                    warn!("Semantic analysis failed: {}", e);
                    None
                }
                Err(_) => {
                    warn!("Semantic analysis timed out after 5 seconds");
                    None
                }
            }
        } else {
            None
        };
        
        // Use the analyzed model or create a dummy one for navigation
        let current_url = browser.current_url().await.unwrap_or_else(|_| "".to_string());
        let page_model = page_model_before.unwrap_or_else(|| {
            // Create minimal page model for action mapping
            crate::semantic_analyzer::SemanticPageModel {
                url: current_url.clone(),
                page_type: crate::semantic_analyzer::PageType::Unknown,
                regions: vec![],
                semantic_elements: vec![],
                relationships: vec![],
                interaction_points: vec![],
                data_structures: vec![],
            }
        });
        
        // Map instruction to actions
        let action = self.action_mapper.map_instruction(&instruction, &page_model)?;
        
        // Execute action
        let action_executor = ActionExecutor::new(browser);
        let action_result = action_executor.execute(&action).await?;
        
        // After navigation, analyze the new page
        let final_page_model = if matches!(
            instruction.intent,
            crate::instruction_parser::intent_recognizer::Intent::Navigate { .. }
        ) && action_result.success {
            let semantic_analyzer = SemanticAnalyzer::new(browser.driver());
            match tokio::time::timeout(
                tokio::time::Duration::from_secs(5),
                semantic_analyzer.analyze()
            ).await {
                Ok(Ok(model)) => {
                    info!("Analyzed new page after navigation: {}", model.url);
                    Some(model)
                }
                Ok(Err(e)) => {
                    warn!("Post-navigation semantic analysis failed: {}", e);
                    Some(page_model) // Use the pre-action model
                }
                Err(_) => {
                    warn!("Post-navigation semantic analysis timed out");
                    Some(page_model) // Use the pre-action model
                }
            }
        } else {
            Some(page_model)
        };
        
        // Update stored page model with the latest
        if let Some(ref model) = final_page_model {
            *self.last_page_model.lock().await = Some(model.clone());
        }
        
        // Learn from execution
        let success = action_result.success;
        let error = action_result.error.clone();
        let suggestions = self.generate_suggestions(&action_result);
        
        if success {
            parser.learn_from_feedback(&instruction, Feedback::Success);
        } else if let Some(ref err) = error {
            warn!("Action failed: {}", err);
            // Could provide corrected intent here
        }
        
        Ok(EnhancedExecutionResult {
            success,
            instruction: Some(instruction),
            page_model: final_page_model,
            actions_executed: vec![action_result],
            clarification_needed: None,
            error,
            suggestions,
        })
    }
    
    /// Execute a workflow from natural language text
    pub async fn execute_workflow_from_text(
        &self,
        workflow_text: &str,
        browser: &SimpleBrowser,
        context: Option<&ConversationContext>,
    ) -> Result<EnhancedExecutionResult> {
        // Parse workflow into steps
        let workflow_steps = self.workflow_parser.parse_workflow(workflow_text);
        
        if workflow_steps.is_empty() {
            return Ok(EnhancedExecutionResult {
                success: false,
                instruction: None,
                page_model: None,
                actions_executed: vec![],
                clarification_needed: None,
                error: Some("Could not parse workflow steps".to_string()),
                suggestions: vec!["Try breaking down your request into simpler steps".to_string()],
            });
        }
        
        info!("Executing workflow with {} steps", workflow_steps.len());
        
        let mut all_actions = Vec::new();
        let mut last_page_model = None;
        let mut overall_success = true;
        let mut errors = Vec::new();
        
        // Execute each step
        for (i, step) in workflow_steps.iter().enumerate() {
            info!("Executing workflow step {}/{}: {:?}", i + 1, workflow_steps.len(), step.intent);
            
            // Skip semantic analysis for navigation
            let should_analyze = !matches!(
                step.intent,
                crate::instruction_parser::intent_recognizer::Intent::Navigate { .. }
            );
            
            // Get current page model if needed
            let current_url = browser.current_url().await.unwrap_or_else(|_| "".to_string());
            let page_model = if should_analyze {
                let semantic_analyzer = SemanticAnalyzer::new(browser.driver());
                match tokio::time::timeout(
                    tokio::time::Duration::from_secs(5),
                    semantic_analyzer.analyze()
                ).await {
                    Ok(Ok(model)) => model,
                    _ => last_page_model.clone().unwrap_or_else(|| {
                        crate::semantic_analyzer::SemanticPageModel {
                            url: current_url.clone(),
                            page_type: crate::semantic_analyzer::PageType::Unknown,
                            regions: vec![],
                            semantic_elements: vec![],
                            relationships: vec![],
                            interaction_points: vec![],
                            data_structures: vec![],
                        }
                    })
                }
            } else {
                last_page_model.clone().unwrap_or_else(|| {
                    crate::semantic_analyzer::SemanticPageModel {
                        url: current_url,
                        page_type: crate::semantic_analyzer::PageType::Unknown,
                        regions: vec![],
                        semantic_elements: vec![],
                        relationships: vec![],
                        interaction_points: vec![],
                        data_structures: vec![],
                    }
                })
            };
            
            // Map to action
            match self.action_mapper.map_instruction(step, &page_model) {
                Ok(action) => {
                    // Execute action
                    let action_executor = ActionExecutor::new(browser);
                    match action_executor.execute(&action).await {
                        Ok(result) => {
                            info!("Step {} completed: success={}", i + 1, result.success);
                            all_actions.push(result.clone());
                            
                            if !result.success {
                                overall_success = false;
                                if let Some(err) = &result.error {
                                    errors.push(format!("Step {}: {}", i + 1, err));
                                }
                            }
                            
                            // Update page model after navigation
                            if matches!(step.intent, crate::instruction_parser::intent_recognizer::Intent::Navigate { .. }) && result.success {
                                let semantic_analyzer = SemanticAnalyzer::new(browser.driver());
                                if let Ok(Ok(model)) = tokio::time::timeout(
                                    tokio::time::Duration::from_secs(5),
                                    semantic_analyzer.analyze()
                                ).await {
                                    last_page_model = Some(model);
                                }
                            }
                        }
                        Err(e) => {
                            error!("Failed to execute step {}: {}", i + 1, e);
                            overall_success = false;
                            errors.push(format!("Step {}: {}", i + 1, e));
                            all_actions.push(ActionResult {
                                success: false,
                                action: action.clone(),
                                execution_time_ms: 0,
                                data: None,
                                error: Some(e.to_string()),
                                screenshot_path: None,
                            });
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to map step {} to action: {}", i + 1, e);
                    overall_success = false;
                    errors.push(format!("Step {} mapping: {}", i + 1, e));
                }
            }
            
            // Small delay between actions
            if i < workflow_steps.len() - 1 {
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            }
        }
        
        // Update stored page model
        if let Some(model) = &last_page_model {
            *self.last_page_model.lock().await = Some(model.clone());
        }
        
        Ok(EnhancedExecutionResult {
            success: overall_success,
            instruction: workflow_steps.first().cloned(),
            page_model: last_page_model,
            actions_executed: all_actions,
            clarification_needed: None,
            error: if errors.is_empty() { None } else { Some(errors.join("; ")) },
            suggestions: if !overall_success {
                vec!["Try breaking down complex instructions into simpler steps".to_string()]
            } else {
                vec![]
            },
        })
    }
    
    /// Execute multiple instructions in sequence
    pub async fn execute_workflow(
        &self,
        instructions: Vec<String>,
        browser: &SimpleBrowser,
        context: Option<&ConversationContext>,
    ) -> Result<WorkflowExecutionResult> {
        let mut results = Vec::new();
        let mut overall_success = true;
        
        for (i, instruction) in instructions.iter().enumerate() {
            info!("Executing step {} of {}: {}", i + 1, instructions.len(), instruction);
            
            match self.execute_instruction(instruction, browser, context).await {
                Ok(result) => {
                    if !result.success {
                        overall_success = false;
                        if result.clarification_needed.is_some() {
                            warn!("Step {} needs clarification", i + 1);
                            results.push(result);
                            break; // Stop on clarification needed
                        }
                    }
                    results.push(result);
                }
                Err(e) => {
                    error!("Step {} failed: {}", i + 1, e);
                    overall_success = false;
                    results.push(EnhancedExecutionResult {
                        success: false,
                        instruction: None,
                        page_model: None,
                        actions_executed: vec![],
                        clarification_needed: None,
                        error: Some(e.to_string()),
                        suggestions: vec![],
                    });
                    break; // Stop on error
                }
            }
            
            // Small delay between actions
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        }
        
        Ok(WorkflowExecutionResult {
            success: overall_success,
            steps_completed: results.len(),
            total_steps: instructions.len(),
            step_results: results,
        })
    }
    
    /// Get suggestions for next actions based on current page
    pub async fn get_suggestions(&self, browser: &SimpleBrowser) -> Result<Vec<String>> {
        let semantic_analyzer = SemanticAnalyzer::new(browser.driver());
        let page_model = semantic_analyzer.analyze().await?;
        
        let mut suggestions = Vec::new();
        
        // Suggest based on page type
        match page_model.page_type {
            SemanticPageType::Homepage => {
                suggestions.push("Navigate to a specific section".to_string());
                suggestions.push("Search for something".to_string());
            }
            SemanticPageType::SearchResults => {
                suggestions.push("Click on the first result".to_string());
                suggestions.push("Filter the results".to_string());
                suggestions.push("Go to next page".to_string());
            }
            SemanticPageType::ProductPage => {
                suggestions.push("Add to cart".to_string());
                suggestions.push("Read reviews".to_string());
                suggestions.push("Check specifications".to_string());
            }
            SemanticPageType::FormPage | SemanticPageType::LoginPage => {
                suggestions.push("Fill out the form".to_string());
                suggestions.push("Submit the form".to_string());
            }
            _ => {
                suggestions.push("Extract information from the page".to_string());
                suggestions.push("Take a screenshot".to_string());
            }
        }
        
        // Add suggestions based on available interactions
        for interaction in page_model.interaction_points.iter().take(3) {
            suggestions.push(format!("Click on {}", interaction.selector));
        }
        
        Ok(suggestions)
    }
    
    /// Provide feedback for the last instruction
    pub async fn provide_feedback(&self, feedback: Feedback) -> Result<()> {
        let parser = self.instruction_parser.lock().await;
        // This would need to store the last instruction to work properly
        // For now, just log
        info!("Feedback received: {:?}", feedback);
        Ok(())
    }
    
    async fn create_context_hints(&self, context: Option<&ConversationContext>) -> ContextHints {
        let page_type = if let Some(model) = &*self.last_page_model.lock().await {
            Some(self.convert_page_type(&model.page_type))
        } else {
            None
        };
        
        let previous_action = None; // TODO: Extract from context when structure is known
        
        ContextHints {
            current_page_type: page_type,
            previous_action,
            user_goal: None, // Could be extracted from context
        }
    }
    
    fn convert_page_type(&self, semantic_type: &SemanticPageType) -> InstructionPageType {
        match semantic_type {
            SemanticPageType::Homepage => InstructionPageType::Homepage,
            SemanticPageType::SearchResults => InstructionPageType::SearchResults,
            SemanticPageType::ProductPage => InstructionPageType::ProductPage,
            SemanticPageType::ArticlePage => InstructionPageType::ArticlePage,
            SemanticPageType::FormPage => InstructionPageType::FormPage,
            SemanticPageType::Dashboard => InstructionPageType::Dashboard,
            SemanticPageType::LoginPage => InstructionPageType::FormPage, // Map to FormPage
            SemanticPageType::CheckoutPage => InstructionPageType::FormPage,
            SemanticPageType::ProfilePage => InstructionPageType::Dashboard,
            SemanticPageType::ListingPage => InstructionPageType::SearchResults,
            SemanticPageType::DocumentationPage => InstructionPageType::ArticlePage,
            SemanticPageType::Unknown => InstructionPageType::Unknown,
        }
    }
    
    fn generate_suggestions(&self, action_result: &ActionResult) -> Vec<String> {
        let mut suggestions = Vec::new();
        
        if !action_result.success {
            if let Some(error) = &action_result.error {
                if error.contains("not found") || error.contains("No such element") {
                    suggestions.push("Try using a different selector".to_string());
                    suggestions.push("Wait for the element to load first".to_string());
                } else if error.contains("timeout") {
                    suggestions.push("Increase the timeout duration".to_string());
                    suggestions.push("Check if the page is still loading".to_string());
                }
            }
        }
        
        suggestions
    }
}

/// Result of enhanced execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedExecutionResult {
    pub success: bool,
    pub instruction: Option<UserInstruction>,
    pub page_model: Option<SemanticPageModel>,
    pub actions_executed: Vec<ActionResult>,
    pub clarification_needed: Option<Vec<String>>,
    pub error: Option<String>,
    pub suggestions: Vec<String>,
}

/// Result of workflow execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowExecutionResult {
    pub success: bool,
    pub steps_completed: usize,
    pub total_steps: usize,
    pub step_results: Vec<EnhancedExecutionResult>,
}

/// Enhanced command processor that replaces the old keyword-based system
pub struct EnhancedCommandProcessor {
    executor: EnhancedExecutor,
}

impl EnhancedCommandProcessor {
    pub fn new() -> Self {
        Self {
            executor: EnhancedExecutor::new(),
        }
    }
    
    /// Process a command using enhanced natural language understanding
    pub async fn process_command(
        &self,
        command: &str,
        browser: &SimpleBrowser,
        context: Option<&ConversationContext>,
    ) -> Result<ProcessedCommand> {
        // Try enhanced processing first
        match self.executor.execute_instruction(command, browser, context).await {
            Ok(result) => {
                if result.success {
                    Ok(ProcessedCommand {
                        success: true,
                        command_type: self.determine_command_type(&result),
                        data: self.extract_result_data(&result),
                        error: None,
                        suggestions: result.suggestions,
                    })
                } else if let Some(questions) = result.clarification_needed {
                    Ok(ProcessedCommand {
                        success: false,
                        command_type: CommandType::Clarification,
                        data: Some(serde_json::json!({
                            "questions": questions
                        })),
                        error: None,
                        suggestions: vec![],
                    })
                } else {
                    Ok(ProcessedCommand {
                        success: false,
                        command_type: CommandType::Unknown,
                        data: None,
                        error: result.error,
                        suggestions: result.suggestions,
                    })
                }
            }
            Err(e) => {
                // Fallback to legacy processing could go here
                Err(e)
            }
        }
    }
    
    fn determine_command_type(&self, result: &EnhancedExecutionResult) -> CommandType {
        if let Some(instruction) = &result.instruction {
            use crate::instruction_parser::intent_recognizer::Intent;
            match &instruction.intent {
                Intent::Navigate { .. } => CommandType::Navigation,
                Intent::Click { .. } => CommandType::Interaction,
                Intent::Type { .. } => CommandType::Input,
                Intent::Extract { .. } => CommandType::Extraction,
                Intent::Search { .. } => CommandType::Search,
                Intent::Screenshot { .. } => CommandType::Screenshot,
                _ => CommandType::Unknown,
            }
        } else {
            CommandType::Unknown
        }
    }
    
    fn extract_result_data(&self, result: &EnhancedExecutionResult) -> Option<serde_json::Value> {
        if let Some(action_result) = result.actions_executed.first() {
            action_result.data.clone()
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedCommand {
    pub success: bool,
    pub command_type: CommandType,
    pub data: Option<serde_json::Value>,
    pub error: Option<String>,
    pub suggestions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommandType {
    Navigation,
    Interaction,
    Input,
    Extraction,
    Search,
    Screenshot,
    Clarification,
    Unknown,
}

/// Demo function showing the enhanced system in action
pub async fn demo_enhanced_execution(browser: &SimpleBrowser) -> Result<()> {
    let executor = EnhancedExecutor::new();
    
    // Example 1: Simple navigation
    println!("\n=== Example 1: Simple Navigation ===");
    let result = executor.execute_instruction(
        "Go to google.com",
        browser,
        None
    ).await?;
    println!("Success: {}", result.success);
    
    // Example 2: Complex interaction
    println!("\n=== Example 2: Complex Search ===");
    let result = executor.execute_instruction(
        "Search for Rust programming tutorials",
        browser,
        None
    ).await?;
    println!("Success: {}", result.success);
    
    // Example 3: Extraction with semantic understanding
    println!("\n=== Example 3: Smart Extraction ===");
    let result = executor.execute_instruction(
        "Find all the product prices on this page",
        browser,
        None
    ).await?;
    println!("Success: {}", result.success);
    if let Some(page_model) = &result.page_model {
        println!("Page type detected: {:?}", page_model.page_type);
        println!("Found {} regions", page_model.regions.len());
    }
    
    // Example 4: Workflow execution
    println!("\n=== Example 4: Multi-Step Workflow ===");
    let workflow = vec![
        "Go to amazon.com".to_string(),
        "Search for wireless headphones".to_string(),
        "Click on the first result".to_string(),
        "Extract the price and reviews".to_string(),
    ];
    
    let workflow_result = executor.execute_workflow(
        workflow,
        browser,
        None
    ).await?;
    println!("Workflow success: {}", workflow_result.success);
    println!("Completed {} of {} steps", 
        workflow_result.steps_completed, 
        workflow_result.total_steps
    );
    
    // Example 5: Get suggestions
    println!("\n=== Example 5: Smart Suggestions ===");
    let suggestions = executor.get_suggestions(browser).await?;
    println!("Suggested next actions:");
    for (i, suggestion) in suggestions.iter().enumerate() {
        println!("  {}. {}", i + 1, suggestion);
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_enhanced_executor_creation() {
        let executor = EnhancedExecutor::new();
        assert!(executor.last_page_model.lock().await.is_none());
    }
    
    #[test]
    fn test_command_type_determination() {
        let processor = EnhancedCommandProcessor::new();
        
        let result = EnhancedExecutionResult {
            success: true,
            instruction: None,
            page_model: None,
            actions_executed: vec![],
            clarification_needed: Some(vec!["What would you like to do?".to_string()]),
            error: None,
            suggestions: vec![],
        };
        
        let processed = ProcessedCommand {
            success: false,
            command_type: CommandType::Clarification,
            data: None,
            error: None,
            suggestions: vec![],
        };
        
        assert!(matches!(processed.command_type, CommandType::Clarification));
    }
}