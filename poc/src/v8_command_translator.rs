//! V8.0 Command Translation Layer
//! 
//! Translates natural language V8.0 commands into tool-specific parameters
//! and integrates with the perception engine for timing-aware execution.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use anyhow::Result;
use tracing::{info, debug, warn};
use std::sync::Arc;
use std::time::Duration;

use crate::{
    v8_perception::{PerceptionEngine, PerceptionMode, PerceptionResult},
    browser::SimpleBrowser,
    ParsedCommand, CommandParams,
};

/// V8.0 Command Types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum V8CommandType {
    // Navigation commands
    NavigateToUrl,
    ScrollToPosition,
    ScrollToElement,
    ScrollSmooth,
    
    // Interaction commands
    Click,
    DoubleClick,
    RightClick,
    TypeText,
    SelectOption,
    
    // Advanced commands
    WaitFor,
    ExtractData,
    TakeScreenshot,
    
    // Perception commands
    AnalyzePage,
    DetectElements,
    ValidateState,
    
    // Synchronization
    WaitForElement,
    WaitForCondition,
    WaitForNavigation,
}

/// V8.0 Command Parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct V8CommandParams {
    /// Command type
    pub command_type: V8CommandType,
    
    /// Target (URL, selector, position, etc.)
    pub target: Option<String>,
    
    /// Additional value (text to type, option to select, etc.)
    pub value: Option<String>,
    
    /// Advanced options
    pub options: V8CommandOptions,
    
    /// Perception mode to use
    pub perception_mode: PerceptionMode,
}

/// V8.0 Command Options
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct V8CommandOptions {
    /// Position for scrolling (x, y)
    pub position: Option<(f64, f64)>,
    
    /// Scroll behavior
    pub smooth: bool,
    
    /// Scroll alignment
    pub alignment: Option<String>, // "start", "center", "end", "nearest"
    
    /// Click modifiers
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
    pub meta: bool,
    
    /// Mouse button
    pub button: Option<String>, // "left", "middle", "right"
    
    /// Wait strategy
    pub wait_strategy: Option<String>, // "visible", "enabled", "ready", "complete"
    
    /// Timeout in milliseconds
    pub timeout: u64,
    
    /// Take screenshot after action
    pub screenshot: bool,
    
    /// Validate result
    pub validate: bool,
}

/// V8.0 Command Result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct V8CommandResult {
    /// Whether the command was successful
    pub success: bool,
    
    /// Command that was executed
    pub command: V8CommandType,
    
    /// Perception result if applicable
    pub perception: Option<PerceptionResult>,
    
    /// Action-specific result data
    pub data: Value,
    
    /// Execution time in milliseconds
    pub execution_time_ms: u64,
    
    /// Error message if failed
    pub error: Option<String>,
}

/// V8.0 Command Translator
pub struct V8CommandTranslator {
    perception_engine: Arc<PerceptionEngine>,
}

impl V8CommandTranslator {
    /// Create a new V8 command translator
    pub fn new(perception_mode: PerceptionMode) -> Self {
        Self {
            perception_engine: Arc::new(PerceptionEngine::new(perception_mode)),
        }
    }
    
    /// Create an adaptive translator
    pub fn adaptive() -> Self {
        Self {
            perception_engine: Arc::new(PerceptionEngine::adaptive()),
        }
    }
    
    /// Parse natural language V8.0 command
    pub fn parse_v8_command(&self, command: &str) -> Result<V8CommandParams> {
        let lower = command.to_lowercase();
        
        // Analyze page commands with perception modes
        if lower.contains("analyze") || lower.contains("perception") {
            let perception_mode = if lower.contains("lightning") || lower.contains("50ms") {
                PerceptionMode::Lightning
            } else if lower.contains("quick") || lower.contains("200ms") {
                PerceptionMode::Quick
            } else if lower.contains("deep") || lower.contains("1000ms") || lower.contains("semantic") {
                PerceptionMode::Deep
            } else {
                PerceptionMode::Standard
            };
            
            return Ok(V8CommandParams {
                command_type: V8CommandType::AnalyzePage,
                target: None,
                value: None,
                options: V8CommandOptions::default(),
                perception_mode,
            });
        }
        
        // Scroll commands with V8.0 parameters
        if lower.contains("scroll") {
            if lower.contains("position") || lower.contains("coordinates") {
                // Extract position from command (simplified)
                let position = if lower.contains("500, 1000") {
                    Some((500.0, 1000.0))
                } else if lower.contains("top") {
                    Some((0.0, 0.0))
                } else if lower.contains("bottom") {
                    Some((0.0, 10000.0))
                } else {
                    Some((0.0, 500.0)) // Default scroll
                };
                
                return Ok(V8CommandParams {
                    command_type: V8CommandType::ScrollToPosition,
                    target: None,
                    value: None,
                    options: V8CommandOptions {
                        position,
                        smooth: lower.contains("smooth"),
                        ..Default::default()
                    },
                    perception_mode: PerceptionMode::Lightning,
                });
            }
            
            if lower.contains("element") || lower.contains("#") || lower.contains(".") {
                // Extract selector (simplified)
                let selector = self.extract_selector(&lower);
                let alignment = if lower.contains("center") {
                    Some("center".to_string())
                } else if lower.contains("start") || lower.contains("top") {
                    Some("start".to_string())
                } else if lower.contains("end") || lower.contains("bottom") {
                    Some("end".to_string())
                } else {
                    Some("nearest".to_string())
                };
                
                return Ok(V8CommandParams {
                    command_type: V8CommandType::ScrollToElement,
                    target: selector,
                    value: None,
                    options: V8CommandOptions {
                        alignment,
                        smooth: lower.contains("smooth"),
                        ..Default::default()
                    },
                    perception_mode: PerceptionMode::Quick,
                });
            }
        }
        
        // Click commands with modifiers
        if lower.contains("click") {
            let command_type = if lower.contains("double") {
                V8CommandType::DoubleClick
            } else if lower.contains("right") || lower.contains("context") {
                V8CommandType::RightClick
            } else {
                V8CommandType::Click
            };
            
            let selector = self.extract_selector(&lower);
            
            return Ok(V8CommandParams {
                command_type,
                target: selector,
                value: None,
                options: V8CommandOptions {
                    ctrl: lower.contains("ctrl") || lower.contains("control"),
                    shift: lower.contains("shift"),
                    alt: lower.contains("alt"),
                    meta: lower.contains("meta") || lower.contains("cmd"),
                    button: if lower.contains("middle") {
                        Some("middle".to_string())
                    } else if lower.contains("right") {
                        Some("right".to_string())
                    } else {
                        Some("left".to_string())
                    },
                    ..Default::default()
                },
                perception_mode: PerceptionMode::Quick,
            });
        }
        
        // Type text commands
        if lower.contains("type") || lower.contains("input") || lower.contains("enter text") {
            let selector = self.extract_selector(&lower);
            let text = self.extract_text(&lower);
            
            return Ok(V8CommandParams {
                command_type: V8CommandType::TypeText,
                target: selector,
                value: text,
                options: V8CommandOptions {
                    validate: lower.contains("validate"),
                    ..Default::default()
                },
                perception_mode: PerceptionMode::Quick,
            });
        }
        
        // Select option commands
        if lower.contains("select") || lower.contains("dropdown") || lower.contains("choose") {
            let selector = self.extract_selector(&lower);
            let value = self.extract_text(&lower);
            
            return Ok(V8CommandParams {
                command_type: V8CommandType::SelectOption,
                target: selector,
                value,
                options: V8CommandOptions::default(),
                perception_mode: PerceptionMode::Quick,
            });
        }
        
        // Wait commands
        if lower.contains("wait") {
            if lower.contains("element") {
                let selector = self.extract_selector(&lower);
                let wait_strategy = if lower.contains("visible") {
                    Some("visible".to_string())
                } else if lower.contains("enabled") {
                    Some("enabled".to_string())
                } else if lower.contains("ready") {
                    Some("ready".to_string())
                } else {
                    Some("complete".to_string())
                };
                
                return Ok(V8CommandParams {
                    command_type: V8CommandType::WaitForElement,
                    target: selector,
                    value: None,
                    options: V8CommandOptions {
                        wait_strategy,
                        timeout: self.extract_timeout(&lower),
                        ..Default::default()
                    },
                    perception_mode: PerceptionMode::Standard,
                });
            }
        }
        
        // Extract data commands
        if lower.contains("extract") || lower.contains("scrape") || lower.contains("get data") {
            let selector = self.extract_selector(&lower);
            
            return Ok(V8CommandParams {
                command_type: V8CommandType::ExtractData,
                target: selector,
                value: None,
                options: V8CommandOptions {
                    validate: true,
                    ..Default::default()
                },
                perception_mode: PerceptionMode::Deep,
            });
        }
        
        // Navigation commands
        if lower.contains("navigate") || lower.contains("go to") || lower.contains("open") {
            let url = self.extract_url(&lower);
            
            return Ok(V8CommandParams {
                command_type: V8CommandType::NavigateToUrl,
                target: url,
                value: None,
                options: V8CommandOptions {
                    screenshot: lower.contains("screenshot"),
                    wait_strategy: Some("complete".to_string()),
                    ..Default::default()
                },
                perception_mode: PerceptionMode::Standard,
            });
        }
        
        // Screenshot commands
        if lower.contains("screenshot") || lower.contains("capture") {
            return Ok(V8CommandParams {
                command_type: V8CommandType::TakeScreenshot,
                target: None,
                value: None,
                options: V8CommandOptions {
                    screenshot: true,
                    ..Default::default()
                },
                perception_mode: PerceptionMode::Lightning,
            });
        }
        
        // Default to navigation if URL-like
        if lower.contains("http") || lower.contains(".com") || lower.contains(".org") {
            return Ok(V8CommandParams {
                command_type: V8CommandType::NavigateToUrl,
                target: self.extract_url(&lower),
                value: None,
                options: V8CommandOptions::default(),
                perception_mode: PerceptionMode::Standard,
            });
        }
        
        // Default to element detection
        Ok(V8CommandParams {
            command_type: V8CommandType::DetectElements,
            target: None,
            value: None,
            options: V8CommandOptions::default(),
            perception_mode: PerceptionMode::Standard,
        })
    }
    
    /// Execute V8 command with browser
    pub async fn execute_v8_command(
        &self,
        browser: &SimpleBrowser,
        params: &V8CommandParams,
    ) -> Result<V8CommandResult> {
        let start = std::time::Instant::now();
        
        // First, run perception if needed
        let perception = if matches!(params.command_type, 
            V8CommandType::AnalyzePage | 
            V8CommandType::DetectElements | 
            V8CommandType::ValidateState) {
            Some(self.perception_engine.perceive(browser).await?)
        } else {
            None
        };
        
        // Execute the actual command
        let result = match params.command_type {
            V8CommandType::NavigateToUrl => {
                if let Some(url) = &params.target {
                    browser.navigate_to(url).await?;
                    json!({
                        "navigated": true,
                        "url": url,
                        "title": browser.get_title().await.ok()
                    })
                } else {
                    return Err(anyhow::anyhow!("No URL specified for navigation"));
                }
            },
            
            V8CommandType::ScrollToPosition => {
                if let Some((x, y)) = params.options.position {
                    browser.execute_script(
                        &format!("window.scrollTo({}, {})", x, y),
                        vec![]
                    ).await?;
                    json!({
                        "scrolled": true,
                        "position": { "x": x, "y": y }
                    })
                } else {
                    return Err(anyhow::anyhow!("No position specified for scroll"));
                }
            },
            
            V8CommandType::ScrollToElement => {
                if let Some(selector) = &params.target {
                    let alignment = params.options.alignment.as_deref().unwrap_or("center");
                    browser.execute_script(
                        &format!(
                            "document.querySelector('{}').scrollIntoView({{ behavior: '{}', block: '{}' }})",
                            selector,
                            if params.options.smooth { "smooth" } else { "auto" },
                            alignment
                        ),
                        vec![]
                    ).await?;
                    json!({
                        "scrolled": true,
                        "element": selector,
                        "alignment": alignment
                    })
                } else {
                    return Err(anyhow::anyhow!("No element selector specified for scroll"));
                }
            },
            
            V8CommandType::Click | V8CommandType::DoubleClick | V8CommandType::RightClick => {
                if let Some(selector) = &params.target {
                    let element = browser.find_element(selector).await?;
                    
                    // Apply modifiers if needed
                    if params.options.ctrl || params.options.shift || 
                       params.options.alt || params.options.meta {
                        // Execute click with modifiers using JavaScript
                        let event_type = match params.command_type {
                            V8CommandType::DoubleClick => "dblclick",
                            V8CommandType::RightClick => "contextmenu",
                            _ => "click",
                        };
                        
                        browser.execute_script(
                            &format!(
                                r#"
                                const el = document.querySelector('{}');
                                const event = new MouseEvent('{}', {{
                                    bubbles: true,
                                    cancelable: true,
                                    ctrlKey: {},
                                    shiftKey: {},
                                    altKey: {},
                                    metaKey: {},
                                    button: {}
                                }});
                                el.dispatchEvent(event);
                                "#,
                                selector, event_type,
                                params.options.ctrl,
                                params.options.shift,
                                params.options.alt,
                                params.options.meta,
                                match params.options.button.as_deref() {
                                    Some("middle") => 1,
                                    Some("right") => 2,
                                    _ => 0,
                                }
                            ),
                            vec![]
                        ).await?;
                    } else {
                        // Simple click
                        element.click().await?;
                    }
                    
                    json!({
                        "clicked": true,
                        "element": selector,
                        "type": format!("{:?}", params.command_type)
                    })
                } else {
                    return Err(anyhow::anyhow!("No element selector specified for click"));
                }
            },
            
            V8CommandType::TypeText => {
                if let Some(selector) = &params.target {
                    let element = browser.find_element(selector).await?;
                    if let Some(text) = &params.value {
                        element.send_keys(text).await?;
                        json!({
                            "typed": true,
                            "element": selector,
                            "text": text
                        })
                    } else {
                        return Err(anyhow::anyhow!("No text specified to type"));
                    }
                } else {
                    return Err(anyhow::anyhow!("No element selector specified for typing"));
                }
            },
            
            V8CommandType::SelectOption => {
                if let Some(selector) = &params.target {
                    if let Some(value) = &params.value {
                        browser.execute_script(
                            &format!(
                                "document.querySelector('{}').value = '{}'",
                                selector, value
                            ),
                            vec![]
                        ).await?;
                        json!({
                            "selected": true,
                            "element": selector,
                            "value": value
                        })
                    } else {
                        return Err(anyhow::anyhow!("No value specified to select"));
                    }
                } else {
                    return Err(anyhow::anyhow!("No element selector specified for select"));
                }
            },
            
            V8CommandType::WaitForElement => {
                if let Some(selector) = &params.target {
                    // Simple wait implementation
                    let timeout = Duration::from_millis(params.options.timeout);
                    let start = std::time::Instant::now();
                    
                    while start.elapsed() < timeout {
                        if browser.find_element(selector).await.is_ok() {
                            break;
                        }
                        tokio::time::sleep(Duration::from_millis(100)).await;
                    }
                    
                    json!({
                        "waited": true,
                        "element": selector,
                        "found": browser.find_element(selector).await.is_ok()
                    })
                } else {
                    return Err(anyhow::anyhow!("No element selector specified for wait"));
                }
            },
            
            V8CommandType::AnalyzePage | V8CommandType::DetectElements => {
                // Perception already done above
                if let Some(ref p) = perception {
                    json!({
                        "analyzed": true,
                        "mode": format!("{:?}", p.mode),
                        "elements_found": p.elements_found,
                        "page_status": format!("{:?}", p.page_status),
                        "confidence": p.confidence,
                        "execution_time_ms": p.execution_time.as_millis()
                    })
                } else {
                    json!({
                        "analyzed": false,
                        "error": "Perception failed"
                    })
                }
            },
            
            V8CommandType::TakeScreenshot => {
                let filename = format!("v8_screenshot_{}.png", chrono::Utc::now().timestamp());
                browser.take_screenshot(&filename).await?;
                json!({
                    "screenshot": true,
                    "filename": filename,
                    "path": format!("screenshots/{}", filename)
                })
            },
            
            _ => {
                json!({
                    "executed": false,
                    "command": format!("{:?}", params.command_type),
                    "message": "Command not yet implemented"
                })
            }
        };
        
        // Take screenshot if requested
        if params.options.screenshot && !matches!(params.command_type, V8CommandType::TakeScreenshot) {
            let filename = format!("v8_action_{}.png", chrono::Utc::now().timestamp());
            let _ = browser.take_screenshot(&filename).await;
        }
        
        let execution_time = start.elapsed();
        
        Ok(V8CommandResult {
            success: true,
            command: params.command_type.clone(),
            perception,
            data: result,
            execution_time_ms: execution_time.as_millis() as u64,
            error: None,
        })
    }
    
    // Helper methods
    
    fn extract_selector(&self, command: &str) -> Option<String> {
        // Simple selector extraction (can be improved)
        if command.contains("#") {
            // Extract ID selector
            let parts: Vec<&str> = command.split_whitespace().collect();
            for part in parts {
                if part.starts_with('#') {
                    return Some(part.to_string());
                }
            }
        }
        
        if command.contains("class=") {
            // Extract class selector
            if let Some(start) = command.find("class=") {
                let after_class = &command[start + 6..];
                if let Some(end) = after_class.find(|c: char| c.is_whitespace() || c == '"') {
                    return Some(format!(".{}", &after_class[..end]));
                }
            }
        }
        
        // Look for common element references
        if command.contains("button") {
            return Some("button".to_string());
        }
        if command.contains("input") {
            return Some("input".to_string());
        }
        if command.contains("link") {
            return Some("a".to_string());
        }
        
        None
    }
    
    fn extract_text(&self, command: &str) -> Option<String> {
        // Extract text between quotes
        if let Some(start) = command.find('"') {
            if let Some(end) = command[start + 1..].find('"') {
                return Some(command[start + 1..start + 1 + end].to_string());
            }
        }
        
        if let Some(start) = command.find('\'') {
            if let Some(end) = command[start + 1..].find('\'') {
                return Some(command[start + 1..start + 1 + end].to_string());
            }
        }
        
        // For type commands, extract after "type"
        if command.contains("type ") {
            if let Some(start) = command.find("type ") {
                let after_type = &command[start + 5..];
                return Some(after_type.trim().to_string());
            }
        }
        
        None
    }
    
    fn extract_url(&self, command: &str) -> Option<String> {
        // Extract URL
        if let Some(start) = command.find("http://") {
            let url_part = &command[start..];
            if let Some(end) = url_part.find(|c: char| c.is_whitespace()) {
                return Some(url_part[..end].to_string());
            }
            return Some(url_part.to_string());
        }
        
        if let Some(start) = command.find("https://") {
            let url_part = &command[start..];
            if let Some(end) = url_part.find(|c: char| c.is_whitespace()) {
                return Some(url_part[..end].to_string());
            }
            return Some(url_part.to_string());
        }
        
        // Look for common domains
        for domain in &[".com", ".org", ".net", ".io"] {
            if command.contains(domain) {
                // Try to extract domain
                let parts: Vec<&str> = command.split_whitespace().collect();
                for part in parts {
                    if part.contains(domain) {
                        // Add https:// if not present
                        if !part.starts_with("http") {
                            return Some(format!("https://{}", part));
                        }
                        return Some(part.to_string());
                    }
                }
            }
        }
        
        None
    }
    
    fn extract_timeout(&self, command: &str) -> u64 {
        // Extract timeout value
        if let Some(pos) = command.find("timeout") {
            let after_timeout = &command[pos + 7..];
            // Look for number
            let parts: Vec<&str> = after_timeout.split_whitespace().collect();
            if let Some(first) = parts.first() {
                if let Ok(ms) = first.trim_end_matches("ms").parse::<u64>() {
                    return ms;
                }
                if let Ok(s) = first.trim_end_matches('s').parse::<u64>() {
                    return s * 1000;
                }
            }
        }
        
        // Default timeout
        30000
    }
}

/// Convert V8 command to legacy ParsedCommand for compatibility
pub fn v8_to_parsed_command(params: &V8CommandParams) -> ParsedCommand {
    let action = match params.command_type {
        V8CommandType::NavigateToUrl => "navigate",
        V8CommandType::Click | V8CommandType::DoubleClick | V8CommandType::RightClick => "click",
        V8CommandType::ScrollToPosition | V8CommandType::ScrollToElement => "scroll",
        V8CommandType::TypeText => "input",
        V8CommandType::SelectOption => "select",
        V8CommandType::TakeScreenshot => "screenshot",
        V8CommandType::WaitForElement => "wait",
        _ => "unknown",
    }.to_string();
    
    // Determine scroll direction if applicable
    let scroll_direction = if matches!(params.command_type, V8CommandType::ScrollToPosition | V8CommandType::ScrollToElement) {
        if let Some((_, y)) = params.options.position {
            if y > 0.0 {
                Some("down".to_string())
            } else {
                Some("up".to_string())
            }
        } else {
            Some("down".to_string())
        }
    } else {
        None
    };
    
    ParsedCommand {
        action,
        url: params.target.clone(),
        urls: params.target.clone().map(|u| vec![u]).unwrap_or_default(),
        screenshot: params.options.screenshot,
        filename: None,
        viewport_width: None,
        viewport_height: None,
        viewport_only: false,
        retries: None,
        timeout: Some(params.options.timeout),
        confidence: 0.9,
        parameters: CommandParams {
            take_screenshot: params.options.screenshot,
            screenshot_filename: None,
            viewport_width: None,
            viewport_height: None,
            retries: None,
            timeout_seconds: Some(params.options.timeout / 1000),
            show_report: false,
        },
        scroll_direction,
        element_selector: params.target.clone(),
        input_text: params.value.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_perception_commands() {
        let translator = V8CommandTranslator::adaptive();
        
        // Test lightning perception
        let cmd = translator.parse_v8_command("analyze page with lightning speed").unwrap();
        assert_eq!(cmd.perception_mode, PerceptionMode::Lightning);
        assert!(matches!(cmd.command_type, V8CommandType::AnalyzePage));
        
        // Test deep perception
        let cmd = translator.parse_v8_command("perform deep semantic analysis").unwrap();
        assert_eq!(cmd.perception_mode, PerceptionMode::Deep);
    }
    
    #[test]
    fn test_parse_scroll_commands() {
        let translator = V8CommandTranslator::adaptive();
        
        // Test scroll to position
        let cmd = translator.parse_v8_command("scroll to position 500, 1000 smoothly").unwrap();
        assert!(matches!(cmd.command_type, V8CommandType::ScrollToPosition));
        assert_eq!(cmd.options.position, Some((500.0, 1000.0)));
        assert!(cmd.options.smooth);
        
        // Test scroll to element
        let cmd = translator.parse_v8_command("scroll to element #header center").unwrap();
        assert!(matches!(cmd.command_type, V8CommandType::ScrollToElement));
        assert_eq!(cmd.target, Some("#header".to_string()));
        assert_eq!(cmd.options.alignment, Some("center".to_string()));
    }
    
    #[test]
    fn test_parse_click_commands() {
        let translator = V8CommandTranslator::adaptive();
        
        // Test double click with modifier
        let cmd = translator.parse_v8_command("double click button with ctrl").unwrap();
        assert!(matches!(cmd.command_type, V8CommandType::DoubleClick));
        assert!(cmd.options.ctrl);
        
        // Test right click
        let cmd = translator.parse_v8_command("right click on #menu").unwrap();
        assert!(matches!(cmd.command_type, V8CommandType::RightClick));
        assert_eq!(cmd.target, Some("#menu".to_string()));
    }
}