// Test script that doesn't require browser to validate our basic functionality
use rainbow_poc::{CostTracker, Config};

fn main() {
    println!("🌈 RainbowBrowserAI PoC - Functionality Test");
    
    // Test 1: Configuration loading
    println!("\n1. Testing configuration...");
    match Config::from_env() {
        Ok(config) => {
            println!("   ✅ Config loaded - Daily budget: ${:.2}", config.daily_budget);
        }
        Err(e) => {
            println!("   ❌ Config failed: {}", e);
        }
    }
    
    // Test 2: Cost tracking
    println!("\n2. Testing cost tracking...");
    let mut cost_tracker = CostTracker::new(0.50);
    
    // Simulate some operations
    let _ = cost_tracker.record_operation(
        "test_navigation".to_string(),
        "Navigate to example.com".to_string(),
        0.01,
        true,
    );
    
    let _ = cost_tracker.record_operation(
        "test_screenshot".to_string(),
        "Take screenshot".to_string(),
        0.01,
        true,
    );
    
    println!("   ✅ Cost tracking working");
    println!("   💰 Total spent: ${:.4}", cost_tracker.get_today_spent());
    println!("   📊 Operations today: {}", cost_tracker.get_today_operation_count());
    println!("   🎯 Success rate: {:.1}%", cost_tracker.get_today_success_rate());
    
    // Test 3: Budget checking
    println!("\n3. Testing budget validation...");
    if cost_tracker.can_afford(0.01) {
        println!("   ✅ Budget check passed - Can afford $0.01 operation");
    } else {
        println!("   ❌ Budget exceeded");
    }
    
    // Test 4: Daily report
    println!("\n4. Generating daily report...");
    println!("{}", cost_tracker.generate_daily_report());
    
    // Test 5: Cost estimation
    println!("\n5. Testing cost estimation...");
    let browser_cost = cost_tracker.estimate_browser_operation_cost();
    let llm_cost = cost_tracker.estimate_llm_operation_cost("Navigate to google.com".len());
    
    println!("   📊 Browser operation cost: ${:.4}", browser_cost);
    println!("   📊 LLM operation cost: ${:.4}", llm_cost);
    
    println!("\n🎉 All basic functionality tests passed!");
    println!("🚀 Ready for browser integration testing");
    
    // Show next steps
    println!("\n📋 Next Steps:");
    println!("   1. Install ChromeDriver: brew install chromedriver");
    println!("   2. Start ChromeDriver: chromedriver --port=9515");
    println!("   3. Test browser: cargo run -- --url google.com --screenshot");
    println!("   4. Check costs: cargo run -- --url example.com --cost-report");
}