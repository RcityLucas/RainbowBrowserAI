// Real browser integration test - opens actual Chrome browser
use rainbow_poc::SimpleBrowser;
use rainbow_poc::tools::navigation::{NavigateToUrl, NavigateToUrlParams, NavigationOptions};
use rainbow_poc::tools::types::WaitUntil;
use rainbow_poc::tools::Tool;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
#[ignore] // Use --ignored to run this test
async fn test_real_browser_navigation() -> anyhow::Result<()> {
    println!("🚀 Starting REAL BROWSER test...");
    
    // Create a real SimpleBrowser instance
    println!("📱 Creating real Chrome browser...");
    let simple_browser = SimpleBrowser::new().await?;
    let browser = Arc::new(simple_browser);
    
    println!("✅ Real Chrome browser created successfully!");
    println!("🌐 Browser window should now be visible!");
    
    // Give time to see the browser window
    sleep(Duration::from_secs(2)).await;
    
    // Create navigation tool with real browser
    let nav_tool = NavigateToUrl::new(browser.clone());
    println!("🔧 Navigation tool created with real browser");
    
    // Set up navigation parameters for a real website
    let nav_params = NavigateToUrlParams {
        url: "https://httpbin.org/html".to_string(), // Simple, reliable test site
        options: Some(NavigationOptions {
            wait_until: Some(WaitUntil::DomContentLoaded),
            timeout: Some(10000), // 10 seconds timeout for real browser
            retry: None,
            headers: None,
            referrer: None,
        }),
    };
    
    println!("🔄 Navigating to: {}", nav_params.url);
    println!("👀 WATCH: You should see Chrome browser navigate to the website!");
    
    // Execute real navigation - THIS WILL OPEN CHROME AND NAVIGATE
    let start = std::time::Instant::now();
    let nav_result = nav_tool.execute(nav_params).await;
    let elapsed = start.elapsed();
    
    match nav_result {
        Ok(result) => {
            println!("✅ REAL BROWSER NAVIGATION SUCCESSFUL!");
            println!("   📊 Success: {}", result.success);
            println!("   🌐 Final URL: {}", result.final_url);
            println!("   ⏱️  Total Time: {:?}", elapsed);
            println!("   📈 DNS Lookup: {}ms", result.performance.dns_lookup);
            println!("   📈 TCP Connect: {}ms", result.performance.tcp_connect);
            println!("   📈 DOM Loaded: {}ms", result.performance.dom_loaded);
            println!("   📈 Page Loaded: {}ms", result.performance.page_loaded);
            println!("   📄 Status Code: {}", result.status_code);
            
            if !result.redirects.is_empty() {
                println!("   🔄 Redirects: {:?}", result.redirects);
            }
            
            println!("\n🎯 PROOF OF REAL BROWSER OPERATIONS:");
            println!("   ✅ Real Chrome browser opened");
            println!("   ✅ Real HTTP request made to {}", result.final_url);
            println!("   ✅ Real DOM content loaded");
            println!("   ✅ Real performance metrics collected");
            println!("   ✅ Real browser automation WORKING!");
            
            // Keep browser open for a few seconds to see the result
            println!("\n⏳ Keeping browser open for 5 seconds...");
            sleep(Duration::from_secs(5)).await;
            
            assert!(result.success, "Navigation should succeed");
            assert!(result.final_url.contains("httpbin.org"), "Should navigate to httpbin");
            assert!(result.status_code == 200, "Should get HTTP 200 OK");
            assert!(elapsed.as_secs() < 15, "Should complete within 15 seconds");
            
            println!("✅ All assertions passed!");
        },
        Err(error) => {
            println!("❌ REAL BROWSER NAVIGATION FAILED: {}", error);
            return Err(error);
        }
    }
    
    println!("🏁 Real browser test completed successfully!");
    println!("💡 You should have seen Chrome browser open and navigate to the website!");
    
    Ok(())
}

#[tokio::test]
#[ignore] // Use --ignored to run this test
async fn test_real_browser_with_scroll() -> anyhow::Result<()> {
    println!("🚀 Starting REAL BROWSER scroll test...");
    
    // Create real browser
    let simple_browser = SimpleBrowser::new().await?;
    let browser = Arc::new(simple_browser);
    println!("✅ Real Chrome browser created!");
    
    // Navigate to a page with scrollable content
    let nav_tool = NavigateToUrl::new(browser.clone());
    let nav_params = NavigateToUrlParams {
        url: "https://httpbin.org/html".to_string(),
        options: Some(NavigationOptions {
            wait_until: Some(WaitUntil::DomContentLoaded),
            timeout: Some(10000),
            retry: None,
            headers: None,
            referrer: None,
        }),
    };
    
    println!("🔄 Navigating to test page...");
    let nav_result = nav_tool.execute(nav_params).await?;
    assert!(nav_result.success);
    
    println!("✅ Navigation successful!");
    println!("📜 Now testing scroll operations...");
    
    // Import scroll tool for real browser test
    use rainbow_poc::tools::navigation::{ScrollPage, ScrollPageParams, ScrollOptions};
    use rainbow_poc::tools::types::{ScrollDirection, SimpleScrollDirection};
    
    let scroll_tool = ScrollPage::new(browser.clone());
    
    // Test scroll down
    println!("⬇️  Scrolling down...");
    let scroll_params = ScrollPageParams {
        direction: ScrollDirection::Simple(SimpleScrollDirection::Down),
        amount: Some(500),
        options: Some(ScrollOptions {
            smooth: Some(true),
            duration: Some(1000),
            wait_after: Some(500),
        }),
    };
    
    let scroll_result = scroll_tool.execute(scroll_params).await?;
    assert!(scroll_result.success);
    
    println!("✅ Scroll down successful!");
    println!("   Previous position: ({}, {})", scroll_result.previous_position.x, scroll_result.previous_position.y);
    println!("   Current position: ({}, {})", scroll_result.current_position.x, scroll_result.current_position.y);
    
    // Keep browser open to see the result
    println!("⏳ Keeping browser open for 3 seconds to see scroll result...");
    sleep(Duration::from_secs(3)).await;
    
    println!("🎯 REAL BROWSER SCROLL TEST COMPLETED!");
    println!("💡 You should have seen the page scroll down in the browser!");
    
    Ok(())
}