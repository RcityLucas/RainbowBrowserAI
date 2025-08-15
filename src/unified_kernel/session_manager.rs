// 会话管理器 - 管理所有会话的生命周期

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use anyhow::{Result, anyhow};
use super::Session;

/// 会话管理器
pub struct SessionManager {
    sessions: Arc<RwLock<HashMap<Uuid, Session>>>,
    max_sessions: usize,
}

impl SessionManager {
    pub async fn new() -> Result<Self> {
        Ok(Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            max_sessions: 100,
        })
    }
    
    /// 注册新会话
    pub async fn register_session(&self, session: &Session) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        
        if sessions.len() >= self.max_sessions {
            return Err(anyhow!("会话数量已达上限"));
        }
        
        sessions.insert(session.id, session.clone());
        Ok(())
    }
    
    /// 获取会话
    pub async fn get_session(&self, session_id: &Uuid) -> Result<Session> {
        let sessions = self.sessions.read().await;
        sessions.get(session_id)
            .cloned()
            .ok_or_else(|| anyhow!("会话不存在: {}", session_id))
    }
    
    /// 注销会话
    pub async fn unregister_session(&self, session_id: &Uuid) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        sessions.remove(session_id)
            .ok_or_else(|| anyhow!("会话不存在: {}", session_id))?;
        Ok(())
    }
    
    /// 获取所有活跃会话
    pub async fn get_active_sessions(&self) -> Result<Vec<Session>> {
        let sessions = self.sessions.read().await;
        Ok(sessions.values().cloned().collect())
    }
    
    /// 获取会话数量
    pub async fn get_session_count(&self) -> usize {
        let sessions = self.sessions.read().await;
        sessions.len()
    }
}