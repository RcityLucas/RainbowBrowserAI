// 彩虹城浏览器V8.0 - 简单演示程序

use anyhow::Result;
use rainbow_browser_ai::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!(r#"
╔══════════════════════════════════════════════════════════╗
║           🌈 彩虹城浏览器 8.0 - 简单演示                ║
║                                                          ║
║  六大引擎架构 | AI智能驱动 | 数字生命进化              ║
╚══════════════════════════════════════════════════════════╝
    "#);

    // 创建AI生命体
    println!("🧬 正在创建AI生命体...");
    let browser = RainbowBrowserV8::new().await?;
    println!("✅ AI生命体创建成功!");

    // 获取系统状态
    println!("\n📊 检查系统状态...");
    let status = browser.get_system_status().await?;
    println!("系统健康状态: {:?}", status.health);

    // 演示1: 处理用户请求
    println!("\n🎯 演示 1: 处理简单请求");
    let response = browser.process_request("搜索人工智能相关信息").await?;
    println!("AI响应: {}", response);

    // 演示2: 系统优化
    println!("\n⚙️ 演示 2: 系统优化");
    browser.optimize().await?;
    println!("✅ 系统优化完成");

    // 演示3: 再次检查状态
    println!("\n📈 演示 3: 优化后状态检查");
    let status_after = browser.get_system_status().await?;
    println!("优化后健康状态: {:?}", status_after.health);

    println!("\n🎉 演示完成! 彩虹城浏览器V8.0运行正常");
    
    Ok(())
}