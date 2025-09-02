// Minimal test binary for perception module
use anyhow::Result;
use rainbow_poc::perception_mvp::{
    browser_connection::{BrowserConnection, BrowserConfig},
    lightning_real::RealLightningPerception,
};
use tracing::{info, error};
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    info!("Starting Perception Module Test");
    
    // Create browser connection
    let config = BrowserConfig::default();
    info!("Connecting to ChromeDriver at {}", config.chromedriver_url);
    
    let browser = match BrowserConnection::new(config).await {
        Ok(b) => {
            info!("✅ Browser connected successfully");
            b
        }
        Err(e) => {
            error!("❌ Failed to connect to browser: {}", e);
            error!("Make sure ChromeDriver is running on port 9515");
            return Err(e);
        }
    };
    
    // Test navigation
    info!("Navigating to example.com...");
    browser.navigate("https://example.com").await?;
    info!("✅ Navigation successful");
    
    // Test Lightning perception
    info!("Testing Lightning perception (<50ms)...");
    let perception = RealLightningPerception::new();
    
    let start = Instant::now();
    let result = perception.scan_page(&browser).await?;
    let elapsed = start.elapsed().as_millis();
    
    info!("✅ Lightning perception completed in {}ms", elapsed);
    
    // Display results
    println!("\n=== Lightning Perception Results ===");
    println!("Scan Time: {}ms", result.scan_time_ms);
    println!("Page Status: {:?}", result.page_status);
    println!("Key Elements Found: {}", result.key_elements.len());
    
    for (i, element) in result.key_elements.iter().enumerate() {
        println!("  {}. {:?} - {} (importance: {:.2})", 
            i + 1, 
            element.element_type,
            element.text.chars().take(50).collect::<String>(),
            element.importance
        );
    }
    
    if !result.urgent_signals.is_empty() {
        println!("Urgent Signals: {}", result.urgent_signals.len());
        for signal in &result.urgent_signals {
            println!("  - {:?}: {}", signal.signal_type, signal.message);
        }
    }
    
    // Test on a more complex page
    info!("\nTesting on google.com...");
    browser.navigate("https://google.com").await?;
    
    let start = Instant::now();
    let result = perception.scan_page(&browser).await?;
    let elapsed = start.elapsed().as_millis();
    
    println!("\n=== Google.com Results ===");
    println!("Scan Time: {}ms", elapsed);
    println!("Key Elements Found: {}", result.key_elements.len());
    
    for element in result.key_elements.iter().take(5) {
        println!("  - {:?}: {} [{}]", 
            element.element_type,
            element.text.chars().take(30).collect::<String>(),
            element.selector
        );
    }
    
    // Clean up
    browser.close().await?;
    info!("✅ Test completed successfully!");
    
    Ok(())
}