use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Unique identifier for a plugin
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PluginId(pub String);

impl PluginId {
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    pub fn from_name(name: &str) -> Self {
        Self(name.to_string())
    }
}

impl std::fmt::Display for PluginId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Plugin type classification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PluginType {
    Action,
    DataProcessor,
    Integration,
    UIExtension,
}

/// Plugin lifecycle state
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PluginState {
    Discovered,
    Loading,
    Loaded,
    Active,
    Suspended,
    Unloading,
    Error(String),
}

impl std::fmt::Display for PluginState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PluginState::Discovered => write!(f, "Discovered"),
            PluginState::Loading => write!(f, "Loading"),
            PluginState::Loaded => write!(f, "Loaded"),
            PluginState::Active => write!(f, "Active"),
            PluginState::Suspended => write!(f, "Suspended"),
            PluginState::Unloading => write!(f, "Unloading"),
            PluginState::Error(err) => write!(f, "Error: {}", err),
        }
    }
}

/// Plugin permission types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Permission {
    Network,
    FilesystemRead(PathBuf),
    FilesystemWrite(PathBuf),
    BrowserControl,
    WorkflowModification,
    MetricsAccess,
    ConfigurationRead,
}

/// Resource limits for plugins
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub max_memory_mb: u64,
    pub max_cpu_percent: f64,
    pub max_network_connections: u32,
    pub timeout_seconds: u64,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_mb: 100,
            max_cpu_percent: 10.0,
            max_network_connections: 10,
            timeout_seconds: 30,
        }
    }
}

/// Plugin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    pub plugin_id: String,
    pub enabled: bool,
    pub settings: serde_json::Value,
    pub permissions: Vec<Permission>,
    pub resource_limits: ResourceLimits,
}

/// Plugin manifest (plugin.toml)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub plugin: PluginMetadata,
    pub dependencies: Option<PluginDependencies>,
    pub capabilities: Option<PluginCapabilities>,
    pub configuration: Option<PluginConfigSchema>,
    pub resources: Option<ResourceLimits>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub license: String,
    #[serde(rename = "type")]
    pub plugin_type: PluginType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginDependencies {
    pub runtime_version: String,
    pub system_requirements: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginCapabilities {
    pub actions: Option<Vec<String>>,
    pub permissions: Vec<Permission>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfigSchema {
    pub required_fields: Vec<String>,
    pub optional_fields: Vec<String>,
}

/// Plugin information for runtime use
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    pub id: PluginId,
    pub manifest: PluginManifest,
    pub state: PluginState,
    pub config: Option<PluginConfig>,
    pub loaded_at: Option<DateTime<Utc>>,
    pub last_error: Option<String>,
    pub path: PathBuf,
}

/// Plugin health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Warning(String),
    Critical(String),
    Unknown,
}

/// Plugin health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginHealth {
    pub status: HealthStatus,
    pub last_heartbeat: DateTime<Utc>,
    pub error_count: u64,
    pub memory_usage: u64,
    pub cpu_usage: f64,
    pub response_time: std::time::Duration,
}

/// Plugin metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetrics {
    pub total_plugins: usize,
    pub active_plugins: usize,
    pub failed_plugins: usize,
    pub total_memory_usage: u64,
    pub total_cpu_usage: f64,
    pub total_executions: u64,
    pub average_response_time: std::time::Duration,
}

/// Service type for integration plugins
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ServiceType {
    Communication,
    Storage,
    Database,
    Analytics,
    Monitoring,
    CiCd,
    Authentication,
    Other(String),
}

/// Authentication credentials
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credentials {
    pub auth_type: String,
    pub credentials: HashMap<String, String>,
}

/// Authentication token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthToken {
    pub token: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub refresh_token: Option<String>,
}

/// Rate limits for integration plugins
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimits {
    pub requests_per_minute: u32,
    pub requests_per_hour: u32,
}

/// Capability for integration plugins
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Capability {
    SendMessage,
    UploadFile,
    CreateChannel,
    ReadData,
    WriteData,
    DeleteData,
    ExecuteQuery,
    Other(String),
}

/// Integration request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationRequest {
    pub action: String,
    pub parameters: serde_json::Value,
    pub headers: Option<HashMap<String, String>>,
}

/// Integration response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationResponse {
    pub success: bool,
    pub data: Option<serde_json::Value>,
    pub error: Option<String>,
    pub headers: Option<HashMap<String, String>>,
}

/// Data processor options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessorOptions {
    pub input_format: String,
    pub output_format: String,
    pub settings: serde_json::Value,
}

/// Data processor schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessorSchema {
    pub input_schema: serde_json::Value,
    pub output_schema: serde_json::Value,
    pub options_schema: serde_json::Value,
}

/// UI route for UI extension plugins
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIRoute {
    pub path: String,
    pub method: String,
    pub handler: String,
}

/// UI action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIAction {
    pub action: String,
    pub data: serde_json::Value,
}

/// UI response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIResponse {
    pub success: bool,
    pub data: Option<serde_json::Value>,
    pub redirect: Option<String>,
}

/// Static asset for UI plugins
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaticAsset {
    pub path: String,
    pub content_type: String,
    pub content: Vec<u8>,
}

/// UI dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIDependency {
    pub name: String,
    pub version: String,
    pub url: Option<String>,
}

/// Error severity levels
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ErrorSeverity {
    Low,
    Medium,
    High,
    Critical,
}