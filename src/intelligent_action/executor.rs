// 行动执行器 - 执行具体的浏览器操作

use anyhow::Result;
use super::Action;
use super::browser_driver::BrowserDriver;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 行动执行器
pub struct ActionExecutor {
    driver: Arc<RwLock<BrowserDriver>>,
    use_real_browser: bool,
}

impl ActionExecutor {
    pub async fn new() -> Result<Self> {
        let mut driver = BrowserDriver::new().await?;
        
        // 尝试初始化真实浏览器
        let use_real_browser = match driver.init().await {
            Ok(_) => {
                log::info!("成功连接到WebDriver");
                true
            }
            Err(e) => {
                log::warn!("无法连接到WebDriver，使用模拟模式: {}", e);
                false
            }
        };
        
        Ok(Self {
            driver: Arc::new(RwLock::new(driver)),
            use_real_browser,
        })
    }
    
    /// 执行行动
    pub async fn execute(&self, action: &Action) -> Result<serde_json::Value> {
        if self.use_real_browser {
            // 使用真实浏览器
            let mut driver = self.driver.write().await;
            let result = driver.execute_action(action.clone()).await?;
            Ok(serde_json::json!({"result": result}))
        } else {
            // 模拟模式
            self.execute_simulated(action).await
        }
    }
    
    /// 模拟执行（当WebDriver不可用时）
    async fn execute_simulated(&self, action: &Action) -> Result<serde_json::Value> {
        match action {
            Action::Click { selector } => {
                log::info!("[模拟] 点击元素: {}", selector);
                Ok(serde_json::json!({ "clicked": selector, "simulated": true }))
            }
            
            Action::Input { selector, text } => {
                log::info!("[模拟] 输入文本到 {}: {}", selector, text);
                Ok(serde_json::json!({ "input": text, "simulated": true }))
            }
            
            Action::Navigate { url } => {
                log::info!("[模拟] 导航到: {}", url);
                Ok(serde_json::json!({ "navigated": url, "simulated": true }))
            }
            
            Action::Wait { selector, timeout_ms, duration_ms } => {
                log::info!("等待元素: {} ({}ms)", selector, duration_ms);
                tokio::time::sleep(tokio::time::Duration::from_millis(*duration_ms)).await;
                Ok(serde_json::json!({ "waited": duration_ms, "selector": selector, "timeout_ms": timeout_ms }))
            }
            
            Action::Screenshot => {
                log::info!("[模拟] 截图");
                Ok(serde_json::json!({ "screenshot": "simulated", "status": "completed" }))
            }
            
            Action::ExecuteScript { script } => {
                log::info!("[模拟] 执行脚本: {}", script);
                Ok(serde_json::json!({ "script_result": "simulated" }))
            }
            
            Action::Scroll { direction, amount } => {
                log::info!("[模拟] 滚动 {:?} {} 像素", direction, amount);
                Ok(serde_json::json!({ "scrolled": amount, "simulated": true }))
            }
            
            Action::Extract { selector } => {
                log::info!("[模拟] 提取元素: {}", selector);
                Ok(serde_json::json!({ "extracted": "simulated content" }))
            }
            
            Action::Type { selector, text } => {
                log::info!("[模拟] 在 {} 输入文本: {}", selector, text);
                Ok(serde_json::json!({ "typed": text, "selector": selector, "simulated": true }))
            }
        }
    }
}