// Action Executor
// Part of the Intelligent Action Engine

use crate::error::{Result, RainbowError};
use crate::action::{Action, ActionType, ElementInfo, BoundingBox};
use chromiumoxide::{Page, Element};
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::collections::HashMap;

/// Main action executor that performs browser actions
#[derive(Debug)]
pub struct ActionExecutor {
    execution_strategies: HashMap<String, Box<dyn ExecutionStrategy + Send + Sync>>,
    performance_monitor: Arc<tokio::sync::RwLock<PerformanceMonitor>>,
}

impl ActionExecutor {
    pub fn new() -> Self {
        let mut strategies: HashMap<String, Box<dyn ExecutionStrategy + Send + Sync>> = HashMap::new();
        
        // Register execution strategies for different action types
        strategies.insert("click".to_string(), Box::new(ClickExecutionStrategy));
        strategies.insert("type".to_string(), Box::new(TypeExecutionStrategy));
        strategies.insert("navigate".to_string(), Box::new(NavigationExecutionStrategy));
        strategies.insert("wait".to_string(), Box::new(WaitExecutionStrategy));
        strategies.insert("screenshot".to_string(), Box::new(ScreenshotExecutionStrategy));
        strategies.insert("scroll".to_string(), Box::new(ScrollExecutionStrategy));
        strategies.insert("hover".to_string(), Box::new(HoverExecutionStrategy));
        strategies.insert("submit".to_string(), Box::new(SubmitExecutionStrategy));

        Self {
            execution_strategies: strategies,
            performance_monitor: Arc::new(tokio::sync::RwLock::new(PerformanceMonitor::new())),
        }
    }

    /// Execute an action and return detailed execution result
    pub async fn execute(
        &self,
        page: Arc<Page>,
        action: &Action,
        element: Element,
    ) -> Result<ExecutionResult> {
        let start_time = Instant::now();
        let strategy_key = self.get_strategy_key(&action.action_type);
        
        // Get appropriate execution strategy
        let strategy = self.execution_strategies
            .get(&strategy_key)
            .ok_or_else(|| RainbowError::ExecutionError(
                format!("No execution strategy for action type: {:?}", action.action_type)
            ))?;

        // Capture element info before execution
        let element_info_before = self.capture_element_info(&element).await?;

        // Execute the action with timeout
        let execution_result = tokio::time::timeout(
            action.timeout,
            strategy.execute(page.clone(), action, element.clone())
        ).await;

        let execution_time = start_time.elapsed();

        let result = match execution_result {
            Ok(Ok(strategy_result)) => {
                // Capture element info after execution
                let element_info_after = self.capture_element_info(&element).await.ok();
                
                ExecutionResult {
                    success: true,
                    execution_time,
                    error_message: None,
                    element_info: Some(element_info_before),
                    element_info_after,
                    screenshot_path: strategy_result.screenshot_path,
                    performance_metrics: self.calculate_performance_metrics(&action.action_type, execution_time).await,
                    metadata: strategy_result.metadata,
                }
            }
            Ok(Err(e)) => {
                ExecutionResult {
                    success: false,
                    execution_time,
                    error_message: Some(e.to_string()),
                    element_info: Some(element_info_before),
                    element_info_after: None,
                    screenshot_path: None,
                    performance_metrics: HashMap::new(),
                    metadata: serde_json::Value::Null,
                }
            }
            Err(_) => {
                ExecutionResult {
                    success: false,
                    execution_time,
                    error_message: Some("Action timed out".to_string()),
                    element_info: Some(element_info_before),
                    element_info_after: None,
                    screenshot_path: None,
                    performance_metrics: HashMap::new(),
                    metadata: serde_json::Value::Null,
                }
            }
        };

        // Update performance monitor
        self.update_performance_monitor(action, &result).await;

        Ok(result)
    }

    /// Get execution statistics
    pub async fn get_performance_stats(&self) -> PerformanceStats {
        self.performance_monitor.read().await.get_stats()
    }

    fn get_strategy_key(&self, action_type: &ActionType) -> String {
        match action_type {
            ActionType::Click | ActionType::DoubleClick | ActionType::RightClick => "click".to_string(),
            ActionType::Type(_) => "type".to_string(),
            ActionType::Navigate(_) | ActionType::GoBack | ActionType::GoForward | ActionType::Refresh => "navigate".to_string(),
            ActionType::Wait(_) => "wait".to_string(),
            ActionType::Screenshot => "screenshot".to_string(),
            ActionType::ScrollTo => "scroll".to_string(),
            ActionType::Hover => "hover".to_string(),
            ActionType::Submit => "submit".to_string(),
            ActionType::Clear => "type".to_string(), // Clear uses type strategy
            ActionType::Focus => "click".to_string(), // Focus uses click strategy
            ActionType::Select(_) => "click".to_string(), // Select uses click strategy
            ActionType::Upload(_) => "type".to_string(), // Upload uses type strategy
            ActionType::KeyPress(_) => "type".to_string(), // KeyPress uses type strategy
        }
    }

    async fn capture_element_info(&self, element: &Element) -> Result<ElementInfo> {
        let js_code = r#"
            const elem = arguments[0];
            const rect = elem.getBoundingClientRect();
            const style = window.getComputedStyle(elem);
            
            // Collect all attributes
            const attributes = {};
            for (let attr of elem.attributes) {
                attributes[attr.name] = attr.value;
            }
            
            return {
                tagName: elem.tagName.toLowerCase(),
                attributes: attributes,
                textContent: elem.textContent || null,
                boundingBox: {
                    x: rect.x,
                    y: rect.y,
                    width: rect.width,
                    height: rect.height
                },
                isVisible: rect.width > 0 && rect.height > 0 && style.visibility !== 'hidden',
                isEnabled: !elem.disabled && !elem.hasAttribute('disabled')
            };
        "#;

        let result = element.call_js_fn(js_code, vec![]).await?;
        let info = result.as_object().ok_or_else(|| {
            RainbowError::ExecutionError("Failed to capture element info".to_string())
        })?;

        Ok(ElementInfo {
            tag_name: info.get("tagName")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            attributes: info.get("attributes")
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or_default(),
            text_content: info.get("textContent")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            bounding_box: info.get("boundingBox")
                .and_then(|bbox| {
                    let bbox_obj = bbox.as_object()?;
                    Some(BoundingBox {
                        x: bbox_obj.get("x")?.as_f64()?,
                        y: bbox_obj.get("y")?.as_f64()?,
                        width: bbox_obj.get("width")?.as_f64()?,
                        height: bbox_obj.get("height")?.as_f64()?,
                    })
                }),
            is_visible: info.get("isVisible")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            is_enabled: info.get("isEnabled")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
        })
    }

    async fn calculate_performance_metrics(
        &self,
        action_type: &ActionType,
        execution_time: Duration,
    ) -> HashMap<String, f64> {
        let mut metrics = HashMap::new();
        
        metrics.insert("execution_time_ms".to_string(), execution_time.as_millis() as f64);
        metrics.insert("action_complexity_score".to_string(), self.calculate_complexity_score(action_type));
        
        // Get historical average for comparison
        let monitor = self.performance_monitor.read().await;
        if let Some(avg_time) = monitor.get_average_time(action_type) {
            let performance_ratio = execution_time.as_millis() as f64 / avg_time.as_millis() as f64;
            metrics.insert("performance_ratio".to_string(), performance_ratio);
        }

        metrics
    }

    fn calculate_complexity_score(&self, action_type: &ActionType) -> f64 {
        match action_type {
            ActionType::Wait(_) => 1.0,
            ActionType::Click | ActionType::Focus => 2.0,
            ActionType::Type(_) | ActionType::Clear => 3.0,
            ActionType::DoubleClick | ActionType::RightClick => 3.5,
            ActionType::Hover | ActionType::ScrollTo => 4.0,
            ActionType::Select(_) | ActionType::KeyPress(_) => 4.5,
            ActionType::Upload(_) => 5.0,
            ActionType::Submit => 6.0,
            ActionType::Screenshot => 7.0,
            ActionType::Navigate(_) | ActionType::GoBack | ActionType::GoForward | ActionType::Refresh => 8.0,
        }
    }

    async fn update_performance_monitor(&self, action: &Action, result: &ExecutionResult) {
        let mut monitor = self.performance_monitor.write().await;
        monitor.record_execution(action, result);
    }
}

impl Default for ActionExecutor {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of action execution with detailed information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub success: bool,
    pub execution_time: Duration,
    pub error_message: Option<String>,
    pub element_info: Option<ElementInfo>,
    pub element_info_after: Option<ElementInfo>,
    pub screenshot_path: Option<String>,
    pub performance_metrics: HashMap<String, f64>,
    pub metadata: serde_json::Value,
}

/// Strategy result from individual execution strategies
#[derive(Debug, Clone)]
pub struct StrategyResult {
    pub screenshot_path: Option<String>,
    pub metadata: serde_json::Value,
}

/// Trait for different action execution strategies
#[async_trait::async_trait]
trait ExecutionStrategy: std::fmt::Debug {
    fn name(&self) -> &'static str;
    
    async fn execute(
        &self,
        page: Arc<Page>,
        action: &Action,
        element: Element,
    ) -> Result<StrategyResult>;
}

/// Click execution strategy
#[derive(Debug)]
struct ClickExecutionStrategy;

#[async_trait::async_trait]
impl ExecutionStrategy for ClickExecutionStrategy {
    fn name(&self) -> &'static str { "ClickExecutionStrategy" }
    
    async fn execute(
        &self,
        page: Arc<Page>,
        action: &Action,
        element: Element,
    ) -> Result<StrategyResult> {
        match &action.action_type {
            ActionType::Click => {
                element.click().await
                    .map_err(|e| RainbowError::ExecutionError(format!("Click failed: {}", e)))?;
            }
            ActionType::DoubleClick => {
                // Double click implementation
                element.click().await?;
                tokio::time::sleep(Duration::from_millis(50)).await;
                element.click().await
                    .map_err(|e| RainbowError::ExecutionError(format!("Double click failed: {}", e)))?;
            }
            ActionType::RightClick => {
                // Right click through JavaScript
                let js_code = r#"
                    const event = new MouseEvent('contextmenu', {
                        bubbles: true,
                        cancelable: true,
                        view: window
                    });
                    arguments[0].dispatchEvent(event);
                "#;
                element.call_js_fn(js_code, vec![]).await
                    .map_err(|e| RainbowError::ExecutionError(format!("Right click failed: {}", e)))?;
            }
            ActionType::Focus => {
                element.focus().await
                    .map_err(|e| RainbowError::ExecutionError(format!("Focus failed: {}", e)))?;
            }
            _ => return Err(RainbowError::ExecutionError("Invalid action type for click strategy".to_string())),
        }

        Ok(StrategyResult {
            screenshot_path: None,
            metadata: serde_json::json!({
                "strategy": "click",
                "action_type": action.action_type
            }),
        })
    }
}

/// Type/input execution strategy
#[derive(Debug)]
struct TypeExecutionStrategy;

#[async_trait::async_trait]
impl ExecutionStrategy for TypeExecutionStrategy {
    fn name(&self) -> &'static str { "TypeExecutionStrategy" }
    
    async fn execute(
        &self,
        page: Arc<Page>,
        action: &Action,
        element: Element,
    ) -> Result<StrategyResult> {
        match &action.action_type {
            ActionType::Type(text) => {
                // Clear first, then type
                element.click().await?; // Focus the element
                element.clear().await?;
                element.type_str(text).await
                    .map_err(|e| RainbowError::ExecutionError(format!("Type failed: {}", e)))?;
            }
            ActionType::Clear => {
                element.click().await?; // Focus the element
                element.clear().await
                    .map_err(|e| RainbowError::ExecutionError(format!("Clear failed: {}", e)))?;
            }
            ActionType::KeyPress(key) => {
                element.focus().await?;
                element.press_key(key).await
                    .map_err(|e| RainbowError::ExecutionError(format!("Key press failed: {}", e)))?;
            }
            ActionType::Upload(file_path) => {
                // File upload implementation
                let js_code = format!(r#"
                    arguments[0].value = '{}';
                    arguments[0].dispatchEvent(new Event('change', {{ bubbles: true }}));
                "#, file_path);
                element.call_js_fn(&js_code, vec![]).await
                    .map_err(|e| RainbowError::ExecutionError(format!("Upload failed: {}", e)))?;
            }
            _ => return Err(RainbowError::ExecutionError("Invalid action type for type strategy".to_string())),
        }

        Ok(StrategyResult {
            screenshot_path: None,
            metadata: serde_json::json!({
                "strategy": "type",
                "action_type": action.action_type
            }),
        })
    }
}

/// Navigation execution strategy
#[derive(Debug)]
struct NavigationExecutionStrategy;

#[async_trait::async_trait]
impl ExecutionStrategy for NavigationExecutionStrategy {
    fn name(&self) -> &'static str { "NavigationExecutionStrategy" }
    
    async fn execute(
        &self,
        page: Arc<Page>,
        action: &Action,
        _element: Element,
    ) -> Result<StrategyResult> {
        match &action.action_type {
            ActionType::Navigate(url) => {
                page.goto(url).await
                    .map_err(|e| RainbowError::ExecutionError(format!("Navigation failed: {}", e)))?;
            }
            ActionType::GoBack => {
                page.go_back().await
                    .map_err(|e| RainbowError::ExecutionError(format!("Go back failed: {}", e)))?;
            }
            ActionType::GoForward => {
                page.go_forward().await
                    .map_err(|e| RainbowError::ExecutionError(format!("Go forward failed: {}", e)))?;
            }
            ActionType::Refresh => {
                page.reload().await
                    .map_err(|e| RainbowError::ExecutionError(format!("Refresh failed: {}", e)))?;
            }
            _ => return Err(RainbowError::ExecutionError("Invalid action type for navigation strategy".to_string())),
        }

        Ok(StrategyResult {
            screenshot_path: None,
            metadata: serde_json::json!({
                "strategy": "navigation",
                "action_type": action.action_type,
                "url": page.url().await.unwrap_or_default()
            }),
        })
    }
}

/// Wait execution strategy
#[derive(Debug)]
struct WaitExecutionStrategy;

#[async_trait::async_trait]
impl ExecutionStrategy for WaitExecutionStrategy {
    fn name(&self) -> &'static str { "WaitExecutionStrategy" }
    
    async fn execute(
        &self,
        page: Arc<Page>,
        action: &Action,
        _element: Element,
    ) -> Result<StrategyResult> {
        if let ActionType::Wait(duration) = &action.action_type {
            tokio::time::sleep(*duration).await;
        } else {
            return Err(RainbowError::ExecutionError("Invalid action type for wait strategy".to_string()));
        }

        Ok(StrategyResult {
            screenshot_path: None,
            metadata: serde_json::json!({
                "strategy": "wait",
                "duration_ms": match &action.action_type {
                    ActionType::Wait(d) => d.as_millis(),
                    _ => 0,
                }
            }),
        })
    }
}

/// Screenshot execution strategy
#[derive(Debug)]
struct ScreenshotExecutionStrategy;

#[async_trait::async_trait]
impl ExecutionStrategy for ScreenshotExecutionStrategy {
    fn name(&self) -> &'static str { "ScreenshotExecutionStrategy" }
    
    async fn execute(
        &self,
        page: Arc<Page>,
        action: &Action,
        _element: Element,
    ) -> Result<StrategyResult> {
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S%.3f");
        let screenshot_path = format!("screenshots/action_{}_{}.png", action.id, timestamp);
        
        // Ensure screenshots directory exists
        std::fs::create_dir_all("screenshots").ok();
        
        page.screenshot(&screenshot_path).await
            .map_err(|e| RainbowError::ExecutionError(format!("Screenshot failed: {}", e)))?;

        Ok(StrategyResult {
            screenshot_path: Some(screenshot_path.clone()),
            metadata: serde_json::json!({
                "strategy": "screenshot",
                "path": screenshot_path
            }),
        })
    }
}

/// Scroll execution strategy
#[derive(Debug)]
struct ScrollExecutionStrategy;

#[async_trait::async_trait]
impl ExecutionStrategy for ScrollExecutionStrategy {
    fn name(&self) -> &'static str { "ScrollExecutionStrategy" }
    
    async fn execute(
        &self,
        page: Arc<Page>,
        action: &Action,
        element: Element,
    ) -> Result<StrategyResult> {
        // Scroll element into view
        let js_code = r#"
            arguments[0].scrollIntoView({ 
                behavior: 'smooth', 
                block: 'center', 
                inline: 'center' 
            });
        "#;
        
        element.call_js_fn(js_code, vec![]).await
            .map_err(|e| RainbowError::ExecutionError(format!("Scroll failed: {}", e)))?;

        // Wait for scroll animation to complete
        tokio::time::sleep(Duration::from_millis(500)).await;

        Ok(StrategyResult {
            screenshot_path: None,
            metadata: serde_json::json!({
                "strategy": "scroll"
            }),
        })
    }
}

/// Hover execution strategy
#[derive(Debug)]
struct HoverExecutionStrategy;

#[async_trait::async_trait]
impl ExecutionStrategy for HoverExecutionStrategy {
    fn name(&self) -> &'static str { "HoverExecutionStrategy" }
    
    async fn execute(
        &self,
        page: Arc<Page>,
        action: &Action,
        element: Element,
    ) -> Result<StrategyResult> {
        element.hover().await
            .map_err(|e| RainbowError::ExecutionError(format!("Hover failed: {}", e)))?;

        Ok(StrategyResult {
            screenshot_path: None,
            metadata: serde_json::json!({
                "strategy": "hover"
            }),
        })
    }
}

/// Submit execution strategy
#[derive(Debug)]
struct SubmitExecutionStrategy;

#[async_trait::async_trait]
impl ExecutionStrategy for SubmitExecutionStrategy {
    fn name(&self) -> &'static str { "SubmitExecutionStrategy" }
    
    async fn execute(
        &self,
        page: Arc<Page>,
        action: &Action,
        element: Element,
    ) -> Result<StrategyResult> {
        // Try form submission through JavaScript
        let js_code = r#"
            const form = arguments[0].closest('form');
            if (form) {
                form.submit();
            } else {
                // If not in a form, try clicking the element
                arguments[0].click();
            }
        "#;
        
        element.call_js_fn(js_code, vec![]).await
            .map_err(|e| RainbowError::ExecutionError(format!("Submit failed: {}", e)))?;

        Ok(StrategyResult {
            screenshot_path: None,
            metadata: serde_json::json!({
                "strategy": "submit"
            }),
        })
    }
}

/// Performance monitoring for action execution
#[derive(Debug)]
struct PerformanceMonitor {
    execution_times: HashMap<String, Vec<Duration>>,
    success_rates: HashMap<String, (u32, u32)>, // (successes, total)
    last_updated: Instant,
}

impl PerformanceMonitor {
    fn new() -> Self {
        Self {
            execution_times: HashMap::new(),
            success_rates: HashMap::new(),
            last_updated: Instant::now(),
        }
    }

    fn record_execution(&mut self, action: &Action, result: &ExecutionResult) {
        let action_key = format!("{:?}", action.action_type);
        
        // Record execution time
        self.execution_times
            .entry(action_key.clone())
            .or_insert_with(Vec::new)
            .push(result.execution_time);

        // Record success/failure
        let (successes, total) = self.success_rates
            .entry(action_key)
            .or_insert((0, 0));
            
        *total += 1;
        if result.success {
            *successes += 1;
        }

        self.last_updated = Instant::now();
    }

    fn get_average_time(&self, action_type: &ActionType) -> Option<Duration> {
        let action_key = format!("{:?}", action_type);
        let times = self.execution_times.get(&action_key)?;
        
        if times.is_empty() {
            return None;
        }

        let total_ms: u128 = times.iter().map(|d| d.as_millis()).sum();
        Some(Duration::from_millis((total_ms / times.len() as u128) as u64))
    }

    fn get_stats(&self) -> PerformanceStats {
        let mut stats = PerformanceStats {
            total_actions: 0,
            average_execution_times: HashMap::new(),
            success_rates: HashMap::new(),
            last_updated: self.last_updated,
        };

        // Calculate averages and success rates
        for (action_type, times) in &self.execution_times {
            if let Some((successes, total)) = self.success_rates.get(action_type) {
                stats.total_actions += total;
                
                let avg_time = if !times.is_empty() {
                    let total_ms: u128 = times.iter().map(|d| d.as_millis()).sum();
                    Duration::from_millis((total_ms / times.len() as u128) as u64)
                } else {
                    Duration::default()
                };
                
                let success_rate = if *total > 0 {
                    *successes as f64 / *total as f64
                } else {
                    0.0
                };

                stats.average_execution_times.insert(action_type.clone(), avg_time);
                stats.success_rates.insert(action_type.clone(), success_rate);
            }
        }

        stats
    }
}

/// Performance statistics for action execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceStats {
    pub total_actions: u32,
    pub average_execution_times: HashMap<String, Duration>,
    pub success_rates: HashMap<String, f64>,
    pub last_updated: Instant,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::ActionTarget;

    #[test]
    fn test_action_executor_creation() {
        let executor = ActionExecutor::new();
        assert_eq!(executor.execution_strategies.len(), 8);
    }

    #[test]
    fn test_strategy_key_mapping() {
        let executor = ActionExecutor::new();
        
        assert_eq!(executor.get_strategy_key(&ActionType::Click), "click");
        assert_eq!(executor.get_strategy_key(&ActionType::Type("test".to_string())), "type");
        assert_eq!(executor.get_strategy_key(&ActionType::Navigate("url".to_string())), "navigate");
        assert_eq!(executor.get_strategy_key(&ActionType::Wait(Duration::from_secs(1))), "wait");
        assert_eq!(executor.get_strategy_key(&ActionType::Screenshot), "screenshot");
    }

    #[test]
    fn test_complexity_scoring() {
        let executor = ActionExecutor::new();
        
        // Wait should have lowest complexity
        assert_eq!(executor.calculate_complexity_score(&ActionType::Wait(Duration::from_secs(1))), 1.0);
        
        // Navigation should have high complexity
        assert_eq!(executor.calculate_complexity_score(&ActionType::Navigate("url".to_string())), 8.0);
        
        // Click should have moderate complexity
        assert_eq!(executor.calculate_complexity_score(&ActionType::Click), 2.0);
    }

    #[test]
    fn test_performance_monitor() {
        let mut monitor = PerformanceMonitor::new();
        
        let action = Action::new(ActionType::Click, ActionTarget::Selector("#test".to_string()));
        let result = ExecutionResult {
            success: true,
            execution_time: Duration::from_millis(100),
            error_message: None,
            element_info: None,
            element_info_after: None,
            screenshot_path: None,
            performance_metrics: HashMap::new(),
            metadata: serde_json::Value::Null,
        };

        monitor.record_execution(&action, &result);
        
        let avg_time = monitor.get_average_time(&ActionType::Click);
        assert_eq!(avg_time, Some(Duration::from_millis(100)));

        let stats = monitor.get_stats();
        assert_eq!(stats.total_actions, 1);
        assert!(stats.success_rates.contains_key("Click"));
        assert_eq!(stats.success_rates["Click"], 1.0);
    }

    #[test]
    fn test_execution_result_creation() {
        let result = ExecutionResult {
            success: true,
            execution_time: Duration::from_millis(150),
            error_message: None,
            element_info: None,
            element_info_after: None,
            screenshot_path: Some("test.png".to_string()),
            performance_metrics: HashMap::new(),
            metadata: serde_json::json!({"test": "data"}),
        };

        assert!(result.success);
        assert_eq!(result.execution_time, Duration::from_millis(150));
        assert_eq!(result.screenshot_path, Some("test.png".to_string()));
    }
}