//! # 工具集成模块
//! 
//! 提供各种辅助工具和实用函数

use std::time::{Duration, SystemTime};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use anyhow::Result;

/// 网络延迟模拟
pub fn simulate_network_delay(ms: u64) {
    std::thread::sleep(Duration::from_millis(ms));
}

/// 处理延迟模拟
pub fn simulate_processing_delay(ms: u64) {
    std::thread::sleep(Duration::from_millis(ms));
}

/// 生成时间戳
pub fn generate_timestamp() -> String {
    chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string()
}

/// 生成唯一ID
pub fn generate_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

/// 格式化持续时间
pub fn format_duration(duration: Duration) -> String {
    let millis = duration.as_millis();
    if millis < 1000 {
        format!("{}ms", millis)
    } else if millis < 60000 {
        format!("{:.1}s", millis as f64 / 1000.0)
    } else {
        format!("{:.1}m", millis as f64 / 60000.0)
    }
}

/// 提取文本中的关键词
pub fn extract_keywords(text: &str, keywords: &[&str]) -> Vec<String> {
    let text_lower = text.to_lowercase();
    keywords
        .iter()
        .filter(|keyword| text_lower.contains(&keyword.to_lowercase()))
        .map(|keyword| keyword.to_string())
        .collect()
}

/// 计算文本相似度（简单版本）
pub fn calculate_similarity(text1: &str, text2: &str) -> f32 {
    let words1: std::collections::HashSet<_> = text1.split_whitespace().collect();
    let words2: std::collections::HashSet<_> = text2.split_whitespace().collect();
    
    let intersection = words1.intersection(&words2).count();
    let union = words1.union(&words2).count();
    
    if union == 0 {
        0.0
    } else {
        intersection as f32 / union as f32
    }
}

/// 标准化的浏览器工具
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BrowserTool {
    NavigateToUrl { url: String },
    Click { selector: String },
    TypeText { selector: String, text: String },
    SelectOption { selector: String, value: String },
    GetText { selector: String },
    WaitForElement { selector: String, timeout_ms: u64 },
    TakeScreenshot { filename: Option<String> },
    ExecuteScript { script: String },
    ScrollPage { direction: ScrollDirection, amount: Option<i32> },
    GetCurrentUrl,
    GoBack,
    GoForward,
    RefreshPage,
}

/// 滚动方向
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScrollDirection {
    Up,
    Down,
    Left, 
    Right,
    ToTop,
    ToBottom,
    ToElement { selector: String },
}

/// 工具执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub tool: String,
    pub success: bool,
    pub data: Option<serde_json::Value>,
    pub error: Option<String>,
    pub duration_ms: u64,
}

/// 工具工厂
pub struct ToolFactory;

impl ToolFactory {
    /// 从字符串创建工具
    pub fn from_string(tool_str: &str, params: HashMap<String, String>) -> Result<BrowserTool> {
        match tool_str.to_lowercase().as_str() {
            "navigate" | "go" => {
                let url = params.get("url")
                    .ok_or_else(|| anyhow::anyhow!("URL parameter required"))?;
                Ok(BrowserTool::NavigateToUrl { url: url.clone() })
            }
            "click" => {
                let selector = params.get("selector")
                    .ok_or_else(|| anyhow::anyhow!("Selector parameter required"))?;
                Ok(BrowserTool::Click { selector: selector.clone() })
            }
            "type" | "input" => {
                let selector = params.get("selector")
                    .ok_or_else(|| anyhow::anyhow!("Selector parameter required"))?;
                let text = params.get("text")
                    .ok_or_else(|| anyhow::anyhow!("Text parameter required"))?;
                Ok(BrowserTool::TypeText { 
                    selector: selector.clone(), 
                    text: text.clone() 
                })
            }
            "screenshot" => {
                let filename = params.get("filename").cloned();
                Ok(BrowserTool::TakeScreenshot { filename })
            }
            "gettext" | "text" => {
                let selector = params.get("selector")
                    .ok_or_else(|| anyhow::anyhow!("Selector parameter required"))?;
                Ok(BrowserTool::GetText { selector: selector.clone() })
            }
            "back" => Ok(BrowserTool::GoBack),
            "forward" => Ok(BrowserTool::GoForward),
            "refresh" => Ok(BrowserTool::RefreshPage),
            "geturl" | "url" => Ok(BrowserTool::GetCurrentUrl),
            _ => Err(anyhow::anyhow!("Unknown tool: {}", tool_str)),
        }
    }

    /// 创建工具链
    pub fn create_tool_chain(steps: Vec<(&str, HashMap<String, String>)>) -> Result<Vec<BrowserTool>> {
        let mut tools = Vec::new();
        
        for (tool_name, params) in steps {
            let tool = Self::from_string(tool_name, params)?;
            tools.push(tool);
        }
        
        Ok(tools)
    }
}