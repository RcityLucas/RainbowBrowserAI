use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::{info, warn};

use super::types::{PluginId, PluginManifest, PluginInfo, PluginState};

/// Plugin registry for discovery and management
#[derive(Debug)]
pub struct PluginRegistry {
    plugins: HashMap<PluginId, PluginRegistryEntry>,
    search_paths: Vec<PathBuf>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            search_paths: vec![
                PathBuf::from("plugins"),
                PathBuf::from("/usr/local/lib/rainbow-browser-ai/plugins"),
                PathBuf::from("~/.rainbow-browser-ai/plugins"),
            ],
        }
    }

    pub fn add_search_path(&mut self, path: PathBuf) {
        if !self.search_paths.contains(&path) {
            self.search_paths.push(path);
        }
    }

    /// Discover plugins in all search paths
    pub async fn discover_plugins(&mut self) -> Result<Vec<PluginId>> {
        let mut discovered = Vec::new();

        for search_path in &self.search_paths.clone() {
            if search_path.exists() {
                match self.discover_plugins_in_path(search_path).await {
                    Ok(mut plugins) => discovered.append(&mut plugins),
                    Err(e) => warn!("Failed to discover plugins in {}: {}", search_path.display(), e),
                }
            }
        }

        info!("Discovered {} plugins", discovered.len());
        Ok(discovered)
    }

    /// Discover plugins in a specific directory
    pub async fn discover_plugins_in_path(&mut self, path: &Path) -> Result<Vec<PluginId>> {
        let mut discovered = Vec::new();
        
        if !path.exists() || !path.is_dir() {
            return Ok(discovered);
        }

        let mut entries = fs::read_dir(path).await?;
        
        while let Some(entry) = entries.next_entry().await? {
            let entry_path = entry.path();
            
            // Look for plugin directories or direct plugin files
            if entry_path.is_dir() {
                if let Ok(plugin_id) = self.load_plugin_from_directory(&entry_path).await {
                    discovered.push(plugin_id);
                }
            } else if self.is_plugin_file(&entry_path) {
                if let Ok(plugin_id) = self.load_plugin_from_file(&entry_path).await {
                    discovered.push(plugin_id);
                }
            }
        }

        Ok(discovered)
    }

    /// Load plugin manifest from directory
    async fn load_plugin_from_directory(&mut self, dir_path: &Path) -> Result<PluginId> {
        let manifest_path = dir_path.join("plugin.toml");
        
        if !manifest_path.exists() {
            return Err(anyhow::anyhow!("No plugin.toml found in {}", dir_path.display()));
        }

        let manifest_content = fs::read_to_string(&manifest_path).await?;
        let manifest: PluginManifest = toml::from_str(&manifest_content)?;
        
        // Look for the actual plugin binary
        let plugin_binary = self.find_plugin_binary(dir_path).await?;
        
        let plugin_id = PluginId::from_name(&manifest.plugin.name);
        let entry = PluginRegistryEntry {
            id: plugin_id.clone(),
            manifest,
            plugin_path: plugin_binary,
            manifest_path,
            state: PluginState::Discovered,
            discovered_at: chrono::Utc::now(),
        };

        self.plugins.insert(plugin_id.clone(), entry);
        info!("Discovered plugin: {} at {}", plugin_id, dir_path.display());
        
        Ok(plugin_id)
    }

    /// Load plugin from a single file
    async fn load_plugin_from_file(&mut self, file_path: &Path) -> Result<PluginId> {
        // For single files, we need to look for an adjacent manifest
        let dir_path = file_path.parent().unwrap_or(Path::new("."));
        let plugin_name = file_path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");
            
        let manifest_path = dir_path.join(format!("{}.toml", plugin_name));
        
        let manifest = if manifest_path.exists() {
            let manifest_content = fs::read_to_string(&manifest_path).await?;
            toml::from_str(&manifest_content)?
        } else {
            // Create a minimal default manifest
            self.create_default_manifest(plugin_name)
        };

        let plugin_id = PluginId::from_name(&manifest.plugin.name);
        let entry = PluginRegistryEntry {
            id: plugin_id.clone(),
            manifest,
            plugin_path: file_path.to_path_buf(),
            manifest_path,
            state: PluginState::Discovered,
            discovered_at: chrono::Utc::now(),
        };

        self.plugins.insert(plugin_id.clone(), entry);
        info!("Discovered plugin: {} at {}", plugin_id, file_path.display());
        
        Ok(plugin_id)
    }

    /// Find plugin binary in directory
    async fn find_plugin_binary(&self, dir_path: &Path) -> Result<PathBuf> {
        let mut entries = fs::read_dir(dir_path).await?;
        
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if self.is_plugin_file(&path) {
                return Ok(path);
            }
        }
        
        Err(anyhow::anyhow!("No plugin binary found in {}", dir_path.display()))
    }

    /// Check if file is a plugin binary
    fn is_plugin_file(&self, path: &Path) -> bool {
        if let Some(extension) = path.extension() {
            let ext = extension.to_string_lossy().to_lowercase();
            matches!(ext.as_str(), "so" | "dylib" | "dll")
        } else {
            false
        }
    }

    /// Create default manifest for plugins without manifests
    fn create_default_manifest(&self, plugin_name: &str) -> PluginManifest {
        PluginManifest {
            plugin: super::types::PluginMetadata {
                name: plugin_name.to_string(),
                version: "1.0.0".to_string(),
                description: "Plugin loaded without manifest".to_string(),
                author: "Unknown".to_string(),
                license: "Unknown".to_string(),
                plugin_type: super::types::PluginType::Action,
            },
            dependencies: None,
            capabilities: None,
            configuration: None,
            resources: Some(super::types::ResourceLimits::default()),
        }
    }

    /// Get plugin registry entry
    pub fn get_plugin(&self, plugin_id: &PluginId) -> Option<&PluginRegistryEntry> {
        self.plugins.get(plugin_id)
    }

    /// Get all plugins
    pub fn list_plugins(&self) -> Vec<&PluginRegistryEntry> {
        self.plugins.values().collect()
    }

    /// Update plugin state
    pub fn update_plugin_state(&mut self, plugin_id: &PluginId, state: PluginState) {
        if let Some(entry) = self.plugins.get_mut(plugin_id) {
            entry.state = state;
        }
    }

    /// Remove plugin from registry
    pub fn remove_plugin(&mut self, plugin_id: &PluginId) -> Option<PluginRegistryEntry> {
        self.plugins.remove(plugin_id)
    }

    /// Search plugins by name or description
    pub fn search_plugins(&self, query: &str) -> Vec<&PluginRegistryEntry> {
        let query_lower = query.to_lowercase();
        
        self.plugins.values()
            .filter(|entry| {
                entry.manifest.plugin.name.to_lowercase().contains(&query_lower) ||
                entry.manifest.plugin.description.to_lowercase().contains(&query_lower)
            })
            .collect()
    }

    /// Get plugins by type
    pub fn get_plugins_by_type(&self, plugin_type: &super::types::PluginType) -> Vec<&PluginRegistryEntry> {
        self.plugins.values()
            .filter(|entry| entry.manifest.plugin.plugin_type == *plugin_type)
            .collect()
    }

    /// Validate plugin manifest
    pub fn validate_manifest(&self, manifest: &PluginManifest) -> Result<()> {
        // Basic validation
        if manifest.plugin.name.is_empty() {
            return Err(anyhow::anyhow!("Plugin name cannot be empty"));
        }

        if manifest.plugin.version.is_empty() {
            return Err(anyhow::anyhow!("Plugin version cannot be empty"));
        }

        // Validate version format (basic semver check)
        if !self.is_valid_version(&manifest.plugin.version) {
            return Err(anyhow::anyhow!("Invalid version format: {}", manifest.plugin.version));
        }

        // Validate permissions if present
        if let Some(capabilities) = &manifest.capabilities {
            for permission in &capabilities.permissions {
                self.validate_permission(permission)?;
            }
        }

        Ok(())
    }

    fn is_valid_version(&self, version: &str) -> bool {
        // Basic semver pattern: X.Y.Z
        let parts: Vec<&str> = version.split('.').collect();
        parts.len() == 3 && parts.iter().all(|part| part.parse::<u32>().is_ok())
    }

    fn validate_permission(&self, permission: &super::types::Permission) -> Result<()> {
        match permission {
            super::types::Permission::FilesystemRead(path) |
            super::types::Permission::FilesystemWrite(path) => {
                if !path.is_absolute() {
                    return Err(anyhow::anyhow!("File permissions must use absolute paths"));
                }
            }
            _ => {} // Other permissions are always valid
        }
        Ok(())
    }

    /// Export registry to JSON for backup/transfer
    pub async fn export_registry(&self, path: &Path) -> Result<()> {
        let export_data: Vec<PluginExport> = self.plugins.values()
            .map(|entry| PluginExport {
                id: entry.id.clone(),
                manifest: entry.manifest.clone(),
                plugin_path: entry.plugin_path.clone(),
                state: entry.state.clone(),
                discovered_at: entry.discovered_at,
            })
            .collect();

        let json_data = serde_json::to_string_pretty(&export_data)?;
        fs::write(path, json_data).await?;
        
        info!("Exported {} plugins to {}", export_data.len(), path.display());
        Ok(())
    }

    /// Import registry from JSON
    pub async fn import_registry(&mut self, path: &Path) -> Result<usize> {
        let json_data = fs::read_to_string(path).await?;
        let export_data: Vec<PluginExport> = serde_json::from_str(&json_data)?;
        
        let mut imported = 0;
        for export in export_data {
            // Verify plugin file still exists
            if export.plugin_path.exists() {
                let entry = PluginRegistryEntry {
                    id: export.id.clone(),
                    manifest: export.manifest,
                    plugin_path: export.plugin_path,
                    manifest_path: PathBuf::new(), // Will be updated on next discovery
                    state: PluginState::Discovered, // Reset to discovered state
                    discovered_at: export.discovered_at,
                };
                
                self.plugins.insert(export.id, entry);
                imported += 1;
            } else {
                warn!("Skipping missing plugin: {}", export.plugin_path.display());
            }
        }
        
        info!("Imported {} plugins from {}", imported, path.display());
        Ok(imported)
    }
}

/// Plugin registry entry
#[derive(Debug, Clone)]
pub struct PluginRegistryEntry {
    pub id: PluginId,
    pub manifest: PluginManifest,
    pub plugin_path: PathBuf,
    pub manifest_path: PathBuf,
    pub state: PluginState,
    pub discovered_at: chrono::DateTime<chrono::Utc>,
}

impl PluginRegistryEntry {
    /// Convert to PluginInfo for API responses
    pub fn to_plugin_info(&self) -> PluginInfo {
        PluginInfo {
            id: self.id.clone(),
            manifest: self.manifest.clone(),
            state: self.state.clone(),
            config: None, // Configuration is managed separately
            loaded_at: None, // Will be set when plugin is loaded
            last_error: None, // Will be set by plugin manager
            path: self.plugin_path.clone(),
        }
    }
}

/// Export format for plugin registry
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PluginExport {
    id: PluginId,
    manifest: PluginManifest,
    plugin_path: PathBuf,
    state: PluginState,
    discovered_at: chrono::DateTime<chrono::Utc>,
}