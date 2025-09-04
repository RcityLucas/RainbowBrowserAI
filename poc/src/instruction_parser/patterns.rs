//! Pattern Matching Module
//! 
//! Fast pattern matching for common instruction patterns

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::intent_recognizer::Intent;
use super::entity_extractor::Entity;

/// Instruction pattern for fast matching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstructionPattern {
    pub template: String,
    pub intent: Intent,
    pub entities: Vec<Entity>,
    pub confidence: f32,
    pub usage_count: u32,
    pub success_rate: f32,
}

/// Pattern matcher for optimized instruction parsing
pub struct PatternMatcher {
    patterns: HashMap<String, InstructionPattern>,
    learned_patterns: HashMap<String, InstructionPattern>,
    improvement_patterns: HashMap<String, String>,
}

impl PatternMatcher {
    pub fn new() -> Self {
        let mut matcher = Self {
            patterns: HashMap::new(),
            learned_patterns: HashMap::new(),
            improvement_patterns: HashMap::new(),
        };
        
        matcher.load_default_patterns();
        matcher
    }
    
    pub fn match_pattern(&self, input: &str) -> Option<InstructionPattern> {
        // Try exact match first
        if let Some(pattern) = self.patterns.get(input) {
            return Some(pattern.clone());
        }
        
        if let Some(pattern) = self.learned_patterns.get(input) {
            return Some(pattern.clone());
        }
        
        // Try fuzzy matching
        self.fuzzy_match(input)
    }
    
    pub fn record_success(&mut self, instruction: &super::UserInstruction) {
        let key = instruction.normalized_text.clone();
        
        // Create pattern from successful instruction
        let pattern = InstructionPattern {
            template: key.clone(),
            intent: instruction.intent.clone(),
            entities: instruction.entities.clone(),
            confidence: instruction.confidence,
            usage_count: 1,
            success_rate: 1.0,
        };
        
        // Update or insert pattern
        self.learned_patterns
            .entry(key)
            .and_modify(|p| {
                p.usage_count += 1;
                p.success_rate = (p.success_rate * (p.usage_count - 1) as f32 + 1.0) / p.usage_count as f32;
                p.confidence = p.confidence.max(instruction.confidence);
            })
            .or_insert(pattern);
    }
    
    pub fn add_improvement_pattern(&mut self, original: &str, improved: &str) {
        self.improvement_patterns.insert(original.to_string(), improved.to_string());
    }
    
    fn load_default_patterns(&mut self) {
        use super::intent_recognizer::{
            Intent, NavigationTarget, DataType, 
            SearchScope, ScrollDirection, ScreenshotArea
        };
        
        // Navigation patterns
        self.add_pattern(
            "go to google",
            Intent::Navigate {
                target: NavigationTarget::Url("https://google.com".to_string()),
                wait_for: None,
            },
            vec![],
            0.95
        );
        
        self.add_pattern(
            "go back",
            Intent::Navigate {
                target: NavigationTarget::Back,
                wait_for: None,
            },
            vec![],
            0.95
        );
        
        self.add_pattern(
            "refresh the page",
            Intent::Navigate {
                target: NavigationTarget::Refresh,
                wait_for: None,
            },
            vec![],
            0.95
        );
        
        // Click patterns
        self.add_pattern(
            "click submit",
            Intent::Click {
                target_description: "submit button".to_string(),
                modifier_keys: vec![],
            },
            vec![],
            0.9
        );
        
        self.add_pattern(
            "click the first link",
            Intent::Click {
                target_description: "first link".to_string(),
                modifier_keys: vec![],
            },
            vec![],
            0.85
        );
        
        // Type patterns
        self.add_pattern(
            "type hello world",
            Intent::Type {
                text: "hello world".to_string(),
                target: None,
                clear_first: true,
            },
            vec![],
            0.85
        );
        
        // Search patterns
        self.add_pattern(
            "search for products",
            Intent::Search {
                query: "products".to_string(),
                scope: SearchScope::CurrentPage,
            },
            vec![],
            0.85
        );
        
        // Extract patterns
        self.add_pattern(
            "get all links",
            Intent::Extract {
                data_type: DataType::Links,
                filters: vec![],
            },
            vec![],
            0.9
        );
        
        self.add_pattern(
            "extract prices",
            Intent::Extract {
                data_type: DataType::Prices,
                filters: vec![],
            },
            vec![],
            0.9
        );
        
        // Screenshot patterns
        self.add_pattern(
            "take a screenshot",
            Intent::Screenshot {
                area: ScreenshotArea::FullPage,
                filename: None,
            },
            vec![],
            0.95
        );
        
        // Scroll patterns
        self.add_pattern(
            "scroll down",
            Intent::Scroll {
                direction: ScrollDirection::Down,
                amount: Some(500),
            },
            vec![],
            0.9
        );
        
        self.add_pattern(
            "scroll to top",
            Intent::Scroll {
                direction: ScrollDirection::Up,
                amount: Some(i32::MAX),
            },
            vec![],
            0.9
        );
    }
    
    fn add_pattern(&mut self, template: &str, intent: Intent, entities: Vec<Entity>, confidence: f32) {
        self.patterns.insert(
            template.to_string(),
            InstructionPattern {
                template: template.to_string(),
                intent,
                entities,
                confidence,
                usage_count: 0,
                success_rate: 0.0,
            }
        );
    }
    
    fn fuzzy_match(&self, input: &str) -> Option<InstructionPattern> {
        // Calculate similarity scores
        let mut best_match = None;
        let mut best_score = 0.0;
        
        for (template, pattern) in &self.patterns {
            let score = self.calculate_similarity(input, template);
            if score > best_score && score > 0.7 {
                best_score = score;
                best_match = Some(pattern.clone());
            }
        }
        
        for (template, pattern) in &self.learned_patterns {
            let score = self.calculate_similarity(input, template);
            // Learned patterns need higher confidence
            if score > best_score && score > 0.8 && pattern.success_rate > 0.6 {
                best_score = score;
                best_match = Some(pattern.clone());
            }
        }
        
        best_match
    }
    
    fn calculate_similarity(&self, s1: &str, s2: &str) -> f32 {
        // Simple word-based similarity
        let words1: Vec<&str> = s1.split_whitespace().collect();
        let words2: Vec<&str> = s2.split_whitespace().collect();
        
        if words1.is_empty() || words2.is_empty() {
            return 0.0;
        }
        
        let mut matches = 0;
        for word1 in &words1 {
            for word2 in &words2 {
                if word1 == word2 || self.are_synonyms(word1, word2) {
                    matches += 1;
                    break;
                }
            }
        }
        
        let max_len = words1.len().max(words2.len()) as f32;
        matches as f32 / max_len
    }
    
    fn are_synonyms(&self, word1: &str, word2: &str) -> bool {
        // Simple synonym check
        let synonyms = vec![
            vec!["click", "press", "tap", "select"],
            vec!["type", "enter", "input", "fill"],
            vec!["go", "navigate", "visit", "open"],
            vec!["find", "search", "look", "locate"],
            vec!["get", "extract", "scrape", "retrieve"],
            vec!["scroll", "move", "pan"],
            vec!["submit", "send", "confirm"],
            vec!["cancel", "close", "exit", "quit"],
        ];
        
        for group in synonyms {
            if group.contains(&word1) && group.contains(&word2) {
                return true;
            }
        }
        
        false
    }
}

/// Pattern optimization strategies
pub struct PatternOptimizer {
    frequency_threshold: u32,
    success_threshold: f32,
}

impl PatternOptimizer {
    pub fn new() -> Self {
        Self {
            frequency_threshold: 5,
            success_threshold: 0.7,
        }
    }
    
    pub fn optimize_patterns(&self, patterns: &HashMap<String, InstructionPattern>) -> Vec<InstructionPattern> {
        // Select patterns worth keeping
        patterns.values()
            .filter(|p| p.usage_count >= self.frequency_threshold && p.success_rate >= self.success_threshold)
            .cloned()
            .collect()
    }
    
    pub fn suggest_improvements(&self, patterns: &HashMap<String, InstructionPattern>) -> Vec<PatternImprovement> {
        let mut improvements = Vec::new();
        
        for (template, pattern) in patterns {
            if pattern.success_rate < 0.5 && pattern.usage_count > 3 {
                improvements.push(PatternImprovement {
                    original: template.clone(),
                    suggestion: self.generate_suggestion(pattern),
                    reason: "Low success rate".to_string(),
                });
            }
        }
        
        improvements
    }
    
    fn generate_suggestion(&self, pattern: &InstructionPattern) -> String {
        // Generate improvement suggestion based on pattern analysis
        match &pattern.intent {
            Intent::Click { target_description, .. } => {
                if target_description.is_empty() {
                    "Specify what to click on".to_string()
                } else {
                    format!("Try using a more specific selector for '{}'", target_description)
                }
            }
            Intent::Type {  target, .. } => {
                if target.is_none() {
                    "Specify where to type the text".to_string()
                } else {
                    format!("Make sure the input field is focused first")
                }
            }
            _ => "Try being more specific in your instruction".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternImprovement {
    pub original: String,
    pub suggestion: String,
    pub reason: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_pattern_matching() {
        let matcher = PatternMatcher::new();
        
        // Test exact match
        let pattern = matcher.match_pattern("go to google");
        assert!(pattern.is_some());
        
        // Test fuzzy match
        let pattern = matcher.match_pattern("navigate to google");
        assert!(pattern.is_some());
    }
    
    #[test]
    fn test_similarity_calculation() {
        let matcher = PatternMatcher::new();
        
        let score = matcher.calculate_similarity("click the button", "click button");
        assert!(score > 0.6);
        
        let score = matcher.calculate_similarity("type hello", "enter hello");
        assert!(score > 0.5); // Synonyms should match
    }
    
    #[test]
    fn test_pattern_learning() {
        use super::super::intent_recognizer::NavigationTarget;
        
        let mut matcher = PatternMatcher::new();
        
        let instruction = super::super::UserInstruction {
            raw_text: "open amazon".to_string(),
            normalized_text: "open amazon".to_string(),
            intent: Intent::Navigate {
                target: NavigationTarget::Url("https://amazon.com".to_string()),
                wait_for: None,
            },
            entities: vec![],
            context: super::super::InstructionContext::new(),
            confidence: 0.9,
            suggestions: vec![],
        };
        
        matcher.record_success(&instruction);
        
        // Should now match the learned pattern
        let pattern = matcher.match_pattern("open amazon");
        assert!(pattern.is_some());
    }
}