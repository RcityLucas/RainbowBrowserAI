// Test for the plugin system
use rainbow_poc::{PluginManager, init_plugin_system};
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔌 Testing Plugin System");

    // Initialize plugin system
    let manager = init_plugin_system().await?;
    println!("✅ Plugin manager initialized");

    // Discover example plugins
    let plugins_dir = Path::new("plugins/examples");
    if plugins_dir.exists() {
        let discovered = manager.discover_plugins(plugins_dir).await?;
        println!("🔍 Discovered {} plugins", discovered.len());
        
        for plugin_id in &discovered {
            println!("  - {}", plugin_id);
        }
    } else {
        println!("⚠️ Examples directory not found");
    }

    // List all plugins
    let all_plugins = manager.list_plugins().await;
    println!("\n📋 All plugins in registry:");
    for plugin in &all_plugins {
        println!("  - {} v{} ({})", 
            plugin.manifest.plugin.name,
            plugin.manifest.plugin.version,
            plugin.state.to_string()
        );
        println!("    Description: {}", plugin.manifest.plugin.description);
        println!("    Type: {:?}", plugin.manifest.plugin.plugin_type);
        
        if let Some(capabilities) = &plugin.manifest.capabilities {
            if let Some(actions) = &capabilities.actions {
                println!("    Actions: {:?}", actions);
            }
        }
        println!();
    }

    // Get metrics
    let metrics = manager.get_metrics().await;
    println!("📊 Plugin metrics:");
    println!("  - Total plugins: {}", metrics.total_plugins);
    println!("  - Active plugins: {}", metrics.active_plugins);
    println!("  - Failed plugins: {}", metrics.failed_plugins);

    println!("\n✅ Plugin system test completed successfully!");
    Ok(())
}