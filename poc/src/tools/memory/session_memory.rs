//! SessionMemory Tool - V8.0 Standard Tool #8
//! 
//! Provides session-level state management for browser automation workflows.
//! Stores context data that persists within a browser session but is cleared when session ends.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::browser::Browser;
use crate::tools::{Tool, Result};
use crate::tools::errors::ToolError;
use super::{MemoryTool, MemoryStats, SessionContext, ActionRecord};

/// Input parameters for SessionMemory operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMemoryInput {
    pub operation: MemoryOperation,
    pub session_id: Option<Uuid>,
}

/// Memory operations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MemoryOperation {
    Store { key: String, value: Value },
    Retrieve { key: String },
    Delete { key: String },
    ListKeys,
    Clear,
    GetStats,
    ExportSession,
}

/// Output from SessionMemory operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMemoryOutput {
    pub success: bool,
    pub operation: String,
    pub result: Option<Value>,
    pub session_id: Uuid,
    pub timestamp: DateTime<Utc>,
}

/// Session memory storage
struct MemoryStorage {
    data: HashMap<String, Value>,
    context: SessionContext,
    access_count: HashMap<String, usize>,
    last_accessed: HashMap<String, DateTime<Utc>>,
}

/// SessionMemory tool implementation
pub struct SessionMemory {
    browser: Arc<Browser>,
    storage: Arc<RwLock<HashMap<Uuid, MemoryStorage>>>,
    max_entries_per_session: usize,
}

impl SessionMemory {
    /// Create a new SessionMemory instance
    pub fn new(browser: Arc<Browser>) -> Self {
        Self {
            browser,
            storage: Arc::new(RwLock::new(HashMap::new())),
            max_entries_per_session: 10000,
        }
    }

    /// Get or create a session storage
    async fn get_or_create_session(&self, session_id: Option<Uuid>) -> Result<Uuid> {
        let mut storage = self.storage.write().await;
        
        let id = session_id.unwrap_or_else(Uuid::new_v4);
        
        if !storage.contains_key(&id) {
            storage.insert(id, MemoryStorage {
                data: HashMap::new(),
                context: SessionContext {
                    session_id: id,
                    created_at: Utc::now(),
                    metadata: HashMap::new(),
                },
                access_count: HashMap::new(),
                last_accessed: HashMap::new(),
            });
        }
        
        Ok(id)
    }

    /// Store a value in session memory
    async fn store_value(&self, session_id: Uuid, key: String, value: Value) -> Result<()> {
        let mut storage = self.storage.write().await;
        
        if let Some(session) = storage.get_mut(&session_id) {
            if session.data.len() >= self.max_entries_per_session {
                return Err(ToolError::InvalidInput(
                    format!("Session memory full: max {} entries", self.max_entries_per_session)
                ).into());
            }
            
            session.data.insert(key.clone(), value);
            *session.access_count.entry(key.clone()).or_insert(0) += 1;
            session.last_accessed.insert(key, Utc::now());
            
            Ok(())
        } else {
            Err(ToolError::NotFound(format!("Session {} not found", session_id)).into())
        }
    }

    /// Retrieve a value from session memory
    async fn retrieve_value(&self, session_id: Uuid, key: &str) -> Result<Option<Value>> {
        let mut storage = self.storage.write().await;
        
        if let Some(session) = storage.get_mut(&session_id) {
            *session.access_count.entry(key.to_string()).or_insert(0) += 1;
            session.last_accessed.insert(key.to_string(), Utc::now());
            
            Ok(session.data.get(key).cloned())
        } else {
            Err(ToolError::NotFound(format!("Session {} not found", session_id)).into())
        }
    }

    /// Export entire session data
    async fn export_session(&self, session_id: Uuid) -> Result<Value> {
        let storage = self.storage.read().await;
        
        if let Some(session) = storage.get(&session_id) {
            Ok(serde_json::json!({
                "session_id": session_id,
                "context": session.context,
                "data": session.data,
                "stats": {
                    "total_entries": session.data.len(),
                    "access_counts": session.access_count,
                    "last_accessed": session.last_accessed,
                }
            }))
        } else {
            Err(ToolError::NotFound(format!("Session {} not found", session_id)).into())
        }
    }

    /// Clear a session's data
    async fn clear_session(&self, session_id: Uuid) -> Result<()> {
        let mut storage = self.storage.write().await;
        
        if let Some(session) = storage.get_mut(&session_id) {
            session.data.clear();
            session.access_count.clear();
            session.last_accessed.clear();
            Ok(())
        } else {
            Err(ToolError::NotFound(format!("Session {} not found", session_id)).into())
        }
    }

    /// Get session statistics
    async fn get_session_stats(&self, session_id: Uuid) -> Result<MemoryStats> {
        let storage = self.storage.read().await;
        
        if let Some(session) = storage.get(&session_id) {
            let total_accesses: usize = session.access_count.values().sum();
            let total_entries = session.data.len();
            
            let hit_rate = if total_accesses > 0 {
                (total_accesses as f32) / (total_entries.max(1) as f32)
            } else {
                0.0
            };
            
            let memory_bytes = serde_json::to_string(&session.data)
                .map(|s| s.len())
                .unwrap_or(0);
            
            let last_accessed = session.last_accessed
                .values()
                .max()
                .copied();
            
            Ok(MemoryStats {
                total_entries,
                memory_bytes,
                last_accessed,
                hit_rate,
            })
        } else {
            Err(ToolError::NotFound(format!("Session {} not found", session_id)).into())
        }
    }
}

#[async_trait]
impl Tool for SessionMemory {
    type Input = SessionMemoryInput;
    type Output = SessionMemoryOutput;

    fn name(&self) -> &str {
        "session_memory"
    }

    fn description(&self) -> &str {
        "Manages session-level state and context for browser automation workflows"
    }

    async fn execute(&self, input: Self::Input) -> Result<Self::Output> {
        let session_id = self.get_or_create_session(input.session_id).await?;
        let timestamp = Utc::now();
        
        let (operation_name, result) = match input.operation {
            MemoryOperation::Store { key, value } => {
                self.store_value(session_id, key, value).await?;
                ("store", None)
            }
            MemoryOperation::Retrieve { key } => {
                let value = self.retrieve_value(session_id, &key).await?;
                ("retrieve", value)
            }
            MemoryOperation::Delete { key } => {
                let mut storage = self.storage.write().await;
                if let Some(session) = storage.get_mut(&session_id) {
                    let removed = session.data.remove(&key);
                    session.access_count.remove(&key);
                    session.last_accessed.remove(&key);
                    ("delete", removed)
                } else {
                    return Err(ToolError::NotFound(format!("Session {} not found", session_id)).into());
                }
            }
            MemoryOperation::ListKeys => {
                let storage = self.storage.read().await;
                if let Some(session) = storage.get(&session_id) {
                    let keys: Vec<String> = session.data.keys().cloned().collect();
                    ("list_keys", Some(serde_json::json!(keys)))
                } else {
                    return Err(ToolError::NotFound(format!("Session {} not found", session_id)).into());
                }
            }
            MemoryOperation::Clear => {
                self.clear_session(session_id).await?;
                ("clear", None)
            }
            MemoryOperation::GetStats => {
                let stats = self.get_session_stats(session_id).await?;
                ("get_stats", Some(serde_json::to_value(stats)?))
            }
            MemoryOperation::ExportSession => {
                let data = self.export_session(session_id).await?;
                ("export_session", Some(data))
            }
        };
        
        Ok(SessionMemoryOutput {
            success: true,
            operation: operation_name.to_string(),
            result,
            session_id,
            timestamp,
        })
    }
}

#[async_trait]
impl MemoryTool for SessionMemory {
    async fn store(&self, key: String, value: Value) -> Result<()> {
        let session_id = self.get_or_create_session(None).await?;
        self.store_value(session_id, key, value).await
    }

    async fn retrieve(&self, key: &str) -> Result<Option<Value>> {
        let storage = self.storage.read().await;
        
        // Search across all sessions (for convenience methods)
        for (_, session) in storage.iter() {
            if let Some(value) = session.data.get(key) {
                return Ok(Some(value.clone()));
            }
        }
        
        Ok(None)
    }

    async fn clear(&self) -> Result<()> {
        let mut storage = self.storage.write().await;
        storage.clear();
        Ok(())
    }

    async fn stats(&self) -> Result<MemoryStats> {
        let storage = self.storage.read().await;
        
        let mut total_entries = 0;
        let mut total_bytes = 0;
        let mut last_accessed: Option<DateTime<Utc>> = None;
        
        for (_, session) in storage.iter() {
            total_entries += session.data.len();
            total_bytes += serde_json::to_string(&session.data)
                .map(|s| s.len())
                .unwrap_or(0);
            
            if let Some(max_time) = session.last_accessed.values().max() {
                last_accessed = Some(match last_accessed {
                    Some(current) if current > *max_time => current,
                    _ => *max_time,
                });
            }
        }
        
        Ok(MemoryStats {
            total_entries,
            memory_bytes: total_bytes,
            last_accessed,
            hit_rate: 0.0, // Would need more tracking for global hit rate
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_session_memory_operations() {
        // Test will be implemented with mock browser
    }
}