// 统一内核 - 系统的生命中枢
// 负责会话管理、状态维护、资源调度和健康监测

use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use anyhow::Result;
use std::time::Instant;

pub mod session_manager;
pub mod state_center;
pub mod health_guardian;
pub mod resource_manager;

use session_manager::SessionManager;
use state_center::StateCenter;
use health_guardian::HealthGuardian;
use resource_manager::ResourceManager;

/// 统一内核 - 生命体的中枢神经系统
pub struct UnifiedKernel {
    // 核心组件
    session_manager: Arc<SessionManager>,
    state_center: Arc<StateCenter>,
    health_guardian: Arc<HealthGuardian>,
    resource_manager: Arc<ResourceManager>,
    
    // 会话池优化(8.0新增)
    session_pool: Arc<RwLock<SessionPool>>,
    
    // 状态缓存(8.0新增)
    state_cache: Arc<RwLock<StateCache>>,
}

/// 会话配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    pub url: String,
    pub timeout: Option<u64>,
    pub max_retries: Option<u32>,
    pub perception_mode: PerceptionMode,
}

// 使用layered_perception模块的PerceptionMode
pub use crate::layered_perception::PerceptionMode;

/// 会话状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionState {
    Created,
    Active,
    Suspended,
    Completed,
    Failed(String),
}

/// 会话对象
#[derive(Debug, Clone)]
pub struct Session {
    pub id: Uuid,
    pub config: SessionConfig,
    pub state: SessionState,
    pub created_at: Instant,
    pub metadata: HashMap<String, String>,
}

impl Default for Session {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            config: SessionConfig::new("https://www.google.com"),
            state: SessionState::Created,
            created_at: Instant::now(),
            metadata: HashMap::new(),
        }
    }
}

impl Session {
    pub fn new(config: SessionConfig) -> Self {
        Self {
            id: Uuid::new_v4(),
            config,
            state: SessionState::Created,
            created_at: Instant::now(),
            metadata: HashMap::new(),
        }
    }
}

/// 会话池 - 8.0优化：减少创建开销
struct SessionPool {
    available: Vec<Session>,
    in_use: HashMap<Uuid, Session>,
    max_size: usize,
}

/// 状态缓存 - 8.0优化：快速状态访问
struct StateCache {
    cache: HashMap<Uuid, CachedState>,
    ttl: std::time::Duration,
}

#[derive(Clone)]
struct CachedState {
    data: SessionState,
    timestamp: Instant,
}

impl UnifiedKernel {
    /// 创建统一内核
    pub async fn new() -> Result<Self> {
        let session_manager = Arc::new(SessionManager::new().await?);
        let state_center = Arc::new(StateCenter::new().await?);
        let health_guardian = Arc::new(HealthGuardian::new().await?);
        let resource_manager = Arc::new(ResourceManager::new().await?);
        
        let session_pool = Arc::new(RwLock::new(SessionPool {
            available: Vec::new(),
            in_use: HashMap::new(),
            max_size: 100,
        }));
        
        let state_cache = Arc::new(RwLock::new(StateCache {
            cache: HashMap::new(),
            ttl: std::time::Duration::from_secs(60),
        }));
        
        Ok(Self {
            session_manager,
            state_center,
            health_guardian,
            resource_manager,
            session_pool,
            state_cache,
        })
    }
    
    /// 创建会话 - 生命周期的开始
    pub async fn create_session(&self, config: SessionConfig) -> Result<Session> {
        // 健康检查
        self.health_guardian.check_system_health().await?;
        
        // 从池中获取或创建新会话
        let session = {
            let mut pool = self.session_pool.write().await;
            if let Some(mut session) = pool.available.pop() {
                // 重用现有会话
                session.config = config;
                session.state = SessionState::Created;
                session.created_at = Instant::now();
                session
            } else {
                // 创建新会话
                Session {
                    id: Uuid::new_v4(),
                    config,
                    state: SessionState::Created,
                    created_at: Instant::now(),
                    metadata: HashMap::new(),
                }
            }
        };
        
        // 注册会话
        self.session_manager.register_session(&session).await?;
        
        // 分配资源
        self.resource_manager.allocate_for_session(&session).await?;
        
        // 更新状态
        self.state_center.update_session_state(&session.id, SessionState::Active).await?;
        
        Ok(session)
    }
    
    /// 获取会话状态
    pub async fn get_session_state(&self, session_id: &Uuid) -> Result<SessionState> {
        // 先检查缓存
        {
            let cache = self.state_cache.read().await;
            if let Some(cached) = cache.cache.get(session_id) {
                if cached.timestamp.elapsed() < cache.ttl {
                    return Ok(cached.data.clone());
                }
            }
        }
        
        // 从状态中心获取
        let state = self.state_center.get_session_state(session_id).await?;
        
        // 更新缓存
        {
            let mut cache = self.state_cache.write().await;
            cache.cache.insert(
                *session_id,
                CachedState {
                    data: state.clone(),
                    timestamp: Instant::now(),
                },
            );
        }
        
        Ok(state)
    }
    
    /// 暂停会话
    pub async fn suspend_session(&self, session_id: &Uuid) -> Result<()> {
        self.state_center.update_session_state(session_id, SessionState::Suspended).await?;
        self.resource_manager.release_session_resources(session_id).await?;
        Ok(())
    }
    
    /// 恢复会话
    pub async fn resume_session(&self, session_id: &Uuid) -> Result<()> {
        let session = self.session_manager.get_session(session_id).await?;
        self.resource_manager.allocate_for_session(&session).await?;
        self.state_center.update_session_state(session_id, SessionState::Active).await?;
        Ok(())
    }
    
    /// 销毁会话 - 生命周期的结束
    pub async fn destroy_session(&self, session_id: &Uuid) -> Result<()> {
        // 释放资源
        self.resource_manager.release_session_resources(session_id).await?;
        
        // 更新状态
        self.state_center.update_session_state(session_id, SessionState::Completed).await?;
        
        // 从管理器移除
        self.session_manager.unregister_session(session_id).await?;
        
        // 尝试回收到池中
        if let Ok(session) = self.session_manager.get_session(session_id).await {
            let mut pool = self.session_pool.write().await;
            if pool.available.len() < pool.max_size {
                pool.available.push(session);
            }
        }
        
        Ok(())
    }
    
    /// 获取系统健康状态
    pub async fn get_health_status(&self) -> Result<HealthStatus> {
        self.health_guardian.get_health_status().await
    }
    
    /// 批处理会话操作 - 8.0优化
    pub async fn batch_create_sessions(&self, configs: Vec<SessionConfig>) -> Result<Vec<Session>> {
        let mut sessions = Vec::new();
        
        // 预先分配资源
        self.resource_manager.pre_allocate_batch(configs.len()).await?;
        
        for config in configs {
            sessions.push(self.create_session(config).await?);
        }
        
        Ok(sessions)
    }
}

/// 健康状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub cpu_usage: f32,
    pub memory_usage: f32,
    pub active_sessions: usize,
    pub status: SystemStatus,
}

impl Default for HealthStatus {
    fn default() -> Self {
        Self {
            cpu_usage: 0.0,
            memory_usage: 0.0,
            active_sessions: 0,
            status: SystemStatus::Healthy,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SystemStatus {
    Healthy,
    Warning(String),
    Critical(String),
}

impl SessionConfig {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            timeout: Some(30000),
            max_retries: Some(3),
            perception_mode: PerceptionMode::Standard,
        }
    }
    
    pub fn with_perception_mode(mut self, mode: PerceptionMode) -> Self {
        self.perception_mode = mode;
        self
    }
}