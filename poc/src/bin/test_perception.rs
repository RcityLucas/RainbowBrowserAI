// Simple binary to test perception modules directly

use rainbow_poc::{
    SimpleBrowser,
    PerceptionEngineMVP, PerceptionPageType,
    SimplePerception, SimplePageType,
};
use tracing::{info, error};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    info!("Testing perception modules...");
    
    // Create a browser
    info!("Creating browser...");
    let browser = SimpleBrowser::new().await?;
    
    // Navigate to test page
    info!("Navigating to example.com...");
    browser.navigate_to("https://example.com").await?;
    
    // Wait for page to load
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    
    // Get WebDriver
    let driver = browser.get_driver();
    
    // Test MVP perception
    info!("Testing MVP perception...");
    let mvp_perception = PerceptionEngineMVP::new(driver.clone());
    
    match mvp_perception.classify_page().await {
        Ok(page_type) => {
            info!("MVP: Page classified as {:?}", page_type);
        }
        Err(e) => {
            error!("MVP: Failed to classify page: {}", e);
        }
    }
    
    match mvp_perception.find_element("title").await {
        Ok(element) => {
            info!("MVP: Found title element: {}", element.text);
        }
        Err(e) => {
            error!("MVP: Failed to find title: {}", e);
        }
    }
    
    // Test simple perception
    info!("Testing simple perception...");
    let simple_perception = SimplePerception::new(driver.clone());
    
    match simple_perception.classify_page().await {
        Ok(page_type) => {
            info!("Simple: Page classified as {:?}", page_type);
        }
        Err(e) => {
            error!("Simple: Failed to classify page: {}", e);
        }
    }
    
    match simple_perception.find_element("title").await {
        Ok(element) => {
            info!("Simple: Found title element: {}", element.text);
        }
        Err(e) => {
            error!("Simple: Failed to find title: {}", e);
        }
    }
    
    info!("Perception module test complete!");
    
    Ok(())
}