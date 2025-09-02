// Enhanced Error Recovery System for Perception Module
// Provides intelligent error handling, graceful degradation, and retry mechanisms

use anyhow::{Result, Context};
use serde::{Serialize, Deserialize};
use std::time::{Duration, Instant};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn, error};
use thirtyfour::{WebDriver, WebElement, By};

use crate::smart_element_detector::{SmartElementDetector, ElementDescriptor, ElementType};

/// Enhanced error recovery system with intelligent fallback strategies
pub struct EnhancedErrorRecovery {
    detector: SmartElementDetector,
    recovery_stats: Arc<RwLock<RecoveryStats>>,
    config: RecoveryConfig,
}

#[derive(Debug, Clone)]
pub struct RecoveryConfig {
    pub max_retries: u32,
    pub base_delay: Duration,
    pub max_delay: Duration,
    pub fallback_enabled: bool,
    pub partial_results_threshold: f32,
    pub graceful_degradation: bool,
}

impl Default for RecoveryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(5),
            fallback_enabled: true,
            partial_results_threshold: 0.6, // Accept results with 60%+ confidence
            graceful_degradation: true,
        }
    }
}

#[derive(Debug, Default)]
pub struct RecoveryStats {
    pub total_attempts: u64,
    pub successful_recoveries: u64,
    pub fallback_uses: u64,
    pub partial_successes: u64,
    pub complete_failures: u64,
    pub average_recovery_time_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryResult<T> {
    pub result: Option<T>,
    pub success: bool,
    pub recovery_strategy_used: RecoveryStrategy,
    pub confidence: f32,
    pub execution_time_ms: u64,
    pub error_message: Option<String>,
    pub partial_data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecoveryStrategy {
    DirectSuccess,
    RetrySuccess,
    FallbackSelector,
    AlternativeElement,
    PartialResult,
    GracefulDegradation,
    CompleteFailure,
}

impl EnhancedErrorRecovery {
    pub fn new(driver: WebDriver, config: Option<RecoveryConfig>) -> Self {
        Self {
            detector: SmartElementDetector::new(driver),
            recovery_stats: Arc::new(RwLock::new(RecoveryStats::default())),
            config: config.unwrap_or_default(),
        }
    }

    /// Find element with intelligent error recovery
    pub async fn find_element_with_recovery(
        &self,
        descriptor: &ElementDescriptor,
    ) -> RecoveryResult<WebElement> {
        let start_time = Instant::now();
        
        // Update stats
        {
            let mut stats = self.recovery_stats.write().await;
            stats.total_attempts += 1;
        }

        // Strategy 1: Direct attempt
        match self.detector.find_element(descriptor).await {
            Ok(element) => {
                return RecoveryResult {
                    result: Some(element),
                    success: true,
                    recovery_strategy_used: RecoveryStrategy::DirectSuccess,
                    confidence: 1.0,
                    execution_time_ms: start_time.elapsed().as_millis() as u64,
                    error_message: None,
                    partial_data: None,
                };
            },
            Err(e) => {
                debug!("Direct attempt failed: {}", e);
            }
        }

        // Strategy 2: Retry with exponential backoff
        if let Ok(element) = self.retry_with_backoff(descriptor).await {
            self.update_success_stats(start_time).await;
            return RecoveryResult {
                result: Some(element),
                success: true,
                recovery_strategy_used: RecoveryStrategy::RetrySuccess,
                confidence: 0.9,
                execution_time_ms: start_time.elapsed().as_millis() as u64,
                error_message: None,
                partial_data: None,
            };
        }

        // Strategy 3: Try alternative selectors
        if let Ok(element) = self.try_alternative_selectors(descriptor).await {
            self.update_success_stats(start_time).await;
            return RecoveryResult {
                result: Some(element),
                success: true,
                recovery_strategy_used: RecoveryStrategy::FallbackSelector,
                confidence: 0.8,
                execution_time_ms: start_time.elapsed().as_millis() as u64,
                error_message: None,
                partial_data: None,
            };
        }

        // Strategy 4: Look for similar elements
        if let Ok(element) = self.find_similar_element(descriptor).await {
            self.update_success_stats(start_time).await;
            return RecoveryResult {
                result: Some(element),
                success: true,
                recovery_strategy_used: RecoveryStrategy::AlternativeElement,
                confidence: 0.7,
                execution_time_ms: start_time.elapsed().as_millis() as u64,
                error_message: None,
                partial_data: None,
            };
        }

        // Strategy 5: Graceful degradation - return partial information
        if self.config.graceful_degradation {
            if let Ok(partial_data) = self.gather_partial_information(descriptor).await {
                self.update_partial_success_stats(start_time).await;
                return RecoveryResult {
                    result: None,
                    success: false,
                    recovery_strategy_used: RecoveryStrategy::PartialResult,
                    confidence: 0.5,
                    execution_time_ms: start_time.elapsed().as_millis() as u64,
                    error_message: Some("Element not found, but gathered partial information".to_string()),
                    partial_data: Some(partial_data),
                };
            }
        }

        // Strategy 6: Complete failure with detailed error information
        self.update_failure_stats(start_time).await;
        RecoveryResult {
            result: None,
            success: false,
            recovery_strategy_used: RecoveryStrategy::CompleteFailure,
            confidence: 0.0,
            execution_time_ms: start_time.elapsed().as_millis() as u64,
            error_message: Some(format!(
                "Failed to find element '{}' after all recovery strategies",
                descriptor.description
            )),
            partial_data: None,
        }
    }

    /// Retry with exponential backoff
    async fn retry_with_backoff(&self, descriptor: &ElementDescriptor) -> Result<WebElement> {
        let mut delay = self.config.base_delay;
        
        for attempt in 0..self.config.max_retries {
            debug!("Retry attempt {} for element: {}", attempt + 1, descriptor.description);
            
            // Wait before retrying
            tokio::time::sleep(delay).await;
            
            // Try again
            if let Ok(element) = self.detector.find_element(descriptor).await {
                info!("Element found on retry attempt {}", attempt + 1);
                return Ok(element);
            }
            
            // Exponential backoff
            delay = std::cmp::min(delay * 2, self.config.max_delay);
        }
        
        Err(anyhow::anyhow!("Element not found after {} retries", self.config.max_retries))
    }

    /// Try alternative selectors based on element type
    async fn try_alternative_selectors(&self, descriptor: &ElementDescriptor) -> Result<WebElement> {
        debug!("Trying alternative selectors for: {}", descriptor.description);
        
        let alternatives = match descriptor.element_type {
            ElementType::SearchBox => vec![
                "input[type='text']:first-of-type",
                "input:not([type='hidden']):first-of-type",
                "[placeholder*='search' i]",
                "[aria-label*='search' i]",
                "form input[type='text']",
            ],
            ElementType::Button => vec![
                "button:first-of-type",
                "[role='button']",
                "input[type='submit']",
                "input[type='button']",
                ".btn:first-of-type",
            ],
            ElementType::Link => vec![
                "a[href]:first-of-type",
                "[role='link']",
                ".link:first-of-type",
            ],
            ElementType::Input => vec![
                "input[type='text']",
                "input:not([type='hidden']):not([type='submit']):not([type='button'])",
                "textarea",
            ],
            _ => vec![],
        };

        for selector in alternatives {
            if let Ok(element) = self.detector.try_selector(selector).await {
                debug!("Found element using alternative selector: {}", selector);
                return Ok(element);
            }
        }

        Err(anyhow::anyhow!("No alternative selectors worked"))
    }

    /// Find similar elements that might serve the same purpose
    async fn find_similar_element(&self, descriptor: &ElementDescriptor) -> Result<WebElement> {
        debug!("Looking for similar elements to: {}", descriptor.description);
        
        // Extract keywords from description
        let keywords = self.extract_keywords(&descriptor.description);
        
        // Try to find elements with similar text content
        for keyword in keywords {
            if keyword.len() > 2 { // Skip very short words
                let xpath = format!(
                    "//*[contains(translate(text(), 'ABCDEFGHIJKLMNOPQRSTUVWXYZ', 'abcdefghijklmnopqrstuvwxyz'), '{}')]",
                    keyword.to_lowercase()
                );
                
                if let Ok(element) = self.detector.driver.find(By::XPath(&xpath)).await {
                    if element.is_displayed().await.unwrap_or(false) {
                        debug!("Found similar element with keyword: {}", keyword);
                        return Ok(element);
                    }
                }
            }
        }

        // Try elements with similar attributes
        let similar_attributes = match descriptor.element_type {
            ElementType::Button => vec!["onclick", "data-action"],
            ElementType::Input => vec!["name", "placeholder"],
            ElementType::Link => vec!["href", "title"],
            _ => vec![],
        };

        for attr in similar_attributes {
            let selector = format!("[{}]", attr);
            if let Ok(elements) = self.detector.driver.find_all(By::Css(&selector)).await {
                for element in elements {
                    if element.is_displayed().await.unwrap_or(false) {
                        debug!("Found similar element with attribute: {}", attr);
                        return Ok(element);
                    }
                }
            }
        }

        Err(anyhow::anyhow!("No similar elements found"))
    }

    /// Gather partial information when element cannot be found
    async fn gather_partial_information(&self, descriptor: &ElementDescriptor) -> Result<serde_json::Value> {
        debug!("Gathering partial information for: {}", descriptor.description);
        
        let mut partial_data = serde_json::Map::new();
        
        // Get page title and URL
        if let Ok(title) = self.detector.driver.title().await {
            partial_data.insert("page_title".to_string(), serde_json::Value::String(title));
        }
        
        if let Ok(url) = self.detector.driver.current_url().await {
            partial_data.insert("page_url".to_string(), serde_json::Value::String(url.to_string()));
        }
        
        // Look for elements of the same type
        let type_elements = match descriptor.element_type {
            ElementType::Button => self.detector.driver.find_all(By::Tag("button")).await.ok(),
            ElementType::Input => self.detector.driver.find_all(By::Tag("input")).await.ok(),
            ElementType::Link => self.detector.driver.find_all(By::Tag("a")).await.ok(),
            _ => None,
        };
        
        if let Some(elements) = type_elements {
            let mut element_info = Vec::new();
            for (i, element) in elements.iter().enumerate().take(5) { // Limit to first 5
                if let (Ok(tag), Ok(text)) = (element.tag_name().await, element.text().await) {
                    element_info.push(serde_json::json!({
                        "index": i,
                        "tag": tag,
                        "text": text.chars().take(50).collect::<String>(), // Limit text length
                        "visible": element.is_displayed().await.unwrap_or(false)
                    }));
                }
            }
            partial_data.insert(
                format!("available_{:?}s", descriptor.element_type).to_lowercase(),
                serde_json::Value::Array(element_info)
            );
        }
        
        // Add search context
        partial_data.insert("search_description".to_string(), 
                          serde_json::Value::String(descriptor.description.clone()));
        partial_data.insert("element_type".to_string(), 
                          serde_json::Value::String(format!("{:?}", descriptor.element_type)));
        
        Ok(serde_json::Value::Object(partial_data))
    }

    /// Extract meaningful keywords from a description
    fn extract_keywords(&self, description: &str) -> Vec<String> {
        let stop_words = vec!["the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for", "of", "with", "by"];
        
        description
            .to_lowercase()
            .split_whitespace()
            .filter(|word| !stop_words.contains(word) && word.len() > 2)
            .map(|word| word.replace(&['(', ')', ',', '.', ';', ':', '!', '?'][..], ""))
            .filter(|word| !word.is_empty())
            .collect()
    }

    /// Update statistics for successful recovery
    async fn update_success_stats(&self, start_time: Instant) {
        let mut stats = self.recovery_stats.write().await;
        stats.successful_recoveries += 1;
        
        let elapsed = start_time.elapsed().as_millis() as f64;
        stats.average_recovery_time_ms = 
            (stats.average_recovery_time_ms * (stats.successful_recoveries - 1) as f64 + elapsed) 
            / stats.successful_recoveries as f64;
    }

    /// Update statistics for partial success
    async fn update_partial_success_stats(&self, start_time: Instant) {
        let mut stats = self.recovery_stats.write().await;
        stats.partial_successes += 1;
        
        let elapsed = start_time.elapsed().as_millis() as f64;
        stats.average_recovery_time_ms = 
            (stats.average_recovery_time_ms * stats.partial_successes as f64 + elapsed) 
            / (stats.partial_successes + 1) as f64;
    }

    /// Update statistics for complete failure
    async fn update_failure_stats(&self, _start_time: Instant) {
        let mut stats = self.recovery_stats.write().await;
        stats.complete_failures += 1;
    }

    /// Get current recovery statistics
    pub async fn get_stats(&self) -> RecoveryStats {
        self.recovery_stats.read().await.clone()
    }

    /// Reset statistics
    pub async fn reset_stats(&self) {
        let mut stats = self.recovery_stats.write().await;
        *stats = RecoveryStats::default();
    }
}

impl SmartElementDetector {
    /// Expose try_selector method for EnhancedErrorRecovery
    pub async fn try_selector(&self, selector: &str) -> Result<WebElement> {
        debug!("Trying selector: {}", selector);
        
        match tokio::time::timeout(
            Duration::from_secs(2),
            self.driver.find(By::Css(selector))
        ).await {
            Ok(Ok(element)) => {
                if element.is_displayed().await.unwrap_or(false) {
                    Ok(element)
                } else {
                    Err(anyhow::anyhow!("Element found but not visible"))
                }
            },
            Ok(Err(e)) => {
                debug!("Selector failed: {} - {}", selector, e);
                Err(e.into())
            },
            Err(_) => {
                debug!("Selector timed out: {}", selector);
                Err(anyhow::anyhow!("Timeout"))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_keywords() {
        let recovery = EnhancedErrorRecovery {
            detector: SmartElementDetector::new(/* mock driver */),
            recovery_stats: Arc::new(RwLock::new(RecoveryStats::default())),
            config: RecoveryConfig::default(),
        };
        
        let keywords = recovery.extract_keywords("click the search button");
        assert_eq!(keywords, vec!["click", "search", "button"]);
        
        let keywords = recovery.extract_keywords("find the login form");
        assert_eq!(keywords, vec!["find", "login", "form"]);
    }

    #[test]
    fn test_recovery_config_default() {
        let config = RecoveryConfig::default();
        assert_eq!(config.max_retries, 3);
        assert!(config.fallback_enabled);
        assert!(config.graceful_degradation);
    }
}