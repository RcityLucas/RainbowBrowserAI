// Pattern Recognition System - The Neural Patterns of Organic Perception
//
// This module implements pattern-based understanding that learns and evolves,
// replacing static keyword matching with dynamic pattern recognition.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use anyhow::Result;

use crate::llm_service::llm_service_enhanced::TaskType;

/// Dynamic pattern that learns from interactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pattern {
    pub pattern_type: PatternType,
    pub keywords: Vec<String>,
    pub weight: f32,
    pub success_rate: f32,
    pub usage_count: u32,
    pub context_indicators: Vec<String>,
    pub negative_indicators: Vec<String>, // Words that reduce confidence
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatternType {
    ExactMatch,
    PartialMatch,
    Contextual,
    Semantic,
    Learned, // Patterns discovered through learning
}

/// Intelligent pattern matcher that evolves over time
pub struct PatternMatcher {
    task_patterns: HashMap<TaskType, Vec<Pattern>>,
    global_patterns: HashMap<String, Pattern>,
    learning_threshold: f32,
}

impl PatternMatcher {
    pub fn new() -> Self {
        let mut matcher = Self {
            task_patterns: HashMap::new(),
            global_patterns: HashMap::new(),
            learning_threshold: 0.7,
        };
        
        // Initialize with seed patterns (evolved from hardcoded keywords)
        matcher.initialize_seed_patterns();
        matcher
    }
    
    /// Initialize with intelligent seed patterns based on analysis of hardcoded rules
    fn initialize_seed_patterns(&mut self) {
        // Planning patterns - expanded and contextualized
        self.add_pattern(TaskType::Planning, Pattern {
            pattern_type: PatternType::Contextual,
            keywords: vec![
                "plan".to_string(), "itinerary".to_string(), "trip".to_string(), 
                "vacation".to_string(), "journey".to_string(), "travel plan".to_string(),
                "organize".to_string(), "schedule".to_string(), "prepare".to_string()
            ],
            weight: 0.9,
            success_rate: 0.85, // Start with optimistic but realistic success rate
            usage_count: 0,
            context_indicators: vec![
                "travel".to_string(), "destination".to_string(), "flight".to_string(),
                "hotel".to_string(), "booking".to_string(), "vacation".to_string()
            ],
            negative_indicators: vec![
                "cancel".to_string(), "delete".to_string(), "remove".to_string()
            ],
        });
        
        // Search patterns - enhanced with intent understanding
        self.add_pattern(TaskType::Search, Pattern {
            pattern_type: PatternType::Semantic,
            keywords: vec![
                "search".to_string(), "find".to_string(), "look for".to_string(),
                "discover".to_string(), "locate".to_string(), "hunt".to_string(),
                "seek".to_string(), "query".to_string()
            ],
            weight: 0.8,
            success_rate: 0.8,
            usage_count: 0,
            context_indicators: vec![
                "information".to_string(), "data".to_string(), "results".to_string(),
                "google".to_string(), "engine".to_string()
            ],
            negative_indicators: vec![
                "don't search".to_string(), "stop".to_string()
            ],
        });
        
        // Analysis patterns - deeper understanding
        self.add_pattern(TaskType::Analysis, Pattern {
            pattern_type: PatternType::Contextual,
            keywords: vec![
                "analyze".to_string(), "review".to_string(), "evaluate".to_string(),
                "assess".to_string(), "examine".to_string(), "study".to_string(),
                "investigate".to_string(), "inspect".to_string()
            ],
            weight: 0.85,
            success_rate: 0.75,
            usage_count: 0,
            context_indicators: vec![
                "data".to_string(), "performance".to_string(), "metrics".to_string(),
                "report".to_string(), "comparison".to_string()
            ],
            negative_indicators: vec![],
        });
        
        // Extraction patterns
        self.add_pattern(TaskType::Extraction, Pattern {
            pattern_type: PatternType::ExactMatch,
            keywords: vec![
                "extract".to_string(), "scrape".to_string(), "collect".to_string(),
                "gather".to_string(), "harvest".to_string(), "pull".to_string()
            ],
            weight: 0.9,
            success_rate: 0.9,
            usage_count: 0,
            context_indicators: vec![
                "data".to_string(), "content".to_string(), "text".to_string(),
                "information".to_string()
            ],
            negative_indicators: vec![],
        });
        
        // Navigation patterns
        self.add_pattern(TaskType::Navigation, Pattern {
            pattern_type: PatternType::ExactMatch,
            keywords: vec![
                "navigate".to_string(), "go to".to_string(), "open".to_string(),
                "visit".to_string(), "browse".to_string(), "load".to_string()
            ],
            weight: 0.95,
            success_rate: 0.95,
            usage_count: 0,
            context_indicators: vec![
                "url".to_string(), "website".to_string(), "page".to_string(),
                "site".to_string(), "http".to_string()
            ],
            negative_indicators: vec![],
        });
        
        // Add patterns for all other task types...
        self.add_monitoring_patterns();
        self.add_testing_patterns();
        self.add_reporting_patterns();
        self.add_screenshot_patterns();
    }
    
    fn add_monitoring_patterns(&mut self) {
        self.add_pattern(TaskType::Monitoring, Pattern {
            pattern_type: PatternType::Contextual,
            keywords: vec![
                "monitor".to_string(), "watch".to_string(), "track".to_string(),
                "observe".to_string(), "check".to_string(), "follow".to_string()
            ],
            weight: 0.8,
            success_rate: 0.7,
            usage_count: 0,
            context_indicators: vec![
                "changes".to_string(), "updates".to_string(), "status".to_string(),
                "real-time".to_string(), "continuous".to_string()
            ],
            negative_indicators: vec!["stop monitoring".to_string()],
        });
    }
    
    fn add_testing_patterns(&mut self) {
        self.add_pattern(TaskType::Testing, Pattern {
            pattern_type: PatternType::Contextual,
            keywords: vec!["test".to_string(), "verify".to_string(), "validate".to_string()],
            weight: 0.7,
            success_rate: 0.8,
            usage_count: 0,
            context_indicators: vec![
                "sites".to_string(), "websites".to_string(), "functionality".to_string(),
                "performance".to_string(), "load".to_string()
            ],
            negative_indicators: vec![],
        });
    }
    
    fn add_reporting_patterns(&mut self) {
        self.add_pattern(TaskType::Reporting, Pattern {
            pattern_type: PatternType::Semantic,
            keywords: vec![
                "report".to_string(), "summary".to_string(), "statistics".to_string(),
                "metrics".to_string(), "dashboard".to_string()
            ],
            weight: 0.8,
            success_rate: 0.75,
            usage_count: 0,
            context_indicators: vec![
                "generate".to_string(), "create".to_string(), "compile".to_string(),
                "data".to_string(), "results".to_string()
            ],
            negative_indicators: vec![],
        });
    }
    
    fn add_screenshot_patterns(&mut self) {
        self.add_pattern(TaskType::Screenshot, Pattern {
            pattern_type: PatternType::ExactMatch,
            keywords: vec![
                "screenshot".to_string(), "capture".to_string(), "snap".to_string(),
                "image".to_string(), "picture".to_string()
            ],
            weight: 0.95,
            success_rate: 0.9,
            usage_count: 0,
            context_indicators: vec![
                "screen".to_string(), "page".to_string(), "visual".to_string()
            ],
            negative_indicators: vec![],
        });
    }
    
    fn add_pattern(&mut self, task_type: TaskType, pattern: Pattern) {
        self.task_patterns.entry(task_type).or_insert_with(Vec::new).push(pattern);
    }
    
    /// Intelligent pattern matching with context awareness
    pub fn match_patterns(&self, input: &str, context: &str) -> Vec<(TaskType, f32)> {
        let input_lower = input.to_lowercase();
        let context_lower = context.to_lowercase();
        let mut matches = Vec::new();
        
        for (task_type, patterns) in &self.task_patterns {
            let mut best_score = 0.0f32;
            
            for pattern in patterns {
                let score = self.calculate_pattern_score(pattern, &input_lower, &context_lower);
                if score > best_score {
                    best_score = score;
                }
            }
            
            if best_score > 0.1 { // Minimum threshold for consideration
                matches.push((*task_type, best_score));
            }
        }
        
        // Sort by score descending
        matches.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        matches
    }
    
    /// Calculate intelligent pattern score considering multiple factors
    fn calculate_pattern_score(&self, pattern: &Pattern, input: &str, context: &str) -> f32 {
        let mut score = 0.0f32;
        
        // Keyword matching with weighted importance
        let keyword_score = self.calculate_keyword_score(pattern, input);
        score += keyword_score * pattern.weight;
        
        // Context indicators boost
        let context_boost = self.calculate_context_boost(pattern, context);
        score += context_boost * 0.3;
        
        // Success rate influences confidence
        score *= pattern.success_rate;
        
        // Negative indicators penalty
        let negative_penalty = self.calculate_negative_penalty(pattern, input);
        score *= (1.0 - negative_penalty);
        
        // Experience factor (patterns used more often get slight boost)
        let experience_factor = 1.0 + (pattern.usage_count as f32 * 0.01).min(0.2);
        score *= experience_factor;
        
        score.min(1.0) // Cap at 1.0
    }
    
    fn calculate_keyword_score(&self, pattern: &Pattern, input: &str) -> f32 {
        let mut matches = 0;
        let mut total_weight = 0.0f32;
        
        for keyword in &pattern.keywords {
            if input.contains(keyword) {
                matches += 1;
                // Longer keywords get higher weight
                total_weight += keyword.len() as f32 / 10.0;
            }
        }
        
        if matches == 0 {
            return 0.0;
        }
        
        // Calculate score based on match ratio and keyword importance
        let match_ratio = matches as f32 / pattern.keywords.len() as f32;
        let weighted_score = total_weight / pattern.keywords.len() as f32;
        
        (match_ratio * 0.7 + weighted_score * 0.3).min(1.0)
    }
    
    fn calculate_context_boost(&self, pattern: &Pattern, context: &str) -> f32 {
        let mut boost = 0.0f32;
        
        for indicator in &pattern.context_indicators {
            if context.contains(indicator) {
                boost += 0.1;
            }
        }
        
        boost.min(0.5) // Maximum 50% boost from context
    }
    
    fn calculate_negative_penalty(&self, pattern: &Pattern, input: &str) -> f32 {
        let mut penalty = 0.0f32;
        
        for negative in &pattern.negative_indicators {
            if input.contains(negative) {
                penalty += 0.3; // Significant penalty for negative indicators
            }
        }
        
        penalty.min(0.8) // Maximum 80% penalty
    }
    
    /// Learn from interaction outcomes to improve patterns
    pub fn learn_from_outcome(&mut self, input: &str, task_type: TaskType, success: bool) -> Result<()> {
        if let Some(patterns) = self.task_patterns.get_mut(&task_type) {
            for pattern in patterns {
                // Check if this pattern would have matched
                let would_match = pattern.keywords.iter().any(|k| input.to_lowercase().contains(k));
                
                if would_match {
                    pattern.usage_count += 1;
                    
                    // Update success rate using exponential moving average
                    let learning_rate = 0.1;
                    let outcome_score = if success { 1.0 } else { 0.0 };
                    pattern.success_rate = pattern.success_rate * (1.0 - learning_rate) + 
                                         outcome_score * learning_rate;
                    
                    // If this pattern consistently fails, reduce its weight
                    if pattern.success_rate < 0.3 && pattern.usage_count > 10 {
                        pattern.weight *= 0.9;
                    }
                    
                    // If this pattern consistently succeeds, boost its weight
                    if pattern.success_rate > 0.9 && pattern.usage_count > 5 {
                        pattern.weight = (pattern.weight * 1.05).min(1.0);
                    }
                }
            }
        }
        
        // Learn new patterns from successful novel inputs
        if success {
            self.learn_new_pattern(input, task_type)?;
        }
        
        Ok(())
    }
    
    /// Discover new patterns from successful interactions
    fn learn_new_pattern(&mut self, input: &str, task_type: TaskType) -> Result<()> {
        let words: Vec<&str> = input.split_whitespace().collect();
        
        // Look for novel keywords that aren't in existing patterns
        if let Some(existing_patterns) = self.task_patterns.get(&task_type) {
            let existing_keywords: std::collections::HashSet<String> = existing_patterns
                .iter()
                .flat_map(|p| p.keywords.iter())
                .cloned()
                .collect();
            
            let novel_words: Vec<String> = words
                .iter()
                .filter(|word| word.len() > 3) // Ignore short words
                .map(|s| s.to_lowercase())
                .filter(|word| !existing_keywords.contains(word))
                .collect();
            
            // If we found novel keywords, create a learned pattern
            if !novel_words.is_empty() && novel_words.len() <= 3 {
                let learned_pattern = Pattern {
                    pattern_type: PatternType::Learned,
                    keywords: novel_words,
                    weight: 0.5, // Start with moderate weight
                    success_rate: 1.0, // Start optimistic
                    usage_count: 1,
                    context_indicators: vec![],
                    negative_indicators: vec![],
                };
                
                self.add_pattern(task_type, learned_pattern);
            }
        }
        
        Ok(())
    }
    
    /// Get statistics about pattern learning and effectiveness
    pub fn get_learning_stats(&self) -> PatternStats {
        let mut total_patterns = 0;
        let mut learned_patterns = 0;
        let mut total_usage = 0;
        let mut average_success_rate = 0.0;
        
        for patterns in self.task_patterns.values() {
            for pattern in patterns {
                total_patterns += 1;
                total_usage += pattern.usage_count;
                average_success_rate += pattern.success_rate;
                
                if matches!(pattern.pattern_type, PatternType::Learned) {
                    learned_patterns += 1;
                }
            }
        }
        
        PatternStats {
            total_patterns,
            learned_patterns,
            total_usage,
            average_success_rate: if total_patterns > 0 { 
                average_success_rate / total_patterns as f32 
            } else { 
                0.0 
            },
        }
    }
}

/// Statistics about pattern learning and performance
#[derive(Debug, Serialize, Deserialize)]
pub struct PatternStats {
    pub total_patterns: u32,
    pub learned_patterns: u32,
    pub total_usage: u32,
    pub average_success_rate: f32,
}

impl Default for PatternMatcher {
    fn default() -> Self {
        Self::new()
    }
}