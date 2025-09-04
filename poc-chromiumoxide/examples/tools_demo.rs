// Comprehensive demonstration of all browser automation tools
use anyhow::Result;
use poc_chromiumoxide::{
    browser::{Browser, BrowserConfig},
    tools::{
        registry::ToolRegistry,
        traits::{DynamicToolWrapper, ToolCategory},
        // Navigation tools
        navigation::{NavigateTool, ScrollTool, RefreshTool, GoBackTool, GoForwardTool},
        // Interaction tools  
        interaction::{ClickTool, TypeTextTool, SelectOptionTool, HoverTool, FocusTool},
        // Data extraction tools
        extraction::{ExtractTextTool, ExtractLinksTool, ExtractDataTool, ExtractTableTool, ExtractFormTool},
        // Synchronization tools
        synchronization::{WaitForElementTool, WaitForConditionTool},
        // Memory tools
        memory::{ScreenshotTool, SessionMemoryTool, GetElementInfoTool, HistoryTrackerTool, PersistentCacheTool},
    },
};
use serde_json::json;
use std::sync::Arc;
use tracing::{info, error};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    
    info!("Starting comprehensive tools demonstration");
    
    // Create browser instance
    let config = BrowserConfig::default();
    let browser = Arc::new(Browser::new(config).await?);
    
    // Create tool registry
    let registry = ToolRegistry::new(browser.clone());
    
    // Register all tools
    register_all_tools(&registry, browser.clone()).await?;
    
    // List all registered tools
    info!("=== Registered Tools ===");
    let tools = registry.list_tools().await;
    for (name, metadata) in tools {
        info!("  {} - {} [{:?}]", name, metadata.description, metadata.category);
    }
    
    // Run demonstrations
    demo_navigation(&registry).await?;
    demo_interaction(&registry).await?;
    demo_extraction(&registry).await?;
    demo_synchronization(&registry).await?;
    demo_memory(&registry).await?;
    demo_tool_chaining(&registry).await?;
    
    // Display statistics
    display_statistics(&registry).await;
    
    info!("Tools demonstration completed successfully");
    Ok(())
}

async fn register_all_tools(registry: &ToolRegistry, browser: Arc<Browser>) -> Result<()> {
    info!("Registering all tools...");
    
    // Navigation tools
    registry.register(Arc::new(DynamicToolWrapper::new(NavigateTool::new(browser.clone())))).await?;
    registry.register(Arc::new(DynamicToolWrapper::new(ScrollTool::new(browser.clone())))).await?;
    registry.register(Arc::new(DynamicToolWrapper::new(RefreshTool::new(browser.clone())))).await?;
    registry.register(Arc::new(DynamicToolWrapper::new(GoBackTool::new(browser.clone())))).await?;
    registry.register(Arc::new(DynamicToolWrapper::new(GoForwardTool::new(browser.clone())))).await?;
    
    // Interaction tools
    registry.register(Arc::new(DynamicToolWrapper::new(ClickTool::new(browser.clone())))).await?;
    registry.register(Arc::new(DynamicToolWrapper::new(TypeTextTool::new(browser.clone())))).await?;
    registry.register(Arc::new(DynamicToolWrapper::new(SelectOptionTool::new(browser.clone())))).await?;
    registry.register(Arc::new(DynamicToolWrapper::new(HoverTool::new(browser.clone())))).await?;
    registry.register(Arc::new(DynamicToolWrapper::new(FocusTool::new(browser.clone())))).await?;
    
    // Data extraction tools
    registry.register(Arc::new(DynamicToolWrapper::new(ExtractTextTool::new(browser.clone())))).await?;
    registry.register(Arc::new(DynamicToolWrapper::new(ExtractLinksTool::new(browser.clone())))).await?;
    registry.register(Arc::new(DynamicToolWrapper::new(ExtractDataTool::new(browser.clone())))).await?;
    registry.register(Arc::new(DynamicToolWrapper::new(ExtractTableTool::new(browser.clone())))).await?;
    registry.register(Arc::new(DynamicToolWrapper::new(ExtractFormTool::new(browser.clone())))).await?;
    
    // Synchronization tools
    registry.register(Arc::new(DynamicToolWrapper::new(WaitForElementTool::new(browser.clone())))).await?;
    registry.register(Arc::new(DynamicToolWrapper::new(WaitForConditionTool::new(browser.clone())))).await?;
    
    // Memory tools
    registry.register(Arc::new(DynamicToolWrapper::new(ScreenshotTool::new(browser.clone())))).await?;
    registry.register(Arc::new(DynamicToolWrapper::new(SessionMemoryTool::new(browser.clone())))).await?;
    registry.register(Arc::new(DynamicToolWrapper::new(GetElementInfoTool::new(browser.clone())))).await?;
    registry.register(Arc::new(DynamicToolWrapper::new(HistoryTrackerTool::new(browser.clone())))).await?;
    registry.register(Arc::new(DynamicToolWrapper::new(PersistentCacheTool::new(browser.clone())))).await?;
    
    info!("All tools registered successfully");
    Ok(())
}

async fn demo_navigation(registry: &ToolRegistry) -> Result<()> {
    info!("\n=== Navigation Tools Demo ===");
    
    // Navigate to example.com
    let result = registry.execute("navigate_to_url", json!({
        "url": "https://example.com"
    })).await?;
    info!("Navigate result: {:?}", result);
    
    // Scroll down
    let result = registry.execute("scroll_page", json!({
        "y": 500
    })).await?;
    info!("Scroll result: {:?}", result);
    
    // Refresh page
    let result = registry.execute("refresh_page", json!({
        "hard_reload": false
    })).await?;
    info!("Refresh result: {:?}", result);
    
    Ok(())
}

async fn demo_interaction(registry: &ToolRegistry) -> Result<()> {
    info!("\n=== Interaction Tools Demo ===");
    
    // Navigate to a form page
    registry.execute("navigate_to_url", json!({
        "url": "https://www.google.com"
    })).await?;
    
    // Wait for search box
    let result = registry.execute("wait_for_element", json!({
        "selector": "input[name='q']",
        "timeout_ms": 5000
    })).await?;
    info!("Wait for element result: {:?}", result);
    
    // Type in search box
    let result = registry.execute("type_text", json!({
        "selector": "input[name='q']",
        "text": "Rust programming language",
        "clear_first": true
    })).await?;
    info!("Type text result: {:?}", result);
    
    // Focus on the search box
    let result = registry.execute("focus", json!({
        "selector": "input[name='q']"
    })).await?;
    info!("Focus result: {:?}", result);
    
    Ok(())
}

async fn demo_extraction(registry: &ToolRegistry) -> Result<()> {
    info!("\n=== Data Extraction Tools Demo ===");
    
    // Navigate to example.com
    registry.execute("navigate_to_url", json!({
        "url": "https://example.com"
    })).await?;
    
    // Extract text
    let result = registry.execute("extract_text", json!({
        "selector": "h1",
        "trim": true
    })).await?;
    info!("Extract text result: {:?}", result);
    
    // Extract all links
    let result = registry.execute("extract_links", json!({
        "include_external": true,
        "include_internal": true,
        "absolute_urls": true
    })).await?;
    info!("Extract links result: {:?}", result);
    
    // Get element info
    let result = registry.execute("get_element_info", json!({
        "selector": "h1",
        "include_attributes": true,
        "include_position": true
    })).await?;
    info!("Element info result: {:?}", result);
    
    Ok(())
}

async fn demo_synchronization(registry: &ToolRegistry) -> Result<()> {
    info!("\n=== Synchronization Tools Demo ===");
    
    // Navigate to a dynamic page
    registry.execute("navigate_to_url", json!({
        "url": "https://example.com"
    })).await?;
    
    // Wait for specific element
    let result = registry.execute("wait_for_element", json!({
        "selector": "h1",
        "timeout_ms": 5000,
        "visible": true
    })).await?;
    info!("Wait for element result: {:?}", result);
    
    // Wait for custom condition
    let result = registry.execute("wait_for_condition", json!({
        "condition": "document.readyState === 'complete'",
        "timeout_ms": 10000,
        "check_interval_ms": 100
    })).await?;
    info!("Wait for condition result: {:?}", result);
    
    Ok(())
}

async fn demo_memory(registry: &ToolRegistry) -> Result<()> {
    info!("\n=== Memory Tools Demo ===");
    
    // Take screenshot
    let result = registry.execute("screenshot", json!({
        "full_page": false,
        "quality": 90,
        "format": "png"
    })).await?;
    if let Some(output) = result.as_object() {
        if let Some(size) = output.get("size_bytes") {
            info!("Screenshot taken, size: {} bytes", size);
        }
    }
    
    // Store in session memory
    let result = registry.execute("session_memory", json!({
        "action": "Store",
        "key": "test_data",
        "value": {
            "timestamp": 1234567890,
            "message": "Hello from session memory"
        }
    })).await?;
    info!("Session memory store result: {:?}", result);
    
    // Retrieve from session memory
    let result = registry.execute("session_memory", json!({
        "action": "Retrieve",
        "key": "test_data"
    })).await?;
    info!("Session memory retrieve result: {:?}", result);
    
    // Store in persistent cache with TTL
    let result = registry.execute("persistent_cache", json!({
        "action": "Store",
        "key": "cached_data",
        "value": {
            "data": "This will persist"
        },
        "ttl_seconds": 3600
    })).await?;
    info!("Persistent cache store result: {:?}", result);
    
    // Track history
    let result = registry.execute("history_tracker", json!({
        "action": "Get"
    })).await?;
    info!("History tracker result: {:?}", result);
    
    Ok(())
}

async fn demo_tool_chaining(registry: &ToolRegistry) -> Result<()> {
    info!("\n=== Tool Chaining Demo ===");
    
    // Execute a chain of tools for a complete workflow
    let chain = vec![
        ("navigate_to_url", json!({"url": "https://example.com"})),
        ("wait_for_element", json!({"selector": "body", "timeout_ms": 5000})),
        ("scroll_page", json!({"y": 200})),
        ("extract_text", json!({"selector": "h1", "trim": true})),
        ("screenshot", json!({"full_page": false})),
        ("session_memory", json!({
            "action": "Store",
            "key": "page_title",
            "value": "Example Domain"
        })),
    ];
    
    let results = registry.execute_chain(chain).await?;
    info!("Chain execution completed with {} results", results.len());
    
    for (i, result) in results.iter().enumerate() {
        if let Some(obj) = result.as_object() {
            if let Some(success) = obj.get("success") {
                info!("  Step {}: success={}", i + 1, success);
            }
        }
    }
    
    Ok(())
}

async fn display_statistics(registry: &ToolRegistry) {
    info!("\n=== Execution Statistics ===");
    
    let stats = registry.get_statistics().await;
    
    // Group by category
    let mut category_stats: std::collections::HashMap<ToolCategory, Vec<(String, f64, u32)>> = 
        std::collections::HashMap::new();
    
    for (tool_name, stat) in stats {
        let tools = registry.list_tools().await;
        if let Some(metadata) = tools.get(&tool_name) {
            category_stats
                .entry(metadata.category)
                .or_insert_with(Vec::new)
                .push((tool_name.clone(), stat.success_rate, stat.total_executions));
        }
    }
    
    // Display by category
    for (category, tools) in category_stats {
        info!("\n{:?} Tools:", category);
        for (name, success_rate, executions) in tools {
            info!("  {} - {} executions, {:.1}% success", 
                  name, executions, success_rate * 100.0);
        }
    }
    
    // Get execution history
    let history = registry.get_execution_history().await;
    info!("\nRecent Executions (last 5):");
    for entry in history.iter().take(5) {
        info!("  {} - {} - Success: {}", 
              entry.tool_name, 
              entry.timestamp.format("%H:%M:%S"),
              entry.success);
    }
}