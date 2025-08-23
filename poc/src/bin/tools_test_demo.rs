//! Tools Test Demo Binary
//! 
//! Demonstrates all browser automation tools developed in RainbowBrowserAI

use anyhow::Result;
use tracing_subscriber;

// Import the tools demo module
include!("../tools_test_demo.rs");

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    println!("🎯 RainbowBrowserAI Browser Tools Demonstration");
    println!("=".repeat(50));
    
    // Run the tools demonstration
    BrowserToolsDemo::demonstrate_tools().await?;
    
    println!("\n📊 Tool Statistics:");
    println!("=".repeat(30));
    let stats = BrowserToolsDemo::get_tool_statistics();
    for (key, value) in stats {
        println!("{}: {}", key, value);
    }
    
    println!("\n✅ Browser Tools Demo Complete!");
    
    Ok(())
}