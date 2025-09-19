// Simple test of the tools system
use anyhow::Result;
use poc_chromiumoxide::{
    browser::{Browser, BrowserConfig, ScreenshotOptions},
    tools::{
        extraction::ExtractTextTool, memory::ScreenshotTool, navigation::NavigateTool,
        registry::ToolRegistry, traits::DynamicToolWrapper,
    },
};
use serde_json::json;
use std::sync::Arc;
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("🚀 Testing RainbowBrowserAI Tools System");

    // Create browser
    let config = BrowserConfig::builder().build().unwrap();
    let browser = Arc::new(Browser::new(config).await?);

    // Create tool registry
    let registry = ToolRegistry::new(browser.clone());

    // Register tools
    registry
        .register(Arc::new(DynamicToolWrapper::new(NavigateTool::new(
            browser.clone(),
        ))))
        .await?;
    registry
        .register(Arc::new(DynamicToolWrapper::new(ExtractTextTool::new(
            browser.clone(),
        ))))
        .await?;
    registry
        .register(Arc::new(DynamicToolWrapper::new(ScreenshotTool::new(
            browser.clone(),
        ))))
        .await?;

    println!("✅ Tools registered successfully");

    // Test navigation
    println!("\n📍 Testing navigation...");
    let result = registry
        .execute(
            "navigate_to_url",
            json!({
                "url": "https://example.com"
            }),
        )
        .await?;
    println!("✅ Navigation result: success = {}", result.success);

    // Test text extraction
    println!("\n📝 Testing text extraction...");
    let result = registry
        .execute(
            "extract_text",
            json!({
                "selector": "h1",
                "trim": true
            }),
        )
        .await?;
    println!("✅ Text extraction result: success = {}", result.success);
    if let Some(output) = result.output.as_object() {
        if let Some(text) = output.get("text") {
            println!("   Extracted text: {}", text);
        }
    }

    // Test screenshot
    println!("\n📸 Testing screenshot...");
    let result = registry
        .execute(
            "screenshot",
            json!({
                "full_page": false,
                "quality": 90
            }),
        )
        .await?;
    println!("✅ Screenshot result: success = {}", result.success);
    if let Some(output) = result.output.as_object() {
        if let Some(size) = output.get("size_bytes") {
            println!("   Screenshot size: {} bytes", size);
        }
    }

    // Show statistics
    println!("\n📊 Tool execution statistics:");
    let stats = registry.get_statistics().await;
    for (tool_name, stat) in stats {
        println!(
            "   {} - {} executions, {:.1}% success rate",
            tool_name,
            stat.total_executions,
            stat.success_rate * 100.0
        );
    }

    println!("\n🎉 All tools tests completed successfully!");
    println!("🔧 Tools system is operational and ready for integration!");

    Ok(())
}
