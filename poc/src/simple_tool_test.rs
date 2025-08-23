use crate::browser::SimpleBrowser;
use anyhow::Result;

/// Simple test to validate browser functionality
pub async fn test_basic_browser_functionality() -> Result<()> {
    println!("🧪 Testing basic browser functionality...");
    
    // Create browser instance
    let browser = SimpleBrowser::new().await?;
    println!("✅ Browser created successfully");
    
    // Test navigation
    browser.navigate_to("https://example.com").await?;
    println!("✅ Navigation successful");
    
    // Test element finding
    let elements = browser.find_elements("p").await?;
    println!("✅ Found {} paragraph elements", elements.len());
    
    // Test script execution
    let result = browser.execute_script("return document.title", vec![]).await?;
    println!("✅ Script executed, result type: {:?}", std::any::type_name_of_val(&result));
    
    // Clean up
    browser.close().await?;
    println!("✅ Browser closed successfully");
    
    println!("🎉 All basic browser tests passed!");
    Ok(())
}

/// Test just the navigation tool in isolation
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_navigation_only() {
        let result = test_basic_browser_functionality().await;
        if let Err(e) = result {
            println!("❌ Test failed: {}", e);
            panic!("Browser functionality test failed");
        }
    }
}