use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};
use chrono::Utc;

use super::{
    types::*,
    events::{EventBus, DefaultEventBus, EventPublisher},
    sandbox::SandboxManager,
    registry::PluginRegistry,
};

/// Core plugin manager for the RainbowBrowserAI system
pub struct PluginManager {
    // Core components
    pub registry: Arc<RwLock<PluginRegistry>>,
    loaded_plugins: Arc<RwLock<HashMap<PluginId, LoadedPlugin>>>,
    sandbox_manager: Arc<SandboxManager>,
    event_bus: Arc<dyn EventBus>,
    event_publisher: EventPublisher,
    
    // Configuration
    config: PluginManagerConfig,
}

impl PluginManager {
    /// Create a new plugin manager
    pub async fn new() -> Result<Self> {
        let event_bus: Arc<dyn EventBus> = Arc::new(DefaultEventBus::new());
        let event_publisher = EventPublisher::new(event_bus.clone());
        
        Ok(Self {
            registry: Arc::new(RwLock::new(PluginRegistry::new())),
            loaded_plugins: Arc::new(RwLock::new(HashMap::new())),
            sandbox_manager: Arc::new(SandboxManager::new()),
            event_bus,
            event_publisher,
            config: PluginManagerConfig::default(),
        })
    }

    /// Discover plugins in the specified directory
    pub async fn discover_plugins(&self, path: &Path) -> Result<Vec<PluginId>> {
        let mut registry = self.registry.write().await;
        registry.add_search_path(path.to_path_buf());
        registry.discover_plugins().await
    }

    /// Load a plugin from file path
    pub async fn load_plugin(&self, path: &Path) -> Result<PluginId> {
        // First add to registry if not already present
        let plugin_id = {
            let mut registry = self.registry.write().await;
            match registry.discover_plugins_in_path(path.parent().unwrap_or(Path::new("."))).await {
                Ok(discovered) => {
                    discovered.into_iter().find(|id| {
                        registry.get_plugin(id)
                            .map(|entry| entry.plugin_path == path)
                            .unwrap_or(false)
                    }).ok_or_else(|| anyhow::anyhow!("Plugin not found in registry after discovery"))?
                }
                Err(e) => return Err(anyhow::anyhow!("Failed to add plugin to registry: {}", e)),
            }
        };

        self.load_plugin_by_id(&plugin_id).await?;
        Ok(plugin_id)
    }

    /// Load a plugin by its string ID
    pub async fn load_plugin_by_string_id(&self, plugin_id: &str) -> Result<()> {
        let plugin_id = PluginId(plugin_id.to_string());
        self.load_plugin_by_id(&plugin_id).await
    }

    /// Load a plugin by its ID
    pub async fn load_plugin_by_id(&self, plugin_id: &PluginId) -> Result<()> {
        // Check if already loaded
        {
            let loaded_plugins = self.loaded_plugins.read().await;
            if loaded_plugins.contains_key(plugin_id) {
                return Err(anyhow::anyhow!("Plugin {} is already loaded", plugin_id));
            }
        }

        // Get plugin info from registry
        let manifest = {
            let registry = self.registry.read().await;
            let entry = registry.get_plugin(plugin_id)
                .ok_or_else(|| anyhow::anyhow!("Plugin {} not found in registry", plugin_id))?;
            entry.manifest.clone()
        };

        // Update state to loading
        {
            let mut registry = self.registry.write().await;
            registry.update_plugin_state(plugin_id, PluginState::Loading);
        }

        // Validate plugin
        self.validate_plugin(&manifest).await?;

        // Create sandbox
        let permissions = manifest.capabilities
            .as_ref()
            .map(|caps| caps.permissions.clone())
            .unwrap_or_default();
        let resource_limits = manifest.resources.clone().unwrap_or_default();

        self.sandbox_manager
            .create_sandbox(plugin_id.clone(), permissions, resource_limits.clone())
            .await?;

        // Create plugin configuration
        let config = PluginConfig {
            plugin_id: plugin_id.to_string(),
            enabled: true,
            settings: serde_json::Value::Object(serde_json::Map::new()),
            permissions: manifest.capabilities
                .as_ref()
                .map(|caps| caps.permissions.clone())
                .unwrap_or_default(),
            resource_limits,
        };

        // Create loaded plugin entry
        let loaded_plugin = LoadedPlugin {
            id: plugin_id.clone(),
            manifest: manifest.clone(),
            config,
            state: PluginState::Loaded,
            loaded_at: Utc::now(),
            last_error: None,
            health: PluginHealth {
                status: HealthStatus::Healthy,
                last_heartbeat: Utc::now(),
                error_count: 0,
                memory_usage: 0,
                cpu_usage: 0.0,
                response_time: std::time::Duration::from_millis(0),
            },
        };

        // Add to loaded plugins
        {
            let mut loaded_plugins = self.loaded_plugins.write().await;
            loaded_plugins.insert(plugin_id.clone(), loaded_plugin);
        }

        // Update registry state
        {
            let mut registry = self.registry.write().await;
            registry.update_plugin_state(plugin_id, PluginState::Loaded);
        }

        // Publish event
        self.event_publisher.plugin_loaded(plugin_id.clone()).await?;

        info!("Successfully loaded plugin: {}", plugin_id);
        Ok(())
    }

    /// Unload a plugin by string ID
    pub async fn unload_plugin_by_string_id(&self, plugin_id: &str) -> Result<()> {
        let plugin_id = PluginId(plugin_id.to_string());
        self.unload_plugin(&plugin_id).await
    }

    /// Unload a plugin
    pub async fn unload_plugin(&self, plugin_id: &PluginId) -> Result<()> {
        // Update state to unloading
        {
            let mut loaded_plugins = self.loaded_plugins.write().await;
            if let Some(plugin) = loaded_plugins.get_mut(plugin_id) {
                plugin.state = PluginState::Unloading;
            } else {
                return Err(anyhow::anyhow!("Plugin {} is not loaded", plugin_id));
            }
        }

        // Remove sandbox
        self.sandbox_manager.remove_sandbox(plugin_id).await?;

        // Remove from loaded plugins
        {
            let mut loaded_plugins = self.loaded_plugins.write().await;
            loaded_plugins.remove(plugin_id);
        }

        // Update registry state
        {
            let mut registry = self.registry.write().await;
            registry.update_plugin_state(plugin_id, PluginState::Discovered);
        }

        // Publish event
        self.event_publisher.plugin_unloaded(plugin_id.clone()).await?;

        info!("Successfully unloaded plugin: {}", plugin_id);
        Ok(())
    }

    /// Activate a plugin
    pub async fn activate_plugin(&self, plugin_id: &PluginId) -> Result<()> {
        let mut loaded_plugins = self.loaded_plugins.write().await;
        
        let plugin = loaded_plugins.get_mut(plugin_id)
            .ok_or_else(|| anyhow::anyhow!("Plugin {} is not loaded", plugin_id))?;

        if plugin.state == PluginState::Active {
            return Ok(()); // Already active
        }

        plugin.state = PluginState::Active;
        info!("Activated plugin: {}", plugin_id);
        Ok(())
    }

    /// Suspend a plugin
    pub async fn suspend_plugin(&self, plugin_id: &PluginId) -> Result<()> {
        let mut loaded_plugins = self.loaded_plugins.write().await;
        
        let plugin = loaded_plugins.get_mut(plugin_id)
            .ok_or_else(|| anyhow::anyhow!("Plugin {} is not loaded", plugin_id))?;

        plugin.state = PluginState::Suspended;
        info!("Suspended plugin: {}", plugin_id);
        Ok(())
    }

    /// Get plugin information
    pub async fn get_plugin_info(&self, plugin_id: &PluginId) -> Option<PluginInfo> {
        let registry = self.registry.read().await;
        let loaded_plugins = self.loaded_plugins.read().await;
        
        if let Some(entry) = registry.get_plugin(plugin_id) {
            let mut info = entry.to_plugin_info();
            
            // Add runtime information if loaded
            if let Some(loaded_plugin) = loaded_plugins.get(plugin_id) {
                info.state = loaded_plugin.state.clone();
                info.config = Some(loaded_plugin.config.clone());
                info.loaded_at = Some(loaded_plugin.loaded_at);
                info.last_error = loaded_plugin.last_error.clone();
            }
            
            Some(info)
        } else {
            None
        }
    }

    /// List all plugins
    pub async fn list_plugins(&self) -> Vec<PluginInfo> {
        let registry = self.registry.read().await;
        let loaded_plugins = self.loaded_plugins.read().await;
        
        registry.list_plugins().iter().map(|entry| {
            let mut info = entry.to_plugin_info();
            
            // Add runtime information if loaded
            if let Some(loaded_plugin) = loaded_plugins.get(&entry.id) {
                info.state = loaded_plugin.state.clone();
                info.config = Some(loaded_plugin.config.clone());
                info.loaded_at = Some(loaded_plugin.loaded_at);
                info.last_error = loaded_plugin.last_error.clone();
            }
            
            info
        }).collect()
    }

    /// Configure a plugin by string ID with JSON value
    pub async fn configure_plugin_by_string_id(&self, plugin_id: &str, config: serde_json::Value) -> Result<()> {
        let plugin_id = PluginId(plugin_id.to_string());
        let plugin_config = PluginConfig {
            plugin_id: plugin_id.0.clone(),
            enabled: true,
            settings: config,
            permissions: Vec::new(),
            resource_limits: ResourceLimits::default(),
        };
        self.configure_plugin(&plugin_id, plugin_config).await
    }

    /// Configure a plugin
    pub async fn configure_plugin(&self, plugin_id: &PluginId, config: PluginConfig) -> Result<()> {
        let mut loaded_plugins = self.loaded_plugins.write().await;
        
        let plugin = loaded_plugins.get_mut(plugin_id)
            .ok_or_else(|| anyhow::anyhow!("Plugin {} is not loaded", plugin_id))?;

        // Validate configuration
        self.validate_plugin_config(&plugin.manifest, &config).await?;

        plugin.config = config;
        info!("Updated configuration for plugin: {}", plugin_id);
        Ok(())
    }

    /// Get plugin configuration
    pub async fn get_plugin_config(&self, plugin_id: &PluginId) -> Option<PluginConfig> {
        let loaded_plugins = self.loaded_plugins.read().await;
        loaded_plugins.get(plugin_id).map(|plugin| plugin.config.clone())
    }

    /// Health check for a specific plugin
    pub async fn health_check(&self, plugin_id: &PluginId) -> Result<PluginHealth> {
        let loaded_plugins = self.loaded_plugins.read().await;
        
        let plugin = loaded_plugins.get(plugin_id)
            .ok_or_else(|| anyhow::anyhow!("Plugin {} is not loaded", plugin_id))?;

        // Get sandbox health
        if let Some(sandbox) = self.sandbox_manager.get_sandbox(plugin_id).await {
            let health = sandbox.health_check().await;
            let stats = sandbox.get_resource_stats().await;
            
            Ok(PluginHealth {
                status: health,
                last_heartbeat: Utc::now(),
                error_count: stats.error_count,
                memory_usage: stats.memory_usage,
                cpu_usage: stats.cpu_usage,
                response_time: std::time::Duration::from_millis(0), // TODO: Track actual response times
            })
        } else {
            Ok(plugin.health.clone())
        }
    }

    /// Get metrics for all plugins
    pub async fn get_metrics(&self) -> PluginMetrics {
        let loaded_plugins = self.loaded_plugins.read().await;
        let registry = self.registry.read().await;
        
        let total_plugins = registry.list_plugins().len();
        let active_plugins = loaded_plugins.values()
            .filter(|p| p.state == PluginState::Active)
            .count();
        let failed_plugins = loaded_plugins.values()
            .filter(|p| matches!(p.state, PluginState::Error(_)))
            .count();
        
        // Calculate resource usage
        let (total_memory, total_cpu) = loaded_plugins.values()
            .fold((0u64, 0.0f64), |(mem, cpu), plugin| {
                (mem + plugin.health.memory_usage, cpu + plugin.health.cpu_usage)
            });

        PluginMetrics {
            total_plugins,
            active_plugins,
            failed_plugins,
            total_memory_usage: total_memory,
            total_cpu_usage: total_cpu,
            total_executions: 0, // TODO: Track executions
            average_response_time: std::time::Duration::from_millis(0), // TODO: Track response times
        }
    }

    /// Validate plugin manifest and dependencies
    async fn validate_plugin(&self, manifest: &PluginManifest) -> Result<()> {
        // Validate manifest format
        {
            let registry = self.registry.read().await;
            registry.validate_manifest(manifest)?;
        }

        // Check dependencies
        if let Some(deps) = &manifest.dependencies {
            self.validate_dependencies(deps).await?;
        }

        // Validate permissions
        if let Some(capabilities) = &manifest.capabilities {
            self.validate_permissions(&capabilities.permissions).await?;
        }

        Ok(())
    }

    async fn validate_dependencies(&self, deps: &PluginDependencies) -> Result<()> {
        // Check runtime version compatibility
        let current_version = env!("CARGO_PKG_VERSION");
        if !self.is_version_compatible(&deps.runtime_version, current_version) {
            return Err(anyhow::anyhow!(
                "Runtime version mismatch: required {}, current {}",
                deps.runtime_version,
                current_version
            ));
        }

        // Check system requirements
        for requirement in &deps.system_requirements {
            if !self.check_system_requirement(requirement).await {
                return Err(anyhow::anyhow!(
                    "System requirement not met: {}",
                    requirement
                ));
            }
        }

        Ok(())
    }

    async fn validate_permissions(&self, permissions: &[Permission]) -> Result<()> {
        for permission in permissions {
            match permission {
                Permission::FilesystemRead(path) | Permission::FilesystemWrite(path) => {
                    if !path.is_absolute() {
                        return Err(anyhow::anyhow!(
                            "File permissions must use absolute paths: {}",
                            path.display()
                        ));
                    }
                }
                _ => {} // Other permissions are always valid for now
            }
        }
        Ok(())
    }

    async fn validate_plugin_config(&self, manifest: &PluginManifest, config: &PluginConfig) -> Result<()> {
        // Validate required configuration fields
        if let Some(config_schema) = &manifest.configuration {
            for required_field in &config_schema.required_fields {
                if !config.settings.as_object()
                    .and_then(|obj| obj.get(required_field))
                    .is_some() {
                    return Err(anyhow::anyhow!(
                        "Required configuration field missing: {}",
                        required_field
                    ));
                }
            }
        }

        Ok(())
    }

    fn is_version_compatible(&self, required: &str, current: &str) -> bool {
        // Simple version compatibility check
        // TODO: Implement proper semver compatibility
        required <= current
    }

    async fn check_system_requirement(&self, requirement: &str) -> bool {
        match requirement {
            "network" => true, // Always available
            "filesystem" => true, // Always available
            _ => {
                warn!("Unknown system requirement: {}", requirement);
                false
            }
        }
    }
}

/// Loaded plugin runtime information
#[derive(Debug, Clone)]
struct LoadedPlugin {
    id: PluginId,
    manifest: PluginManifest,
    config: PluginConfig,
    state: PluginState,
    loaded_at: chrono::DateTime<chrono::Utc>,
    last_error: Option<String>,
    health: PluginHealth,
}

/// Plugin manager configuration
#[derive(Debug, Clone)]
struct PluginManagerConfig {
    max_plugins: usize,
    default_timeout: std::time::Duration,
    security_enabled: bool,
    auto_discovery: bool,
}

impl Default for PluginManagerConfig {
    fn default() -> Self {
        Self {
            max_plugins: 100,
            default_timeout: std::time::Duration::from_secs(30),
            security_enabled: true,
            auto_discovery: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use tokio::fs;

    #[tokio::test]
    async fn test_plugin_manager_creation() {
        let manager = PluginManager::new().await.unwrap();
        let metrics = manager.get_metrics().await;
        assert_eq!(metrics.total_plugins, 0);
    }

    #[tokio::test]
    async fn test_plugin_discovery() {
        let manager = PluginManager::new().await.unwrap();
        let temp_dir = tempdir().unwrap();
        
        // Create a test plugin manifest
        let manifest_content = r#"
[plugin]
name = "test-plugin"
version = "1.0.0"
description = "Test plugin"
author = "Test Author"
license = "MIT"
type = "action"
"#;
        
        let plugin_dir = temp_dir.path().join("test-plugin");
        fs::create_dir(&plugin_dir).await.unwrap();
        fs::write(plugin_dir.join("plugin.toml"), manifest_content).await.unwrap();
        fs::write(plugin_dir.join("plugin.so"), b"fake binary").await.unwrap();
        
        let discovered = manager.discover_plugins(temp_dir.path()).await.unwrap();
        assert_eq!(discovered.len(), 1);
        
        let plugins = manager.list_plugins().await;
        assert_eq!(plugins.len(), 1);
        assert_eq!(plugins[0].manifest.plugin.name, "test-plugin");
    }
}