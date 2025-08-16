use rainbow_poc::{
    SimpleBrowser, BrowserPool, WorkflowEngine, Workflow, LLMService,
    MetricsCollector, Config, CostTracker, PluginManager, init_plugin_system,
    create_router, ApiState, SecurityMiddleware,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use std::time::Duration;
use tempfile::TempDir;
use std::fs;
use std::path::Path;

#[tokio::test]
async fn test_browser_navigation() {
    let browser = SimpleBrowser::new().await.expect("Failed to create browser");
    
    // Navigate to a test page
    browser.navigate_to("https://www.example.com")
        .await
        .expect("Failed to navigate");
    
    // Get title
    let title = browser.get_title()
        .await
        .expect("Failed to get title");
    
    assert!(title.contains("Example"));
}

#[tokio::test]
async fn test_browser_pool_lifecycle() {
    let pool = BrowserPool::new();
    
    // Get a browser from pool
    let browser_handle = pool.get_browser()
        .await
        .expect("Failed to get browser from pool");
    
    // Use the browser
    browser_handle.browser.navigate_to("https://www.rust-lang.org")
        .await
        .expect("Failed to navigate");
    
    // Browser should be returned to pool when dropped
    drop(browser_handle);
    
    // Verify pool metrics
    let metrics = pool.get_metrics().await;
    assert_eq!(metrics.total_created, 1);
}

#[tokio::test]
async fn test_workflow_execution() {
    let workflow_yaml = r#"
name: test-workflow
description: Test workflow for integration testing
steps:
  - name: navigate
    action:
      type: navigate
      url: https://www.example.com
  - name: wait
    action:
      type: wait
      wait_for: time
      seconds: 1
"#;

    let workflow = Workflow::from_yaml(workflow_yaml)
        .expect("Failed to parse workflow");
    
    let mut engine = WorkflowEngine::new_simple();
    let result = engine.execute(&workflow).await
        .expect("Failed to execute workflow");
    
    assert!(result.success);
    assert_eq!(result.steps_executed, 2);
    assert_eq!(result.steps_failed, 0);
}

#[tokio::test]
async fn test_workflow_with_variables() {
    let workflow_yaml = r#"
name: variable-workflow
variables:
  search_term: "Rust programming"
steps:
  - name: navigate
    action:
      type: navigate
      url: "https://www.google.com/search?q={{search_term}}"
"#;

    let workflow = Workflow::from_yaml(workflow_yaml)
        .expect("Failed to parse workflow");
    
    let mut engine = WorkflowEngine::new_simple();
    let result = engine.execute(&workflow).await
        .expect("Failed to execute workflow");
    
    assert!(result.success);
    assert!(result.variables.contains_key("search_term"));
}

#[tokio::test]
async fn test_cost_tracking() {
    let mut cost_tracker = CostTracker::new(100.0);
    
    // Record operations
    cost_tracker.record_operation(
        "browser".to_string(),
        "Navigate to test page".to_string(),
        0.001,
        true,
    ).expect("Failed to record operation");
    
    cost_tracker.record_operation(
        "llm".to_string(),
        "Parse command".to_string(),
        0.01,
        true,
    ).expect("Failed to record LLM operation");
    
    // Check totals
    let daily_total = cost_tracker.get_daily_total();
    assert!(daily_total > 0.0);
    assert!(daily_total < cost_tracker.daily_budget);
    
    // Generate report
    let report = cost_tracker.generate_daily_report();
    assert!(report.contains("Daily Budget"));
    assert!(report.contains("browser"));
    assert!(report.contains("llm"));
}

#[tokio::test]
async fn test_metrics_collection() {
    let metrics = MetricsCollector::new();
    
    // Record some operations
    metrics.record_operation(
        Duration::from_millis(100),
        true,
        0.001,
    ).await;
    
    metrics.record_operation(
        Duration::from_millis(200),
        false,
        0.002,
    ).await;
    
    // Get metrics
    let current_metrics = metrics.get_metrics().await;
    assert_eq!(current_metrics.operations_total, 2);
    assert_eq!(current_metrics.operations_success, 1);
    assert_eq!(current_metrics.operations_failed, 1);
    
    // Get summary
    let summary = metrics.get_summary().await;
    assert!(summary.avg_response_time_ms > 0.0);
    assert!(summary.total_cost > 0.0);
}

#[tokio::test]
async fn test_plugin_system_lifecycle() {
    // Create temporary plugin directory
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let plugin_dir = temp_dir.path().join("plugins");
    fs::create_dir(&plugin_dir).expect("Failed to create plugin dir");
    
    // Create a test plugin manifest
    let manifest = r#"
[plugin]
name = "test-plugin"
version = "1.0.0"
description = "Test plugin for integration testing"
author = "Test Author"
license = "MIT"
type = "Action"

[capabilities]
permissions = ["BrowserControl"]
"#;
    
    let plugin_path = plugin_dir.join("test-plugin");
    fs::create_dir(&plugin_path).expect("Failed to create plugin directory");
    fs::write(plugin_path.join("plugin.toml"), manifest)
        .expect("Failed to write manifest");
    
    // Initialize plugin manager
    let plugin_manager = PluginManager::new().await
        .expect("Failed to create plugin manager");
    
    // Discover plugins
    let discovered = plugin_manager.discover_plugins(&plugin_dir).await
        .expect("Failed to discover plugins");
    
    assert_eq!(discovered.len(), 1);
    assert_eq!(discovered[0].0, "test-plugin");
    
    // Get plugin metrics
    let metrics = plugin_manager.get_metrics().await;
    assert_eq!(metrics.total_plugins, 1);
    assert_eq!(metrics.discovered_plugins, 1);
}

#[tokio::test]
async fn test_api_health_endpoint() {
    // Set up API state
    let config = Config::default();
    let browser_pool = Arc::new(BrowserPool::new());
    let llm_service = Arc::new(LLMService::new(String::new()));
    let metrics = Arc::new(MetricsCollector::new());
    let security = Arc::new(SecurityMiddleware::new(Default::default()));
    let cost_tracker = Arc::new(RwLock::new(CostTracker::new(100.0)));
    let plugin_manager = Arc::new(RwLock::new(
        PluginManager::new().await.expect("Failed to create plugin manager")
    ));
    
    let state = ApiState {
        browser_pool,
        llm_service,
        metrics,
        security,
        config: Arc::new(config),
        cost_tracker,
        sessions: Arc::new(RwLock::new(HashMap::new())),
        plugin_manager,
    };
    
    // Create router
    let app = create_router(state);
    
    // Test health endpoint
    let response = app
        .clone()
        .oneshot(
            http::Request::builder()
                .uri("/health")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn test_workflow_with_conditionals() {
    let workflow_yaml = r#"
name: conditional-workflow
variables:
  should_proceed: true
steps:
  - name: check-condition
    action:
      type: conditional
      if:
        check: variable_equals
        var: should_proceed
        value: true
      then:
        - name: success-step
          action:
            type: wait
            wait_for: time
            seconds: 1
      else:
        - name: failure-step
          action:
            type: wait
            wait_for: time
            seconds: 2
"#;

    let workflow = Workflow::from_yaml(workflow_yaml)
        .expect("Failed to parse workflow");
    
    let mut engine = WorkflowEngine::new_simple();
    let result = engine.execute(&workflow).await
        .expect("Failed to execute workflow");
    
    assert!(result.success);
    // Should execute the 'then' branch
    assert_eq!(result.steps_executed, 2); // conditional + success-step
}

#[tokio::test]
async fn test_workflow_error_handling() {
    let workflow_yaml = r#"
name: error-workflow
on_error: continue
steps:
  - name: failing-step
    action:
      type: navigate
      url: "http://invalid-url-that-should-fail"
    on_error: continue
  - name: success-step
    action:
      type: wait
      wait_for: time
      seconds: 1
"#;

    let workflow = Workflow::from_yaml(workflow_yaml)
        .expect("Failed to parse workflow");
    
    let mut engine = WorkflowEngine::new_simple();
    let result = engine.execute(&workflow).await
        .expect("Failed to execute workflow");
    
    // Should continue despite error
    assert_eq!(result.steps_executed, 2);
    assert!(result.steps_failed > 0);
}

#[tokio::test]
async fn test_session_management() {
    // Create two browsers in different sessions
    let browser1 = SimpleBrowser::new().await
        .expect("Failed to create browser 1");
    let browser2 = SimpleBrowser::new().await
        .expect("Failed to create browser 2");
    
    // Navigate to different pages
    browser1.navigate_to("https://www.rust-lang.org")
        .await
        .expect("Failed to navigate browser 1");
    
    browser2.navigate_to("https://www.github.com")
        .await
        .expect("Failed to navigate browser 2");
    
    // Get URLs to verify they're different
    let url1 = browser1.get_url().await
        .expect("Failed to get URL 1");
    let url2 = browser2.get_url().await
        .expect("Failed to get URL 2");
    
    assert_ne!(url1, url2);
    assert!(url1.contains("rust-lang"));
    assert!(url2.contains("github"));
}

#[tokio::test]
async fn test_concurrent_workflow_execution() {
    let workflow_yaml = r#"
name: concurrent-test
steps:
  - name: wait-step
    action:
      type: wait
      wait_for: time
      seconds: 1
"#;

    let workflow = Workflow::from_yaml(workflow_yaml)
        .expect("Failed to parse workflow");
    
    // Run multiple workflows concurrently
    let mut handles = vec![];
    
    for i in 0..5 {
        let workflow_clone = workflow.clone();
        let handle = tokio::spawn(async move {
            let mut engine = WorkflowEngine::new_simple();
            let result = engine.execute(&workflow_clone).await
                .expect(&format!("Failed to execute workflow {}", i));
            result.success
        });
        handles.push(handle);
    }
    
    // Wait for all workflows to complete
    for handle in handles {
        let success = handle.await.expect("Task panicked");
        assert!(success);
    }
}

#[tokio::test]
async fn test_plugin_configuration() {
    let plugin_manager = PluginManager::new().await
        .expect("Failed to create plugin manager");
    
    // Create a test configuration
    let config = serde_json::json!({
        "setting1": "value1",
        "setting2": 42,
        "setting3": true
    });
    
    // Test configuration serialization
    let config_str = config.to_string();
    assert!(config_str.contains("setting1"));
    assert!(config_str.contains("value1"));
    assert!(config_str.contains("42"));
}

#[tokio::test]
async fn test_metrics_export() {
    let metrics = MetricsCollector::new();
    
    // Record various operations
    for i in 0..10 {
        metrics.record_operation(
            Duration::from_millis(100 + i * 10),
            i % 2 == 0,
            0.001 * i as f64,
        ).await;
    }
    
    // Export metrics in Prometheus format
    let prometheus_metrics = metrics.export_prometheus().await;
    
    // Verify metrics format
    assert!(prometheus_metrics.contains("# HELP"));
    assert!(prometheus_metrics.contains("# TYPE"));
    assert!(prometheus_metrics.contains("rainbow_operations_total"));
    assert!(prometheus_metrics.contains("rainbow_operations_success"));
    assert!(prometheus_metrics.contains("rainbow_response_time_ms"));
}

#[tokio::test]
async fn test_cache_functionality() {
    use rainbow_poc::{LLMCache, WorkflowCache};
    
    // Test LLM cache
    let llm_cache = LLMCache::new(10, Duration::from_secs(60));
    
    llm_cache.set(
        "test_command",
        serde_json::json!({"action": "navigate", "url": "test.com"}),
    ).await;
    
    let cached = llm_cache.get("test_command").await;
    assert!(cached.is_some());
    assert_eq!(cached.unwrap()["action"], "navigate");
    
    // Test workflow cache
    let workflow_cache = WorkflowCache::new(5, Duration::from_secs(60));
    
    let workflow = Workflow::from_yaml(r#"
name: cached-workflow
steps:
  - name: test
    action:
      type: wait
      wait_for: time
      seconds: 1
"#).expect("Failed to parse workflow");
    
    workflow_cache.set("test_workflow", workflow.clone()).await;
    
    let cached_workflow = workflow_cache.get("test_workflow").await;
    assert!(cached_workflow.is_some());
    assert_eq!(cached_workflow.unwrap().name, "cached-workflow");
}