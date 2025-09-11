// Concurrent Action Controller
// Part of the Intelligent Action Engine

use crate::error::{Result, RainbowError};
use crate::action::{Action, ActionResult, ActionType};
use chromiumoxide::Page;
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Semaphore, RwLock};
use tokio::task::JoinHandle;
use std::collections::HashMap;

/// Controller for managing concurrent action execution
#[derive(Debug)]
pub struct ConcurrentController {
    max_concurrent_actions: usize,
    semaphore: Arc<Semaphore>,
    execution_stats: Arc<RwLock<ExecutionStats>>,
    conflict_detector: Arc<ConflictDetector>,
}

impl ConcurrentController {
    pub fn new() -> Self {
        Self {
            max_concurrent_actions: 5,
            semaphore: Arc::new(Semaphore::new(5)),
            execution_stats: Arc::new(RwLock::new(ExecutionStats::new())),
            conflict_detector: Arc::new(ConflictDetector::new()),
        }
    }

    pub fn with_max_concurrent(max_concurrent: usize) -> Self {
        Self {
            max_concurrent_actions: max_concurrent,
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            execution_stats: Arc::new(RwLock::new(ExecutionStats::new())),
            conflict_detector: Arc::new(ConflictDetector::new()),
        }
    }

    /// Execute multiple actions in parallel with intelligent coordination
    pub async fn execute_parallel(
        &self,
        page: Arc<Page>,
        actions: Vec<Action>,
        session_id: Option<String>,
    ) -> Result<Vec<ActionResult>> {
        if actions.is_empty() {
            return Ok(Vec::new());
        }

        let start_time = Instant::now();
        
        // Analyze actions for conflicts
        let conflict_groups = self.conflict_detector
            .analyze_conflicts(&actions)
            .await?;

        // Execute conflict-free groups in parallel
        let mut all_results = Vec::new();
        
        for group in conflict_groups {
            let group_results = self.execute_conflict_free_group(
                page.clone(),
                group,
                session_id.clone(),
            ).await?;
            
            all_results.extend(group_results);
        }

        // Update execution statistics
        let execution_time = start_time.elapsed();
        self.update_stats(&actions, &all_results, execution_time).await;

        // Sort results by original action order
        all_results.sort_by_key(|result| {
            actions.iter()
                .position(|action| action.id == result.action_id)
                .unwrap_or(usize::MAX)
        });

        Ok(all_results)
    }

    /// Execute a group of actions that don't conflict with each other
    async fn execute_conflict_free_group(
        &self,
        page: Arc<Page>,
        actions: Vec<Action>,
        session_id: Option<String>,
    ) -> Result<Vec<ActionResult>> {
        let mut handles: Vec<JoinHandle<Result<ActionResult>>> = Vec::new();

        // Launch all actions in the group concurrently
        for action in actions {
            let page_clone = page.clone();
            let session_id_clone = session_id.clone();
            let semaphore = self.semaphore.clone();

            let handle = tokio::spawn(async move {
                // Acquire semaphore permit to limit concurrency
                let _permit = semaphore.acquire().await
                    .map_err(|e| RainbowError::ExecutionError(format!("Semaphore error: {}", e)))?;

                // Execute the action
                Self::execute_single_action(page_clone, action, session_id_clone).await
            });

            handles.push(handle);
        }

        // Wait for all actions to complete
        let mut results = Vec::new();
        for handle in handles {
            match handle.await {
                Ok(result) => results.push(result?),
                Err(e) => {
                    results.push(ActionResult {
                        action_id: uuid::Uuid::new_v4(), // We lost the original ID
                        success: false,
                        execution_time: Duration::default(),
                        attempts: 1,
                        error: Some(format!("Task join error: {}", e)),
                        element_info: None,
                        screenshot_path: None,
                        verification_result: None,
                        metadata: serde_json::Value::Null,
                    });
                }
            }
        }

        Ok(results)
    }

    /// Execute a single action (placeholder implementation)
    async fn execute_single_action(
        page: Arc<Page>,
        action: Action,
        session_id: Option<String>,
    ) -> Result<ActionResult> {
        let start_time = Instant::now();
        
        // This is a simplified implementation
        // In reality, this would use the actual action executor
        let result = match action.action_type {
            ActionType::Click => {
                // Simulate click execution
                tokio::time::sleep(Duration::from_millis(100)).await;
                Ok(())
            }
            ActionType::Type(ref text) => {
                // Simulate typing execution
                let delay = Duration::from_millis(10 * text.len() as u64);
                tokio::time::sleep(delay).await;
                Ok(())
            }
            ActionType::Navigate(ref url) => {
                // Simulate navigation
                tokio::time::sleep(Duration::from_millis(500)).await;
                Ok(())
            }
            ActionType::Wait(duration) => {
                tokio::time::sleep(duration).await;
                Ok(())
            }
            ActionType::Screenshot => {
                // Simulate screenshot
                tokio::time::sleep(Duration::from_millis(200)).await;
                Ok(())
            }
            _ => {
                // Generic simulation for other actions
                tokio::time::sleep(Duration::from_millis(50)).await;
                Ok(())
            }
        };

        let execution_time = start_time.elapsed();

        Ok(ActionResult {
            action_id: action.id,
            success: result.is_ok(),
            execution_time,
            attempts: 1,
            error: result.err().map(|e| e.to_string()),
            element_info: None,
            screenshot_path: None,
            verification_result: None,
            metadata: serde_json::json!({
                "session_id": session_id,
                "concurrent_execution": true,
                "action_type": action.action_type
            }),
        })
    }

    /// Get current execution statistics
    pub async fn get_stats(&self) -> ExecutionStats {
        self.execution_stats.read().await.clone()
    }

    /// Reset execution statistics
    pub async fn reset_stats(&self) {
        *self.execution_stats.write().await = ExecutionStats::new();
    }

    async fn update_stats(
        &self,
        actions: &[Action],
        results: &[ActionResult],
        execution_time: Duration,
    ) {
        let mut stats = self.execution_stats.write().await;
        
        stats.total_parallel_batches += 1;
        stats.total_actions_executed += actions.len() as u32;
        stats.total_execution_time += execution_time;
        
        let successful_actions = results.iter().filter(|r| r.success).count() as u32;
        stats.successful_actions += successful_actions;
        
        // Update success rate
        stats.success_rate = if stats.total_actions_executed > 0 {
            stats.successful_actions as f64 / stats.total_actions_executed as f64
        } else {
            0.0
        };

        // Update average execution time
        stats.average_execution_time = if stats.total_parallel_batches > 0 {
            stats.total_execution_time / stats.total_parallel_batches
        } else {
            Duration::default()
        };

        // Track concurrency efficiency
        if actions.len() > 1 {
            let theoretical_sequential_time: Duration = results
                .iter()
                .map(|r| r.execution_time)
                .sum();
            
            let efficiency = if execution_time.as_millis() > 0 {
                theoretical_sequential_time.as_millis() as f64 / execution_time.as_millis() as f64
            } else {
                1.0
            };

            // Update efficiency using exponential moving average
            let alpha = 0.2;
            stats.concurrency_efficiency = alpha * efficiency + (1.0 - alpha) * stats.concurrency_efficiency;
        }
    }
}

impl Default for ConcurrentController {
    fn default() -> Self {
        Self::new()
    }
}

/// Conflict detector to identify actions that cannot run in parallel
#[derive(Debug)]
struct ConflictDetector {
    conflict_rules: Vec<Box<dyn ConflictRule + Send + Sync>>,
}

impl ConflictDetector {
    fn new() -> Self {
        Self {
            conflict_rules: vec![
                Box::new(SameElementConflictRule),
                Box::new(NavigationConflictRule),
                Box::new(PageModificationConflictRule),
                Box::new(ResourceConflictRule),
            ],
        }
    }

    async fn analyze_conflicts(&self, actions: &[Action]) -> Result<Vec<Vec<Action>>> {
        if actions.len() <= 1 {
            return Ok(vec![actions.to_vec()]);
        }

        let mut conflict_matrix = vec![vec![false; actions.len()]; actions.len()];
        
        // Build conflict matrix
        for i in 0..actions.len() {
            for j in i + 1..actions.len() {
                let has_conflict = self.check_conflict(&actions[i], &actions[j]).await;
                conflict_matrix[i][j] = has_conflict;
                conflict_matrix[j][i] = has_conflict;
            }
        }

        // Group actions into conflict-free sets
        let groups = self.create_conflict_free_groups(actions, &conflict_matrix);
        Ok(groups)
    }

    async fn check_conflict(&self, action1: &Action, action2: &Action) -> bool {
        for rule in &self.conflict_rules {
            if rule.has_conflict(action1, action2).await {
                return true;
            }
        }
        false
    }

    fn create_conflict_free_groups(
        &self,
        actions: &[Action],
        conflict_matrix: &[Vec<bool>],
    ) -> Vec<Vec<Action>> {
        let mut groups = Vec::new();
        let mut assigned = vec![false; actions.len()];

        for i in 0..actions.len() {
            if assigned[i] {
                continue;
            }

            let mut group = vec![actions[i].clone()];
            assigned[i] = true;

            // Try to add more actions to this group
            for j in i + 1..actions.len() {
                if assigned[j] {
                    continue;
                }

                // Check if action j conflicts with any action in the current group
                let mut can_add = true;
                for (group_idx, group_action) in group.iter().enumerate() {
                    let original_idx = actions.iter().position(|a| a.id == group_action.id).unwrap();
                    if conflict_matrix[original_idx][j] {
                        can_add = false;
                        break;
                    }
                }

                if can_add {
                    group.push(actions[j].clone());
                    assigned[j] = true;
                }
            }

            groups.push(group);
        }

        groups
    }
}

/// Execution statistics for concurrent operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStats {
    pub total_parallel_batches: u32,
    pub total_actions_executed: u32,
    pub successful_actions: u32,
    pub success_rate: f64,
    pub total_execution_time: Duration,
    pub average_execution_time: Duration,
    pub concurrency_efficiency: f64, // How well we utilize parallelism
    pub max_concurrent_achieved: usize,
}

impl ExecutionStats {
    fn new() -> Self {
        Self {
            total_parallel_batches: 0,
            total_actions_executed: 0,
            successful_actions: 0,
            success_rate: 0.0,
            total_execution_time: Duration::default(),
            average_execution_time: Duration::default(),
            concurrency_efficiency: 1.0,
            max_concurrent_achieved: 0,
        }
    }
}

/// Trait for conflict detection rules
#[async_trait::async_trait]
trait ConflictRule: std::fmt::Debug {
    fn name(&self) -> &'static str;
    async fn has_conflict(&self, action1: &Action, action2: &Action) -> bool;
}

/// Rule: Actions targeting the same element conflict
#[derive(Debug)]
struct SameElementConflictRule;

#[async_trait::async_trait]
impl ConflictRule for SameElementConflictRule {
    fn name(&self) -> &'static str { "SameElementConflict" }
    
    async fn has_conflict(&self, action1: &Action, action2: &Action) -> bool {
        // Compare targets to see if they might reference the same element
        match (&action1.target, &action2.target) {
            (crate::action::ActionTarget::Selector(s1), crate::action::ActionTarget::Selector(s2)) => s1 == s2,
            (crate::action::ActionTarget::Id(id1), crate::action::ActionTarget::Id(id2)) => id1 == id2,
            (crate::action::ActionTarget::XPath(x1), crate::action::ActionTarget::XPath(x2)) => x1 == x2,
            _ => false, // Different target types generally don't conflict
        }
    }
}

/// Rule: Navigation actions conflict with everything
#[derive(Debug)]
struct NavigationConflictRule;

#[async_trait::async_trait]
impl ConflictRule for NavigationConflictRule {
    fn name(&self) -> &'static str { "NavigationConflict" }
    
    async fn has_conflict(&self, action1: &Action, action2: &Action) -> bool {
        matches!(action1.action_type, 
            ActionType::Navigate(_) | 
            ActionType::GoBack | 
            ActionType::GoForward | 
            ActionType::Refresh
        ) || matches!(action2.action_type, 
            ActionType::Navigate(_) | 
            ActionType::GoBack | 
            ActionType::GoForward | 
            ActionType::Refresh
        )
    }
}

/// Rule: Actions that modify page state conflict
#[derive(Debug)]
struct PageModificationConflictRule;

#[async_trait::async_trait]
impl ConflictRule for PageModificationConflictRule {
    fn name(&self) -> &'static str { "PageModificationConflict" }
    
    async fn has_conflict(&self, action1: &Action, action2: &Action) -> bool {
        let modifying_actions = [
            std::mem::discriminant(&ActionType::Submit),
            std::mem::discriminant(&ActionType::Clear),
        ];

        let action1_modifies = modifying_actions.contains(&std::mem::discriminant(&action1.action_type));
        let action2_modifies = modifying_actions.contains(&std::mem::discriminant(&action2.action_type));

        // If both actions modify the page, they conflict
        action1_modifies && action2_modifies
    }
}

/// Rule: Actions competing for limited resources
#[derive(Debug)]
struct ResourceConflictRule;

#[async_trait::async_trait]
impl ConflictRule for ResourceConflictRule {
    fn name(&self) -> &'static str { "ResourceConflict" }
    
    async fn has_conflict(&self, action1: &Action, action2: &Action) -> bool {
        // For example, multiple file uploads might conflict
        matches!(action1.action_type, ActionType::Upload(_)) && 
        matches!(action2.action_type, ActionType::Upload(_))
    }
}

/// Configuration for concurrent execution
#[derive(Debug, Clone)]
pub struct ConcurrencyConfig {
    pub max_concurrent: usize,
    pub enable_conflict_detection: bool,
    pub timeout_per_action: Duration,
    pub batch_timeout: Duration,
}

impl Default for ConcurrencyConfig {
    fn default() -> Self {
        Self {
            max_concurrent: 5,
            enable_conflict_detection: true,
            timeout_per_action: Duration::from_secs(10),
            batch_timeout: Duration::from_secs(60),
        }
    }
}

/// Builder for concurrent execution configuration
pub struct ConcurrencyConfigBuilder {
    config: ConcurrencyConfig,
}

impl ConcurrencyConfigBuilder {
    pub fn new() -> Self {
        Self {
            config: ConcurrencyConfig::default(),
        }
    }

    pub fn max_concurrent(mut self, max: usize) -> Self {
        self.config.max_concurrent = max;
        self
    }

    pub fn enable_conflict_detection(mut self, enabled: bool) -> Self {
        self.config.enable_conflict_detection = enabled;
        self
    }

    pub fn timeout_per_action(mut self, timeout: Duration) -> Self {
        self.config.timeout_per_action = timeout;
        self
    }

    pub fn batch_timeout(mut self, timeout: Duration) -> Self {
        self.config.batch_timeout = timeout;
        self
    }

    pub fn build(self) -> ConcurrencyConfig {
        self.config
    }
}

impl Default for ConcurrencyConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::ActionTarget;

    #[tokio::test]
    async fn test_concurrent_controller_creation() {
        let controller = ConcurrentController::new();
        assert_eq!(controller.max_concurrent_actions, 5);
    }

    #[tokio::test]
    async fn test_conflict_detection() {
        let detector = ConflictDetector::new();
        
        let action1 = Action::new(
            ActionType::Click,
            ActionTarget::Selector("#button".to_string())
        );
        
        let action2 = Action::new(
            ActionType::Type("test".to_string()),
            ActionTarget::Selector("#button".to_string())
        );

        let has_conflict = detector.check_conflict(&action1, &action2).await;
        assert!(has_conflict); // Same selector should conflict
    }

    #[tokio::test]
    async fn test_navigation_conflicts() {
        let detector = ConflictDetector::new();
        
        let nav_action = Action::new(
            ActionType::Navigate("https://example.com".to_string()),
            ActionTarget::Coordinate { x: 0, y: 0 }
        );
        
        let click_action = Action::new(
            ActionType::Click,
            ActionTarget::Selector("#button".to_string())
        );

        let has_conflict = detector.check_conflict(&nav_action, &click_action).await;
        assert!(has_conflict); // Navigation conflicts with everything
    }

    #[tokio::test]
    async fn test_conflict_free_grouping() {
        let detector = ConflictDetector::new();
        
        let actions = vec![
            Action::new(ActionType::Click, ActionTarget::Selector("#button1".to_string())),
            Action::new(ActionType::Click, ActionTarget::Selector("#button2".to_string())),
            Action::new(ActionType::Navigate("https://example.com".to_string()), ActionTarget::Coordinate { x: 0, y: 0 }),
        ];

        let groups = detector.analyze_conflicts(&actions).await.unwrap();
        
        // Navigation should be in its own group
        assert!(groups.len() >= 2);
        
        // Check that navigation action is isolated
        let nav_group = groups.iter()
            .find(|group| group.iter().any(|action| matches!(action.action_type, ActionType::Navigate(_))))
            .unwrap();
        assert_eq!(nav_group.len(), 1);
    }

    #[test]
    fn test_execution_stats() {
        let stats = ExecutionStats::new();
        assert_eq!(stats.total_parallel_batches, 0);
        assert_eq!(stats.success_rate, 0.0);
        assert_eq!(stats.concurrency_efficiency, 1.0);
    }

    #[test]
    fn test_concurrency_config_builder() {
        let config = ConcurrencyConfigBuilder::new()
            .max_concurrent(10)
            .enable_conflict_detection(false)
            .timeout_per_action(Duration::from_secs(5))
            .build();

        assert_eq!(config.max_concurrent, 10);
        assert!(!config.enable_conflict_detection);
        assert_eq!(config.timeout_per_action, Duration::from_secs(5));
    }
}