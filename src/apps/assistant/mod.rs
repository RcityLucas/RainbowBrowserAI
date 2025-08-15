//! # 智能助手应用
//! 
//! 基于LLM的通用智能助手

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::core::{llm::*, executor::*};
use crate::base::storage::*;
use crate::types::{ExecutionResult, SessionInfo, OperationResult};
use crate::error::*;
use crate::unified_kernel::session_manager::SessionManager;
use crate::optimized_persistence::OptimizedPersistence;
use std::time::SystemTime;

/// 智能助手
pub struct SmartAssistant {
    executor: SmartExecutor,
    storage: SimpleStorage,
    session_id: String,
    // 新增组件
    session_manager: Arc<SessionManager>,
    persistence: Arc<OptimizedPersistence>,
    context: Arc<RwLock<AssistantContext>>,
    capabilities: Vec<AssistantCapability>,
    user_profile: Arc<RwLock<UserProfile>>,
}

/// 助手上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssistantContext {
    pub current_task: Option<String>,
    pub user_intent: Option<String>,
    pub conversation_state: ConversationState,
    pub active_applications: Vec<String>,
    pub current_website: Option<String>,
    pub user_preferences: HashMap<String, String>,
}

/// 对话状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConversationState {
    Initial,
    InProgress,
    WaitingForUserInput,
    ExecutingTask,
    Completed,
    Error(String),
}

/// 助手能力
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AssistantCapability {
    WebSearch,
    WebNavigation,
    DataExtraction,
    ContentSummarization,
    TaskAutomation,
    ConversationalAI,
    FormFilling,
    DocumentProcessing,
    ImageAnalysis,
    SchedulingAndReminders,
}

/// 用户配置文件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub user_id: Uuid,
    pub preferences: HashMap<String, String>,
    pub favorite_websites: Vec<String>,
    pub common_tasks: Vec<String>,
    pub language: String,
    pub timezone: String,
    pub created_at: SystemTime,
    pub last_updated: SystemTime,
}

impl SmartAssistant {
    /// 创建新的智能助手实例
    pub async fn new() -> Result<Self> {
        let llm_config = LLMConfig::default();
        let executor = SmartExecutor::new(llm_config);
        let storage = SimpleStorage::new();
        let session = storage.create_session();
        
        // 初始化新组件
        let session_manager = Arc::new(SessionManager::new().await?);
        let persistence = Arc::new(OptimizedPersistence::new().await.map_err(|e| 
            BrowserError::ConfigError(format!("Failed to initialize persistence: {}", e)))?);
        
        let context = Arc::new(RwLock::new(AssistantContext {
            current_task: None,
            user_intent: None,
            conversation_state: ConversationState::Initial,
            active_applications: Vec::new(),
            current_website: None,
            user_preferences: HashMap::new(),
        }));
        
        let capabilities = vec![
            AssistantCapability::WebSearch,
            AssistantCapability::WebNavigation,
            AssistantCapability::DataExtraction,
            AssistantCapability::ContentSummarization,
            AssistantCapability::TaskAutomation,
            AssistantCapability::ConversationalAI,
            AssistantCapability::FormFilling,
        ];
        
        let user_profile = Arc::new(RwLock::new(UserProfile {
            user_id: Uuid::new_v4(),
            preferences: HashMap::new(),
            favorite_websites: Vec::new(),
            common_tasks: Vec::new(),
            language: "zh-CN".to_string(),
            timezone: "Asia/Shanghai".to_string(),
            created_at: SystemTime::now(),
            last_updated: SystemTime::now(),
        }));
        
        Ok(Self {
            executor,
            storage,
            session_id: session.id,
            session_manager,
            persistence,
            context,
            capabilities,
            user_profile,
        })
    }
    
    /// 使用用户配置创建助手
    pub async fn new_with_profile(profile: UserProfile) -> Result<Self> {
        let assistant = Self::new().await?;
        *assistant.user_profile.write().await = profile;
        Ok(assistant)
    }
    
    /// 处理用户请求
    pub async fn process_request(&mut self, user_input: &str) -> Result<AssistantResponse> {
        // 更新上下文状态
        {
            let mut context = self.context.write().await;
            context.conversation_state = ConversationState::InProgress;
        }
        
        // 更新会话访问时间
        self.storage.touch_session(&self.session_id);
        
        // 分析用户意图
        let intent = self.analyze_user_intent(user_input).await?;
        
        {
            let mut context = self.context.write().await;
            context.user_intent = Some(intent.clone());
            context.conversation_state = ConversationState::ExecutingTask;
        }
        
        // 使用智能执行器处理请求
        let result = self.executor.execute_smart_request(user_input).await
            .map_err(|e| BrowserError::ExecutionError(e.to_string()))?;
        
        // 保存到持久化系统
        let memory_data = crate::optimized_persistence::MemoryData {
            id: Uuid::new_v4(),
            session_id: Uuid::parse_str(&self.session_id)
                .unwrap_or_else(|_| Uuid::new_v4()),
            timestamp: SystemTime::now(),
            data_type: crate::optimized_persistence::DataType::Conversation,
            content: serde_json::json!({
                "user_input": user_input,
                "assistant_response": result.llm_response,
                "intent": intent,
            }),
            metadata: HashMap::new(),
        };
        
        if let Err(e) = self.persistence.store(memory_data).await {
            log::warn!("Failed to store conversation: {}", e);
        }
        
        // 保存对话记录(保持向后兼容)
        self.storage.add_conversation(
            &self.session_id,
            user_input.to_string(),
            result.llm_response.clone(),
        );
        
        // 更新上下文状态
        {
            let mut context = self.context.write().await;
            context.conversation_state = ConversationState::Completed;
        }
        
        Ok(AssistantResponse {
            content: result.llm_response,
            intent: intent,
            suggestions: self.generate_suggestions(user_input).await,
            metadata: HashMap::new(),
        })
    }
    
    /// 分析用户意图
    async fn analyze_user_intent(&self, user_input: &str) -> Result<String> {
        // 简单的意图识别逻辑
        let input_lower = user_input.to_lowercase();
        
        if input_lower.contains("搜索") || input_lower.contains("search") {
            return Ok("search".to_string());
        } else if input_lower.contains("打开") || input_lower.contains("访问") || input_lower.contains("navigate") {
            return Ok("navigate".to_string());
        } else if input_lower.contains("提取") || input_lower.contains("extract") {
            return Ok("extract".to_string());
        } else if input_lower.contains("填写") || input_lower.contains("fill") {
            return Ok("form_fill".to_string());
        } else if input_lower.contains("总结") || input_lower.contains("summarize") {
            return Ok("summarize".to_string());
        }
        
        Ok("conversation".to_string())
    }
    
    /// 生成建议
    async fn generate_suggestions(&self, user_input: &str) -> Vec<String> {
        let mut suggestions = Vec::new();
        
        // 基于用户输入和上下文生成建议
        if user_input.to_lowercase().contains("搜索") {
            suggestions.push("你可以试试说“在百度搜索XXXX”".to_string());
        }
        
        if user_input.to_lowercase().contains("打开") {
            suggestions.push("你可以说“打开网站 https://example.com”".to_string());
        }
        
        suggestions.push("你还可以说“帮我截个图”或“获取页面文本”".to_string());
        
        suggestions
    }
    
    /// 获取对话历史
    pub fn get_conversation_history(&self) -> Vec<crate::base::storage::ConversationEntry> {
        self.storage.get_conversation_history(&self.session_id)
    }
    
    /// 获取执行历史
    pub fn get_execution_history(&self) -> &[crate::core::executor::ExecutionRecord] {
        self.executor.get_execution_history()
    }
    
    /// 获取助手上下文
    pub async fn get_context(&self) -> AssistantContext {
        self.context.read().await.clone()
    }
    
    /// 更新用户偏好
    pub async fn update_user_preference(&self, key: String, value: String) {
        let mut context = self.context.write().await;
        context.user_preferences.insert(key.clone(), value.clone());
        
        let mut profile = self.user_profile.write().await;
        profile.preferences.insert(key, value);
        profile.last_updated = SystemTime::now();
    }
    
    /// 获取助手能力列表
    pub fn get_capabilities(&self) -> &[AssistantCapability] {
        &self.capabilities
    }
    
    /// 获取用户配置
    pub async fn get_user_profile(&self) -> UserProfile {
        self.user_profile.read().await.clone()
    }
    
    /// 设置当前任务
    pub async fn set_current_task(&self, task: String) {
        let mut context = self.context.write().await;
        context.current_task = Some(task);
    }
    
    /// 设置当前网站
    pub async fn set_current_website(&self, website: String) {
        let mut context = self.context.write().await;
        context.current_website = Some(website);
    }
    
    /// 获取智能建议
    pub async fn get_smart_suggestions(&self) -> Result<Vec<String>> {
        let context = self.context.read().await;
        let mut suggestions = Vec::new();
        
        // 基于当前上下文的智能建议
        if let Some(ref website) = context.current_website {
            suggestions.push(format!("在 {} 上搜索信息", website));
            suggestions.push(format!("提取 {} 的内容", website));
        }
        
        if context.current_task.is_none() {
            suggestions.push("开始一个新任务".to_string());
        }
        
        Ok(suggestions)
    }
}

/// 助手响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssistantResponse {
    pub content: String,
    pub intent: String,
    pub suggestions: Vec<String>,
    pub metadata: HashMap<String, String>,
}

/// 默认实现
 impl Default for AssistantContext {
    fn default() -> Self {
        Self {
            current_task: None,
            user_intent: None,
            conversation_state: ConversationState::Initial,
            active_applications: Vec::new(),
            current_website: None,
            user_preferences: HashMap::new(),
        }
    }
}