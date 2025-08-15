//! # 简化数据存储模块
//! 
//! 提供基础的数据存储和会话管理功能

use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// 简化的存储管理器
#[derive(Debug)]
pub struct SimpleStorage {
    sessions: Arc<RwLock<HashMap<String, SessionData>>>,
    conversations: Arc<RwLock<HashMap<String, Vec<ConversationEntry>>>>,
}

/// 会话数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionData {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub last_accessed: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

/// 对话记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationEntry {
    pub id: String,
    pub session_id: String,
    pub user_input: String,
    pub ai_response: String,
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

impl SimpleStorage {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            conversations: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// 创建新会话
    pub fn create_session(&self) -> SessionData {
        let session_id = Uuid::new_v4().to_string();
        let now = Utc::now();
        
        let session = SessionData {
            id: session_id.clone(),
            created_at: now,
            last_accessed: now,
            metadata: HashMap::new(),
        };
        
        self.sessions.write().insert(session_id.clone(), session.clone());
        self.conversations.write().insert(session_id, Vec::new());
        
        session
    }
    
    /// 获取会话
    pub fn get_session(&self, session_id: &str) -> Option<SessionData> {
        self.sessions.read().get(session_id).cloned()
    }
    
    /// 更新会话访问时间
    pub fn touch_session(&self, session_id: &str) {
        if let Some(session) = self.sessions.write().get_mut(session_id) {
            session.last_accessed = Utc::now();
        }
    }
    
    /// 添加对话记录
    pub fn add_conversation(&self, session_id: &str, user_input: String, ai_response: String) {
        let entry = ConversationEntry {
            id: Uuid::new_v4().to_string(),
            session_id: session_id.to_string(),
            user_input,
            ai_response,
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        };
        
        if let Some(conversations) = self.conversations.write().get_mut(session_id) {
            conversations.push(entry);
        }
    }
    
    /// 获取对话历史
    pub fn get_conversation_history(&self, session_id: &str) -> Vec<ConversationEntry> {
        self.conversations.read()
            .get(session_id)
            .cloned()
            .unwrap_or_default()
    }
    
    /// 清理过期会话
    pub fn cleanup_expired_sessions(&self, max_age_hours: i64) {
        let cutoff = Utc::now() - chrono::Duration::hours(max_age_hours);
        let mut sessions = self.sessions.write();
        let mut conversations = self.conversations.write();
        
        let expired_ids: Vec<String> = sessions
            .iter()
            .filter(|(_, session)| session.last_accessed < cutoff)
            .map(|(id, _)| id.clone())
            .collect();
        
        for id in expired_ids {
            sessions.remove(&id);
            conversations.remove(&id);
        }
    }
}

impl Default for SimpleStorage {
    fn default() -> Self {
        Self::new()
    }
}