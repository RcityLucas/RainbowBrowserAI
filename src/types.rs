//! # 类型定义
//! 
//! 定义项目中使用的核心类型

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// 执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub success: bool,
    pub message: String,
    pub data: HashMap<String, String>,
    pub duration_ms: u64,
}

/// 执行记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRecord {
    pub id: String,
    pub user_input: String,
    pub timestamp: DateTime<Utc>,
    pub success: bool,
    pub duration_ms: u64,
    pub metadata: HashMap<String, String>,
}

/// 智能执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartExecutionResult {
    pub success: bool,
    pub execution_summary: String,
    pub extracted_data: HashMap<String, Vec<String>>,
    pub recommendations: Vec<String>,
    pub next_actions: Vec<String>,
    pub llm_response: String,
}

/// 浏览器会话信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

/// 会话
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub created_at: std::time::SystemTime,
    pub last_access: std::time::SystemTime,
    pub metadata: HashMap<String, String>,
}

/// 对话条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationEntry {
    pub id: String,
    pub user_input: String,
    pub assistant_response: String,
    pub timestamp: std::time::SystemTime,
}

/// 操作结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationResult {
    pub success: bool,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

/// 健康状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Warning,
    Critical,
    Offline,
}

/// 默认实现
impl Default for Session {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            created_at: std::time::SystemTime::now(),
            last_access: std::time::SystemTime::now(),
            metadata: HashMap::new(),
        }
    }
}