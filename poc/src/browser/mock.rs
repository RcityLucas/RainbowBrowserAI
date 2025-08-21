// Mock browser implementation for testing
use anyhow::Result;
use serde_json::{Value, json};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use std::time::Duration;
use async_trait::async_trait;
use crate::browser::BrowserOps;

/// Mock browser for testing tools without a real browser
#[derive(Clone)]
pub struct MockBrowser {
    state: Arc<RwLock<MockBrowserState>>,
}

#[derive(Default)]
struct MockBrowserState {
    current_url: String,
    elements: HashMap<String, MockElement>,
    script_results: HashMap<String, Value>,
    scroll_position: (i32, i32),
    page_dimensions: (u32, u32),
    selected_options: HashMap<String, Vec<String>>,
}

#[derive(Clone)]
struct MockElement {
    tag_name: String,
    text: String,
    value: String,
    visible: bool,
    enabled: bool,
}

impl MockBrowser {
    pub fn new() -> Self {
        let mut state = MockBrowserState::default();
        state.page_dimensions = (1920, 1080);
        
        // Pre-populate some mock elements for testing
        state.elements.insert("#submit-button".to_string(), MockElement {
            tag_name: "button".to_string(),
            text: "Submit".to_string(),
            value: "".to_string(),
            visible: true,
            enabled: true,
        });
        
        state.elements.insert("#username".to_string(), MockElement {
            tag_name: "input".to_string(),
            text: "".to_string(),
            value: "".to_string(),
            visible: true,
            enabled: true,
        });
        
        state.elements.insert("#country".to_string(), MockElement {
            tag_name: "select".to_string(),
            text: "".to_string(),
            value: "US".to_string(),
            visible: true,
            enabled: true,
        });
        
        Self {
            state: Arc::new(RwLock::new(state)),
        }
    }
    
    pub async fn navigate_to(&self, url: &str) -> Result<()> {
        let mut state = self.state.write().await;
        state.current_url = url.to_string();
        Ok(())
    }
    
    pub async fn current_url(&self) -> Result<String> {
        let state = self.state.read().await;
        Ok(state.current_url.clone())
    }
    
    pub async fn wait_for_load_event(&self) -> Result<()> {
        // Mock implementation - simulate wait
        tokio::time::sleep(Duration::from_millis(10)).await;
        Ok(())
    }
    
    pub async fn wait_for_dom_content_loaded(&self) -> Result<()> {
        // Mock implementation - simulate wait
        tokio::time::sleep(Duration::from_millis(10)).await;
        Ok(())
    }
    
    pub async fn execute_script(&self, _script: &str, _args: Vec<Value>) -> Result<Value> {
        // Return mock script result
        Ok(json!({
            "success": true,
            "result": "mock_result"
        }))
    }
    
    pub async fn click(&self, selector: &str) -> Result<()> {
        let state = self.state.read().await;
        if !state.elements.contains_key(selector) && !selector.is_empty() {
            // Allow any non-empty selector in mock mode
            return Ok(());
        }
        Ok(())
    }
    
    pub async fn send_keys(&self, selector: &str, text: &str) -> Result<()> {
        let mut state = self.state.write().await;
        if let Some(element) = state.elements.get_mut(selector) {
            element.value = text.to_string();
        }
        Ok(())
    }
    
    pub async fn clear(&self, selector: &str) -> Result<()> {
        let mut state = self.state.write().await;
        if let Some(element) = state.elements.get_mut(selector) {
            element.value.clear();
        }
        Ok(())
    }
    
    pub async fn get_text(&self, selector: &str) -> Result<String> {
        let state = self.state.read().await;
        if let Some(element) = state.elements.get(selector) {
            Ok(element.text.clone())
        } else {
            Ok("Mock text".to_string())
        }
    }
    
    pub async fn element_exists(&self, selector: &str) -> Result<bool> {
        let state = self.state.read().await;
        Ok(state.elements.contains_key(selector) || !selector.is_empty())
    }
    
    pub async fn wait_for_element(&self, _selector: &str, _timeout: Duration) -> Result<()> {
        // Mock implementation - simulate wait
        tokio::time::sleep(Duration::from_millis(10)).await;
        Ok(())
    }
    
    pub async fn scroll_to(&self, x: i32, y: i32) -> Result<()> {
        let mut state = self.state.write().await;
        state.scroll_position = (x, y);
        Ok(())
    }
    
    pub async fn get_scroll_position(&self) -> Result<(i32, i32)> {
        let state = self.state.read().await;
        Ok(state.scroll_position)
    }
    
    // Fix method signature conflicts - remove duplicate methods
    /* Duplicate methods removed:
    pub async fn scroll(&self, x: f64, y: f64) -> Result<()>
    pub async fn get_scroll_position(&self) -> Result<(f64, f64)>
    */
    
    pub async fn get_page_dimensions(&self) -> Result<(u32, u32)> {
        let state = self.state.read().await;
        Ok(state.page_dimensions)
    }
    
    pub async fn select_by_value(&self, selector: &str, value: &str) -> Result<()> {
        let mut state = self.state.write().await;
        state.selected_options.insert(selector.to_string(), vec![value.to_string()]);
        Ok(())
    }
    
    pub async fn get_selected_options(&self, selector: &str) -> Result<Vec<String>> {
        let state = self.state.read().await;
        Ok(state.selected_options.get(selector).cloned().unwrap_or_default())
    }
    
    pub async fn find_element(&self, selector: &str) -> Result<()> {
        // Mock implementation - just check if selector is not empty
        if selector.is_empty() {
            anyhow::bail!("Element not found");
        }
        Ok(())
    }
    
    pub async fn type_text(&self, _selector: &str, _text: &str) -> Result<()> {
        // Mock typing implementation
        Ok(())
    }
    
    pub async fn execute_script_detailed(&self, script: &str, _args: Vec<Value>) -> Result<Value> {
        let state = self.state.read().await;
        
        // Return mock results based on script content
        if script.contains("querySelector") {
            if script.contains("getBoundingClientRect") {
                return Ok(serde_json::json!({
                    "x": 100,
                    "y": 200,
                    "width": 300,
                    "height": 50
                }));
            }
            
            if script.contains(".value") {
                return Ok(serde_json::json!("test_value"));
            }
            
            // Mock element info
            return Ok(serde_json::json!({
                "tagName": "button",
                "id": "test-id",
                "classes": ["btn", "btn-primary"],
                "text": "Click me",
                "type": "submit",
                "href": null,
                "boundingBox": {
                    "x": 100,
                    "y": 200,
                    "width": 300,
                    "height": 50
                },
                "wasVisible": true,
                "wasEnabled": true
            }));
        }
        
        if script.contains("window.location.href") {
            return Ok(serde_json::json!(state.current_url));
        }
        
        if script.contains("document.title") {
            return Ok(serde_json::json!("Test Page"));
        }
        
        if script.contains("scrollY") || script.contains("pageYOffset") {
            return Ok(serde_json::json!(state.scroll_position.1));
        }
        
        if script.contains("scrollX") || script.contains("pageXOffset") {
            return Ok(serde_json::json!(state.scroll_position.0));
        }
        
        // Default return
        Ok(serde_json::json!(true))
    }
    
    pub async fn get_viewport_size(&self) -> Result<(u32, u32)> {
        Ok((1920, 1080))
    }
    
    pub async fn get_document_size(&self) -> Result<(u32, u32)> {
        Ok((1920, 3000))
    }
}

// Implement SimpleBrowser-compatible interface
impl MockBrowser {
    pub fn browser(&self) -> Option<&Self> {
        Some(self)
    }
}

// Implement BrowserOps trait for MockBrowser
#[async_trait]
impl BrowserOps for MockBrowser {
    async fn navigate_to(&self, url: &str) -> Result<()> {
        self.navigate_to(url).await
    }
    
    async fn current_url(&self) -> Result<String> {
        self.current_url().await
    }
    
    async fn wait_for_load_event(&self) -> Result<()> {
        self.wait_for_load_event().await
    }
    
    async fn wait_for_dom_content_loaded(&self) -> Result<()> {
        self.wait_for_dom_content_loaded().await
    }
}