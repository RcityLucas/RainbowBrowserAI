//! # 浏览器控制演示 - 实际操作Chrome浏览器
//! 
//! 需要先启动ChromeDriver:
//! ```bash
//! chromedriver --port=9515
//! ```

use std::time::Duration;
use tokio::time::sleep;

// 如果主项目编译有问题，我们直接使用独立的实现
#[path = "../src/browser_control/mod.rs"]
mod browser_control;

use browser_control::{BrowserController, BrowserControlConfig, BrowserType};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌈 彩虹城浏览器 V8.0 - 实际浏览器控制演示");
    println!("=" .repeat(50));
    println!();
    
    // 配置浏览器
    let config = BrowserControlConfig {
        browser_type: BrowserType::Chrome,
        headless: false,  // 显示浏览器窗口
        window_size: (1280, 720),
        timeout: Duration::from_secs(30),
        webdriver_url: "http://localhost:9515".to_string(),
    };
    
    // 创建浏览器控制器
    let mut browser = BrowserController::new(config).await?;
    
    // 启动浏览器
    println!("🚀 正在启动Chrome浏览器...");
    match browser.start().await {
        Ok(_) => println!("✅ 浏览器启动成功！\n"),
        Err(e) => {
            println!("❌ 浏览器启动失败: {}", e);
            println!("\n请确保:");
            println!("1. Chrome浏览器已安装");
            println!("2. ChromeDriver已启动: chromedriver --port=9515");
            println!("3. ChromeDriver版本与Chrome版本匹配");
            return Err(e);
        }
    }
    
    // 演示1: 基础导航
    println!("📍 演示1: 基础导航");
    browser.navigate("https://www.example.com").await?;
    sleep(Duration::from_secs(2)).await;
    
    // 演示2: 搜索引擎操作
    println!("\n🔍 演示2: 搜索引擎操作");
    browser.navigate("https://www.bing.com").await?;
    sleep(Duration::from_secs(2)).await;
    
    // 尝试在搜索框输入
    match browser.input_text("input[name='q']", "彩虹城浏览器 AI").await {
        Ok(_) => {
            println!("✅ 搜索词输入成功");
            sleep(Duration::from_secs(1)).await;
            
            // 尝试点击搜索按钮
            match browser.click("input[type='submit']").await {
                Ok(_) => {
                    println!("✅ 搜索执行成功");
                    sleep(Duration::from_secs(3)).await;
                },
                Err(_) => println!("⚠️ 搜索按钮点击失败，可能需要调整选择器")
            }
        },
        Err(_) => println!("⚠️ 搜索框未找到，可能页面结构已变化")
    }
    
    // 演示3: 多标签页操作
    println!("\n📑 演示3: 多标签页操作");
    browser.new_tab("github").await?;
    browser.navigate("https://github.com").await?;
    sleep(Duration::from_secs(2)).await;
    
    browser.new_tab("docs").await?;
    browser.navigate("https://docs.rs").await?;
    sleep(Duration::from_secs(2)).await;
    
    // 切换回主标签
    browser.switch_tab("main").await?;
    println!("✅ 切换回主标签页");
    sleep(Duration::from_secs(2)).await;
    
    // 演示4: JavaScript执行
    println!("\n⚙️ 演示4: JavaScript执行");
    let result = browser.execute_script("return document.title;").await?;
    println!("页面标题: {:?}", result);
    
    let result = browser.execute_script("return window.location.href;").await?;
    println!("当前URL: {:?}", result);
    
    // 演示5: 页面交互
    println!("\n🎯 演示5: 页面交互");
    browser.navigate("https://www.wikipedia.org").await?;
    sleep(Duration::from_secs(2)).await;
    
    // 尝试在维基百科搜索
    match browser.input_text("#searchInput", "Artificial Intelligence").await {
        Ok(_) => {
            println!("✅ 维基百科搜索词输入成功");
            sleep(Duration::from_secs(1)).await;
            
            // 点击搜索按钮
            match browser.click("button[type='submit']").await {
                Ok(_) => {
                    println!("✅ 搜索执行成功");
                    sleep(Duration::from_secs(3)).await;
                },
                Err(_) => println!("⚠️ 搜索按钮未找到")
            }
        },
        Err(_) => println!("⚠️ 维基百科搜索框未找到")
    }
    
    // 演示6: 截图
    println!("\n📸 演示6: 截取屏幕");
    match browser.screenshot().await {
        Ok(data) => {
            // 保存截图
            let filename = "screenshot.png";
            std::fs::write(filename, data)?;
            println!("✅ 截图已保存到: {}", filename);
        },
        Err(e) => println!("❌ 截图失败: {}", e)
    }
    
    // 等待用户查看
    println!("\n✨ 演示完成！浏览器将在5秒后关闭...");
    sleep(Duration::from_secs(5)).await;
    
    // 关闭浏览器
    browser.quit().await?;
    
    println!("\n🎉 彩虹城浏览器演示结束！");
    println!("🌈 让AI真正'活'在数字世界中");
    
    Ok(())
}