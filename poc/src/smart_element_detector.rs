use anyhow::{Result, Context};
use thirtyfour::{WebDriver, WebElement, By};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use tracing::{info, warn, debug};
use std::time::Duration;
use tokio::time::timeout;

/// Smart element detection with multiple fallback strategies
pub struct SmartElementDetector {
    driver: WebDriver,
    fallback_selectors: HashMap<String, Vec<String>>,
    max_retries: u32,
    retry_delay: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementDescriptor {
    pub description: String,
    pub element_type: ElementType,
    pub attributes: HashMap<String, String>,
    pub context: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ElementType {
    SearchBox,
    Button,
    Link,
    Input,
    Select,
    Checkbox,
    Radio,
    TextArea,
    Image,
    Navigation,
    Form,
    Unknown,
}

impl SmartElementDetector {
    pub fn new(driver: WebDriver) -> Self {
        let mut fallback_selectors = HashMap::new();
        
        // Common search box selectors across popular websites
        fallback_selectors.insert("search_box".to_string(), vec![
            // Amazon
            "#twotabsearchtextbox".to_string(),
            "input[id*='search']".to_string(),
            // Google
            "input[name='q']".to_string(),
            "input[aria-label*='Search']".to_string(),
            // Generic patterns
            "input[type='search']".to_string(),
            "input[placeholder*='search']".to_string(),
            "input[placeholder*='Search']".to_string(),
            "[data-testid*='search']".to_string(),
            ".search-input".to_string(),
            "#search".to_string(),
            ".search-box".to_string(),
        ]);
        
        // Common button patterns
        fallback_selectors.insert("button".to_string(), vec![
            "button".to_string(),
            "input[type='button']".to_string(),
            "input[type='submit']".to_string(),
            "[role='button']".to_string(),
            ".button".to_string(),
            ".btn".to_string(),
        ]);
        
        // Login/Sign in patterns
        fallback_selectors.insert("login".to_string(), vec![
            "button:contains('Sign in')".to_string(),
            "button:contains('Login')".to_string(),
            "a:contains('Sign in')".to_string(),
            "a:contains('Login')".to_string(),
            "[data-testid*='login']".to_string(),
            "#login".to_string(),
            ".login-button".to_string(),
        ]);
        
        // Common button selectors
        fallback_selectors.insert("submit_button".to_string(), vec![
            "button[type='submit']".to_string(),
            "input[type='submit']".to_string(),
            "[type='submit']".to_string(),
            "button[aria-label*='Search']".to_string(),
            "button[aria-label*='search']".to_string(),
            ".search-button".to_string(),
            ".btn-search".to_string(),
            "#search-button".to_string(),
            "[data-testid*='search-button']".to_string(),
            "[data-testid*='submit']".to_string(),
        ]);
        
        // Login form elements
        fallback_selectors.insert("username_input".to_string(), vec![
            "input[name='username']".to_string(),
            "input[name='email']".to_string(),
            "input[type='email']".to_string(),
            "input[id*='username']".to_string(),
            "input[id*='email']".to_string(),
            "input[placeholder*='Username']".to_string(),
            "input[placeholder*='Email']".to_string(),
            "input[aria-label*='Username']".to_string(),
            "input[aria-label*='Email']".to_string(),
            "[data-testid*='username']".to_string(),
            "[data-testid*='email']".to_string(),
        ]);
        
        fallback_selectors.insert("password_input".to_string(), vec![
            "input[type='password']".to_string(),
            "input[name='password']".to_string(),
            "input[id*='password']".to_string(),
            "input[placeholder*='Password']".to_string(),
            "input[aria-label*='Password']".to_string(),
            "[data-testid*='password']".to_string(),
        ]);
        
        // Shopping cart elements
        fallback_selectors.insert("add_to_cart".to_string(), vec![
            "#add-to-cart-button".to_string(),
            "button[id*='add-to-cart']".to_string(),
            "button[aria-label*='Add to cart']".to_string(),
            "button[aria-label*='Add to Cart']".to_string(),
            ".add-to-cart".to_string(),
            ".btn-add-to-cart".to_string(),
            "[data-testid*='add-to-cart']".to_string(),
            "button:contains('Add to Cart')".to_string(),
        ]);
        
        Self {
            driver,
            fallback_selectors,
            max_retries: 3,
            retry_delay: Duration::from_millis(500),
        }
    }
    
    /// Find an element using multiple strategies
    pub async fn find_element(&self, descriptor: &ElementDescriptor) -> Result<WebElement> {
        info!("Smart detection for: {} (type: {:?})", descriptor.description, descriptor.element_type);
        
        // Strategy 1: Try direct selector if provided in attributes
        if let Some(selector) = descriptor.attributes.get("selector") {
            if let Ok(element) = self.try_selector(selector).await {
                info!("Found element using direct selector: {}", selector);
                return Ok(element);
            }
        }
        
        // Strategy 2: Try ID-based selection
        if let Some(id) = descriptor.attributes.get("id") {
            if let Ok(element) = self.try_selector(&format!("#{}", id)).await {
                info!("Found element by ID: {}", id);
                return Ok(element);
            }
        }
        
        // Strategy 3: Try name-based selection
        if let Some(name) = descriptor.attributes.get("name") {
            if let Ok(element) = self.try_selector(&format!("[name='{}']", name)).await {
                info!("Found element by name: {}", name);
                return Ok(element);
            }
        }
        
        // Strategy 4: Try fallback selectors based on element type
        if let Ok(element) = self.try_fallback_selectors(&descriptor.element_type).await {
            info!("Found element using fallback selector for type: {:?}", descriptor.element_type);
            return Ok(element);
        }
        
        // Strategy 5: Try intelligent text-based search
        if let Ok(element) = self.find_by_text_content(&descriptor.description).await {
            info!("Found element by text content: {}", descriptor.description);
            return Ok(element);
        }
        
        // Strategy 6: Try aria-label search
        if let Ok(element) = self.find_by_aria_label(&descriptor.description).await {
            info!("Found element by aria-label: {}", descriptor.description);
            return Ok(element);
        }
        
        // Strategy 7: Try fuzzy matching on common patterns
        if let Ok(element) = self.fuzzy_pattern_match(descriptor).await {
            info!("Found element using fuzzy pattern matching");
            return Ok(element);
        }
        
        Err(anyhow::anyhow!(
            "Could not find element: {} after trying all strategies", 
            descriptor.description
        ))
    }
    
    /// Try a single CSS selector
    async fn try_selector(&self, selector: &str) -> Result<WebElement> {
        debug!("Trying selector: {}", selector);
        
        match timeout(
            Duration::from_secs(2),
            self.driver.find(By::Css(selector))
        ).await {
            Ok(Ok(element)) => {
                // Verify element is visible and interactable
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
    
    /// Try fallback selectors based on element type
    async fn try_fallback_selectors(&self, element_type: &ElementType) -> Result<WebElement> {
        let key = match element_type {
            ElementType::SearchBox => "search_box",
            ElementType::Button => "submit_button",
            ElementType::Input => {
                // Try both username and generic input
                if let Ok(element) = self.try_selector_list(&["username_input"]).await {
                    return Ok(element);
                }
                return self.try_selector_list(&["password_input"]).await;
            },
            _ => return Err(anyhow::anyhow!("No fallback selectors for type: {:?}", element_type)),
        };
        
        self.try_selector_list(&[key]).await
    }
    
    /// Try a list of selector keys from fallback_selectors
    async fn try_selector_list(&self, keys: &[&str]) -> Result<WebElement> {
        for key in keys {
            if let Some(selectors) = self.fallback_selectors.get(*key) {
                for selector in selectors {
                    if let Ok(element) = self.try_selector(selector).await {
                        return Ok(element);
                    }
                }
            }
        }
        Err(anyhow::anyhow!("No matching elements found in fallback selectors"))
    }
    
    /// Find element by text content
    async fn find_by_text_content(&self, text: &str) -> Result<WebElement> {
        debug!("Searching for element with text: {}", text);
        
        // Try exact match first
        let xpath = format!("//*[text()='{}']", text);
        if let Ok(element) = self.driver.find(By::XPath(&xpath)).await {
            if element.is_displayed().await.unwrap_or(false) {
                return Ok(element);
            }
        }
        
        // Try contains match
        let xpath = format!("//*[contains(text(), '{}')]", text);
        if let Ok(element) = self.driver.find(By::XPath(&xpath)).await {
            if element.is_displayed().await.unwrap_or(false) {
                return Ok(element);
            }
        }
        
        // Try case-insensitive match
        let xpath = format!(
            "//*[contains(translate(text(), 'ABCDEFGHIJKLMNOPQRSTUVWXYZ', 'abcdefghijklmnopqrstuvwxyz'), '{}')]",
            text.to_lowercase()
        );
        if let Ok(element) = self.driver.find(By::XPath(&xpath)).await {
            if element.is_displayed().await.unwrap_or(false) {
                return Ok(element);
            }
        }
        
        Err(anyhow::anyhow!("No element found with text: {}", text))
    }
    
    /// Find element by aria-label
    async fn find_by_aria_label(&self, label: &str) -> Result<WebElement> {
        debug!("Searching for element with aria-label: {}", label);
        
        // Try exact match
        let selector = format!("[aria-label='{}']", label);
        if let Ok(element) = self.try_selector(&selector).await {
            return Ok(element);
        }
        
        // Try contains match
        let selector = format!("[aria-label*='{}']", label);
        if let Ok(element) = self.try_selector(&selector).await {
            return Ok(element);
        }
        
        // Try case-insensitive contains
        let selectors = vec![
            format!("[aria-label*='{}']", label.to_lowercase()),
            format!("[aria-label*='{}']", label.to_uppercase()),
            format!("[aria-label*='{}']", capitalize_first(label)),
        ];
        
        for selector in selectors {
            if let Ok(element) = self.try_selector(&selector).await {
                return Ok(element);
            }
        }
        
        Err(anyhow::anyhow!("No element found with aria-label: {}", label))
    }
    
    /// Fuzzy pattern matching for common UI patterns
    async fn fuzzy_pattern_match(&self, descriptor: &ElementDescriptor) -> Result<WebElement> {
        debug!("Attempting fuzzy pattern match for: {}", descriptor.description);
        
        let description_lower = descriptor.description.to_lowercase();
        
        // Check for common action keywords
        let patterns = if description_lower.contains("search") {
            vec![
                "input[type='text']:first-of-type",
                "input:not([type='hidden']):first-of-type",
                "form input[type='text']",
            ]
        } else if description_lower.contains("login") || description_lower.contains("sign in") {
            vec![
                "button:contains('Login')",
                "button:contains('Sign In')",
                "input[type='submit'][value*='Login']",
                "input[type='submit'][value*='Sign']",
            ]
        } else if description_lower.contains("submit") || description_lower.contains("continue") {
            vec![
                "button[type='submit']",
                "input[type='submit']",
                "button:contains('Submit')",
                "button:contains('Continue')",
            ]
        } else {
            vec![]
        };
        
        for pattern in patterns {
            if let Ok(element) = self.try_selector(pattern).await {
                return Ok(element);
            }
        }
        
        Err(anyhow::anyhow!("Fuzzy pattern matching failed"))
    }
    
    /// Wait for an element to become available
    pub async fn wait_for_element(&self, descriptor: &ElementDescriptor, timeout: Duration) -> Result<WebElement> {
        let start = std::time::Instant::now();
        
        while start.elapsed() < timeout {
            if let Ok(element) = self.find_element(descriptor).await {
                return Ok(element);
            }
            
            tokio::time::sleep(self.retry_delay).await;
        }
        
        Err(anyhow::anyhow!(
            "Timeout waiting for element: {} after {:?}",
            descriptor.description,
            timeout
        ))
    }
    
    /// Find multiple elements matching a descriptor
    pub async fn find_elements(&self, descriptor: &ElementDescriptor) -> Result<Vec<WebElement>> {
        let mut elements = Vec::new();
        
        // Try direct selector if provided
        if let Some(selector) = descriptor.attributes.get("selector") {
            if let Ok(found) = self.driver.find_all(By::Css(selector)).await {
                elements.extend(found);
            }
        }
        
        // Try fallback selectors
        if let Some(selectors) = self.get_fallback_selectors(&descriptor.element_type) {
            for selector in selectors {
                if let Ok(found) = self.driver.find_all(By::Css(selector)).await {
                    elements.extend(found);
                }
            }
        }
        
        if elements.is_empty() {
            Err(anyhow::anyhow!("No elements found matching descriptor"))
        } else {
            Ok(elements)
        }
    }
    
    fn get_fallback_selectors(&self, element_type: &ElementType) -> Option<&Vec<String>> {
        let key = match element_type {
            ElementType::SearchBox => "search_box",
            ElementType::Button => "submit_button",
            ElementType::Input => "username_input",
            _ => return None,
        };
        
        self.fallback_selectors.get(key)
    }
}

fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().chain(chars).collect(),
    }
}

/// Helper function to detect element type from description
pub fn detect_element_type(description: &str) -> ElementType {
    let desc_lower = description.to_lowercase();
    
    if desc_lower.contains("search") && (desc_lower.contains("box") || desc_lower.contains("field") || desc_lower.contains("input")) {
        ElementType::SearchBox
    } else if desc_lower.contains("button") || desc_lower.contains("submit") || desc_lower.contains("click") {
        ElementType::Button
    } else if desc_lower.contains("link") || desc_lower.contains("navigate") {
        ElementType::Link
    } else if desc_lower.contains("select") || desc_lower.contains("dropdown") || desc_lower.contains("choose") {
        ElementType::Select
    } else if desc_lower.contains("checkbox") || desc_lower.contains("check box") {
        ElementType::Checkbox
    } else if desc_lower.contains("radio") {
        ElementType::Radio
    } else if desc_lower.contains("input") || desc_lower.contains("field") || desc_lower.contains("text") {
        ElementType::Input
    } else if desc_lower.contains("form") {
        ElementType::Form
    } else {
        ElementType::Unknown
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_detect_element_type() {
        assert!(matches!(detect_element_type("search box"), ElementType::SearchBox));
        assert!(matches!(detect_element_type("Submit button"), ElementType::Button));
        assert!(matches!(detect_element_type("email input field"), ElementType::Input));
        assert!(matches!(detect_element_type("dropdown menu"), ElementType::Select));
        assert!(matches!(detect_element_type("click here link"), ElementType::Link));
    }
    
    #[test]
    fn test_capitalize_first() {
        assert_eq!(capitalize_first("hello"), "Hello");
        assert_eq!(capitalize_first("WORLD"), "WORLD");
        assert_eq!(capitalize_first(""), "");
    }
}