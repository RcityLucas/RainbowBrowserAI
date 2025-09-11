use super::traits::{Tool, ToolCategory};
use crate::browser::{Browser, core::BrowserOps};
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use std::sync::Arc;
use std::time::Duration;
use tracing::info;

// ============================================================================
// Wait For Element Tool
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct WaitForElementInput {
    pub selector: String,
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,
    #[serde(default)]
    pub visible: bool,
}

fn default_timeout() -> u64 {
    10000
}

#[derive(Debug, Serialize)]
pub struct WaitForElementOutput {
    pub success: bool,
    pub element_found: bool,
    pub wait_time_ms: u64,
}

pub struct WaitForElementTool {
    browser: Arc<Browser>,
}

impl WaitForElementTool {
    pub fn new(browser: Arc<Browser>) -> Self {
        Self { browser }
    }
}

#[async_trait]
impl Tool for WaitForElementTool {
    type Input = WaitForElementInput;
    type Output = WaitForElementOutput;
    
    fn name(&self) -> &str {
        "wait_for_element"
    }
    
    fn description(&self) -> &str {
        "Wait for an element to appear on the page"
    }
    
    fn category(&self) -> ToolCategory {
        ToolCategory::Synchronization
    }
    
    async fn execute(&self, input: Self::Input) -> Result<Self::Output> {
        info!("Waiting for element: {}", input.selector);
        let start = std::time::Instant::now();
        
        let timeout = Duration::from_millis(input.timeout_ms);
        
        match self.browser.wait_for_selector(&input.selector, timeout).await {
            Ok(_) => {
                let wait_time = start.elapsed().as_millis() as u64;
                Ok(WaitForElementOutput {
                    success: true,
                    element_found: true,
                    wait_time_ms: wait_time,
                })
            }
            Err(_) => {
                Ok(WaitForElementOutput {
                    success: false,
                    element_found: false,
                    wait_time_ms: input.timeout_ms,
                })
            }
        }
    }
    
    async fn validate_input(&self, input: &Self::Input) -> Result<()> {
        if input.selector.is_empty() {
            return Err(anyhow!("Selector cannot be empty"));
        }
        Ok(())
    }
}

// ============================================================================
// Wait For Condition Tool
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct WaitForConditionInput {
    pub condition: String,  // JavaScript expression
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,
    #[serde(default = "default_check_interval")]
    pub check_interval_ms: u64,
}

fn default_check_interval() -> u64 {
    100
}

#[derive(Debug, Serialize)]
pub struct WaitForConditionOutput {
    pub success: bool,
    pub condition_met: bool,
    pub wait_time_ms: u64,
    pub result: Option<serde_json::Value>,
}

pub struct WaitForConditionTool {
    browser: Arc<Browser>,
}

impl WaitForConditionTool {
    pub fn new(browser: Arc<Browser>) -> Self {
        Self { browser }
    }
}

#[async_trait]
impl Tool for WaitForConditionTool {
    type Input = WaitForConditionInput;
    type Output = WaitForConditionOutput;
    
    fn name(&self) -> &str {
        "wait_for_condition"
    }
    
    fn description(&self) -> &str {
        "Wait for a JavaScript condition to become true"
    }
    
    fn category(&self) -> ToolCategory {
        ToolCategory::Synchronization
    }
    
    async fn execute(&self, input: Self::Input) -> Result<Self::Output> {
        info!("Waiting for condition: {}", input.condition);
        let start = std::time::Instant::now();
        let timeout = Duration::from_millis(input.timeout_ms);
        let check_interval = Duration::from_millis(input.check_interval_ms);
        
        loop {
            if start.elapsed() > timeout {
                return Ok(WaitForConditionOutput {
                    success: false,
                    condition_met: false,
                    wait_time_ms: input.timeout_ms,
                    result: None,
                });
            }
            
            let script = format!("({})", input.condition);
            match self.browser.execute_script(&script).await {
                Ok(result) => {
                    if result.as_bool().unwrap_or(false) {
                        let wait_time = start.elapsed().as_millis() as u64;
                        return Ok(WaitForConditionOutput {
                            success: true,
                            condition_met: true,
                            wait_time_ms: wait_time,
                            result: Some(result),
                        });
                    }
                }
                Err(_) => {
                    // Script error, continue waiting
                }
            }
            
            tokio::time::sleep(check_interval).await;
        }
    }
    
    async fn validate_input(&self, input: &Self::Input) -> Result<()> {
        if input.condition.is_empty() {
            return Err(anyhow!("Condition cannot be empty"));
        }
        Ok(())
    }
}

// ============================================================================
// Wait For Navigation Tool
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct WaitForNavigationInput {
    #[serde(default = "default_navigation_timeout")]
    pub timeout_ms: u64,
    #[serde(default)]
    pub wait_for_load: bool,
    #[serde(default)]
    pub wait_for_network_idle: bool,
    #[serde(default)]
    pub expected_url: Option<String>,
}

fn default_navigation_timeout() -> u64 {
    30000
}

#[derive(Debug, Serialize)]
pub struct WaitForNavigationOutput {
    pub success: bool,
    pub navigation_completed: bool,
    pub final_url: String,
    pub wait_time_ms: u64,
    pub load_time_ms: Option<u64>,
}

pub struct WaitForNavigationTool {
    browser: Arc<Browser>,
}

impl WaitForNavigationTool {
    pub fn new(browser: Arc<Browser>) -> Self {
        Self { browser }
    }
}

#[async_trait]
impl Tool for WaitForNavigationTool {
    type Input = WaitForNavigationInput;
    type Output = WaitForNavigationOutput;
    
    fn name(&self) -> &str {
        "wait_for_navigation"
    }
    
    fn description(&self) -> &str {
        "Wait for page navigation to complete with various load states"
    }
    
    fn category(&self) -> ToolCategory {
        ToolCategory::Synchronization
    }
    
    async fn execute(&self, input: Self::Input) -> Result<Self::Output> {
        info!("Waiting for navigation to complete");
        let start = std::time::Instant::now();
        let timeout = Duration::from_millis(input.timeout_ms);
        
        // Wait for basic navigation completion
        let navigation_result = tokio::time::timeout(
            timeout,
            self.browser.wait_for_load()
        ).await;
        
        match navigation_result {
            Ok(Ok(())) => {
                let mut wait_time = start.elapsed().as_millis() as u64;
                let mut load_time_ms = None;
                
                // Additional wait for network idle if requested
                if input.wait_for_network_idle {
                    let idle_start = std::time::Instant::now();
                    if let Err(_) = tokio::time::timeout(
                        Duration::from_millis(input.timeout_ms - wait_time),
                        self.browser.wait_for_network_idle(Duration::from_millis(1000))
                    ).await {
                        // Network idle timeout, but navigation still succeeded
                    }
                    let idle_time = idle_start.elapsed().as_millis() as u64;
                    load_time_ms = Some(idle_time);
                    wait_time = start.elapsed().as_millis() as u64;
                }
                
                let final_url = self.browser.current_url().await.unwrap_or_default();
                
                // Check expected URL if provided
                let url_matches = if let Some(expected) = &input.expected_url {
                    final_url.contains(expected) || final_url == *expected
                } else {
                    true
                };
                
                Ok(WaitForNavigationOutput {
                    success: url_matches,
                    navigation_completed: true,
                    final_url,
                    wait_time_ms: wait_time,
                    load_time_ms,
                })
            }
            Ok(Err(e)) => {
                info!("Navigation failed: {}", e);
                Ok(WaitForNavigationOutput {
                    success: false,
                    navigation_completed: false,
                    final_url: self.browser.current_url().await.unwrap_or_default(),
                    wait_time_ms: start.elapsed().as_millis() as u64,
                    load_time_ms: None,
                })
            }
            Err(_) => {
                info!("Navigation timeout after {}ms", input.timeout_ms);
                Ok(WaitForNavigationOutput {
                    success: false,
                    navigation_completed: false,
                    final_url: self.browser.current_url().await.unwrap_or_default(),
                    wait_time_ms: input.timeout_ms,
                    load_time_ms: None,
                })
            }
        }
    }
    
    async fn validate_input(&self, input: &Self::Input) -> Result<()> {
        if input.timeout_ms == 0 {
            return Err(anyhow!("Timeout must be greater than 0"));
        }
        if input.timeout_ms > 300000 { // 5 minutes max
            return Err(anyhow!("Timeout cannot exceed 300 seconds"));
        }
        Ok(())
    }
}

// ============================================================================
// Wait For Network Idle Tool
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct WaitForNetworkIdleInput {
    #[serde(default = "default_idle_time_ms")]
    pub idle_time_ms: u64,
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,
}

fn default_idle_time_ms() -> u64 {
    500 // 500ms network idle threshold
}

#[derive(Debug, Serialize)]
pub struct WaitForNetworkIdleOutput {
    pub success: bool,
    pub network_idle_achieved: bool,
    pub wait_time_ms: u64,
    pub final_active_requests: usize,
}

pub struct WaitForNetworkIdleTool {
    browser: Arc<Browser>,
}

impl WaitForNetworkIdleTool {
    pub fn new(browser: Arc<Browser>) -> Self {
        Self { browser }
    }
}

#[async_trait]
impl Tool for WaitForNetworkIdleTool {
    type Input = WaitForNetworkIdleInput;
    type Output = WaitForNetworkIdleOutput;
    
    fn name(&self) -> &str {
        "wait_for_network_idle"
    }
    
    fn description(&self) -> &str {
        "Wait for network activity to be idle using enhanced CDP Network domain tracking"
    }
    
    fn category(&self) -> ToolCategory {
        ToolCategory::Synchronization
    }
    
    async fn execute(&self, input: Self::Input) -> Result<Self::Output> {
        info!("Waiting for network idle: {}ms threshold, {}ms timeout (CDP-backed)", input.idle_time_ms, input.timeout_ms);
        let start = std::time::Instant::now();
        
        let idle_duration = Duration::from_millis(input.idle_time_ms);
        let timeout = Duration::from_millis(input.timeout_ms);
        
        // Use the enhanced CDP-backed network idle detection from browser
        let result = tokio::time::timeout(timeout, self.browser.wait_for_network_idle(idle_duration)).await;
        
        match result {
            Ok(Ok(())) => {
                info!("Network idle achieved after {}ms (CDP-tracked)", start.elapsed().as_millis());
                Ok(WaitForNetworkIdleOutput {
                    success: true,
                    network_idle_achieved: true,
                    wait_time_ms: start.elapsed().as_millis() as u64,
                    final_active_requests: 0, // Idle means 0 active requests
                })
            }
            Ok(Err(e)) => {
                info!("Network idle failed: {}", e);
                Ok(WaitForNetworkIdleOutput {
                    success: false,
                    network_idle_achieved: false,
                    wait_time_ms: start.elapsed().as_millis() as u64,
                    final_active_requests: 0, // Unknown, assume 0
                })
            }
            Err(_) => {
                info!("Network idle timeout after {}ms (CDP-tracked)", input.timeout_ms);
                Ok(WaitForNetworkIdleOutput {
                    success: false,
                    network_idle_achieved: false,
                    wait_time_ms: input.timeout_ms,
                    final_active_requests: 0, // Unknown due to timeout
                })
            }
        }
    }
    
    async fn validate_input(&self, input: &Self::Input) -> Result<()> {
        if input.idle_time_ms == 0 {
            return Err(anyhow!("Idle time must be greater than 0"));
        }
        if input.idle_time_ms > 10000 {
            return Err(anyhow!("Idle time cannot exceed 10 seconds"));
        }
        if input.timeout_ms == 0 {
            return Err(anyhow!("Timeout must be greater than 0"));
        }
        if input.timeout_ms > 300000 { // 5 minutes max
            return Err(anyhow!("Timeout cannot exceed 300 seconds"));
        }
        Ok(())
    }
}