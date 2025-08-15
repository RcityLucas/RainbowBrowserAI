//! # 配置管理
//! 
//! 定义项目配置结构

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// 浏览器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserConfig {
    pub llm: LLMConfig,
    pub browser: BrowserSettings,
    pub session: SessionSettings,
}

/// LLM配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMConfig {
    pub provider: String,
    pub model: String,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub max_tokens: u32,
    pub temperature: f32,
    pub timeout_secs: u64,
}

/// 浏览器设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserSettings {
    pub headless: bool,
    pub window_width: u32,
    pub window_height: u32,
    pub timeout_secs: u64,
    pub webdriver_url: String,
}

/// 会话设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSettings {
    pub max_sessions: usize,
    pub session_timeout_hours: u64,
    pub cleanup_interval_minutes: u64,
}

impl Default for BrowserConfig {
    fn default() -> Self {
        Self {
            llm: LLMConfig::default(),
            browser: BrowserSettings::default(),
            session: SessionSettings::default(),
        }
    }
}

impl Default for LLMConfig {
    fn default() -> Self {
        Self {
            provider: "local".to_string(),
            model: "llama2".to_string(),
            api_key: None,
            base_url: Some("http://localhost:11434".to_string()),
            max_tokens: 2048,
            temperature: 0.7,
            timeout_secs: 30,
        }
    }
}

impl Default for BrowserSettings {
    fn default() -> Self {
        Self {
            headless: true,
            window_width: 1920,
            window_height: 1080,
            timeout_secs: 30,
            webdriver_url: "http://localhost:9515".to_string(),
        }
    }
}

impl Default for SessionSettings {
    fn default() -> Self {
        Self {
            max_sessions: 100,
            session_timeout_hours: 24,
            cleanup_interval_minutes: 60,
        }
    }
}