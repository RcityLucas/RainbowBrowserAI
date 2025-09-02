// Perception Module MVP - Start here for immediate improvements
// This is a minimal but functional perception system you can build on

use std::collections::HashMap;
use anyhow::{Result, Context};
use serde::{Serialize, Deserialize};
use thirtyfour::{WebDriver, WebElement, By};
use regex::Regex;

pub mod integration;
pub mod visual;
pub mod semantic;
pub mod context_aware;
pub mod smart_forms;
pub mod dynamic_handler;
pub mod testing_framework;
pub mod browser_connection;
pub mod lightning_real;
pub mod quick_real;
pub mod standard_perception;
pub mod deep_perception;
pub mod cache_system;
pub mod natural_language;
pub mod perception_orchestrator;
pub mod enhanced_error_recovery;
pub mod enhanced_form_handler;
pub mod advanced_perception_engine;

// Re-export key types for external use
pub use perception_orchestrator::{
    PerceptionOrchestrator, UnifiedPerceptionResult, PerceptionLevel, 
    ExecutionInfo, PerformanceMetrics, Recommendation, RecommendationType,
    RecommendationPriority, ImplementationEffort
};
pub use advanced_perception_engine::{
    AdvancedPerceptionEngine, AdvancedPerceptionResult, AdvancedPerceptionConfig,
    PerceptionStrategy, PerceptionStats, ComprehensiveStats
};
pub use enhanced_error_recovery::{
    EnhancedErrorRecovery, RecoveryResult, RecoveryConfig, RecoveryStrategy
};
pub use enhanced_form_handler::{
    EnhancedFormHandler, FormInteractionResult, FormFieldType, FormFieldState
};
pub use lightning_real::{LightningData, RealLightningPerception, PageStatus, PageState};
pub use quick_real::{QuickData, RealQuickPerception};
pub use standard_perception::{StandardData, RealStandardPerception};
pub use deep_perception::{DeepData, RealDeepPerception};
pub use browser_connection::{BrowserConnection, BrowserConfig};

/// Core perception engine that understands web pages
pub struct PerceptionEngineMVP {
    driver: WebDriver,
    context: PerceptionContext,
    element_cache: HashMap<String, CachedElement>,
}

/// Maintains context across interactions
#[derive(Debug, Clone)]
pub struct PerceptionContext {
    pub current_url: String,
    pub page_type: PageType,
    pub last_action: Option<String>,
    pub last_element: Option<String>,
    pub form_state: HashMap<String, String>,
    pub named_elements: HashMap<String, String>, // "the search box" -> selector
}

/// Categorizes what type of page we're on
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PageType {
    Homepage,
    SearchResults,
    LoginPage,
    ProductPage,
    ArticlePage,
    FormPage,
    Unknown,
}

/// Represents an element with metadata
#[derive(Debug, Clone)]
pub struct PerceivedElement {
    pub selector: String,
    pub text: String,
    pub element_type: ElementType,
    pub clickable: bool,
    pub visible: bool,
    pub confidence: f32,
    pub attributes: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ElementType {
    Button,
    Link,
    Input,
    Select,
    TextArea,
    Image,
    Text,
    Container,
    Form,
    Navigation,
    Media,
    Other,
    Unknown,
}

/// Cached element for performance
#[derive(Debug, Clone)]
struct CachedElement {
    selector: String,
    element_type: ElementType,
    last_seen: std::time::Instant,
}

impl PerceptionEngineMVP {
    pub fn new(driver: WebDriver) -> Self {
        Self {
            driver,
            context: PerceptionContext {
                current_url: String::new(),
                page_type: PageType::Unknown,
                last_action: None,
                last_element: None,
                form_state: HashMap::new(),
                named_elements: HashMap::new(),
            },
            element_cache: HashMap::new(),
        }
    }

    /// Main entry point: Find element using natural language
    pub async fn find_element(&mut self, description: &str) -> Result<PerceivedElement> {
        // Step 1: Check if this is a reference to a previous element
        if let Some(element) = self.resolve_reference(description).await? {
            return Ok(element);
        }

        // Step 2: Try to find by common patterns
        let candidates = self.find_candidates(description).await?;
        
        // Step 3: Score and rank candidates
        let best = self.select_best_candidate(candidates, description)?;
        
        // Step 4: Cache the result
        self.cache_element(description, &best);
        
        Ok(best)
    }

    /// Find elements that might match the description
    async fn find_candidates(&self, description: &str) -> Result<Vec<PerceivedElement>> {
        let mut candidates = Vec::new();
        let desc_lower = description.to_lowercase();

        // Strategy 1: Direct text match on buttons and links
        if desc_lower.contains("button") || desc_lower.contains("click") {
            candidates.extend(self.find_buttons_by_text(&desc_lower).await?);
        }

        // Strategy 2: Form elements by label
        if desc_lower.contains("field") || desc_lower.contains("input") || 
           desc_lower.contains("type") || desc_lower.contains("enter") {
            candidates.extend(self.find_inputs_by_label(&desc_lower).await?);
        }

        // Strategy 3: Links by text
        if desc_lower.contains("link") || desc_lower.contains("go to") {
            candidates.extend(self.find_links_by_text(&desc_lower).await?);
        }

        // Strategy 4: Common UI patterns
        candidates.extend(self.find_by_common_patterns(&desc_lower).await?);

        // Strategy 5: Aria labels and roles
        candidates.extend(self.find_by_accessibility(&desc_lower).await?);

        Ok(candidates)
    }

    /// Find buttons by their text content
    async fn find_buttons_by_text(&self, text: &str) -> Result<Vec<PerceivedElement>> {
        let mut elements = Vec::new();
        
        // Check actual button elements
        let buttons = self.driver.find_all(By::Tag("button")).await?;
        for button in buttons {
            if let Ok(btn_text) = button.text().await {
                if self.text_matches(&btn_text.to_lowercase(), text) {
                    elements.push(self.create_perceived_element(button, ElementType::Button).await?);
                }
            }
        }

        // Check input type=button/submit
        let input_buttons = self.driver.find_all(By::Css("input[type='button'], input[type='submit']")).await?;
        for button in input_buttons {
            if let Ok(value) = button.attr("value").await {
                if let Some(v) = value {
                    if self.text_matches(&v.to_lowercase(), text) {
                        elements.push(self.create_perceived_element(button, ElementType::Button).await?);
                    }
                }
            }
        }

        // Check links that look like buttons
        let link_buttons = self.driver.find_all(By::Css("a.button, a.btn, a[role='button']")).await?;
        for link in link_buttons {
            if let Ok(link_text) = link.text().await {
                if self.text_matches(&link_text.to_lowercase(), text) {
                    elements.push(self.create_perceived_element(link, ElementType::Button).await?);
                }
            }
        }

        Ok(elements)
    }

    /// Find input fields by their labels
    async fn find_inputs_by_label(&self, description: &str) -> Result<Vec<PerceivedElement>> {
        let mut elements = Vec::new();
        
        // Find all labels
        let labels = self.driver.find_all(By::Tag("label")).await?;
        for label in labels {
            if let Ok(label_text) = label.text().await {
                if self.text_matches(&label_text.to_lowercase(), description) {
                    // Find associated input
                    if let Ok(Some(for_attr)) = label.attr("for").await {
                        // Label has 'for' attribute
                        if let Ok(input) = self.driver.find(By::Id(&for_attr)).await {
                            elements.push(self.create_perceived_element(input, ElementType::Input).await?);
                        }
                    } else {
                        // Label might contain the input
                        if let Ok(input) = label.find(By::Tag("input")).await {
                            elements.push(self.create_perceived_element(input, ElementType::Input).await?);
                        }
                    }
                }
            }
        }

        // Also check placeholder text
        let inputs_with_placeholder = self.driver.find_all(By::Tag("input")).await?;
        for input in inputs_with_placeholder {
            if let Ok(Some(placeholder)) = input.attr("placeholder").await {
                if self.text_matches(&placeholder.to_lowercase(), description) {
                    elements.push(self.create_perceived_element(input, ElementType::Input).await?);
                }
            }
        }

        Ok(elements)
    }

    /// Find links by text content
    async fn find_links_by_text(&self, text: &str) -> Result<Vec<PerceivedElement>> {
        let mut elements = Vec::new();
        let links = self.driver.find_all(By::Tag("a")).await?;
        
        for link in links {
            if let Ok(link_text) = link.text().await {
                if self.text_matches(&link_text.to_lowercase(), text) {
                    elements.push(self.create_perceived_element(link, ElementType::Link).await?);
                }
            }
        }

        Ok(elements)
    }

    /// Find elements by common UI patterns
    async fn find_by_common_patterns(&self, description: &str) -> Result<Vec<PerceivedElement>> {
        let mut elements = Vec::new();

        // Search box patterns
        if description.contains("search") {
            let search_selectors = vec![
                "input[type='search']",
                "input[name*='search']", 
                "input[placeholder*='search']",
                "#search",
                ".search-input",
                "input[aria-label*='search']",
            ];

            for selector in search_selectors {
                if let Ok(elem) = self.driver.find(By::Css(selector)).await {
                    elements.push(self.create_perceived_element(elem, ElementType::Input).await?);
                    break;
                }
            }
        }

        // Login/Sign in patterns
        if description.contains("login") || description.contains("sign in") {
            let login_selectors = vec![
                "button:contains('Login')",
                "button:contains('Sign in')",
                "a:contains('Login')",
                "a:contains('Sign in')",
                "#login",
                ".login-button",
            ];

            for selector in login_selectors {
                if let Ok(elem) = self.driver.find(By::Css(selector)).await {
                    elements.push(self.create_perceived_element(elem, ElementType::Button).await?);
                    break;
                }
            }
        }

        // Navigation patterns
        if description.contains("menu") || description.contains("navigation") {
            let nav_selectors = vec![
                "nav",
                ".navigation",
                ".menu",
                "#menu",
                "[role='navigation']",
            ];

            for selector in nav_selectors {
                if let Ok(elem) = self.driver.find(By::Css(selector)).await {
                    elements.push(self.create_perceived_element(elem, ElementType::Container).await?);
                    break;
                }
            }
        }

        Ok(elements)
    }

    /// Find elements using accessibility attributes
    async fn find_by_accessibility(&self, description: &str) -> Result<Vec<PerceivedElement>> {
        let mut elements = Vec::new();
        
        // Find by aria-label
        let aria_elements = self.driver.find_all(By::Css("[aria-label]")).await?;
        for elem in aria_elements {
            if let Ok(Some(label)) = elem.attr("aria-label").await {
                if self.text_matches(&label.to_lowercase(), description) {
                    let elem_type = self.detect_element_type(&elem).await?;
                    elements.push(self.create_perceived_element(elem, elem_type).await?);
                }
            }
        }

        // Find by role
        if description.contains("button") {
            let role_buttons = self.driver.find_all(By::Css("[role='button']")).await?;
            for button in role_buttons {
                elements.push(self.create_perceived_element(button, ElementType::Button).await?);
            }
        }

        Ok(elements)
    }

    /// Check if text matches (fuzzy matching)
    fn text_matches(&self, element_text: &str, search_text: &str) -> bool {
        // Remove common words for better matching
        let binding = search_text
            .replace("click", "")
            .replace("button", "")
            .replace("the", "")
            .replace("on", "");
        let clean_search = binding.trim();

        // Direct containment check
        if element_text.contains(clean_search) {
            return true;
        }

        // Check individual words
        let search_words: Vec<&str> = clean_search.split_whitespace().collect();
        for word in search_words {
            if word.len() > 2 && element_text.contains(word) {
                return true;
            }
        }

        // Common synonyms
        let synonyms = vec![
            ("sign in", "login"),
            ("sign up", "register"),
            ("submit", "send"),
            ("search", "find"),
        ];

        for (syn1, syn2) in synonyms {
            if (search_text.contains(syn1) && element_text.contains(syn2)) ||
               (search_text.contains(syn2) && element_text.contains(syn1)) {
                return true;
            }
        }

        false
    }

    /// Select the best candidate from multiple options
    fn select_best_candidate(&self, candidates: Vec<PerceivedElement>, description: &str) -> Result<PerceivedElement> {
        if candidates.is_empty() {
            anyhow::bail!("No elements found matching: {}", description);
        }

        // For now, return the first visible, clickable element
        // TODO: Implement proper scoring based on:
        // - Text similarity
        // - Visual prominence
        // - Position on page
        // - Element type match

        candidates.into_iter()
            .filter(|e| e.visible)
            .max_by_key(|e| (e.confidence * 100.0) as i32)
            .ok_or_else(|| anyhow::anyhow!("No suitable element found for: {}", description))
    }

    /// Resolve references like "it", "that button", etc.
    async fn resolve_reference(&self, description: &str) -> Result<Option<PerceivedElement>> {
        // Check for pronouns
        if description == "it" || description == "that" {
            if let Some(last_elem) = &self.context.last_element {
                if let Ok(elem) = self.driver.find(By::Css(last_elem)).await {
                    let elem_type = self.detect_element_type(&elem).await?;
                    return Ok(Some(self.create_perceived_element(elem, elem_type).await?));
                }
            }
        }

        // Check named elements
        if let Some(selector) = self.context.named_elements.get(description) {
            if let Ok(elem) = self.driver.find(By::Css(selector)).await {
                let elem_type = self.detect_element_type(&elem).await?;
                return Ok(Some(self.create_perceived_element(elem, elem_type).await?));
            }
        }

        Ok(None)
    }

    /// Cache element for future reference
    fn cache_element(&mut self, description: &str, element: &PerceivedElement) {
        self.context.last_element = Some(element.selector.clone());
        
        // Store named references
        if description.contains("the ") {
            self.context.named_elements.insert(description.to_string(), element.selector.clone());
        }

        // Cache for performance
        self.element_cache.insert(
            description.to_string(),
            CachedElement {
                selector: element.selector.clone(),
                element_type: element.element_type.clone(),
                last_seen: std::time::Instant::now(),
            }
        );
    }

    /// Detect what type of element this is
    async fn detect_element_type(&self, element: &WebElement) -> Result<ElementType> {
        let tag = element.tag_name().await?.to_lowercase();
        
        Ok(match tag.as_str() {
            "button" => ElementType::Button,
            "a" => ElementType::Link,
            "input" => ElementType::Input,
            "select" => ElementType::Select,
            "textarea" => ElementType::TextArea,
            "img" => ElementType::Image,
            "div" | "section" | "article" => ElementType::Container,
            _ => ElementType::Unknown,
        })
    }

    /// Create a PerceivedElement from a WebElement
    async fn create_perceived_element(&self, element: WebElement, elem_type: ElementType) -> Result<PerceivedElement> {
        let text = element.text().await.unwrap_or_default();
        let is_displayed = element.is_displayed().await.unwrap_or(false);
        let is_enabled = element.is_enabled().await.unwrap_or(false);
        
        // Get CSS selector (simplified - you might want a better selector generator)
        let id = element.attr("id").await?.unwrap_or_default();
        let selector = if !id.is_empty() {
            format!("#{}", id)
        } else {
            // Generate a unique selector
            // This is simplified - in production, you'd want a robust selector generator
            format!("{}:nth-of-type(1)", element.tag_name().await?)
        };

        // Collect attributes
        let mut attributes = HashMap::new();
        for attr in ["class", "name", "type", "placeholder", "aria-label"] {
            if let Ok(Some(value)) = element.attr(attr).await {
                attributes.insert(attr.to_string(), value);
            }
        }

        Ok(PerceivedElement {
            selector,
            text,
            element_type: elem_type,
            clickable: is_enabled && is_displayed,
            visible: is_displayed,
            confidence: 0.8, // Default confidence
            attributes,
        })
    }

    /// Classify what type of page this is
    pub async fn classify_page(&mut self) -> Result<PageType> {
        let url = self.driver.current_url().await?;
        let title = self.driver.title().await?;
        
        // Simple heuristics - expand this with ML or more rules
        let url_str = url.as_str();
        let page_type = if url_str.contains("login") || url_str.contains("signin") {
            PageType::LoginPage
        } else if url_str.contains("search") || title.contains("Search") {
            PageType::SearchResults
        } else if url_str.contains("product") || url_str.contains("item") {
            PageType::ProductPage
        } else if self.driver.find(By::Tag("article")).await.is_ok() {
            PageType::ArticlePage
        } else if self.driver.find_all(By::Tag("form")).await?.len() > 0 {
            PageType::FormPage
        } else if url_str == self.driver.current_url().await?.as_str() {
            PageType::Homepage
        } else {
            PageType::Unknown
        };

        self.context.page_type = page_type.clone();
        Ok(page_type)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These would need a mock WebDriver implementation
    // For now, just testing the text matching logic
    
    #[test]
    fn test_text_matching() {
        // This would need to be refactored to not require WebDriver
        // Just showing the test structure
        
        // Test direct matches
        // assert!(text_matches("Sign In", "click sign in"));
        // assert!(text_matches("Login", "login button"));
        // assert!(text_matches("Search", "search box"));
        
        // Test synonym matching
        // assert!(text_matches("Sign In", "login"));
        // assert!(text_matches("Register", "sign up"));
    }
}