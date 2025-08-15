// LLM集成 - 大语言模型智能集成

use anyhow::Result;
use serde::{Deserialize, Serialize};
use super::{Action, ActionResult, SmartTask};

/// LLM集成模块
pub struct LLMIntegration {
    client: LLMClient,
}

/// LLM提供商
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LLMProvider {
    OpenAI { api_key: String, model: String, base_url: Option<String> },
    Local { endpoint: String, model: String },
    Ollama { endpoint: String, model: String },
    Claude { api_key: String, model: String },
}

/// LLM客户端 (模拟实现)
pub struct LLMClient {
    provider: LLMProvider,
    // http_client: reqwest::Client,  // 临时禁用
}

impl LLMIntegration {
    pub async fn new() -> Result<Self> {
        // 从环境变量读取配置
        let provider = Self::get_provider_from_env();
        
        Ok(Self {
            client: LLMClient::new(provider),
        })
    }
    
    /// 从环境变量获取LLM提供商配置
    fn get_provider_from_env() -> LLMProvider {
        // 优先使用OpenAI
        if let Ok(api_key) = std::env::var("OPENAI_API_KEY") {
            log::info!("使用OpenAI API");
            return LLMProvider::OpenAI {
                api_key,
                model: std::env::var("OPENAI_MODEL")
                    .unwrap_or_else(|_| "gpt-3.5-turbo".to_string()),
                base_url: std::env::var("OPENAI_BASE_URL").ok(),
            };
        }
        
        // 其次使用Claude
        if let Ok(api_key) = std::env::var("ANTHROPIC_API_KEY") {
            log::info!("使用Claude API");
            return LLMProvider::Claude {
                api_key,
                model: std::env::var("CLAUDE_MODEL")
                    .unwrap_or_else(|_| "claude-3-sonnet-20240229".to_string()),
            };
        }
        
        // 最后使用Ollama本地模型
        log::info!("使用Ollama本地模型");
        LLMProvider::Ollama {
            endpoint: std::env::var("OLLAMA_ENDPOINT")
                .unwrap_or_else(|_| "http://localhost:11434".to_string()),
            model: std::env::var("OLLAMA_MODEL")
                .unwrap_or_else(|_| "llama2".to_string()),
        }
    }
    
    /// 分析用户请求
    pub async fn analyze_request(&self, user_request: &str) -> Result<SmartTask> {
        let prompt = format!(r#"
分析用户请求并生成浏览器操作计划。

用户请求: {}

请生成一个包含以下字段的JSON对象:
- intent: 用户意图类型 (search/navigate/extract/interact)
- actions: 操作步骤数组，每个操作包含 type 和 parameters
- expected_outcome: 预期结果描述

示例操作类型:
- Navigate: {{"type": "navigate", "url": "https://example.com"}}
- Input: {{"type": "input", "selector": "input[name='q']", "text": "search text"}}
- Click: {{"type": "click", "selector": "button[type='submit']"}}
- Extract: {{"type": "extract", "selector": ".result"}}
        "#, user_request);
        
        let response = self.client.query(&prompt).await?;
        
        // 尝试解析JSON响应
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&response) {
            let intent = json["intent"].as_str().unwrap_or("search").to_string();
            
            let mut actions = Vec::new();
            if let Some(action_array) = json["actions"].as_array() {
                for action_json in action_array {
                    if let Some(action_type) = action_json["type"].as_str() {
                        match action_type {
                            "navigate" => {
                                if let Some(url) = action_json["url"].as_str() {
                                    actions.push(Action::Navigate { url: url.to_string() });
                                }
                            }
                            "input" => {
                                if let (Some(selector), Some(text)) = (
                                    action_json["selector"].as_str(),
                                    action_json["text"].as_str()
                                ) {
                                    actions.push(Action::Input {
                                        selector: selector.to_string(),
                                        text: text.to_string(),
                                    });
                                }
                            }
                            "click" => {
                                if let Some(selector) = action_json["selector"].as_str() {
                                    actions.push(Action::Click { selector: selector.to_string() });
                                }
                            }
                            "extract" => {
                                if let Some(selector) = action_json["selector"].as_str() {
                                    actions.push(Action::Extract { selector: selector.to_string() });
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            
            // 如果没有解析到操作，使用默认操作
            if actions.is_empty() {
                actions = Self::get_default_actions(user_request);
            }
            
            return Ok(SmartTask {
                description: user_request.to_string(),
                intent,
                actions,
                expected_outcome: json["expected_outcome"]
                    .as_str()
                    .unwrap_or("完成用户请求")
                    .to_string(),
            });
        }
        
        // 如果JSON解析失败，使用默认策略
        Ok(SmartTask {
            description: user_request.to_string(),
            intent: "search".to_string(),
            actions: Self::get_default_actions(user_request),
            expected_outcome: "搜索相关信息".to_string(),
        })
    }
    
    /// 获取默认操作
    fn get_default_actions(user_request: &str) -> Vec<Action> {
        vec![
            Action::Navigate { url: "https://www.google.com".to_string() },
            Action::Input {
                selector: "input[name='q']".to_string(),
                text: user_request.to_string(),
            },
            Action::Click { selector: "input[type='submit']".to_string() },
        ]
    }
    
    /// 判断错误后是否继续
    pub async fn should_continue_after_error(&self, result: &ActionResult) -> Result<bool> {
        if let Some(error) = &result.error {
            let prompt = format!(
                "执行操作时遇到错误: {}\n是否应该继续执行后续步骤？(yes/no)",
                error
            );
            
            let response = self.client.query(&prompt).await?;
            Ok(response.to_lowercase().contains("yes"))
        } else {
            Ok(true)
        }
    }
}

impl LLMClient {
    pub fn new(provider: LLMProvider) -> Self {
        Self {
            provider,
            // http_client: reqwest::Client::new(),
        }
    }
    
    /// 查询LLM
    pub async fn query(&self, prompt: &str) -> Result<String> {
        match &self.provider {
            LLMProvider::Ollama { endpoint, model } => {
                self.query_ollama(prompt, endpoint, model).await
            }
            LLMProvider::OpenAI { api_key, model, base_url } => {
                self.query_openai(prompt, api_key, model, base_url.as_deref()).await
            }
            _ => Ok("模拟响应".to_string()),
        }
    }
    
    async fn query_ollama(&self, prompt: &str, endpoint: &str, model: &str) -> Result<String> {
        let url = format!("{}/api/generate", endpoint);
        
        let body = serde_json::json!({
            "model": model,
            "prompt": prompt,
            "stream": false,
        });
        
        // 模拟HTTP请求
        log::info!("模拟发送请求到: {}", url);
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        // 返回模拟响应
        Ok(format!("模拟Ollama响应: {}", prompt))
    }
    
    async fn query_openai(&self, prompt: &str, api_key: &str, model: &str, base_url: Option<&str>) -> Result<String> {
        let url = format!("{}/v1/chat/completions", 
            base_url.unwrap_or("https://api.openai.com"));
        
        let body = serde_json::json!({
            "model": model,
            "messages": [
                {"role": "user", "content": prompt}
            ],
        });
        
        // 模拟HTTP请求
        log::info!("模拟发送OpenAI请求");
        tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;
        
        // 返回模拟响应
        Ok(format!("模拟OpenAI响应: {}", prompt))
    }
}