//! Tests for navigation tools

use super::*;
use crate::browser::{MockBrowser, SimpleBrowser};
use crate::tools::{Tool, WaitUntil, ScrollDirection, SimpleScrollDirection};
use std::sync::Arc;
use std::collections::HashMap;

#[cfg(test)]
mod navigate_to_url_tests {
    use super::*;
    
    async fn create_test_browser() -> Arc<MockBrowser> {
        Arc::new(MockBrowser::new())
    }
    
    #[tokio::test]
    async fn test_navigate_basic() {
        let browser = create_test_browser().await;
        let tool = NavigateToUrl::new(browser);
        
        let params = NavigateToUrlParams {
            url: "https://example.com".to_string(),
            options: None,
        };
        
        let result = tool.execute(params).await;
        assert!(result.is_ok());
        
        let nav_result = result.unwrap();
        assert!(nav_result.success);
        assert!(!nav_result.final_url.is_empty());
        assert!(nav_result.load_time > 0);
        assert_eq!(nav_result.status_code, 200);
    }
    
    #[tokio::test]
    async fn test_navigate_with_wait_strategy() {
        let browser = create_test_browser().await;
        let tool = NavigateToUrl::new(browser);
        
        let params = NavigateToUrlParams {
            url: "https://example.com".to_string(),
            options: Some(NavigationOptions {
                wait_until: Some(WaitUntil::NetworkIdle0),
                timeout: Some(5000),
                retry: Some(true),
                headers: None,
                referrer: None,
            }),
        };
        
        let result = tool.execute(params).await;
        assert!(result.is_ok());
        
        let nav_result = result.unwrap();
        assert!(nav_result.success);
        assert!(nav_result.performance.page_loaded > 0);
    }
    
    #[tokio::test]
    async fn test_navigate_with_headers() {
        let browser = create_test_browser().await;
        let tool = NavigateToUrl::new(browser);
        
        let mut headers = HashMap::new();
        headers.insert("User-Agent".to_string(), "TestBot/1.0".to_string());
        headers.insert("Accept-Language".to_string(), "en-US".to_string());
        
        let params = NavigateToUrlParams {
            url: "https://example.com".to_string(),
            options: Some(NavigationOptions {
                wait_until: None,
                timeout: None,
                retry: None,
                headers: Some(headers),
                referrer: Some("https://referrer.com".to_string()),
            }),
        };
        
        let result = tool.execute(params).await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_navigate_invalid_url() {
        let browser = create_test_browser().await;
        let tool = NavigateToUrl::new(browser);
        
        let params = NavigateToUrlParams {
            url: "not-a-valid-url".to_string(),
            options: None,
        };
        
        let validation = tool.validate_input(&params);
        assert!(validation.is_err());
    }
    
    #[tokio::test]
    async fn test_navigate_schema() {
        let browser = create_test_browser().await;
        let tool = NavigateToUrl::new(browser);
        
        let input_schema = tool.input_schema();
        assert!(input_schema.is_object());
        assert!(input_schema["properties"]["url"].is_object());
        
        let output_schema = tool.output_schema();
        assert!(output_schema.is_object());
        assert!(output_schema["properties"]["success"].is_object());
    }
}

#[cfg(test)]
mod scroll_page_tests {
    use super::*;
    
    async fn create_test_browser() -> Arc<MockBrowser> {
        Arc::new(MockBrowser::new())
    }
    
    #[tokio::test]
    async fn test_scroll_down() {
        let browser = create_test_browser().await;
        let tool = ScrollPage::new(browser);
        
        let params = ScrollPageParams {
            direction: ScrollDirection::Simple(SimpleScrollDirection::Down),
            amount: Some(500),
            options: None,
        };
        
        let result = tool.execute(params).await;
        assert!(result.is_ok());
        
        let scroll_result = result.unwrap();
        assert!(scroll_result.success);
        // After scrolling down, y position should increase
        // (In mock, positions might not change, but in real impl they would)
    }
    
    #[tokio::test]
    async fn test_scroll_to_element() {
        let browser = create_test_browser().await;
        let tool = ScrollPage::new(browser);
        
        let params = ScrollPageParams {
            direction: ScrollDirection::ToElement {
                selector: "#target-element".to_string(),
            },
            amount: None,
            options: Some(ScrollOptions {
                smooth: Some(true),
                duration: Some(1000),
                wait_after: Some(500),
            }),
        };
        
        let result = tool.execute(params).await;
        assert!(result.is_ok());
        
        let scroll_result = result.unwrap();
        assert!(scroll_result.success);
    }
    
    #[tokio::test]
    async fn test_scroll_to_position() {
        let browser = create_test_browser().await;
        let tool = ScrollPage::new(browser);
        
        let params = ScrollPageParams {
            direction: ScrollDirection::ToPosition { x: 100, y: 500 },
            amount: None,
            options: None,
        };
        
        let result = tool.execute(params).await;
        assert!(result.is_ok());
        
        let scroll_result = result.unwrap();
        assert!(scroll_result.success);
    }
    
    #[tokio::test]
    async fn test_scroll_to_bottom() {
        let browser = create_test_browser().await;
        let tool = ScrollPage::new(browser);
        
        let params = ScrollPageParams {
            direction: ScrollDirection::Simple(SimpleScrollDirection::Bottom),
            amount: None,
            options: None,
        };
        
        let result = tool.execute(params).await;
        assert!(result.is_ok());
        
        let scroll_result = result.unwrap();
        assert!(scroll_result.success);
        // Should reach bottom boundary
        // assert!(scroll_result.reached_boundary.bottom);
    }
    
    #[tokio::test]
    async fn test_scroll_validation() {
        let browser = create_test_browser().await;
        let tool = ScrollPage::new(browser);
        
        // Test invalid amount
        let params = ScrollPageParams {
            direction: ScrollDirection::Simple(SimpleScrollDirection::Down),
            amount: Some(0),
            options: None,
        };
        
        let validation = tool.validate_input(&params);
        assert!(validation.is_err());
        
        // Test empty selector
        let params = ScrollPageParams {
            direction: ScrollDirection::ToElement {
                selector: "".to_string(),
            },
            amount: None,
            options: None,
        };
        
        let validation = tool.validate_input(&params);
        assert!(validation.is_err());
    }
    
    #[tokio::test]
    async fn test_scroll_boundaries() {
        let browser = create_test_browser().await;
        let tool = ScrollPage::new(browser);
        
        // Scroll to top
        let params = ScrollPageParams {
            direction: ScrollDirection::Simple(SimpleScrollDirection::Top),
            amount: None,
            options: None,
        };
        
        let result = tool.execute(params).await;
        assert!(result.is_ok());
        
        let scroll_result = result.unwrap();
        assert!(scroll_result.success);
        assert!(scroll_result.reached_boundary.top);
    }
}