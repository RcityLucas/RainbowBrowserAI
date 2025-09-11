// 智能行动引擎 (Intelligent Action Engine)
// RainbowBrowserAI 8.0 - Third Engine Implementation

pub mod element_locator;
pub mod verification_engine;
pub mod retry_mechanism;
pub mod concurrent_controller;
pub mod action_executor;
pub mod chain_optimizer;

use crate::error::Result;
use chromiumoxide::{Page, Element};
use serde::{Serialize, Deserialize};
use std::time::{Duration, Instant};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

// Re-exports
pub use element_locator::*;
pub use verification_engine::*;
pub use retry_mechanism::*;
pub use concurrent_controller::*;
pub use action_executor::*;
pub use chain_optimizer::*;

/// Action types supported by the intelligent action engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionType {
    Click,
    DoubleClick,
    RightClick,
    Type(String),
    Clear,
    Submit,
    ScrollTo,
    Hover,
    Focus,
    Select(String),
    Upload(String),
    KeyPress(String),
    Wait(Duration),
    Screenshot,
    Navigate(String),
    GoBack,
    GoForward,
    Refresh,
}

/// Target specification for actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionTarget {
    Selector(String),
    XPath(String),
    Text(String),
    Id(String),
    Class(String),
    Name(String),
    Placeholder(String),
    Value(String),
    Role(String),
    Coordinate { x: i32, y: i32 },
    Element(String), // Element ID from previous action
}

/// Action configuration and execution context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    pub id: Uuid,
    pub action_type: ActionType,
    pub target: ActionTarget,
    pub timeout: Duration,
    pub retry_count: u32,
    pub verify_result: bool,
    pub description: Option<String>,
    pub metadata: serde_json::Value,
}

impl Action {
    pub fn new(action_type: ActionType, target: ActionTarget) -> Self {
        Self {
            id: Uuid::new_v4(),
            action_type,
            target,
            timeout: Duration::from_secs(10),
            retry_count: 3,
            verify_result: true,
            description: None,
            metadata: serde_json::Value::Null,
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn with_retry_count(mut self, retry_count: u32) -> Self {
        self.retry_count = retry_count;
        self
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    pub fn without_verification(mut self) -> Self {
        self.verify_result = false;
        self
    }
}

/// Result of action execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionResult {
    pub action_id: Uuid,
    pub success: bool,
    pub execution_time: Duration,
    pub attempts: u32,
    pub error: Option<String>,
    pub element_info: Option<ElementInfo>,
    pub screenshot_path: Option<String>,
    pub verification_result: Option<VerificationResult>,
    pub metadata: serde_json::Value,
}

/// Element information captured during action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementInfo {
    pub tag_name: String,
    pub attributes: std::collections::HashMap<String, String>,
    pub text_content: Option<String>,
    pub bounding_box: Option<BoundingBox>,
    pub is_visible: bool,
    pub is_enabled: bool,
}

/// Bounding box for element positioning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundingBox {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// Action chain for executing multiple actions in sequence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionChain {
    pub id: Uuid,
    pub actions: Vec<Action>,
    pub stop_on_failure: bool,
    pub parallel_groups: Vec<Vec<usize>>, // Indices of actions that can run in parallel
    pub description: Option<String>,
}

impl ActionChain {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            actions: Vec::new(),
            stop_on_failure: true,
            parallel_groups: Vec::new(),
            description: None,
        }
    }

    pub fn add_action(mut self, action: Action) -> Self {
        self.actions.push(action);
        self
    }

    pub fn add_parallel_group(mut self, action_indices: Vec<usize>) -> Self {
        self.parallel_groups.push(action_indices);
        self
    }

    pub fn continue_on_failure(mut self) -> Self {
        self.stop_on_failure = false;
        self
    }
}

/// Main intelligent action engine
#[derive(Debug)]
pub struct IntelligentActionEngine {
    element_locator: Arc<ElementLocator>,
    verification_engine: Arc<VerificationEngine>,
    retry_mechanism: Arc<RetryMechanism>,
    concurrent_controller: Arc<ConcurrentController>,
    action_executor: Arc<ActionExecutor>,
    chain_optimizer: Arc<ChainOptimizer>,
    session_cache: Arc<RwLock<std::collections::HashMap<String, SessionActionCache>>>,
}

#[derive(Debug, Clone)]
struct SessionActionCache {
    element_cache: std::collections::HashMap<String, Element>,
    last_action_time: Instant,
    success_rate: f64,
}

impl Default for IntelligentActionEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl IntelligentActionEngine {
    pub fn new() -> Self {
        Self {
            element_locator: Arc::new(ElementLocator::new()),
            verification_engine: Arc::new(VerificationEngine::new()),
            retry_mechanism: Arc::new(RetryMechanism::new()),
            concurrent_controller: Arc::new(ConcurrentController::new()),
            action_executor: Arc::new(ActionExecutor::new()),
            chain_optimizer: Arc::new(ChainOptimizer::new()),
            session_cache: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Execute a single action with intelligent retry and verification
    pub async fn execute_action(
        &self,
        page: Arc<Page>,
        action: Action,
        session_id: Option<String>,
    ) -> Result<ActionResult> {
        let start_time = Instant::now();
        let mut attempts = 0;

        // Try to execute with retry mechanism
        let result = self.retry_mechanism
            .execute_with_retry(|| async {
                attempts += 1;
                
                // Locate element using advanced strategies
                let element = self.element_locator
                    .locate_element(page.clone(), &action.target)
                    .await?;

                // Execute the action
                let execution_result = self.action_executor
                    .execute(page.clone(), &action, element.clone())
                    .await?;

                // Verify result if requested
                let verification_result = if action.verify_result {
                    Some(self.verification_engine
                        .verify_action_result(page.clone(), &action, &element)
                        .await?)
                } else {
                    None
                };

                Ok((execution_result, verification_result))
            })
            .await;

        let execution_time = start_time.elapsed();

        // Build result
        let action_result = match result {
            Ok((execution_result, verification_result)) => {
                ActionResult {
                    action_id: action.id,
                    success: true,
                    execution_time,
                    attempts,
                    error: None,
                    element_info: execution_result.element_info,
                    screenshot_path: execution_result.screenshot_path,
                    verification_result,
                    metadata: serde_json::json!({
                        "session_id": session_id,
                        "action_type": action.action_type,
                        "target": action.target
                    }),
                }
            },
            Err(e) => {
                ActionResult {
                    action_id: action.id,
                    success: false,
                    execution_time,
                    attempts,
                    error: Some(e.to_string()),
                    element_info: None,
                    screenshot_path: None,
                    verification_result: None,
                    metadata: serde_json::json!({
                        "session_id": session_id,
                        "action_type": action.action_type,
                        "target": action.target,
                        "error": e.to_string()
                    }),
                }
            }
        };

        // Update session cache if available
        if let Some(session_id) = session_id {
            self.update_session_cache(&session_id, &action_result).await;
        }

        Ok(action_result)
    }

    /// Execute an action chain with optimization and parallelization
    pub async fn execute_action_chain(
        &self,
        page: Arc<Page>,
        mut chain: ActionChain,
        session_id: Option<String>,
    ) -> Result<Vec<ActionResult>> {
        // Optimize the chain before execution
        chain = self.chain_optimizer
            .optimize_chain(page.clone(), chain)
            .await?;

        let mut results = Vec::new();
        let mut action_results = std::collections::HashMap::new();

        // Execute sequential actions
        for (index, action) in chain.actions.iter().enumerate() {
            // Check if this action is part of a parallel group
            let is_parallel = chain.parallel_groups
                .iter()
                .any(|group| group.contains(&index));

            if !is_parallel {
                let result = self.execute_action(
                    page.clone(), 
                    action.clone(), 
                    session_id.clone()
                ).await?;

                if !result.success && chain.stop_on_failure {
                    results.push(result);
                    break;
                }

                action_results.insert(index, result.clone());
                results.push(result);
            }
        }

        // Execute parallel groups
        for group in chain.parallel_groups {
            let group_actions: Vec<_> = group.iter()
                .filter_map(|&i| chain.actions.get(i))
                .cloned()
                .collect();

            let parallel_results = self.concurrent_controller
                .execute_parallel(page.clone(), group_actions, session_id.clone())
                .await?;

            for (i, result) in group.iter().zip(parallel_results.iter()) {
                action_results.insert(*i, result.clone());
                if !result.success && chain.stop_on_failure {
                    results.extend(parallel_results);
                    return Ok(results);
                }
            }

            results.extend(parallel_results);
        }

        // Sort results by original action order
        results.sort_by_key(|r| {
            chain.actions.iter().position(|a| a.id == r.action_id).unwrap_or(usize::MAX)
        });

        Ok(results)
    }

    /// Get action statistics for a session
    pub async fn get_session_stats(&self, session_id: &str) -> Option<SessionActionStats> {
        let cache = self.session_cache.read().await;
        cache.get(session_id).map(|session_cache| {
            SessionActionStats {
                success_rate: session_cache.success_rate,
                cached_elements: session_cache.element_cache.len(),
                last_activity: session_cache.last_action_time,
            }
        })
    }

    async fn update_session_cache(&self, session_id: &str, result: &ActionResult) {
        let mut cache = self.session_cache.write().await;
        let session_cache = cache.entry(session_id.to_string())
            .or_insert_with(|| SessionActionCache {
                element_cache: std::collections::HashMap::new(),
                last_action_time: Instant::now(),
                success_rate: 1.0,
            });

        session_cache.last_action_time = Instant::now();
        
        // Update success rate using exponential moving average
        let alpha = 0.1;
        let success_value = if result.success { 1.0 } else { 0.0 };
        session_cache.success_rate = alpha * success_value + (1.0 - alpha) * session_cache.success_rate;
    }
}

/// Statistics for session actions
#[derive(Debug, Clone, Serialize)]
pub struct SessionActionStats {
    pub success_rate: f64,
    pub cached_elements: usize,
    pub last_activity: Instant,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_creation() {
        let action = Action::new(
            ActionType::Click,
            ActionTarget::Selector("#button".to_string())
        ).with_timeout(Duration::from_secs(5))
         .with_description("Click submit button".to_string());

        assert_eq!(action.timeout, Duration::from_secs(5));
        assert_eq!(action.description, Some("Click submit button".to_string()));
        assert_eq!(action.retry_count, 3);
        assert!(action.verify_result);
    }

    #[test]
    fn test_action_chain_creation() {
        let chain = ActionChain::new()
            .add_action(Action::new(
                ActionType::Type("test@example.com".to_string()),
                ActionTarget::Id("email".to_string())
            ))
            .add_action(Action::new(
                ActionType::Type("password123".to_string()),
                ActionTarget::Id("password".to_string())
            ))
            .add_action(Action::new(
                ActionType::Click,
                ActionTarget::Selector("button[type='submit']".to_string())
            ))
            .add_parallel_group(vec![0, 1])
            .continue_on_failure();

        assert_eq!(chain.actions.len(), 3);
        assert_eq!(chain.parallel_groups.len(), 1);
        assert!(!chain.stop_on_failure);
    }

    #[tokio::test]
    async fn test_intelligent_action_engine_creation() {
        let engine = IntelligentActionEngine::new();
        assert!(engine.session_cache.read().await.is_empty());
    }
}