// 资源管理器 - 管理和分配系统资源

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use anyhow::{Result, anyhow};
use super::Session;

/// 资源管理器
pub struct ResourceManager {
    allocations: Arc<RwLock<HashMap<Uuid, ResourceAllocation>>>,
    resource_pool: Arc<RwLock<ResourcePool>>,
}

#[derive(Debug, Clone)]
struct ResourceAllocation {
    session_id: Uuid,
    memory_mb: u64,
    cpu_cores: f32,
    bandwidth_mbps: f32,
    allocated_at: std::time::Instant,
}

#[derive(Debug)]
struct ResourcePool {
    total_memory_mb: u64,
    available_memory_mb: u64,
    total_cpu_cores: f32,
    available_cpu_cores: f32,
    total_bandwidth_mbps: f32,
    available_bandwidth_mbps: f32,
}

impl ResourceManager {
    pub async fn new() -> Result<Self> {
        // 获取系统资源
        let total_memory_mb = 8192; // 8GB 默认值
        let total_cpu_cores = 4.0;  // 4核 默认值
        let total_bandwidth_mbps = 100.0; // 100Mbps 默认值
        
        Ok(Self {
            allocations: Arc::new(RwLock::new(HashMap::new())),
            resource_pool: Arc::new(RwLock::new(ResourcePool {
                total_memory_mb,
                available_memory_mb: total_memory_mb,
                total_cpu_cores,
                available_cpu_cores: total_cpu_cores,
                total_bandwidth_mbps,
                available_bandwidth_mbps: total_bandwidth_mbps,
            })),
        })
    }
    
    /// 为会话分配资源
    pub async fn allocate_for_session(&self, session: &Session) -> Result<()> {
        let mut pool = self.resource_pool.write().await;
        let mut allocations = self.allocations.write().await;
        
        // 根据感知模式确定资源需求
        let (memory_mb, cpu_cores, bandwidth_mbps) = match session.config.perception_mode {
            super::PerceptionMode::Lightning => (256, 0.5, 10.0),
            super::PerceptionMode::Quick => (512, 1.0, 20.0),
            super::PerceptionMode::Standard => (1024, 2.0, 50.0),
            super::PerceptionMode::Deep => (2048, 4.0, 100.0),
        };
        
        // 检查资源是否足够
        if pool.available_memory_mb < memory_mb {
            return Err(anyhow!("内存资源不足"));
        }
        if pool.available_cpu_cores < cpu_cores {
            return Err(anyhow!("CPU资源不足"));
        }
        if pool.available_bandwidth_mbps < bandwidth_mbps {
            return Err(anyhow!("带宽资源不足"));
        }
        
        // 分配资源
        pool.available_memory_mb -= memory_mb;
        pool.available_cpu_cores -= cpu_cores;
        pool.available_bandwidth_mbps -= bandwidth_mbps;
        
        allocations.insert(session.id, ResourceAllocation {
            session_id: session.id,
            memory_mb,
            cpu_cores,
            bandwidth_mbps,
            allocated_at: std::time::Instant::now(),
        });
        
        Ok(())
    }
    
    /// 释放会话资源
    pub async fn release_session_resources(&self, session_id: &Uuid) -> Result<()> {
        let mut pool = self.resource_pool.write().await;
        let mut allocations = self.allocations.write().await;
        
        if let Some(allocation) = allocations.remove(session_id) {
            // 归还资源到池中
            pool.available_memory_mb += allocation.memory_mb;
            pool.available_cpu_cores += allocation.cpu_cores;
            pool.available_bandwidth_mbps += allocation.bandwidth_mbps;
            Ok(())
        } else {
            Err(anyhow!("会话资源分配不存在: {}", session_id))
        }
    }
    
    /// 预分配批量资源
    pub async fn pre_allocate_batch(&self, count: usize) -> Result<()> {
        let pool = self.resource_pool.read().await;
        
        // 估算批量资源需求（按标准模式）
        let required_memory = 1024 * count as u64;
        let required_cpu = 2.0 * count as f32;
        
        if pool.available_memory_mb < required_memory {
            return Err(anyhow!("批量分配内存不足"));
        }
        if pool.available_cpu_cores < required_cpu {
            return Err(anyhow!("批量分配CPU不足"));
        }
        
        Ok(())
    }
    
    /// 获取资源使用情况
    pub async fn get_resource_usage(&self) -> Result<ResourceUsage> {
        let pool = self.resource_pool.read().await;
        let allocations = self.allocations.read().await;
        
        Ok(ResourceUsage {
            total_memory_mb: pool.total_memory_mb,
            used_memory_mb: pool.total_memory_mb - pool.available_memory_mb,
            total_cpu_cores: pool.total_cpu_cores,
            used_cpu_cores: pool.total_cpu_cores - pool.available_cpu_cores,
            total_bandwidth_mbps: pool.total_bandwidth_mbps,
            used_bandwidth_mbps: pool.total_bandwidth_mbps - pool.available_bandwidth_mbps,
            active_allocations: allocations.len(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct ResourceUsage {
    pub total_memory_mb: u64,
    pub used_memory_mb: u64,
    pub total_cpu_cores: f32,
    pub used_cpu_cores: f32,
    pub total_bandwidth_mbps: f32,
    pub used_bandwidth_mbps: f32,
    pub active_allocations: usize,
}