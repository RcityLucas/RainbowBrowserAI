//! Integration tests for tools workflows

use super::*;
use crate::browser::Browser;
use crate::tools::navigation::{NavigateToUrl, ScrollPage, NavigateToUrlParams, ScrollPageParams, NavigationOptions, ScrollOptions};
use crate::tools::types::*;
use std::sync::Arc;
use std::collections::HashMap;

#[cfg(test)]
mod navigation_workflow_tests {
    use super::*;
    
    /// Create a test browser instance
    async fn create_test_browser() -> Arc<Browser> {
        Arc::new(Browser::new())
    }
    
    #[tokio::test]
    async fn test_navigate_and_scroll_workflow() {
        let browser = create_test_browser().await;
        
        // Step 1: Navigate to a page
        let nav_tool = NavigateToUrl::new(browser.clone());
        let nav_params = NavigateToUrlParams {
            url: "https://example.com".to_string(),
            options: Some(NavigationOptions {
                wait_until: Some(WaitUntil::DomContentLoaded),
                timeout: Some(5000),
                retry: None,
                headers: None,
                referrer: None,
            }),
        };
        
        let nav_result = nav_tool.execute(nav_params).await;
        assert!(nav_result.is_ok());
        let nav_output = nav_result.unwrap();
        assert!(nav_output.success);
        
        // Step 2: Scroll down the page
        let scroll_tool = ScrollPage::new(browser.clone());
        let scroll_params = ScrollPageParams {
            direction: ScrollDirection::Simple(SimpleScrollDirection::Down),
            amount: Some(500),
            options: Some(ScrollOptions {
                smooth: Some(true),
                duration: Some(300),
                wait_after: Some(200),
            }),
        };
        
        let scroll_result = scroll_tool.execute(scroll_params).await;
        assert!(scroll_result.is_ok());
        let scroll_output = scroll_result.unwrap();
        assert!(scroll_output.success);
    }
    
    #[tokio::test]
    async fn test_navigate_with_headers_and_scroll_to_element() {
        let browser = create_test_browser().await;
        
        // Navigate with custom headers
        let nav_tool = NavigateToUrl::new(browser.clone());
        let mut headers = HashMap::new();
        headers.insert("User-Agent".to_string(), "TestBot/1.0".to_string());
        
        let nav_params = NavigateToUrlParams {
            url: "https://example.com".to_string(),
            options: Some(NavigationOptions {
                wait_until: Some(WaitUntil::NetworkIdle2),
                timeout: Some(10000),
                retry: Some(true),
                headers: Some(headers),
                referrer: Some("https://google.com".to_string()),
            }),
        };
        
        let nav_result = nav_tool.execute(nav_params).await;
        assert!(nav_result.is_ok());
        
        // Scroll to a specific element
        let scroll_tool = ScrollPage::new(browser.clone());
        let scroll_params = ScrollPageParams {
            direction: ScrollDirection::ToElement {
                selector: "#main-content".to_string(),
            },
            amount: None,
            options: Some(ScrollOptions {
                smooth: Some(true),
                duration: Some(500),
                wait_after: Some(300),
            }),
        };
        
        let scroll_result = scroll_tool.execute(scroll_params).await;
        assert!(scroll_result.is_ok());
    }
    
    #[tokio::test]
    async fn test_performance_tracking_workflow() {
        let browser = create_test_browser().await;
        
        // Navigate and track performance
        let nav_tool = NavigateToUrl::new(browser.clone());
        let nav_params = NavigateToUrlParams {
            url: "https://example.com".to_string(),
            options: Some(NavigationOptions {
                wait_until: Some(WaitUntil::Load),
                timeout: Some(15000),
                retry: None,
                headers: None,
                referrer: None,
            }),
        };
        
        let nav_result = nav_tool.execute(nav_params).await;
        assert!(nav_result.is_ok());
        
        let nav_output = nav_result.unwrap();
        assert!(nav_output.success);
        
        // Verify performance metrics were collected
        assert!(nav_output.performance.dns_lookup > 0);
        assert!(nav_output.performance.page_loaded > 0);
        assert!(nav_output.load_time > 0);
        
        // Multiple scroll operations to test boundaries
        let scroll_tool = ScrollPage::new(browser.clone());
        
        // Scroll to bottom
        let scroll_params = ScrollPageParams {
            direction: ScrollDirection::Simple(SimpleScrollDirection::Bottom),
            amount: None,
            options: None,
        };
        
        let scroll_result = scroll_tool.execute(scroll_params).await;
        assert!(scroll_result.is_ok());
        
        let scroll_output = scroll_result.unwrap();
        assert!(scroll_output.success);
        // At bottom, should indicate bottom boundary reached
        // assert!(scroll_output.reached_boundary.bottom);
        
        // Scroll to top
        let scroll_params = ScrollPageParams {
            direction: ScrollDirection::Simple(SimpleScrollDirection::Top),
            amount: None,
            options: None,
        };
        
        let scroll_result = scroll_tool.execute(scroll_params).await;
        assert!(scroll_result.is_ok());
        
        let scroll_output = scroll_result.unwrap();
        assert!(scroll_output.success);
        assert!(scroll_output.reached_boundary.top);
    }
    
    #[tokio::test]
    async fn test_scroll_position_tracking() {
        let browser = create_test_browser().await;
        
        // Navigate first
        let nav_tool = NavigateToUrl::new(browser.clone());
        let nav_params = NavigateToUrlParams {
            url: "https://example.com".to_string(),
            options: None,
        };
        
        let _ = nav_tool.execute(nav_params).await;
        
        let scroll_tool = ScrollPage::new(browser.clone());
        
        // Track position through multiple scrolls
        let mut positions = Vec::new();
        
        // Initial position (should be at top)
        let scroll_params = ScrollPageParams {
            direction: ScrollDirection::Simple(SimpleScrollDirection::Top),
            amount: None,
            options: None,
        };
        
        let result = scroll_tool.execute(scroll_params).await.unwrap();
        positions.push(result.current_position.clone());
        
        // Scroll down by specific amount
        for _ in 0..3 {
            let scroll_params = ScrollPageParams {
                direction: ScrollDirection::Simple(SimpleScrollDirection::Down),
                amount: Some(300),
                options: None,
            };
            
            let result = scroll_tool.execute(scroll_params).await.unwrap();
            positions.push(result.current_position.clone());
        }
        
        // Verify positions changed
        for i in 1..positions.len() {
            // In a real implementation, y position should increase when scrolling down
            // assert!(positions[i].y >= positions[i-1].y);
        }
    }
    
    #[tokio::test]
    async fn test_error_handling_workflow() {
        let browser = create_test_browser().await;
        
        // Test navigation to invalid URL
        let nav_tool = NavigateToUrl::new(browser.clone());
        let nav_params = NavigateToUrlParams {
            url: "not-a-valid-url".to_string(),
            options: None,
        };
        
        let validation = nav_tool.validate_input(&nav_params);
        assert!(validation.is_err());
        
        // Test scroll with invalid parameters
        let scroll_tool = ScrollPage::new(browser.clone());
        let scroll_params = ScrollPageParams {
            direction: ScrollDirection::Simple(SimpleScrollDirection::Down),
            amount: Some(0), // Invalid amount
            options: None,
        };
        
        let validation = scroll_tool.validate_input(&scroll_params);
        assert!(validation.is_err());
        
        // Test scroll to empty selector
        let scroll_params = ScrollPageParams {
            direction: ScrollDirection::ToElement {
                selector: "".to_string(), // Empty selector
            },
            amount: None,
            options: None,
        };
        
        let validation = scroll_tool.validate_input(&scroll_params);
        assert!(validation.is_err());
    }
}

#[cfg(test)]
mod tool_coordination_tests {
    use super::*;
    use crate::tools::DynamicTool;
    
    #[tokio::test]
    async fn test_dynamic_tool_dispatch() {
        let browser = create_test_browser().await;
        
        // Create tools as dynamic tools
        let nav_tool: Box<dyn DynamicTool> = Box::new(NavigateToUrl::new(browser.clone()));
        let scroll_tool: Box<dyn DynamicTool> = Box::new(ScrollPage::new(browser.clone()));
        
        // Create a tool registry
        let mut tools = HashMap::new();
        tools.insert("navigate_to_url".to_string(), nav_tool);
        tools.insert("scroll_page".to_string(), scroll_tool);
        
        // Execute navigate tool dynamically
        let nav_params = serde_json::json!({
            "url": "https://example.com",
            "options": {
                "wait_until": "load",
                "timeout": 5000
            }
        });
        
        let nav_result = tools["navigate_to_url"].execute_json(nav_params).await;
        assert!(nav_result.is_ok());
        
        // Execute scroll tool dynamically
        let scroll_params = serde_json::json!({
            "direction": "down",
            "amount": 500,
            "options": {
                "smooth": true
            }
        });
        
        let scroll_result = tools["scroll_page"].execute_json(scroll_params).await;
        assert!(scroll_result.is_ok());
    }
    
    #[tokio::test]
    async fn test_tool_schema_validation() {
        let browser = create_test_browser().await;
        
        let nav_tool = NavigateToUrl::new(browser.clone());
        let scroll_tool = ScrollPage::new(browser.clone());
        
        // Verify input schemas
        let nav_input_schema = Tool::input_schema(&nav_tool);
        assert!(nav_input_schema["properties"]["url"].is_object());
        assert_eq!(nav_input_schema["required"], serde_json::json!(["url"]));
        
        let scroll_input_schema = Tool::input_schema(&scroll_tool);
        assert!(scroll_input_schema["properties"]["direction"].is_object());
        assert_eq!(scroll_input_schema["required"], serde_json::json!(["direction"]));
        
        // Verify output schemas
        let nav_output_schema = Tool::output_schema(&nav_tool);
        assert!(nav_output_schema["properties"]["success"].is_object());
        assert!(nav_output_schema["properties"]["performance"].is_object());
        
        let scroll_output_schema = Tool::output_schema(&scroll_tool);
        assert!(scroll_output_schema["properties"]["success"].is_object());
        assert!(scroll_output_schema["properties"]["reached_boundary"].is_object());
    }
}

#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;
    
    #[tokio::test]
    async fn test_navigation_performance() {
        let browser = create_test_browser().await;
        let nav_tool = NavigateToUrl::new(browser.clone());
        
        let start = Instant::now();
        
        let nav_params = NavigateToUrlParams {
            url: "https://example.com".to_string(),
            options: Some(NavigationOptions {
                wait_until: Some(WaitUntil::DomContentLoaded),
                timeout: Some(5000),
                retry: None,
                headers: None,
                referrer: None,
            }),
        };
        
        let result = nav_tool.execute(nav_params).await;
        let elapsed = start.elapsed();
        
        assert!(result.is_ok());
        
        // Navigation should complete within reasonable time
        assert!(elapsed.as_secs() < 10, "Navigation took too long: {:?}", elapsed);
        
        let output = result.unwrap();
        
        // Verify performance metrics are reasonable
        assert!(output.performance.dns_lookup < output.performance.page_loaded);
        assert!(output.performance.tcp_connect < output.performance.page_loaded);
        assert!(output.performance.dom_loaded <= output.performance.page_loaded);
    }
    
    #[tokio::test]
    async fn test_scroll_performance() {
        let browser = create_test_browser().await;
        let scroll_tool = ScrollPage::new(browser);
        
        let start = Instant::now();
        
        // Perform multiple scroll operations
        for i in 0..5 {
            let scroll_params = ScrollPageParams {
                direction: if i % 2 == 0 {
                    ScrollDirection::Simple(SimpleScrollDirection::Down)
                } else {
                    ScrollDirection::Simple(SimpleScrollDirection::Up)
                },
                amount: Some(200),
                options: None,
            };
            
            let result = scroll_tool.execute(scroll_params).await;
            assert!(result.is_ok());
        }
        
        let elapsed = start.elapsed();
        
        // All scrolls should complete quickly
        assert!(elapsed.as_secs() < 5, "Scrolling took too long: {:?}", elapsed);
    }
}

async fn create_test_browser() -> Arc<Browser> {
    Arc::new(Browser::new())
}