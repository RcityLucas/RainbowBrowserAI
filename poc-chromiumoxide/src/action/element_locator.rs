// Advanced Element Location Strategies
// Part of the Intelligent Action Engine

use crate::error::{Result, RainbowError};
use crate::action::{ActionTarget, BoundingBox};
use chromiumoxide::{Page, Element};
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use std::time::Duration;
use std::collections::HashMap;

/// Advanced element locator with multiple fallback strategies
#[derive(Debug)]
pub struct ElementLocator {
    strategies: Vec<Box<dyn LocationStrategy + Send + Sync>>,
    cache: tokio::sync::RwLock<HashMap<String, CachedElement>>,
}

#[derive(Debug, Clone)]
struct CachedElement {
    element_id: String,
    last_seen: std::time::Instant,
    bounding_box: Option<BoundingBox>,
    attributes: HashMap<String, String>,
}

impl ElementLocator {
    pub fn new() -> Self {
        Self {
            strategies: vec![
                Box::new(SelectorStrategy),
                Box::new(XPathStrategy),
                Box::new(TextContentStrategy),
                Box::new(AttributeStrategy),
                Box::new(SemanticStrategy),
                Box::new(VisualStrategy),
                Box::new(FuzzyMatchStrategy),
            ],
            cache: tokio::sync::RwLock::new(HashMap::new()),
        }
    }

    /// Locate element using multiple strategies with intelligent fallbacks
    pub async fn locate_element(
        &self,
        page: Arc<Page>,
        target: &ActionTarget,
    ) -> Result<Element> {
        // Try cache first
        if let Some(cached) = self.try_from_cache(page.clone(), target).await? {
            return Ok(cached);
        }

        let mut last_error = None;

        // Try each strategy in order of reliability
        for strategy in &self.strategies {
            if !strategy.can_handle(target) {
                continue;
            }

            match strategy.locate(page.clone(), target).await {
                Ok(element) => {
                    // Cache the successful result
                    self.cache_element(target, &element).await?;
                    return Ok(element);
                }
                Err(e) => {
                    last_error = Some(e);
                    // Log strategy failure but continue to next
                    tracing::debug!("Strategy {} failed: {:?}", strategy.name(), last_error);
                }
            }
        }

        // If all strategies failed, try intelligent fallbacks
        if let Ok(element) = self.try_intelligent_fallback(page.clone(), target).await {
            return Ok(element);
        }

        Err(last_error.unwrap_or_else(|| {
            RainbowError::ElementNotFound(format!("No strategy could locate target: {:?}", target))
        }))
    }

    /// Locate multiple elements matching the target
    pub async fn locate_elements(
        &self,
        page: Arc<Page>,
        target: &ActionTarget,
    ) -> Result<Vec<Element>> {
        let mut results = Vec::new();

        for strategy in &self.strategies {
            if !strategy.can_handle(target) {
                continue;
            }

            if let Ok(elements) = strategy.locate_multiple(page.clone(), target).await {
                results.extend(elements);
                break; // Use first successful strategy
            }
        }

        if results.is_empty() {
            return Err(RainbowError::ElementNotFound(
                format!("No elements found for target: {:?}", target)
            ));
        }

        Ok(results)
    }

    /// Find the most interactable element from multiple candidates
    pub async fn find_best_element(
        &self,
        page: Arc<Page>,
        elements: Vec<Element>,
    ) -> Result<Element> {
        if elements.is_empty() {
            return Err(RainbowError::ElementNotFound("No elements to choose from".to_string()));
        }

        if elements.len() == 1 {
            return Ok(elements.into_iter().next().unwrap());
        }

        // Score elements based on multiple criteria
        let mut scored_elements = Vec::new();

        for element in elements {
            let score = self.score_element(page.clone(), &element).await?;
            scored_elements.push((element, score));
        }

        // Sort by score (highest first)
        scored_elements.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        Ok(scored_elements.into_iter().next().unwrap().0)
    }

    async fn try_from_cache(
        &self,
        page: Arc<Page>,
        target: &ActionTarget,
    ) -> Result<Option<Element>> {
        let cache_key = self.generate_cache_key(target);
        let cache = self.cache.read().await;

        if let Some(cached) = cache.get(&cache_key) {
            // Check if cache is still valid (5 seconds)
            if cached.last_seen.elapsed() < Duration::from_secs(5) {
                // Try to get the element by its cached ID
                if let Ok(element) = page.find_element(&format!("*[data-rainbow-id='{}']", cached.element_id)).await {
                    return Ok(Some(element));
                }
            }
        }

        Ok(None)
    }

    async fn cache_element(&self, target: &ActionTarget, element: &Element) -> Result<()> {
        let cache_key = self.generate_cache_key(target);
        let element_id = uuid::Uuid::new_v4().to_string();

        // Add a custom attribute to track this element
        let js_code = format!(
            "arguments[0].setAttribute('data-rainbow-id', '{}');",
            element_id
        );
        let _ = element.call_js_fn(&js_code, vec![]).await;

        // Get element attributes
        let mut attributes = HashMap::new();
        if let Ok(tag_name) = element.tag_name().await {
            attributes.insert("tagName".to_string(), tag_name);
        }

        // Get bounding box
        let bounding_box = self.get_element_bounds(element).await.ok();

        let cached_element = CachedElement {
            element_id,
            last_seen: std::time::Instant::now(),
            bounding_box,
            attributes,
        };

        let mut cache = self.cache.write().await;
        cache.insert(cache_key, cached_element);

        Ok(())
    }

    async fn try_intelligent_fallback(
        &self,
        page: Arc<Page>,
        target: &ActionTarget,
    ) -> Result<Element> {
        // Try semantic fallbacks based on target type
        match target {
            ActionTarget::Text(text) => {
                // Try finding by aria-label, title, or placeholder
                let fallback_selectors = vec![
                    format!("[aria-label*='{}']", text),
                    format!("[title*='{}']", text),
                    format!("[placeholder*='{}']", text),
                    format!("[alt*='{}']", text),
                ];

                for selector in fallback_selectors {
                    if let Ok(element) = page.find_element(&selector).await {
                        return Ok(element);
                    }
                }
            }
            ActionTarget::Selector(selector) => {
                // Try CSS selector variations
                let variations = self.generate_selector_variations(selector);
                for variant in variations {
                    if let Ok(element) = page.find_element(&variant).await {
                        return Ok(element);
                    }
                }
            }
            _ => {}
        }

        Err(RainbowError::ElementNotFound("All fallback strategies failed".to_string()))
    }

    async fn score_element(&self, page: Arc<Page>, element: &Element) -> Result<f64> {
        let mut score = 0.0;

        // Visibility score (highest priority)
        if let Ok(is_visible) = self.is_element_visible(page.clone(), element).await {
            score += if is_visible { 50.0 } else { 0.0 };
        }

        // Interactability score
        if let Ok(is_enabled) = self.is_element_enabled(element).await {
            score += if is_enabled { 30.0 } else { 0.0 };
        }

        // Size score (prefer reasonably sized elements)
        if let Ok(bounds) = self.get_element_bounds(element).await {
            let area = bounds.width * bounds.height;
            if area > 100.0 && area < 100000.0 {
                score += 10.0;
            }
        }

        // Position score (prefer elements in the main content area)
        if let Ok(bounds) = self.get_element_bounds(element).await {
            if bounds.y > 100.0 && bounds.y < 800.0 {
                score += 10.0;
            }
        }

        Ok(score)
    }

    async fn is_element_visible(&self, page: Arc<Page>, element: &Element) -> Result<bool> {
        let js_code = r#"
            function isElementVisible(element) {
                if (!element) return false;
                
                const rect = element.getBoundingClientRect();
                const style = window.getComputedStyle(element);
                
                return rect.width > 0 && 
                       rect.height > 0 && 
                       style.visibility !== 'hidden' && 
                       style.display !== 'none' &&
                       style.opacity !== '0';
            }
            return isElementVisible(arguments[0]);
        "#;

        let result = element.call_js_fn(js_code, vec![]).await?;
        Ok(result.as_bool().unwrap_or(false))
    }

    async fn is_element_enabled(&self, element: &Element) -> Result<bool> {
        let js_code = r#"
            const element = arguments[0];
            return !element.disabled && 
                   !element.hasAttribute('disabled') &&
                   !element.getAttribute('aria-disabled');
        "#;

        let result = element.call_js_fn(js_code, vec![]).await?;
        Ok(result.as_bool().unwrap_or(true))
    }

    async fn get_element_bounds(&self, element: &Element) -> Result<BoundingBox> {
        let js_code = r#"
            const rect = arguments[0].getBoundingClientRect();
            return {
                x: rect.x,
                y: rect.y,
                width: rect.width,
                height: rect.height
            };
        "#;

        let result = element.call_js_fn(js_code, vec![]).await?;
        let bounds_obj = result.as_object().ok_or_else(|| {
            RainbowError::ExecutionError("Failed to get bounding box".to_string())
        })?;

        Ok(BoundingBox {
            x: bounds_obj.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0),
            y: bounds_obj.get("y").and_then(|v| v.as_f64()).unwrap_or(0.0),
            width: bounds_obj.get("width").and_then(|v| v.as_f64()).unwrap_or(0.0),
            height: bounds_obj.get("height").and_then(|v| v.as_f64()).unwrap_or(0.0),
        })
    }

    fn generate_cache_key(&self, target: &ActionTarget) -> String {
        format!("{:?}", target)
    }

    fn generate_selector_variations(&self, selector: &str) -> Vec<String> {
        let mut variations = vec![selector.to_string()];

        // Add case-insensitive variations
        if selector.contains("[") && selector.contains("]") {
            let case_insensitive = selector.replace("]", " i]");
            variations.push(case_insensitive);
        }

        // Add :visible pseudo-selector
        variations.push(format!("{}:visible", selector));

        // Add :enabled pseudo-selector for interactive elements
        if selector.contains("input") || selector.contains("button") {
            variations.push(format!("{}:enabled", selector));
        }

        variations
    }
}

impl Default for ElementLocator {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for different element location strategies
#[async_trait::async_trait]
trait LocationStrategy: std::fmt::Debug {
    fn name(&self) -> &'static str;
    fn can_handle(&self, target: &ActionTarget) -> bool;
    
    async fn locate(&self, page: Arc<Page>, target: &ActionTarget) -> Result<Element>;
    
    async fn locate_multiple(&self, page: Arc<Page>, target: &ActionTarget) -> Result<Vec<Element>> {
        // Default implementation returns single element as vector
        match self.locate(page, target).await {
            Ok(element) => Ok(vec![element]),
            Err(e) => Err(e),
        }
    }
}

/// CSS Selector strategy
#[derive(Debug)]
struct SelectorStrategy;

#[async_trait::async_trait]
impl LocationStrategy for SelectorStrategy {
    fn name(&self) -> &'static str { "SelectorStrategy" }
    
    fn can_handle(&self, target: &ActionTarget) -> bool {
        matches!(target, ActionTarget::Selector(_))
    }
    
    async fn locate(&self, page: Arc<Page>, target: &ActionTarget) -> Result<Element> {
        if let ActionTarget::Selector(selector) = target {
            page.find_element(selector).await
                .map_err(|e| RainbowError::ElementNotFound(e.to_string()))
        } else {
            Err(RainbowError::ElementNotFound("Invalid target type for selector strategy".to_string()))
        }
    }

    async fn locate_multiple(&self, page: Arc<Page>, target: &ActionTarget) -> Result<Vec<Element>> {
        if let ActionTarget::Selector(selector) = target {
            page.find_elements(selector).await
                .map_err(|e| RainbowError::ElementNotFound(e.to_string()))
        } else {
            Err(RainbowError::ElementNotFound("Invalid target type for selector strategy".to_string()))
        }
    }
}

/// XPath strategy
#[derive(Debug)]
struct XPathStrategy;

#[async_trait::async_trait]
impl LocationStrategy for XPathStrategy {
    fn name(&self) -> &'static str { "XPathStrategy" }
    
    fn can_handle(&self, target: &ActionTarget) -> bool {
        matches!(target, ActionTarget::XPath(_))
    }
    
    async fn locate(&self, page: Arc<Page>, target: &ActionTarget) -> Result<Element> {
        if let ActionTarget::XPath(xpath) = target {
            // XPath support through JavaScript evaluation
            let js_code = format!(r#"
                const result = document.evaluate(
                    '{}', 
                    document, 
                    null, 
                    XPathResult.FIRST_ORDERED_NODE_TYPE, 
                    null
                );
                return result.singleNodeValue;
            "#, xpath);

            let result = page.evaluate(&js_code).await?;
            if result.is_null() {
                return Err(RainbowError::ElementNotFound(format!("XPath not found: {}", xpath)));
            }

            // Convert result to element (this would need proper implementation)
            // For now, fall back to a CSS selector equivalent if possible
            Err(RainbowError::ElementNotFound("XPath strategy needs proper implementation".to_string()))
        } else {
            Err(RainbowError::ElementNotFound("Invalid target type for XPath strategy".to_string()))
        }
    }
}

/// Text content strategy
#[derive(Debug)]
struct TextContentStrategy;

#[async_trait::async_trait]
impl LocationStrategy for TextContentStrategy {
    fn name(&self) -> &'static str { "TextContentStrategy" }
    
    fn can_handle(&self, target: &ActionTarget) -> bool {
        matches!(target, ActionTarget::Text(_))
    }
    
    async fn locate(&self, page: Arc<Page>, target: &ActionTarget) -> Result<Element> {
        if let ActionTarget::Text(text) = target {
            let js_code = format!(r#"
                function findByText(text) {{
                    const walker = document.createTreeWalker(
                        document.body,
                        NodeFilter.SHOW_ELEMENT,
                        {{
                            acceptNode: function(node) {{
                                return node.textContent && 
                                       node.textContent.trim().includes('{}') 
                                       ? NodeFilter.FILTER_ACCEPT 
                                       : NodeFilter.FILTER_SKIP;
                            }}
                        }}
                    );
                    
                    const elements = [];
                    let node;
                    while (node = walker.nextNode()) {{
                        // Prefer exact matches
                        if (node.textContent.trim() === '{}') {{
                            return node;
                        }}
                        elements.push(node);
                    }}
                    
                    // Return first partial match if no exact match
                    return elements.length > 0 ? elements[0] : null;
                }}
                return findByText('{}');
            "#, text, text, text);

            let result = page.evaluate(&js_code).await?;
            if result.is_null() {
                return Err(RainbowError::ElementNotFound(format!("Text not found: {}", text)));
            }

            // This would need proper element conversion
            // For now, try common selectors that might contain the text
            let selector = format!("*:contains('{}')", text);
            page.find_element(&selector).await
                .map_err(|e| RainbowError::ElementNotFound(e.to_string()))
        } else {
            Err(RainbowError::ElementNotFound("Invalid target type for text strategy".to_string()))
        }
    }
}

/// Attribute-based strategy
#[derive(Debug)]
struct AttributeStrategy;

#[async_trait::async_trait]
impl LocationStrategy for AttributeStrategy {
    fn name(&self) -> &'static str { "AttributeStrategy" }
    
    fn can_handle(&self, target: &ActionTarget) -> bool {
        matches!(target, 
            ActionTarget::Id(_) | 
            ActionTarget::Class(_) | 
            ActionTarget::Name(_) |
            ActionTarget::Placeholder(_) |
            ActionTarget::Value(_) |
            ActionTarget::Role(_)
        )
    }
    
    async fn locate(&self, page: Arc<Page>, target: &ActionTarget) -> Result<Element> {
        let selector = match target {
            ActionTarget::Id(id) => format!("#{}", id),
            ActionTarget::Class(class) => format!(".{}", class),
            ActionTarget::Name(name) => format!("[name='{}']", name),
            ActionTarget::Placeholder(placeholder) => format!("[placeholder='{}']", placeholder),
            ActionTarget::Value(value) => format!("[value='{}']", value),
            ActionTarget::Role(role) => format!("[role='{}']", role),
            _ => return Err(RainbowError::ElementNotFound("Invalid target type".to_string())),
        };

        page.find_element(&selector).await
            .map_err(|e| RainbowError::ElementNotFound(e.to_string()))
    }
}

/// Semantic strategy (ARIA, roles, etc.)
#[derive(Debug)]
struct SemanticStrategy;

#[async_trait::async_trait]
impl LocationStrategy for SemanticStrategy {
    fn name(&self) -> &'static str { "SemanticStrategy" }
    
    fn can_handle(&self, target: &ActionTarget) -> bool {
        // Can handle any target by looking for semantic equivalents
        true
    }
    
    async fn locate(&self, page: Arc<Page>, target: &ActionTarget) -> Result<Element> {
        let semantic_selectors = match target {
            ActionTarget::Text(text) => vec![
                format!("[aria-label='{}']", text),
                format("[aria-labelledby*='{}']", text),
                format!("button:contains('{}')", text),
                format!("a:contains('{}')", text),
            ],
            ActionTarget::Role(role) => vec![
                format!("[role='{}']", role),
                format!("[aria-role='{}']", role),
            ],
            _ => vec![],
        };

        for selector in semantic_selectors {
            if let Ok(element) = page.find_element(&selector).await {
                return Ok(element);
            }
        }

        Err(RainbowError::ElementNotFound("No semantic match found".to_string()))
    }
}

/// Visual strategy (coordinates, bounding boxes)
#[derive(Debug)]
struct VisualStrategy;

#[async_trait::async_trait]
impl LocationStrategy for VisualStrategy {
    fn name(&self) -> &'static str { "VisualStrategy" }
    
    fn can_handle(&self, target: &ActionTarget) -> bool {
        matches!(target, ActionTarget::Coordinate { .. })
    }
    
    async fn locate(&self, page: Arc<Page>, target: &ActionTarget) -> Result<Element> {
        if let ActionTarget::Coordinate { x, y } = target {
            let js_code = format!(r#"
                return document.elementFromPoint({}, {});
            "#, x, y);

            let result = page.evaluate(&js_code).await?;
            if result.is_null() {
                return Err(RainbowError::ElementNotFound(
                    format!("No element at coordinates ({}, {})", x, y)
                ));
            }

            // This would need proper element conversion
            Err(RainbowError::ElementNotFound("Visual strategy needs proper implementation".to_string()))
        } else {
            Err(RainbowError::ElementNotFound("Invalid target type for visual strategy".to_string()))
        }
    }
}

/// Fuzzy matching strategy
#[derive(Debug)]
struct FuzzyMatchStrategy;

#[async_trait::async_trait]
impl LocationStrategy for FuzzyMatchStrategy {
    fn name(&self) -> &'static str { "FuzzyMatchStrategy" }
    
    fn can_handle(&self, _target: &ActionTarget) -> bool {
        true // Last resort strategy
    }
    
    async fn locate(&self, page: Arc<Page>, target: &ActionTarget) -> Result<Element> {
        // Implement fuzzy matching logic
        // This could use string similarity algorithms like Levenshtein distance
        // For now, return an error as it's a complex implementation
        Err(RainbowError::ElementNotFound("Fuzzy matching not yet implemented".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_element_locator_creation() {
        let locator = ElementLocator::new();
        assert_eq!(locator.strategies.len(), 7);
    }

    #[test]
    fn test_strategy_can_handle() {
        let selector_strategy = SelectorStrategy;
        assert!(selector_strategy.can_handle(&ActionTarget::Selector("#test".to_string())));
        assert!(!selector_strategy.can_handle(&ActionTarget::Id("test".to_string())));

        let attribute_strategy = AttributeStrategy;
        assert!(attribute_strategy.can_handle(&ActionTarget::Id("test".to_string())));
        assert!(!attribute_strategy.can_handle(&ActionTarget::Selector("#test".to_string())));
    }

    #[test]
    fn test_cache_key_generation() {
        let locator = ElementLocator::new();
        let target = ActionTarget::Selector("#test".to_string());
        let key = locator.generate_cache_key(&target);
        assert!(!key.is_empty());
    }
}