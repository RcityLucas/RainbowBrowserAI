use super::traits::{Tool, ToolCategory};
use crate::browser::Browser;
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