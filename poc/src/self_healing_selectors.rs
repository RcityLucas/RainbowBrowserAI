// Self-Healing Selector System
// Automatically adapts to UI changes and finds alternative selectors

use std::collections::HashMap;
use anyhow::Result;
use serde::{Serialize, Deserialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Self-healing selector system that adapts to UI changes
pub struct SelfHealingSelectors {
    selector_history: HashMap<String, SelectorHistory>,
    fallback_strategies: Vec<FallbackStrategy>,
    learning_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectorHistory {
    pub primary_selector: String,
    pub alternative_selectors: Vec<String>,
    pub success_count: u32,
    pub failure_count: u32,
    pub last_success: Option<u64>,
    pub last_failure: Option<u64>,
    pub confidence_score: f32,
}

#[derive(Debug, Clone)]
pub struct FallbackStrategy {
    pub name: String,
    pub generator: fn(&str) -> Vec<String>,
    pub priority: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealedSelector {
    pub original: String,
    pub healed: String,
    pub strategy_used: String,
    pub confidence: f32,
    pub alternatives: Vec<String>,
}

impl SelfHealingSelectors {
    pub fn new() -> Self {
        Self {
            selector_history: HashMap::new(),
            fallback_strategies: Self::initialize_strategies(),
            learning_enabled: true,
        }
    }
    
    fn initialize_strategies() -> Vec<FallbackStrategy> {
        vec![
            FallbackStrategy {
                name: "ID-based".to_string(),
                generator: Self::generate_id_selectors,
                priority: 1,
            },
            FallbackStrategy {
                name: "Class-based".to_string(),
                generator: Self::generate_class_selectors,
                priority: 2,
            },
            FallbackStrategy {
                name: "Attribute-based".to_string(),
                generator: Self::generate_attribute_selectors,
                priority: 3,
            },
            FallbackStrategy {
                name: "Text-based".to_string(),
                generator: Self::generate_text_selectors,
                priority: 4,
            },
            FallbackStrategy {
                name: "Position-based".to_string(),
                generator: Self::generate_position_selectors,
                priority: 5,
            },
            FallbackStrategy {
                name: "Parent-child".to_string(),
                generator: Self::generate_parent_child_selectors,
                priority: 6,
            },
            FallbackStrategy {
                name: "Sibling-based".to_string(),
                generator: Self::generate_sibling_selectors,
                priority: 7,
            },
        ]
    }
    
    /// Heal a broken selector
    pub fn heal_selector(&mut self, broken_selector: &str, context: &str) -> Result<HealedSelector> {
        // Check if we have history for this selector
        if let Some(history) = self.selector_history.get(broken_selector) {
            if history.confidence_score > 0.8 && !history.alternative_selectors.is_empty() {
                // Try alternatives from history first
                for alt in &history.alternative_selectors {
                    if self.validate_selector(alt) {
                        self.record_success(broken_selector, alt);
                        return Ok(HealedSelector {
                            original: broken_selector.to_string(),
                            healed: alt.clone(),
                            strategy_used: "History".to_string(),
                            confidence: history.confidence_score,
                            alternatives: history.alternative_selectors.clone(),
                        });
                    }
                }
            }
        }
        
        // Generate new alternatives using fallback strategies
        let mut all_alternatives = Vec::new();
        let mut best_selector = None;
        let mut best_strategy = String::new();
        let mut best_confidence = 0.0;
        
        for strategy in &self.fallback_strategies {
            let alternatives = (strategy.generator)(context);
            
            for selector in &alternatives {
                if self.validate_selector(selector) {
                    let confidence = self.calculate_confidence(selector, &strategy.name);
                    
                    if confidence > best_confidence {
                        best_confidence = confidence;
                        best_selector = Some(selector.clone());
                        best_strategy = strategy.name.clone();
                    }
                    
                    all_alternatives.push(selector.clone());
                }
            }
            
            // Early exit if we found a high-confidence selector
            if best_confidence > 0.9 {
                break;
            }
        }
        
        if let Some(healed) = best_selector {
            // Learn from this healing
            if self.learning_enabled {
                self.learn_selector_mapping(broken_selector, &healed, &all_alternatives);
            }
            
            Ok(HealedSelector {
                original: broken_selector.to_string(),
                healed: healed.clone(),
                strategy_used: best_strategy,
                confidence: best_confidence,
                alternatives: all_alternatives,
            })
        } else {
            // Last resort: generate generic selectors
            let generic = self.generate_generic_selector(context);
            Ok(HealedSelector {
                original: broken_selector.to_string(),
                healed: generic.clone(),
                strategy_used: "Generic".to_string(),
                confidence: 0.3,
                alternatives: vec![generic],
            })
        }
    }
    
    /// Validate if a selector would work (simplified for demo)
    fn validate_selector(&self, selector: &str) -> bool {
        // In real implementation, this would test against the actual DOM
        // For now, just check if it's a valid selector syntax
        !selector.is_empty() && 
        !selector.contains("undefined") &&
        !selector.contains("null") &&
        selector.len() < 200
    }
    
    /// Calculate confidence score for a selector
    fn calculate_confidence(&self, selector: &str, strategy: &str) -> f32 {
        let mut confidence = 0.5;
        
        // Boost confidence based on selector specificity
        if selector.starts_with("#") {
            confidence += 0.3; // ID selectors are most specific
        } else if selector.contains("[data-test") {
            confidence += 0.25; // Test attributes are reliable
        } else if selector.contains("[aria-label") {
            confidence += 0.2; // Accessibility attributes are stable
        } else if selector.contains(".") {
            confidence += 0.1; // Class selectors
        }
        
        // Adjust based on strategy
        match strategy {
            "ID-based" => confidence += 0.1,
            "Attribute-based" => confidence += 0.05,
            _ => {}
        }
        
        // Check selector complexity (simpler is often more stable)
        let complexity_penalty = (selector.len() as f32 / 100.0).min(0.2);
        confidence -= complexity_penalty;
        
        confidence.max(0.1).min(1.0)
    }
    
    /// Learn from successful selector healing
    fn learn_selector_mapping(&mut self, original: &str, healed: &str, alternatives: &[String]) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let history = self.selector_history.entry(original.to_string()).or_insert(SelectorHistory {
            primary_selector: original.to_string(),
            alternative_selectors: Vec::new(),
            success_count: 0,
            failure_count: 0,
            last_success: None,
            last_failure: None,
            confidence_score: 0.5,
        });
        
        history.alternative_selectors.clear();
        history.alternative_selectors.push(healed.to_string());
        history.alternative_selectors.extend(alternatives.iter().take(5).cloned());
        history.last_success = Some(now);
        history.success_count += 1;
        history.confidence_score = (history.confidence_score * 0.8 + 0.2).min(1.0);
    }
    
    /// Record successful selector use
    fn record_success(&mut self, original: &str, used: &str) {
        if let Some(history) = self.selector_history.get_mut(original) {
            history.success_count += 1;
            history.last_success = Some(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
            );
            history.confidence_score = (history.confidence_score * 1.1).min(1.0);
        }
    }
    
    // Selector generation strategies
    
    fn generate_id_selectors(context: &str) -> Vec<String> {
        let mut selectors = Vec::new();
        let words: Vec<&str> = context.split_whitespace().collect();
        
        for word in words {
            if word.len() > 2 {
                selectors.push(format!("#{}", word.to_lowercase()));
                selectors.push(format!("#{}-button", word.to_lowercase()));
                selectors.push(format!("#{}-btn", word.to_lowercase()));
                selectors.push(format!("#{}-link", word.to_lowercase()));
            }
        }
        
        selectors
    }
    
    fn generate_class_selectors(context: &str) -> Vec<String> {
        let mut selectors = Vec::new();
        let words: Vec<&str> = context.split_whitespace().collect();
        
        for word in words {
            if word.len() > 2 {
                selectors.push(format!(".{}", word.to_lowercase()));
                selectors.push(format!(".{}-button", word.to_lowercase()));
                selectors.push(format!(".btn-{}", word.to_lowercase()));
                selectors.push(format!(".{}-link", word.to_lowercase()));
            }
        }
        
        selectors
    }
    
    fn generate_attribute_selectors(context: &str) -> Vec<String> {
        let mut selectors = Vec::new();
        let words: Vec<&str> = context.split_whitespace().collect();
        
        for word in words {
            if word.len() > 2 {
                let word_lower = word.to_lowercase();
                selectors.push(format!("[aria-label*='{}']", word_lower));
                selectors.push(format!("[data-test*='{}']", word_lower));
                selectors.push(format!("[title*='{}']", word_lower));
                selectors.push(format!("[alt*='{}']", word_lower));
                selectors.push(format!("[name*='{}']", word_lower));
                selectors.push(format!("[placeholder*='{}']", word_lower));
            }
        }
        
        selectors
    }
    
    fn generate_text_selectors(context: &str) -> Vec<String> {
        let mut selectors = Vec::new();
        
        // Generate XPath selectors for text content
        selectors.push(format!("//button[contains(text(), '{}')]", context));
        selectors.push(format!("//a[contains(text(), '{}')]", context));
        selectors.push(format!("//*[contains(text(), '{}')]", context));
        
        // CSS pseudo-selectors (not standard but for demonstration)
        selectors.push(format!("button:contains('{}')", context));
        selectors.push(format!("a:contains('{}')", context));
        
        selectors
    }
    
    fn generate_position_selectors(context: &str) -> Vec<String> {
        vec![
            "button:first-of-type".to_string(),
            "button:last-of-type".to_string(),
            "button:nth-of-type(2)".to_string(),
            "a:first-child".to_string(),
            "a:last-child".to_string(),
            "input:first-of-type".to_string(),
        ]
    }
    
    fn generate_parent_child_selectors(context: &str) -> Vec<String> {
        let mut selectors = Vec::new();
        
        selectors.push("form button".to_string());
        selectors.push("div > button".to_string());
        selectors.push("nav a".to_string());
        selectors.push("header button".to_string());
        selectors.push("footer a".to_string());
        selectors.push(".container button".to_string());
        
        selectors
    }
    
    fn generate_sibling_selectors(context: &str) -> Vec<String> {
        vec![
            "label + input".to_string(),
            "input + button".to_string(),
            "h1 ~ button".to_string(),
            "p + a".to_string(),
        ]
    }
    
    fn generate_generic_selector(&self, context: &str) -> String {
        if context.contains("button") {
            "button".to_string()
        } else if context.contains("link") {
            "a".to_string()
        } else if context.contains("input") || context.contains("field") {
            "input".to_string()
        } else {
            "*".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_heal_broken_selector() {
        let mut healer = SelfHealingSelectors::new();
        
        let result = healer.heal_selector("#broken-id", "submit button").unwrap();
        assert!(!result.alternatives.is_empty());
        assert!(result.confidence > 0.0);
    }
    
    #[test]
    fn test_confidence_calculation() {
        let healer = SelfHealingSelectors::new();
        
        let id_confidence = healer.calculate_confidence("#specific-id", "ID-based");
        let class_confidence = healer.calculate_confidence(".generic-class", "Class-based");
        
        assert!(id_confidence > class_confidence);
    }
    
    #[test]
    fn test_learning_mechanism() {
        let mut healer = SelfHealingSelectors::new();
        
        healer.learn_selector_mapping("#old", "#new", &vec!["#alt1".to_string(), "#alt2".to_string()]);
        
        assert!(healer.selector_history.contains_key("#old"));
        let history = healer.selector_history.get("#old").unwrap();
        assert!(history.alternative_selectors.contains(&"#new".to_string()));
    }
    
    #[test]
    fn test_fallback_strategies() {
        let healer = SelfHealingSelectors::new();
        
        assert!(!healer.fallback_strategies.is_empty());
        assert!(healer.fallback_strategies.iter().any(|s| s.name == "ID-based"));
    }
}