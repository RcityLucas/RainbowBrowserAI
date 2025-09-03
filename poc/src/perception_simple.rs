// Simple, working perception module
// This is a minimal implementation that compiles and provides basic functionality

use anyhow::Result;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use thirtyfour::{WebDriver, WebElement, By};
use tracing::info;

/// Simple perception engine for web page understanding
pub struct SimplePerception {
    driver: WebDriver,
}

/// Represents an element found on the page
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FoundElement {
    pub selector: String,
    pub text: String,
    pub element_type: String,
    pub tag: String,
    pub confidence: f32,
}

/// Page type classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PageType {
    Homepage,
    SearchResults,
    ProductPage,
    Article,
    Form,
    Unknown,
}

impl SimplePerception {
    /// Create a new perception instance
    pub fn new(driver: WebDriver) -> Self {
        Self { driver }
    }

    /// Find an element by description
    pub async fn find_element(&self, description: &str) -> Result<FoundElement> {
        info!("Finding element: {}", description);
        
        // Simple heuristic-based element finding
        let selector = self.description_to_selector(description);
        
        match self.driver.find(By::Css(&selector)).await {
            Ok(element) => {
                let text = element.text().await.unwrap_or_default();
                let tag = element.tag_name().await.unwrap_or_else(|_| "unknown".to_string());
                Ok(FoundElement {
                    selector,
                    text,
                    element_type: self.infer_element_type(description),
                    tag,
                    confidence: 0.8,
                })
            }
            Err(_) => {
                // Try alternative selectors
                let alt_selector = self.get_alternative_selector(description);
                let element = self.driver.find(By::Css(&alt_selector)).await?;
                let text = element.text().await.unwrap_or_default();
                let tag = element.tag_name().await.unwrap_or_else(|_| "unknown".to_string());
                
                Ok(FoundElement {
                    selector: alt_selector,
                    text,
                    element_type: self.infer_element_type(description),
                    tag,
                    confidence: 0.6,
                })
            }
        }
    }

    /// Convert description to CSS selector
    fn description_to_selector(&self, description: &str) -> String {
        let desc_lower = description.to_lowercase();
        
        if desc_lower.contains("button") {
            "button, input[type='submit'], input[type='button'], a.btn, .button".to_string()
        } else if desc_lower.contains("search") {
            "input[type='search'], input[name*='search'], input[placeholder*='search']".to_string()
        } else if desc_lower.contains("input") || desc_lower.contains("field") {
            "input[type='text'], input[type='email'], textarea".to_string()
        } else if desc_lower.contains("link") {
            "a".to_string()
        } else if desc_lower.contains("image") {
            "img".to_string()
        } else if desc_lower.contains("title") || desc_lower.contains("heading") {
            "h1, h2, h3".to_string()
        } else {
            "*".to_string() // Fallback to any element
        }
    }

    /// Get alternative selector if primary fails
    fn get_alternative_selector(&self, description: &str) -> String {
        let desc_lower = description.to_lowercase();
        
        // Try to find by partial text match
        if desc_lower.contains("submit") || desc_lower.contains("send") {
            "button[type='submit'], input[type='submit']".to_string()
        } else if desc_lower.contains("cancel") || desc_lower.contains("close") {
            "button.cancel, button.close, a.cancel, a.close".to_string()
        } else {
            "div, span, p".to_string() // Generic fallback
        }
    }

    /// Infer element type from description
    fn infer_element_type(&self, description: &str) -> String {
        let desc_lower = description.to_lowercase();
        
        if desc_lower.contains("button") {
            "button".to_string()
        } else if desc_lower.contains("link") {
            "link".to_string()
        } else if desc_lower.contains("input") || desc_lower.contains("field") {
            "input".to_string()
        } else if desc_lower.contains("image") {
            "image".to_string()
        } else if desc_lower.contains("text") {
            "text".to_string()
        } else {
            "unknown".to_string()
        }
    }

    /// Classify the current page type (alias for analyze_page)
    pub async fn classify_page(&self) -> Result<PageType> {
        self.analyze_page().await
    }
    
    /// Analyze the current page type
    pub async fn analyze_page(&self) -> Result<PageType> {
        info!("Analyzing page type");
        
        // Get page URL
        let url = self.driver.current_url().await?;
        let url_str = url.as_str();
        
        // Simple URL-based classification
        if url_str.contains("search") || url_str.contains("results") {
            return Ok(PageType::SearchResults);
        }
        if url_str.contains("product") || url_str.contains("item") {
            return Ok(PageType::ProductPage);
        }
        if url_str.contains("article") || url_str.contains("blog") || url_str.contains("post") {
            return Ok(PageType::Article);
        }
        
        // Check for common elements
        if self.driver.find(By::Css("form")).await.is_ok() {
            let inputs = self.driver.find_all(By::Css("input")).await?;
            if inputs.len() > 3 {
                return Ok(PageType::Form);
            }
        }
        
        // Check if it looks like a homepage
        if url_str.ends_with('/') || url_str.ends_with(".com") || url_str.ends_with(".org") {
            return Ok(PageType::Homepage);
        }
        
        Ok(PageType::Unknown)
    }

    /// Get all interactive elements on the page
    pub async fn get_interactive_elements(&self) -> Result<Vec<FoundElement>> {
        info!("Getting all interactive elements");
        
        let mut elements = Vec::new();
        
        // Find buttons
        if let Ok(buttons) = self.driver.find_all(By::Css("button, input[type='submit']")).await {
            for button in buttons.iter().take(10) {
                if let Ok(text) = button.text().await {
                    elements.push(FoundElement {
                        selector: "button".to_string(),
                        text,
                        element_type: "button".to_string(),
                        confidence: 0.9,
                    });
                }
            }
        }
        
        // Find links
        if let Ok(links) = self.driver.find_all(By::Css("a[href]")).await {
            for link in links.iter().take(10) {
                if let Ok(text) = link.text().await {
                    if !text.is_empty() {
                        elements.push(FoundElement {
                            selector: "a".to_string(),
                            text,
                            element_type: "link".to_string(),
                            confidence: 0.9,
                        });
                    }
                }
            }
        }
        
        // Find input fields
        if let Ok(inputs) = self.driver.find_all(By::Css("input[type='text'], textarea")).await {
            for input in inputs.iter().take(5) {
                let placeholder = input.attr("placeholder").await.ok().flatten().unwrap_or_default();
                elements.push(FoundElement {
                    selector: "input".to_string(),
                    text: placeholder,
                    element_type: "input".to_string(),
                    confidence: 0.9,
                });
            }
        }
        
        Ok(elements)
    }
}

/// Simple executor that uses perception
pub struct PerceptionExecutor {
    perception: SimplePerception,
    driver: WebDriver,
}

impl PerceptionExecutor {
    pub fn new(driver: WebDriver) -> Self {
        let perception = SimplePerception::new(driver.clone());
        Self { perception, driver }
    }

    /// Execute a high-level command using perception
    pub async fn execute(&self, command: &str) -> Result<String> {
        info!("Executing command with perception: {}", command);
        
        let command_lower = command.to_lowercase();
        
        if command_lower.contains("click") {
            // Extract what to click
            let target = command.replace("click", "").trim().to_string();
            let element = self.perception.find_element(&target).await?;
            
            // Click the element
            let web_element = self.driver.find(By::Css(&element.selector)).await?;
            web_element.click().await?;
            
            Ok(format!("Clicked on: {}", element.text))
            
        } else if command_lower.contains("type") {
            // Extract what to type and where
            // Simple parsing - in production this would be more sophisticated
            let parts: Vec<&str> = command.split("into").collect();
            if parts.len() == 2 {
                let text = parts[0].replace("type", "").trim().to_string();
                let target = parts[1].trim();
                
                let element = self.perception.find_element(target).await?;
                let web_element = self.driver.find(By::Css(&element.selector)).await?;
                web_element.send_keys(&text).await?;
                
                Ok(format!("Typed '{}' into {}", text, target))
            } else {
                Ok("Could not parse type command".to_string())
            }
            
        } else if command_lower.contains("find") {
            let target = command.replace("find", "").trim().to_string();
            let element = self.perception.find_element(&target).await?;
            Ok(format!("Found element: {} (text: {})", element.element_type, element.text))
            
        } else if command_lower.contains("analyze") {
            let page_type = self.perception.analyze_page().await?;
            let elements = self.perception.get_interactive_elements().await?;
            Ok(format!("Page type: {:?}, Found {} interactive elements", page_type, elements.len()))
            
        } else {
            Ok(format!("Command not recognized: {}", command))
        }
    }
}