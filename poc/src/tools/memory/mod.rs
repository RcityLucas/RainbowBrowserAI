//! Memory Tools Module - V8.0 Standard Compliance
//! 
//! Provides persistent state management and context preservation for browser automation.
//! Part of the V8.0 standard's Memory category (记忆类).

pub mod session_memory;
pub mod persistent_cache;
pub mod history_tracker;
pub mod get_element_info;
pub mod take_screenshot;
pub mod retrieve_history;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;

use crate::tools::{Tool, Result};
use crate::tools::errors::ToolError;

/// Common trait for all memory tools
#[async_trait]
pub trait MemoryTool: Tool {
    /// Store a key-value pair in memory
    async fn store(&self, key: String, value: Value) -> Result<()>;
    
    /// Retrieve a value from memory by key
    async fn retrieve(&self, key: &str) -> Result<Option<Value>>;
    
    /// Clear all stored data
    async fn clear(&self) -> Result<()>;
    
    /// Get current memory usage statistics
    async fn stats(&self) -> Result<MemoryStats>;
}

/// Memory usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    pub total_entries: usize,
    pub memory_bytes: usize,
    pub last_accessed: Option<chrono::DateTime<chrono::Utc>>,
    pub hit_rate: f32,
}

/// Session context for memory operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionContext {
    pub session_id: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub metadata: HashMap<String, Value>,
}

impl SessionContext {
    pub fn new() -> Self {
        Self {
            session_id: Uuid::new_v4(),
            created_at: chrono::Utc::now(),
            metadata: HashMap::new(),
        }
    }
}

/// Action record for history tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionRecord {
    pub id: Uuid,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub action_type: String,
    pub tool_name: String,
    pub input: Value,
    pub output: Option<Value>,
    pub duration_ms: u64,
    pub success: bool,
    pub error: Option<String>,
}

/// Cache strategy for persistent storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CacheStrategy {
    LRU { max_size: usize },
    TTL { ttl_seconds: u64 },
    FIFO { max_size: usize },
    Adaptive,
}

impl Default for CacheStrategy {
    fn default() -> Self {
        CacheStrategy::LRU { max_size: 1000 }
    }
}

/// Re-export for convenience
pub use self::{
    session_memory::SessionMemory,
    persistent_cache::PersistentCache,
    history_tracker::HistoryTracker,
    get_element_info::{GetElementInfo, GetElementInfoParams, ElementInfo},
    take_screenshot::{TakeScreenshot, TakeScreenshotParams, Screenshot},
    retrieve_history::{RetrieveHistory, RetrieveHistoryParams, HistoryResult},
};