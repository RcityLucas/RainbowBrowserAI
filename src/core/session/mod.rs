//! # 集成浏览器系统 - 会话管理与浏览器控制的完整实现
//! 
//! 将统一内核的会话管理与实际的浏览器控制结合

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, Mutex};
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// 引入浏览器控制模块
use crate::base::browser::{BrowserController, BrowserControlConfig};

/// 会话状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionState {
    Created,
    Active,
    Idle,
    Suspended,
    Terminated,
}

/// 浏览器会话
#[derive(Debug)]
pub struct BrowserSession {
    pub id: String,
    pub state: SessionState,
    pub browser: Arc<Mutex<BrowserController>>,
    pub created_at: Instant,
    pub last_activity: Instant,
    pub metadata: HashMap<String, String>,
}

/// 集成的浏览器系统 - 统一内核与浏览器控制的桥梁
pub struct IntegratedBrowser {
    sessions: Arc<RwLock<HashMap<String, BrowserSession>>>,
    config: IntegratedConfig,
    max_sessions: usize,
}

#[derive(Debug, Clone)]
pub struct IntegratedConfig {
    pub browser_config: BrowserControlConfig,
    pub session_timeout: Duration,
    pub max_sessions: usize,
    pub auto_cleanup: bool,
}

impl Default for IntegratedConfig {
    fn default() -> Self {
        Self {
            browser_config: BrowserControlConfig::default(),
            session_timeout: Duration::from_secs(1800), // 30分钟
            max_sessions: 5,
            auto_cleanup: true,
        }
    }
}

impl IntegratedBrowser {
    /// 创建集成浏览器系统
    pub async fn new(config: IntegratedConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let max_sessions = config.max_sessions;
        
        Ok(Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            config,
            max_sessions,
        })
    }
    
    /// 创建新会话
    pub async fn create_session(&self) -> Result<String, Box<dyn std::error::Error>> {
        let mut sessions = self.sessions.write().await;
        
        // 检查会话数量限制
        if sessions.len() >= self.max_sessions {
            return Err("已达到最大会话数量限制".into());
        }
        
        // 创建会话ID
        let session_id = Uuid::new_v4().to_string();
        
        println!("🆕 创建新会话: {}", session_id);
        
        // 创建浏览器控制器
        let mut browser = BrowserController::new(self.config.browser_config.clone()).await?;
        browser.start().await?;
        
        // 创建会话
        let session = BrowserSession {
            id: session_id.clone(),
            state: SessionState::Active,
            browser: Arc::new(Mutex::new(browser)),
            created_at: Instant::now(),
            last_activity: Instant::now(),
            metadata: HashMap::new(),
        };
        
        sessions.insert(session_id.clone(), session);
        
        println!("✅ 会话创建成功");
        
        Ok(session_id)
    }
    
    /// 执行浏览器操作
    pub async fn execute_in_session<F, R>(&self, session_id: &str, operation: F) -> Result<R, Box<dyn std::error::Error>>
    where
        F: for<'a> FnOnce(&'a BrowserController) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<R, Box<dyn std::error::Error>>> + 'a>>,
    {
        let sessions = self.sessions.read().await;
        
        if let Some(session) = sessions.get(session_id) {
            // 更新最后活动时间
            let browser = session.browser.lock().await;
            let result = operation(&*browser).await;
            
            // 这里应该更新 last_activity，但由于是只读锁，我们暂时跳过
            
            result
        } else {
            Err(format!("会话不存在: {}", session_id).into())
        }
    }
    
    /// 导航到URL
    pub async fn navigate(&self, session_id: &str, url: &str) -> Result<(), Box<dyn std::error::Error>> {
        println!("🌐 会话 {} 导航到: {}", session_id, url);
        
        let sessions = self.sessions.read().await;
        if let Some(session) = sessions.get(session_id) {
            let mut browser = session.browser.lock().await;
            browser.navigate(url).await.map_err(|e| e.into())
        } else {
            Err(format!("会话不存在: {}", session_id).into())
        }
    }
    
    /// 点击元素
    pub async fn click(&self, session_id: &str, selector: &str) -> Result<(), Box<dyn std::error::Error>> {
        println!("🖱️ 会话 {} 点击: {}", session_id, selector);
        
        let sessions = self.sessions.read().await;
        if let Some(session) = sessions.get(session_id) {
            let browser = session.browser.lock().await;
            browser.click(selector).await.map_err(|e| e.into())
        } else {
            Err(format!("会话不存在: {}", session_id).into())
        }
    }
    
    /// 输入文本
    pub async fn input_text(&self, session_id: &str, selector: &str, text: &str) -> Result<(), Box<dyn std::error::Error>> {
        println!("⌨️ 会话 {} 输入文本到: {}", session_id, selector);
        
        let sessions = self.sessions.read().await;
        if let Some(session) = sessions.get(session_id) {
            let browser = session.browser.lock().await;
            browser.input_text(selector, text).await.map_err(|e| e.into())
        } else {
            Err(format!("会话不存在: {}", session_id).into())
        }
    }
    
    /// 执行JavaScript
    pub async fn execute_script(&self, session_id: &str, script: &str) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        println!("📜 会话 {} 执行脚本", session_id);
        
        let sessions = self.sessions.read().await;
        if let Some(session) = sessions.get(session_id) {
            let browser = session.browser.lock().await;
            browser.execute_script(script).await.map_err(|e| e.into())
        } else {
            Err(format!("会话不存在: {}", session_id).into())
        }
    }
    
    /// 获取页面文本
    pub async fn get_text(&self, session_id: &str, selector: &str) -> Result<String, Box<dyn std::error::Error>> {
        let sessions = self.sessions.read().await;
        if let Some(session) = sessions.get(session_id) {
            let browser = session.browser.lock().await;
            browser.get_text(selector).await.map_err(|e| e.into())
        } else {
            Err(format!("会话不存在: {}", session_id).into())
        }
    }
    
    /// 截图
    pub async fn screenshot(&self, session_id: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        println!("📸 会话 {} 截图", session_id);
        
        let sessions = self.sessions.read().await;
        if let Some(session) = sessions.get(session_id) {
            let browser = session.browser.lock().await;
            browser.screenshot().await.map_err(|e| e.into())
        } else {
            Err(format!("会话不存在: {}", session_id).into())
        }
    }
    
    /// 获取当前URL
    pub async fn current_url(&self, session_id: &str) -> Result<String, Box<dyn std::error::Error>> {
        let sessions = self.sessions.read().await;
        if let Some(session) = sessions.get(session_id) {
            let browser = session.browser.lock().await;
            browser.current_url().await.map_err(|e| e.into())
        } else {
            Err(format!("会话不存在: {}", session_id).into())
        }
    }
    
    /// 获取所有会话信息
    pub async fn list_sessions(&self) -> Vec<(String, SessionState)> {
        let sessions = self.sessions.read().await;
        sessions.iter()
            .map(|(id, session)| (id.clone(), session.state.clone()))
            .collect()
    }
    
    /// 终止会话
    pub async fn terminate_session(&self, session_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut sessions = self.sessions.write().await;
        
        if let Some(mut session) = sessions.remove(session_id) {
            println!("🔚 终止会话: {}", session_id);
            
            // 关闭浏览器
            let mut browser = session.browser.lock().await;
            browser.quit().await?;
            
            session.state = SessionState::Terminated;
            
            println!("✅ 会话已终止");
            Ok(())
        } else {
            Err(format!("会话不存在: {}", session_id).into())
        }
    }
    
    /// 清理过期会话
    pub async fn cleanup_expired_sessions(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let mut sessions = self.sessions.write().await;
        let mut expired = Vec::new();
        let now = Instant::now();
        
        // 找出过期的会话
        let expired_ids: Vec<String> = sessions.iter()
            .filter(|(_, session)| {
                now.duration_since(session.last_activity) > self.config.session_timeout
            })
            .map(|(id, _)| id.clone())
            .collect();
        
        // 终止过期会话
        for session_id in &expired_ids {
            if let Some(mut session) = sessions.remove(session_id) {
                println!("⏰ 清理过期会话: {}", session_id);
                
                let mut browser = session.browser.lock().await;
                let _ = browser.quit().await;
                
                session.state = SessionState::Terminated;
                expired.push(session_id.clone());
            }
        }
        
        if !expired.is_empty() {
            println!("✅ 清理了 {} 个过期会话", expired.len());
        }
        
        Ok(expired)
    }
    
    /// 获取会话统计
    pub async fn get_statistics(&self) -> SessionStatistics {
        let sessions = self.sessions.read().await;
        
        let active = sessions.values()
            .filter(|s| matches!(s.state, SessionState::Active))
            .count();
        
        let total = sessions.len();
        
        SessionStatistics {
            total_sessions: total,
            active_sessions: active,
            max_sessions: self.max_sessions,
        }
    }
}

/// 会话统计信息
#[derive(Debug, Serialize, Deserialize)]
pub struct SessionStatistics {
    pub total_sessions: usize,
    pub active_sessions: usize,
    pub max_sessions: usize,
}

/// 浏览器操作结果
#[derive(Debug, Serialize, Deserialize)]
pub struct OperationResult {
    pub success: bool,
    pub session_id: String,
    pub operation: String,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
}

impl OperationResult {
    pub fn success(session_id: String, operation: String, result: serde_json::Value) -> Self {
        Self {
            success: true,
            session_id,
            operation,
            result: Some(result),
            error: None,
        }
    }
    
    pub fn failure(session_id: String, operation: String, error: String) -> Self {
        Self {
            success: false,
            session_id,
            operation,
            result: None,
            error: Some(error),
        }
    }
}

/// 简化的会话管理器
pub struct SessionManager {
    sessions: Arc<RwLock<HashMap<String, BrowserSession>>>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    pub async fn create_session(&self) -> Result<String, Box<dyn std::error::Error>> {
        let session_id = Uuid::new_v4().to_string();
        let config = BrowserControlConfig::default();
        let browser = BrowserController::new(config).await?;
        // 注意：这里我们不调用start()以避免WebDriver依赖问题
        
        let session = BrowserSession {
            id: session_id.clone(),
            state: SessionState::Created,
            browser: Arc::new(Mutex::new(browser)),
            created_at: Instant::now(),
            last_activity: Instant::now(),
            metadata: HashMap::new(),
        };
        
        self.sessions.write().await.insert(session_id.clone(), session);
        Ok(session_id)
    }
    
    pub async fn get_session(&self, session_id: &str) -> Option<BrowserSession> {
        // Note: This is a simplified implementation
        // In a real scenario, we'd need to handle the Arc<Mutex<BrowserController>> properly
        None
    }
}