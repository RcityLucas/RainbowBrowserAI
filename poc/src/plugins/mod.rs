// Plugin system module
pub mod manager;
pub mod traits;
pub mod types;
pub mod sandbox;
pub mod events;
pub mod registry;

pub use manager::PluginManager;
pub use traits::*;
pub use types::*;
pub use events::{EventBus, PluginEvent};

use anyhow::Result;
use std::path::Path;

/// Initialize the plugin system
pub async fn init_plugin_system() -> Result<PluginManager> {
    let manager = PluginManager::new().await?;
    
    // Discover plugins in the plugins directory
    let plugins_dir = Path::new("plugins");
    if plugins_dir.exists() {
        manager.discover_plugins(plugins_dir).await?;
    }
    
    Ok(manager)
}