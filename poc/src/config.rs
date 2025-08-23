use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::time::Duration;
use tracing::{info, warn};

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Browser configuration
    pub browser: BrowserConfig,
    
    /// LLM service configuration
    pub llm: LlmConfig,
    
    /// Workflow configuration
    pub workflow: WorkflowConfig,
    
    /// Pool configuration
    pub pool: PoolConfig,
    
    /// Cache configuration
    pub cache: CacheConfig,
    
    /// Budget configuration
    pub budget: BudgetConfig,
    
    /// Logging configuration
    pub logging: LoggingConfig,
    
    /// Metrics configuration
    pub metrics: MetricsConfig,
    
    /// API configuration
    pub api: Option<ApiConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserConfig {
    /// ChromeDriver URL
    pub driver_url: String,
    
    /// Default viewport width
    pub default_width: u32,
    
    /// Default viewport height
    pub default_height: u32,
    
    /// Headless mode
    pub headless: bool,
    
    /// Screenshot directory
    pub screenshot_dir: String,
    
    /// Enable Chrome DevTools Protocol
    pub enable_cdp: bool,
    
    /// Browser arguments
    pub browser_args: Vec<String>,
    
    /// Navigation timeout in seconds
    pub navigation_timeout: u64,
    
    /// Script execution timeout in seconds
    pub script_timeout: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    /// OpenAI API key (can be overridden by env var)
    pub api_key: Option<String>,
    
    /// Model to use
    pub model: String,
    
    /// Temperature for generation
    pub temperature: f32,
    
    /// Maximum tokens
    pub max_tokens: u32,
    
    /// Request timeout in seconds
    pub timeout: u64,
    
    /// Maximum retries
    pub max_retries: u32,
    
    /// Base URL override
    pub base_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowConfig {
    /// Workflow templates directory
    pub templates_dir: String,
    
    /// Maximum workflow steps
    pub max_steps: usize,
    
    /// Maximum loop iterations
    pub max_loop_iterations: usize,
    
    /// Maximum parallel actions
    pub max_parallel: usize,
    
    /// Default step timeout in seconds
    pub default_timeout: u64,
    
    /// Enable dry run mode
    pub dry_run: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolConfig {
    /// Maximum pool size
    pub max_size: usize,
    
    /// Idle timeout in seconds
    pub idle_timeout: u64,
    
    /// Maximum lifetime in seconds
    pub max_lifetime: u64,
    
    /// Maximum usage count
    pub max_usage: usize,
    
    /// Enable pooling
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Enable caching
    pub enabled: bool,
    
    /// LLM cache TTL in seconds
    pub llm_ttl: u64,
    
    /// LLM cache max size
    pub llm_max_size: usize,
    
    /// Workflow cache TTL in seconds
    pub workflow_ttl: u64,
    
    /// Workflow cache max size
    pub workflow_max_size: usize,
    
    /// Cache directory
    pub cache_dir: String,
    
    /// Enable persistent cache
    pub persistent: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetConfig {
    /// Daily budget limit
    pub daily_limit: f64,
    
    /// Warning threshold (percentage)
    pub warning_threshold: f64,
    
    /// Block operations when limit reached
    pub block_when_exceeded: bool,
    
    /// Cost per LLM token (estimated)
    pub cost_per_token: f64,
    
    /// Cost per browser operation (estimated)
    pub cost_per_browser_op: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level (trace, debug, info, warn, error)
    pub level: String,
    
    /// Log format (json, pretty)
    pub format: String,
    
    /// Log file path
    pub file: Option<String>,
    
    /// Enable console output
    pub console: bool,
    
    /// Enable structured logging
    pub structured: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// Enable metrics collection
    pub enabled: bool,
    
    /// Metrics export port
    pub port: u16,
    
    /// Export format (prometheus, json)
    pub format: String,
    
    /// Export interval in seconds
    pub export_interval: u64,
    
    /// Histogram buckets
    pub histogram_buckets: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    /// Enable API server
    pub enabled: bool,
    
    /// API server port
    pub port: Option<u16>,
    
    /// Authentication token
    pub auth_token: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            browser: BrowserConfig {
                driver_url: "http://localhost:9515".to_string(),
                default_width: 1920,
                default_height: 1080,
                headless: false,
                screenshot_dir: "screenshots".to_string(),
                enable_cdp: false,
                browser_args: vec![
                    "--disable-gpu".to_string(),
                    "--no-sandbox".to_string(),
                    "--disable-dev-shm-usage".to_string(),
                ],
                navigation_timeout: 30,
                script_timeout: 10,
            },
            llm: LlmConfig {
                api_key: None,
                model: "gpt-4".to_string(),
                temperature: 0.7,
                max_tokens: 2000,
                timeout: 30,
                max_retries: 3,
                base_url: None,
            },
            workflow: WorkflowConfig {
                templates_dir: "workflows/templates".to_string(),
                max_steps: 100,
                max_loop_iterations: 100,
                max_parallel: 10,
                default_timeout: 30,
                dry_run: false,
            },
            pool: PoolConfig {
                max_size: 3,
                idle_timeout: 300,
                max_lifetime: 3600,
                max_usage: 100,
                enabled: true,
            },
            cache: CacheConfig {
                enabled: true,
                llm_ttl: 3600,
                llm_max_size: 1000,
                workflow_ttl: 86400,
                workflow_max_size: 100,
                cache_dir: "cache".to_string(),
                persistent: false,
            },
            budget: BudgetConfig {
                daily_limit: 5.0,
                warning_threshold: 0.8,
                block_when_exceeded: true,
                cost_per_token: 0.00003,
                cost_per_browser_op: 0.001,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                format: "pretty".to_string(),
                file: None,
                console: true,
                structured: false,
            },
            metrics: MetricsConfig {
                enabled: true,
                port: 9091,
                format: "prometheus".to_string(),
                export_interval: 60,
                histogram_buckets: vec![
                    0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0
                ],
            },
            api: Some(ApiConfig {
                enabled: true,
                port: Some(3000),
                auth_token: None,
            }),
        }
    }
}

impl Config {
    /// Load configuration from file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let config: Config = if content.trim().starts_with('{') {
            serde_json::from_str(&content)?
        } else {
            serde_yaml::from_str(&content)?
        };
        
        info!("Configuration loaded from file");
        Ok(config)
    }
    
    /// Load configuration with environment variable overrides
    pub fn from_env() -> Result<Self> {
        let mut config = Self::default();
        
        // Override with environment variables
        if let Ok(url) = std::env::var("CHROME_DRIVER_URL") {
            config.browser.driver_url = url;
        }
        
        if let Ok(key) = std::env::var("OPENAI_API_KEY") {
            config.llm.api_key = Some(key);
        }
        
        if let Ok(budget) = std::env::var("DAILY_BUDGET") {
            if let Ok(limit) = budget.parse() {
                config.budget.daily_limit = limit;
            }
        }
        
        if let Ok(level) = std::env::var("RUST_LOG") {
            config.logging.level = level;
        }
        
        if let Ok(headless) = std::env::var("HEADLESS") {
            config.browser.headless = headless.to_lowercase() == "true";
        }
        
        info!("Configuration loaded from environment");
        Ok(config)
    }
    
    /// Load configuration from file with env overrides
    pub fn load<P: AsRef<Path>>(path: Option<P>) -> Result<Self> {
        let mut config = if let Some(path) = path {
            if path.as_ref().exists() {
                Self::from_file(path)?
            } else {
                warn!("Config file not found, using defaults");
                Self::default()
            }
        } else {
            Self::default()
        };
        
        // Apply environment overrides
        if let Ok(url) = std::env::var("CHROME_DRIVER_URL") {
            config.browser.driver_url = url;
        }
        
        if let Ok(key) = std::env::var("OPENAI_API_KEY") {
            config.llm.api_key = Some(key);
        }
        
        if let Ok(budget) = std::env::var("DAILY_BUDGET") {
            if let Ok(limit) = budget.parse() {
                config.budget.daily_limit = limit;
            }
        }
        
        if let Ok(level) = std::env::var("RUST_LOG") {
            config.logging.level = level;
        }
        
        Ok(config)
    }
    
    /// Save configuration to file
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = if path.as_ref().extension()
            .and_then(|s| s.to_str())
            .map(|s| s == "json")
            .unwrap_or(false)
        {
            serde_json::to_string_pretty(self)?
        } else {
            serde_yaml::to_string(self)?
        };
        
        fs::write(path, content)?;
        info!("Configuration saved to file");
        Ok(())
    }
    
    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        // Check required fields
        if self.browser.driver_url.is_empty() {
            return Err(anyhow::anyhow!("Chrome driver URL is required"));
        }
        
        // Validate budget
        if self.budget.daily_limit <= 0.0 {
            return Err(anyhow::anyhow!("Daily budget must be positive"));
        }
        
        if self.budget.warning_threshold < 0.0 || self.budget.warning_threshold > 1.0 {
            return Err(anyhow::anyhow!("Warning threshold must be between 0 and 1"));
        }
        
        // Validate pool settings
        if self.pool.max_size == 0 {
            return Err(anyhow::anyhow!("Pool max size must be at least 1"));
        }
        
        // Validate cache settings
        if self.cache.llm_max_size == 0 || self.cache.workflow_max_size == 0 {
            return Err(anyhow::anyhow!("Cache max size must be at least 1"));
        }
        
        Ok(())
    }
    
    /// Get browser pool configuration as durations
    pub fn pool_durations(&self) -> (Duration, Duration) {
        (
            Duration::from_secs(self.pool.idle_timeout),
            Duration::from_secs(self.pool.max_lifetime),
        )
    }
    
    /// Get cache TTL durations
    pub fn cache_durations(&self) -> (Duration, Duration) {
        (
            Duration::from_secs(self.cache.llm_ttl),
            Duration::from_secs(self.cache.workflow_ttl),
        )
    }
    
    // Compatibility methods for existing code
    pub fn chrome_driver_path(&self) -> String {
        self.browser.driver_url.clone()
    }
    
    pub fn daily_budget(&self) -> f64 {
        self.budget.daily_limit
    }
    
    pub fn openai_api_key(&self) -> Option<String> {
        self.llm.api_key.clone()
    }
}

/// Global configuration instance
lazy_static::lazy_static! {
    pub static ref CONFIG: Config = {
        Config::load(Some("config.yaml")).unwrap_or_else(|e| {
            warn!("Failed to load config: {}, using defaults", e);
            Config::default()
        })
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.browser.driver_url, "http://localhost:9515");
        assert_eq!(config.budget.daily_limit, 5.0);
    }
    
    #[test]
    fn test_config_validation() {
        let mut config = Config::default();
        assert!(config.validate().is_ok());
        
        config.budget.daily_limit = -1.0;
        assert!(config.validate().is_err());
        
        config = Config::default();
        config.budget.warning_threshold = 1.5;
        assert!(config.validate().is_err());
    }
    
    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        
        // Test JSON serialization
        let json = serde_json::to_string(&config).unwrap();
        let loaded: Config = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.browser.driver_url, config.browser.driver_url);
        
        // Test YAML serialization
        let yaml = serde_yaml::to_string(&config).unwrap();
        let loaded: Config = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(loaded.browser.driver_url, config.browser.driver_url);
    }
}