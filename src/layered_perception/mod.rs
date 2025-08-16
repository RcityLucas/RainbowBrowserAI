// 分层感知 - AI生命体的感官系统
// 实现四层感知架构：Lightning、Quick、Standard、Deep

use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

pub mod lightning;
pub mod quick;
pub mod standard;
pub mod deep;
pub mod adaptive;
pub mod dom_analyzer;
pub mod strategy;

use lightning::LightningPerception;
use quick::QuickPerception;
use standard::StandardPerception;
use deep::DeepPerception;
use adaptive::AdaptiveScheduler;
use strategy::PerceptionStrategyFactory;

/// 感知模式
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PerceptionMode {
    Lightning,  // <50ms - 本能反应
    Quick,      // <200ms - 感官知觉
    Standard,   // <500ms - 认知理解
    Deep,       // <1000ms - 智慧洞察
}

/// 分层感知系统 - 四层感知架构
pub struct LayeredPerception {
    // Strategy factory for perception modes
    strategy_factory: Arc<PerceptionStrategyFactory>,
    
    // 自适应调度器
    scheduler: Arc<AdaptiveScheduler>,
    
    // 感知缓存
    cache: Arc<RwLock<PerceptionCache>>,
}

/// 感知结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerceptionResult {
    pub mode: PerceptionMode,
    pub duration_ms: u64,
    pub timestamp: std::time::SystemTime,
    pub data: PerceptionData,
}

/// 感知数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerceptionData {
    Lightning(LightningData),
    Quick(QuickData),
    Standard(StandardData),
    Deep(DeepData),
}

/// Lightning层感知数据 - 极速感知
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightningData {
    pub key_elements: Vec<KeyElement>,      // ≤10个关键元素
    pub page_status: PageStatus,            // 页面状态
    pub urgent_signals: Vec<Signal>,        // 紧急信号
}

/// Quick层感知数据 - 快速感知
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuickData {
    pub lightning_data: LightningData,      // 包含Lightning层数据
    pub interaction_elements: Vec<Element>, // 可交互元素
    pub layout_structure: LayoutInfo,       // 布局结构
    pub navigation_paths: Vec<NavPath>,     // 导航路径
}

/// Standard层感知数据 - 标准感知
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardData {
    pub quick_data: QuickData,              // 包含Quick层数据
    pub content_analysis: ContentAnalysis,  // 内容分析
    pub form_structures: Vec<FormInfo>,     // 表单结构
    pub media_elements: Vec<MediaInfo>,     // 媒体元素
}

/// Deep层感知数据 - 深度感知
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepData {
    pub standard_data: StandardData,        // 包含Standard层数据
    pub semantic_analysis: SemanticResult,  // 语义分析
    pub interaction_graph: InteractionGraph,// 交互关系图
    pub page_model: PageModel,             // 完整页面模型
    pub temporal_patterns: Vec<Pattern>,    // 时序模式
}

/// 关键元素
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyElement {
    pub selector: String,
    pub element_type: ElementType,
    pub importance: f32,
}

/// 元素类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ElementType {
    Button,
    Link,
    Input,
    Form,
    Navigation,
    Content,
    Media,
    Other,
}

/// 页面状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PageStatus {
    Loading,
    Ready,
    Interactive,
    Error(String),
    Unknown,
}

/// 信号
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signal {
    pub signal_type: SignalType,
    pub priority: Priority,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SignalType {
    Alert,
    Popup,
    Redirect,
    Error,
    Success,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub enum Priority {
    Critical,
    High,
    Medium,
    Low,
}

/// 感知缓存
struct PerceptionCache {
    entries: HashMap<String, CachedPerception>,
    max_size: usize,
    ttl: Duration,
}

struct CachedPerception {
    result: PerceptionResult,
    cached_at: Instant,
}

use std::collections::HashMap;

impl LayeredPerception {
    /// 创建分层感知系统
    pub async fn new() -> Result<Self> {
        Ok(Self {
            strategy_factory: Arc::new(PerceptionStrategyFactory::new().await?),
            scheduler: Arc::new(AdaptiveScheduler::new().await?),
            cache: Arc::new(RwLock::new(PerceptionCache {
                entries: HashMap::new(),
                max_size: 100,
                ttl: Duration::from_secs(60),
            })),
        })
    }
    
    /// 执行感知
    pub async fn perceive(&self, url: &str, mode: PerceptionMode) -> Result<PerceptionResult> {
        // 检查缓存
        let cache_key = format!("{}:{:?}", url, mode);
        if let Some(cached) = self.get_from_cache(&cache_key).await {
            return Ok(cached);
        }
        
        let start = Instant::now();
        
        // Use strategy pattern instead of hard-coded enum dispatch
        let strategy = self.strategy_factory.get_strategy(mode)
            .ok_or_else(|| anyhow::anyhow!("No strategy found for mode: {:?}", mode))?;
        
        let data = strategy.perceive(url).await?;
        
        let duration_ms = start.elapsed().as_millis() as u64;
        
        // 验证时间约束
        let max_duration = strategy.max_duration_ms();
        
        if duration_ms > max_duration {
            log::warn!("{:?}模式感知超时: {}ms > {}ms", mode, duration_ms, max_duration);
        }
        
        let result = PerceptionResult {
            mode,
            duration_ms,
            timestamp: std::time::SystemTime::now(),
            data,
        };
        
        // 存入缓存
        self.cache_result(&cache_key, result.clone()).await;
        
        Ok(result)
    }
    
    /// 自适应感知 - 根据场景自动选择最优模式
    pub async fn adaptive_perceive(&self, url: &str, context: &Context) -> Result<PerceptionResult> {
        let mode = self.scheduler.select_mode(context).await?;
        self.perceive(url, mode).await
    }
    
    /// 获取缓存
    async fn get_from_cache(&self, key: &str) -> Option<PerceptionResult> {
        let cache = self.cache.read().await;
        cache.entries.get(key).and_then(|entry| {
            if entry.cached_at.elapsed() < cache.ttl {
                Some(entry.result.clone())
            } else {
                None
            }
        })
    }
    
    /// 缓存结果
    async fn cache_result(&self, key: &str, result: PerceptionResult) {
        let mut cache = self.cache.write().await;
        
        // 如果缓存满了，移除最旧的条目
        if cache.entries.len() >= cache.max_size {
            if let Some(oldest_key) = cache.entries.iter()
                .min_by_key(|(_, v)| v.cached_at)
                .map(|(k, _)| k.clone()) {
                cache.entries.remove(&oldest_key);
            }
        }
        
        cache.entries.insert(key.to_string(), CachedPerception {
            result,
            cached_at: Instant::now(),
        });
    }
}

/// 感知上下文
#[derive(Debug, Clone)]
pub struct Context {
    pub task_type: TaskType,
    pub priority: Priority,
    pub time_constraint: Option<Duration>,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum TaskType {
    Navigation,
    FormFilling,
    DataExtraction,
    Interaction,
    Monitoring,
}

// 占位类型定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Element {
    pub selector: String,
    pub element_type: ElementType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutInfo {
    pub structure: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavPath {
    pub path: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentAnalysis {
    pub text_content: String,
    pub keywords: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormInfo {
    pub form_id: String,
    pub fields: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaInfo {
    pub media_type: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticResult {
    pub entities: Vec<String>,
    pub topics: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionGraph {
    pub nodes: Vec<String>,
    pub edges: Vec<(String, String)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageModel {
    pub structure: String,
    pub components: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pattern {
    pub pattern_type: String,
    pub frequency: f32,
}