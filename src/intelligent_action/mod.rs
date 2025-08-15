// 智能行动 - AI生命体的运动系统
// 结合LLM智能和浏览器控制，执行智能交互

pub mod executor;
pub mod llm_integration;
pub mod smart_executor;
pub mod tools;
pub mod browser_driver;

use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use executor::ActionExecutor;
use llm_integration::LLMIntegration;
use smart_executor::SmartExecutor;

/// 智能行动系统 - 执行器官
pub struct IntelligentAction {
    // 核心执行能力
    action_executor: Arc<ActionExecutor>,
    
    // LLM智能集成
    llm_integration: Arc<LLMIntegration>,
    
    // 智能执行器
    smart_executor: Arc<SmartExecutor>,
    
    // 执行历史
    execution_history: Arc<RwLock<Vec<ExecutionRecord>>>,
}

/// 行动类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Action {
    Click { selector: String },
    Input { selector: String, text: String },
    Type { selector: String, text: String },  // 添加Type动作
    Navigate { url: String },
    Wait { selector: String, timeout_ms: u64, duration_ms: u64 },
    Screenshot,  // 简化为无字段
    ExecuteScript { script: String },
    Scroll { direction: ScrollDirection, amount: i32 },
    Extract { selector: String },
}

/// 滚动方向
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ScrollDirection {
    Up,
    Down,
    Left,
    Right,
}

/// 执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionResult {
    pub action: Action,
    pub success: bool,
    pub data: Option<serde_json::Value>,
    pub error: Option<String>,
    pub duration_ms: u64,
}

/// 执行记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRecord {
    pub session_id: Uuid,
    pub timestamp: std::time::SystemTime,
    pub action: Action,
    pub result: ActionResult,
}

/// 智能任务 - LLM分析后的任务
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartTask {
    pub description: String,
    pub intent: String,
    pub actions: Vec<Action>,
    pub expected_outcome: String,
}

impl IntelligentAction {
    /// 创建智能行动系统
    pub async fn new() -> Result<Self> {
        let action_executor = Arc::new(ActionExecutor::new().await?);
        let llm_integration = Arc::new(LLMIntegration::new().await?);
        let smart_executor = Arc::new(SmartExecutor::new().await?);
        
        Ok(Self {
            action_executor,
            llm_integration,
            smart_executor,
            execution_history: Arc::new(RwLock::new(Vec::new())),
        })
    }
    
    /// 执行单个行动
    pub async fn execute_action(&self, session_id: Uuid, action: Action) -> Result<ActionResult> {
        let start = std::time::Instant::now();
        
        // 执行行动
        let result = self.action_executor.execute(&action).await;
        
        let duration_ms = start.elapsed().as_millis() as u64;
        
        let action_result = match result {
            Ok(data) => ActionResult {
                action: action.clone(),
                success: true,
                data: Some(data),
                error: None,
                duration_ms,
            },
            Err(e) => ActionResult {
                action: action.clone(),
                success: false,
                data: None,
                error: Some(e.to_string()),
                duration_ms,
            },
        };
        
        // 记录执行历史
        self.record_execution(session_id, action.clone(), action_result.clone()).await;
        
        Ok(action_result)
    }
    
    /// 使用LLM分析用户意图并执行
    pub async fn execute_smart_task(&self, session_id: Uuid, user_request: &str) -> Result<Vec<ActionResult>> {
        // 使用LLM分析用户意图
        let smart_task = self.llm_integration.analyze_request(user_request).await?;
        
        // 执行智能任务
        let mut results = Vec::new();
        for action in smart_task.actions {
            let result = self.execute_action(session_id, action).await?;
            
            // 如果某个步骤失败，可以使用LLM决定是否继续
            if !result.success {
                let should_continue = self.llm_integration.should_continue_after_error(&result).await?;
                if !should_continue {
                    break;
                }
            }
            
            results.push(result);
        }
        
        Ok(results)
    }
    
    /// 批量执行行动
    pub async fn execute_batch(&self, session_id: Uuid, actions: Vec<Action>) -> Result<Vec<ActionResult>> {
        let mut results = Vec::new();
        
        for action in actions {
            let result = self.execute_action(session_id, action).await?;
            results.push(result);
        }
        
        Ok(results)
    }
    
    /// 执行带验证的行动
    pub async fn execute_with_verification(
        &self,
        session_id: Uuid,
        action: Action,
        verification: impl Fn(&ActionResult) -> bool,
    ) -> Result<ActionResult> {
        let mut attempts = 0;
        const MAX_ATTEMPTS: u32 = 3;
        
        loop {
            let result = self.execute_action(session_id, action.clone()).await?;
            
            if verification(&result) || attempts >= MAX_ATTEMPTS {
                return Ok(result);
            }
            
            attempts += 1;
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        }
    }
    
    /// 记录执行历史
    async fn record_execution(&self, session_id: Uuid, action: Action, result: ActionResult) {
        let mut history = self.execution_history.write().await;
        
        history.push(ExecutionRecord {
            session_id,
            timestamp: std::time::SystemTime::now(),
            action,
            result,
        });
        
        // 限制历史记录大小
        if history.len() > 1000 {
            history.drain(0..100);
        }
    }
    
    /// 获取执行历史
    pub async fn get_execution_history(&self, session_id: Option<Uuid>) -> Vec<ExecutionRecord> {
        let history = self.execution_history.read().await;
        
        if let Some(id) = session_id {
            history.iter()
                .filter(|record| record.session_id == id)
                .cloned()
                .collect()
        } else {
            history.clone()
        }
    }
    
    /// 清理执行历史
    pub async fn clear_history(&self, session_id: Option<Uuid>) {
        let mut history = self.execution_history.write().await;
        
        if let Some(id) = session_id {
            history.retain(|record| record.session_id != id);
        } else {
            history.clear();
        }
    }
}