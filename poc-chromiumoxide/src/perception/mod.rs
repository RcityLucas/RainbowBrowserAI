// Perception Module for Chromiumoxide Edition
// Advanced visual understanding and element detection for browser automation

use std::collections::HashMap;
use anyhow::Result;
use serde::{Serialize, Deserialize};
use crate::browser::Browser;
use tracing::{info, debug};

pub mod visual;
pub mod semantic;
pub mod context_aware;
pub mod smart_forms;
pub mod integration;

/// Core perception engine that understands web pages using chromiumoxide
pub struct PerceptionEngine {
    browser: std::sync::Arc<Browser>,
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
    pub screenshot_cache: Option<Vec<u8>>, // Latest screenshot for visual analysis
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
    Dashboard,
    Settings,
    Unknown,
}

/// Analysis result for a web page
#[derive(Debug, Serialize, Deserialize)]
pub struct PageAnalysis {
    pub url: String,
    pub page_type: PageType,
    pub title: String,
    pub semantic_analysis: semantic::SemanticAnalysis,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Represents an element with enhanced metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerceivedElement {
    pub selector: String,
    pub text: String,
    pub element_type: ElementType,
    pub clickable: bool,
    pub visible: bool,
    pub confidence: f32,
    pub attributes: HashMap<String, String>,
    pub position: Option<ElementPosition>,
    pub visual_context: Option<VisualContext>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ElementType {
    Button,
    Link,
    Input,
    Select,
    TextArea,
    Image,
    Text,
    Container,
    Navigation,
    Modal,
    Dropdown,
    Checkbox,
    Radio,
    Unknown,
}

/// Element position information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementPosition {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// Visual context for element understanding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualContext {
    pub nearby_elements: Vec<String>,
    pub parent_context: Option<String>,
    pub visual_prominence: f32, // 0.0 to 1.0
}

/// Cached element for performance
#[derive(Debug, Clone)]
struct CachedElement {
    selector: String,
    element_type: ElementType,
    last_seen: std::time::Instant,
    confidence: f32,
}

impl PerceptionEngine {
    /// Create new perception engine
    pub fn new(browser: std::sync::Arc<Browser>) -> Self {
        Self {
            browser,
            context: PerceptionContext {
                current_url: String::new(),
                page_type: PageType::Unknown,
                last_action: None,
                last_element: None,
                form_state: HashMap::new(),
                named_elements: HashMap::new(),
                screenshot_cache: None,
            },
            element_cache: HashMap::new(),
        }
    }

    /// Analyze the current page and return page information
    pub async fn analyze_page(&mut self) -> Result<PageAnalysis> {
        // Get current URL
        let url = self.browser.current_url().await.unwrap_or_else(|_| "unknown".to_string());
        self.context.current_url = url.clone();
        
        // Classify page type
        let page_type = self.classify_page().await?;
        self.context.page_type = page_type.clone();
        
        // Get page content for semantic analysis - use execute_script to get HTML content
        let page_source_script = "(function() { return document.documentElement.outerHTML; })();";
        let page_source = self.browser.execute_script(page_source_script).await?
            .as_str().unwrap_or("").to_string();
        let semantic_analyzer = semantic::SemanticAnalyzer::new();
        let semantic_analysis = semantic_analyzer.analyze_page_semantics(&page_source).await?;
        
        // Get page title
        let title_script = "(function() { return document.title || 'Unknown'; })();";
        let title = self.browser.execute_script(title_script).await?
            .as_str().unwrap_or("Unknown").to_string();
        
        Ok(PageAnalysis {
            url,
            page_type,
            title,
            semantic_analysis,
            timestamp: chrono::Utc::now(),
        })
    }

    /// Main entry point: Find element using natural language or description
    pub async fn find_element(&mut self, description: &str) -> Result<PerceivedElement> {
        debug!("Finding element with description: {}", description);

        // Step 1: Check cache for recent lookups
        if let Some(cached) = self.check_cache(description) {
            debug!("Found cached element for: {}", description);
            return Ok(self.create_perceived_from_cache(&cached).await?);
        }

        // Step 2: Check for references to previous elements
        if let Some(element) = self.resolve_reference(description).await? {
            return Ok(element);
        }

        // Step 3: Find candidates using multiple strategies
        let candidates = self.find_candidates(description).await?;
        
        // Step 4: Score and select the best candidate
        let best = self.select_best_candidate(candidates, description).await?;
        
        // Step 5: Cache the result for future use
        self.cache_element(description, &best);
        
        Ok(best)
    }

    /// Find multiple elements matching a description
    pub async fn find_elements(&mut self, description: &str) -> Result<Vec<PerceivedElement>> {
        debug!("Finding multiple elements with description: {}", description);
        self.find_candidates(description).await
    }

    /// Classify the current page type
    pub async fn classify_page(&mut self) -> Result<PageType> {
        let url = self.browser.current_url().await?;
        self.context.current_url = url.clone();
        
        // Take a screenshot for visual analysis
        let screenshot = self.browser.screenshot(crate::browser::ScreenshotOptions::default()).await?;
        self.context.screenshot_cache = Some(screenshot);
        
        // Use URL and page content analysis
        let page_type = self.classify_by_url_and_content(&url).await?;
        self.context.page_type = page_type.clone();
        
        info!("Classified page as: {:?}", page_type);
        Ok(page_type)
    }

    /// Extract structured data based on page type
    pub async fn extract_page_data(&mut self) -> Result<serde_json::Value> {
        let page_type = &self.context.page_type.clone();
        
        match page_type {
            PageType::ProductPage => self.extract_product_data().await,
            PageType::ArticlePage => self.extract_article_data().await,
            PageType::SearchResults => self.extract_search_results().await,
            PageType::FormPage => self.extract_form_data().await,
            _ => self.extract_generic_data().await,
        }
    }

    /// Update context after an action
    pub fn update_context(&mut self, action: &str, element_selector: Option<&str>) {
        self.context.last_action = Some(action.to_string());
        if let Some(selector) = element_selector {
            self.context.last_element = Some(selector.to_string());
        }
    }

    // Private helper methods

    async fn find_candidates(&self, description: &str) -> Result<Vec<PerceivedElement>> {
        let mut candidates = Vec::new();
        let desc_lower = description.to_lowercase();

        // Strategy 1: Direct element type matching
        candidates.extend(self.find_by_element_type(&desc_lower).await?);
        
        // Strategy 2: Text content matching
        candidates.extend(self.find_by_text_content(&desc_lower).await?);
        
        // Strategy 3: Common UI patterns
        candidates.extend(self.find_by_ui_patterns(&desc_lower).await?);
        
        // Strategy 4: Accessibility attributes
        candidates.extend(self.find_by_accessibility(&desc_lower).await?);
        
        // Strategy 5: Visual context (using screenshot analysis)
        if self.context.screenshot_cache.is_some() {
            candidates.extend(self.find_by_visual_context(&desc_lower).await?);
        }

        Ok(candidates)
    }

    async fn find_by_element_type(&self, description: &str) -> Result<Vec<PerceivedElement>> {
        let mut elements = Vec::new();

        // Button detection
        if description.contains("button") || description.contains("click") {
            let button_script = r#"
                (function() {
                    return Array.from(document.querySelectorAll('button, input[type="button"], input[type="submit"], [role="button"]'))
                        .map(el => ({
                            selector: el.tagName.toLowerCase() + (el.id ? '#' + el.id : ''),
                            text: el.textContent?.trim() || el.value || '',
                            type: el.tagName.toLowerCase(),
                            visible: el.offsetParent !== null,
                            clickable: !el.disabled
                        }));
                })()
            "#;
            
            if let Ok(result) = self.browser.execute_script(button_script).await {
                if let Ok(buttons) = serde_json::from_value::<Vec<serde_json::Value>>(result) {
                    for button in buttons {
                        if let Ok(element) = self.create_perceived_element_from_json(button, ElementType::Button).await {
                            elements.push(element);
                        }
                    }
                }
            }
        }

        // Input field detection
        if description.contains("input") || description.contains("field") || description.contains("type") {
            let input_script = r#"
                (function() {
                    return Array.from(document.querySelectorAll('input, textarea'))
                        .map(el => ({
                            selector: el.tagName.toLowerCase() + (el.id ? '#' + el.id : ''),
                            text: el.placeholder || el.getAttribute('aria-label') || '',
                            type: el.type || 'text',
                            visible: el.offsetParent !== null,
                            clickable: !el.disabled
                        }));
                })()
            "#;
            
            if let Ok(result) = self.browser.execute_script(input_script).await {
                if let Ok(inputs) = serde_json::from_value::<Vec<serde_json::Value>>(result) {
                    for input in inputs {
                        if let Ok(element) = self.create_perceived_element_from_json(input, ElementType::Input).await {
                            elements.push(element);
                        }
                    }
                }
            }
        }

        Ok(elements)
    }

    async fn find_by_text_content(&self, description: &str) -> Result<Vec<PerceivedElement>> {
        let mut elements = Vec::new();

        // Extract meaningful words from description
        let words: Vec<&str> = description.split_whitespace()
            .filter(|w| !["the", "a", "an", "click", "on", "button", "link"].contains(w))
            .collect();

        if words.is_empty() {
            return Ok(elements);
        }

        let search_text = words.join(" ");
        
        let text_search_script = format!(r#"
            const searchText = '{}';
            const results = [];
            
            // Find elements containing the text
            const walker = document.createTreeWalker(
                document.body,
                NodeFilter.SHOW_ELEMENT,
                null,
                false
            );
            
            let node;
            while (node = walker.nextNode()) {{
                const text = node.textContent?.trim().toLowerCase() || '';
                if (text.includes(searchText.toLowerCase()) && text.length < 200) {{
                    results.push({{
                        selector: node.tagName.toLowerCase() + (node.id ? '#' + node.id : ''),
                        text: node.textContent?.trim() || '',
                        type: node.tagName.toLowerCase(),
                        visible: node.offsetParent !== null,
                        clickable: ['a', 'button', 'input'].includes(node.tagName.toLowerCase())
                    }});
                }}
            }}
            
            return results.slice(0, 10); // Limit results
        "#, search_text);

        if let Ok(result) = self.browser.execute_script(&text_search_script).await {
            if let Ok(text_elements) = serde_json::from_value::<Vec<serde_json::Value>>(result) {
                for elem in text_elements {
                    if let Ok(element) = self.create_perceived_element_from_json(elem, ElementType::Unknown).await {
                        elements.push(element);
                    }
                }
            }
        }

        Ok(elements)
    }

    async fn find_by_ui_patterns(&self, description: &str) -> Result<Vec<PerceivedElement>> {
        let mut elements = Vec::new();

        // Search patterns
        if description.contains("search") {
            let search_patterns = vec![
                "input[type='search']",
                "input[name*='search' i]",
                "input[placeholder*='search' i]",
                "#search",
                ".search-input",
                ".search-box",
                "[aria-label*='search' i]",
            ];

            for pattern in search_patterns {
                if let Ok(text) = self.browser.get_text(pattern).await {
                    if !text.is_empty() {
                        if let Ok(element) = self.create_perceived_element_from_selector(pattern, ElementType::Input).await {
                            elements.push(element);
                            break; // Found a search element
                        }
                    }
                }
            }
        }

        // Login patterns
        if description.contains("login") || description.contains("sign in") {
            let login_selectors = vec![
                "button:contains('Login')",
                "a[href*='login']",
                "#login",
                ".login-button",
                ".sign-in",
            ];

            // Note: CSS :contains() isn't supported in all browsers, so we'll use JavaScript
            let login_script = r#"
                const results = [];
                const buttons = document.querySelectorAll('button, a, input[type="submit"]');
                
                buttons.forEach(btn => {
                    const text = (btn.textContent || btn.value || '').toLowerCase();
                    if (text.includes('login') || text.includes('sign in')) {
                        results.push({
                            selector: btn.tagName.toLowerCase() + (btn.id ? '#' + btn.id : ''),
                            text: btn.textContent?.trim() || btn.value || '',
                            type: btn.tagName.toLowerCase(),
                            visible: btn.offsetParent !== null,
                            clickable: !btn.disabled
                        });
                    }
                });
                
                return results;
            "#;

            if let Ok(result) = self.browser.execute_script(login_script).await {
                if let Ok(login_elements) = serde_json::from_value::<Vec<serde_json::Value>>(result) {
                    for elem in login_elements {
                        if let Ok(element) = self.create_perceived_element_from_json(elem, ElementType::Button).await {
                            elements.push(element);
                        }
                    }
                }
            }
        }

        Ok(elements)
    }

    async fn find_by_accessibility(&self, description: &str) -> Result<Vec<PerceivedElement>> {
        let mut elements = Vec::new();

        let aria_script = format!(r#"
            const searchText = '{}';
            const results = [];
            
            // Find elements with aria-label
            const ariaElements = document.querySelectorAll('[aria-label]');
            ariaElements.forEach(el => {{
                const label = el.getAttribute('aria-label').toLowerCase();
                if (label.includes(searchText.toLowerCase())) {{
                    results.push({{
                        selector: el.tagName.toLowerCase() + (el.id ? '#' + el.id : ''),
                        text: el.getAttribute('aria-label'),
                        type: el.tagName.toLowerCase(),
                        visible: el.offsetParent !== null,
                        clickable: ['a', 'button', 'input'].includes(el.tagName.toLowerCase())
                    }});
                }}
            }});
            
            return results;
        "#, description);

        if let Ok(result) = self.browser.execute_script(&aria_script).await {
            if let Ok(aria_elements) = serde_json::from_value::<Vec<serde_json::Value>>(result) {
                for elem in aria_elements {
                    if let Ok(element) = self.create_perceived_element_from_json(elem, ElementType::Unknown).await {
                        elements.push(element);
                    }
                }
            }
        }

        Ok(elements)
    }

    async fn find_by_visual_context(&self, _description: &str) -> Result<Vec<PerceivedElement>> {
        // TODO: Implement visual analysis using the cached screenshot
        // This would involve computer vision to identify elements visually
        // For now, return empty - this is a future enhancement
        Ok(vec![])
    }

    async fn select_best_candidate(&self, candidates: Vec<PerceivedElement>, description: &str) -> Result<PerceivedElement> {
        if candidates.is_empty() {
            return Err(anyhow::anyhow!("No elements found matching: {}", description));
        }

        // Score candidates based on various factors
        let mut scored_candidates: Vec<(f32, PerceivedElement)> = candidates.into_iter()
            .map(|elem| {
                let score = self.calculate_element_score(&elem, description);
                (score, elem)
            })
            .collect();

        // Sort by score (highest first)
        scored_candidates.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        // Return the best candidate
        scored_candidates.into_iter()
            .next()
            .map(|(_, elem)| elem)
            .ok_or_else(|| anyhow::anyhow!("No suitable element found for: {}", description))
    }

    fn calculate_element_score(&self, element: &PerceivedElement, description: &str) -> f32 {
        let mut score = element.confidence;

        // Boost score for visible and clickable elements
        if element.visible {
            score += 0.2;
        }
        if element.clickable {
            score += 0.2;
        }

        // Boost score for text similarity
        let text_similarity = self.calculate_text_similarity(&element.text, description);
        score += text_similarity * 0.3;

        // Element type matching
        if self.element_type_matches(&element.element_type, description) {
            score += 0.2;
        }

        score.min(1.0) // Cap at 1.0
    }

    fn calculate_text_similarity(&self, element_text: &str, description: &str) -> f32 {
        let elem_lower = element_text.to_lowercase();
        let desc_lower = description.to_lowercase();

        if elem_lower.contains(&desc_lower) || desc_lower.contains(&elem_lower) {
            return 0.8;
        }

        // Check word overlap
        let elem_words: std::collections::HashSet<&str> = elem_lower.split_whitespace().collect();
        let desc_words: std::collections::HashSet<&str> = desc_lower.split_whitespace().collect();
        
        let intersection = elem_words.intersection(&desc_words).count();
        let union = elem_words.union(&desc_words).count();

        if union > 0 {
            intersection as f32 / union as f32
        } else {
            0.0
        }
    }

    fn element_type_matches(&self, element_type: &ElementType, description: &str) -> bool {
        match element_type {
            ElementType::Button => description.contains("button") || description.contains("click"),
            ElementType::Input => description.contains("input") || description.contains("field") || description.contains("type"),
            ElementType::Link => description.contains("link") || description.contains("go to"),
            ElementType::Select => description.contains("dropdown") || description.contains("select"),
            _ => false,
        }
    }

    // Helper methods for element creation and caching

    async fn create_perceived_element_from_json(&self, json: serde_json::Value, elem_type: ElementType) -> Result<PerceivedElement> {
        let selector = json.get("selector")
            .and_then(|s| s.as_str())
            .unwrap_or("")
            .to_string();
        
        let text = json.get("text")
            .and_then(|s| s.as_str())
            .unwrap_or("")
            .to_string();
        
        let visible = json.get("visible")
            .and_then(|b| b.as_bool())
            .unwrap_or(false);
        
        let clickable = json.get("clickable")
            .and_then(|b| b.as_bool())
            .unwrap_or(false);

        Ok(PerceivedElement {
            selector,
            text,
            element_type: elem_type,
            clickable,
            visible,
            confidence: 0.7, // Default confidence
            attributes: HashMap::new(),
            position: None, // TODO: Extract position from element
            visual_context: None,
        })
    }

    async fn create_perceived_element_from_selector(&self, selector: &str, elem_type: ElementType) -> Result<PerceivedElement> {
        let text = self.browser.get_text(selector).await.unwrap_or_default();
        
        Ok(PerceivedElement {
            selector: selector.to_string(),
            text,
            element_type: elem_type,
            clickable: true, // Assume clickable if we found it by selector
            visible: true,   // Assume visible if we found it
            confidence: 0.8,
            attributes: HashMap::new(),
            position: None,
            visual_context: None,
        })
    }

    fn check_cache(&self, description: &str) -> Option<&CachedElement> {
        self.element_cache.get(description).and_then(|cached| {
            // Check if cache is still valid (within 30 seconds)
            if cached.last_seen.elapsed().as_secs() < 30 {
                Some(cached)
            } else {
                None
            }
        })
    }

    async fn create_perceived_from_cache(&self, cached: &CachedElement) -> Result<PerceivedElement> {
        let text = self.browser.get_text(&cached.selector).await.unwrap_or_default();
        
        Ok(PerceivedElement {
            selector: cached.selector.clone(),
            text,
            element_type: cached.element_type.clone(),
            clickable: true,
            visible: true,
            confidence: cached.confidence,
            attributes: HashMap::new(),
            position: None,
            visual_context: None,
        })
    }

    fn cache_element(&mut self, description: &str, element: &PerceivedElement) {
        self.context.last_element = Some(element.selector.clone());
        
        // Store named references
        if description.starts_with("the ") {
            self.context.named_elements.insert(description.to_string(), element.selector.clone());
        }

        // Cache for performance
        self.element_cache.insert(
            description.to_string(),
            CachedElement {
                selector: element.selector.clone(),
                element_type: element.element_type.clone(),
                last_seen: std::time::Instant::now(),
                confidence: element.confidence,
            }
        );
    }

    async fn resolve_reference(&self, description: &str) -> Result<Option<PerceivedElement>> {
        // Handle pronouns and references
        if description == "it" || description == "that" {
            if let Some(last_selector) = &self.context.last_element {
                return Ok(Some(self.create_perceived_element_from_selector(last_selector, ElementType::Unknown).await?));
            }
        }

        // Handle named elements
        if let Some(selector) = self.context.named_elements.get(description) {
            return Ok(Some(self.create_perceived_element_from_selector(selector, ElementType::Unknown).await?));
        }

        Ok(None)
    }

    // Page classification methods

    async fn classify_by_url_and_content(&self, url: &str) -> Result<PageType> {
        // URL-based classification
        let url_lower = url.to_lowercase();
        
        if url_lower.contains("login") || url_lower.contains("signin") || url_lower.contains("auth") {
            return Ok(PageType::LoginPage);
        }
        
        if url_lower.contains("search") || url_lower.contains("results") {
            return Ok(PageType::SearchResults);
        }
        
        if url_lower.contains("product") || url_lower.contains("item") || url_lower.contains("shop") {
            return Ok(PageType::ProductPage);
        }

        // Content-based classification using JavaScript
        let classification_script = r#"
            const indicators = {
                form: document.querySelectorAll('form').length,
                articles: document.querySelectorAll('article, .article, .post').length,
                products: document.querySelectorAll('[class*="product"], [id*="product"]').length,
                login: document.querySelectorAll('input[type="password"], [class*="login"], [id*="login"]').length,
                search: document.querySelectorAll('input[type="search"], [class*="search"], [id*="search"]').length,
                dashboard: document.querySelectorAll('[class*="dashboard"], [class*="admin"], nav').length
            };
            
            return indicators;
        "#;

        if let Ok(result) = self.browser.execute_script(classification_script).await {
            if let Some(indicators) = result.as_object() {
                let form_count = indicators.get("form").and_then(|v| v.as_u64()).unwrap_or(0);
                let article_count = indicators.get("articles").and_then(|v| v.as_u64()).unwrap_or(0);
                let product_count = indicators.get("products").and_then(|v| v.as_u64()).unwrap_or(0);
                let login_count = indicators.get("login").and_then(|v| v.as_u64()).unwrap_or(0);
                let search_count = indicators.get("search").and_then(|v| v.as_u64()).unwrap_or(0);
                let dashboard_count = indicators.get("dashboard").and_then(|v| v.as_u64()).unwrap_or(0);

                // Determine page type based on indicators
                if login_count > 0 {
                    return Ok(PageType::LoginPage);
                } else if search_count > 2 {
                    return Ok(PageType::SearchResults);
                } else if product_count > 3 {
                    return Ok(PageType::ProductPage);
                } else if article_count > 0 {
                    return Ok(PageType::ArticlePage);
                } else if form_count > 0 {
                    return Ok(PageType::FormPage);
                } else if dashboard_count > 3 {
                    return Ok(PageType::Dashboard);
                }
            }
        }

        Ok(PageType::Unknown)
    }

    // Data extraction methods (simplified for now)
    
    async fn extract_product_data(&self) -> Result<serde_json::Value> {
        // TODO: Implement product data extraction
        Ok(serde_json::json!({
            "type": "product",
            "title": "",
            "price": "",
            "description": "",
            "images": []
        }))
    }

    async fn extract_article_data(&self) -> Result<serde_json::Value> {
        // TODO: Implement article data extraction
        Ok(serde_json::json!({
            "type": "article",
            "title": "",
            "author": "",
            "content": "",
            "published_date": null
        }))
    }

    async fn extract_search_results(&self) -> Result<serde_json::Value> {
        // TODO: Implement search results extraction
        Ok(serde_json::json!({
            "type": "search_results",
            "results": []
        }))
    }

    async fn extract_form_data(&self) -> Result<serde_json::Value> {
        // TODO: Implement form data extraction
        Ok(serde_json::json!({
            "type": "form",
            "fields": []
        }))
    }

    async fn extract_generic_data(&self) -> Result<serde_json::Value> {
        let generic_script = r#"
            return {
                title: document.title,
                headings: Array.from(document.querySelectorAll('h1, h2, h3')).map(h => h.textContent?.trim()).filter(t => t),
                links: Array.from(document.querySelectorAll('a[href]')).slice(0, 10).map(a => ({
                    text: a.textContent?.trim(),
                    href: a.href
                })),
                images: Array.from(document.querySelectorAll('img[src]')).slice(0, 10).map(img => ({
                    src: img.src,
                    alt: img.alt
                }))
            };
        "#;

        let result = self.browser.execute_script(generic_script).await?;
        Ok(result)
    }
}

// Re-export key types
pub use visual::*;
pub use semantic::*;
pub use context_aware::*;
pub use smart_forms::*;
pub use integration::*;