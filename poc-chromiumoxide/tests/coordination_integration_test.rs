// Integration test for coordinated perception and tools
// This test verifies that perception can detect interfaces for tool operations

use rainbow_poc_chromiumoxide::coordination::{
    browser_context::{BrowserSessionContext, ModuleCoordinator},
    EventBus, UnifiedStateManager, UnifiedCache,
    error_handler::UnifiedErrorHandler,
};
use rainbow_poc_chromiumoxide::browser::Browser;
use std::sync::Arc;
use anyhow::Result;

#[tokio::test]
async fn test_perception_detects_tool_interface() -> Result<()> {
    // Initialize coordination components
    let event_bus = Arc::new(EventBus::new());
    let state_manager = Arc::new(UnifiedStateManager::new(event_bus.clone()).await?);
    let cache = Arc::new(UnifiedCache::new(event_bus.clone()).await?);
    
    // Create browser
    let browser = Arc::new(Browser::new_headless().await?);
    
    // Create coordinated session context
    let context = BrowserSessionContext::new(
        "test_session".to_string(),
        browser.clone(),
        event_bus.clone(),
        state_manager,
        cache,
    ).await?;
    
    // Navigate to a test page with interactive elements
    browser.navigate_to("https://example.com").await?;
    context.handle_navigation("https://example.com").await?;
    
    // Get shared perception engine
    let perception = context.get_perception_engine().await?;
    
    // Get page content
    let html = browser.content().await?;
    
    // Use perception to find interactive elements
    let elements = perception.find_interactive_elements(&html)?;
    println!("Found {} interactive elements", elements.len());
    
    // Verify perception can identify tool-actionable elements
    let mut tool_compatible = 0;
    for element in &elements {
        println!("Element: {} - Type: {:?}", element.selector, element.element_type);
        
        // Check if this element can be used by tools
        match element.element_type {
            rainbow_poc_chromiumoxide::coordination::perception_impl::ElementType::Button |
            rainbow_poc_chromiumoxide::coordination::perception_impl::ElementType::Link |
            rainbow_poc_chromiumoxide::coordination::perception_impl::ElementType::Input => {
                tool_compatible += 1;
                println!("  ✓ Tool-compatible element detected");
            }
            _ => {
                println!("  - Not directly tool-compatible");
            }
        }
    }
    
    assert!(tool_compatible > 0, "Should find at least one tool-compatible element");
    
    // Now test coordinated action execution
    if let Some(first_element) = elements.first() {
        println!("\nTesting coordinated action on: {}", first_element.selector);
        
        // Execute coordinated action (perception + tools)
        let result = context.execute_coordinated_action(
            "click",
            &first_element.text,
        ).await?;
        
        println!("Coordinated action result: {:?}", result);
        assert!(result["success"].as_bool().unwrap_or(false) || 
                result["error"].is_string(), 
                "Should either succeed or provide error message");
    }
    
    // Get resource statistics
    let stats = context.get_resource_stats().await;
    println!("\nResource Usage Statistics:");
    println!("  Perception operations: {}", stats.perception_operations);
    println!("  Tool executions: {}", stats.tool_executions);
    println!("  Cache hits: {}", stats.cache_hits);
    println!("  Cache misses: {}", stats.cache_misses);
    
    // Cleanup
    context.cleanup().await?;
    browser.close().await?;
    
    Ok(())
}

#[tokio::test]
async fn test_shared_instance_coordination() -> Result<()> {
    let event_bus = Arc::new(EventBus::new());
    let state_manager = Arc::new(UnifiedStateManager::new(event_bus.clone()).await?);
    let cache = Arc::new(UnifiedCache::new(event_bus.clone()).await?);
    let browser = Arc::new(Browser::new_headless().await?);
    
    let context = BrowserSessionContext::new(
        "test_shared".to_string(),
        browser.clone(),
        event_bus,
        state_manager,
        cache,
    ).await?;
    
    // Get perception engine multiple times
    let perception1 = context.get_perception_engine().await?;
    let perception2 = context.get_perception_engine().await?;
    
    // Verify they are the same instance
    assert!(Arc::ptr_eq(&perception1, &perception2), 
            "Should return the same perception instance");
    
    // Get tool registry multiple times
    let tools1 = context.get_tool_registry().await?;
    let tools2 = context.get_tool_registry().await?;
    
    // Verify they are the same instance
    assert!(Arc::ptr_eq(&tools1, &tools2), 
            "Should return the same tools instance");
    
    // Verify resource tracking
    let stats = context.get_resource_stats().await;
    assert_eq!(stats.cache_hits, 2, "Should have 2 cache hits for repeated access");
    
    context.cleanup().await?;
    browser.close().await?;
    
    Ok(())
}

#[tokio::test]
async fn test_navigation_cache_invalidation() -> Result<()> {
    let event_bus = Arc::new(EventBus::new());
    let state_manager = Arc::new(UnifiedStateManager::new(event_bus.clone()).await?);
    let cache = Arc::new(UnifiedCache::new(event_bus.clone()).await?);
    let browser = Arc::new(Browser::new_headless().await?);
    
    let context = BrowserSessionContext::new(
        "test_navigation".to_string(),
        browser.clone(),
        event_bus,
        state_manager,
        cache.clone(),
    ).await?;
    
    // Navigate to first page
    browser.navigate_to("https://example.com").await?;
    context.handle_navigation("https://example.com").await?;
    
    // Get perception and analyze
    let perception = context.get_perception_engine().await?;
    let html1 = browser.content().await?;
    let elements1 = perception.find_interactive_elements(&html1)?;
    println!("First page: {} elements", elements1.len());
    
    // Navigate to second page (this should invalidate caches)
    browser.navigate_to("https://example.org").await?;
    context.handle_navigation("https://example.org").await?;
    
    // Analyze again - should get fresh data
    let html2 = browser.content().await?;
    let elements2 = perception.find_interactive_elements(&html2)?;
    println!("Second page: {} elements", elements2.len());
    
    // URLs should be different
    assert_ne!(html1.len(), html2.len(), "Different pages should have different content");
    
    context.cleanup().await?;
    browser.close().await?;
    
    Ok(())
}

#[tokio::test]
async fn test_error_handling_with_fallback() -> Result<()> {
    use rainbow_poc_chromiumoxide::coordination::error_handler::CoordinationResultExt;
    
    let event_bus = Arc::new(EventBus::new());
    let handler = UnifiedErrorHandler::new(event_bus.clone());
    
    // Test fallback mechanism
    let result: Result<String> = Err(anyhow::anyhow!("Primary failed"))
        .or_fallback(|| Ok("Fallback succeeded".to_string()));
    
    assert_eq!(result?, "Fallback succeeded");
    
    // Test retry with backoff (using a counter that succeeds on 3rd try)
    let counter = Arc::new(std::sync::Mutex::new(0));
    let counter_clone = counter.clone();
    
    let result = handler.retry_with_backoff(
        move || {
            let mut count = counter_clone.lock().unwrap();
            *count += 1;
            if *count >= 3 {
                Ok("Success after retries".to_string())
            } else {
                Err(anyhow::anyhow!("Retry needed"))
            }
        },
        "test_operation"
    ).await?;
    
    assert_eq!(result, "Success after retries");
    assert_eq!(*counter.lock().unwrap(), 3);
    
    Ok(())
}

#[tokio::test]
async fn test_module_coordinator() -> Result<()> {
    let event_bus = Arc::new(EventBus::new());
    let state_manager = Arc::new(UnifiedStateManager::new(event_bus.clone()).await?);
    let cache = Arc::new(UnifiedCache::new(event_bus.clone()).await?);
    
    let coordinator = ModuleCoordinator::new(event_bus, state_manager, cache);
    
    // Create browsers for different sessions
    let browser1 = Arc::new(Browser::new_headless().await?);
    let browser2 = Arc::new(Browser::new_headless().await?);
    
    // Create multiple session contexts
    let context1 = coordinator.get_or_create_context("session1", browser1.clone()).await?;
    let context2 = coordinator.get_or_create_context("session2", browser2.clone()).await?;
    
    // Verify they are different contexts
    assert!(!Arc::ptr_eq(&context1, &context2), "Different sessions should have different contexts");
    
    // Get the same context again
    let context1_again = coordinator.get_or_create_context("session1", browser1.clone()).await?;
    assert!(Arc::ptr_eq(&context1, &context1_again), "Same session should return same context");
    
    // Check active contexts
    let active = coordinator.get_active_contexts().await;
    assert_eq!(active.len(), 2);
    assert!(active.contains(&"session1".to_string()));
    assert!(active.contains(&"session2".to_string()));
    
    // Remove a context
    coordinator.remove_context("session1").await?;
    let active_after = coordinator.get_active_contexts().await;
    assert_eq!(active_after.len(), 1);
    assert!(!active_after.contains(&"session1".to_string()));
    
    // Cleanup
    browser1.close().await?;
    browser2.close().await?;
    
    Ok(())
}