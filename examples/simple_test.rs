// 简单测试 - 验证基本功能

use rainbow_browser_ai::prelude::*;

#[tokio::main]
async fn main() {
    println!("🌈 彩虹城浏览器 8.0 - 简单测试");
    
    // 测试基本功能
    match test_basic_functions().await {
        Ok(_) => println!("✅ 所有测试通过"),
        Err(e) => println!("❌ 测试失败: {}", e),
    }
}

async fn test_basic_functions() -> anyhow::Result<()> {
    println!("\n1. 创建AI生命体...");
    let browser = RainbowBrowserV8::new().await?;
    println!("   ✅ 创建成功");
    
    println!("\n2. 创建会话...");
    let config = SessionConfig::new("https://example.com");
    let session = browser.unified_kernel.create_session(config).await?;
    println!("   ✅ 会话ID: {}", session.id);
    
    println!("\n3. 获取系统状态...");
    let status = browser.get_system_status().await?;
    println!("   ✅ 系统健康: {}", status.health.status);
    
    println!("\n4. 销毁会话...");
    browser.unified_kernel.destroy_session(&session.id).await?;
    println!("   ✅ 会话已销毁");
    
    Ok(())
}