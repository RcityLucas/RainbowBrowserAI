//! # 应用层 (Apps Layer)
//! 
//! 提供面向用户的智能助手应用

pub mod assistant;
pub mod travel;
pub mod shopping;

use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::error::Result;

// 重新导出主要类型
pub use assistant::{SmartAssistant, AssistantResponse, AssistantContext};
pub use travel::{TravelAssistant, TravelResponse, TravelContext};
pub use shopping::{ShoppingAssistant, ShoppingResponse, ShoppingContext};

/// 应用管理器
pub struct AppManager {
    smart_assistant: Arc<tokio::sync::Mutex<SmartAssistant>>,
    travel_assistant: Arc<tokio::sync::Mutex<TravelAssistant>>,
    shopping_assistant: Arc<tokio::sync::Mutex<ShoppingAssistant>>,
    current_app: AppType,
}

/// 应用类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AppType {
    SmartAssistant,
    TravelAssistant,
    ShoppingAssistant,
}

/// 统一应用响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AppResponse {
    Assistant(AssistantResponse),
    Travel(TravelResponse),
    Shopping(ShoppingResponse),
}

impl AppManager {
    /// 创建新的应用管理器
    pub async fn new() -> Result<Self> {
        Ok(Self {
            smart_assistant: Arc::new(tokio::sync::Mutex::new(SmartAssistant::new().await?)),
            travel_assistant: Arc::new(tokio::sync::Mutex::new(TravelAssistant::new().await?)),
            shopping_assistant: Arc::new(tokio::sync::Mutex::new(ShoppingAssistant::new().await?)),
            current_app: AppType::SmartAssistant,
        })
    }

    /// 切换应用
    pub async fn switch_app(&mut self, app_type: AppType) {
        self.current_app = app_type;
    }

    /// 获取当前应用类型
    pub fn current_app(&self) -> AppType {
        self.current_app.clone()
    }

    /// 处理用户请求
    pub async fn process_request(&self, user_input: &str) -> Result<AppResponse> {
        match self.current_app {
            AppType::SmartAssistant => {
                let mut assistant = self.smart_assistant.lock().await;
                let response = assistant.process_request(user_input).await?;
                Ok(AppResponse::Assistant(response))
            }
            AppType::TravelAssistant => {
                let mut assistant = self.travel_assistant.lock().await;
                let response = assistant.process_travel_request(user_input).await?;
                Ok(AppResponse::Travel(response))
            }
            AppType::ShoppingAssistant => {
                let mut assistant = self.shopping_assistant.lock().await;
                let response = assistant.process_shopping_request(user_input).await?;
                Ok(AppResponse::Shopping(response))
            }
        }
    }

    /// 智能路由 - 自动选择最适合的应用
    pub async fn smart_route(&mut self, user_input: &str) -> Result<AppResponse> {
        let app_type = self.analyze_intent(user_input).await;
        
        if app_type != self.current_app {
            self.switch_app(app_type).await;
        }

        self.process_request(user_input).await
    }

    /// 分析用户意图选择应用
    async fn analyze_intent(&self, user_input: &str) -> AppType {
        let input_lower = user_input.to_lowercase();

        // 旅行相关关键词
        if input_lower.contains("旅行") || input_lower.contains("旅游") || 
           input_lower.contains("机票") || input_lower.contains("酒店") ||
           input_lower.contains("travel") || input_lower.contains("trip") ||
           input_lower.contains("目的地") || input_lower.contains("行程") {
            return AppType::TravelAssistant;
        }

        // 购物相关关键词
        if input_lower.contains("购物") || input_lower.contains("买") ||
           input_lower.contains("价格") || input_lower.contains("商品") ||
           input_lower.contains("shopping") || input_lower.contains("buy") ||
           input_lower.contains("购买") || input_lower.contains("比价") {
            return AppType::ShoppingAssistant;
        }

        // 默认使用智能助手
        AppType::SmartAssistant
    }

    /// 获取所有应用状态
    pub async fn get_app_status(&self) -> AppStatus {
        let assistant_context = {
            let assistant = self.smart_assistant.lock().await;
            assistant.get_context().await
        };

        let travel_context = {
            let assistant = self.travel_assistant.lock().await;
            assistant.get_travel_context().await
        };

        let shopping_context = {
            let assistant = self.shopping_assistant.lock().await;
            assistant.get_shopping_context().await
        };

        AppStatus {
            current_app: self.current_app.clone(),
            assistant_context,
            travel_context,
            shopping_context,
        }
    }
}

/// 应用状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppStatus {
    pub current_app: AppType,
    pub assistant_context: AssistantContext,
    pub travel_context: TravelContext,
    pub shopping_context: ShoppingContext,
}