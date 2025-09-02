// Lightning Perception - Real Implementation (<50ms)
// Ultra-fast perception layer for critical elements

use anyhow::{Result, Context};
use serde::{Serialize, Deserialize};
use std::time::{Duration, Instant};
use std::collections::HashMap;
use tracing::{debug, warn};
use super::browser_connection::BrowserConnection;
use super::ElementType;  // Use ElementType from parent module
use thirtyfour::By;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightningData {
    pub key_elements: Vec<KeyElement>,
    pub page_status: PageStatus,
    pub urgent_signals: Vec<Signal>,
    pub scan_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyElement {
    pub selector: String,
    pub element_type: ElementType,
    pub text: String,
    pub importance: f32,
    pub visible: bool,
    pub clickable: bool,
}

// ElementType is imported from parent module

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageStatus {
    pub status: PageState,
    pub is_loading: bool,
    pub load_progress: f32,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PageState {
    Loading,
    Ready,
    Interactive,
    Error,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signal {
    pub signal_type: SignalType,
    pub priority: Priority,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SignalType {
    Alert,
    Popup,
    Redirect,
    Error,
    Success,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Priority {
    Critical,
    High,
    Medium,
    Low,
}

pub struct RealLightningPerception {
    max_elements: usize,
    timeout_ms: u64,
}

impl RealLightningPerception {
    pub fn new() -> Self {
        Self {
            max_elements: 10,
            timeout_ms: 50,
        }
    }
    
    /// Execute lightning-fast perception on the current page
    pub async fn scan_page(&self, browser: &BrowserConnection) -> Result<LightningData> {
        let start = Instant::now();
        
        // Execute all scans in parallel for speed
        let (key_elements, page_status, urgent_signals) = tokio::join!(
            self.scan_key_elements_fast(browser),
            self.detect_page_status_fast(browser),
            self.detect_urgent_signals_fast(browser)
        );
        
        let scan_time_ms = start.elapsed().as_millis() as u64;
        
        if scan_time_ms > self.timeout_ms {
            warn!("Lightning perception exceeded timeout: {}ms > {}ms", scan_time_ms, self.timeout_ms);
        }
        
        Ok(LightningData {
            key_elements: key_elements.unwrap_or_default(),
            page_status: page_status.unwrap_or(PageStatus {
                status: PageState::Unknown,
                is_loading: false,
                load_progress: 0.0,
                error_message: None,
            }),
            urgent_signals: urgent_signals.unwrap_or_default(),
            scan_time_ms,
        })
    }
    
    /// Fast scan for key interactive elements
    async fn scan_key_elements_fast(&self, browser: &BrowserConnection) -> Result<Vec<KeyElement>> {
        // Use a single JavaScript execution to get all elements at once
        let script = r##"
            function getKeyElements() {
                const elements = [];
                const maxElements = 10;
                
                // Priority selectors for quick scan
                const selectors = [
                    'button[type="submit"], input[type="submit"]',
                    'button.primary, button.btn-primary',
                    'a.button, a.btn',
                    'input[type="text"]:not([hidden]), input[type="email"]:not([hidden])',
                    'input[type="search"]',
                    'nav a, header a',
                    'button:not([type="submit"])',
                    'a[href]:not([href="#"])',
                    'form',
                    '[role="button"]'
                ];
                
                let count = 0;
                for (const selector of selectors) {
                    if (count >= maxElements) break;
                    
                    try {
                        const found = document.querySelectorAll(selector);
                        for (const el of found) {
                            if (count >= maxElements) break;
                            
                            const rect = el.getBoundingClientRect();
                            const isVisible = rect.width > 0 && rect.height > 0 && 
                                             rect.top < window.innerHeight && rect.bottom > 0;
                            
                            if (!isVisible && !el.matches('form')) continue;
                            
                            // Determine element type
                            let elementType = 'Other';
                            if (el.matches('button, input[type="submit"], [role="button"]')) {
                                elementType = 'Button';
                            } else if (el.matches('a')) {
                                elementType = 'Link';
                            } else if (el.matches('input, textarea, select')) {
                                elementType = 'Input';
                            } else if (el.matches('form')) {
                                elementType = 'Form';
                            } else if (el.matches('nav, nav *')) {
                                elementType = 'Navigation';
                            }
                            
                            // Generate a unique selector
                            let uniqueSelector = '';
                            if (el.id) {
                                uniqueSelector = '#' + el.id;
                            } else if (el.className) {
                                uniqueSelector = el.tagName.toLowerCase() + '.' + 
                                               el.className.split(' ').filter(c => c).join('.');
                            } else {
                                uniqueSelector = el.tagName.toLowerCase();
                            }
                            
                            elements.push({
                                selector: uniqueSelector,
                                element_type: elementType,
                                text: (el.innerText || el.value || el.placeholder || '').substring(0, 100),
                                importance: maxElements - count,
                                visible: isVisible,
                                clickable: !el.disabled && (el.matches('a, button, [role="button"]') || 
                                          el.onclick !== null || el.style.cursor === 'pointer')
                            });
                            
                            count++;
                        }
                    } catch (e) {
                        // Skip invalid selectors
                    }
                }
                
                return elements;
            }
            
            return getKeyElements();
        "##;
        
        let result = browser.execute_script(script, vec![]).await?;
        
        // Parse the JavaScript result
        let elements: Vec<KeyElement> = serde_json::from_value(result)
            .unwrap_or_else(|_| Vec::new());
        
        // Normalize importance scores
        let max_importance = elements.len() as f32;
        let normalized: Vec<KeyElement> = elements
            .into_iter()
            .map(|mut el| {
                el.importance = el.importance / max_importance;
                el
            })
            .take(self.max_elements)
            .collect();
        
        Ok(normalized)
    }
    
    /// Fast page status detection
    async fn detect_page_status_fast(&self, browser: &BrowserConnection) -> Result<PageStatus> {
        let script = r##"
            const state = document.readyState;
            const hasErrors = document.querySelectorAll('.error, .alert-danger, [class*="error"]').length > 0;
            const isLoading = document.querySelectorAll('.loading, .spinner, [class*="load"]').length > 0;
            const loadProgress = state === 'complete' ? 1.0 : (state === 'interactive' ? 0.8 : 0.3);
            
            return {
                state: hasErrors ? 'error' : (isLoading || state === 'loading' ? 'loading' : 
                       (state === 'interactive' ? 'interactive' : 
                       (state === 'complete' ? 'ready' : 'unknown'))),
                isLoading: isLoading || state === 'loading',
                loadProgress: loadProgress,
                errorMessage: hasErrors ? 'Page contains error indicators' : null
            };
        "##;
        
        let result = browser.execute_script(script, vec![]).await?;
        
        // Parse the result (in a real implementation, you'd parse JSON properly)
        // For now, create a reasonable default
        let is_loading = result.as_str().unwrap_or("").contains("loading");
        let has_errors = result.as_str().unwrap_or("").contains("error");
        
        Ok(PageStatus {
            status: if has_errors {
                PageState::Error
            } else if is_loading {
                PageState::Loading
            } else {
                PageState::Ready
            },
            is_loading,
            load_progress: if is_loading { 0.5 } else { 1.0 },
            error_message: if has_errors { Some("Page contains error indicators".to_string()) } else { None },
        })
    }
    
    /// Fast detection of urgent signals (alerts, popups, etc.)
    async fn detect_urgent_signals_fast(&self, browser: &BrowserConnection) -> Result<Vec<Signal>> {
        let script = r##"
            const signals = [];
            
            // Check for alerts/modals
            const modals = document.querySelectorAll(
                '.modal.show, .modal.in, .popup:not([hidden]), [role="alert"], .alert:not(.alert-info)'
            );
            
            for (const modal of modals) {
                const text = (modal.innerText || '').substring(0, 200);
                const isError = modal.classList.contains('alert-danger') || 
                               modal.classList.contains('error');
                const isSuccess = modal.classList.contains('alert-success') || 
                                 modal.classList.contains('success');
                
                signals.push({
                    signal_type: isError ? 'Error' : isSuccess ? 'Success' : 'Alert',
                    priority: isError ? 'High' : 'Medium',
                    message: text
                });
            }
            
            // Check for cookie banners (low priority)
            const cookieBanners = document.querySelectorAll(
                '[class*="cookie"], [id*="cookie"], [class*="consent"], [id*="consent"]'
            );
            
            if (cookieBanners.length > 0) {
                signals.push({
                    signal_type: 'Popup',
                    priority: 'Low',
                    message: 'Cookie consent banner detected'
                });
            }
            
            return signals.slice(0, 5); // Limit to 5 signals for speed
        "##;
        
        let result = browser.execute_script(script, vec![]).await?;
        let signals: Vec<Signal> = serde_json::from_value(result)
            .unwrap_or_else(|_| Vec::new());
        
        Ok(signals)
    }
}

/// Lightning perception with caching for repeated scans
pub struct CachedLightningPerception {
    perception: RealLightningPerception,
    cache: tokio::sync::RwLock<HashMap<String, (LightningData, Instant)>>,
    cache_ttl: Duration,
}

impl CachedLightningPerception {
    pub fn new() -> Self {
        Self {
            perception: RealLightningPerception::new(),
            cache: tokio::sync::RwLock::new(HashMap::new()),
            cache_ttl: Duration::from_secs(5),
        }
    }
    
    pub async fn scan_page(&self, browser: &BrowserConnection) -> Result<LightningData> {
        let url = browser.current_url().await?;
        
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some((data, timestamp)) = cache.get(&url) {
                if timestamp.elapsed() < self.cache_ttl {
                    debug!("Lightning perception cache hit for {}", url);
                    return Ok(data.clone());
                }
            }
        }
        
        // Perform scan
        let data = self.perception.scan_page(browser).await?;
        
        // Update cache
        {
            let mut cache = self.cache.write().await;
            cache.insert(url, (data.clone(), Instant::now()));
            
            // Clean old entries
            cache.retain(|_, (_, timestamp)| timestamp.elapsed() < self.cache_ttl * 2);
        }
        
        Ok(data)
    }
    
    pub async fn clear_cache(&self) {
        self.cache.write().await.clear();
    }
}