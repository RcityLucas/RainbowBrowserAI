// Layered Perception - 四层感知架构实现
// 基于设计文档：分层感知系统，结合chromiumoxide的高级功能

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::timeout;
use tracing::{debug, info};
// Note: Some CDP features may not be available in chromiumoxide 0.5
// This is a design template - actual CDP access may need adjustment based on chromiumoxide version

use crate::browser::Browser;

/// 四层感知架构 - Lightning/Quick/Standard/Deep
pub struct LayeredPerception {
    browser: Arc<Browser>,
    cache: PerceptionCache,
    config: PerceptionConfig,
}

/// 感知配置
#[derive(Debug, Clone)]
pub struct PerceptionConfig {
    pub lightning_timeout: Duration, // <50ms
    pub quick_timeout: Duration,     // <200ms
    pub standard_timeout: Duration,  // <1000ms
    pub deep_timeout: Duration,      // <5000ms
    pub enable_cache: bool,
    pub cache_ttl: Duration,
    pub max_cache_size: usize,
}

impl Default for PerceptionConfig {
    fn default() -> Self {
        Self {
            lightning_timeout: Duration::from_millis(50),
            quick_timeout: Duration::from_millis(200),
            standard_timeout: Duration::from_millis(1000),
            deep_timeout: Duration::from_millis(5000),
            enable_cache: true,
            cache_ttl: Duration::from_secs(30),
            max_cache_size: 1000,
        }
    }
}

/// 感知缓存
#[derive(Debug, Clone)]
pub struct PerceptionCache {
    entries: HashMap<String, CacheEntry>,
    last_cleanup: Instant,
    config: PerceptionConfig,
}

#[derive(Debug, Clone)]
struct CacheEntry {
    data: PerceptionResult,
    created_at: Instant,
    access_count: usize,
}

/// Lightning - 极速感知层 (<50ms)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LightningPerception {
    /// 基础页面信息
    pub url: String,
    pub title: String,
    pub ready_state: String,

    /// 关键元素计数
    pub clickable_count: usize,
    pub input_count: usize,
    pub link_count: usize,
    pub form_count: usize,

    /// 响应时间统计
    pub perception_time_ms: u64,

    /// 缓存状态
    #[serde(skip)]
    pub from_cache: bool,
}

/// Quick - 快速感知层 (<200ms)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct QuickPerception {
    #[serde(flatten)]
    pub lightning: LightningPerception,

    /// 可交互元素列表
    pub interactive_elements: Vec<InteractiveElement>,
    pub visible_text_blocks: Vec<TextBlock>,
    pub form_fields: Vec<FormField>,

    /// 页面布局信息
    pub layout_info: LayoutInfo,
}

/// Standard - 标准感知层 (<1000ms)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StandardPerception {
    #[serde(flatten)]
    pub quick: QuickPerception,

    /// 语义分析结果
    pub semantic_structure: SemanticStructure,
    pub accessibility_info: AccessibilityInfo,
    pub computed_styles: HashMap<String, ComputedStyleInfo>,

    /// 页面性能指标
    pub performance_metrics: PerformanceMetrics,
}

/// Deep - 深度感知层 (<5000ms)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeepPerception {
    #[serde(flatten)]
    pub standard: StandardPerception,

    /// 完整的DOM树分析
    pub dom_analysis: DomAnalysis,
    pub visual_analysis: VisualAnalysis,
    pub behavioral_patterns: BehaviorPatterns,

    /// AI分析结果
    pub ai_insights: AiInsights,
}

/// 统一感知结果
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum PerceptionResult {
    Lightning(LightningPerception),
    Quick(QuickPerception),
    Standard(StandardPerception),
    Deep(DeepPerception),
}

/// 感知模式枚举
#[derive(Debug, Clone, Copy)]
pub enum PerceptionMode {
    Lightning, // 极速模式
    Quick,     // 快速模式
    Standard,  // 标准模式
    Deep,      // 深度模式
    Adaptive,  // 自适应模式
}

impl LayeredPerception {
    /// 创建新的分层感知实例
    pub fn new(browser: Arc<Browser>) -> Self {
        Self {
            browser,
            cache: PerceptionCache::new(PerceptionConfig::default()),
            config: PerceptionConfig::default(),
        }
    }

    /// 使用自定义配置创建
    pub fn with_config(browser: Arc<Browser>, config: PerceptionConfig) -> Self {
        Self {
            browser,
            cache: PerceptionCache::new(config.clone()),
            config,
        }
    }

    /// 主感知接口 - 自适应选择最佳模式
    pub async fn perceive(&mut self, mode: PerceptionMode) -> Result<PerceptionResult> {
        let _start_time = Instant::now();

        // 获取当前页面URL作为缓存键
        let cache_key = self.get_cache_key().await?;

        // 检查缓存
        if self.config.enable_cache {
            if let Some(cached) = self.cache.get(&cache_key) {
                debug!("Using cached perception result for {}", cache_key);
                return Ok(cached);
            }
        }

        // 根据模式执行相应的感知
        let result = match mode {
            PerceptionMode::Lightning => self
                .perceive_lightning()
                .await
                .map(PerceptionResult::Lightning),
            PerceptionMode::Quick => self.perceive_quick().await.map(PerceptionResult::Quick),
            PerceptionMode::Standard => self
                .perceive_standard()
                .await
                .map(PerceptionResult::Standard),
            PerceptionMode::Deep => self.perceive_deep().await.map(PerceptionResult::Deep),
            PerceptionMode::Adaptive => {
                // Inline adaptive logic to avoid recursion
                let complexity = self.estimate_page_complexity().await?;
                let selected_mode = match complexity {
                    PageComplexity::Simple => PerceptionMode::Lightning,
                    PageComplexity::Moderate => PerceptionMode::Quick,
                    PageComplexity::Complex => PerceptionMode::Standard,
                    PageComplexity::VeryComplex => PerceptionMode::Deep,
                };

                info!(
                    "Adaptive mode selected {:?} based on complexity {:?}",
                    selected_mode, complexity
                );

                // Call the appropriate perceive method directly
                match selected_mode {
                    PerceptionMode::Lightning => self
                        .perceive_lightning()
                        .await
                        .map(PerceptionResult::Lightning),
                    PerceptionMode::Quick => {
                        self.perceive_quick().await.map(PerceptionResult::Quick)
                    }
                    PerceptionMode::Standard => self
                        .perceive_standard()
                        .await
                        .map(PerceptionResult::Standard),
                    PerceptionMode::Deep => self.perceive_deep().await.map(PerceptionResult::Deep),
                    PerceptionMode::Adaptive => {
                        // Fallback to Quick to prevent infinite recursion
                        self.perceive_quick().await.map(PerceptionResult::Quick)
                    }
                }
            }
        }?;

        // 缓存结果
        if self.config.enable_cache {
            self.cache.set(cache_key, result.clone());
        }

        let elapsed = _start_time.elapsed();
        info!(
            "Perception completed in {:?} using mode {:?}",
            elapsed, mode
        );

        Ok(result)
    }

    /// Lightning感知 - 极速基础信息 (<50ms)
    async fn perceive_lightning(&self) -> Result<LightningPerception> {
        let _start_time = Instant::now();

        let perception_future = async {
            // 并行获取基础页面信息
            let (url, title, ready_state) = tokio::try_join!(
                self.browser.current_url(),
                self.get_page_title(),
                self.get_ready_state()
            )?;

            // 快速计算关键元素数量（使用优化的选择器）
            let (clickable_count, input_count, link_count, form_count) = tokio::try_join!(
                self.count_elements(
                    "button,input[type=button],input[type=submit],.btn,[role=button]"
                ),
                self.count_elements("input,textarea,select"),
                self.count_elements("a[href]"),
                self.count_elements("form")
            )?;

            Ok(LightningPerception {
                url,
                title,
                ready_state,
                clickable_count,
                input_count,
                link_count,
                form_count,
                perception_time_ms: _start_time.elapsed().as_millis() as u64,
                from_cache: false,
            })
        };

        // 应用超时限制
        timeout(self.config.lightning_timeout, perception_future)
            .await
            .map_err(|_| {
                anyhow!(
                    "Lightning perception timed out after {:?}",
                    self.config.lightning_timeout
                )
            })?
    }

    /// Quick感知 - 快速交互信息 (<200ms)
    async fn perceive_quick(&self) -> Result<QuickPerception> {
        let _start_time = Instant::now();

        let perception_future = async {
            // 先执行Lightning感知
            let lightning = self.perceive_lightning().await?;

            // 并行获取交互元素信息
            let (interactive_elements, visible_text_blocks, form_fields, layout_info) = tokio::try_join!(
                self.get_interactive_elements(),
                self.get_visible_text_blocks(),
                self.get_form_fields(),
                self.get_layout_info()
            )?;

            Ok(QuickPerception {
                lightning,
                interactive_elements,
                visible_text_blocks,
                form_fields,
                layout_info,
            })
        };

        timeout(self.config.quick_timeout, perception_future)
            .await
            .map_err(|_| {
                anyhow!(
                    "Quick perception timed out after {:?}",
                    self.config.quick_timeout
                )
            })?
    }

    /// Standard感知 - 标准语义分析 (<1000ms)
    async fn perceive_standard(&self) -> Result<StandardPerception> {
        let _start_time = Instant::now();

        let perception_future = async {
            // 先执行Quick感知
            let quick = self.perceive_quick().await?;

            // 并行获取语义和样式信息
            let (semantic_structure, accessibility_info, computed_styles, performance_metrics) = tokio::try_join!(
                self.analyze_semantic_structure(),
                self.get_accessibility_info(),
                self.get_computed_styles(),
                self.get_performance_metrics()
            )?;

            Ok(StandardPerception {
                quick,
                semantic_structure,
                accessibility_info,
                computed_styles,
                performance_metrics,
            })
        };

        timeout(self.config.standard_timeout, perception_future)
            .await
            .map_err(|_| {
                anyhow!(
                    "Standard perception timed out after {:?}",
                    self.config.standard_timeout
                )
            })?
    }

    /// Deep感知 - 深度全面分析 (<5000ms)
    async fn perceive_deep(&self) -> Result<DeepPerception> {
        let _start_time = Instant::now();

        let perception_future = async {
            // 先执行Standard感知
            let standard = self.perceive_standard().await?;

            // 并行执行深度分析
            let (dom_analysis, visual_analysis, behavioral_patterns, ai_insights) = tokio::try_join!(
                self.analyze_dom_structure(),
                self.analyze_visual_content(),
                self.analyze_behavioral_patterns(),
                self.generate_ai_insights()
            )?;

            Ok(DeepPerception {
                standard,
                dom_analysis,
                visual_analysis,
                behavioral_patterns,
                ai_insights,
            })
        };

        timeout(self.config.deep_timeout, perception_future)
            .await
            .map_err(|_| {
                anyhow!(
                    "Deep perception timed out after {:?}",
                    self.config.deep_timeout
                )
            })?
    }

    /// 自适应感知 - 根据场景自动选择最佳模式
    #[allow(dead_code)] // Used by adaptive mode selection
    async fn perceive_adaptive(&mut self) -> Result<PerceptionResult> {
        // 快速分析页面复杂度
        let complexity = self.estimate_page_complexity().await?;

        // 根据复杂度和性能要求选择合适的模式
        let selected_mode = match complexity {
            PageComplexity::Simple => PerceptionMode::Lightning,
            PageComplexity::Moderate => PerceptionMode::Quick,
            PageComplexity::Complex => PerceptionMode::Standard,
            PageComplexity::VeryComplex => PerceptionMode::Deep,
        };

        info!(
            "Adaptive mode selected {:?} based on complexity {:?}",
            selected_mode, complexity
        );
        self.perceive(selected_mode).await
    }

    // === 辅助方法实现 ===

    async fn get_cache_key(&self) -> Result<String> {
        let url = self.browser.current_url().await?;
        // 使用URL和时间戳创建缓存键
        Ok(format!("{}:{}", url, chrono::Utc::now().timestamp() / 30)) // 30秒缓存窗口
    }

    async fn get_page_title(&self) -> Result<String> {
        let script = "document.title || 'Unknown'";
        let result = self.browser.execute_script(script).await?;
        Ok(result.as_str().unwrap_or("Unknown").to_string())
    }

    async fn get_ready_state(&self) -> Result<String> {
        let script = "document.readyState";
        let result = self.browser.execute_script(script).await?;
        Ok(result.as_str().unwrap_or("unknown").to_string())
    }

    async fn count_elements(&self, selector: &str) -> Result<usize> {
        let script = format!("document.querySelectorAll('{}').length", selector);
        let result = self.browser.execute_script(&script).await?;
        Ok(result.as_u64().unwrap_or(0) as usize)
    }

    async fn estimate_page_complexity(&self) -> Result<PageComplexity> {
        // 快速估算页面复杂度
        let script = r#"
            (function() {
                const dom_size = document.all.length;
                const forms = document.forms.length;
                const interactive = document.querySelectorAll('button,input,select,textarea,a').length;
                const scripts = document.scripts.length;
                
                return {
                    dom_size: dom_size,
                    forms: forms,
                    interactive: interactive,
                    scripts: scripts,
                    score: dom_size * 0.1 + interactive * 2 + forms * 5 + scripts * 1
                };
            })()
        "#;

        let result = self.browser.execute_script(script).await?;
        let score = result["score"].as_f64().unwrap_or(0.0);

        Ok(match score {
            s if s < 50.0 => PageComplexity::Simple,
            s if s < 150.0 => PageComplexity::Moderate,
            s if s < 300.0 => PageComplexity::Complex,
            _ => PageComplexity::VeryComplex,
        })
    }

    // TODO: 实现其他辅助方法
    async fn get_interactive_elements(&self) -> Result<Vec<InteractiveElement>> {
        // 实现获取交互元素的逻辑
        Ok(vec![])
    }

    async fn get_visible_text_blocks(&self) -> Result<Vec<TextBlock>> {
        Ok(vec![])
    }

    async fn get_form_fields(&self) -> Result<Vec<FormField>> {
        Ok(vec![])
    }

    async fn get_layout_info(&self) -> Result<LayoutInfo> {
        Ok(LayoutInfo::default())
    }

    async fn analyze_semantic_structure(&self) -> Result<SemanticStructure> {
        Ok(SemanticStructure::default())
    }

    async fn get_accessibility_info(&self) -> Result<AccessibilityInfo> {
        Ok(AccessibilityInfo::default())
    }

    async fn get_computed_styles(&self) -> Result<HashMap<String, ComputedStyleInfo>> {
        Ok(HashMap::new())
    }

    async fn get_performance_metrics(&self) -> Result<PerformanceMetrics> {
        Ok(PerformanceMetrics::default())
    }

    async fn analyze_dom_structure(&self) -> Result<DomAnalysis> {
        // Deep DOM analysis with full tree traversal
        let script = r#"
            function analyzeDom() {
                let nodeCount = 0;
                let maxDepth = 0;
                let interactiveNodes = 0;
                
                function traverse(node, depth) {
                    nodeCount++;
                    maxDepth = Math.max(maxDepth, depth);
                    
                    // Check if node is interactive
                    if (node.nodeType === 1) {
                        const tagName = node.tagName.toLowerCase();
                        const interactiveTags = ['a', 'button', 'input', 'select', 'textarea', 'form'];
                        if (interactiveTags.includes(tagName) || node.onclick || node.getAttribute('role') === 'button') {
                            interactiveNodes++;
                        }
                    }
                    
                    // Traverse children
                    for (let child of node.childNodes) {
                        traverse(child, depth + 1);
                    }
                }
                
                traverse(document.documentElement, 0);
                
                return {
                    total_nodes: nodeCount,
                    max_depth: maxDepth,
                    interactive_nodes: interactiveNodes
                };
            }
            analyzeDom();
        "#;

        let result = self.browser.execute_script(script).await?;

        Ok(DomAnalysis {
            total_nodes: result["total_nodes"].as_u64().unwrap_or(0) as u32,
            max_depth: result["max_depth"].as_u64().unwrap_or(0) as u32,
            interactive_nodes: result["interactive_nodes"].as_u64().unwrap_or(0) as u32,
        })
    }

    async fn analyze_visual_content(&self) -> Result<VisualAnalysis> {
        // Visual content analysis including colors and elements
        let script = r#"
            function analyzeVisual() {
                // Get dominant colors from computed styles
                const elements = document.querySelectorAll('*');
                const colors = new Set();
                const visualElements = new Set();
                
                for (let el of elements) {
                    const style = window.getComputedStyle(el);
                    
                    // Collect colors
                    if (style.backgroundColor && style.backgroundColor !== 'rgba(0, 0, 0, 0)') {
                        colors.add(style.backgroundColor);
                    }
                    if (style.color) {
                        colors.add(style.color);
                    }
                    
                    // Identify visual elements
                    if (el.tagName === 'IMG') visualElements.add('image');
                    if (el.tagName === 'VIDEO') visualElements.add('video');
                    if (el.tagName === 'CANVAS') visualElements.add('canvas');
                    if (el.tagName === 'SVG') visualElements.add('svg');
                    if (style.backgroundImage && style.backgroundImage !== 'none') {
                        visualElements.add('background-image');
                    }
                }
                
                // Create a simple hash based on page content
                const content = document.documentElement.innerHTML;
                let hash = 0;
                for (let i = 0; i < Math.min(content.length, 1000); i++) {
                    hash = ((hash << 5) - hash) + content.charCodeAt(i);
                    hash = hash & hash;
                }
                
                return {
                    screenshot_hash: Math.abs(hash).toString(16),
                    color_palette: Array.from(colors).slice(0, 10),
                    visual_elements: Array.from(visualElements)
                };
            }
            analyzeVisual();
        "#;

        let result = self.browser.execute_script(script).await?;

        Ok(VisualAnalysis {
            screenshot_hash: result["screenshot_hash"].as_str().unwrap_or("").to_string(),
            color_palette: result["color_palette"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
            visual_elements: result["visual_elements"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
        })
    }

    async fn analyze_behavioral_patterns(&self) -> Result<BehaviorPatterns> {
        // Analyze potential user flows and interaction patterns
        let script = r#"
            function analyzeBehavior() {
                const forms = document.querySelectorAll('form');
                const links = document.querySelectorAll('a[href]');
                const buttons = document.querySelectorAll('button, [role="button"]');
                
                // Identify user flows
                const userFlows = [];
                if (forms.length > 0) {
                    userFlows.push('form-submission');
                }
                if (links.length > 10) {
                    userFlows.push('navigation-heavy');
                }
                if (buttons.length > 5) {
                    userFlows.push('interactive-actions');
                }
                
                // Find interaction hotspots
                const hotspots = [];
                
                // Check header area
                const header = document.querySelector('header, [role="banner"], .header, #header');
                if (header) {
                    const headerButtons = header.querySelectorAll('button, a');
                    if (headerButtons.length > 0) {
                        hotspots.push('header-navigation');
                    }
                }
                
                // Check main content area
                const main = document.querySelector('main, [role="main"], .main, #main');
                if (main) {
                    const mainInteractive = main.querySelectorAll('button, input, a');
                    if (mainInteractive.length > 0) {
                        hotspots.push('main-content-interaction');
                    }
                }
                
                // Check footer area
                const footer = document.querySelector('footer, [role="contentinfo"], .footer, #footer');
                if (footer) {
                    hotspots.push('footer-links');
                }
                
                return {
                    user_flows: userFlows,
                    interaction_hotspots: hotspots
                };
            }
            analyzeBehavior();
        "#;

        let result = self.browser.execute_script(script).await?;

        Ok(BehaviorPatterns {
            user_flows: result["user_flows"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
            interaction_hotspots: result["interaction_hotspots"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
        })
    }

    async fn generate_ai_insights(&self) -> Result<AiInsights> {
        // Generate AI-based insights about the page
        let script = r#"
            function generateInsights() {
                // Analyze page purpose
                const title = document.title.toLowerCase();
                const h1 = document.querySelector('h1');
                const forms = document.querySelectorAll('form').length;
                const links = document.querySelectorAll('a').length;
                const images = document.querySelectorAll('img').length;
                
                let pagePurpose = 'general-content';
                const recommendations = [];
                
                // Determine page purpose
                if (forms > 0) {
                    if (title.includes('login') || title.includes('sign')) {
                        pagePurpose = 'authentication';
                        recommendations.push('Fill and submit login form');
                    } else if (title.includes('search')) {
                        pagePurpose = 'search';
                        recommendations.push('Enter search query');
                    } else {
                        pagePurpose = 'data-collection';
                        recommendations.push('Complete form fields');
                    }
                } else if (links > 20) {
                    pagePurpose = 'navigation-hub';
                    recommendations.push('Navigate to relevant section');
                } else if (images > 10) {
                    pagePurpose = 'media-gallery';
                    recommendations.push('Browse visual content');
                } else if (title.includes('article') || title.includes('blog')) {
                    pagePurpose = 'article';
                    recommendations.push('Read main content');
                }
                
                // Calculate usability score
                let usabilityScore = 0.5;
                
                // Positive factors
                if (h1) usabilityScore += 0.1;
                if (document.querySelector('nav')) usabilityScore += 0.1;
                if (forms > 0 && forms < 3) usabilityScore += 0.1;
                
                // Negative factors
                if (links > 100) usabilityScore -= 0.1;
                if (forms > 5) usabilityScore -= 0.1;
                
                // Clamp between 0 and 1
                usabilityScore = Math.max(0, Math.min(1, usabilityScore));
                
                // Add contextual recommendations
                if (forms > 0) {
                    recommendations.push('Validate form inputs before submission');
                }
                if (links > 50) {
                    recommendations.push('Use search or navigation menu for efficiency');
                }
                
                return {
                    page_purpose: pagePurpose,
                    recommended_actions: recommendations,
                    usability_score: usabilityScore
                };
            }
            generateInsights();
        "#;

        let result = self.browser.execute_script(script).await?;

        Ok(AiInsights {
            page_purpose: result["page_purpose"]
                .as_str()
                .unwrap_or("unknown")
                .to_string(),
            recommended_actions: result["recommended_actions"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
            usability_score: result["usability_score"].as_f64().unwrap_or(0.5),
        })
    }
}

impl PerceptionCache {
    fn new(config: PerceptionConfig) -> Self {
        Self {
            entries: HashMap::new(),
            last_cleanup: Instant::now(),
            config,
        }
    }

    fn get(&mut self, key: &str) -> Option<PerceptionResult> {
        self.cleanup_if_needed();

        if let Some(entry) = self.entries.get_mut(key) {
            if entry.created_at.elapsed() <= self.config.cache_ttl {
                entry.access_count += 1;
                return Some(entry.data.clone());
            }
        }
        None
    }

    fn set(&mut self, key: String, data: PerceptionResult) {
        self.cleanup_if_needed();

        if self.entries.len() >= self.config.max_cache_size {
            self.evict_lru();
        }

        self.entries.insert(
            key,
            CacheEntry {
                data,
                created_at: Instant::now(),
                access_count: 1,
            },
        );
    }

    fn cleanup_if_needed(&mut self) {
        if self.last_cleanup.elapsed() > Duration::from_secs(60) {
            self.cleanup_expired();
            self.last_cleanup = Instant::now();
        }
    }

    fn cleanup_expired(&mut self) {
        let ttl = self.config.cache_ttl;
        self.entries
            .retain(|_, entry| entry.created_at.elapsed() <= ttl);
    }

    fn evict_lru(&mut self) {
        if let Some(lru_key) = self
            .entries
            .iter()
            .min_by_key(|(_, entry)| entry.access_count)
            .map(|(key, _)| key.clone())
        {
            self.entries.remove(&lru_key);
        }
    }
}

// === 数据结构定义 ===

#[derive(Debug, Clone, Copy)]
enum PageComplexity {
    Simple,
    Moderate,
    Complex,
    VeryComplex,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct InteractiveElement {
    pub selector: String,
    pub element_type: String,
    pub text: String,
    pub is_visible: bool,
    pub bounds: ElementBounds,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct TextBlock {
    pub content: String,
    pub tag_name: String,
    pub is_heading: bool,
    pub font_size: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct FormField {
    pub name: String,
    pub field_type: String,
    pub required: bool,
    pub placeholder: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct LayoutInfo {
    pub viewport_width: u32,
    pub viewport_height: u32,
    pub content_width: u32,
    pub content_height: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ElementBounds {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct SemanticStructure {
    pub headings: Vec<String>,
    pub main_content: String,
    pub navigation: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct AccessibilityInfo {
    pub aria_labels: Vec<String>,
    pub alt_texts: Vec<String>,
    pub accessibility_violations: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ComputedStyleInfo {
    pub display: String,
    pub visibility: String,
    pub z_index: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct PerformanceMetrics {
    pub load_time: f64,
    pub dom_ready_time: f64,
    pub resource_count: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct DomAnalysis {
    pub total_nodes: u32,
    pub max_depth: u32,
    pub interactive_nodes: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct VisualAnalysis {
    pub screenshot_hash: String,
    pub color_palette: Vec<String>,
    pub visual_elements: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct BehaviorPatterns {
    pub user_flows: Vec<String>,
    pub interaction_hotspots: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct AiInsights {
    pub page_purpose: String,
    pub recommended_actions: Vec<String>,
    pub usability_score: f64,
}
