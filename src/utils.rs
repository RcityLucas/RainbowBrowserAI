//! # 彩虹城浏览器工具模块
//!
//! 提供 AI 生命体各个器官系统共用的基础工具和实用函数。
//! 这些工具就像生命体内的酶和蛋白质，支撑着各种生命活动的进行。

use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::RwLock;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// 时间工具 - 生命体的时间感知能力
pub mod time {
    use super::*;
    
    /// 获取当前UTC时间
    pub fn now_utc() -> DateTime<Utc> {
        Utc::now()
    }
    
    /// 获取当前时间戳（毫秒）
    pub fn now_timestamp_ms() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    }
    
    /// 格式化持续时间为人类可读格式
    pub fn format_duration(duration: Duration) -> String {
        let total_seconds = duration.as_secs();
        let hours = total_seconds / 3600;
        let minutes = (total_seconds % 3600) / 60;
        let seconds = total_seconds % 60;
        let millis = duration.subsec_millis();
        
        if hours > 0 {
            format!("{}h{}m{}s", hours, minutes, seconds)
        } else if minutes > 0 {
            format!("{}m{}s", minutes, seconds)
        } else if seconds > 0 {
            format!("{}.{}s", seconds, millis)
        } else {
            format!("{}ms", millis)
        }
    }
    
    /// 计算两个时间点之间的间隔
    pub fn duration_since(start: DateTime<Utc>) -> Duration {
        let now = Utc::now();
        if now > start {
            (now - start).to_std().unwrap_or_default()
        } else {
            Duration::default()
        }
    }
    
    /// 检查是否超时
    pub fn is_timeout(start: Instant, timeout: Duration) -> bool {
        start.elapsed() > timeout
    }
    
    /// 睡眠指定时间，带有取消支持
    pub async fn sleep_with_cancel(
        duration: Duration,
        cancel_token: tokio_util::sync::CancellationToken,
    ) -> Result<(), tokio::time::error::Elapsed> {
        tokio::select! {
            _ = tokio::time::sleep(duration) => Ok(()),
            _ = cancel_token.cancelled() => Err(tokio::time::error::Elapsed::new()),
        }
    }
}

/// 字符串工具 - 文本处理和格式化
pub mod string {
    use super::*;
    
    /// 截断字符串到指定长度
    pub fn truncate(s: &str, max_len: usize) -> String {
        if s.len() <= max_len {
            s.to_string()
        } else {
            format!("{}...", &s[..max_len.saturating_sub(3)])
        }
    }
    
    /// 清理HTML标签
    pub fn strip_html(html: &str) -> String {
        // 简单的HTML标签清理
        let re = regex::Regex::new(r"<[^>]*>").unwrap();
        re.replace_all(html, "").to_string()
    }
    
    /// 标准化空白字符
    pub fn normalize_whitespace(s: &str) -> String {
        let re = regex::Regex::new(r"\s+").unwrap();
        re.replace_all(s.trim(), " ").to_string()
    }
    
    /// 生成安全的文件名
    pub fn safe_filename(name: &str) -> String {
        let invalid_chars = ['<', '>', ':', '"', '/', '\\', '|', '?', '*'];
        name.chars()
            .map(|c| if invalid_chars.contains(&c) { '_' } else { c })
            .collect()
    }
    
    /// 计算字符串相似度（简化版Levenshtein距离）
    pub fn similarity(s1: &str, s2: &str) -> f64 {
        if s1.is_empty() && s2.is_empty() {
            return 1.0;
        }
        if s1.is_empty() || s2.is_empty() {
            return 0.0;
        }
        
        let len1 = s1.chars().count();
        let len2 = s2.chars().count();
        let max_len = len1.max(len2);
        
        let distance = levenshtein_distance(s1, s2);
        1.0 - (distance as f64 / max_len as f64)
    }
    
    fn levenshtein_distance(s1: &str, s2: &str) -> usize {
        let chars1: Vec<char> = s1.chars().collect();
        let chars2: Vec<char> = s2.chars().collect();
        let len1 = chars1.len();
        let len2 = chars2.len();
        
        let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];
        
        for i in 0..=len1 {
            matrix[i][0] = i;
        }
        for j in 0..=len2 {
            matrix[0][j] = j;
        }
        
        for i in 1..=len1 {
            for j in 1..=len2 {
                let cost = if chars1[i - 1] == chars2[j - 1] { 0 } else { 1 };
                matrix[i][j] = (matrix[i - 1][j] + 1)
                    .min(matrix[i][j - 1] + 1)
                    .min(matrix[i - 1][j - 1] + cost);
            }
        }
        
        matrix[len1][len2]
    }
}

/// URL工具 - 网络地址处理
pub mod url {
    use super::*;
    
    /// 验证URL是否有效
    pub fn is_valid_url(url: &str) -> bool {
        url::Url::parse(url).is_ok()
    }
    
    /// 标准化URL
    pub fn normalize_url(url: &str) -> Result<String, url::ParseError> {
        let mut parsed = url::Url::parse(url)?;
        
        // 移除片段
        parsed.set_fragment(None);
        
        // 标准化路径
        if parsed.path() == "" {
            parsed.set_path("/");
        }
        
        Ok(parsed.to_string())
    }
    
    /// 提取域名
    pub fn extract_domain(url: &str) -> Option<String> {
        url::Url::parse(url)
            .ok()?
            .host_str()
            .map(|s| s.to_string())
    }
    
    /// 判断是否为HTTPS URL
    pub fn is_https(url: &str) -> bool {
        url::Url::parse(url)
            .map(|u| u.scheme() == "https")
            .unwrap_or(false)
    }
    
    /// 合并相对URL
    pub fn join_url(base: &str, relative: &str) -> Result<String, url::ParseError> {
        let base_url = url::Url::parse(base)?;
        let joined = base_url.join(relative)?;
        Ok(joined.to_string())
    }
}

/// 哈希工具 - 数据指纹和校验
pub mod hash {
    use super::*;
    use sha2::{Sha256, Digest};
    
    /// 计算字符串的SHA256哈希
    pub fn sha256_string(input: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(input.as_bytes());
        format!("{:x}", hasher.finalize())
    }
    
    /// 计算字节数组的SHA256哈希
    pub fn sha256_bytes(input: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(input);
        format!("{:x}", hasher.finalize())
    }
    
    /// 生成内容摘要（前8个字符）
    pub fn content_digest(content: &str) -> String {
        let hash = sha256_string(content);
        hash[..8].to_string()
    }
    
    /// 快速哈希（用于HashMap键）
    pub fn fast_hash(input: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        input.hash(&mut hasher);
        hasher.finish()
    }
}

/// 内存工具 - 内存使用监控和优化
pub mod memory {
    use super::*;
    
    /// 获取当前进程内存使用量（字节）
    pub fn current_memory_usage() -> u64 {
        #[cfg(unix)]
        {
            use std::fs;
            if let Ok(content) = fs::read_to_string("/proc/self/status") {
                for line in content.lines() {
                    if line.starts_with("VmRSS:") {
                        if let Some(kb) = line
                            .split_whitespace()
                            .nth(1)
                            .and_then(|s| s.parse::<u64>().ok())
                        {
                            return kb * 1024; // 转换为字节
                        }
                    }
                }
            }
        }
        
        #[cfg(windows)]
        {
            // Windows平台的实现需要使用winapi
            // 这里返回一个估算值
            return 0;
        }
        
        #[cfg(not(any(unix, windows)))]
        {
            // 其他平台返回0
            return 0;
        }
        
        0
    }
    
    /// 格式化内存大小
    pub fn format_bytes(bytes: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
        let mut size = bytes as f64;
        let mut unit_idx = 0;
        
        while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
            size /= 1024.0;
            unit_idx += 1;
        }
        
        if unit_idx == 0 {
            format!("{} {}", bytes, UNITS[unit_idx])
        } else {
            format!("{:.2} {}", size, UNITS[unit_idx])
        }
    }
    
    /// 检查内存压力
    pub fn check_memory_pressure(threshold_percent: f64) -> bool {
        // 简化实现，实际应该检查系统内存使用率
        let current = current_memory_usage();
        let system_total = 8u64 * 1024 * 1024 * 1024; // 假设8GB系统内存
        (current as f64 / system_total as f64) > threshold_percent
    }
}

/// 并发工具 - 线程安全和异步操作
pub mod concurrent {
    use super::*;
    use tokio::sync::{Semaphore, Mutex};
    use std::sync::Arc;
    
    /// 并发限制器
    pub struct ConcurrencyLimiter {
        semaphore: Arc<Semaphore>,
    }
    
    impl ConcurrencyLimiter {
        /// 创建新的并发限制器
        pub fn new(max_concurrent: usize) -> Self {
            Self {
                semaphore: Arc::new(Semaphore::new(max_concurrent)),
            }
        }
        
        /// 获取许可证（会阻塞直到有可用许可）
        pub async fn acquire(&self) -> tokio::sync::SemaphorePermit {
            self.semaphore.acquire().await.unwrap()
        }
        
        /// 尝试获取许可证（非阻塞）
        pub fn try_acquire(&self) -> Option<tokio::sync::SemaphorePermit> {
            self.semaphore.try_acquire().ok()
        }
        
        /// 获取可用许可数
        pub fn available_permits(&self) -> usize {
            self.semaphore.available_permits()
        }
    }
    
    /// 带超时的异步操作
    pub async fn with_timeout<T>(
        future: impl std::future::Future<Output = T>,
        timeout: Duration,
    ) -> Result<T, tokio::time::error::Elapsed> {
        tokio::time::timeout(timeout, future).await
    }
    
    /// 重试装饰器
    pub async fn retry_with_backoff<T, E, F, Fut>(
        mut operation: F,
        max_attempts: u32,
        initial_delay: Duration,
    ) -> Result<T, E>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T, E>>,
    {
        let mut attempts = 0;
        let mut delay = initial_delay;
        
        loop {
            attempts += 1;
            match operation().await {
                Ok(result) => return Ok(result),
                Err(err) => {
                    if attempts >= max_attempts {
                        return Err(err);
                    }
                    tokio::time::sleep(delay).await;
                    delay = delay * 2; // 指数退避
                }
            }
        }
    }
}

/// 配置工具 - 配置文件处理
pub mod config_utils {
    use super::*;
    use std::path::Path;
    
    /// 查找配置文件
    pub fn find_config_file(name: &str) -> Option<String> {
        let possible_paths = vec![
            format!("./{}", name),
            format!("./config/{}", name),
            format!("/etc/rainbow-browser/{}", name),
            format!("{}/rainbow-browser/{}", 
                std::env::var("HOME").unwrap_or_default(), name),
        ];
        
        for path in possible_paths {
            if Path::new(&path).exists() {
                return Some(path);
            }
        }
        
        None
    }
    
    /// 合并环境变量到配置
    pub fn merge_env_vars(prefix: &str) -> HashMap<String, String> {
        let mut env_config = HashMap::new();
        
        for (key, value) in std::env::vars() {
            if key.starts_with(prefix) {
                let config_key = key.strip_prefix(prefix)
                    .unwrap_or(&key)
                    .to_lowercase();
                env_config.insert(config_key, value);
            }
        }
        
        env_config
    }
}

/// 验证工具 - 数据验证和清理
pub mod validation {
    use super::*;
    
    /// 验证CSS选择器
    pub fn is_valid_css_selector(selector: &str) -> bool {
        // 简单的CSS选择器验证
        !selector.is_empty() && 
        !selector.contains("..") && 
        !selector.contains("//")
    }
    
    /// 验证XPath表达式
    pub fn is_valid_xpath(xpath: &str) -> bool {
        // 简单的XPath验证
        !xpath.is_empty() && xpath.starts_with('/') || xpath.starts_with("//")
    }
    
    /// 清理用户输入
    pub fn sanitize_input(input: &str) -> String {
        input
            .chars()
            .filter(|c| !c.is_control() || c == &'\n' || c == &'\t')
            .collect()
    }
    
    /// 验证端口号
    pub fn is_valid_port(port: u16) -> bool {
        port > 0 && port <= 65535
    }
    
    /// 验证文件扩展名
    pub fn is_valid_extension(filename: &str, allowed_exts: &[&str]) -> bool {
        if let Some(ext) = std::path::Path::new(filename)
            .extension()
            .and_then(|s| s.to_str())
        {
            allowed_exts.contains(&ext.to_lowercase().as_str())
        } else {
            false
        }
    }
}

/// 性能监控工具
pub mod performance {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    
    /// 简单的性能计数器
    pub struct PerformanceCounter {
        success_count: AtomicU64,
        failure_count: AtomicU64,
        total_duration: AtomicU64,
        start_time: Instant,
    }
    
    impl PerformanceCounter {
        /// 创建新的性能计数器
        pub fn new() -> Self {
            Self {
                success_count: AtomicU64::new(0),
                failure_count: AtomicU64::new(0),
                total_duration: AtomicU64::new(0),
                start_time: Instant::now(),
            }
        }
        
        /// 记录成功操作
        pub fn record_success(&self, duration: Duration) {
            self.success_count.fetch_add(1, Ordering::Relaxed);
            self.total_duration.fetch_add(duration.as_millis() as u64, Ordering::Relaxed);
        }
        
        /// 记录失败操作
        pub fn record_failure(&self) {
            self.failure_count.fetch_add(1, Ordering::Relaxed);
        }
        
        /// 获取成功率
        pub fn success_rate(&self) -> f64 {
            let success = self.success_count.load(Ordering::Relaxed);
            let failure = self.failure_count.load(Ordering::Relaxed);
            let total = success + failure;
            
            if total == 0 {
                0.0
            } else {
                success as f64 / total as f64
            }
        }
        
        /// 获取平均响应时间
        pub fn average_duration(&self) -> Duration {
            let success = self.success_count.load(Ordering::Relaxed);
            let total_ms = self.total_duration.load(Ordering::Relaxed);
            
            if success == 0 {
                Duration::default()
            } else {
                Duration::from_millis(total_ms / success)
            }
        }
        
        /// 获取吞吐量（每秒操作数）
        pub fn throughput(&self) -> f64 {
            let total_ops = self.success_count.load(Ordering::Relaxed) + 
                          self.failure_count.load(Ordering::Relaxed);
            let elapsed_secs = self.start_time.elapsed().as_secs_f64();
            
            if elapsed_secs == 0.0 {
                0.0
            } else {
                total_ops as f64 / elapsed_secs
            }
        }
    }
    
    impl Default for PerformanceCounter {
        fn default() -> Self {
            Self::new()
        }
    }
}

/// 缓存工具 - 简单的内存缓存实现
pub mod cache {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::RwLock;
    use std::collections::HashMap;
    use std::time::Instant;
    
    /// 缓存项
    #[derive(Debug, Clone)]
    struct CacheItem<T> {
        value: T,
        created_at: Instant,
        accessed_at: Arc<RwLock<Instant>>,
        ttl: Option<Duration>,
    }
    
    /// 简单的内存缓存
    pub struct SimpleCache<K, V>
    where
        K: std::hash::Hash + Eq + Clone,
        V: Clone,
    {
        items: Arc<RwLock<HashMap<K, CacheItem<V>>>>,
        default_ttl: Option<Duration>,
        max_size: usize,
    }
    
    impl<K, V> SimpleCache<K, V>
    where
        K: std::hash::Hash + Eq + Clone,
        V: Clone,
    {
        /// 创建新的缓存
        pub fn new(max_size: usize, default_ttl: Option<Duration>) -> Self {
            Self {
                items: Arc::new(RwLock::new(HashMap::new())),
                default_ttl,
                max_size,
            }
        }
        
        /// 插入缓存项
        pub async fn insert(&self, key: K, value: V) {
            self.insert_with_ttl(key, value, self.default_ttl).await;
        }
        
        /// 插入带TTL的缓存项
        pub async fn insert_with_ttl(&self, key: K, value: V, ttl: Option<Duration>) {
            let mut items = self.items.write().await;
            
            // 检查缓存大小限制
            if items.len() >= self.max_size && !items.contains_key(&key) {
                // 移除最老的项
                if let Some(oldest_key) = self.find_oldest_key(&items).await {
                    items.remove(&oldest_key);
                }
            }
            
            let item = CacheItem {
                value,
                created_at: Instant::now(),
                accessed_at: Arc::new(RwLock::new(Instant::now())),
                ttl,
            };
            
            items.insert(key, item);
        }
        
        /// 获取缓存项
        pub async fn get(&self, key: &K) -> Option<V> {
            let items = self.items.read().await;
            if let Some(item) = items.get(key) {
                // 检查TTL
                if let Some(ttl) = item.ttl {
                    if item.created_at.elapsed() > ttl {
                        drop(items);
                        self.remove(key).await;
                        return None;
                    }
                }
                
                // 更新访问时间
                let mut accessed_at = item.accessed_at.write().await;
                *accessed_at = Instant::now();
                
                Some(item.value.clone())
            } else {
                None
            }
        }
        
        /// 移除缓存项
        pub async fn remove(&self, key: &K) {
            let mut items = self.items.write().await;
            items.remove(key);
        }
        
        /// 清空缓存
        pub async fn clear(&self) {
            let mut items = self.items.write().await;
            items.clear();
        }
        
        /// 获取缓存大小
        pub async fn size(&self) -> usize {
            let items = self.items.read().await;
            items.len()
        }
        
        /// 查找最老的缓存项键
        async fn find_oldest_key(&self, items: &HashMap<K, CacheItem<V>>) -> Option<K> {
            let mut oldest_key = None;
            let mut oldest_time = Instant::now();
            
            for (key, item) in items.iter() {
                let accessed_at = item.accessed_at.read().await;
                if *accessed_at < oldest_time {
                    oldest_time = *accessed_at;
                    oldest_key = Some(key.clone());
                }
            }
            
            oldest_key
        }
        
        /// 清理过期项
        pub async fn cleanup_expired(&self) {
            let mut items = self.items.write().await;
            let now = Instant::now();
            
            items.retain(|_, item| {
                if let Some(ttl) = item.ttl {
                    item.created_at + ttl > now
                } else {
                    true
                }
            });
        }
    }
}

/// ID生成器 - 各种ID生成工具
pub mod id {
    use super::*;
    
    /// 生成UUID v4
    pub fn generate_uuid() -> Uuid {
        Uuid::new_v4()
    }
    
    /// 生成短ID（8字符）
    pub fn generate_short_id() -> String {
        generate_uuid().to_string()[..8].to_string()
    }
    
    /// 生成会话ID
    pub fn generate_session_id() -> String {
        format!("session_{}", generate_short_id())
    }
    
    /// 生成请求ID
    pub fn generate_request_id() -> String {
        format!("req_{}", generate_short_id())
    }
    
    /// 生成追踪ID
    pub fn generate_trace_id() -> String {
        format!("trace_{}", generate_uuid())
    }
}

/// 通用宏定义
#[macro_export]
macro_rules! log_duration {
    ($operation:expr, $code:block) => {{
        let start = std::time::Instant::now();
        let result = $code;
        let duration = start.elapsed();
        tracing::info!("{} completed in {}", $operation, $crate::utils::time::format_duration(duration));
        result
    }};
}

#[macro_export]
macro_rules! measure_async {
    ($operation:expr, $code:block) => {{
        let start = std::time::Instant::now();
        let result = $code.await;
        let duration = start.elapsed();
        tracing::debug!("{} took {}", $operation, $crate::utils::time::format_duration(duration));
        result
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_time_format_duration() {
        assert_eq!(time::format_duration(Duration::from_millis(500)), "500ms");
        assert_eq!(time::format_duration(Duration::from_secs(1)), "1.0s");
        assert_eq!(time::format_duration(Duration::from_secs(65)), "1m5s");
    }
    
    #[test]
    fn test_string_similarity() {
        assert_eq!(string::similarity("hello", "hello"), 1.0);
        assert_eq!(string::similarity("", ""), 1.0);
        assert!(string::similarity("hello", "hallo") > 0.8);
        assert!(string::similarity("hello", "world") < 0.5);
    }
    
    #[test]
    fn test_url_validation() {
        assert!(url::is_valid_url("https://example.com"));
        assert!(url::is_valid_url("http://localhost:8080/path"));
        assert!(!url::is_valid_url("not-a-url"));
        assert!(!url::is_valid_url(""));
    }
    
    #[test]
    fn test_hash_functions() {
        let text = "hello world";
        let hash1 = hash::sha256_string(text);
        let hash2 = hash::sha256_string(text);
        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 64); // SHA256 is 64 hex characters
    }
    
    #[test]
    fn test_memory_format() {
        assert_eq!(memory::format_bytes(1024), "1.00 KB");
        assert_eq!(memory::format_bytes(1024 * 1024), "1.00 MB");
        assert_eq!(memory::format_bytes(1024 * 1024 * 1024), "1.00 GB");
    }
    
    #[tokio::test]
    async fn test_simple_cache() {
        let cache = cache::SimpleCache::new(2, Some(Duration::from_secs(1)));
        
        cache.insert("key1", "value1").await;
        assert_eq!(cache.get(&"key1").await, Some("value1".to_string()));
        
        // 测试TTL
        tokio::time::sleep(Duration::from_secs(2)).await;
        assert_eq!(cache.get(&"key1").await, None);
    }
}