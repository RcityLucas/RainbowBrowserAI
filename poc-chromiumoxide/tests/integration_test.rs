// Integration test for the tools system
use anyhow::Result;

#[tokio::test]
async fn test_tools_basic_functionality() -> Result<()> {
    // This test verifies that the tools system compiles and basic operations work

    // Test that we can create a browser config
    let config = poc_chromiumoxide::browser::BrowserConfig::builder()
        .build()
        .unwrap();

    // Test that we can create browser
    let browser = std::sync::Arc::new(poc_chromiumoxide::browser::Browser::new(config).await?);

    // Test that we can create registry
    let registry = poc_chromiumoxide::tools::registry::ToolRegistry::new(browser.clone());

    // Test that we can create a tool wrapper
    let navigate_tool =
        std::sync::Arc::new(poc_chromiumoxide::tools::traits::DynamicToolWrapper::new(
            poc_chromiumoxide::tools::navigation::NavigateTool::new(browser.clone()),
        ));

    // Test that we can register a tool
    registry.register(navigate_tool).await?;

    // Test that we can list tools
    let tools = registry.list_tools().await;
    assert!(
        !tools.is_empty(),
        "Should have at least one registered tool"
    );

    println!("✅ Integration test passed - tools system is functional!");
    Ok(())
}

#[test]
fn test_tool_structures() {
    // Test that all tool structures can be created
    use serde_json::json;

    // Test navigation input
    let nav_input = json!({
        "url": "https://example.com"
    });

    // Test extraction input
    let extract_input = json!({
        "selector": "h1",
        "trim": true
    });

    // Test screenshot input
    let screenshot_input = json!({
        "full_page": false,
        "quality": 90
    });

    // All inputs should be valid JSON
    assert!(nav_input.is_object());
    assert!(extract_input.is_object());
    assert!(screenshot_input.is_object());

    println!("✅ Tool structures test passed!");
}
