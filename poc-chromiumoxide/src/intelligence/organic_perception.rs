// Organic Perception Engine
// Advanced AI-driven perception that learns and adapts to different websites and contexts

use crate::browser::{BrowserOps, ElementInfo};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info};

/// Organic perception engine for intelligent element detection
#[derive(Debug)]
pub struct OrganicPerceptionEngine {
    perception_models: HashMap<String, PerceptionModel>,
    context_memory: ContextMemory,
    adaptation_rules: Vec<AdaptationRule>,
    learning_enabled: bool,
}

/// Perception model for specific website types or contexts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerceptionModel {
    pub name: String,
    pub domain_patterns: Vec<String>,
    pub element_patterns: HashMap<String, ElementPattern>,
    pub success_rate: f64,
    pub usage_count: u64,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// Pattern for identifying specific types of elements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementPattern {
    pub element_type: String, // "button", "input", "link", etc.
    pub selectors: Vec<String>,
    pub text_patterns: Vec<String>,
    pub context_clues: Vec<String>,
    pub confidence_weight: f64,
    pub success_count: u64,
    pub failure_count: u64,
}

/// Context memory for learning from past interactions
#[derive(Debug, Clone)]
pub struct ContextMemory {
    site_contexts: HashMap<String, SiteContext>,
    #[allow(dead_code)] // Reserved for pattern learning
    global_patterns: Vec<GlobalPattern>,
    max_memory_entries: usize,
}

/// Context information for a specific site
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteContext {
    pub domain: String,
    pub successful_selectors: HashMap<String, f64>, // selector -> success rate
    pub failed_selectors: HashMap<String, u32>,     // selector -> failure count
    pub page_characteristics: PageCharacteristics,
    pub interaction_patterns: Vec<InteractionPattern>,
    pub last_visit: chrono::DateTime<chrono::Utc>,
}

/// Page characteristics for adaptation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageCharacteristics {
    pub is_spa: bool,
    pub uses_jquery: bool,
    pub uses_react: bool,
    pub uses_angular: bool,
    pub has_shadow_dom: bool,
    pub dynamic_loading: bool,
    pub average_load_time: u64,
    pub element_change_frequency: f64,
}

/// Pattern that works across multiple sites
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalPattern {
    pub name: String,
    pub pattern_type: String,
    pub selectors: Vec<String>,
    pub success_rate: f64,
    pub applicable_domains: Vec<String>,
}

/// Interaction pattern learned from successful actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionPattern {
    pub action_sequence: Vec<String>,
    pub context_description: String,
    pub success_rate: f64,
    pub timing_requirements: Option<u64>, // milliseconds to wait
}

/// Adaptation rule for handling different scenarios
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptationRule {
    pub rule_name: String,
    pub condition: String,
    pub adaptation: String,
    pub priority: u32,
}

/// Result of organic perception analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerceptionResult {
    pub elements: Vec<ElementInsight>,
    pub page_complexity: f64, // 0.0 to 1.0
    pub dynamic_elements: u32,
    pub confidence: f64,
    pub processing_time_ms: u64,
}

/// Deep insight about a page element
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementInsight {
    pub selector: String,
    pub element_type: String,
    pub confidence: f64,
    pub context_score: f64,
    pub interaction_likelihood: f64,
    pub visual_prominence: f64,
    pub semantic_meaning: Option<String>,
    pub alternative_selectors: Vec<String>,
    pub predicted_behavior: Option<String>,
    pub risk_factors: Vec<String>,
}

/// Page context for analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageContext {
    pub url: String,
    pub domain: String,
    pub title: String,
    pub html_content: Option<String>,
    pub viewport_info: ViewportInfo,
    pub performance_metrics: Option<PerformanceMetrics>,
}

/// Viewport information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewportInfo {
    pub width: u32,
    pub height: u32,
    pub device_pixel_ratio: f64,
    pub is_mobile: bool,
}

/// Performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub load_time: u64,
    pub dom_content_loaded: u64,
    pub first_paint: u64,
    pub network_requests: u32,
}

impl OrganicPerceptionEngine {
    /// Create new organic perception engine
    pub fn new() -> Self {
        info!("Initializing Organic Perception Engine");

        let mut engine = Self {
            perception_models: HashMap::new(),
            context_memory: ContextMemory {
                site_contexts: HashMap::new(),
                global_patterns: Vec::new(),
                max_memory_entries: 1000,
            },
            adaptation_rules: Vec::new(),
            learning_enabled: true,
        };

        engine.initialize_default_models();
        engine.initialize_adaptation_rules();
        engine
    }

    /// Initialize default perception models
    fn initialize_default_models(&mut self) {
        debug!("Loading default perception models");

        // E-commerce model
        let ecommerce_model = PerceptionModel {
            name: "e-commerce".to_string(),
            domain_patterns: vec![
                "amazon.com".to_string(),
                "ebay.com".to_string(),
                "shop".to_string(),
                "store".to_string(),
            ],
            element_patterns: {
                let mut patterns = HashMap::new();
                patterns.insert(
                    "add_to_cart".to_string(),
                    ElementPattern {
                        element_type: "button".to_string(),
                        selectors: vec![
                            "[data-testid*='add-to-cart']".to_string(),
                            "button[id*='add-to-cart']".to_string(),
                            ".add-to-cart".to_string(),
                            "button:contains('Add to Cart')".to_string(),
                        ],
                        text_patterns: vec![
                            "add to cart".to_string(),
                            "add to bag".to_string(),
                            "buy now".to_string(),
                        ],
                        context_clues: vec![
                            "near price".to_string(),
                            "product page".to_string(),
                            "shopping context".to_string(),
                        ],
                        confidence_weight: 0.9,
                        success_count: 0,
                        failure_count: 0,
                    },
                );
                patterns
            },
            success_rate: 0.85,
            usage_count: 0,
            last_updated: chrono::Utc::now(),
        };

        self.perception_models
            .insert("e-commerce".to_string(), ecommerce_model);

        // Social media model
        let social_model = PerceptionModel {
            name: "social-media".to_string(),
            domain_patterns: vec![
                "facebook.com".to_string(),
                "twitter.com".to_string(),
                "linkedin.com".to_string(),
                "instagram.com".to_string(),
            ],
            element_patterns: {
                let mut patterns = HashMap::new();
                patterns.insert(
                    "like_button".to_string(),
                    ElementPattern {
                        element_type: "button".to_string(),
                        selectors: vec![
                            "[data-testid*='like']".to_string(),
                            ".like-button".to_string(),
                            "button[aria-label*='like']".to_string(),
                        ],
                        text_patterns: vec!["like".to_string(), "♥".to_string(), "👍".to_string()],
                        context_clues: vec![
                            "post context".to_string(),
                            "social interaction".to_string(),
                        ],
                        confidence_weight: 0.8,
                        success_count: 0,
                        failure_count: 0,
                    },
                );
                patterns
            },
            success_rate: 0.8,
            usage_count: 0,
            last_updated: chrono::Utc::now(),
        };

        self.perception_models
            .insert("social-media".to_string(), social_model);

        // Form model
        let form_model = PerceptionModel {
            name: "forms".to_string(),
            domain_patterns: vec!["*".to_string()], // Universal
            element_patterns: {
                let mut patterns = HashMap::new();
                patterns.insert(
                    "submit_button".to_string(),
                    ElementPattern {
                        element_type: "button".to_string(),
                        selectors: vec![
                            "input[type='submit']".to_string(),
                            "button[type='submit']".to_string(),
                            ".submit-btn".to_string(),
                            "button:contains('Submit')".to_string(),
                        ],
                        text_patterns: vec![
                            "submit".to_string(),
                            "send".to_string(),
                            "save".to_string(),
                            "continue".to_string(),
                        ],
                        context_clues: vec![
                            "form context".to_string(),
                            "input fields nearby".to_string(),
                        ],
                        confidence_weight: 0.85,
                        success_count: 0,
                        failure_count: 0,
                    },
                );
                patterns
            },
            success_rate: 0.9,
            usage_count: 0,
            last_updated: chrono::Utc::now(),
        };

        self.perception_models
            .insert("forms".to_string(), form_model);
    }

    /// Initialize adaptation rules
    fn initialize_adaptation_rules(&mut self) {
        self.adaptation_rules = vec![
            AdaptationRule {
                rule_name: "spa_wait_rule".to_string(),
                condition: "is_spa && dynamic_loading".to_string(),
                adaptation: "increase_wait_time".to_string(),
                priority: 1,
            },
            AdaptationRule {
                rule_name: "mobile_viewport_rule".to_string(),
                condition: "is_mobile".to_string(),
                adaptation: "use_touch_events".to_string(),
                priority: 2,
            },
            AdaptationRule {
                rule_name: "slow_network_rule".to_string(),
                condition: "load_time > 5000".to_string(),
                adaptation: "extend_timeouts".to_string(),
                priority: 3,
            },
        ];
    }

    /// Analyze page deeply using organic perception
    pub async fn analyze_page_deeply(
        &mut self,
        page_context: &PageContext,
        browser: &crate::browser::Browser,
    ) -> Result<PerceptionResult> {
        let start_time = std::time::Instant::now();
        info!(
            "Starting deep organic perception analysis for: {}",
            page_context.url
        );

        // 1. Identify applicable models
        let applicable_models = self.get_applicable_models(&page_context.domain);
        debug!(
            "Found {} applicable models for domain: {}",
            applicable_models.len(),
            page_context.domain
        );

        // 2. Get page characteristics
        let page_characteristics = self.analyze_page_characteristics(browser).await?;
        debug!(
            "Page characteristics: SPA={}, Dynamic={}",
            page_characteristics.is_spa, page_characteristics.dynamic_loading
        );

        // 3. Extract elements using multiple perception strategies
        let mut elements = Vec::new();

        // Strategy 1: Model-based perception
        for model in &applicable_models {
            let model_elements = self.apply_perception_model(model, browser).await?;
            elements.extend(model_elements);
        }

        // Strategy 2: Context-aware perception
        let context_elements = self
            .apply_contextual_perception(page_context, browser)
            .await?;
        elements.extend(context_elements);

        // Strategy 3: Learning-based perception (from memory)
        if let Some(site_context) = self.context_memory.site_contexts.get(&page_context.domain) {
            let learned_elements = self.apply_learned_perception(site_context, browser).await?;
            elements.extend(learned_elements);
        }

        // 4. Consolidate and rank elements
        elements = self.consolidate_elements(elements);
        elements.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());

        // 5. Calculate page complexity and other metrics
        let page_complexity = self.calculate_page_complexity(&page_characteristics, elements.len());
        let dynamic_elements = self.count_dynamic_elements(&elements);
        let overall_confidence = self.calculate_overall_confidence(&elements);

        // 6. Update context memory
        if self.learning_enabled {
            self.update_context_memory(page_context, &page_characteristics)
                .await;
        }

        let processing_time = start_time.elapsed().as_millis() as u64;

        info!("Organic perception analysis completed in {}ms. Found {} elements with {:.2} confidence", 
            processing_time, elements.len(), overall_confidence);

        Ok(PerceptionResult {
            elements,
            page_complexity,
            dynamic_elements,
            confidence: overall_confidence,
            processing_time_ms: processing_time,
        })
    }

    /// Get perception models applicable to a domain
    fn get_applicable_models(&self, domain: &str) -> Vec<&PerceptionModel> {
        self.perception_models
            .values()
            .filter(|model| {
                model.domain_patterns.iter().any(|pattern| {
                    if pattern == "*" {
                        true
                    } else {
                        domain.contains(pattern)
                    }
                })
            })
            .collect()
    }

    /// Analyze page characteristics for adaptation
    async fn analyze_page_characteristics(
        &self,
        browser: &crate::browser::Browser,
    ) -> Result<PageCharacteristics> {
        // JavaScript to detect page characteristics
        let js_code = r#"
            (function() {
                const characteristics = {
                    is_spa: false,
                    uses_jquery: typeof jQuery !== 'undefined',
                    uses_react: typeof React !== 'undefined' || 
                               document.querySelector('[data-reactroot]') !== null ||
                               document.querySelector('[data-react-checksum]') !== null,
                    uses_angular: typeof angular !== 'undefined' || 
                                 document.querySelector('[ng-app]') !== null ||
                                 document.querySelector('[data-ng-app]') !== null,
                    has_shadow_dom: document.querySelector('*').attachShadow !== undefined,
                    dynamic_loading: false,
                    average_load_time: performance.timing.loadEventEnd - performance.timing.navigationStart,
                    element_change_frequency: 0.0
                };
                
                // Detect SPA characteristics
                const historyLength = window.history.length;
                const hasHashRouting = window.location.hash.length > 1;
                const hasPushStateRouting = typeof window.history.pushState === 'function';
                characteristics.is_spa = historyLength > 1 || hasHashRouting || hasPushStateRouting;
                
                // Detect dynamic loading
                const observer = new MutationObserver(function(mutations) {
                    characteristics.dynamic_loading = mutations.length > 0;
                });
                
                // Quick mutation check
                observer.observe(document.body, { childList: true, subtree: true });
                setTimeout(() => observer.disconnect(), 1000);
                
                return characteristics;
            })()
        "#;

        let result = browser
            .execute_script(js_code)
            .await
            .unwrap_or(serde_json::Value::Null);

        // Parse JavaScript result or use defaults
        if let Ok(characteristics) = serde_json::from_value::<PageCharacteristics>(result) {
            Ok(characteristics)
        } else {
            // Default characteristics
            Ok(PageCharacteristics {
                is_spa: false,
                uses_jquery: false,
                uses_react: false,
                uses_angular: false,
                has_shadow_dom: false,
                dynamic_loading: false,
                average_load_time: 3000,
                element_change_frequency: 0.1,
            })
        }
    }

    /// Apply perception model to find elements
    async fn apply_perception_model(
        &self,
        model: &PerceptionModel,
        browser: &crate::browser::Browser,
    ) -> Result<Vec<ElementInsight>> {
        let mut elements = Vec::new();

        for (element_type, pattern) in &model.element_patterns {
            for selector in &pattern.selectors {
                if let Ok(nodes) = browser.find_elements(selector).await {
                    for element in nodes {
                        if let Some(insight) = self
                            .create_element_insight(
                                &element,
                                selector,
                                element_type,
                                pattern.confidence_weight,
                            )
                            .await
                        {
                            elements.push(insight);
                        }
                    }
                }
            }
        }

        Ok(elements)
    }

    /// Apply contextual perception based on page context
    async fn apply_contextual_perception(
        &self,
        page_context: &PageContext,
        browser: &crate::browser::Browser,
    ) -> Result<Vec<ElementInsight>> {
        let mut elements = Vec::new();

        // Context-aware selectors based on page type
        let contextual_selectors = if page_context.url.contains("login")
            || page_context.title.to_lowercase().contains("login")
        {
            vec![
                ("input[type='email']", "email_input"),
                ("input[type='password']", "password_input"),
                ("button:contains('Login')", "login_button"),
                ("button:contains('Sign In')", "signin_button"),
            ]
        } else if page_context.url.contains("search")
            || page_context.title.to_lowercase().contains("search")
        {
            vec![
                ("input[type='search']", "search_input"),
                ("input[name*='search']", "search_input"),
                ("button:contains('Search')", "search_button"),
            ]
        } else {
            vec![
                ("button", "generic_button"),
                ("input", "generic_input"),
                ("a", "generic_link"),
            ]
        };

        for (selector, element_type) in contextual_selectors {
            if let Ok(nodes) = browser.find_elements(selector).await {
                for element in nodes.into_iter().take(10) {
                    // Limit to avoid overwhelming results
                    if let Some(insight) = self
                        .create_element_insight(
                            &element,
                            selector,
                            element_type,
                            0.7, // Contextual confidence
                        )
                        .await
                    {
                        elements.push(insight);
                    }
                }
            }
        }

        Ok(elements)
    }

    /// Apply learned perception from site context
    async fn apply_learned_perception(
        &self,
        site_context: &SiteContext,
        browser: &crate::browser::Browser,
    ) -> Result<Vec<ElementInsight>> {
        let mut elements = Vec::new();

        // Use successful selectors from past interactions
        for (selector, success_rate) in &site_context.successful_selectors {
            if *success_rate > 0.6 {
                // Only use relatively successful selectors
                if let Ok(nodes) = browser.find_elements(selector).await {
                    for element in nodes.into_iter().take(5) {
                        if let Some(insight) = self
                            .create_element_insight(&element, selector, "learned", *success_rate)
                            .await
                        {
                            elements.push(insight);
                        }
                    }
                }
            }
        }

        Ok(elements)
    }

    /// Create detailed element insight
    async fn create_element_insight(
        &self,
        element: &ElementInfo,
        selector: &str,
        element_type: &str,
        base_confidence: f64,
    ) -> Option<ElementInsight> {
        // Calculate various scores
        let context_score = self.calculate_context_score(element);
        let interaction_likelihood = self.calculate_interaction_likelihood(element, element_type);
        let visual_prominence = self.calculate_visual_prominence(element);

        // Adjust confidence based on multiple factors
        let adjusted_confidence = base_confidence
            * (context_score * 0.3 + interaction_likelihood * 0.4 + visual_prominence * 0.3);

        // Generate alternative selectors
        let alternative_selectors = self.generate_alternative_selectors(element);

        Some(ElementInsight {
            selector: selector.to_string(),
            element_type: element_type.to_string(),
            confidence: adjusted_confidence,
            context_score,
            interaction_likelihood,
            visual_prominence,
            semantic_meaning: self.extract_semantic_meaning(element),
            alternative_selectors,
            predicted_behavior: self.predict_element_behavior(element, element_type),
            risk_factors: self.identify_risk_factors(element),
        })
    }

    /// Calculate context relevance score
    fn calculate_context_score(&self, element: &ElementInfo) -> f64 {
        // Factors: position on page, surrounding elements, text content
        let mut score = 0.5; // Base score

        // Check if element has meaningful text
        if !element.text.trim().is_empty() {
            score += 0.2;
        }

        // Check for accessibility attributes
        if !element.attributes.is_empty() {
            score += 0.1;
        }

        f64::min(score, 1.0)
    }

    /// Calculate likelihood of user interaction
    fn calculate_interaction_likelihood(&self, _element: &ElementInfo, element_type: &str) -> f64 {
        match element_type {
            "button" | "submit_button" => 0.9,
            "input" | "email_input" | "password_input" => 0.8,
            "link" => 0.7,
            "select" => 0.7,
            "checkbox" => 0.6,
            _ => 0.4,
        }
    }

    /// Calculate visual prominence score
    fn calculate_visual_prominence(&self, _element: &ElementInfo) -> f64 {
        // This would ideally use computed styles, dimensions, color contrast, etc.
        // For now, return a default value
        0.6
    }

    /// Generate alternative selectors for robustness
    fn generate_alternative_selectors(&self, element: &ElementInfo) -> Vec<String> {
        let mut alternatives = Vec::new();

        // Generate alternatives based on element properties
        alternatives.push(element.tag_name.to_lowercase());

        if let Some(id) = element.attributes.get("id") {
            alternatives.push(format!("#{}", id));
        }

        // Add more sophisticated selector generation here
        alternatives
    }

    /// Extract semantic meaning from element
    fn extract_semantic_meaning(&self, element: &ElementInfo) -> Option<String> {
        // Analyze text content, attributes, and context to understand semantic meaning
        Some(element.text.clone())
    }

    /// Predict element behavior when interacted with
    fn predict_element_behavior(
        &self,
        _element: &ElementInfo,
        element_type: &str,
    ) -> Option<String> {
        match element_type {
            "button" | "submit_button" => Some("Will submit form or trigger action".to_string()),
            "link" => Some("Will navigate to new page".to_string()),
            "input" => Some("Will accept user input".to_string()),
            _ => None,
        }
    }

    /// Identify potential risk factors
    fn identify_risk_factors(&self, element: &ElementInfo) -> Vec<String> {
        let mut risks = Vec::new();

        // Check for dynamic content indicators
        for key in element.attributes.keys() {
            if key.to_lowercase().contains("data-") || key.to_lowercase().contains("ng-") {
                risks.push("Dynamic content - may change".to_string());
                break;
            }
        }

        risks
    }

    /// Consolidate duplicate or similar elements
    fn consolidate_elements(&self, mut elements: Vec<ElementInsight>) -> Vec<ElementInsight> {
        // Remove duplicates and merge similar elements
        elements.dedup_by(|a, b| a.selector == b.selector);

        // Could add more sophisticated consolidation logic here
        elements
    }

    /// Calculate overall page complexity
    fn calculate_page_complexity(
        &self,
        characteristics: &PageCharacteristics,
        element_count: usize,
    ) -> f64 {
        let mut complexity = 0.0;

        // Base complexity from element count
        complexity += (element_count as f64 / 100.0).min(0.4);

        // Add complexity based on page characteristics
        if characteristics.is_spa {
            complexity += 0.2;
        }
        if characteristics.dynamic_loading {
            complexity += 0.2;
        }
        if characteristics.has_shadow_dom {
            complexity += 0.1;
        }
        if characteristics.uses_react || characteristics.uses_angular {
            complexity += 0.1;
        }

        complexity.min(1.0)
    }

    /// Count dynamic elements
    fn count_dynamic_elements(&self, elements: &[ElementInsight]) -> u32 {
        elements
            .iter()
            .filter(|e| e.risk_factors.iter().any(|r| r.contains("Dynamic")))
            .count() as u32
    }

    /// Calculate overall confidence
    fn calculate_overall_confidence(&self, elements: &[ElementInsight]) -> f64 {
        if elements.is_empty() {
            0.0
        } else {
            elements.iter().map(|e| e.confidence).sum::<f64>() / elements.len() as f64
        }
    }

    /// Update context memory with learning
    async fn update_context_memory(
        &mut self,
        page_context: &PageContext,
        characteristics: &PageCharacteristics,
    ) {
        let domain = page_context.domain.clone();

        let site_context = self
            .context_memory
            .site_contexts
            .entry(domain.clone())
            .or_insert_with(|| SiteContext {
                domain: domain.clone(),
                successful_selectors: HashMap::new(),
                failed_selectors: HashMap::new(),
                page_characteristics: characteristics.clone(),
                interaction_patterns: Vec::new(),
                last_visit: chrono::Utc::now(),
            });

        site_context.last_visit = chrono::Utc::now();
        site_context.page_characteristics = characteristics.clone();

        // Cleanup old entries if memory limit exceeded
        if self.context_memory.site_contexts.len() > self.context_memory.max_memory_entries {
            // Remove oldest entries (simple cleanup strategy)
            let oldest_domains: Vec<String> = self
                .context_memory
                .site_contexts
                .iter()
                .map(|(domain, context)| (domain.clone(), context.last_visit))
                .collect::<Vec<_>>()
                .into_iter()
                .take(self.context_memory.max_memory_entries / 10) // Remove 10%
                .map(|(domain, _)| domain)
                .collect();

            for domain in oldest_domains {
                self.context_memory.site_contexts.remove(&domain);
            }
        }
    }

    /// Record successful selector for learning
    pub async fn record_success(&mut self, domain: &str, selector: &str) {
        if let Some(site_context) = self.context_memory.site_contexts.get_mut(domain) {
            let success_rate = site_context
                .successful_selectors
                .entry(selector.to_string())
                .or_insert(0.0);
            *success_rate = (*success_rate * 0.8) + 0.2; // Weighted update
        }
    }

    /// Record failed selector for learning
    pub async fn record_failure(&mut self, domain: &str, selector: &str) {
        if let Some(site_context) = self.context_memory.site_contexts.get_mut(domain) {
            let failure_count = site_context
                .failed_selectors
                .entry(selector.to_string())
                .or_insert(0);
            *failure_count += 1;

            // Decrease success rate for this selector
            if let Some(success_rate) = site_context.successful_selectors.get_mut(selector) {
                *success_rate = (*success_rate * 0.9).max(0.1); // Don't go to zero completely
            }
        }
    }
}

impl Default for OrganicPerceptionEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_organic_perception_engine_creation() {
        let engine = OrganicPerceptionEngine::new();
        assert_eq!(engine.perception_models.len(), 3); // Default models
        assert!(engine.learning_enabled);
    }

    #[test]
    fn test_applicable_models() {
        let engine = OrganicPerceptionEngine::new();

        let models = engine.get_applicable_models("amazon.com");
        assert!(!models.is_empty());

        let ecommerce_model = models.iter().find(|m| m.name == "e-commerce");
        assert!(ecommerce_model.is_some());
    }

    #[test]
    fn test_page_complexity_calculation() {
        let engine = OrganicPerceptionEngine::new();

        let characteristics = PageCharacteristics {
            is_spa: true,
            dynamic_loading: true,
            has_shadow_dom: false,
            uses_react: true,
            uses_angular: false,
            uses_jquery: false,
            average_load_time: 2000,
            element_change_frequency: 0.5,
        };

        let complexity = engine.calculate_page_complexity(&characteristics, 50);
        assert!(complexity > 0.5); // Should be high due to SPA + dynamic loading
    }
}
