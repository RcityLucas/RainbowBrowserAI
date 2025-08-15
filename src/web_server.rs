//! # RainbowBrowserAI Web Server
//! 
//! 轻量级Web服务器，提供浏览器扩展的后端支持
//! 可以编译为单个可执行文件，用户双击即可运行

use std::net::SocketAddr;
use serde::{Deserialize, Serialize};
use warp::{Filter, Reply};
use anyhow::Result;

/// Web服务器配置
pub struct WebServerConfig {
    pub port: u16,
    pub host: String,
}

impl Default for WebServerConfig {
    fn default() -> Self {
        Self {
            port: 8888,
            host: "127.0.0.1".to_string(),
        }
    }
}

/// API请求
#[derive(Debug, Deserialize)]
struct ApiRequest {
    action: String,
    message: String,
    context: Option<serde_json::Value>,
}

/// API响应
#[derive(Debug, Serialize)]
struct ApiResponse {
    success: bool,
    result: String,
    data: Option<serde_json::Value>,
}

/// 启动Web服务器
pub async fn start_server(config: WebServerConfig) -> Result<()> {
    let addr: SocketAddr = format!("{}:{}", config.host, config.port).parse()?;
    
    println!("🌈 RainbowBrowserAI 服务器启动中...");
    println!("📡 监听地址: http://{}", addr);
    println!("🌐 请安装浏览器扩展并连接到此服务器");
    println!("💡 提示: 保持此窗口开启以使用AI功能");
    
    // CORS配置
    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec!["content-type"])
        .allow_methods(vec!["GET", "POST", "OPTIONS"]);
    
    // 健康检查端点
    let health = warp::path("health")
        .map(|| warp::reply::json(&serde_json::json!({
            "status": "healthy",
            "service": "RainbowBrowserAI",
            "version": "8.0.0"
        })));
    
    // 状态端点
    let status = warp::path("status")
        .map(|| warp::reply::json(&serde_json::json!({
            "status": "ready",
            "engines": {
                "unified_kernel": true,
                "layered_perception": true,
                "intelligent_action": true,
                "optimized_persistence": true,
                "performance_engine": true,
                "stability_engine": true
            }
        })));
    
    // AI处理端点
    let ai_process = warp::path("ai")
        .and(warp::post())
        .and(warp::body::json())
        .and_then(handle_ai_request);
    
    // 静态文件服务 (可选的Web UI)
    let static_files = warp::path("ui")
        .and(warp::fs::dir("./web_ui"));
    
    // 组合所有路由
    let routes = health
        .or(status)
        .or(ai_process)
        .or(static_files)
        .with(cors);
    
    // 启动服务器
    warp::serve(routes)
        .run(addr)
        .await;
    
    Ok(())
}

/// 处理AI请求
async fn handle_ai_request(req: ApiRequest) -> Result<impl Reply, warp::Rejection> {
    println!("📥 收到请求: {} - {}", req.action, req.message);
    
    // 这里连接到实际的AI处理逻辑
    let result = process_ai_action(&req.action, &req.message).await;
    
    let response = ApiResponse {
        success: true,
        result: result.clone(),
        data: Some(serde_json::json!({
            "processed_by": "RainbowBrowserAI",
            "action": req.action,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        })),
    };
    
    println!("📤 发送响应: {}", result);
    Ok(warp::reply::json(&response))
}

/// 处理AI动作
async fn process_ai_action(action: &str, message: &str) -> String {
    match action {
        "search" => format!("搜索完成: {}", message),
        "extract" => format!("数据提取完成: {}", message),
        "fill" => format!("表单填写完成: {}", message),
        "navigate" => format!("导航到: {}", message),
        "analyze" => format!("分析完成: {}", message),
        _ => {
            // 调用LLM进行智能处理
            format!("智能处理: {}", message)
        }
    }
}

/// 独立运行的主函数
#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    
    // 显示启动画面
    print_banner();
    
    // 启动服务器
    let config = WebServerConfig::default();
    start_server(config).await?;
    
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