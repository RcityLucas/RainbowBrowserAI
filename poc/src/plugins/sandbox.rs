use anyhow::Result;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use sysinfo::{System, SystemExt, ProcessExt, Pid};

use super::types::{Permission, ResourceLimits, PluginId, HealthStatus};

/// Plugin sandbox for security and resource management
#[derive(Debug)]
pub struct PluginSandbox {
    plugin_id: PluginId,
    permissions: Vec<Permission>,
    resource_limits: ResourceLimits,
    resource_monitor: Arc<RwLock<ResourceMonitor>>,
    rate_limiter: Arc<RwLock<RateLimiter>>,
}

impl PluginSandbox {
    pub fn new(
        plugin_id: PluginId,
        permissions: Vec<Permission>,
        resource_limits: ResourceLimits,
    ) -> Self {
        Self {
            plugin_id: plugin_id.clone(),
            permissions,
            resource_limits: resource_limits.clone(),
            resource_monitor: Arc::new(RwLock::new(ResourceMonitor::new(plugin_id.clone()))),
            rate_limiter: Arc::new(RwLock::new(RateLimiter::new())),
        }
    }

    /// Check if plugin has specific permission
    pub fn has_permission(&self, permission: &Permission) -> bool {
        self.permissions.iter().any(|p| self.permissions_match(p, permission))
    }

    /// Validate file access
    pub fn validate_file_access(&self, path: &Path, write: bool) -> Result<()> {
        let permission_type = if write {
            self.permissions.iter().find(|p| {
                if let Permission::FilesystemWrite(allowed_path) = p {
                    path.starts_with(allowed_path)
                } else {
                    false
                }
            })
        } else {
            self.permissions.iter().find(|p| {
                match p {
                    Permission::FilesystemRead(allowed_path) |
                    Permission::FilesystemWrite(allowed_path) => {
                        path.starts_with(allowed_path)
                    }
                    _ => false,
                }
            })
        };

        match permission_type {
            Some(_) => Ok(()),
            None => {
                tracing::warn!(
                    "Plugin {} denied file access to {}: {} permission required",
                    self.plugin_id,
                    path.display(),
                    if write { "write" } else { "read" }
                );
                Err(anyhow::anyhow!(
                    "File access denied: insufficient permissions for {}",
                    path.display()
                ))
            }
        }
    }

    /// Check resource usage limits
    pub async fn validate_resource_usage(&self, memory: u64, cpu: f64) -> Result<()> {
        if memory > self.resource_limits.max_memory_mb * 1024 * 1024 {
            return Err(anyhow::anyhow!(
                "Memory usage {} MB exceeds limit {} MB",
                memory / 1024 / 1024,
                self.resource_limits.max_memory_mb
            ));
        }

        if cpu > self.resource_limits.max_cpu_percent {
            return Err(anyhow::anyhow!(
                "CPU usage {:.1}% exceeds limit {:.1}%",
                cpu,
                self.resource_limits.max_cpu_percent
            ));
        }

        let mut monitor = self.resource_monitor.write().await;
        monitor.record_usage(memory, cpu);

        Ok(())
    }

    /// Check rate limits
    pub async fn check_rate_limit(&self) -> Result<()> {
        let mut limiter = self.rate_limiter.write().await;
        limiter.check_rate_limit()
    }

    /// Get resource usage statistics
    pub async fn get_resource_stats(&self) -> ResourceStats {
        let monitor = self.resource_monitor.read().await;
        monitor.get_stats()
    }

    /// Get sandbox health status
    pub async fn health_check(&self) -> HealthStatus {
        let stats = self.get_resource_stats().await;
        
        if stats.memory_usage > self.resource_limits.max_memory_mb * 1024 * 1024 {
            return HealthStatus::Critical(
                format!("Memory usage {} MB exceeds limit", stats.memory_usage / 1024 / 1024)
            );
        }

        if stats.cpu_usage > self.resource_limits.max_cpu_percent {
            return HealthStatus::Warning(
                format!("CPU usage {:.1}% is high", stats.cpu_usage)
            );
        }

        if stats.error_count > 10 {
            return HealthStatus::Warning(
                format!("High error count: {}", stats.error_count)
            );
        }

        HealthStatus::Healthy
    }

    fn permissions_match(&self, granted: &Permission, requested: &Permission) -> bool {
        match (granted, requested) {
            (Permission::Network, Permission::Network) => true,
            (Permission::BrowserControl, Permission::BrowserControl) => true,
            (Permission::WorkflowModification, Permission::WorkflowModification) => true,
            (Permission::MetricsAccess, Permission::MetricsAccess) => true,
            (Permission::ConfigurationRead, Permission::ConfigurationRead) => true,
            (Permission::FilesystemRead(granted_path), Permission::FilesystemRead(requested_path)) => {
                requested_path.starts_with(granted_path)
            }
            (Permission::FilesystemWrite(granted_path), Permission::FilesystemWrite(requested_path)) => {
                requested_path.starts_with(granted_path)
            }
            (Permission::FilesystemWrite(granted_path), Permission::FilesystemRead(requested_path)) => {
                // Write permission includes read
                requested_path.starts_with(granted_path)
            }
            _ => false,
        }
    }
}

/// Resource monitoring for plugins
#[derive(Debug)]
pub struct ResourceMonitor {
    plugin_id: PluginId,
    start_time: Instant,
    memory_samples: Vec<u64>,
    cpu_samples: Vec<f64>,
    error_count: u64,
    last_update: Instant,
}

impl ResourceMonitor {
    pub fn new(plugin_id: PluginId) -> Self {
        Self {
            plugin_id,
            start_time: Instant::now(),
            memory_samples: Vec::new(),
            cpu_samples: Vec::new(),
            error_count: 0,
            last_update: Instant::now(),
        }
    }

    pub fn record_usage(&mut self, memory: u64, cpu: f64) {
        self.memory_samples.push(memory);
        self.cpu_samples.push(cpu);
        self.last_update = Instant::now();

        // Keep only last 100 samples
        if self.memory_samples.len() > 100 {
            self.memory_samples.remove(0);
        }
        if self.cpu_samples.len() > 100 {
            self.cpu_samples.remove(0);
        }
    }

    pub fn record_error(&mut self) {
        self.error_count += 1;
    }

    pub fn get_stats(&self) -> ResourceStats {
        let avg_memory = if self.memory_samples.is_empty() {
            0
        } else {
            self.memory_samples.iter().sum::<u64>() / self.memory_samples.len() as u64
        };

        let avg_cpu = if self.cpu_samples.is_empty() {
            0.0
        } else {
            self.cpu_samples.iter().sum::<f64>() / self.cpu_samples.len() as f64
        };

        let peak_memory = self.memory_samples.iter().max().copied().unwrap_or(0);
        let peak_cpu = self.cpu_samples.iter().copied().fold(0.0f64, f64::max);

        ResourceStats {
            plugin_id: self.plugin_id.clone(),
            uptime: self.start_time.elapsed(),
            memory_usage: avg_memory,
            peak_memory_usage: peak_memory,
            cpu_usage: avg_cpu,
            peak_cpu_usage: peak_cpu,
            error_count: self.error_count,
            last_update: self.last_update.elapsed(),
        }
    }
}

/// Rate limiter for plugin operations
#[derive(Debug)]
pub struct RateLimiter {
    requests: Vec<Instant>,
    max_requests_per_minute: u32,
    max_requests_per_hour: u32,
}

impl RateLimiter {
    pub fn new() -> Self {
        Self {
            requests: Vec::new(),
            max_requests_per_minute: 60,
            max_requests_per_hour: 1000,
        }
    }

    pub fn with_limits(max_per_minute: u32, max_per_hour: u32) -> Self {
        Self {
            requests: Vec::new(),
            max_requests_per_minute: max_per_minute,
            max_requests_per_hour: max_per_hour,
        }
    }

    pub fn check_rate_limit(&mut self) -> Result<()> {
        let now = Instant::now();
        
        // Clean old requests
        self.requests.retain(|&time| now.duration_since(time) < Duration::from_secs(3600));
        
        // Check hourly limit
        if self.requests.len() >= self.max_requests_per_hour as usize {
            return Err(anyhow::anyhow!("Hourly rate limit exceeded"));
        }
        
        // Check minute limit
        let minute_requests = self.requests.iter()
            .filter(|&&time| now.duration_since(time) < Duration::from_secs(60))
            .count();
            
        if minute_requests >= self.max_requests_per_minute as usize {
            return Err(anyhow::anyhow!("Per-minute rate limit exceeded"));
        }
        
        self.requests.push(now);
        Ok(())
    }
}

/// Resource usage statistics
#[derive(Debug, Clone)]
pub struct ResourceStats {
    pub plugin_id: PluginId,
    pub uptime: Duration,
    pub memory_usage: u64,
    pub peak_memory_usage: u64,
    pub cpu_usage: f64,
    pub peak_cpu_usage: f64,
    pub error_count: u64,
    pub last_update: Duration,
}

/// System-wide plugin sandbox manager
#[derive(Debug)]
pub struct SandboxManager {
    sandboxes: Arc<RwLock<HashMap<PluginId, PluginSandbox>>>,
    system: Arc<RwLock<System>>,
}

impl SandboxManager {
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        
        Self {
            sandboxes: Arc::new(RwLock::new(HashMap::new())),
            system: Arc::new(RwLock::new(system)),
        }
    }

    pub async fn create_sandbox(
        &self,
        plugin_id: PluginId,
        permissions: Vec<Permission>,
        resource_limits: ResourceLimits,
    ) -> Result<()> {
        let sandbox = PluginSandbox::new(plugin_id.clone(), permissions, resource_limits);
        
        let mut sandboxes = self.sandboxes.write().await;
        sandboxes.insert(plugin_id, sandbox);
        
        Ok(())
    }

    pub async fn remove_sandbox(&self, plugin_id: &PluginId) -> Result<()> {
        let mut sandboxes = self.sandboxes.write().await;
        sandboxes.remove(plugin_id);
        Ok(())
    }

    pub async fn get_sandbox(&self, plugin_id: &PluginId) -> Option<PluginSandbox> {
        let sandboxes = self.sandboxes.read().await;
        sandboxes.get(plugin_id).cloned()
    }

    pub async fn validate_file_access(
        &self,
        plugin_id: &PluginId,
        path: &Path,
        write: bool,
    ) -> Result<()> {
        let sandboxes = self.sandboxes.read().await;
        match sandboxes.get(plugin_id) {
            Some(sandbox) => sandbox.validate_file_access(path, write),
            None => Err(anyhow::anyhow!("Plugin sandbox not found: {}", plugin_id)),
        }
    }

    pub async fn check_rate_limit(&self, plugin_id: &PluginId) -> Result<()> {
        let sandboxes = self.sandboxes.read().await;
        match sandboxes.get(plugin_id) {
            Some(sandbox) => sandbox.check_rate_limit().await,
            None => Err(anyhow::anyhow!("Plugin sandbox not found: {}", plugin_id)),
        }
    }

    pub async fn monitor_all_plugins(&self) -> Result<Vec<ResourceStats>> {
        let mut system = self.system.write().await;
        system.refresh_all();
        
        let sandboxes = self.sandboxes.read().await;
        let mut stats = Vec::new();
        
        for sandbox in sandboxes.values() {
            let plugin_stats = sandbox.get_resource_stats().await;
            stats.push(plugin_stats);
        }
        
        Ok(stats)
    }

    pub async fn health_check_all(&self) -> HashMap<PluginId, HealthStatus> {
        let sandboxes = self.sandboxes.read().await;
        let mut health_map = HashMap::new();
        
        for (plugin_id, sandbox) in sandboxes.iter() {
            let health = sandbox.health_check().await;
            health_map.insert(plugin_id.clone(), health);
        }
        
        health_map
    }
}