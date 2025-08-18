use async_trait::async_trait;
use anyhow::Result;
use crate::{SimpleBrowser, WorkflowStep};
use super::types::*;
use std::time::Duration;
use std::collections::HashMap;

// Type aliases for plugin compatibility
pub type WorkflowAction = WorkflowStep;
pub type WorkflowContext = HashMap<String, serde_json::Value>;

/// Base trait for all plugins
#[async_trait]
pub trait Plugin: Send + Sync {
    /// Plugin name
    fn name(&self) -> &'static str;
    
    /// Plugin version
    fn version(&self) -> &'static str;
    
    /// Plugin description
    fn description(&self) -> &'static str;
    
    /// Plugin type
    fn plugin_type(&self) -> PluginType;
    
    /// Called when plugin is loaded
    async fn on_load(&mut self) -> Result<()> {
        Ok(())
    }
    
    /// Called when plugin is activated
    async fn on_activate(&mut self) -> Result<()> {
        Ok(())
    }
    
    /// Called when plugin is suspended
    async fn on_suspend(&mut self) -> Result<()> {
        Ok(())
    }
    
    /// Called when plugin is resumed
    async fn on_resume(&mut self) -> Result<()> {
        Ok(())
    }
    
    /// Called when plugin is unloaded
    async fn on_unload(&mut self) -> Result<()> {
        Ok(())
    }
    
    /// Health check
    async fn health_check(&self) -> HealthStatus {
        HealthStatus::Healthy
    }
}

/// Plugin trait for workflow action extensions
#[async_trait]
pub trait ActionPlugin: Plugin {
    /// Execute an action
    async fn execute(
        &self,
        action: &WorkflowAction,
        context: &mut WorkflowContext,
        browser: Option<&SimpleBrowser>
    ) -> Result<serde_json::Value>;
    
    /// Validate action configuration
    fn validate_config(&self, config: &serde_json::Value) -> Result<()>;
    
    /// Get action schema
    fn schema(&self) -> serde_json::Value;
    
    /// Get supported action types
    fn supported_actions(&self) -> Vec<&'static str>;
}

/// Plugin trait for data processing
#[async_trait]
pub trait DataProcessorPlugin: Plugin {
    /// Get supported input/output formats
    fn supported_formats(&self) -> Vec<&'static str>;
    
    /// Process data
    async fn process(
        &self,
        input: &serde_json::Value,
        options: &ProcessorOptions
    ) -> Result<serde_json::Value>;
    
    /// Get processor schema
    fn schema(&self) -> ProcessorSchema;
    
    /// Validate input data
    fn validate_input(&self, input: &serde_json::Value, format: &str) -> Result<()>;
}

/// Plugin trait for external service integrations
#[async_trait]
pub trait IntegrationPlugin: Plugin {
    /// Get service type
    fn service_type(&self) -> ServiceType;
    
    /// Authenticate with the service
    async fn authenticate(&self, credentials: &Credentials) -> Result<AuthToken>;
    
    /// Execute a request to the service
    async fn execute_request(&self, request: &IntegrationRequest) -> Result<IntegrationResponse>;
    
    /// Get supported capabilities
    fn capabilities(&self) -> Vec<Capability>;
    
    /// Get rate limits
    fn rate_limits(&self) -> RateLimits;
    
    /// Validate credentials
    fn validate_credentials(&self, credentials: &Credentials) -> Result<()>;
}

/// Plugin trait for UI extensions
#[async_trait]
pub trait UIExtensionPlugin: Plugin {
    /// Get UI routes
    fn routes(&self) -> Vec<UIRoute>;
    
    /// Render a UI component
    async fn render_component(&self, component: &str, props: &serde_json::Value) -> Result<String>;
    
    /// Handle UI action
    async fn handle_action(&self, action: &UIAction) -> Result<UIResponse>;
    
    /// Get static assets
    fn static_assets(&self) -> Vec<StaticAsset>;
    
    /// Get UI dependencies
    fn dependencies(&self) -> Vec<UIDependency>;
    
    /// Validate component props
    fn validate_props(&self, component: &str, props: &serde_json::Value) -> Result<()>;
}

/// Plugin lifecycle management
#[async_trait]
pub trait PluginLifecycle {
    /// Plugin lifecycle state change
    async fn on_state_change(&mut self, old_state: PluginState, new_state: PluginState) -> Result<()>;
    
    /// Plugin configuration change
    async fn on_config_change(&mut self, config: &PluginConfig) -> Result<()>;
    
    /// Plugin error occurred
    async fn on_error(&mut self, error: &str, severity: ErrorSeverity) -> Result<()>;
}

/// Plugin metrics collection
pub trait PluginMetricsCollector {
    /// Record execution time
    fn record_execution(&self, duration: Duration);
    
    /// Record error
    fn record_error(&self, error: &str, severity: ErrorSeverity);
    
    /// Record resource usage
    fn record_resource_usage(&self, memory: u64, cpu: f64);
    
    /// Get metrics
    fn get_metrics(&self) -> PluginHealth;
}

/// Plugin security validation
pub trait PluginSecurity {
    /// Check if plugin has permission
    fn has_permission(&self, permission: &Permission) -> bool;
    
    /// Validate resource usage
    fn validate_resource_usage(&self, memory: u64, cpu: f64) -> Result<()>;
    
    /// Check rate limits
    fn check_rate_limit(&self) -> Result<()>;
    
    /// Validate file access
    fn validate_file_access(&self, path: &std::path::Path, write: bool) -> Result<()>;
}