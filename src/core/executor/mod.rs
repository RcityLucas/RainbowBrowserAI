//! # 智能执行器 - LLM驱动的任务执行
//! 
//! 结合LLM智能分析和浏览器控制，实现真正智能的任务执行

use std::collections::HashMap;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use super::llm::{SmartIntentAnalyzer, SmartIntent, TaskStep, LLMConfig};

/// 智能执行器
pub struct SmartExecutor {
    intent_analyzer: SmartIntentAnalyzer,
    browser_manager: BrowserManager,
    execution_history: Vec<ExecutionRecord>,
}

/// 浏览器管理器 (简化版本)
pub struct BrowserManager {
    sessions: HashMap<String, BrowserSession>,
    session_counter: u32,
}

/// 浏览器会话
pub struct BrowserSession {
    pub id: String,
    pub current_url: String,
    pub actions: Vec<String>,
    pub extracted_data: HashMap<String, String>,
}

/// 执行记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRecord {
    pub user_input: String,
    pub intent: SmartIntent,
    pub execution_steps: Vec<ExecutionStep>,
    pub final_result: String,
    pub success: bool,
    pub duration_ms: u64,
    pub timestamp: String,
}

/// 执行步骤
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStep {
    pub step_name: String,
    pub action: String,
    pub target: String,
    pub result: String,
    pub success: bool,
    pub duration_ms: u64,
}

/// 智能执行结果
#[derive(Debug, Serialize, Deserialize)]
pub struct SmartExecutionResult {
    pub success: bool,
    pub user_intent: SmartIntent,
    pub execution_summary: String,
    pub extracted_data: HashMap<String, Vec<String>>,
    pub recommendations: Vec<String>,
    pub next_actions: Vec<String>,
    pub llm_response: String,
}

impl BrowserSession {
    pub fn new(id: String) -> Self {
        Self {
            id,
            current_url: "about:blank".to_string(),
            actions: Vec::new(),
            extracted_data: HashMap::new(),
        }
    }
    
    pub async fn navigate(&mut self, url: &str) -> Result<(), String> {
        println!("🌐 [{}] 导航到: {}", self.id, url);
        // 模拟网络延迟
        tokio::time::sleep(Duration::from_millis(800)).await;
        self.current_url = url.to_string();
        self.actions.push(format!("导航: {}", url));
        println!("✅ 页面加载完成");
        Ok(())
    }
    
    pub async fn search(&mut self, query: &str) -> Result<Vec<String>, String> {
        println!("🔍 [{}] 搜索: {}", self.id, query);
        tokio::time::sleep(Duration::from_millis(600)).await;
        
        // 模拟搜索结果
        let results = self.generate_mock_results(query);
        self.actions.push(format!("搜索: {}", query));
        
        // 存储提取的数据
        self.extracted_data.insert("search_results".to_string(), results.join("; "));
        
        println!("✅ 找到 {} 个结果", results.len());
        Ok(results)
    }
    
    pub async fn extract_data(&mut self, data_type: &str) -> Result<Vec<String>, String> {
        println!("📊 [{}] 提取数据: {}", self.id, data_type);
        tokio::time::sleep(Duration::from_millis(400)).await;
        
        let data = match data_type {
            "prices" => vec!["¥299".to_string(), "¥599".to_string(), "¥899".to_string()],
            "hotels" => vec!["四季酒店".to_string(), "希尔顿酒店".to_string(), "万豪酒店".to_string()],
            "attractions" => vec!["西湖".to_string(), "雷峰塔".to_string(), "断桥".to_string()],
            "companies" => vec!["阿里巴巴".to_string(), "网易".to_string(), "海康威视".to_string()],
            _ => vec!["数据A".to_string(), "数据B".to_string(), "数据C".to_string()],
        };
        
        self.extracted_data.insert(data_type.to_string(), data.join("; "));
        self.actions.push(format!("提取数据: {}", data_type));
        
        println!("✅ 提取了 {} 条数据", data.len());
        Ok(data)
    }
    
    fn generate_mock_results(&self, query: &str) -> Vec<String> {
        if query.contains("旅游") || query.contains("攻略") {
            vec![
                format!("{}旅游完整攻略 - 马蜂窝", self.extract_location(query)),
                format!("{}三日游经典路线 - 携程", self.extract_location(query)),
                format!("{}美食住宿推荐 - 去哪儿", self.extract_location(query)),
            ]
        } else if query.contains("手机") || query.contains("电脑") {
            vec![
                format!("{} - 京东自营官方旗舰店", query),
                format!("{} - 天猫官方直营", query),
                format!("{} - 苏宁易购", query),
            ]
        } else {
            vec![
                format!("关于\"{}\"的权威资料", query),
                format!("{}最新资讯和动态", query),
                format!("{}专业分析报告", query),
            ]
        }
    }
    
    fn extract_location(&self, query: &str) -> String {
        let locations = ["北京", "上海", "杭州", "成都", "西安", "厦门", "三亚"];
        for location in &locations {
            if query.contains(location) {
                return location.to_string();
            }
        }
        "目的地".to_string()
    }
}

impl BrowserManager {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            session_counter: 0,
        }
    }
    
    pub async fn create_session(&mut self) -> String {
        self.session_counter += 1;
        let session_id = format!("smart_session_{}", self.session_counter);
        let session = BrowserSession::new(session_id.clone());
        self.sessions.insert(session_id.clone(), session);
        println!("🆕 创建智能会话: {}", session_id);
        session_id
    }
    
    pub async fn execute_task_step(&mut self, session_id: &str, step: &TaskStep) -> Result<ExecutionStep, String> {
        let start_time = std::time::Instant::now();
        
        if let Some(session) = self.sessions.get_mut(session_id) {
            let result = match step.action.as_str() {
                "navigate" => {
                    session.navigate(&step.target).await?;
                    "导航成功".to_string()
                },
                "search_travel_guide" => {
                    let url = format!("https://www.mafengwo.cn/search?q={}", 
                        step.parameters.get("destination").unwrap_or(&step.target));
                    session.navigate(&url).await?;
                    let results = session.search(&format!("{} 旅游攻略", step.target)).await?;
                    format!("找到 {} 个旅游攻略", results.len())
                },
                "search_products" => {
                    let default_platform = "淘宝".to_string();
                    let platform = step.parameters.get("platform").unwrap_or(&default_platform);
                    let url = format!("https://www.taobao.com/search?q={}", step.target);
                    session.navigate(&url).await?;
                    let results = session.search(&step.target).await?;
                    format!("在{}找到 {} 个商品", platform, results.len())
                },
                "search_information" => {
                    session.navigate("https://www.baidu.com").await?;
                    let results = session.search(&step.target).await?;
                    format!("找到 {} 条相关信息", results.len())
                },
                "extract_prices" => {
                    let prices = session.extract_data("prices").await?;
                    format!("提取了 {} 个价格信息", prices.len())
                },
                "extract_hotels" => {
                    let hotels = session.extract_data("hotels").await?;
                    format!("提取了 {} 个酒店信息", hotels.len())
                },
                "compare_options" => {
                    format!("对比了多个选项，生成了比较分析")
                },
                _ => {
                    format!("执行了操作: {}", step.action)
                }
            };
            
            let duration = start_time.elapsed().as_millis() as u64;
            
            Ok(ExecutionStep {
                step_name: step.expected_result.clone(),
                action: step.action.clone(),
                target: step.target.clone(),
                result: result,
                success: true,
                duration_ms: duration,
            })
        } else {
            Err(format!("会话不存在: {}", session_id))
        }
    }
    
    pub fn get_session_data(&self, session_id: &str) -> Option<&HashMap<String, String>> {
        self.sessions.get(session_id).map(|s| &s.extracted_data)
    }
}

impl SmartExecutor {
    pub fn new(llm_config: LLMConfig) -> Self {
        Self {
            intent_analyzer: SmartIntentAnalyzer::new(llm_config),
            browser_manager: BrowserManager::new(),
            execution_history: Vec::new(),
        }
    }
    
    /// 智能执行用户请求
    pub async fn execute_smart_request(&mut self, user_input: &str) -> Result<SmartExecutionResult, Box<dyn std::error::Error>> {
        let start_time = std::time::Instant::now();
        
        println!("🤖 智能分析用户需求: {}", user_input);
        
        // 1. LLM分析用户意图
        let intent = self.intent_analyzer.analyze_intent(user_input).await?;
        println!("🎯 LLM分析结果: {} (置信度: {:.2})", intent.intent_type, intent.confidence);
        println!("🧠 意图类型: {}", intent.intent_type);
        
        // 2. 创建浏览器会话
        let session_id = self.browser_manager.create_session().await;
        
        // 3. 执行任务计划
        println!("\n🚀 执行LLM生成的任务计划:");
        let mut execution_steps = Vec::new();
        let mut overall_success = true;
        
        for (i, step) in intent.steps.iter().enumerate() {
            println!("\n📋 步骤 {}: {}", i + 1, step.expected_result);
            
            match self.browser_manager.execute_task_step(&session_id, step).await {
                Ok(exec_step) => {
                    println!("✅ {}", exec_step.result);
                    execution_steps.push(exec_step);
                },
                Err(e) => {
                    println!("❌ 执行失败: {}", e);
                    execution_steps.push(ExecutionStep {
                        step_name: step.expected_result.clone(),
                        action: step.action.clone(),
                        target: step.target.clone(),
                        result: format!("失败: {}", e),
                        success: false,
                        duration_ms: 0,
                    });
                    overall_success = false;
                }
            }
        }
        
        // 4. 收集执行结果
        let extracted_data = self.collect_extracted_data(&session_id);
        let execution_summary = self.create_execution_summary(&execution_steps);
        
        // 5. 生成智能回复
        let llm_response = self.intent_analyzer
            .generate_response(&intent, &execution_summary)
            .await
            .unwrap_or_else(|_| "任务执行完成！".to_string());
        
        // 6. 生成推荐和下一步行动
        let recommendations = self.generate_recommendations(&intent, &extracted_data);
        let next_actions = self.generate_next_actions(&intent);
        
        // 7. 记录执行历史
        let execution_record = ExecutionRecord {
            user_input: user_input.to_string(),
            intent: intent.clone(),
            execution_steps: execution_steps.clone(),
            final_result: execution_summary.clone(),
            success: overall_success,
            duration_ms: start_time.elapsed().as_millis() as u64,
            timestamp: chrono::Utc::now().to_rfc3339(),
        };
        self.execution_history.push(execution_record);
        
        Ok(SmartExecutionResult {
            success: overall_success,
            user_intent: intent,
            execution_summary,
            extracted_data,
            recommendations,
            next_actions,
            llm_response,
        })
    }
    
    fn collect_extracted_data(&self, session_id: &str) -> HashMap<String, Vec<String>> {
        let mut result = HashMap::new();
        
        if let Some(session_data) = self.browser_manager.get_session_data(session_id) {
            for (key, value) in session_data {
                let items: Vec<String> = value.split("; ").map(|s| s.to_string()).collect();
                result.insert(key.clone(), items);
            }
        }
        
        result
    }
    
    fn create_execution_summary(&self, steps: &[ExecutionStep]) -> String {
        let total_steps = steps.len();
        let successful_steps = steps.iter().filter(|s| s.success).count();
        let total_duration: u64 = steps.iter().map(|s| s.duration_ms).sum();
        
        format!(
            "执行了 {} 个步骤，成功 {} 个，总耗时 {}ms。\n步骤详情:\n{}",
            total_steps,
            successful_steps, 
            total_duration,
            steps.iter()
                .enumerate()
                .map(|(i, s)| format!("{}. {} - {}", i + 1, s.step_name, 
                    if s.success { "✅" } else { "❌" }))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
    
    fn generate_recommendations(&self, intent: &SmartIntent, data: &HashMap<String, Vec<String>>) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        match intent.intent_type.as_str() {
            "travel_search" => {
                recommendations.push("建议提前预订机票和酒店以获得更好价格".to_string());
                recommendations.push("查看最新的旅游攻略和用户评价".to_string());
                recommendations.push("关注当地天气和最佳旅游时间".to_string());
            },
            "shopping" => {
                recommendations.push("比较不同平台的价格和服务".to_string());
                recommendations.push("查看商品评价和买家真实反馈".to_string());
                recommendations.push("关注促销活动和优惠券".to_string());
            },
            "information_query" => {
                recommendations.push("查看多个权威来源的信息".to_string());
                recommendations.push("关注信息的时效性和准确性".to_string());
            },
            _ => {
                recommendations.push("根据搜索结果选择最相关的信息".to_string());
            }
        }
        
        recommendations
    }
    
    fn generate_next_actions(&self, intent: &SmartIntent) -> Vec<String> {
        let mut actions = Vec::new();
        
        match intent.intent_type.as_str() {
            "travel_search" => {
                actions.push("查看具体的景点详情和门票价格".to_string());
                actions.push("了解当地的交通和美食信息".to_string());
            },
            "shopping" => {
                actions.push("查看商品的详细规格和参数".to_string());
                actions.push("对比不同商家的售后服务".to_string());
            },
            _ => {
                actions.push("深入了解相关的详细信息".to_string());
            }
        }
        
        actions
    }
    
    /// 获取执行历史
    pub fn get_execution_history(&self) -> &[ExecutionRecord] {
        &self.execution_history
    }
}