// 自适应调度器 - 根据场景智能选择感知模式

use anyhow::Result;
use super::{PerceptionMode, Context, TaskType, Priority};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 自适应调度器
pub struct AdaptiveScheduler {
    // 使用模式统计
    usage_stats: Arc<RwLock<HashMap<TaskType, ModeStats>>>,
    
    // 性能历史
    performance_history: Arc<RwLock<Vec<PerformanceRecord>>>,
}

#[derive(Debug, Clone)]
struct ModeStats {
    mode_usage: HashMap<PerceptionMode, usize>,
    success_rate: HashMap<PerceptionMode, f32>,
    avg_duration: HashMap<PerceptionMode, u64>,
}

#[derive(Debug, Clone)]
struct PerformanceRecord {
    task_type: TaskType,
    mode: PerceptionMode,
    duration_ms: u64,
    success: bool,
}

impl AdaptiveScheduler {
    pub async fn new() -> Result<Self> {
        Ok(Self {
            usage_stats: Arc::new(RwLock::new(HashMap::new())),
            performance_history: Arc::new(RwLock::new(Vec::new())),
        })
    }
    
    /// 根据上下文选择最优感知模式
    pub async fn select_mode(&self, context: &Context) -> Result<PerceptionMode> {
        // 基于任务类型和优先级的启发式规则
        let mode = match (&context.task_type, &context.priority) {
            // 紧急任务使用快速模式
            (_, Priority::Critical) => PerceptionMode::Lightning,
            
            // 导航任务通常需要快速响应
            (TaskType::Navigation, Priority::High) => PerceptionMode::Quick,
            (TaskType::Navigation, _) => PerceptionMode::Standard,
            
            // 表单填充需要准确性
            (TaskType::FormFilling, _) => PerceptionMode::Standard,
            
            // 数据提取需要深度分析
            (TaskType::DataExtraction, Priority::Low) => PerceptionMode::Deep,
            (TaskType::DataExtraction, _) => PerceptionMode::Standard,
            
            // 交互任务需要平衡速度和准确性
            (TaskType::Interaction, Priority::High) => PerceptionMode::Quick,
            (TaskType::Interaction, _) => PerceptionMode::Standard,
            
            // 监控任务可以使用深度分析
            (TaskType::Monitoring, _) => PerceptionMode::Deep,
        };
        
        // 如果有时间约束，调整模式
        if let Some(time_limit) = context.time_constraint {
            let time_ms = time_limit.as_millis() as u64;
            if time_ms < 50 {
                return Ok(PerceptionMode::Lightning);
            } else if time_ms < 200 {
                return Ok(PerceptionMode::Quick);
            } else if time_ms < 500 {
                return Ok(PerceptionMode::Standard);
            }
        }
        
        // 基于历史性能数据优化选择
        if let Ok(optimized_mode) = self.optimize_based_on_history(context).await {
            return Ok(optimized_mode);
        }
        
        Ok(mode)
    }
    
    /// 基于历史数据优化模式选择
    async fn optimize_based_on_history(&self, context: &Context) -> Result<PerceptionMode> {
        let stats = self.usage_stats.read().await;
        
        if let Some(task_stats) = stats.get(&context.task_type) {
            // 选择成功率最高且满足时间要求的模式
            let mut best_mode = PerceptionMode::Standard;
            let mut best_score = 0.0;
            
            for (mode, success_rate) in &task_stats.success_rate {
                if let Some(avg_duration) = task_stats.avg_duration.get(mode) {
                    // 计算综合得分：成功率 * 速度因子
                    let speed_factor = 1000.0 / (*avg_duration as f32 + 1.0);
                    let score = success_rate * speed_factor;
                    
                    if score > best_score {
                        best_score = score;
                        best_mode = *mode;
                    }
                }
            }
            
            return Ok(best_mode);
        }
        
        // 没有历史数据，返回默认
        Ok(PerceptionMode::Standard)
    }
    
    /// 记录性能数据
    pub async fn record_performance(
        &self,
        task_type: TaskType,
        mode: PerceptionMode,
        duration_ms: u64,
        success: bool,
    ) -> Result<()> {
        // 添加到历史记录
        {
            let mut history = self.performance_history.write().await;
            history.push(PerformanceRecord {
                task_type: task_type.clone(),
                mode,
                duration_ms,
                success,
            });
            
            // 限制历史记录大小
            if history.len() > 1000 {
                history.drain(0..100);
            }
        }
        
        // 更新统计数据
        {
            let mut stats = self.usage_stats.write().await;
            let task_stats = stats.entry(task_type).or_insert_with(|| ModeStats {
                mode_usage: HashMap::new(),
                success_rate: HashMap::new(),
                avg_duration: HashMap::new(),
            });
            
            // 更新使用次数
            *task_stats.mode_usage.entry(mode).or_insert(0) += 1;
            
            // 更新成功率（简单移动平均）
            let current_rate = task_stats.success_rate.entry(mode).or_insert(0.0);
            *current_rate = (*current_rate * 0.9) + (if success { 0.1 } else { 0.0 });
            
            // 更新平均时长（简单移动平均）
            let current_avg = task_stats.avg_duration.entry(mode).or_insert(duration_ms);
            *current_avg = ((*current_avg * 9) + duration_ms) / 10;
        }
        
        Ok(())
    }
    
    /// 获取推荐配置
    pub async fn get_recommendations(&self) -> HashMap<TaskType, PerceptionMode> {
        let stats = self.usage_stats.read().await;
        let mut recommendations = HashMap::new();
        
        for (task_type, task_stats) in stats.iter() {
            // 找到该任务类型的最佳模式
            if let Some((best_mode, _)) = task_stats.success_rate.iter()
                .max_by(|a, b| a.1.partial_cmp(b.1).unwrap()) {
                recommendations.insert(task_type.clone(), *best_mode);
            }
        }
        
        recommendations
    }
}