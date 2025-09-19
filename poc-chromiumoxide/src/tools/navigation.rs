use super::traits::{Tool, ToolCategory};
use crate::browser::Browser;
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, info};

// ============================================================================
// Navigate Tool
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct NavigateInput {
    pub url: String,
    #[serde(default)]
    pub wait_until: Option<String>,
    #[serde(default)]
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct NavigateOutput {
    pub success: bool,
    pub final_url: String,
    pub load_time_ms: u64,
}

pub struct NavigateTool {
    browser: Arc<Browser>,
}

impl NavigateTool {
    pub fn new(browser: Arc<Browser>) -> Self {
        Self { browser }
    }
}

#[async_trait]
impl Tool for NavigateTool {
    type Input = NavigateInput;
    type Output = NavigateOutput;

    fn name(&self) -> &str {
        "navigate_to_url"
    }

    fn description(&self) -> &str {
        "Navigate to a specified URL"
    }

    fn category(&self) -> ToolCategory {
        ToolCategory::Navigation
    }

    async fn execute(&self, input: Self::Input) -> Result<Self::Output> {
        let start = std::time::Instant::now();
        info!("Navigating to: {}", input.url);

        self.browser.navigate_to(&input.url).await?;

        let final_url = self.browser.current_url().await?;
        let load_time = start.elapsed().as_millis() as u64;

        Ok(NavigateOutput {
            success: true,
            final_url,
            load_time_ms: load_time,
        })
    }

    async fn validate_input(&self, input: &Self::Input) -> Result<()> {
        if input.url.is_empty() {
            return Err(anyhow::anyhow!("URL cannot be empty"));
        }

        // Basic URL validation
        if !input.url.starts_with("http://") && !input.url.starts_with("https://") {
            return Err(anyhow::anyhow!("URL must start with http:// or https://"));
        }

        Ok(())
    }
}

// ============================================================================
// Scroll Tool
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct ScrollInput {
    #[serde(default)]
    pub x: Option<i32>,
    #[serde(default)]
    pub y: Option<i32>,
    #[serde(default)]
    pub element: Option<String>,
    #[serde(default)]
    pub smooth: bool,
}

#[derive(Debug, Serialize)]
pub struct ScrollOutput {
    pub success: bool,
    pub final_position: ScrollPosition,
}

#[derive(Debug, Serialize)]
pub struct ScrollPosition {
    pub x: i32,
    pub y: i32,
}

pub struct ScrollTool {
    browser: Arc<Browser>,
}

impl ScrollTool {
    pub fn new(browser: Arc<Browser>) -> Self {
        Self { browser }
    }
}

#[async_trait]
impl Tool for ScrollTool {
    type Input = ScrollInput;
    type Output = ScrollOutput;

    fn name(&self) -> &str {
        "scroll_page"
    }

    fn description(&self) -> &str {
        "Scroll to a specific position or element on the page"
    }

    fn category(&self) -> ToolCategory {
        ToolCategory::Navigation
    }

    async fn execute(&self, input: Self::Input) -> Result<Self::Output> {
        // If element is specified, scroll to element
        if let Some(selector) = &input.element {
            debug!("Scrolling to element: {}", selector);
            let script = format!(
                "document.querySelector('{}').scrollIntoView({{ behavior: '{}' }})",
                selector,
                if input.smooth { "smooth" } else { "auto" }
            );
            self.browser.execute_script(&script).await?;
        } else {
            // Scroll to position
            let x = input.x.unwrap_or(0);
            let y = input.y.unwrap_or(0);
            debug!("Scrolling to position: ({}, {})", x, y);
            self.browser.scroll_to(x, y).await?;
        }

        // Get current scroll position
        let position_script = "JSON.stringify({ x: window.scrollX, y: window.scrollY })";
        let position_value = self.browser.execute_script(position_script).await?;

        let x = position_value["x"].as_i64().unwrap_or(0) as i32;
        let y = position_value["y"].as_i64().unwrap_or(0) as i32;

        Ok(ScrollOutput {
            success: true,
            final_position: ScrollPosition { x, y },
        })
    }

    async fn validate_input(&self, input: &Self::Input) -> Result<()> {
        // At least one of x, y, or element must be specified
        if input.x.is_none() && input.y.is_none() && input.element.is_none() {
            return Err(anyhow::anyhow!("Must specify x, y, or element selector"));
        }
        Ok(())
    }
}

// ============================================================================
// Refresh Tool
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshInput {
    #[serde(default)]
    pub hard_reload: bool,
}

#[derive(Debug, Serialize)]
pub struct RefreshOutput {
    pub success: bool,
    pub reload_time_ms: u64,
}

pub struct RefreshTool {
    browser: Arc<Browser>,
}

impl RefreshTool {
    pub fn new(browser: Arc<Browser>) -> Self {
        Self { browser }
    }
}

#[async_trait]
impl Tool for RefreshTool {
    type Input = RefreshInput;
    type Output = RefreshOutput;

    fn name(&self) -> &str {
        "refresh_page"
    }

    fn description(&self) -> &str {
        "Refresh/reload the current page"
    }

    fn category(&self) -> ToolCategory {
        ToolCategory::Navigation
    }

    async fn execute(&self, input: Self::Input) -> Result<Self::Output> {
        let start = std::time::Instant::now();

        if input.hard_reload {
            info!("Performing hard reload");
            self.browser.hard_reload().await?;
        } else {
            info!("Performing normal reload");
            self.browser.reload().await?;
        }

        let reload_time = start.elapsed().as_millis() as u64;

        Ok(RefreshOutput {
            success: true,
            reload_time_ms: reload_time,
        })
    }
}

// ============================================================================
// Go Back Tool
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct GoBackInput {
    #[serde(default = "default_steps")]
    pub steps: u32,
}

fn default_steps() -> u32 {
    1
}

#[derive(Debug, Serialize)]
pub struct GoBackOutput {
    pub success: bool,
    pub new_url: String,
}

pub struct GoBackTool {
    browser: Arc<Browser>,
}

impl GoBackTool {
    pub fn new(browser: Arc<Browser>) -> Self {
        Self { browser }
    }
}

#[async_trait]
impl Tool for GoBackTool {
    type Input = GoBackInput;
    type Output = GoBackOutput;

    fn name(&self) -> &str {
        "go_back"
    }

    fn description(&self) -> &str {
        "Navigate back in browser history"
    }

    fn category(&self) -> ToolCategory {
        ToolCategory::Navigation
    }

    async fn execute(&self, input: Self::Input) -> Result<Self::Output> {
        info!("Going back {} steps", input.steps);

        let script = format!("window.history.go(-{})", input.steps);
        self.browser.execute_script(&script).await?;

        // Wait a bit for navigation
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        let new_url = self.browser.current_url().await?;

        Ok(GoBackOutput {
            success: true,
            new_url,
        })
    }
}

// ============================================================================
// Go Forward Tool
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct GoForwardInput {
    #[serde(default = "default_steps")]
    pub steps: u32,
}

#[derive(Debug, Serialize)]
pub struct GoForwardOutput {
    pub success: bool,
    pub new_url: String,
}

pub struct GoForwardTool {
    browser: Arc<Browser>,
}

impl GoForwardTool {
    pub fn new(browser: Arc<Browser>) -> Self {
        Self { browser }
    }
}

#[async_trait]
impl Tool for GoForwardTool {
    type Input = GoForwardInput;
    type Output = GoForwardOutput;

    fn name(&self) -> &str {
        "go_forward"
    }

    fn description(&self) -> &str {
        "Navigate forward in browser history"
    }

    fn category(&self) -> ToolCategory {
        ToolCategory::Navigation
    }

    async fn execute(&self, input: Self::Input) -> Result<Self::Output> {
        info!("Going forward {} steps", input.steps);

        let script = format!("window.history.go({})", input.steps);
        self.browser.execute_script(&script).await?;

        // Wait a bit for navigation
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        let new_url = self.browser.current_url().await?;

        Ok(GoForwardOutput {
            success: true,
            new_url,
        })
    }
}
