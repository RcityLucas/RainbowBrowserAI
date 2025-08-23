// Test adapter to make MockBrowser compatible with SimpleBrowser interface
use super::mock::MockBrowser;
use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;

/// Test adapter that wraps MockBrowser to provide SimpleBrowser-compatible interface
pub struct TestBrowserAdapter {
    mock: MockBrowser,
}

impl TestBrowserAdapter {
    pub fn new() -> Self {
        Self {
            mock: MockBrowser::new(),
        }
    }
    
    // SimpleBrowser-compatible methods
    pub async fn navigate_to(&self, url: &str) -> Result<()> {
        self.mock.navigate_to(url).await
    }
    
    pub async fn current_url(&self) -> Result<String> {
        self.mock.current_url().await
    }
    
    pub async fn wait_for_load_event(&self) -> Result<()> {
        self.mock.wait_for_load_event().await
    }
    
    pub async fn wait_for_dom_content_loaded(&self) -> Result<()> {
        self.mock.wait_for_dom_content_loaded().await
    }
    
    pub async fn execute_script(&self, script: &str, args: Vec<serde_json::Value>) -> Result<serde_json::Value> {
        self.mock.execute_script(script, args).await
    }
    
    pub async fn click(&self, selector: &str) -> Result<()> {
        self.mock.click(selector).await
    }
    
    pub async fn send_keys(&self, selector: &str, text: &str) -> Result<()> {
        self.mock.send_keys(selector, text).await
    }
    
    pub async fn clear(&self, selector: &str) -> Result<()> {
        self.mock.clear(selector).await
    }
    
    pub async fn get_text(&self, selector: &str) -> Result<String> {
        self.mock.get_text(selector).await
    }
    
    pub async fn element_exists(&self, selector: &str) -> Result<bool> {
        self.mock.element_exists(selector).await
    }
    
    pub async fn wait_for_element(&self, selector: &str, timeout: Duration) -> Result<()> {
        self.mock.wait_for_element(selector, timeout).await
    }
    
    pub async fn scroll_to(&self, x: i32, y: i32) -> Result<()> {
        self.mock.scroll_to(x, y).await
    }
    
    pub async fn get_scroll_position(&self) -> Result<(i32, i32)> {
        self.mock.get_scroll_position().await
    }
    
    pub async fn get_page_dimensions(&self) -> Result<(u32, u32)> {
        self.mock.get_page_dimensions().await
    }
    
    pub async fn select_by_value(&self, selector: &str, value: &str) -> Result<()> {
        self.mock.select_by_value(selector, value).await
    }
    
    pub async fn get_selected_options(&self, selector: &str) -> Result<Vec<String>> {
        self.mock.get_selected_options(selector).await
    }
}

// Create an Arc-compatible constructor for tests
pub fn create_test_browser() -> Arc<TestBrowserAdapter> {
    Arc::new(TestBrowserAdapter::new())
}