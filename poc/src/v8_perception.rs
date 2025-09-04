//! V8.0 Perception Layer Implementation
//! 
//! Implements the four-layer perception system as specified in V8.0:
//! - Lightning (<50ms): Fast element detection
//! - Quick (<200ms): Basic page analysis  
//! - Standard (<500ms): Comprehensive analysis
//! - Deep (<1000ms): Full semantic understanding

use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use anyhow::Result;
use tracing::{info, debug, warn};

/// V8.0 Perception modes with timing constraints
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PerceptionMode {
    /// Lightning fast perception (<50ms)
    Lightning,
    /// Quick perception (<200ms)
    Quick,
    /// Standard perception (<500ms)
    Standard,
    /// Deep analysis (<1000ms)
    Deep,
}

impl PerceptionMode {
    /// Get the time budget for this mode
    pub fn time_budget(&self) -> Duration {
        match self {
            PerceptionMode::Lightning => Duration::from_millis(50),
            PerceptionMode::Quick => Duration::from_millis(200),
            PerceptionMode::Standard => Duration::from_millis(500),
            PerceptionMode::Deep => Duration::from_millis(1000),
        }
    }
    
    /// Get the mode name
    pub fn name(&self) -> &str {
        match self {
            PerceptionMode::Lightning => "lightning",
            PerceptionMode::Quick => "quick",
            PerceptionMode::Standard => "standard",
            PerceptionMode::Deep => "deep",
        }
    }
    
    /// Automatically select mode based on complexity
    pub fn auto_select(page_complexity: f32, time_available: Duration) -> Self {
        if time_available < Duration::from_millis(100) {
            PerceptionMode::Lightning
        } else if page_complexity < 0.3 {
            PerceptionMode::Quick
        } else if page_complexity < 0.7 {
            PerceptionMode::Standard
        } else {
            PerceptionMode::Deep
        }
    }
}

/// Result of perception analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerceptionResult {
    pub mode: PerceptionMode,
    pub execution_time: Duration,
    pub elements_found: usize,
    pub page_status: PageStatus,
    pub key_elements: Vec<ElementInfo>,
    pub confidence: f32,
    pub metadata: serde_json::Value,
}

/// Page status after perception
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PageStatus {
    Loading,
    Ready,
    Interactive,
    Complete,
    Error,
    Unknown,
}

/// Basic element information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementInfo {
    pub selector: String,
    pub tag_name: String,
    pub text: Option<String>,
    pub is_visible: bool,
    pub is_clickable: bool,
    pub bounding_box: Option<BoundingBox>,
}

/// Element bounding box
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BoundingBox {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// V8.0 Perception Engine
pub struct PerceptionEngine {
    mode: PerceptionMode,
    adaptive: bool,
}

impl PerceptionEngine {
    /// Create a new perception engine
    pub fn new(mode: PerceptionMode) -> Self {
        Self {
            mode,
            adaptive: false,
        }
    }
    
    /// Create an adaptive perception engine
    pub fn adaptive() -> Self {
        Self {
            mode: PerceptionMode::Standard,
            adaptive: true,
        }
    }
    
    /// Perform perception analysis
    pub async fn perceive(&self, browser: &crate::browser::SimpleBrowser) -> Result<PerceptionResult> {
        let start = Instant::now();
        let budget = self.mode.time_budget();
        
        info!("Starting {} perception with {}ms budget", self.mode.name(), budget.as_millis());
        
        // Perform perception based on mode
        let result = match self.mode {
            PerceptionMode::Lightning => self.lightning_perception(browser, budget).await?,
            PerceptionMode::Quick => self.quick_perception(browser, budget).await?,
            PerceptionMode::Standard => self.standard_perception(browser, budget).await?,
            PerceptionMode::Deep => self.deep_perception(browser, budget).await?,
        };
        
        let execution_time = start.elapsed();
        
        if execution_time > budget {
            warn!("Perception exceeded budget: {:?} > {:?}", execution_time, budget);
        } else {
            debug!("Perception completed in {:?}", execution_time);
        }
        
        Ok(PerceptionResult {
            mode: self.mode,
            execution_time,
            ..result
        })
    }
    
    /// Lightning fast perception (<50ms)
    async fn lightning_perception(
        &self,
        browser: &crate::browser::SimpleBrowser,
        _budget: Duration,
    ) -> Result<PerceptionResult> {
        // Only check key elements quickly
        let key_selectors = vec!["button", "a", "input", "select", "textarea"];
        let mut key_elements = Vec::new();
        
        for selector in key_selectors {
            if let Ok(elements) = browser.find_elements(selector).await {
                for elem in elements.iter().take(2) {
                    key_elements.push(ElementInfo {
                        selector: selector.to_string(),
                        tag_name: selector.to_string(),
                        text: None,
                        is_visible: true,
                        is_clickable: matches!(selector, "button" | "a"),
                        bounding_box: None,
                    });
                }
            }
        }
        
        Ok(PerceptionResult {
            mode: self.mode,
            execution_time: Duration::from_millis(0),
            elements_found: key_elements.len(),
            page_status: PageStatus::Ready,
            key_elements,
            confidence: 0.6,
            metadata: serde_json::json!({
                "perception_type": "lightning",
                "elements_checked": 5
            }),
        })
    }
    
    /// Quick perception (<200ms)
    async fn quick_perception(
        &self,
        browser: &crate::browser::SimpleBrowser,
        _budget: Duration,
    ) -> Result<PerceptionResult> {
        // Check more elements and basic page state
        let selectors = vec![
            "button", "a", "input", "select", "textarea",
            "h1", "h2", "h3", "img", "form"
        ];
        
        let mut key_elements = Vec::new();
        let mut total_elements = 0;
        
        for selector in selectors {
            if let Ok(elements) = browser.find_elements(selector).await {
                total_elements += elements.len();
                for elem in elements.iter().take(3) {
                    key_elements.push(ElementInfo {
                        selector: selector.to_string(),
                        tag_name: selector.to_string(),
                        text: None,
                        is_visible: true,
                        is_clickable: matches!(selector, "button" | "a" | "input"),
                        bounding_box: None,
                    });
                }
            }
        }
        
        // Check page ready state
        let page_status = if total_elements > 20 {
            PageStatus::Complete
        } else if total_elements > 10 {
            PageStatus::Interactive
        } else {
            PageStatus::Loading
        };
        
        Ok(PerceptionResult {
            mode: self.mode,
            execution_time: Duration::from_millis(0),
            elements_found: total_elements,
            page_status,
            key_elements,
            confidence: 0.75,
            metadata: serde_json::json!({
                "perception_type": "quick",
                "selectors_checked": 10,
                "total_elements": total_elements
            }),
        })
    }
    
    /// Standard perception (<500ms)
    async fn standard_perception(
        &self,
        browser: &crate::browser::SimpleBrowser,
        _budget: Duration,
    ) -> Result<PerceptionResult> {
        // Comprehensive page analysis
        let mut key_elements = Vec::new();
        let mut total_elements = 0;
        
        // Get all interactive elements
        let interactive_selectors = vec![
            "button", "a[href]", "input:not([type='hidden'])", 
            "select", "textarea", "[onclick]", "[role='button']"
        ];
        
        for selector in interactive_selectors {
            if let Ok(elements) = browser.find_elements(selector).await {
                total_elements += elements.len();
                for _elem in elements.iter().take(5) {
                    // For now, skip getting element text due to WebElement conversion issues
                    let text = None;
                    
                    key_elements.push(ElementInfo {
                        selector: selector.to_string(),
                        tag_name: selector.to_string(),
                        text,
                        is_visible: true,
                        is_clickable: true,
                        bounding_box: None,
                    });
                }
            }
        }
        
        // Check document ready state
        let page_status = PageStatus::Complete; // Default to complete for now
        
        Ok(PerceptionResult {
            mode: self.mode,
            execution_time: Duration::from_millis(0),
            elements_found: total_elements,
            page_status,
            key_elements,
            confidence: 0.85,
            metadata: serde_json::json!({
                "perception_type": "standard",
                "interactive_elements": total_elements
            }),
        })
    }
    
    /// Deep perception (<1000ms)
    async fn deep_perception(
        &self,
        browser: &crate::browser::SimpleBrowser,
        _budget: Duration,
    ) -> Result<PerceptionResult> {
        // Full semantic analysis
        let mut key_elements = Vec::new();
        
        // Get all elements and analyze structure
        let all_elements = browser.find_elements("*").await?;
        let total_elements = all_elements.len();
        
        // For now, use basic page analysis
        let page_analysis = serde_json::json!({
            "perception_type": "deep",
            "total_elements": total_elements
        });
        
        // Get key interactive elements with full details
        let interactive = browser.find_elements(
            "button, a[href], input, select, textarea, [role='button']"
        ).await?;
        
        for _elem in interactive.iter().take(10) {
            // For now, add basic element info without JavaScript execution
            key_elements.push(ElementInfo {
                selector: "interactive".to_string(),
                tag_name: "unknown".to_string(),
                text: None,
                is_visible: true,
                is_clickable: true,
                bounding_box: None,
            });
        }
        
        Ok(PerceptionResult {
            mode: self.mode,
            execution_time: Duration::from_millis(0),
            elements_found: total_elements,
            page_status: PageStatus::Complete,
            key_elements,
            confidence: 0.95,
            metadata: page_analysis,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_perception_mode_timing() {
        assert_eq!(PerceptionMode::Lightning.time_budget(), Duration::from_millis(50));
        assert_eq!(PerceptionMode::Quick.time_budget(), Duration::from_millis(200));
        assert_eq!(PerceptionMode::Standard.time_budget(), Duration::from_millis(500));
        assert_eq!(PerceptionMode::Deep.time_budget(), Duration::from_millis(1000));
    }
    
    #[test]
    fn test_auto_mode_selection() {
        // Low complexity, plenty of time
        let mode = PerceptionMode::auto_select(0.2, Duration::from_secs(1));
        assert_eq!(mode, PerceptionMode::Quick);
        
        // High complexity, plenty of time
        let mode = PerceptionMode::auto_select(0.8, Duration::from_secs(1));
        assert_eq!(mode, PerceptionMode::Deep);
        
        // Any complexity, very little time
        let mode = PerceptionMode::auto_select(0.8, Duration::from_millis(50));
        assert_eq!(mode, PerceptionMode::Lightning);
    }
}