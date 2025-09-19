// Integration layer - connects the perception module to the chromiumoxide API
// This provides high-level intelligent automation capabilities

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::info;

use crate::browser::Browser;
use crate::perception::{ElementType, PageType, PerceivedElement, PerceptionEngine};

/// Enhanced browser automation with perception capabilities
pub struct PerceptionAwareBrowser {
    pub perception: PerceptionEngine,
    pub browser: Arc<Browser>,
}

/// Command structure for perception-aware operations
#[derive(Debug, Serialize, Deserialize)]
pub struct IntelligentCommand {
    pub action: String,
    pub target_description: Option<String>, // Natural language description
    pub input_text: Option<String>,
    pub options: CommandOptions,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommandOptions {
    pub wait_for_element: Option<u64>, // milliseconds
    pub take_screenshot: bool,
    pub extract_data: bool,
    pub confidence_threshold: Option<f32>,
}

impl Default for CommandOptions {
    fn default() -> Self {
        Self {
            wait_for_element: Some(5000),
            take_screenshot: false,
            extract_data: false,
            confidence_threshold: Some(0.7),
        }
    }
}

/// Result of executing an intelligent command
#[derive(Debug, Serialize)]
pub struct IntelligentCommandResult {
    pub success: bool,
    pub action: String,
    pub message: String,
    pub element_info: Option<PerceivedElement>,
    pub screenshot: Option<String>, // base64 encoded
    pub extracted_data: Option<serde_json::Value>,
    pub page_type: Option<PageType>,
    pub confidence: f32,
}

impl PerceptionAwareBrowser {
    /// Create a new perception-aware browser
    pub async fn new(browser: Arc<Browser>) -> Result<Self> {
        let perception = PerceptionEngine::new(browser.clone()).await?;
        Ok(Self {
            perception,
            browser,
        })
    }

    /// Execute an intelligent command using natural language
    pub async fn execute_intelligent_command(
        &mut self,
        command: IntelligentCommand,
    ) -> Result<IntelligentCommandResult> {
        info!("Executing intelligent command: {:?}", command.action);

        // Classify the current page first
        let page_type = self.perception.classify_page().await.ok();

        // Take screenshot if requested
        let screenshot = if command.options.take_screenshot {
            let screenshot_data = self
                .browser
                .screenshot(crate::browser::ScreenshotOptions::default())
                .await?;
            Some(base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                &screenshot_data,
            ))
        } else {
            None
        };

        // Execute the specific command
        let action = command.action.clone();
        let result = match action.as_str() {
            "click" => self.intelligent_click(command).await,
            "type" | "input" => self.intelligent_type(command).await,
            "select" => self.intelligent_select(command).await,
            "extract" => self.intelligent_extract(command).await,
            "search" => self.intelligent_search(command).await,
            "navigate" => self.intelligent_navigate(command).await,
            "wait" => self.intelligent_wait(command).await,
            _ => Err(anyhow::anyhow!("Unknown intelligent command: {}", action)),
        };

        // Build the response
        match result {
            Ok(mut cmd_result) => {
                cmd_result.screenshot = screenshot;
                cmd_result.page_type = page_type;
                Ok(cmd_result)
            }
            Err(e) => Ok(IntelligentCommandResult {
                success: false,
                action,
                message: format!("Command failed: {}", e),
                element_info: None,
                screenshot,
                extracted_data: None,
                page_type,
                confidence: 0.0,
            }),
        }
    }

    /// Intelligent click that finds elements by description
    async fn intelligent_click(
        &mut self,
        command: IntelligentCommand,
    ) -> Result<IntelligentCommandResult> {
        let description = command
            .target_description
            .ok_or_else(|| anyhow::anyhow!("No target description provided for click"))?;

        // Find the element using perception
        let element = self.perception.find_element(&description).await?;

        // Check confidence threshold
        if let Some(threshold) = command.options.confidence_threshold {
            if element.confidence < threshold {
                return Err(anyhow::anyhow!(
                    "Element confidence ({:.2}) below threshold ({:.2})",
                    element.confidence,
                    threshold
                ));
            }
        }

        // Perform the click
        self.browser.click(&element.selector).await?;

        // Update context
        self.perception
            .update_context("click", Some(&element.selector));

        // Clone values we need before moving element
        let element_text = element.text.clone();
        let element_confidence = element.confidence;

        Ok(IntelligentCommandResult {
            success: true,
            action: "click".to_string(),
            message: format!("Successfully clicked: {}", element_text),
            element_info: Some(element),
            screenshot: None,
            extracted_data: None,
            page_type: None,
            confidence: element_confidence,
        })
    }

    /// Intelligent text input with smart field detection
    async fn intelligent_type(
        &mut self,
        command: IntelligentCommand,
    ) -> Result<IntelligentCommandResult> {
        let text = command
            .input_text
            .ok_or_else(|| anyhow::anyhow!("No input text provided"))?;

        let description = command
            .target_description
            .unwrap_or_else(|| "input field".to_string());

        // Find the input element
        let element = self.perception.find_element(&description).await?;

        // Verify it's an input element
        if !matches!(
            element.element_type,
            ElementType::Input | ElementType::TextArea
        ) {
            return Err(anyhow::anyhow!(
                "Target element is not an input field: {:?}",
                element.element_type
            ));
        }

        // Clear and type
        self.browser.click(&element.selector).await?; // Focus first

        // Clear existing content (Ctrl+A, Delete)
        let clear_script = format!(
            r#"
            const element = document.querySelector('{}');
            if (element) {{
                element.select();
                element.value = '';
                element.dispatchEvent(new Event('input', {{ bubbles: true }}));
            }}
        "#,
            element.selector
        );
        self.browser.execute_script(&clear_script).await?;

        // Type the new text
        self.browser.type_text(&element.selector, &text).await?;

        // Update context
        self.perception
            .update_context("type", Some(&element.selector));

        // Clone values we need before moving element
        let element_text = element.text.clone();
        let element_confidence = element.confidence;

        Ok(IntelligentCommandResult {
            success: true,
            action: "type".to_string(),
            message: format!("Typed '{}' into {}", text, element_text),
            element_info: Some(element),
            screenshot: None,
            extracted_data: None,
            page_type: None,
            confidence: element_confidence,
        })
    }

    /// Intelligent dropdown selection
    async fn intelligent_select(
        &mut self,
        command: IntelligentCommand,
    ) -> Result<IntelligentCommandResult> {
        let value = command
            .input_text
            .ok_or_else(|| anyhow::anyhow!("No option value provided"))?;

        let description = command
            .target_description
            .unwrap_or_else(|| "dropdown".to_string());

        // Find the select element
        let element = self.perception.find_element(&description).await?;

        // Verify it's a select element
        if element.element_type != ElementType::Select {
            return Err(anyhow::anyhow!(
                "Target element is not a dropdown: {:?}",
                element.element_type
            ));
        }

        // Select the option
        self.browser
            .select_option(&element.selector, &value)
            .await?;

        // Update context
        self.perception
            .update_context("select", Some(&element.selector));

        // Clone values we need before moving element
        let element_confidence = element.confidence;

        Ok(IntelligentCommandResult {
            success: true,
            action: "select".to_string(),
            message: format!("Selected '{}' in dropdown", value),
            element_info: Some(element),
            screenshot: None,
            extracted_data: None,
            page_type: None,
            confidence: element_confidence,
        })
    }

    /// Intelligent data extraction based on page type
    async fn intelligent_extract(
        &mut self,
        _command: IntelligentCommand,
    ) -> Result<IntelligentCommandResult> {
        // Extract data based on page classification
        let extracted_data = self.perception.extract_page_data().await?;

        Ok(IntelligentCommandResult {
            success: true,
            action: "extract".to_string(),
            message: "Successfully extracted page data".to_string(),
            element_info: None,
            screenshot: None,
            extracted_data: Some(extracted_data),
            page_type: None,
            confidence: 1.0,
        })
    }

    /// Intelligent search that finds and uses search functionality
    async fn intelligent_search(
        &mut self,
        command: IntelligentCommand,
    ) -> Result<IntelligentCommandResult> {
        let query = command
            .input_text
            .ok_or_else(|| anyhow::anyhow!("No search query provided"))?;

        // Try to find search box on current page
        match self.perception.find_element("search box").await {
            Ok(search_box) => {
                // Found search box, use it
                self.browser.click(&search_box.selector).await?;

                // Clear existing content
                let clear_script = format!(
                    r#"
                    const element = document.querySelector('{}');
                    if (element) {{
                        element.select();
                        element.value = '';
                    }}
                "#,
                    search_box.selector
                );
                self.browser.execute_script(&clear_script).await?;

                // Type search query
                self.browser.type_text(&search_box.selector, &query).await?;

                // Try to find and click search button
                let search_result =
                    if let Ok(search_btn) = self.perception.find_element("search button").await {
                        self.browser.click(&search_btn.selector).await?;
                        format!("Searched for '{}' using page search", query)
                    } else {
                        // Press Enter if no button found
                        let press_enter = format!(
                            r#"
                        const element = document.querySelector('{}');
                        if (element) {{
                            const event = new KeyboardEvent('keydown', {{
                                key: 'Enter',
                                code: 'Enter',
                                keyCode: 13,
                                bubbles: true
                            }});
                            element.dispatchEvent(event);
                        }}
                    "#,
                            search_box.selector
                        );
                        self.browser.execute_script(&press_enter).await?;
                        format!("Searched for '{}' (pressed Enter)", query)
                    };

                // Update context
                self.perception
                    .update_context("search", Some(&search_box.selector));

                // Clone values we need before moving search_box
                let search_confidence = search_box.confidence;

                Ok(IntelligentCommandResult {
                    success: true,
                    action: "search".to_string(),
                    message: search_result,
                    element_info: Some(search_box),
                    screenshot: None,
                    extracted_data: None,
                    page_type: None,
                    confidence: search_confidence,
                })
            }
            Err(_) => {
                // No search box found, navigate to Google
                let google_url = format!(
                    "https://www.google.com/search?q={}",
                    urlencoding::encode(&query)
                );
                self.browser.navigate_to(&google_url).await?;

                Ok(IntelligentCommandResult {
                    success: true,
                    action: "search".to_string(),
                    message: format!("Searched Google for '{}'", query),
                    element_info: None,
                    screenshot: None,
                    extracted_data: None,
                    page_type: Some(PageType::SearchResults),
                    confidence: 1.0,
                })
            }
        }
    }

    /// Intelligent navigation with page analysis
    async fn intelligent_navigate(
        &mut self,
        command: IntelligentCommand,
    ) -> Result<IntelligentCommandResult> {
        let url = command
            .target_description
            .ok_or_else(|| anyhow::anyhow!("No URL provided for navigation"))?;

        // Navigate to the URL
        self.browser.navigate_to(&url).await?;

        // Wait a moment for page to load
        tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;

        // Classify the new page
        let page_type = self.perception.classify_page().await?;

        // Update context
        self.perception.update_context("navigate", None);

        Ok(IntelligentCommandResult {
            success: true,
            action: "navigate".to_string(),
            message: format!("Navigated to {} (detected as {:?})", url, page_type),
            element_info: None,
            screenshot: None,
            extracted_data: None,
            page_type: Some(page_type),
            confidence: 1.0,
        })
    }

    /// Intelligent waiting with element detection
    async fn intelligent_wait(
        &mut self,
        command: IntelligentCommand,
    ) -> Result<IntelligentCommandResult> {
        let description = command
            .target_description
            .unwrap_or_else(|| "page to load".to_string());

        let timeout = command.options.wait_for_element.unwrap_or(10000);

        if description == "page to load" {
            // Wait for page to be ready
            let wait_script = r#"
                (function() {
                    return new Promise((resolve) => {
                        if (document.readyState === 'complete') {
                            resolve('ready');
                        } else {
                            window.addEventListener('load', () => resolve('ready'));
                            setTimeout(() => resolve('timeout'), 10000);
                        }
                    });
                })()
            "#;

            self.browser.execute_script(wait_script).await?;

            Ok(IntelligentCommandResult {
                success: true,
                action: "wait".to_string(),
                message: "Page loaded successfully".to_string(),
                element_info: None,
                screenshot: None,
                extracted_data: None,
                page_type: None,
                confidence: 1.0,
            })
        } else {
            // Wait for specific element
            let start_time = std::time::Instant::now();

            loop {
                if start_time.elapsed().as_millis() > timeout as u128 {
                    return Err(anyhow::anyhow!(
                        "Timeout waiting for element: {}",
                        description
                    ));
                }

                match self.perception.find_element(&description).await {
                    Ok(element) => {
                        // Clone values we need before moving element
                        let element_text = element.text.clone();
                        let element_confidence = element.confidence;

                        return Ok(IntelligentCommandResult {
                            success: true,
                            action: "wait".to_string(),
                            message: format!("Found element: {}", element_text),
                            element_info: Some(element),
                            screenshot: None,
                            extracted_data: None,
                            page_type: None,
                            confidence: element_confidence,
                        });
                    }
                    Err(_) => {
                        // Wait a bit before retrying
                        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                    }
                }
            }
        }
    }

    /// Get current page analysis
    pub async fn analyze_current_page(&mut self) -> Result<PageAnalysis> {
        let page_type = self.perception.classify_page().await?;
        let url = self.browser.current_url().await?;
        let extracted_data = self.perception.extract_page_data().await?;

        // Find interactive elements
        let interactive_elements = self.find_interactive_elements().await?;

        Ok(PageAnalysis {
            url,
            page_type,
            interactive_elements,
            extracted_data,
            timestamp: chrono::Utc::now(),
        })
    }

    /// Find all interactive elements on the page
    async fn find_interactive_elements(&mut self) -> Result<Vec<PerceivedElement>> {
        let mut elements = Vec::new();

        // Find buttons
        if let Ok(buttons) = self.perception.find_elements("button").await {
            elements.extend(buttons);
        }

        // Find links
        if let Ok(links) = self.perception.find_elements("link").await {
            elements.extend(links);
        }

        // Find input fields
        if let Ok(inputs) = self.perception.find_elements("input").await {
            elements.extend(inputs);
        }

        Ok(elements)
    }
}

/// Page analysis result
#[derive(Debug, Serialize)]
pub struct PageAnalysis {
    pub url: String,
    pub page_type: PageType,
    pub interactive_elements: Vec<PerceivedElement>,
    pub extracted_data: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Convenience functions for creating common intelligent commands
impl IntelligentCommand {
    pub fn click(description: &str) -> Self {
        Self {
            action: "click".to_string(),
            target_description: Some(description.to_string()),
            input_text: None,
            options: CommandOptions::default(),
        }
    }

    pub fn type_text(field_description: &str, text: &str) -> Self {
        Self {
            action: "type".to_string(),
            target_description: Some(field_description.to_string()),
            input_text: Some(text.to_string()),
            options: CommandOptions::default(),
        }
    }

    pub fn search(query: &str) -> Self {
        Self {
            action: "search".to_string(),
            target_description: None,
            input_text: Some(query.to_string()),
            options: CommandOptions::default(),
        }
    }

    pub fn navigate(url: &str) -> Self {
        Self {
            action: "navigate".to_string(),
            target_description: Some(url.to_string()),
            input_text: None,
            options: CommandOptions::default(),
        }
    }

    pub fn extract_data() -> Self {
        Self {
            action: "extract".to_string(),
            target_description: None,
            input_text: None,
            options: CommandOptions {
                extract_data: true,
                ..CommandOptions::default()
            },
        }
    }

    pub fn wait_for(description: &str, timeout_ms: u64) -> Self {
        Self {
            action: "wait".to_string(),
            target_description: Some(description.to_string()),
            input_text: None,
            options: CommandOptions {
                wait_for_element: Some(timeout_ms),
                ..CommandOptions::default()
            },
        }
    }

    pub fn with_screenshot(mut self) -> Self {
        self.options.take_screenshot = true;
        self
    }

    pub fn with_confidence_threshold(mut self, threshold: f32) -> Self {
        self.options.confidence_threshold = Some(threshold);
        self
    }
}

// Add required dependencies to Cargo.toml:
// base64 = "0.21"
// urlencoding = "2.1"
// chrono = { version = "0.4", features = ["serde"] }
