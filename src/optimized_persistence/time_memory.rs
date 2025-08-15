// 时序记忆 - 经历和过程的轨迹

use anyhow::Result;
use std::sync::Arc;
use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};
use super::{SurrealClient, MemoryData, QueryCondition};

/// 时序记忆
pub struct TimeSeriesMemory {
    client: Arc<SurrealClient>,
    // 按时间排序的内存索引
    time_index: Arc<tokio::sync::RwLock<BTreeMap<std::time::SystemTime, uuid::Uuid>>>,
    // 数据统计
    stats: Arc<tokio::sync::RwLock<TimeSeriesStats>>,
}

/// 时序数据条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesEntry {
    pub id: uuid::Uuid,
    pub timestamp: std::time::SystemTime,
    pub data: MemoryData,
    pub sequence: u64,
    pub duration_ms: Option<u64>,
}

/// 时序统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesStats {
    pub total_entries: usize,
    pub time_span_hours: f32,
    pub avg_frequency_per_hour: f32,
    pub last_entry: Option<std::time::SystemTime>,
}

impl TimeSeriesMemory {
    pub async fn new(client: Arc<SurrealClient>) -> Result<Self> {
        Ok(Self { 
            client,
            time_index: Arc::new(tokio::sync::RwLock::new(BTreeMap::new())),
            stats: Arc::new(tokio::sync::RwLock::new(TimeSeriesStats {
                total_entries: 0,
                time_span_hours: 0.0,
                avg_frequency_per_hour: 0.0,
                last_entry: None,
            })),
        })
    }
    
    /// 存储时序数据
    pub async fn store(&self, memory: MemoryData) -> Result<()> {
        // 更新时间索引
        {
            let mut time_index = self.time_index.write().await;
            time_index.insert(memory.timestamp, memory.id);
        }
        
        // 更新统计信息
        {
            let mut stats = self.stats.write().await;
            stats.total_entries += 1;
            stats.last_entry = Some(memory.timestamp);
        }
        
        let time_data = serde_json::json!({
            "id": memory.id,
            "type": "timeseries",
            "timestamp": memory.timestamp,
            "data": memory,
        });
        
        self.client.store(memory.id, time_data).await
    }
    
    /// 查询时序数据
    pub async fn query(&self, condition: &QueryCondition) -> Result<Vec<MemoryData>> {
        // 使用客户端的查询功能
        let mut results = self.client.query_memories(condition).await?;
        
        // 按时间排序
        results.sort_by_key(|m| m.timestamp);
        
        Ok(results)
    }
    
    /// 获取时间范围内的数据
    pub async fn get_range(
        &self,
        start: std::time::SystemTime,
        end: std::time::SystemTime,
    ) -> Result<Vec<MemoryData>> {
        let condition = QueryCondition {
            session_id: None,
            data_type: None,
            time_range: Some((start, end)),
            keywords: vec![],
            limit: None,
        };
        
        self.query(&condition).await
    }
    
    /// 获取最近的N条记录
    pub async fn get_recent(&self, count: usize) -> Result<Vec<MemoryData>> {
        let condition = QueryCondition {
            session_id: None,
            data_type: None,
            time_range: None,
            keywords: vec![],
            limit: Some(count),
        };
        
        let mut results = self.query(&condition).await?;
        // 按时间倒序排列
        results.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        results.truncate(count);
        
        Ok(results)
    }
    
    /// 获取按时间统计的数据
    pub async fn get_hourly_stats(&self, hours: usize) -> Result<Vec<HourlyStats>> {
        let mut stats = Vec::new();
        let now = std::time::SystemTime::now();
        let hour_duration = std::time::Duration::from_secs(3600);
        
        for i in 0..hours {
            let end_time = now - hour_duration * i as u32;
            let start_time = end_time - hour_duration;
            
            let condition = QueryCondition {
                session_id: None,
                data_type: None,
                time_range: Some((start_time, end_time)),
                keywords: vec![],
                limit: None,
            };
            
            let entries = self.query(&condition).await?;
            
            stats.push(HourlyStats {
                hour: i,
                start_time,
                end_time,
                count: entries.len(),
                avg_size: if entries.is_empty() { 0 } else { 
                    entries.iter().map(|e| e.content.to_string().len()).sum::<usize>() / entries.len()
                },
            });
        }
        
        Ok(stats)
    }
    
    /// 获取时序统计
    pub async fn get_stats(&self) -> TimeSeriesStats {
        self.stats.read().await.clone()
    }
    
    /// 检测异常模式
    pub async fn detect_anomalies(&self, threshold_std_dev: f32) -> Result<Vec<AnomalyAlert>> {
        let recent = self.get_recent(1000).await?;
        let mut alerts = Vec::new();
        
        // 简单的异常检测逻辑
        if recent.len() > 100 {
            let avg_interval = self.calculate_avg_interval(&recent).await?;
            
            for window in recent.windows(10) {
                let window_interval = self.calculate_avg_interval(window).await?;
                if (window_interval - avg_interval).abs() > threshold_std_dev * avg_interval {
                    alerts.push(AnomalyAlert {
                        detected_at: std::time::SystemTime::now(),
                        anomaly_type: if window_interval > avg_interval {
                            AnomalyType::UnusualDelay
                        } else {
                            AnomalyType::UnusualBurst
                        },
                        severity: if (window_interval - avg_interval).abs() > 2.0 * threshold_std_dev * avg_interval {
                            Severity::High
                        } else {
                            Severity::Medium
                        },
                        description: format!("检测到时序异常: 平均间隔{:.2}s vs 正常{:.2}s", window_interval, avg_interval),
                    });
                }
            }
        }
        
        Ok(alerts)
    }
    
    async fn calculate_avg_interval(&self, memories: &[MemoryData]) -> Result<f32> {
        if memories.len() < 2 {
            return Ok(0.0);
        }
        
        let mut total_duration = 0.0;
        for window in memories.windows(2) {
            if let Ok(duration) = window[1].timestamp.duration_since(window[0].timestamp) {
                total_duration += duration.as_secs_f32();
            }
        }
        
        Ok(total_duration / (memories.len() - 1) as f32)
    }
}

/// 每小时统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HourlyStats {
    pub hour: usize,
    pub start_time: std::time::SystemTime,
    pub end_time: std::time::SystemTime,
    pub count: usize,
    pub avg_size: usize,
}

/// 异常警告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyAlert {
    pub detected_at: std::time::SystemTime,
    pub anomaly_type: AnomalyType,
    pub severity: Severity,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnomalyType {
    UnusualDelay,
    UnusualBurst,
    MissingData,
    CorruptedData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}