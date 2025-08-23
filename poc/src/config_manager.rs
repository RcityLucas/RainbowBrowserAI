//! Configuration Management System
//!
//! This module provides comprehensive configuration management for the
//! RainbowBrowserAI system, including environment-based configuration,
//! hot reloading, validation, and secure secrets management.

use anyhow::{Result, Context};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::fs;
use tracing::{info, warn, debug, error};
use uuid::Uuid;

use crate::llm_integration::LLMConfig;
use crate::error_recovery::ErrorRecoveryConfig;
use crate::health_monitor::HealthMonitorConfig;

/// Configuration manager for centralized config handling
pub struct ConfigManager {
    /// Configuration sources and their priorities
    config_sources: Arc<RwLock<Vec<ConfigSource>>>,
    /// Merged configuration cache
    config_cache: Arc<RwLock<MasterConfiguration>>,
    /// Configuration validation rules
    validators: Arc<RwLock<Vec<Box<dyn ConfigValidator + Send + Sync>>>>,
    /// Hot reload configuration
    hot_reload_config: Arc<RwLock<HotReloadConfig>>,
    /// Configuration change listeners
    change_listeners: Arc<RwLock<Vec<Box<dyn ConfigChangeListener + Send + Sync>>>>,
    /// Secrets manager
    secrets_manager: Arc<RwLock<SecretsManager>>,
    /// Configuration manager settings
    manager_config: Arc<RwLock<ConfigManagerConfig>>,
}

/// Master configuration structure
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MasterConfiguration {
    /// Application-level configuration
    pub application: ApplicationConfig,
    /// Browser automation configuration
    pub browser: BrowserConfig,
    /// LLM integration configuration
    pub llm: LLMConfig,
    /// Error recovery configuration
    pub error_recovery: ErrorRecoveryConfig,
    /// Health monitoring configuration
    pub health_monitor: HealthMonitorConfig,
    /// API configuration
    pub api: ApiConfig,
    /// Logging configuration
    pub logging: LoggingConfig,
    /// Performance configuration
    pub performance: PerformanceConfig,
    /// Security configuration
    pub security: SecurityConfig,
    /// Environment-specific overrides
    pub environment_overrides: HashMap<String, serde_json::Value>,
    /// Configuration metadata
    pub metadata: ConfigMetadata,
}

/// Application-level configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationConfig {
    /// Application name
    pub name: String,
    /// Application version
    pub version: String,
    /// Environment (development, staging, production)
    pub environment: String,
    /// Debug mode
    pub debug_mode: bool,
    /// Data directory
    pub data_dir: PathBuf,
    /// Temporary directory
    pub temp_dir: PathBuf,
    /// Plugin directory
    pub plugin_dir: PathBuf,
    /// Maximum concurrent operations
    pub max_concurrent_operations: usize,
    /// Request timeout (milliseconds)
    pub request_timeout_ms: u64,
}

impl Default for ApplicationConfig {
    fn default() -> Self {
        Self {
            name: "RainbowBrowserAI".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            environment: "development".to_string(),
            debug_mode: cfg!(debug_assertions),
            data_dir: PathBuf::from("./data"),
            temp_dir: PathBuf::from("./tmp"),
            plugin_dir: PathBuf::from("./plugins"),
            max_concurrent_operations: 10,
            request_timeout_ms: 30000,
        }
    }
}

/// Browser configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserConfig {
    /// Browser pool size
    pub pool_size: usize,
    /// Default viewport width
    pub default_viewport_width: u32,
    /// Default viewport height
    pub default_viewport_height: u32,
    /// Browser timeout (milliseconds)
    pub browser_timeout_ms: u64,
    /// Enable headless mode
    pub headless_mode: bool,
    /// Chrome binary path (optional)
    pub chrome_binary_path: Option<PathBuf>,
    /// Chrome user data directory
    pub user_data_dir: PathBuf,
    /// Additional Chrome flags
    pub chrome_flags: Vec<String>,
    /// Screenshot configuration
    pub screenshot: ScreenshotConfig,
}

impl Default for BrowserConfig {
    fn default() -> Self {
        Self {
            pool_size: 5,
            default_viewport_width: 1920,
            default_viewport_height: 1080,
            browser_timeout_ms: 30000,
            headless_mode: true,
            chrome_binary_path: None,
            user_data_dir: PathBuf::from("./browser-data"),
            chrome_flags: vec![
                "--no-sandbox".to_string(),
                "--disable-dev-shm-usage".to_string(),
                "--disable-gpu".to_string(),
            ],
            screenshot: ScreenshotConfig::default(),
        }
    }
}

/// Screenshot configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenshotConfig {
    /// Default screenshot format
    pub format: String,
    /// Default quality (for JPEG)
    pub quality: u8,
    /// Full page screenshots by default
    pub full_page_default: bool,
    /// Screenshot directory
    pub output_dir: PathBuf,
    /// Maximum screenshot size (bytes)
    pub max_size_bytes: usize,
}

impl Default for ScreenshotConfig {
    fn default() -> Self {
        Self {
            format: "png".to_string(),
            quality: 90,
            full_page_default: true,
            output_dir: PathBuf::from("./screenshots"),
            max_size_bytes: 10 * 1024 * 1024, // 10MB
        }
    }
}

/// API configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    /// API server host
    pub host: String,
    /// API server port
    pub port: u16,
    /// Enable CORS
    pub enable_cors: bool,
    /// API rate limiting
    pub rate_limiting: RateLimitingConfig,
    /// API key authentication
    pub api_key_auth: bool,
    /// JWT configuration
    pub jwt: JwtConfig,
    /// Request size limits
    pub request_limits: RequestLimitsConfig,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 3000,
            enable_cors: true,
            rate_limiting: RateLimitingConfig::default(),
            api_key_auth: false,
            jwt: JwtConfig::default(),
            request_limits: RequestLimitsConfig::default(),
        }
    }
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitingConfig {
    /// Enable rate limiting
    pub enabled: bool,
    /// Requests per minute
    pub requests_per_minute: u32,
    /// Burst capacity
    pub burst_capacity: u32,
    /// Rate limiting window (seconds)
    pub window_seconds: u32,
}

impl Default for RateLimitingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            requests_per_minute: 60,
            burst_capacity: 10,
            window_seconds: 60,
        }
    }
}

/// JWT configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtConfig {
    /// JWT secret key (will be loaded from secrets manager)
    #[serde(skip)]
    pub secret_key: Option<String>,
    /// Token expiration time (seconds)
    pub expiration_seconds: u64,
    /// Issuer
    pub issuer: String,
    /// Audience
    pub audience: String,
}

impl Default for JwtConfig {
    fn default() -> Self {
        Self {
            secret_key: None,
            expiration_seconds: 3600, // 1 hour
            issuer: "RainbowBrowserAI".to_string(),
            audience: "api-clients".to_string(),
        }
    }
}

/// Request limits configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestLimitsConfig {
    /// Maximum request body size (bytes)
    pub max_body_size: usize,
    /// Maximum JSON payload size (bytes)
    pub max_json_payload: usize,
    /// Request timeout (milliseconds)
    pub timeout_ms: u64,
}

impl Default for RequestLimitsConfig {
    fn default() -> Self {
        Self {
            max_body_size: 10 * 1024 * 1024, // 10MB
            max_json_payload: 1024 * 1024,   // 1MB
            timeout_ms: 30000,               // 30 seconds
        }
    }
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level
    pub level: String,
    /// Log format (json, pretty, compact)
    pub format: String,
    /// Log output targets
    pub targets: Vec<LogTarget>,
    /// Log rotation
    pub rotation: LogRotationConfig,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: "pretty".to_string(),
            targets: vec![LogTarget::Console, LogTarget::File { path: PathBuf::from("./logs/app.log") }],
            rotation: LogRotationConfig::default(),
        }
    }
}

/// Log output targets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogTarget {
    Console,
    File { path: PathBuf },
    Syslog,
    RemoteEndpoint { url: String },
}

/// Log rotation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogRotationConfig {
    /// Enable log rotation
    pub enabled: bool,
    /// Maximum file size (bytes)
    pub max_file_size: usize,
    /// Maximum number of files to keep
    pub max_files: u32,
    /// Rotation schedule
    pub schedule: RotationSchedule,
}

impl Default for LogRotationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_file_size: 100 * 1024 * 1024, // 100MB
            max_files: 10,
            schedule: RotationSchedule::Daily,
        }
    }
}

/// Log rotation schedules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RotationSchedule {
    Hourly,
    Daily,
    Weekly,
    Monthly,
}

/// Performance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Thread pool size
    pub thread_pool_size: Option<usize>,
    /// Memory limits
    pub memory_limits: MemoryLimitsConfig,
    /// Cache configuration
    pub cache: CacheConfig,
    /// Optimization settings
    pub optimizations: OptimizationConfig,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            thread_pool_size: None, // Use default
            memory_limits: MemoryLimitsConfig::default(),
            cache: CacheConfig::default(),
            optimizations: OptimizationConfig::default(),
        }
    }
}

/// Memory limits configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryLimitsConfig {
    /// Maximum heap size (bytes)
    pub max_heap_size: Option<usize>,
    /// Memory warning threshold (percentage)
    pub warning_threshold_percent: f64,
    /// Memory critical threshold (percentage)
    pub critical_threshold_percent: f64,
    /// Enable memory monitoring
    pub enable_monitoring: bool,
}

impl Default for MemoryLimitsConfig {
    fn default() -> Self {
        Self {
            max_heap_size: None, // No limit
            warning_threshold_percent: 80.0,
            critical_threshold_percent: 90.0,
            enable_monitoring: true,
        }
    }
}

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Enable caching
    pub enabled: bool,
    /// Cache size limit (entries)
    pub max_entries: usize,
    /// Cache TTL (seconds)
    pub ttl_seconds: u64,
    /// Cache cleanup interval (seconds)
    pub cleanup_interval_seconds: u64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_entries: 1000,
            ttl_seconds: 3600, // 1 hour
            cleanup_interval_seconds: 300, // 5 minutes
        }
    }
}

/// Optimization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationConfig {
    /// Enable parallel processing
    pub enable_parallel_processing: bool,
    /// Enable request batching
    pub enable_request_batching: bool,
    /// Enable compression
    pub enable_compression: bool,
    /// Compression level (0-9)
    pub compression_level: u32,
}

impl Default for OptimizationConfig {
    fn default() -> Self {
        Self {
            enable_parallel_processing: true,
            enable_request_batching: true,
            enable_compression: true,
            compression_level: 6,
        }
    }
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Enable security headers
    pub enable_security_headers: bool,
    /// HTTPS configuration
    pub https: HttpsConfig,
    /// Content Security Policy
    pub csp: CspConfig,
    /// Input validation
    pub input_validation: InputValidationConfig,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            enable_security_headers: true,
            https: HttpsConfig::default(),
            csp: CspConfig::default(),
            input_validation: InputValidationConfig::default(),
        }
    }
}

/// HTTPS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpsConfig {
    /// Enable HTTPS
    pub enabled: bool,
    /// Certificate path
    pub cert_path: Option<PathBuf>,
    /// Private key path
    pub key_path: Option<PathBuf>,
    /// Redirect HTTP to HTTPS
    pub redirect_http: bool,
}

impl Default for HttpsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            cert_path: None,
            key_path: None,
            redirect_http: true,
        }
    }
}

/// Content Security Policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CspConfig {
    /// Enable CSP
    pub enabled: bool,
    /// CSP directives
    pub directives: HashMap<String, Vec<String>>,
    /// Report-only mode
    pub report_only: bool,
}

impl Default for CspConfig {
    fn default() -> Self {
        let mut directives = HashMap::new();
        directives.insert("default-src".to_string(), vec!["'self'".to_string()]);
        directives.insert("script-src".to_string(), vec!["'self'".to_string(), "'unsafe-inline'".to_string()]);
        directives.insert("style-src".to_string(), vec!["'self'".to_string(), "'unsafe-inline'".to_string()]);
        
        Self {
            enabled: true,
            directives,
            report_only: false,
        }
    }
}

/// Input validation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputValidationConfig {
    /// Enable strict validation
    pub strict_validation: bool,
    /// Maximum string length
    pub max_string_length: usize,
    /// Allowed file extensions for uploads
    pub allowed_file_extensions: Vec<String>,
    /// Maximum file size (bytes)
    pub max_file_size: usize,
}

impl Default for InputValidationConfig {
    fn default() -> Self {
        Self {
            strict_validation: true,
            max_string_length: 10000,
            allowed_file_extensions: vec![
                "png".to_string(),
                "jpg".to_string(),
                "jpeg".to_string(),
                "pdf".to_string(),
                "txt".to_string(),
            ],
            max_file_size: 10 * 1024 * 1024, // 10MB
        }
    }
}

/// Configuration metadata
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConfigMetadata {
    /// Configuration version
    pub version: String,
    /// Last updated timestamp
    pub last_updated: Option<DateTime<Utc>>,
    /// Configuration checksum
    pub checksum: Option<String>,
    /// Source information
    pub sources: Vec<String>,
    /// Validation status
    pub validation_status: ValidationStatus,
}

/// Configuration validation status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationStatus {
    Valid,
    Warning { messages: Vec<String> },
    Error { errors: Vec<String> },
    NotValidated,
}

impl Default for ValidationStatus {
    fn default() -> Self {
        ValidationStatus::NotValidated
    }
}

/// Configuration sources
#[derive(Debug, Clone)]
pub struct ConfigSource {
    /// Source name
    pub name: String,
    /// Source type
    pub source_type: ConfigSourceType,
    /// Source priority (higher = more priority)
    pub priority: u32,
    /// Watch for changes
    pub watch_changes: bool,
    /// Last loaded timestamp
    pub last_loaded: Option<DateTime<Utc>>,
}

/// Configuration source types
#[derive(Debug, Clone)]
pub enum ConfigSourceType {
    /// File-based configuration
    File { path: PathBuf },
    /// Environment variables
    Environment { prefix: String },
    /// Command line arguments
    CommandLine,
    /// Remote configuration server
    Remote { url: String, auth_token: Option<String> },
    /// In-memory configuration
    Memory { config: serde_json::Value },
}

/// Hot reload configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotReloadConfig {
    /// Enable hot reloading
    pub enabled: bool,
    /// File watch debounce time (milliseconds)
    pub debounce_ms: u64,
    /// Validation on reload
    pub validate_on_reload: bool,
    /// Backup configuration on reload
    pub backup_on_reload: bool,
}

impl Default for HotReloadConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            debounce_ms: 1000,
            validate_on_reload: true,
            backup_on_reload: true,
        }
    }
}

/// Configuration manager settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigManagerConfig {
    /// Configuration file paths to load
    pub config_files: Vec<PathBuf>,
    /// Environment variable prefix
    pub env_prefix: String,
    /// Enable configuration validation
    pub enable_validation: bool,
    /// Configuration backup directory
    pub backup_dir: PathBuf,
    /// Maximum number of backups to keep
    pub max_backups: u32,
}

impl Default for ConfigManagerConfig {
    fn default() -> Self {
        Self {
            config_files: vec![
                PathBuf::from("config.yaml"),
                PathBuf::from("config.json"),
                PathBuf::from("config.toml"),
            ],
            env_prefix: "RAINBOW_".to_string(),
            enable_validation: true,
            backup_dir: PathBuf::from("./config-backups"),
            max_backups: 10,
        }
    }
}

/// Secrets manager for secure credential storage
pub struct SecretsManager {
    /// Encrypted secrets storage
    secrets: HashMap<String, String>,
    /// Secrets configuration
    config: SecretsConfig,
}

/// Secrets configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretsConfig {
    /// Secrets storage backend
    pub backend: SecretsBackend,
    /// Encryption key path
    pub encryption_key_path: Option<PathBuf>,
    /// Auto-refresh secrets interval (seconds)
    pub refresh_interval_seconds: Option<u64>,
}

/// Secrets storage backends
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecretsBackend {
    /// File-based secrets (encrypted)
    File { path: PathBuf },
    /// Environment variables
    Environment,
    /// AWS Secrets Manager
    AwsSecretsManager { region: String },
    /// HashiCorp Vault
    Vault { url: String, auth_method: String },
    /// Azure Key Vault
    AzureKeyVault { vault_url: String },
}

impl Default for SecretsConfig {
    fn default() -> Self {
        Self {
            backend: SecretsBackend::Environment,
            encryption_key_path: None,
            refresh_interval_seconds: Some(3600), // 1 hour
        }
    }
}

/// Configuration validation trait
pub trait ConfigValidator {
    /// Validate configuration
    fn validate(&self, config: &MasterConfiguration) -> Result<ValidationResult>;
    
    /// Get validator name
    fn name(&self) -> &str;
}

/// Validation result
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Validation passed
    pub valid: bool,
    /// Warning messages
    pub warnings: Vec<String>,
    /// Error messages
    pub errors: Vec<String>,
    /// Validator name
    pub validator_name: String,
}

/// Configuration change listener trait
pub trait ConfigChangeListener {
    /// Handle configuration change
    fn on_config_changed(&self, old_config: &MasterConfiguration, new_config: &MasterConfiguration);
    
    /// Get listener name
    fn name(&self) -> &str;
}

impl ConfigManager {
    /// Create new configuration manager
    pub async fn new(manager_config: ConfigManagerConfig) -> Result<Self> {
        let manager = Self {
            config_sources: Arc::new(RwLock::new(Vec::new())),
            config_cache: Arc::new(RwLock::new(MasterConfiguration::default())),
            validators: Arc::new(RwLock::new(Vec::new())),
            hot_reload_config: Arc::new(RwLock::new(HotReloadConfig::default())),
            change_listeners: Arc::new(RwLock::new(Vec::new())),
            secrets_manager: Arc::new(RwLock::new(SecretsManager::new().await?)),
            manager_config: Arc::new(RwLock::new(manager_config)),
        };

        // Initialize configuration sources
        manager.initialize_default_sources().await?;
        
        // Register default validators
        manager.register_default_validators().await?;

        // Load initial configuration
        manager.reload_configuration().await?;

        info!("📋 Configuration Manager initialized");
        Ok(manager)
    }

    /// Add configuration source
    pub async fn add_config_source(&self, source: ConfigSource) -> Result<()> {
        let mut sources = self.config_sources.write().await;
        sources.push(source);
        
        // Sort by priority (highest first)
        sources.sort_by(|a, b| b.priority.cmp(&a.priority));
        
        info!("Added configuration source: {}", sources.last().unwrap().name);
        Ok(())
    }

    /// Load configuration from all sources
    pub async fn reload_configuration(&self) -> Result<()> {
        info!("🔄 Reloading configuration from all sources...");
        
        let sources = self.config_sources.read().await;
        let mut merged_config = MasterConfiguration::default();
        let mut source_names = Vec::new();

        // Load from each source in priority order
        for source in sources.iter() {
            match self.load_from_source(source).await {
                Ok(config_value) => {
                    self.merge_configuration(&mut merged_config, &config_value).await?;
                    source_names.push(source.name.clone());
                    debug!("Loaded configuration from source: {}", source.name);
                }
                Err(e) => {
                    warn!("Failed to load configuration from source {}: {}", source.name, e);
                }
            }
        }

        // Update metadata
        merged_config.metadata.version = env!("CARGO_PKG_VERSION").to_string();
        merged_config.metadata.last_updated = Some(Utc::now());
        merged_config.metadata.sources = source_names;

        // Validate configuration
        if self.manager_config.read().await.enable_validation {
            let validation_result = self.validate_configuration(&merged_config).await?;
            merged_config.metadata.validation_status = if validation_result.valid {
                ValidationStatus::Valid
            } else if !validation_result.errors.is_empty() {
                ValidationStatus::Error { errors: validation_result.errors }
            } else {
                ValidationStatus::Warning { messages: validation_result.warnings }
            };
        }

        // Update cached configuration
        let old_config = self.config_cache.read().await.clone();
        *self.config_cache.write().await = merged_config.clone();

        // Notify change listeners
        self.notify_change_listeners(&old_config, &merged_config).await;

        info!("✅ Configuration reloaded successfully");
        Ok(())
    }

    /// Get current configuration
    pub async fn get_config(&self) -> MasterConfiguration {
        self.config_cache.read().await.clone()
    }

    /// Get specific configuration section
    pub async fn get_section<T>(&self, section: &str) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let config = self.config_cache.read().await;
        let config_value = serde_json::to_value(&*config)?;
        
        let section_value = config_value
            .get(section)
            .context(format!("Configuration section '{}' not found", section))?;
            
        let section_config: T = serde_json::from_value(section_value.clone())
            .context(format!("Failed to deserialize section '{}'", section))?;
            
        Ok(section_config)
    }

    /// Update configuration section
    pub async fn update_section<T>(&self, section: &str, new_config: T) -> Result<()>
    where
        T: Serialize,
    {
        let mut config = self.config_cache.write().await;
        let mut config_value = serde_json::to_value(&*config)?;
        
        let new_section_value = serde_json::to_value(new_config)?;
        config_value[section] = new_section_value;
        
        *config = serde_json::from_value(config_value)
            .context("Failed to update configuration section")?;
        
        // Update metadata
        config.metadata.last_updated = Some(Utc::now());
        
        info!("Updated configuration section: {}", section);
        Ok(())
    }

    /// Register configuration validator
    pub async fn register_validator(&self, validator: Box<dyn ConfigValidator + Send + Sync>) -> Result<()> {
        let mut validators = self.validators.write().await;
        let validator_name = validator.name().to_string();
        validators.push(validator);
        info!("Registered configuration validator: {}", validator_name);
        Ok(())
    }

    /// Register configuration change listener
    pub async fn register_change_listener(&self, listener: Box<dyn ConfigChangeListener + Send + Sync>) -> Result<()> {
        let mut listeners = self.change_listeners.write().await;
        let listener_name = listener.name().to_string();
        listeners.push(listener);
        info!("Registered configuration change listener: {}", listener_name);
        Ok(())
    }

    /// Save current configuration to file
    pub async fn save_configuration(&self, path: &Path) -> Result<()> {
        let config = self.config_cache.read().await;
        let config_yaml = serde_yaml::to_string(&*config)
            .context("Failed to serialize configuration to YAML")?;
        
        fs::write(path, config_yaml).await
            .context(format!("Failed to write configuration to {}", path.display()))?;
        
        info!("Configuration saved to: {}", path.display());
        Ok(())
    }

    /// Load configuration from file
    pub async fn load_from_file(&self, path: &Path) -> Result<MasterConfiguration> {
        let content = fs::read_to_string(path).await
            .context(format!("Failed to read configuration file: {}", path.display()))?;
        
        let config = if path.extension().and_then(|s| s.to_str()) == Some("yaml") || 
                       path.extension().and_then(|s| s.to_str()) == Some("yml") {
            serde_yaml::from_str(&content)
                .context("Failed to parse YAML configuration")?
        } else if path.extension().and_then(|s| s.to_str()) == Some("json") {
            serde_json::from_str(&content)
                .context("Failed to parse JSON configuration")?
        } else {
            return Err(anyhow::anyhow!("Unsupported configuration file format"));
        };
        
        Ok(config)
    }

    /// Initialize default configuration sources
    async fn initialize_default_sources(&self) -> Result<()> {
        let manager_config = self.manager_config.read().await;
        
        // Add file sources
        for config_file in &manager_config.config_files {
            if config_file.exists() {
                let source = ConfigSource {
                    name: format!("file:{}", config_file.display()),
                    source_type: ConfigSourceType::File { path: config_file.clone() },
                    priority: 100,
                    watch_changes: true,
                    last_loaded: None,
                };
                self.add_config_source(source).await?;
            }
        }

        // Add environment variable source
        let env_source = ConfigSource {
            name: "environment".to_string(),
            source_type: ConfigSourceType::Environment { 
                prefix: manager_config.env_prefix.clone() 
            },
            priority: 200,
            watch_changes: false,
            last_loaded: None,
        };
        self.add_config_source(env_source).await?;

        Ok(())
    }

    /// Register default validators
    async fn register_default_validators(&self) -> Result<()> {
        // Add basic configuration validator
        self.register_validator(Box::new(BasicConfigValidator)).await?;
        Ok(())
    }

    /// Load configuration from a specific source
    async fn load_from_source(&self, source: &ConfigSource) -> Result<serde_json::Value> {
        match &source.source_type {
            ConfigSourceType::File { path } => {
                let config = self.load_from_file(path).await?;
                Ok(serde_json::to_value(config)?)
            }
            ConfigSourceType::Environment { prefix } => {
                self.load_from_environment(prefix).await
            }
            ConfigSourceType::Memory { config } => {
                Ok(config.clone())
            }
            _ => {
                Err(anyhow::anyhow!("Unsupported config source type"))
            }
        }
    }

    /// Load configuration from environment variables
    async fn load_from_environment(&self, prefix: &str) -> Result<serde_json::Value> {
        let mut config = serde_json::Map::new();
        
        for (key, value) in std::env::vars() {
            if key.starts_with(prefix) {
                let config_key = key[prefix.len()..].to_lowercase().replace('_', ".");
                let json_value = if value.parse::<i64>().is_ok() {
                    serde_json::Value::Number(value.parse::<i64>().unwrap().into())
                } else if value.parse::<f64>().is_ok() {
                    serde_json::Value::Number(serde_json::Number::from_f64(value.parse().unwrap()).unwrap())
                } else if value.parse::<bool>().is_ok() {
                    serde_json::Value::Bool(value.parse().unwrap())
                } else {
                    serde_json::Value::String(value)
                };
                
                self.set_nested_value(&mut config, &config_key, json_value);
            }
        }
        
        Ok(serde_json::Value::Object(config))
    }

    /// Set nested value in JSON object
    fn set_nested_value(&self, object: &mut serde_json::Map<String, serde_json::Value>, 
                       key_path: &str, value: serde_json::Value) {
        let keys: Vec<&str> = key_path.split('.').collect();
        
        if keys.is_empty() {
            return;
        }
        
        // Handle the case where there's only one key
        if keys.len() == 1 {
            object.insert(keys[0].to_string(), value);
            return;
        }
        
        // Build the nested structure recursively
        self.ensure_nested_path(object, &keys, value);
    }
    
    fn ensure_nested_path(&self, object: &mut serde_json::Map<String, serde_json::Value>, 
                         keys: &[&str], value: serde_json::Value) {
        if keys.is_empty() {
            return;
        }
        
        if keys.len() == 1 {
            object.insert(keys[0].to_string(), value);
            return;
        }
        
        let key = keys[0];
        let remaining_keys = &keys[1..];
        
        // Ensure the key exists and is an object
        let entry = object.entry(key.to_string()).or_insert_with(|| {
            serde_json::Value::Object(serde_json::Map::new())
        });
        
        // Convert to object if it's not already
        if !entry.is_object() {
            *entry = serde_json::Value::Object(serde_json::Map::new());
        }
        
        // Recursively set in the nested object
        if let serde_json::Value::Object(nested_obj) = entry {
            self.ensure_nested_path(nested_obj, remaining_keys, value);
        }
    }

    /// Merge configurations with priority
    async fn merge_configuration(&self, base: &mut MasterConfiguration, 
                                overlay: &serde_json::Value) -> Result<()> {
        let mut base_value = serde_json::to_value(&mut *base)?;
        self.merge_json_values(&mut base_value, overlay);
        *base = serde_json::from_value(base_value)?;
        Ok(())
    }

    /// Recursively merge JSON values
    fn merge_json_values(&self, base: &mut serde_json::Value, overlay: &serde_json::Value) {
        match (base, overlay) {
            (serde_json::Value::Object(base_obj), serde_json::Value::Object(overlay_obj)) => {
                for (key, value) in overlay_obj {
                    if let Some(base_value) = base_obj.get_mut(key) {
                        self.merge_json_values(base_value, value);
                    } else {
                        base_obj.insert(key.clone(), value.clone());
                    }
                }
            },
            (base_val, overlay_val) => {
                *base_val = overlay_val.clone();
            }
        }
    }

    /// Validate configuration using all registered validators
    async fn validate_configuration(&self, config: &MasterConfiguration) -> Result<ValidationResult> {
        let validators = self.validators.read().await;
        let mut combined_result = ValidationResult {
            valid: true,
            warnings: Vec::new(),
            errors: Vec::new(),
            validator_name: "combined".to_string(),
        };

        for validator in validators.iter() {
            match validator.validate(config) {
                Ok(result) => {
                    if !result.valid {
                        combined_result.valid = false;
                    }
                    combined_result.warnings.extend(result.warnings);
                    combined_result.errors.extend(result.errors);
                }
                Err(e) => {
                    error!("Validation error in {}: {}", validator.name(), e);
                    combined_result.valid = false;
                    combined_result.errors.push(format!("Validator {} failed: {}", validator.name(), e));
                }
            }
        }

        Ok(combined_result)
    }

    /// Notify all change listeners
    async fn notify_change_listeners(&self, old_config: &MasterConfiguration, new_config: &MasterConfiguration) {
        let listeners = self.change_listeners.read().await;
        for listener in listeners.iter() {
            listener.on_config_changed(old_config, new_config);
        }
    }
}

impl SecretsManager {
    async fn new() -> Result<Self> {
        Ok(Self {
            secrets: HashMap::new(),
            config: SecretsConfig::default(),
        })
    }

    /// Get secret by key
    pub fn get_secret(&self, key: &str) -> Option<&String> {
        self.secrets.get(key)
    }

    /// Set secret
    pub fn set_secret(&mut self, key: String, value: String) {
        self.secrets.insert(key, value);
    }

    /// Load secrets from backend
    pub async fn load_secrets(&mut self) -> Result<()> {
        match &self.config.backend {
            SecretsBackend::Environment => {
                self.load_from_environment().await?;
            }
            _ => {
                warn!("Secrets backend not yet implemented");
            }
        }
        Ok(())
    }

    async fn load_from_environment(&mut self) -> Result<()> {
        for (key, value) in std::env::vars() {
            if key.starts_with("SECRET_") || key.contains("KEY") || key.contains("TOKEN") {
                self.secrets.insert(key, value);
            }
        }
        Ok(())
    }
}

/// Basic configuration validator implementation
struct BasicConfigValidator;

impl ConfigValidator for BasicConfigValidator {
    fn validate(&self, config: &MasterConfiguration) -> Result<ValidationResult> {
        let mut result = ValidationResult {
            valid: true,
            warnings: Vec::new(),
            errors: Vec::new(),
            validator_name: "basic".to_string(),
        };

        // Validate application config
        if config.application.name.is_empty() {
            result.valid = false;
            result.errors.push("Application name cannot be empty".to_string());
        }

        // Validate API config
        if config.api.port < 1024 {
            result.warnings.push("API port is below 1024, may require elevated privileges".to_string());
        }

        // Validate browser config
        if config.browser.pool_size == 0 {
            result.valid = false;
            result.errors.push("Browser pool size must be greater than 0".to_string());
        }

        // Validate directories exist or can be created
        let dirs = [
            &config.application.data_dir,
            &config.application.temp_dir,
            &config.application.plugin_dir,
        ];

        for dir in &dirs {
            if let Some(parent) = dir.parent() {
                if !parent.exists() {
                    result.warnings.push(format!("Parent directory does not exist: {}", parent.display()));
                }
            }
        }

        Ok(result)
    }

    fn name(&self) -> &str {
        "basic"
    }
}

/// Create configuration manager with default settings
pub async fn create_config_manager() -> Result<ConfigManager> {
    let manager_config = ConfigManagerConfig::default();
    ConfigManager::new(manager_config).await
}

/// Create configuration manager with custom settings
pub async fn create_custom_config_manager(manager_config: ConfigManagerConfig) -> Result<ConfigManager> {
    ConfigManager::new(manager_config).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_config_manager_creation() {
        let manager = create_config_manager().await;
        assert!(manager.is_ok());
    }

    #[tokio::test]
    async fn test_configuration_loading() {
        let temp_dir = tempdir().unwrap();
        let config_file = temp_dir.path().join("test_config.yaml");
        
        let test_config = MasterConfiguration::default();
        let config_yaml = serde_yaml::to_string(&test_config).unwrap();
        tokio::fs::write(&config_file, config_yaml).await.unwrap();

        let mut manager_config = ConfigManagerConfig::default();
        manager_config.config_files = vec![config_file];

        let manager = create_custom_config_manager(manager_config).await.unwrap();
        let loaded_config = manager.get_config().await;
        
        assert_eq!(loaded_config.application.name, "RainbowBrowserAI");
    }

    #[tokio::test]
    async fn test_configuration_validation() {
        let manager = create_config_manager().await.unwrap();
        let config = manager.get_config().await;
        
        let validator = BasicConfigValidator;
        let result = validator.validate(&config).unwrap();
        
        assert!(result.valid);
    }
}