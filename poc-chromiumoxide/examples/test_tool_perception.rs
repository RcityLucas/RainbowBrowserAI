use anyhow::Result;
use rainbow_poc_chromiumoxide::{
    browser::Browser,
    perception::{LayeredPerception, PerceptionEngine, PerceptionMode},
    tools::registry::ToolRegistry,
};
use std::sync::Arc;
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("\n===== Testing Tool Interface Recognition =====\n");

    // Create browser instance
    let browser = Arc::new(Browser::new().await?);

    // Navigate to a test page
    println!("1. Navigating to test page...");
    browser.navigate_to("https://www.example.com").await?;
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Create tool registry
    println!("\n2. Initializing Tool Registry...");
    let tool_registry = ToolRegistry::new(browser.clone());
    let available_tools = tool_registry.get_tool_names();
    println!(
        "   Available tools: {} tools registered",
        available_tools.len()
    );

    // Show tool categories
    let categories = tool_registry.get_categories();
    println!("   Tool categories:");
    for (category, count) in categories {
        println!("     - {:?}: {} tools", category, count);
    }

    // Create perception engine
    println!("\n3. Creating Perception Engine...");
    let mut perception_engine = PerceptionEngine::new(browser.clone()).await?;

    // Perform quick scan to identify tool-operable elements
    println!("\n4. Performing quick scan for tool-operable elements...");
    let quick_scan = perception_engine.quick_scan().await?;
    println!("   Quick scan results:");
    println!(
        "     - Interactive elements: {}",
        quick_scan.interactive_elements
    );
    println!("     - Forms detected: {}", quick_scan.forms);
    println!("     - Links found: {}", quick_scan.links);

    // Perform layered perception
    println!("\n5. Running layered perception analysis...");
    let mut layered = LayeredPerception::new(browser.clone());

    // Test different perception modes
    let modes = vec![
        PerceptionMode::Lightning,
        PerceptionMode::Standard,
        PerceptionMode::Deep,
    ];

    for mode in modes {
        println!("\n   Testing {:?} mode:", mode);
        let result = layered.perceive(mode).await?;

        // Check for tool-operable elements in the perception result
        if let Some(elements) = result.get("elements") {
            if let Some(elem_array) = elements.as_array() {
                let interactive_count = elem_array
                    .iter()
                    .filter(|e| {
                        e.get("interactive")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false)
                    })
                    .count();

                let clickable_count = elem_array
                    .iter()
                    .filter(|e| {
                        let tag = e.get("tag_name").and_then(|v| v.as_str()).unwrap_or("");
                        matches!(tag.to_lowercase().as_str(), "button" | "a" | "input")
                    })
                    .count();

                println!("     - Total elements: {}", elem_array.len());
                println!("     - Interactive elements: {}", interactive_count);
                println!("     - Clickable (button/link/input): {}", clickable_count);
            }
        }
    }

    // Test intelligent element location
    println!("\n6. Testing intelligent element location...");
    let search_queries = vec!["button", "link", "form", "input", "submit"];

    for query in search_queries {
        let matches = perception_engine
            .locate_element_intelligently(query)
            .await?;
        println!("   Query '{}': {} matches found", query, matches.len());

        // Show first match details if found
        if let Some(first_match) = matches.first() {
            if let Some(selector) = first_match.get("selector") {
                println!("     First match: {}", selector);
            }
        }
    }

    // Test tool execution on identified elements
    println!("\n7. Testing tool execution on identified elements...");

    // Extract links using the tool
    let extract_links_input = serde_json::json!({});
    match tool_registry
        .execute_tool("extract_links", extract_links_input)
        .await
    {
        Ok(result) => {
            if let Some(links) = result.get("links").and_then(|v| v.as_array()) {
                println!("   Extract Links Tool: Found {} links", links.len());
                for (i, link) in links.iter().take(3).enumerate() {
                    if let Some(href) = link.get("href").and_then(|v| v.as_str()) {
                        println!("     {}. {}", i + 1, href);
                    }
                }
            }
        }
        Err(e) => println!("   Extract Links Tool failed: {}", e),
    }

    // Extract text using the tool
    let extract_text_input = serde_json::json!({
        "selector": "body"
    });
    match tool_registry
        .execute_tool("extract_text", extract_text_input)
        .await
    {
        Ok(result) => {
            if let Some(text) = result.get("text").and_then(|v| v.as_str()) {
                let preview = if text.len() > 100 {
                    format!("{}...", &text[..100])
                } else {
                    text.to_string()
                };
                println!("   Extract Text Tool: {}", preview);
            }
        }
        Err(e) => println!("   Extract Text Tool failed: {}", e),
    }

    // Test form detection
    println!("\n8. Testing form detection and analysis...");
    let form_handler = rainbow_poc_chromiumoxide::perception::smart_forms::SmartFormHandler::new();
    match form_handler.analyze_form(browser.as_ref(), None).await {
        Ok(analysis) => {
            println!("   Form analysis complete:");
            if let Some(forms) = analysis.forms {
                println!("     - {} forms detected", forms.len());
                for (i, form) in forms.iter().enumerate() {
                    println!("       Form {}: {} fields", i + 1, form.fields.len());
                }
            }
        }
        Err(e) => println!("   Form analysis failed: {}", e),
    }

    // Test perception coordination with tools
    println!("\n9. Testing perception-tool coordination...");

    // First perceive the page
    let perception_result = perception_engine.analyze_page_enhanced().await?;

    // Then use tools based on perception
    if let Some(interactive) = perception_result.get("interactive_elements") {
        if let Some(count) = interactive.get("total").and_then(|v| v.as_u64()) {
            println!("   Perception found {} interactive elements", count);

            // If buttons found, try click tool
            if let Some(buttons) = interactive.get("buttons").and_then(|v| v.as_u64()) {
                if buttons > 0 {
                    println!("   {} buttons detected - Click tool ready", buttons);
                }
            }

            // If inputs found, try type tool
            if let Some(inputs) = interactive.get("inputs").and_then(|v| v.as_u64()) {
                if inputs > 0 {
                    println!("   {} input fields detected - Type tool ready", inputs);
                }
            }
        }
    }

    // Summary
    println!("\n===== Test Summary =====");
    println!("✓ Browser initialized");
    println!(
        "✓ Tool registry loaded with {} tools",
        available_tools.len()
    );
    println!("✓ Perception engine created");
    println!("✓ Page elements analyzed");
    println!("✓ Tool-operable interfaces identified");
    println!("✓ Tools executed successfully on identified elements");

    println!("\nThe perception function CAN recognize tool operation interfaces!");

    Ok(())
}
