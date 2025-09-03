// Test perception integration with tools system

use rainbow_poc::{
    SimpleBrowser,
    PerceptionAnalyzer, PerceptionAnalyzerInput, PerceptionEngine, AnalysisType,
    Tool, ToolRegistry,
};
use std::sync::Arc;
use tracing::{info, error};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    info!("Testing perception integration with tools system...");
    
    // Create browser and get WebDriver
    let browser = SimpleBrowser::new().await?;
    let driver = Arc::new(browser.get_driver());
    
    // Navigate to test page
    info!("Navigating to example.com...");
    driver.get("https://example.com").await?;
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    
    // Create perception analyzer tool
    let perception_tool = PerceptionAnalyzer::new(driver.clone())
        .with_name("smart_perception".to_string())
        .with_description("Advanced perception analysis tool".to_string());
    
    // Test 1: Page Classification
    info!("Testing page classification...");
    let classify_input = PerceptionAnalyzerInput {
        url: None, // Use current page
        engine: PerceptionEngine::Combined,
        analysis_type: AnalysisType::ClassifyPage,
        element_description: None,
    };
    
    match perception_tool.execute(classify_input).await {
        Ok(result) => {
            info!("✅ Page Classification: {} (confidence: {:.2}, engine: {})", 
                  result.page_type.unwrap_or("Unknown".to_string()), 
                  result.confidence, 
                  result.engine_used);
        }
        Err(e) => error!("❌ Page classification failed: {}", e),
    }
    
    // Test 2: Element Finding
    info!("Testing element finding...");
    let find_input = PerceptionAnalyzerInput {
        url: None,
        engine: PerceptionEngine::Combined,
        analysis_type: AnalysisType::FindElement,
        element_description: Some("title".to_string()),
    };
    
    match perception_tool.execute(find_input).await {
        Ok(result) => {
            info!("✅ Element Finding: Found {} elements (confidence: {:.2})", 
                  result.elements.len(), result.confidence);
            for (i, element) in result.elements.iter().enumerate() {
                info!("  {}. {} - '{}' (confidence: {:.2})", 
                      i + 1, element.element_type, element.text, element.confidence);
            }
        }
        Err(e) => error!("❌ Element finding failed: {}", e),
    }
    
    // Test 3: Data Extraction
    info!("Testing data extraction...");
    let extract_input = PerceptionAnalyzerInput {
        url: None,
        engine: PerceptionEngine::MVP,
        analysis_type: AnalysisType::ExtractData,
        element_description: None,
    };
    
    match perception_tool.execute(extract_input).await {
        Ok(result) => {
            info!("✅ Data Extraction: Success (engine: {})", result.engine_used);
            if let Some(data) = result.data {
                info!("Extracted data: {}", serde_json::to_string_pretty(&data)?);
            }
        }
        Err(e) => error!("❌ Data extraction failed: {}", e),
    }
    
    // Test 4: Tool Registry Integration
    info!("Testing tool registry integration...");
    let mut registry = ToolRegistry::new();
    registry.register(perception_tool);
    
    let tool_names = registry.list();
    info!("✅ Registered tools: {:?}", tool_names);
    
    if let Some(tool) = registry.get("smart_perception") {
        info!("✅ Tool retrieved from registry: {}", tool.name());
        
        // Test JSON execution
        let json_input = serde_json::json!({
            "engine": "Combined",
            "analysis_type": "ClassifyPage"
        });
        
        match tool.execute_json(json_input).await {
            Ok(json_result) => {
                info!("✅ JSON execution successful");
                info!("Result: {}", serde_json::to_string_pretty(&json_result)?);
            }
            Err(e) => error!("❌ JSON execution failed: {}", e),
        }
    }
    
    info!("🎉 Perception tools integration test complete!");
    
    Ok(())
}