//! # Standalone Browser Module
//! 
//! 提供嵌入式浏览器功能，支持创建独立可执行文件
//! 使用系统原生WebView组件，无需依赖外部浏览器

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

/// 嵌入式浏览器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandaloneBrowserConfig {
    /// 窗口标题
    pub title: String,
    /// 窗口宽度
    pub width: u32,
    /// 窗口高度
    pub height: u32,
    /// 初始URL
    pub initial_url: String,
    /// 是否可调整大小
    pub resizable: bool,
    /// 是否显示开发者工具
    pub debug: bool,
}

impl Default for StandaloneBrowserConfig {
    fn default() -> Self {
        Self {
            title: "RainbowBrowserAI - 智能浏览器助手".to_string(),
            width: 1280,
            height: 800,
            initial_url: "http://localhost:8888".to_string(),
            resizable: true,
            debug: cfg!(debug_assertions),
        }
    }
}

/// 嵌入式浏览器实例
pub struct StandaloneBrowser {
    config: StandaloneBrowserConfig,
    command_sender: Option<mpsc::Sender<BrowserCommand>>,
    state: Arc<Mutex<BrowserState>>,
}

/// 浏览器状态
#[derive(Debug, Clone, Serialize, Deserialize)]
struct BrowserState {
    current_url: String,
    is_loading: bool,
    title: String,
}

/// 浏览器命令
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BrowserCommand {
    Navigate(String),
    ExecuteScript(String),
    Reload,
    GoBack,
    GoForward,
    Screenshot,
    InjectCSS(String),
    SetTitle(String),
}

/// 浏览器事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BrowserEvent {
    PageLoaded(String),
    TitleChanged(String),
    NavigationStarted(String),
    Error(String),
    ScriptResult(serde_json::Value),
}

impl StandaloneBrowser {
    /// 创建新的嵌入式浏览器实例
    pub fn new(config: StandaloneBrowserConfig) -> Self {
        Self {
            config,
            command_sender: None,
            state: Arc::new(Mutex::new(BrowserState {
                current_url: String::new(),
                is_loading: false,
                title: String::new(),
            })),
        }
    }

    /// 启动浏览器窗口
    pub async fn launch(&mut self) -> Result<()> {
        log::info!("启动嵌入式浏览器窗口");
        
        let (cmd_tx, mut cmd_rx) = mpsc::channel::<BrowserCommand>(100);
        self.command_sender = Some(cmd_tx);
        
        let config = self.config.clone();
        let state = self.state.clone();
        
        // 启动浏览器窗口线程
        std::thread::spawn(move || {
            if let Err(e) = Self::run_browser_window(config, state, cmd_rx) {
                log::error!("浏览器窗口错误: {}", e);
            }
        });
        
        Ok(())
    }

    /// 运行浏览器窗口 - 使用系统默认浏览器
    #[cfg(feature = "standalone")]
    fn run_browser_window(
        config: StandaloneBrowserConfig,
        state: Arc<Mutex<BrowserState>>,
        mut cmd_rx: mpsc::Receiver<BrowserCommand>,
    ) -> Result<()> {
        log::info!("启动独立浏览器模式");
        
        // 启动一个简单的本地服务器来提供欢迎页面
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                Self::start_welcome_server(&config).await.unwrap_or_else(|e| {
                    log::error!("启动欢迎服务器失败: {}", e);
                });
            });
        });
        
        // 等待服务器启动
        std::thread::sleep(std::time::Duration::from_millis(1000));
        
        // 打开系统默认浏览器
        let welcome_url = format!("http://localhost:8889");
        if let Err(e) = Self::open_browser(&welcome_url) {
            log::error!("打开浏览器失败: {}", e);
            log::info!("请手动打开浏览器并访问: {}", welcome_url);
        }
        
        // 处理命令
        std::thread::spawn(move || {
            while let Some(cmd) = cmd_rx.blocking_recv() {
                log::info!("接收到命令: {:?}", cmd);
                match cmd {
                    BrowserCommand::Navigate(url) => {
                        if let Ok(mut s) = state.lock() {
                            s.current_url = url.clone();
                            log::info!("导航到: {}", url);
                            let _ = Self::open_browser(&url);
                        }
                    }
                    _ => {}
                }
            }
        });
        
        // 保持线程运行
        std::thread::park();
        Ok(())
    }

    /// 启动欢迎服务器
    async fn start_welcome_server(config: &StandaloneBrowserConfig) -> Result<()> {
        use std::net::SocketAddr;
        
        let addr: SocketAddr = "127.0.0.1:8889".parse()?;
        let html_content = Self::generate_html_content(config);
        
        // 使用简单的HTTP服务器
        let listener = tokio::net::TcpListener::bind(addr).await?;
        log::info!("欢迎服务器启动在: http://{}", addr);
        
        loop {
            let (stream, _) = listener.accept().await?;
            let html = html_content.clone();
            
            tokio::spawn(async move {
                if let Err(e) = Self::handle_request(stream, html).await {
                    log::warn!("处理请求失败: {}", e);
                }
            });
        }
    }
    
    /// 处理HTTP请求
    async fn handle_request(
        stream: tokio::net::TcpStream,
        html_content: String,
    ) -> Result<()> {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        
        let mut stream = stream;
        let mut buffer = [0; 1024];
        let _ = stream.read(&mut buffer).await?;
        
        let response = format!(
            "HTTP/1.1 200 OK\r\n\
             Content-Type: text/html; charset=utf-8\r\n\
             Content-Length: {}\r\n\
             Connection: close\r\n\
             \r\n\
             {}",
            html_content.len(),
            html_content
        );
        
        stream.write_all(response.as_bytes()).await?;
        stream.flush().await?;
        Ok(())
    }
    
    /// 打开系统默认浏览器
    fn open_browser(url: &str) -> Result<()> {
        #[cfg(target_os = "windows")]
        {
            std::process::Command::new("cmd")
                .args(["/C", "start", url])
                .output()?;
        }
        
        #[cfg(target_os = "macos")]
        {
            std::process::Command::new("open")
                .arg(url)
                .output()?;
        }
        
        #[cfg(target_os = "linux")]
        {
            std::process::Command::new("xdg-open")
                .arg(url)
                .output()?;
        }
        
        Ok(())
    }

    /// 运行浏览器窗口（无standalone特性时的备用实现）
    #[cfg(not(feature = "standalone"))]
    fn run_browser_window(
        config: StandaloneBrowserConfig,
        state: Arc<Mutex<BrowserState>>,
        mut cmd_rx: mpsc::Receiver<BrowserCommand>,
    ) -> Result<()> {
        log::warn!("web-view功能未启用，使用模拟浏览器");
        
        // 模拟浏览器运行
        std::thread::spawn(move || {
            while let Some(cmd) = cmd_rx.blocking_recv() {
                log::info!("接收到命令: {:?}", cmd);
                match cmd {
                    BrowserCommand::Navigate(url) => {
                        if let Ok(mut s) = state.lock() {
                            s.current_url = url.clone();
                            log::info!("模拟导航到: {}", url);
                        }
                    }
                    _ => {}
                }
            }
        });
        
        // 保持线程运行
        std::thread::park();
        Ok(())
    }

    /// 生成初始HTML内容
    fn generate_html_content(config: &StandaloneBrowserConfig) -> String {
        format!(r#"
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{}</title>
    <style>
        body {{
            margin: 0;
            padding: 0;
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            display: flex;
            justify-content: center;
            align-items: center;
            height: 100vh;
            color: white;
        }}
        .container {{
            text-align: center;
            padding: 2rem;
            background: rgba(255, 255, 255, 0.1);
            border-radius: 20px;
            backdrop-filter: blur(10px);
            box-shadow: 0 20px 40px rgba(0, 0, 0, 0.2);
        }}
        h1 {{
            font-size: 3rem;
            margin-bottom: 1rem;
            text-shadow: 2px 2px 4px rgba(0, 0, 0, 0.3);
        }}
        .rainbow {{
            background: linear-gradient(90deg, 
                #ff0000, #ff7f00, #ffff00, #00ff00, #0000ff, #4b0082, #9400d3);
            -webkit-background-clip: text;
            -webkit-text-fill-color: transparent;
            background-clip: text;
            animation: rainbow 3s ease-in-out infinite;
        }}
        @keyframes rainbow {{
            0%, 100% {{ filter: hue-rotate(0deg); }}
            50% {{ filter: hue-rotate(180deg); }}
        }}
        .loading {{
            margin-top: 2rem;
            font-size: 1.2rem;
            opacity: 0.8;
        }}
        .spinner {{
            border: 3px solid rgba(255, 255, 255, 0.3);
            border-top: 3px solid white;
            border-radius: 50%;
            width: 40px;
            height: 40px;
            animation: spin 1s linear infinite;
            margin: 2rem auto;
        }}
        @keyframes spin {{
            0% {{ transform: rotate(0deg); }}
            100% {{ transform: rotate(360deg); }}
        }}
        .features {{
            margin-top: 2rem;
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 1rem;
        }}
        .feature {{
            padding: 1rem;
            background: rgba(255, 255, 255, 0.1);
            border-radius: 10px;
            transition: transform 0.3s;
        }}
        .feature:hover {{
            transform: translateY(-5px);
            background: rgba(255, 255, 255, 0.2);
        }}
        .feature-icon {{
            font-size: 2rem;
            margin-bottom: 0.5rem;
        }}
    </style>
</head>
<body>
    <div class="container">
        <h1 class="rainbow">🌈 RainbowBrowserAI</h1>
        <p style="font-size: 1.5rem;">智能浏览器助手</p>
        <div class="spinner"></div>
        <div class="loading">正在启动AI引擎...</div>
        
        <div class="features">
            <div class="feature">
                <div class="feature-icon">🤖</div>
                <div>AI驱动</div>
            </div>
            <div class="feature">
                <div class="feature-icon">🎯</div>
                <div>智能操作</div>
            </div>
            <div class="feature">
                <div class="feature-icon">⚡</div>
                <div>高速处理</div>
            </div>
            <div class="feature">
                <div class="feature-icon">🔒</div>
                <div>安全可靠</div>
            </div>
        </div>
    </div>
    
    <script>
        // 通知Rust端浏览器已就绪
        setTimeout(() => {{
            if (window.external && window.external.invoke) {{
                window.external.invoke('ready');
            }}
        }}, 1000);
        
        // 监听页面导航
        window.addEventListener('load', () => {{
            if (window.external && window.external.invoke) {{
                window.external.invoke('log:页面加载完成 - ' + window.location.href);
            }}
        }});
        
        // AI助手初始化
        window.RainbowAI = {{
            navigate: function(url) {{
                if (window.external && window.external.invoke) {{
                    window.external.invoke('navigate:' + url);
                }}
            }},
            log: function(msg) {{
                if (window.external && window.external.invoke) {{
                    window.external.invoke('log:' + msg);
                }}
            }}
        }};
        
        // 3秒后自动导航到主界面
        setTimeout(() => {{
            window.location.href = '{}';
        }}, 3000);
    </script>
</body>
</html>
        "#, config.title, config.initial_url)
    }

    /// 导航到指定URL
    pub async fn navigate(&self, url: &str) -> Result<()> {
        if let Some(sender) = &self.command_sender {
            sender.send(BrowserCommand::Navigate(url.to_string())).await?;
            Ok(())
        } else {
            Err(anyhow!("浏览器未启动"))
        }
    }

    /// 执行JavaScript脚本
    pub async fn execute_script(&self, script: &str) -> Result<()> {
        if let Some(sender) = &self.command_sender {
            sender.send(BrowserCommand::ExecuteScript(script.to_string())).await?;
            Ok(())
        } else {
            Err(anyhow!("浏览器未启动"))
        }
    }

    /// 重新加载页面
    pub async fn reload(&self) -> Result<()> {
        if let Some(sender) = &self.command_sender {
            sender.send(BrowserCommand::Reload).await?;
            Ok(())
        } else {
            Err(anyhow!("浏览器未启动"))
        }
    }

    /// 后退
    pub async fn go_back(&self) -> Result<()> {
        if let Some(sender) = &self.command_sender {
            sender.send(BrowserCommand::GoBack).await?;
            Ok(())
        } else {
            Err(anyhow!("浏览器未启动"))
        }
    }

    /// 前进
    pub async fn go_forward(&self) -> Result<()> {
        if let Some(sender) = &self.command_sender {
            sender.send(BrowserCommand::GoForward).await?;
            Ok(())
        } else {
            Err(anyhow!("浏览器未启动"))
        }
    }

    /// 获取当前URL
    pub fn current_url(&self) -> String {
        if let Ok(state) = self.state.lock() {
            state.current_url.clone()
        } else {
            String::new()
        }
    }

    /// 获取页面标题
    pub fn title(&self) -> String {
        if let Ok(state) = self.state.lock() {
            state.title.clone()
        } else {
            String::new()
        }
    }

    /// 检查是否正在加载
    pub fn is_loading(&self) -> bool {
        if let Ok(state) = self.state.lock() {
            state.is_loading
        } else {
            false
        }
    }
}

/// 创建并启动独立浏览器应用
pub async fn launch_standalone_app() -> Result<()> {
    log::info!("启动RainbowBrowserAI独立应用");
    
    // 初始化配置
    let config = StandaloneBrowserConfig {
        title: "RainbowBrowserAI - 智能浏览器助手".to_string(),
        width: 1400,
        height: 900,
        initial_url: "http://localhost:8888/app".to_string(),
        resizable: true,
        debug: true,
    };
    
    // 创建浏览器实例
    let mut browser = StandaloneBrowser::new(config);
    
    // 启动浏览器窗口
    browser.launch().await?;
    
    // 启动本地AI服务器
    #[cfg(feature = "web-server")]
    {
        log::info!("启动本地AI服务器...");
        
        // 在后台启动一个简单的AI服务器
        tokio::spawn(async move {
            if let Err(e) = start_ai_server().await {
                log::error!("AI服务器启动失败: {}", e);
            }
        });
    }
    
    // 等待用户输入
    log::info!("独立应用已启动，按Ctrl+C退出");
    tokio::signal::ctrl_c().await?;
    
    Ok(())
}

/// 启动简单的AI服务器
async fn start_ai_server() -> Result<()> {
        use std::net::SocketAddr;
        
        let addr: SocketAddr = "127.0.0.1:8888".parse()?;
        let listener = tokio::net::TcpListener::bind(addr).await?;
        log::info!("AI服务器启动在: http://{}", addr);
        
        loop {
            let (stream, _) = listener.accept().await?;
            
            tokio::spawn(async move {
                if let Err(e) = handle_ai_request(stream).await {
                    log::warn!("处理AI请求失败: {}", e);
                }
            });
        }
}

/// 处理AI请求
async fn handle_ai_request(stream: tokio::net::TcpStream) -> Result<()> {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        
        let mut stream = stream;
        let mut buffer = [0; 1024];
        let n = stream.read(&mut buffer).await?;
        let request = String::from_utf8_lossy(&buffer[..n]);
        
        log::info!("收到AI请求: {}", request.lines().next().unwrap_or(""));
        
        let response_body = serde_json::json!({
            "status": "success",
            "message": "RainbowBrowserAI独立版本",
            "version": "8.0.0",
            "features": ["智能浏览器控制", "自然语言交互", "本地AI处理"]
        });
        
        let response = format!(
            "HTTP/1.1 200 OK\r\n\
             Content-Type: application/json; charset=utf-8\r\n\
             Access-Control-Allow-Origin: *\r\n\
             Access-Control-Allow-Methods: GET, POST, OPTIONS\r\n\
             Access-Control-Allow-Headers: Content-Type\r\n\
             Content-Length: {}\r\n\
             Connection: close\r\n\
             \r\n\
             {}",
            response_body.to_string().len(),
            response_body
        );
        
        stream.write_all(response.as_bytes()).await?;
        stream.flush().await?;
        Ok(())
}