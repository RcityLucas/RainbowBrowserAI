//! # RainbowBrowserAI Standalone Executable
//! 
//! 独立可执行文件入口点，包含嵌入式浏览器

use anyhow::Result;

#[cfg(feature = "standalone")]
use rainbow_browser_ai::standalone_browser;

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志系统
    init_logger("info")?;
    
    // 打印启动信息
    print_banner();
    
    // 启动独立应用
    #[cfg(feature = "standalone")]
    {
        standalone_browser::launch_standalone_app().await?;
    }
    
    #[cfg(not(feature = "standalone"))]
    {
        println!("❌ standalone功能未启用");
        println!("请使用以下命令重新编译:");
        println!("cargo build --release --bin rainbow-browser-standalone --features standalone");
        return Ok(());
    }
    
    Ok(())
}

fn print_banner() {
    println!(r#"
╔══════════════════════════════════════════════════════════════════╗
║                                                                  ║
║         🌈 RainbowBrowserAI Standalone v8.0                     ║
║                                                                  ║
║            智能浏览器助手 - 独立可执行版本                       ║
║                                                                  ║
║  特性:                                                           ║
║  • 内置浏览器引擎，无需外部浏览器                               ║
║  • 集成AI助手，自然语言控制                                     ║
║  • 本地运行，数据安全                                           ║
║  • 跨平台支持 (Windows/macOS/Linux)                             ║
║                                                                  ║
║  使用说明:                                                       ║
║  • 应用启动后会自动打开浏览器窗口                               ║
║  • AI服务器在后台自动运行                                       ║
║  • 使用自然语言与AI助手交互                                     ║
║  • 按Ctrl+C安全退出应用                                         ║
║                                                                  ║
╚══════════════════════════════════════════════════════════════════╝
    "#);
}

fn init_logger(level: &str) -> Result<()> {
    use tracing_subscriber::prelude::*;
    
    let filter = match level {
        "trace" => tracing::Level::TRACE,
        "debug" => tracing::Level::DEBUG,
        "info" => tracing::Level::INFO,
        "warn" => tracing::Level::WARN,
        "error" => tracing::Level::ERROR,
        _ => tracing::Level::INFO,
    };
    
    // 配置日志格式
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(false)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_file(false)
        .with_line_number(false)
        .with_level(true)
        .with_ansi(true)
        .compact();
    
    // 构建订阅者
    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(tracing_subscriber::filter::LevelFilter::from(filter))
        .init();
    
    Ok(())
}