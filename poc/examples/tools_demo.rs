// Demonstration of all 5 browser automation tools
// Run with: cargo run --example tools_demo

use anyhow::Result;
use thirtyfour::{WebDriver, ChromeCapabilities, ChromiumLikeCapabilities, By};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 RainbowBrowserAI - 5 Tools Demonstration");
    println!("=========================================\n");
    
    // Configure Chrome for visible testing
    let mut caps = ChromeCapabilities::new();
    caps.add_arg("--no-headless")?;
    caps.add_arg("--start-maximized")?;
    
    // Connect to ChromeDriver
    println!("📱 Connecting to ChromeDriver on port 9520...");
    let driver = WebDriver::new("http://localhost:9520", caps).await?;
    println!("✅ Browser opened!\n");
    
    // Tool 1: Navigation
    println!("🔧 Tool 1: NavigateToUrl");
    println!("------------------------");
    driver.goto("https://example.com").await?;
    println!("✅ Navigated to example.com");
    sleep(Duration::from_secs(2)).await;
    
    // Tool 2: Scrolling
    println!("\n🔧 Tool 2: ScrollPage");
    println!("---------------------");
    
    // Add content to make page scrollable
    driver.execute(r#"
        document.body.innerHTML += '<div style="height: 2000px;">' +
        '<h2 style="margin-top: 500px;">Middle of page</h2>' +
        '<h2 style="margin-top: 1000px;">Bottom of page</h2></div>';
    "#, vec![]).await?;
    
    driver.execute("window.scrollTo(0, 500)", vec![]).await?;
    println!("✅ Scrolled down 500px");
    sleep(Duration::from_secs(1)).await;
    
    driver.execute("window.scrollTo(0, 0)", vec![]).await?;
    println!("✅ Scrolled back to top");
    
    // Tool 3: Click
    println!("\n🔧 Tool 3: Click");
    println!("----------------");
    
    // Create a clickable button
    driver.goto("data:text/html,
        <html><body>
            <h1>Click Test</h1>
            <button id='testBtn' onclick='this.innerHTML=\"Clicked!\"'>Click Me</button>
        </body></html>
    ").await?;
    sleep(Duration::from_secs(1)).await;
    
    let button = driver.find(By::Id("testBtn")).await?;
    button.click().await?;
    println!("✅ Clicked the button");
    sleep(Duration::from_secs(1)).await;
    
    // Tool 4: Type Text
    println!("\n🔧 Tool 4: TypeText");
    println!("-------------------");
    
    driver.goto("data:text/html,
        <html><body>
            <h1>Input Test</h1>
            <input type='text' id='nameInput' placeholder='Enter your name'>
            <textarea id='messageInput' placeholder='Enter message'></textarea>
        </body></html>
    ").await?;
    sleep(Duration::from_secs(1)).await;
    
    let name_field = driver.find(By::Id("nameInput")).await?;
    name_field.send_keys("Rainbow Browser").await?;
    println!("✅ Typed text into input field");
    
    let message_field = driver.find(By::Id("messageInput")).await?;
    message_field.send_keys("Testing all 5 tools!").await?;
    println!("✅ Typed text into textarea");
    sleep(Duration::from_secs(1)).await;
    
    // Tool 5: Select Option
    println!("\n🔧 Tool 5: SelectOption");
    println!("-----------------------");
    
    driver.goto("data:text/html,
        <html><body>
            <h1>Dropdown Test</h1>
            <select id='colorSelect'>
                <option value=''>Choose color</option>
                <option value='red'>Red</option>
                <option value='green'>Green</option>
                <option value='blue'>Blue</option>
            </select>
        </body></html>
    ").await?;
    sleep(Duration::from_secs(1)).await;
    
    let select = driver.find(By::Id("colorSelect")).await?;
    select.send_keys("Green").await?;
    println!("✅ Selected 'Green' from dropdown");
    sleep(Duration::from_secs(2)).await;
    
    // Summary
    println!("\n🎉 All 5 Tools Successfully Demonstrated!");
    println!("=========================================");
    println!("✅ NavigateToUrl - Navigate to any webpage");
    println!("✅ ScrollPage - Scroll in any direction");  
    println!("✅ Click - Click elements via CSS selectors");
    println!("✅ TypeText - Type into input fields");
    println!("✅ SelectOption - Select from dropdowns\n");
    
    println!("⏳ Keeping browser open for 5 seconds...");
    sleep(Duration::from_secs(5)).await;
    
    driver.quit().await?;
    println!("✨ Demo complete!");
    
    Ok(())
}