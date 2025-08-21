//! Security Module - Input Validation and Sanitization
//! 
//! CRITICAL: This module provides security primitives to prevent:
//! - Script injection attacks
//! - Path traversal vulnerabilities  
//! - Command injection
//! - XSS attacks

use regex::Regex;
use std::path::{Path, PathBuf};
use url::Url;
use lazy_static::lazy_static;
use thiserror::Error;

/// Security-related errors
#[derive(Debug, Error)]
pub enum SecurityError {
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    
    #[error("Potentially malicious selector: {0}")]
    MaliciousSelector(String),
    
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
    
    #[error("Path traversal attempt: {0}")]
    PathTraversal(String),
    
    #[error("Forbidden characters in input: {0}")]
    ForbiddenCharacters(String),
    
    #[error("Input exceeds maximum length: {max} characters")]
    InputTooLong { max: usize },
}

lazy_static! {
    /// Regex for valid CSS selectors (conservative)
    static ref VALID_CSS_SELECTOR: Regex = Regex::new(
        r"^[a-zA-Z0-9\s\-_#\.\[\]=':,>+~\(\)]+$"
    ).unwrap();
    
    /// Regex for detecting script tags
    static ref SCRIPT_TAG: Regex = Regex::new(
        r"(?i)<\s*script[^>]*>.*?</\s*script\s*>"
    ).unwrap();
    
    /// Regex for detecting event handlers
    static ref EVENT_HANDLERS: Regex = Regex::new(
        r"(?i)\bon\w+\s*="
    ).unwrap();
    
    /// Regex for JavaScript protocol
    static ref JS_PROTOCOL: Regex = Regex::new(
        r"(?i)javascript:"
    ).unwrap();
}

/// Input sanitization utilities
pub struct InputSanitizer;

impl InputSanitizer {
    /// Sanitize CSS selector to prevent injection
    pub fn sanitize_selector(selector: &str) -> Result<String, SecurityError> {
        // Check length
        if selector.len() > 500 {
            return Err(SecurityError::InputTooLong { max: 500 });
        }
        
        // Remove any quotes and backslashes that could break out
        let cleaned = selector
            .replace('\'', "")
            .replace('"', "")
            .replace('\\', "")
            .replace('\n', "")
            .replace('\r', "")
            .trim()
            .to_string();
        
        // Check if it matches valid CSS selector pattern
        if !VALID_CSS_SELECTOR.is_match(&cleaned) {
            return Err(SecurityError::MaliciousSelector(
                format!("Invalid characters in selector: {}", selector)
            ));
        }
        
        // Additional checks for malicious patterns
        if cleaned.contains("javascript:") || 
           cleaned.contains("<script") ||
           cleaned.contains("onerror") ||
           cleaned.contains("onclick") {
            return Err(SecurityError::MaliciousSelector(
                "Potentially malicious content detected".to_string()
            ));
        }
        
        Ok(cleaned)
    }
    
    /// Sanitize text input to prevent XSS
    pub fn sanitize_text(text: &str) -> String {
        text
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#x27;")
            .replace('/', "&#x2F;")
    }
    
    /// Validate and sanitize URLs
    pub fn validate_url(url_str: &str) -> Result<String, SecurityError> {
        // Parse URL
        let url = Url::parse(url_str)
            .map_err(|e| SecurityError::InvalidUrl(e.to_string()))?;
        
        // Check protocol - only allow http(s)
        match url.scheme() {
            "http" | "https" => {},
            scheme => {
                return Err(SecurityError::InvalidUrl(
                    format!("Forbidden protocol: {}", scheme)
                ));
            }
        }
        
        // Check for javascript: protocol in any form
        if JS_PROTOCOL.is_match(url_str) {
            return Err(SecurityError::InvalidUrl(
                "JavaScript protocol not allowed".to_string()
            ));
        }
        
        // Check for localhost/internal IPs (optional - depends on requirements)
        if let Some(host) = url.host_str() {
            if host == "localhost" || 
               host == "127.0.0.1" || 
               host.starts_with("192.168.") ||
               host.starts_with("10.") ||
               host.starts_with("172.") {
                // You might want to make this configurable
                // return Err(SecurityError::InvalidUrl(
                //     "Internal URLs not allowed".to_string()
                // ));
            }
        }
        
        Ok(url.to_string())
    }
    
    /// Validate file paths to prevent traversal attacks
    pub fn validate_path(path_str: &str, base_dir: Option<&Path>) -> Result<PathBuf, SecurityError> {
        let path = Path::new(path_str);
        
        // Check for path traversal attempts
        for component in path.components() {
            match component {
                std::path::Component::ParentDir => {
                    return Err(SecurityError::PathTraversal(
                        "Parent directory references not allowed".to_string()
                    ));
                }
                std::path::Component::RootDir => {
                    return Err(SecurityError::PathTraversal(
                        "Absolute paths not allowed".to_string()
                    ));
                }
                _ => {}
            }
        }
        
        // If base directory is provided, ensure path is within it
        if let Some(base) = base_dir {
            let full_path = base.join(path);
            let canonical = full_path.canonicalize()
                .map_err(|_| SecurityError::PathTraversal(
                    "Invalid path".to_string()
                ))?;
            
            if !canonical.starts_with(base) {
                return Err(SecurityError::PathTraversal(
                    "Path outside allowed directory".to_string()
                ));
            }
            
            Ok(canonical)
        } else {
            Ok(path.to_path_buf())
        }
    }
    
    /// Create a parameterized script for safe execution
    pub fn create_safe_script(template: &str, params: Vec<(&str, &str)>) -> String {
        let mut script = template.to_string();
        
        for (placeholder, value) in params {
            // Escape the value for JavaScript
            let escaped = Self::escape_js_string(value);
            script = script.replace(placeholder, &escaped);
        }
        
        script
    }
    
    /// Escape string for safe JavaScript inclusion
    pub fn escape_js_string(s: &str) -> String {
        s.chars()
            .map(|c| match c {
                '"' => r#"\""#.to_string(),
                '\'' => r"\'".to_string(),
                '\\' => r"\\".to_string(),
                '\n' => r"\n".to_string(),
                '\r' => r"\r".to_string(),
                '\t' => r"\t".to_string(),
                '\u{0008}' => r"\b".to_string(),
                '\u{000C}' => r"\f".to_string(),
                c if c.is_control() => format!(r"\u{:04x}", c as u32),
                c => c.to_string(),
            })
            .collect()
    }
    
    /// Validate that a string doesn't contain script injection attempts
    pub fn validate_no_scripts(input: &str) -> Result<(), SecurityError> {
        if SCRIPT_TAG.is_match(input) {
            return Err(SecurityError::ForbiddenCharacters(
                "Script tags not allowed".to_string()
            ));
        }
        
        if EVENT_HANDLERS.is_match(input) {
            return Err(SecurityError::ForbiddenCharacters(
                "Event handlers not allowed".to_string()
            ));
        }
        
        if JS_PROTOCOL.is_match(input) {
            return Err(SecurityError::ForbiddenCharacters(
                "JavaScript protocol not allowed".to_string()
            ));
        }
        
        Ok(())
    }
}

/// Secure credential storage
pub struct SecureCredentials {
    data: Vec<u8>,
}

impl SecureCredentials {
    /// Create new secure credentials (in production, use encryption)
    pub fn new(sensitive_data: &str) -> Self {
        // TODO: In production, encrypt this data
        // For now, just store as bytes to prevent accidental logging
        Self {
            data: sensitive_data.as_bytes().to_vec(),
        }
    }
    
    /// Get the credentials (in production, decrypt)
    pub fn reveal(&self) -> String {
        // TODO: In production, decrypt this data
        String::from_utf8_lossy(&self.data).to_string()
    }
}

impl Drop for SecureCredentials {
    fn drop(&mut self) {
        // Zero out memory when dropped
        self.data.iter_mut().for_each(|b| *b = 0);
    }
}

/// Rate limiting for tool execution
pub struct RateLimiter {
    max_requests: usize,
    window_seconds: u64,
    requests: std::collections::VecDeque<std::time::Instant>,
}

impl RateLimiter {
    pub fn new(max_requests: usize, window_seconds: u64) -> Self {
        Self {
            max_requests,
            window_seconds,
            requests: std::collections::VecDeque::new(),
        }
    }
    
    pub fn check_rate_limit(&mut self) -> Result<(), SecurityError> {
        let now = std::time::Instant::now();
        let window = std::time::Duration::from_secs(self.window_seconds);
        
        // Remove old requests outside the window
        while let Some(&front) = self.requests.front() {
            if now.duration_since(front) > window {
                self.requests.pop_front();
            } else {
                break;
            }
        }
        
        // Check if we're at the limit
        if self.requests.len() >= self.max_requests {
            return Err(SecurityError::InvalidInput(
                "Rate limit exceeded".to_string()
            ));
        }
        
        // Add this request
        self.requests.push_back(now);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_sanitize_selector() {
        // Valid selectors
        assert!(InputSanitizer::sanitize_selector("#id").is_ok());
        assert!(InputSanitizer::sanitize_selector(".class").is_ok());
        assert!(InputSanitizer::sanitize_selector("div > span").is_ok());
        assert!(InputSanitizer::sanitize_selector("[data-attr='value']").is_ok());
        
        // Invalid selectors
        assert!(InputSanitizer::sanitize_selector("'; DROP TABLE--").is_err());
        assert!(InputSanitizer::sanitize_selector("<script>alert(1)</script>").is_err());
        assert!(InputSanitizer::sanitize_selector("javascript:alert(1)").is_err());
    }
    
    #[test]
    fn test_validate_url() {
        // Valid URLs
        assert!(InputSanitizer::validate_url("https://example.com").is_ok());
        assert!(InputSanitizer::validate_url("http://google.com/search").is_ok());
        
        // Invalid URLs
        assert!(InputSanitizer::validate_url("javascript:alert(1)").is_err());
        assert!(InputSanitizer::validate_url("file:///etc/passwd").is_err());
        assert!(InputSanitizer::validate_url("ftp://example.com").is_err());
    }
    
    #[test]
    fn test_validate_path() {
        let base = Path::new("/safe/directory");
        
        // Valid paths
        assert!(InputSanitizer::validate_path("subdir/file.txt", Some(base)).is_ok());
        
        // Invalid paths
        assert!(InputSanitizer::validate_path("../../../etc/passwd", Some(base)).is_err());
        assert!(InputSanitizer::validate_path("/etc/passwd", Some(base)).is_err());
    }
    
    #[test]
    fn test_escape_js_string() {
        assert_eq!(
            InputSanitizer::escape_js_string("Hello \"World\""),
            r#"Hello \"World\""#
        );
        assert_eq!(
            InputSanitizer::escape_js_string("Line1\nLine2"),
            r"Line1\nLine2"
        );
    }
}