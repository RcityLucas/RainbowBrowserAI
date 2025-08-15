// 智能行动 - AI生命体的运动系统
// 结合LLM智能和浏览器控制，执行智能交互

pub mod executor;
pub mod llm_integration;
pub mod smart_executor;
pub mod tools;

use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;
use std::time::Duration;

/// 智能行动系统 - 执行器官
pub struct IntelligentAction {
    // 核心执行能力
    action_executor: Arc<executor::ActionExecutor>,
    
    // LLM智能集成
    llm_integration: Arc<llm_integration::LLMIntegration>,
    
    // 智能执行器
    smart_executor: Arc<smart_executor::SmartExecutor>,
    
    // 执行历史
    execution_history: Arc<RwLock<Vec<ExecutionRecord>>>,
}

/// 执行记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRecord {
    pub session_id: Uuid,
    pub timestamp: std::time::SystemTime,
    pub action: Action,
    pub result: ActionResult,
}

/// 行动类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Action {
    Click { selector: String },
    Input { selector: String, text: String },
    Navigate { url: String },
    Wait { duration_ms: u64 },
    Screenshot { filename: String },
    ExecuteScript { script: String },
    Scroll { direction: ScrollDirection, amount: i32 },
    Extract { selector: String },
}

/// 滚动方向
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// LLM提供商类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LLMProvider {
    OpenAI {
        api_key: String,
        model: String,        // gpt-3.5-turbo, gpt-4, etc.
        base_url: Option<String>,
    },
    Local {
        endpoint: String,     // 本地模型API端点
        model: String,
    },
    Ollama {
        endpoint: String,     // Ollama服务端点  
        model: String,        // llama2, codellama, etc.
    },
    Claude {
        api_key: String,
        model: String,        // claude-3-sonnet, etc.
    },
}

/// LLM配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMConfig {
    pub provider: LLMProvider,
    pub max_tokens: u32,
    pub temperature: f32,
    pub timeout: Duration,
    pub retry_count: u32,
}

/// LLM请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMRequest {
    pub system_prompt: String,
    pub user_message: String,
    pub context: Option<String>,
    pub tools: Option<Vec<ToolDefinition>>,
}

/// LLM响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMResponse {
    pub content: String,
    pub tool_calls: Option<Vec<ToolCall>>,
    pub reasoning: Option<String>,
    pub confidence: f32,
    pub tokens_used: u32,
}

/// 工具定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

/// 工具调用
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub name: String,
    pub arguments: serde_json::Value,
    pub reasoning: Option<String>,
}

/// 智能意图分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartIntent {
    pub intent_type: String,
    pub confidence: f32,
    pub entities: HashMap<String, String>,
    pub task_plan: Vec<TaskStep>,
    pub reasoning: String,
}

/// 任务步骤
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStep {
    pub action: String,
    pub target: String,
    pub parameters: HashMap<String, String>,
    pub description: String,
    pub priority: u32,
}

/// LLM客户端
pub struct LLMClient {
    config: LLMConfig,
    http_client: reqwest::Client,
}

impl Default for LLMConfig {
    fn default() -> Self {
        Self {
            provider: LLMProvider::Local {
                endpoint: "http://localhost:11434".to_string(), // Ollama默认端点
                model: "llama2".to_string(),
            },
            max_tokens: 2048,
            temperature: 0.7,
            timeout: Duration::from_secs(30),
            retry_count: 3,
        }
    }
}

impl LLMClient {
    pub fn new(config: LLMConfig) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(config.timeout)
            .build()
            .expect("Failed to create HTTP client");
            
        Self {
            config,
            http_client,
        }
    }
    
    /// 发送LLM请求
    pub async fn send_request(&self, request: LLMRequest) -> Result<LLMResponse, Box<dyn std::error::Error>> {
        match &self.config.provider {
            LLMProvider::OpenAI { api_key, model, base_url } => {
                self.call_openai(request, api_key, model, base_url.as_deref()).await
            },
            LLMProvider::Local { endpoint, model } => {
                self.call_local(request, endpoint, model).await
            },
            LLMProvider::Ollama { endpoint, model } => {
                self.call_ollama(request, endpoint, model).await
            },
            LLMProvider::Claude { api_key, model } => {
                self.call_claude(request, api_key, model).await
            },
        }
    }
    
    /// 调用OpenAI API
    async fn call_openai(
        &self, 
        request: LLMRequest, 
        api_key: &str, 
        model: &str,
        base_url: Option<&str>
    ) -> Result<LLMResponse, Box<dyn std::error::Error>> {
        let url = format!("{}/v1/chat/completions", 
            base_url.unwrap_or("https://api.openai.com"));
        
        let mut messages = vec![
            serde_json::json!({
                "role": "system",
                "content": request.system_prompt
            })
        ];
        
        if let Some(context) = &request.context {
            messages.push(serde_json::json!({
                "role": "assistant", 
                "content": format!("上下文信息: {}", context)
            }));
        }
        
        messages.push(serde_json::json!({
            "role": "user",
            "content": request.user_message
        }));
        
        let mut body = serde_json::json!({
            "model": model,
            "messages": messages,
            "max_tokens": self.config.max_tokens,
            "temperature": self.config.temperature
        });
        
        // 如果有工具定义，添加到请求中
        if let Some(tools) = request.tools {
            body["tools"] = serde_json::to_value(tools)?;
            body["tool_choice"] = serde_json::Value::String("auto".to_string());
        }
        
        let response = self.http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;
            
        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(format!("OpenAI API error: {}", error_text).into());
        }
        
        let response_json: serde_json::Value = response.json().await?;
        
        // 解析响应
        let choice = &response_json["choices"][0];
        let message = &choice["message"];
        
        let content = message["content"].as_str().unwrap_or("").to_string();
        let mut tool_calls = None;
        
        if let Some(calls) = message["tool_calls"].as_array() {
            let mut parsed_calls = Vec::new();
            for call in calls {
                if let Some(function) = call["function"].as_object() {
                    parsed_calls.push(ToolCall {
                        name: function["name"].as_str().unwrap_or("").to_string(),
                        arguments: function["arguments"].clone(),
                        reasoning: None,
                    });
                }
            }
            if !parsed_calls.is_empty() {
                tool_calls = Some(parsed_calls);
            }
        }
        
        let tokens_used = response_json["usage"]["total_tokens"].as_u64().unwrap_or(0) as u32;
        
        Ok(LLMResponse {
            content,
            tool_calls,
            reasoning: None,
            confidence: 0.8, // OpenAI不提供置信度，使用默认值
            tokens_used,
        })
    }
    
    /// 调用Ollama本地模型
    async fn call_ollama(
        &self,
        request: LLMRequest,
        endpoint: &str,
        model: &str
    ) -> Result<LLMResponse, Box<dyn std::error::Error>> {
        let url = format!("{}/api/generate", endpoint);
        
        let prompt = if let Some(context) = &request.context {
            format!("系统提示: {}\n\n上下文: {}\n\n用户: {}", 
                request.system_prompt, context, request.user_message)
        } else {
            format!("系统提示: {}\n\n用户: {}", 
                request.system_prompt, request.user_message)
        };
        
        let body = serde_json::json!({
            "model": model,
            "prompt": prompt,
            "options": {
                "temperature": self.config.temperature,
                "num_predict": self.config.max_tokens
            }
        });
        
        let response = self.http_client
            .post(&url)
            .json(&body)
            .send()
            .await?;
            
        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(format!("Ollama API error: {}", error_text).into());
        }
        
        let response_json: serde_json::Value = response.json().await?;
        
        let content = response_json["response"].as_str().unwrap_or("").to_string();
        
        Ok(LLMResponse {
            content,
            tool_calls: None,
            reasoning: None,
            confidence: 0.75,
            tokens_used: 0, // Ollama响应中可能没有token计数
        })
    }
    
    /// 调用本地模型API (通用格式)
    async fn call_local(
        &self,
        request: LLMRequest,
        endpoint: &str,
        model: &str
    ) -> Result<LLMResponse, Box<dyn std::error::Error>> {
        // 假设本地API使用类似OpenAI的格式
        self.call_openai(request, "local-key", model, Some(endpoint)).await
    }
    
    /// 调用Claude API
    async fn call_claude(
        &self,
        request: LLMRequest,
        api_key: &str,
        model: &str
    ) -> Result<LLMResponse, Box<dyn std::error::Error>> {
        let url = "https://api.anthropic.com/v1/messages";
        
        let messages = if let Some(context) = &request.context {
            vec![serde_json::json!({
                "role": "user",
                "content": format!("上下文: {}\n\n{}", context, request.user_message)
            })]
        } else {
            vec![serde_json::json!({
                "role": "user", 
                "content": request.user_message
            })]
        };
        
        let body = serde_json::json!({
            "model": model,
            "max_tokens": self.config.max_tokens,
            "temperature": self.config.temperature,
            "system": request.system_prompt,
            "messages": messages
        });
        
        let response = self.http_client
            .post(url)
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await?;
            
        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(format!("Claude API error: {}", error_text).into());
        }
        
        let response_json: serde_json::Value = response.json().await?;
        
        let content = response_json["content"][0]["text"].as_str().unwrap_or("").to_string();
        let tokens_used = response_json["usage"]["output_tokens"].as_u64().unwrap_or(0) as u32;
        
        Ok(LLMResponse {
            content,
            tool_calls: None,
            reasoning: None,
            confidence: 0.85,
            tokens_used,
        })
    }
}

/// 智能意图分析器
pub struct SmartIntentAnalyzer {
    llm_client: LLMClient,
}

impl SmartIntentAnalyzer {
    pub fn new(llm_config: LLMConfig) -> Self {
        Self {
            llm_client: LLMClient::new(llm_config),
        }
    }
    
    /// 分析用户意图
    pub async fn analyze_intent(&self, user_input: &str, context: Option<&str>) -> Result<SmartIntent, Box<dyn std::error::Error>> {
        let system_prompt = r#"
你是一个智能意图分析器，能够理解用户的需求并制定执行计划。

请分析用户输入，识别意图类型，提取关键实体，并制定详细的任务执行计划。

支持的意图类型:
1. travel_search - 旅游攻略搜索
2. shopping - 购物需求
3. information_query - 信息查询
4. booking - 预订需求
5. comparison - 比较分析
6. recommendation - 推荐需求

请以JSON格式返回分析结果:
{
  "intent_type": "意图类型",
  "confidence": 0.95,
  "entities": {
    "destination": "目的地",
    "product": "商品名称",
    "price_range": "价格范围",
    "duration": "时长",
    "category": "类别"
  },
  "task_plan": [
    {
      "action": "search_travel_guide",
      "target": "马蜂窝",
      "parameters": {"destination": "杭州", "type": "攻略"},
      "description": "在马蜂窝搜索杭州旅游攻略",
      "priority": 1
    }
  ],
  "reasoning": "分析推理过程"
}
"#;

        let request = LLMRequest {
            system_prompt: system_prompt.to_string(),
            user_message: user_input.to_string(),
            context: context.map(|s| s.to_string()),
            tools: None,
        };
        
        let response = self.llm_client.send_request(request).await?;
        
        // 尝试解析JSON响应
        if let Ok(intent) = serde_json::from_str::<SmartIntent>(&response.content) {
            Ok(intent)
        } else {
            // 如果JSON解析失败，创建一个基础的意图
            Ok(SmartIntent {
                intent_type: "information_query".to_string(),
                confidence: 0.5,
                entities: HashMap::new(),
                task_plan: vec![
                    TaskStep {
                        action: "search_general".to_string(),
                        target: "百度".to_string(),
                        parameters: {
                            let mut params = HashMap::new();
                            params.insert("query".to_string(), user_input.to_string());
                            params
                        },
                        description: format!("搜索: {}", user_input),
                        priority: 1,
                    }
                ],
                reasoning: response.content,
            })
        }
    }
    
    /// 生成智能回复
    pub async fn generate_response(&self, intent: &SmartIntent, execution_result: &str) -> Result<String, Box<dyn std::error::Error>> {
        let system_prompt = r#"
你是一个友好的AI助手，负责根据用户意图和执行结果生成自然的回复。

请用友好、专业的语气回复用户，包含:
1. 对用户需求的理解确认
2. 执行过程的简要说明  
3. 结果的总结和建议
4. 下一步的行动建议

回复要自然、有用、有温度。
"#;

        let user_message = format!(
            "用户意图: {:?}\n执行结果: {}\n\n请生成友好的回复。",
            intent, execution_result
        );
        
        let request = LLMRequest {
            system_prompt: system_prompt.to_string(),
            user_message,
            context: None,
            tools: None,
        };
        
        let response = self.llm_client.send_request(request).await?;
        Ok(response.content)
    }
}