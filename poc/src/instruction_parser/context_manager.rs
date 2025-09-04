//! Context Management Module
//! 
//! Maintains conversation context and state for better instruction understanding

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::time::Duration;

use super::{UserInstruction, ContextHints, PageType};
use super::intent_recognizer::Intent;

/// Maximum number of instructions to keep in history
const MAX_HISTORY_SIZE: usize = 20;

/// Maximum time to keep context valid (5 minutes)
const CONTEXT_VALIDITY_DURATION: Duration = Duration::from_secs(300);

/// Instruction context for maintaining conversation state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstructionContext {
    pub session_id: String,
    pub current_page: Option<PageInfo>,
    pub recent_actions: VecDeque<ActionRecord>,
    pub user_preferences: UserPreferences,
    pub task_stack: Vec<Task>,
    pub variables: HashMap<String, String>,
    pub last_error: Option<ErrorContext>,
    pub created_at: std::time::SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageInfo {
    pub url: String,
    pub title: String,
    pub page_type: PageType,
    pub key_elements: Vec<String>,
    pub timestamp: std::time::SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionRecord {
    pub instruction: String,
    pub intent: Intent,
    pub success: bool,
    pub timestamp: std::time::SystemTime,
    pub execution_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    pub default_timeout_ms: u64,
    pub auto_screenshots: bool,
    pub verbose_mode: bool,
    pub preferred_selectors: HashMap<String, String>, // Mapping of descriptions to selectors
    pub common_values: HashMap<String, String>, // Common inputs like email, name, etc.
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            default_timeout_ms: 30000,
            auto_screenshots: false,
            verbose_mode: false,
            preferred_selectors: HashMap::new(),
            common_values: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub description: String,
    pub steps: Vec<String>,
    pub current_step: usize,
    pub status: TaskStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskStatus {
    NotStarted,
    InProgress,
    Completed,
    Failed,
    Paused,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    pub message: String,
    pub instruction: String,
    pub recovery_suggestions: Vec<String>,
    pub timestamp: std::time::SystemTime,
}

/// Context manager for maintaining conversation state
pub struct ContextManager {
    current_context: InstructionContext,
    context_history: VecDeque<InstructionContext>,
    pattern_cache: HashMap<String, Intent>,
    success_patterns: HashMap<String, f32>, // Pattern -> success rate
}

impl ContextManager {
    pub fn new() -> Self {
        Self {
            current_context: InstructionContext::new(),
            context_history: VecDeque::with_capacity(MAX_HISTORY_SIZE),
            pattern_cache: HashMap::new(),
            success_patterns: HashMap::new(),
        }
    }
    
    pub fn get_current_context(&self) -> InstructionContext {
        self.current_context.clone()
    }
    
    pub fn add_instruction(&mut self, instruction: &UserInstruction) {
        // Add to recent actions
        self.current_context.recent_actions.push_back(ActionRecord {
            instruction: instruction.raw_text.clone(),
            intent: instruction.intent.clone(),
            success: instruction.confidence > 0.7,
            timestamp: std::time::SystemTime::now(),
            execution_time_ms: 0, // Will be updated after execution
        });
        
        // Keep history bounded
        if self.current_context.recent_actions.len() > MAX_HISTORY_SIZE {
            self.current_context.recent_actions.pop_front();
        }
        
        // Update pattern cache
        self.pattern_cache.insert(
            instruction.normalized_text.clone(),
            instruction.intent.clone()
        );
    }
    
    pub fn apply_hints(&mut self, hints: ContextHints) {
        if let Some(page_type) = hints.current_page_type {
            if let Some(ref mut page_info) = self.current_context.current_page {
                page_info.page_type = page_type;
            }
        }
        
        if let Some(goal) = hints.user_goal {
            // Add or update current task
            if let Some(task) = self.current_context.task_stack.last_mut() {
                task.description = goal;
            } else {
                self.current_context.task_stack.push(Task {
                    id: uuid::Uuid::new_v4().to_string(),
                    description: goal,
                    steps: Vec::new(),
                    current_step: 0,
                    status: TaskStatus::NotStarted,
                });
            }
        }
    }
    
    pub fn update_page_info(&mut self, url: String, title: String) {
        self.current_context.current_page = Some(PageInfo {
            url,
            title,
            page_type: PageType::Unknown,
            key_elements: Vec::new(),
            timestamp: std::time::SystemTime::now(),
        });
    }
    
    pub fn get_last_successful_pattern(&self, similar_to: &str) -> Option<Intent> {
        // Look for similar successful patterns
        let similar_key = self.find_similar_pattern(similar_to);
        similar_key.and_then(|key| self.pattern_cache.get(&key).cloned())
    }
    
    pub fn record_success(&mut self, instruction: &str, execution_time_ms: u64) {
        // Update last action with success
        if let Some(last_action) = self.current_context.recent_actions.back_mut() {
            last_action.success = true;
            last_action.execution_time_ms = execution_time_ms;
        }
        
        // Update success rate
        let current_rate = self.success_patterns.get(instruction).unwrap_or(&0.0);
        self.success_patterns.insert(
            instruction.to_string(),
            (current_rate + 1.0) / 2.0 // Moving average
        );
    }
    
    pub fn record_error(&mut self, instruction: &str, error: String, suggestions: Vec<String>) {
        self.current_context.last_error = Some(ErrorContext {
            message: error,
            instruction: instruction.to_string(),
            recovery_suggestions: suggestions,
            timestamp: std::time::SystemTime::now(),
        });
        
        // Update failure rate
        let current_rate = self.success_patterns.get(instruction).unwrap_or(&1.0);
        self.success_patterns.insert(
            instruction.to_string(),
            current_rate * 0.5 // Reduce success rate
        );
    }
    
    pub fn get_user_preference(&self, key: &str) -> Option<String> {
        self.current_context.user_preferences.common_values.get(key).cloned()
    }
    
    pub fn set_user_preference(&mut self, key: String, value: String) {
        self.current_context.user_preferences.common_values.insert(key, value);
    }
    
    pub fn get_preferred_selector(&self, description: &str) -> Option<String> {
        self.current_context.user_preferences.preferred_selectors.get(description).cloned()
    }
    
    pub fn learn_selector_preference(&mut self, description: String, selector: String) {
        self.current_context.user_preferences.preferred_selectors.insert(description, selector);
    }
    
    pub fn get_variable(&self, name: &str) -> Option<String> {
        self.current_context.variables.get(name).cloned()
    }
    
    pub fn set_variable(&mut self, name: String, value: String) {
        self.current_context.variables.insert(name, value);
    }
    
    pub fn push_task(&mut self, task: Task) {
        self.current_context.task_stack.push(task);
    }
    
    pub fn pop_task(&mut self) -> Option<Task> {
        self.current_context.task_stack.pop()
    }
    
    pub fn get_current_task(&self) -> Option<&Task> {
        self.current_context.task_stack.last()
    }
    
    pub fn update_current_task_status(&mut self, status: TaskStatus) {
        if let Some(task) = self.current_context.task_stack.last_mut() {
            task.status = status;
        }
    }
    
    pub fn advance_task_step(&mut self) {
        if let Some(task) = self.current_context.task_stack.last_mut() {
            task.current_step += 1;
            if task.current_step >= task.steps.len() {
                task.status = TaskStatus::Completed;
            }
        }
    }
    
    pub fn save_checkpoint(&mut self) {
        // Save current context to history
        self.context_history.push_back(self.current_context.clone());
        if self.context_history.len() > MAX_HISTORY_SIZE {
            self.context_history.pop_front();
        }
    }
    
    pub fn restore_checkpoint(&mut self, steps_back: usize) -> Result<()> {
        if steps_back == 0 || steps_back > self.context_history.len() {
            return Err(anyhow::anyhow!("Invalid checkpoint index"));
        }
        
        if let Some(context) = self.context_history.iter().rev().nth(steps_back - 1) {
            self.current_context = context.clone();
            Ok(())
        } else {
            Err(anyhow::anyhow!("Checkpoint not found"))
        }
    }
    
    pub fn clear_context(&mut self) {
        self.current_context = InstructionContext::new();
        self.pattern_cache.clear();
    }
    
    pub fn is_context_valid(&self) -> bool {
        // Check if context is still valid based on age
        if let Ok(elapsed) = self.current_context.created_at.elapsed() {
            elapsed < CONTEXT_VALIDITY_DURATION
        } else {
            false
        }
    }
    
    pub fn get_context_suggestions(&self) -> Vec<String> {
        let mut suggestions = Vec::new();
        
        // Suggest based on current page type
        if let Some(ref page_info) = self.current_context.current_page {
            match page_info.page_type {
                PageType::FormPage => {
                    suggestions.push("Fill out the form with your information".to_string());
                    suggestions.push("Submit the form".to_string());
                }
                PageType::SearchResults => {
                    suggestions.push("Click on the first result".to_string());
                    suggestions.push("Go to the next page".to_string());
                }
                PageType::ProductPage => {
                    suggestions.push("Add to cart".to_string());
                    suggestions.push("View reviews".to_string());
                }
                _ => {}
            }
        }
        
        // Suggest based on recent patterns
        for (pattern, rate) in &self.success_patterns {
            if *rate > 0.8 {
                suggestions.push(pattern.clone());
            }
        }
        
        suggestions
    }
    
    fn find_similar_pattern(&self, text: &str) -> Option<String> {
        // Simple similarity check - could be improved with fuzzy matching
        let lower = text.to_lowercase();
        
        for key in self.pattern_cache.keys() {
            if key.contains(&lower) || lower.contains(key) {
                return Some(key.clone());
            }
        }
        
        None
    }
}

impl InstructionContext {
    pub fn new() -> Self {
        Self {
            session_id: uuid::Uuid::new_v4().to_string(),
            current_page: None,
            recent_actions: VecDeque::with_capacity(MAX_HISTORY_SIZE),
            user_preferences: UserPreferences::default(),
            task_stack: Vec::new(),
            variables: HashMap::new(),
            last_error: None,
            created_at: std::time::SystemTime::now(),
        }
    }
    
    pub fn with_page(mut self, url: String, title: String) -> Self {
        self.current_page = Some(PageInfo {
            url,
            title,
            page_type: PageType::Unknown,
            key_elements: Vec::new(),
            timestamp: std::time::SystemTime::now(),
        });
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instruction_parser::intent_recognizer::NavigationTarget;
    
    #[test]
    fn test_context_management() {
        let mut manager = ContextManager::new();
        
        // Update page info
        manager.update_page_info("https://example.com".to_string(), "Example".to_string());
        
        let context = manager.get_current_context();
        assert!(context.current_page.is_some());
        assert_eq!(context.current_page.unwrap().url, "https://example.com");
    }
    
    #[test]
    fn test_preference_learning() {
        let mut manager = ContextManager::new();
        
        // Learn selector preference
        manager.learn_selector_preference(
            "submit button".to_string(),
            "#submit-btn".to_string()
        );
        
        assert_eq!(
            manager.get_preferred_selector("submit button"),
            Some("#submit-btn".to_string())
        );
    }
    
    #[test]
    fn test_task_management() {
        let mut manager = ContextManager::new();
        
        // Add a task
        let task = Task {
            id: "1".to_string(),
            description: "Complete purchase".to_string(),
            steps: vec![
                "Add item to cart".to_string(),
                "Go to checkout".to_string(),
                "Enter payment".to_string(),
            ],
            current_step: 0,
            status: TaskStatus::NotStarted,
        };
        
        manager.push_task(task);
        assert!(manager.get_current_task().is_some());
        
        // Advance task
        manager.advance_task_step();
        assert_eq!(manager.get_current_task().unwrap().current_step, 1);
    }
    
    #[test]
    fn test_checkpoint_restore() {
        let mut manager = ContextManager::new();
        
        // Create initial state
        manager.set_variable("test".to_string(), "value1".to_string());
        manager.save_checkpoint();
        
        // Modify state
        manager.set_variable("test".to_string(), "value2".to_string());
        assert_eq!(manager.get_variable("test"), Some("value2".to_string()));
        
        // Restore checkpoint
        manager.restore_checkpoint(1).unwrap();
        assert_eq!(manager.get_variable("test"), Some("value1".to_string()));
    }
}