//! # RainbowBrowserAI - Main Entry Point
//! 
//! 智能浏览器AI助手的主程序入口
//! 提供Web服务器模式和交互式CLI模式

#[cfg(feature = "web-server")]
mod web_server;

use anyhow::Result;
use rainbow_browser_ai::prelude::*;
use std::io::{self, Write};

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    init_logger("info")?;
    
    // 检查命令行参数
    let args: Vec<String> = std::env::args().collect();
    
    // 如果有参数且第一个参数是 "server"，直接启动服务器
    if args.len() > 1 && args[1] == "server" {
        #[cfg(feature = "web-server")]
        {
            print_server_banner();
            let config = web_server::WebServerConfig::default();
            web_server::start_server(config).await?;
            return Ok(());
        }
        
        #[cfg(not(feature = "web-server"))]
        {
            println!("❌ Web服务器功能未启用");
            println!("请使用以下命令重新编译:");
            println!("cargo build --release --features web-server");
            return Ok(());
        }
    }
    
    // 显示主菜单
    print_banner();
    
    // 如果有其他参数，作为命令处理
    if args.len() > 1 {
        let browser = RainbowBrowserV8::new().await?;
        let command = &args[1];
        match command.as_str() {
            "status" => {
                println!("📊 系统状态:");
                let status = browser.get_system_status().await?;
                println!("{:#?}", status);
            }
            "optimize" => {
                println!("🔧 开始系统优化...");
                browser.optimize().await?;
                println!("✅ 优化完成!");
            }
            "help" => {
                print_help();
            }
            _ => {
                // 将剩余参数作为请求处理
                let request = args[1..].join(" ");
                println!("🚀 执行任务: {}", request);
                let result = browser.process_request(&request).await?;
                println!("\n{}", result);
            }
        }
    } else {
        // 交互式菜单模式
        interactive_menu().await?;
    }
    
    Ok(())
}

/// 交互式菜单
async fn interactive_menu() -> Result<()> {
    loop {
        print_menu();
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let choice = input.trim();
        
        match choice {
            "1" => {
                #[cfg(feature = "web-server")]
                {
                    println!("\n🚀 正在启动Web服务器...");
                    println!("📡 服务器将运行在: http://localhost:8888");
                    println!("💡 请安装浏览器扩展并连接到此服务器");
                    println!("⚠️  保持此窗口开启以使用AI功能\n");
                    
                    let config = web_server::WebServerConfig::default();
                    web_server::start_server(config).await?;
                }
                
                #[cfg(not(feature = "web-server"))]
                {
                    println!("\n❌ Web服务器功能未启用");
                    println!("请使用以下命令重新编译:");
                    println!("cargo build --release --features web-server");
                }
            },
            "2" => {
                println!("\n💬 进入AI对话模式...");
                println!("输入 'help' 查看帮助，'quit' 返回主菜单");
                let browser = RainbowBrowserV8::new().await?;
                interactive_mode(browser).await?;
            },
            "3" => {
                println!("\n🎯 运行演示程序...");
                println!("请使用以下命令运行具体的演示:");
                println!("  cargo run --example real_world_demo");
                println!("  cargo run --example simple_demo");
            },
            "4" => {
                print_usage();
            },
            "5" => {
                println!("\n👋 感谢使用RainbowBrowserAI！再见！");
                break;
            },
            _ => {
                println!("\n❌ 无效的选择，请输入1-5之间的数字");
            }
        }
    }
    
    Ok(())
}

/// 交互模式
async fn interactive_mode(browser: RainbowBrowserV8) -> Result<()> {
    loop {
        print!("\n🌈 > ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();
        
        match input {
            "quit" | "exit" => {
                println!("👋 返回主菜单...");
                break;
            }
            "help" => {
                print_help();
            }
            "status" => {
                let status = browser.get_system_status().await?;
                println!("{:#?}", status);
            }
            "optimize" => {
                browser.optimize().await?;
                println!("✅ 优化完成!");
            }
            "" => continue,
            request => {
                match browser.process_request(request).await {
                    Ok(response) => println!("{}", response),
                    Err(e) => println!("❌ 错误: {}", e),
                }
            }
        }
    }
    
    Ok(())
}

fn print_banner() {
    println!(r#"
╔══════════════════════════════════════════════════════════════════╗
║                                                                  ║
║              🌈 RainbowBrowserAI v8.0                           ║
║                                                                  ║
║         智能浏览器助手 - 用AI控制你的浏览器                      ║
║                                                                  ║
║  六大引擎架构 | LLM智能驱动 | 自然语言控制                      ║
║                                                                  ║
╚══════════════════════════════════════════════════════════════════╝
    "#);
}

fn print_server_banner() {
    println!(r#"
╔══════════════════════════════════════════════════════════════════╗
║                                                                  ║
║              🌈 RainbowBrowserAI v8.0                           ║
║                                                                  ║
║         智能浏览器助手 - Web服务器模式                           ║
║                                                                  ║
║  功能特性:                                                       ║
║  • 自然语言控制浏览器                                           ║
║  • 智能网页数据提取                                             ║
║  • 自动化表单填写                                               ║
║  • 智能搜索和导航                                               ║
║  • 批量任务处理                                                 ║
║                                                                  ║
║  使用方法:                                                       ║
║  1. 保持此窗口运行                                              ║
║  2. 安装浏览器扩展                                              ║
║  3. 点击浏览器工具栏的彩虹图标                                  ║
║  4. 开始使用自然语言控制浏览器！                                ║
║                                                                  ║
╚══════════════════════════════════════════════════════════════════╝
    "#);
}

fn print_menu() {
    println!("\n请选择运行模式:");
    println!("1. 🌐 启动Web服务器 (为浏览器扩展提供AI支持)");
    println!("2. 💬 进入AI对话模式");
    println!("3. 🔧 运行演示程序");
    println!("4. 📖 查看使用说明");
    println!("5. 🚪 退出");
    print!("\n请输入选择 (1-5): ");
    io::stdout().flush().unwrap();
}

fn print_help() {
    println!(r#"
📚 可用命令:
  help      - 显示此帮助信息
  status    - 查看系统状态
  optimize  - 优化系统
  quit/exit - 返回主菜单
  
  或直接输入任务描述，例如:
  - "搜索人工智能最新发展"
  - "帮我查找杭州旅游攻略"
  - "购买性价比高的笔记本电脑"
  - "分析这个网页的内容"
  - "提取所有商品价格"
    "#);
}

fn print_usage() {
    println!(r#"
📚 使用说明
═══════════════════════════════════════════════════════════════

🌈 RainbowBrowserAI 提供三种使用方式：

1️⃣  浏览器扩展（推荐）
   • 安装 src/browser_extension 文件夹中的扩展
   • 启动本地服务器（选项1）
   • 点击浏览器工具栏的彩虹图标使用

2️⃣  命令行模式
   • 选择选项2进入AI对话模式
   • 或运行: cargo run -- "你的命令"
   • 示例: cargo run -- "搜索AI新闻"

3️⃣  API模式
   • 启动服务器后访问 http://localhost:8888
   • 使用 POST /ai 端点发送请求

📦 安装浏览器扩展：
   1. 打开浏览器扩展管理页面
      • Chrome: chrome://extensions
      • Edge: edge://extensions
      • Firefox: about:addons
   2. 开启"开发者模式"
   3. 点击"加载已解压的扩展程序"
   4. 选择 src/browser_extension 文件夹

🎯 快捷键：
   • Ctrl+Shift+R (Windows/Linux)
   • Cmd+Shift+R (macOS)

💡 支持的命令示例：
   • "搜索最新科技新闻"
   • "点击登录按钮"
   • "填写表单"
   • "提取所有价格"
   • "导航到GitHub"
   • "截图保存"
   • "滚动到底部"

📖 详细文档请查看 USER_GUIDE.md

═══════════════════════════════════════════════════════════════
    "#);
}

fn init_logger(level: &str) -> Result<()> {
    let filter = match level {
        "trace" => tracing::Level::TRACE,
        "debug" => tracing::Level::DEBUG,
        "info" => tracing::Level::INFO,
        "warn" => tracing::Level::WARN,
        "error" => tracing::Level::ERROR,
        _ => tracing::Level::INFO,
    };
    
    tracing_subscriber::fmt()
        .with_max_level(filter)
        .with_target(false)
        .with_thread_ids(false)
        .with_thread_names(false)
        .init();
    
    Ok(())
}