// 状态中心 - 管理所有会话和系统状态

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use anyhow::{Result, anyhow};
use super::SessionState;

/// 状态中心
pub struct StateCenter {
    session_states: Arc<RwLock<HashMap<Uuid, SessionState>>>,
    system_state: Arc<RwLock<SystemState>>,
}

#[derive(Debug, Clone)]
pub struct SystemState {
    pub is_running: bool,
    pub total_sessions_created: u64,
    pub total_sessions_completed: u64,
    pub total_sessions_failed: u64,
}

impl StateCenter {
    pub async fn new() -> Result<Self> {
        Ok(Self {
            session_states: Arc::new(RwLock::new(HashMap::new())),
            system_state: Arc::new(RwLock::new(SystemState {
                is_running: true,
                total_sessions_created: 0,
                total_sessions_completed: 0,
                total_sessions_failed: 0,
            })),
        })
    }
    
    /// 更新会话状态
    pub async fn update_session_state(&self, session_id: &Uuid, state: SessionState) -> Result<()> {
        let mut states = self.session_states.write().await;
        
        // 更新系统统计
        match &state {
            SessionState::Created => {
                let mut sys_state = self.system_state.write().await;
                sys_state.total_sessions_created += 1;
            }
            SessionState::Completed => {
                let mut sys_state = self.system_state.write().await;
                sys_state.total_sessions_completed += 1;
            }
            SessionState::Failed(_) => {
                let mut sys_state = self.system_state.write().await;
                sys_state.total_sessions_failed += 1;
            }
            _ => {}
        }
        
        states.insert(*session_id, state);
        Ok(())
    }
    
    /// 获取会话状态
    pub async fn get_session_state(&self, session_id: &Uuid) -> Result<SessionState> {
        let states = self.session_states.read().await;
        states.get(session_id)
            .cloned()
            .ok_or_else(|| anyhow!("会话状态不存在: {}", session_id))
    }
    
    /// 获取系统状态
    pub async fn get_system_state(&self) -> Result<SystemState> {
        let state = self.system_state.read().await;
        Ok(state.clone())
    }
    
    /// 清理已完成的会话状态
    pub async fn cleanup_completed_sessions(&self) -> Result<usize> {
        let mut states = self.session_states.write().await;
        let before_count = states.len();
        
        states.retain(|_, state| {
            !matches!(state, SessionState::Completed | SessionState::Failed(_))
        });
        
        let removed = before_count - states.len();
        Ok(removed)
    }
}