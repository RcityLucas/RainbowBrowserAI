use rainbow_poc::{SimpleBrowser, CostTracker, Config, WorkflowEngine, Workflow, BrowserPool, Cache, MetricsCollector};
use anyhow::Result;
use std::time::Duration;
use tokio::time::timeout;

#[tokio::test]
async fn test_browser_navigation() -> Result<()> {
    let browser = SimpleBrowser::new().await?;
    
    // Test navigation to a simple page
    browser.navigate_to("https://example.com").await?;
    
    // Verify we can get the title
    let title = browser.get_title().await?;
    assert!(title.contains("Example"));
    
    browser.close().await?;
    Ok(())
}

#[tokio::test]
async fn test_browser_screenshot() -> Result<()> {
    let browser = SimpleBrowser::new().await?;
    
    browser.navigate_to("https://example.com").await?;
    
    // Take a screenshot
    let filename = "test_screenshot.png";
    browser.take_screenshot(filename).await?;
    
    // Verify file was created
    assert!(std::path::Path::new(&format!("screenshots/{}", filename)).exists());
    
    // Clean up
    std::fs::remove_file(format!("screenshots/{}", filename)).ok();
    browser.close().await?;
    Ok(())
}

#[tokio::test]
async fn test_browser_pool_reuse() -> Result<()> {
    let pool = BrowserPool::new();
    
    // Acquire first browser
    let handle1 = pool.acquire().await?;
    assert!(handle1.browser().is_some());
    
    // Release it back to pool
    handle1.release().await;
    
    // Acquire again - should reuse the same browser
    let handle2 = pool.acquire().await?;
    assert!(handle2.browser().is_some());
    
    // Check pool stats
    let stats = pool.stats().await;
    assert_eq!(stats.total_created, 1); // Only one browser created
    assert_eq!(stats.total_checkouts, 2); // But checked out twice
    
    Ok(())
}

#[tokio::test]
async fn test_browser_pool_concurrent() -> Result<()> {
    let pool = BrowserPool::with_config(2, Duration::from_secs(60), Duration::from_secs(300), 10);
    
    // Acquire multiple browsers concurrently
    let (handle1, handle2) = tokio::join!(
        pool.acquire(),
        pool.acquire()
    );
    
    assert!(handle1.is_ok());
    assert!(handle2.is_ok());
    
    let stats = pool.stats().await;
    assert_eq!(stats.total_created, 2);
    assert_eq!(stats.current_size, 2);
    
    Ok(())
}

#[tokio::test]
async fn test_cost_tracker_budget() -> Result<()> {
    let mut tracker = CostTracker::new(1.0); // $1 budget
    
    // Record operations
    tracker.record_operation(
        "test".to_string(),
        "Test operation 1".to_string(),
        0.30,
        true
    )?;
    
    assert!(tracker.can_afford(0.50)); // Can still afford $0.50
    
    tracker.record_operation(
        "test".to_string(),
        "Test operation 2".to_string(),
        0.60,
        true
    )?;
    
    assert!(!tracker.can_afford(0.20)); // Cannot afford $0.20 (would exceed budget)
    
    Ok(())
}

#[tokio::test]
async fn test_cost_tracker_daily_reset() -> Result<()> {
    let mut tracker = CostTracker::new(1.0);
    
    // Simulate operations from yesterday
    tracker.record_operation(
        "test".to_string(),
        "Yesterday's operation".to_string(),
        0.90,
        true
    )?;
    
    // Force daily reset check
    tracker.reset_if_new_day();
    
    // Should be able to afford full budget again if it's a new day
    // Note: This test might fail if run exactly at midnight
    // In production, we'd mock the time
    
    Ok(())
}

#[tokio::test]
async fn test_cache_ttl() -> Result<()> {
    let cache: Cache<String, String> = Cache::new(Duration::from_millis(100), 10);
    
    // Insert value
    cache.insert("key1".to_string(), "value1".to_string()).await;
    
    // Should be retrievable immediately
    let value = cache.get(&"key1".to_string()).await;
    assert_eq!(value, Some("value1".to_string()));
    
    // Wait for TTL to expire
    tokio::time::sleep(Duration::from_millis(150)).await;
    
    // Should be expired now
    let value = cache.get(&"key1".to_string()).await;
    assert_eq!(value, None);
    
    Ok(())
}

#[tokio::test]
async fn test_cache_lru_eviction() -> Result<()> {
    let cache: Cache<i32, String> = Cache::new(Duration::from_secs(60), 3); // Max 3 items
    
    // Insert 3 items
    cache.insert(1, "one".to_string()).await;
    cache.insert(2, "two".to_string()).await;
    cache.insert(3, "three".to_string()).await;
    
    // Access item 1 to make it recently used
    cache.get(&1).await;
    
    // Insert 4th item - should evict least recently used (item 2)
    cache.insert(4, "four".to_string()).await;
    
    // Item 2 should be evicted
    assert_eq!(cache.get(&2).await, None);
    
    // Others should still be there
    assert_eq!(cache.get(&1).await, Some("one".to_string()));
    assert_eq!(cache.get(&3).await, Some("three".to_string()));
    assert_eq!(cache.get(&4).await, Some("four".to_string()));
    
    Ok(())
}

#[tokio::test]
async fn test_workflow_basic_execution() -> Result<()> {
    let yaml = r#"
name: Test Workflow
description: Simple test workflow
steps:
  - name: Navigate
    action:
      type: navigate
      url: https://example.com
"#;
    
    let workflow: Workflow = serde_yaml::from_str(yaml)?;
    assert_eq!(workflow.name, "Test Workflow");
    assert_eq!(workflow.steps.len(), 1);
    
    // Note: Actual execution would require a browser
    // This just tests parsing and structure
    
    Ok(())
}

#[tokio::test]
async fn test_workflow_with_variables() -> Result<()> {
    let yaml = r#"
name: Test Workflow
description: Workflow with variables
inputs:
  - name: target_url
    input_type: string
    required: true
steps:
  - name: Navigate
    action:
      type: navigate
      url: "{{target_url}}"
"#;
    
    let mut workflow: Workflow = serde_yaml::from_str(yaml)?;
    let mut engine = WorkflowEngine::new();
    
    // Set input variable
    engine.set_variable("target_url", serde_json::json!("https://example.com"));
    
    // Render the URL with variables
    let step = &workflow.steps[0];
    if let serde_json::Value::String(url) = &step.action["url"] {
        let rendered = engine.render_template(url)?;
        assert_eq!(rendered, "https://example.com");
    }
    
    Ok(())
}

#[tokio::test]
async fn test_metrics_collection() -> Result<()> {
    let collector = MetricsCollector::new();
    
    // Record some operations
    collector.record_operation(Duration::from_millis(100), true, 0.01).await;
    collector.record_operation(Duration::from_millis(200), false, 0.02).await;
    collector.record_operation(Duration::from_millis(150), true, 0.015).await;
    
    let metrics = collector.get_metrics().await;
    assert_eq!(metrics.operations_total, 3);
    assert_eq!(metrics.operations_success, 2);
    assert_eq!(metrics.operations_failed, 1);
    assert!((metrics.total_cost - 0.045).abs() < 0.001);
    
    // Check success rate calculation
    let success_rate = metrics.success_rate();
    assert!((success_rate - 66.66).abs() < 1.0);
    
    Ok(())
}

#[tokio::test]
async fn test_metrics_percentiles() -> Result<()> {
    let collector = MetricsCollector::new();
    
    // Record operations with various durations
    for i in 1..=100 {
        collector.record_operation(
            Duration::from_millis(i * 10),
            true,
            0.001
        ).await;
    }
    
    let metrics = collector.get_metrics().await;
    let percentiles = metrics.operation_duration_percentiles();
    
    // P50 should be around 500ms
    assert!((percentiles.p50 - 500.0).abs() < 50.0);
    
    // P95 should be around 950ms
    assert!((percentiles.p95 - 950.0).abs() < 50.0);
    
    Ok(())
}

#[tokio::test]
async fn test_config_loading() -> Result<()> {
    let config = Config::default();
    
    // Test default values
    assert_eq!(config.browser.driver_url, "http://localhost:9515");
    assert_eq!(config.budget.daily_limit, 5.0);
    assert_eq!(config.pool.max_size, 3);
    
    // Test validation
    assert!(config.validate().is_ok());
    
    Ok(())
}

#[tokio::test]
async fn test_config_env_override() -> Result<()> {
    // Set environment variable
    std::env::set_var("DAILY_BUDGET", "10.0");
    
    let config = Config::from_env()?;
    assert_eq!(config.budget.daily_limit, 10.0);
    
    // Clean up
    std::env::remove_var("DAILY_BUDGET");
    
    Ok(())
}

#[tokio::test]
async fn test_browser_timeout() -> Result<()> {
    let browser = SimpleBrowser::new_with_config(1, Duration::from_secs(2)).await?;
    
    // Try to navigate to a non-existent domain (should timeout)
    let result = timeout(
        Duration::from_secs(5),
        browser.navigate_to("http://this-domain-definitely-does-not-exist-123456789.com")
    ).await;
    
    assert!(result.is_ok()); // Timeout didn't expire
    assert!(result.unwrap().is_err()); // But navigation failed
    
    browser.close().await?;
    Ok(())
}

#[tokio::test] 
async fn test_browser_retry_logic() -> Result<()> {
    let browser = SimpleBrowser::new_with_config(3, Duration::from_secs(5)).await?;
    
    // Test retry on a stable site
    let result = browser.navigate_to_with_retry("https://example.com", 3).await;
    assert!(result.is_ok());
    
    browser.close().await?;
    Ok(())
}

// Performance benchmark test
#[tokio::test]
#[ignore] // Run with --ignored flag for benchmarks
async fn bench_browser_operations() -> Result<()> {
    let iterations = 10;
    let mut durations = Vec::new();
    
    for _ in 0..iterations {
        let start = std::time::Instant::now();
        
        let browser = SimpleBrowser::new().await?;
        browser.navigate_to("https://example.com").await?;
        browser.close().await?;
        
        durations.push(start.elapsed());
    }
    
    let total: Duration = durations.iter().sum();
    let avg = total / iterations;
    
    println!("Browser operation benchmark:");
    println!("  Iterations: {}", iterations);
    println!("  Average time: {:?}", avg);
    println!("  Min time: {:?}", durations.iter().min().unwrap());
    println!("  Max time: {:?}", durations.iter().max().unwrap());
    
    assert!(avg < Duration::from_secs(5), "Operations taking too long");
    
    Ok(())
}

#[tokio::test]
#[ignore] // Run with --ignored flag for benchmarks
async fn bench_cache_operations() -> Result<()> {
    let cache: Cache<i32, String> = Cache::new(Duration::from_secs(60), 1000);
    let iterations = 10000;
    
    let start = std::time::Instant::now();
    
    // Benchmark insertions
    for i in 0..iterations {
        cache.insert(i, format!("value_{}", i)).await;
    }
    
    let insert_time = start.elapsed();
    
    let start = std::time::Instant::now();
    
    // Benchmark lookups
    for i in 0..iterations {
        cache.get(&i).await;
    }
    
    let lookup_time = start.elapsed();
    
    println!("Cache benchmark:");
    println!("  {} insertions: {:?}", iterations, insert_time);
    println!("  {} lookups: {:?}", iterations, lookup_time);
    println!("  Avg insert: {:?}", insert_time / iterations);
    println!("  Avg lookup: {:?}", lookup_time / iterations);
    
    assert!(insert_time < Duration::from_secs(1), "Insertions too slow");
    assert!(lookup_time < Duration::from_secs(1), "Lookups too slow");
    
    Ok(())
}