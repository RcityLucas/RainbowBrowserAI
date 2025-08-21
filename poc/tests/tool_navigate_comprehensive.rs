use rainbow_poc::tools::navigation::{NavigateToUrl, NavigateInput};
use rainbow_poc::tools::Tool;
use tokio;
use std::time::Duration;

#[tokio::test]
async fn test_navigate_tool_success() {
    let mut tool = NavigateToUrl::new();
    let input = NavigateInput {
        url: "https://example.com".to_string(),
        timeout: Some(Duration::from_secs(30)),
        retries: Some(3),
        screenshot: Some(false),
        wait_for_load: Some(true),
        screenshot_options: None,
    };
    
    let result = tool.execute(input).await;
    
    // Validate successful result
    assert!(result.is_ok(), "Navigation should succeed for valid URL");
    
    let output = result.unwrap();
    assert_eq!(output.final_url, "https://example.com/");
    assert!(output.page_title.contains("Example"));
    assert!(output.load_time_ms > 0);
    assert!(output.load_time_ms < 30000);
    assert_eq!(output.status_code, 200);
    assert_eq!(output.success, true);
    assert_eq!(output.retry_count, 0);
}

#[tokio::test] 
async fn test_navigate_tool_invalid_url() {
    let mut tool = NavigateToUrl::new();
    let input = NavigateInput {
        url: "not-a-valid-url".to_string(),
        timeout: Some(Duration::from_secs(5)),
        retries: Some(1),
        screenshot: Some(false),
        wait_for_load: Some(false),
        screenshot_options: None,
    };
    
    let result = tool.execute(input).await;
    
    // Should fail for invalid URL
    assert!(result.is_err(), "Should fail for invalid URL");
    
    let error = result.unwrap_err();
    assert!(error.to_string().contains("Invalid URL"));
}

#[tokio::test]
async fn test_navigate_tool_timeout() {
    let mut tool = NavigateToUrl::new();
    let input = NavigateInput {
        url: "http://httpstat.us/200?sleep=35000".to_string(), // 35 second delay
        timeout: Some(Duration::from_secs(5)), // 5 second timeout
        retries: Some(1),
        screenshot: Some(false), 
        wait_for_load: Some(true),
        screenshot_options: None,
    };
    
    let start = std::time::Instant::now();
    let result = tool.execute(input).await;
    let duration = start.elapsed();
    
    // Should timeout and fail
    assert!(result.is_err(), "Should timeout for slow URL");
    assert!(duration < Duration::from_secs(10), "Should timeout quickly");
    
    let error = result.unwrap_err();
    assert!(error.to_string().contains("timeout") || error.to_string().contains("Timeout"));
}

#[tokio::test]
async fn test_navigate_tool_retry_logic() {
    let mut tool = NavigateToUrl::new();
    let input = NavigateInput {
        url: "http://httpstat.us/503".to_string(), // Returns 503 error
        timeout: Some(Duration::from_secs(10)),
        retries: Some(3),
        screenshot: Some(false),
        wait_for_load: Some(false),
        screenshot_options: None,
    };
    
    let result = tool.execute(input).await;
    
    // Should eventually fail after retries
    assert!(result.is_err(), "Should fail after retries for 503 error");
}

#[tokio::test]
async fn test_navigate_tool_screenshot_capture() {
    let mut tool = NavigateToUrl::new();
    let input = NavigateInput {
        url: "https://example.com".to_string(),
        timeout: Some(Duration::from_secs(30)),
        retries: Some(2),
        screenshot: Some(true),
        wait_for_load: Some(true),
        screenshot_options: Some(crate::tools::ScreenshotOptions {
            full_page: true,
            viewport: Some((1920, 1080)),
            format: Some("png".to_string()),
            filename: Some("test_screenshot.png".to_string()),
        }),
    };
    
    let result = tool.execute(input).await;
    
    assert!(result.is_ok(), "Navigation with screenshot should succeed");
    
    let output = result.unwrap();
    assert!(output.screenshot_path.is_some(), "Screenshot path should be provided");
    
    // Verify screenshot file exists and has reasonable size
    let screenshot_path = output.screenshot_path.unwrap();
    assert!(std::path::Path::new(&screenshot_path).exists(), "Screenshot file should exist");
    
    let metadata = std::fs::metadata(&screenshot_path).unwrap();
    assert!(metadata.len() > 1000, "Screenshot should be reasonable size (>1KB)");
    assert!(metadata.len() < 10_000_000, "Screenshot should not be too large (<10MB)");
    
    // Clean up test file
    std::fs::remove_file(&screenshot_path).unwrap();
}

#[tokio::test]
async fn test_navigate_tool_load_metrics() {
    let mut tool = NavigateToUrl::new();
    let input = NavigateInput {
        url: "https://www.google.com".to_string(),
        timeout: Some(Duration::from_secs(30)),
        retries: Some(2),
        screenshot: Some(false),
        wait_for_load: Some(true),
        screenshot_options: None,
    };
    
    let result = tool.execute(input).await;
    assert!(result.is_ok(), "Google navigation should succeed");
    
    let output = result.unwrap();
    
    // Validate performance metrics
    assert!(output.load_time_ms > 0, "Load time should be positive");
    assert!(output.load_time_ms < 30000, "Load time should be under 30 seconds");
    
    // Validate page metrics
    assert!(!output.page_title.is_empty(), "Page title should not be empty");
    assert!(output.final_url.starts_with("https://"), "Final URL should be HTTPS");
    assert_eq!(output.status_code, 200, "Status code should be 200 for Google");
    
    // Validate success metrics
    assert_eq!(output.success, true, "Navigation should be successful");
    assert!(output.retry_count <= 2, "Retry count should not exceed max retries");
}

// Performance benchmark test
#[tokio::test]
async fn test_navigate_tool_performance_overhead() {
    let mut tool = NavigateToUrl::new();
    
    // Test claimed <100ms overhead
    let start_total = std::time::Instant::now();
    
    let input = NavigateInput {
        url: "https://example.com".to_string(),
        timeout: Some(Duration::from_secs(30)),
        retries: Some(1),
        screenshot: Some(false),
        wait_for_load: Some(false), // Skip waiting to measure just navigation overhead
        screenshot_options: None,
    };
    
    let result = tool.execute(input).await;
    let total_time = start_total.elapsed();
    
    assert!(result.is_ok(), "Navigation should succeed");
    
    let output = result.unwrap();
    let overhead = total_time.as_millis() - output.load_time_ms as u128;
    
    // Validate claimed <100ms overhead (allowing some margin)
    assert!(overhead < 500, "Navigation overhead should be <500ms (claimed <100ms), actual: {}ms", overhead);
}