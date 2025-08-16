use anyhow::{Result, Context};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use url::Url;
use tracing::{warn, info};

/// Security configuration
#[derive(Debug, Clone)]
pub struct SecurityConfig {
    /// Maximum requests per minute
    pub rate_limit: u32,
    
    /// Allowed URL schemes
    pub allowed_schemes: Vec<String>,
    
    /// Blocked domains
    pub blocked_domains: Vec<String>,
    
    /// Maximum URL length
    pub max_url_length: usize,
    
    /// Maximum input length
    pub max_input_length: usize,
    
    /// Enable SSRF protection
    pub ssrf_protection: bool,
    
    /// Enable XSS protection
    pub xss_protection: bool,
    
    /// Maximum file size for uploads
    pub max_file_size: usize,
    
    /// Allowed file extensions
    pub allowed_extensions: Vec<String>,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            rate_limit: 60, // 60 requests per minute
            allowed_schemes: vec!["http".to_string(), "https".to_string()],
            blocked_domains: vec![
                "localhost".to_string(),
                "127.0.0.1".to_string(),
                "0.0.0.0".to_string(),
                "169.254.169.254".to_string(), // AWS metadata endpoint
            ],
            max_url_length: 2048,
            max_input_length: 10000,
            ssrf_protection: true,
            xss_protection: true,
            max_file_size: 10 * 1024 * 1024, // 10MB
            allowed_extensions: vec![
                "png".to_string(),
                "jpg".to_string(),
                "jpeg".to_string(),
                "gif".to_string(),
                "pdf".to_string(),
                "txt".to_string(),
            ],
        }
    }
}

/// Rate limiter implementation
pub struct RateLimiter {
    /// Track request counts per IP/identifier
    requests: Arc<RwLock<HashMap<String, Vec<Instant>>>>,
    
    /// Rate limit configuration
    limit: u32,
    
    /// Time window for rate limiting
    window: Duration,
}

impl RateLimiter {
    /// Create a new rate limiter
    pub fn new(requests_per_minute: u32) -> Self {
        Self {
            requests: Arc::new(RwLock::new(HashMap::new())),
            limit: requests_per_minute,
            window: Duration::from_secs(60),
        }
    }
    
    /// Check if a request is allowed
    pub async fn check(&self, identifier: &str) -> Result<bool> {
        let mut requests = self.requests.write().await;
        let now = Instant::now();
        
        // Get or create request history for this identifier
        let history = requests.entry(identifier.to_string()).or_insert_with(Vec::new);
        
        // Remove old requests outside the window
        history.retain(|&timestamp| now.duration_since(timestamp) < self.window);
        
        // Check if under limit
        if history.len() >= self.limit as usize {
            warn!("Rate limit exceeded for {}", identifier);
            return Ok(false);
        }
        
        // Record this request
        history.push(now);
        Ok(true)
    }
    
    /// Clean up old entries
    pub async fn cleanup(&self) {
        let mut requests = self.requests.write().await;
        let now = Instant::now();
        
        requests.retain(|_, history| {
            history.retain(|&timestamp| now.duration_since(timestamp) < self.window);
            !history.is_empty()
        });
        
        info!("Rate limiter cleanup: {} active identifiers", requests.len());
    }
}

/// Input validator for security
pub struct InputValidator {
    config: SecurityConfig,
}

impl InputValidator {
    /// Create a new input validator
    pub fn new(config: SecurityConfig) -> Self {
        Self { config }
    }
    
    /// Validate a URL for safety
    pub fn validate_url(&self, url_str: &str) -> Result<Url> {
        // Check length
        if url_str.len() > self.config.max_url_length {
            return Err(anyhow::anyhow!("URL exceeds maximum length"));
        }
        
        // Parse URL
        let url = Url::parse(url_str)
            .context("Invalid URL format")?;
        
        // Check scheme
        let scheme = url.scheme();
        if !self.config.allowed_schemes.contains(&scheme.to_string()) {
            return Err(anyhow::anyhow!("URL scheme '{}' not allowed", scheme));
        }
        
        // Check for blocked domains (SSRF protection)
        if self.config.ssrf_protection {
            if let Some(host) = url.host_str() {
                // Check against blocked domains
                for blocked in &self.config.blocked_domains {
                    if host == blocked || host.ends_with(&format!(".{}", blocked)) {
                        return Err(anyhow::anyhow!("Access to domain '{}' is blocked", host));
                    }
                }
                
                // Check for private IP addresses
                if self.is_private_ip(host) {
                    return Err(anyhow::anyhow!("Access to private IP addresses is blocked"));
                }
            }
        }
        
        Ok(url)
    }
    
    /// Check if a host is a private IP
    fn is_private_ip(&self, host: &str) -> bool {
        if let Ok(ip) = host.parse::<std::net::IpAddr>() {
            match ip {
                std::net::IpAddr::V4(ipv4) => {
                    ipv4.is_private() || 
                    ipv4.is_loopback() || 
                    ipv4.is_link_local() ||
                    ipv4.is_unspecified()
                }
                std::net::IpAddr::V6(ipv6) => {
                    ipv6.is_loopback() || 
                    ipv6.is_unspecified()
                }
            }
        } else {
            false
        }
    }
    
    /// Validate general text input
    pub fn validate_input(&self, input: &str, max_length: Option<usize>) -> Result<String> {
        let max_len = max_length.unwrap_or(self.config.max_input_length);
        
        // Check length
        if input.len() > max_len {
            return Err(anyhow::anyhow!("Input exceeds maximum length"));
        }
        
        // Sanitize for XSS if enabled
        let sanitized = if self.config.xss_protection {
            self.sanitize_html(input)
        } else {
            input.to_string()
        };
        
        // Check for null bytes
        if sanitized.contains('\0') {
            return Err(anyhow::anyhow!("Input contains null bytes"));
        }
        
        // Check for control characters (except newline and tab)
        for ch in sanitized.chars() {
            if ch.is_control() && ch != '\n' && ch != '\t' && ch != '\r' {
                return Err(anyhow::anyhow!("Input contains invalid control characters"));
            }
        }
        
        Ok(sanitized)
    }
    
    /// Basic HTML sanitization
    fn sanitize_html(&self, input: &str) -> String {
        input
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#x27;")
            .replace('/', "&#x2F;")
    }
    
    /// Validate file upload
    pub fn validate_file(&self, filename: &str, size: usize, content: &[u8]) -> Result<()> {
        // Check file size
        if size > self.config.max_file_size {
            return Err(anyhow::anyhow!("File size exceeds maximum allowed"));
        }
        
        // Check file extension
        if let Some(extension) = std::path::Path::new(filename)
            .extension()
            .and_then(|e| e.to_str())
        {
            if !self.config.allowed_extensions.contains(&extension.to_lowercase()) {
                return Err(anyhow::anyhow!("File type '{}' not allowed", extension));
            }
        } else {
            return Err(anyhow::anyhow!("File must have an extension"));
        }
        
        // Check file magic bytes for common file types
        self.validate_file_magic(content, filename)?;
        
        Ok(())
    }
    
    /// Validate file magic bytes
    fn validate_file_magic(&self, content: &[u8], filename: &str) -> Result<()> {
        if content.len() < 4 {
            return Err(anyhow::anyhow!("File too small"));
        }
        
        let extension = std::path::Path::new(filename)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        
        match extension.as_str() {
            "png" => {
                if &content[0..8] != b"\x89PNG\r\n\x1a\n" {
                    return Err(anyhow::anyhow!("Invalid PNG file"));
                }
            }
            "jpg" | "jpeg" => {
                if &content[0..2] != b"\xFF\xD8" {
                    return Err(anyhow::anyhow!("Invalid JPEG file"));
                }
            }
            "gif" => {
                if &content[0..6] != b"GIF87a" && &content[0..6] != b"GIF89a" {
                    return Err(anyhow::anyhow!("Invalid GIF file"));
                }
            }
            "pdf" => {
                if &content[0..4] != b"%PDF" {
                    return Err(anyhow::anyhow!("Invalid PDF file"));
                }
            }
            _ => {} // Other file types don't have magic byte validation
        }
        
        Ok(())
    }
    
    /// Validate workflow YAML/JSON
    pub fn validate_workflow(&self, content: &str) -> Result<()> {
        // Check size
        if content.len() > self.config.max_input_length * 10 {
            return Err(anyhow::anyhow!("Workflow file too large"));
        }
        
        // Basic validation - try to parse as YAML or JSON
        if content.trim().starts_with('{') {
            // Try JSON
            serde_json::from_str::<serde_json::Value>(content)
                .context("Invalid JSON in workflow")?;
        } else {
            // Try YAML
            serde_yaml::from_str::<serde_yaml::Value>(content)
                .context("Invalid YAML in workflow")?;
        }
        
        // Check for suspicious patterns
        let suspicious_patterns = [
            "file:///",
            "data:",
            "javascript:",
            "../",
            "\\x",
            "%00",
            "\0",
        ];
        
        for pattern in &suspicious_patterns {
            if content.contains(pattern) {
                return Err(anyhow::anyhow!("Workflow contains suspicious pattern: {}", pattern));
            }
        }
        
        Ok(())
    }
}

/// Security middleware for the application
pub struct SecurityMiddleware {
    rate_limiter: RateLimiter,
    validator: InputValidator,
}

impl SecurityMiddleware {
    /// Create new security middleware
    pub fn new(config: SecurityConfig) -> Self {
        let rate_limit = config.rate_limit;
        Self {
            rate_limiter: RateLimiter::new(rate_limit),
            validator: InputValidator::new(config),
        }
    }
    
    /// Check if request is allowed
    pub async fn check_request(&self, identifier: &str) -> Result<()> {
        if !self.rate_limiter.check(identifier).await? {
            return Err(anyhow::anyhow!("Rate limit exceeded"));
        }
        Ok(())
    }
    
    /// Validate URL input
    pub fn validate_url(&self, url: &str) -> Result<Url> {
        self.validator.validate_url(url)
    }
    
    /// Validate text input
    pub fn validate_input(&self, input: &str) -> Result<String> {
        self.validator.validate_input(input, None)
    }
    
    /// Validate workflow YAML/JSON
    pub fn validate_workflow(&self, workflow_content: &str) -> Result<()> {
        self.validator.validate_workflow(workflow_content)
    }
    
    /// Start background cleanup task
    pub fn start_cleanup_task(self: Arc<Self>) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(300)); // Every 5 minutes
            loop {
                interval.tick().await;
                self.rate_limiter.cleanup().await;
            }
        });
    }
}

/// Password strength validator
pub struct PasswordValidator {
    min_length: usize,
    require_uppercase: bool,
    require_lowercase: bool,
    require_digit: bool,
    require_special: bool,
}

impl Default for PasswordValidator {
    fn default() -> Self {
        Self {
            min_length: 8,
            require_uppercase: true,
            require_lowercase: true,
            require_digit: true,
            require_special: true,
        }
    }
}

impl PasswordValidator {
    /// Validate password strength
    pub fn validate(&self, password: &str) -> Result<()> {
        if password.len() < self.min_length {
            return Err(anyhow::anyhow!("Password must be at least {} characters", self.min_length));
        }
        
        if self.require_uppercase && !password.chars().any(|c| c.is_uppercase()) {
            return Err(anyhow::anyhow!("Password must contain an uppercase letter"));
        }
        
        if self.require_lowercase && !password.chars().any(|c| c.is_lowercase()) {
            return Err(anyhow::anyhow!("Password must contain a lowercase letter"));
        }
        
        if self.require_digit && !password.chars().any(|c| c.is_ascii_digit()) {
            return Err(anyhow::anyhow!("Password must contain a digit"));
        }
        
        if self.require_special && !password.chars().any(|c| !c.is_alphanumeric()) {
            return Err(anyhow::anyhow!("Password must contain a special character"));
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_rate_limiter() {
        let limiter = RateLimiter::new(3); // 3 requests per minute
        
        // First 3 requests should pass
        assert!(limiter.check("user1").await.unwrap());
        assert!(limiter.check("user1").await.unwrap());
        assert!(limiter.check("user1").await.unwrap());
        
        // 4th request should fail
        assert!(!limiter.check("user1").await.unwrap());
        
        // Different user should pass
        assert!(limiter.check("user2").await.unwrap());
    }
    
    #[test]
    fn test_url_validation() {
        let config = SecurityConfig::default();
        let validator = InputValidator::new(config);
        
        // Valid URLs
        assert!(validator.validate_url("https://example.com").is_ok());
        assert!(validator.validate_url("http://google.com/search").is_ok());
        
        // Invalid URLs
        assert!(validator.validate_url("javascript:alert(1)").is_err());
        assert!(validator.validate_url("file:///etc/passwd").is_err());
        assert!(validator.validate_url("http://localhost/admin").is_err());
        assert!(validator.validate_url("http://127.0.0.1:8080").is_err());
        assert!(validator.validate_url("http://169.254.169.254/").is_err());
    }
    
    #[test]
    fn test_input_sanitization() {
        let config = SecurityConfig::default();
        let validator = InputValidator::new(config);
        
        // XSS attempts should be sanitized
        let input = "<script>alert('xss')</script>";
        let result = validator.validate_input(input, None).unwrap();
        assert!(!result.contains("<script>"));
        assert!(result.contains("&lt;script&gt;"));
        
        // Null bytes should be rejected
        let input = "hello\0world";
        assert!(validator.validate_input(input, None).is_err());
    }
    
    #[test]
    fn test_password_validation() {
        let validator = PasswordValidator::default();
        
        // Valid password
        assert!(validator.validate("SecureP@ss123").is_ok());
        
        // Too short
        assert!(validator.validate("Pass1!").is_err());
        
        // Missing uppercase
        assert!(validator.validate("password123!").is_err());
        
        // Missing special character
        assert!(validator.validate("Password123").is_err());
    }
}