// Learning Engine - Continuous Learning from Interactions
//
// This module implements the learning system that enables the AI to improve
// its understanding over time, evolving from each interaction.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use anyhow::Result;
use chrono::{DateTime, Utc};

use crate::llm_service::llm_service_enhanced::TaskType;

/// Learning engine that improves understanding over time
pub struct LearningEngine {
    interaction_log: Vec<InteractionOutcome>,
    learning_patterns: HashMap<String, LearningPattern>,
    adaptation_rate: f32,
    min_interactions_for_learning: u32,
}

/// Record of an interaction and its outcome for learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionOutcome {
    pub timestamp: DateTime<Utc>,
    pub user_input: String,
    pub predicted_task_type: TaskType,
    pub predicted_confidence: f32,
    pub actual_success: bool,
    pub execution_time_ms: u64,
    pub error_type: Option<String>,
    pub context: String,
    pub user_feedback: Option<UserFeedback>,
}

/// User feedback for learning improvement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserFeedback {
    pub correct_interpretation: bool,
    pub suggestion: Option<String>,
    pub satisfaction_score: Option<f32>, // 0.0 to 1.0
}

/// Learned pattern from interaction analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
struct LearningPattern {
    pattern_text: String,
    associated_task_type: TaskType,
    success_correlation: f32,
    occurrence_count: u32,
    last_seen: DateTime<Utc>,
    confidence_impact: f32, // How much this pattern should adjust confidence
}

impl LearningEngine {
    pub fn new() -> Self {
        Self {
            interaction_log: Vec::new(),
            learning_patterns: HashMap::new(),
            adaptation_rate: 0.1,
            min_interactions_for_learning: 5,
        }
    }
    
    /// Observe an interaction and learn from it
    pub fn observe_interaction(&mut self, outcome: InteractionOutcome) -> Result<()> {
        // Add to interaction log
        self.interaction_log.push(outcome.clone());
        
        // Keep only recent interactions (last 1000)
        if self.interaction_log.len() > 1000 {
            self.interaction_log.remove(0);
        }
        
        // Extract learning patterns from the interaction
        self.extract_learning_patterns(&outcome)?;
        
        // Analyze success/failure patterns
        self.analyze_success_patterns()?;
        
        // Update adaptation strategies
        self.update_adaptation_strategies(&outcome)?;
        
        Ok(())
    }
    
    /// Extract learning patterns from successful and failed interactions
    fn extract_learning_patterns(&mut self, outcome: &InteractionOutcome) -> Result<()> {
        // Extract meaningful phrases from user input
        let phrases = self.extract_meaningful_phrases(&outcome.user_input);
        
        for phrase in phrases {
            let pattern_key = format!("{}_{:?}", phrase, outcome.predicted_task_type);
            
            let pattern = self.learning_patterns.entry(pattern_key.clone()).or_insert_with(|| {
                LearningPattern {
                    pattern_text: phrase.clone(),
                    associated_task_type: outcome.predicted_task_type,
                    success_correlation: 0.5,
                    occurrence_count: 0,
                    last_seen: Utc::now(),
                    confidence_impact: 0.0,
                }
            });
            
            // Update pattern statistics
            pattern.occurrence_count += 1;
            pattern.last_seen = outcome.timestamp;
            
            // Update success correlation using exponential moving average
            let outcome_value = if outcome.actual_success { 1.0 } else { 0.0 };
            pattern.success_correlation = pattern.success_correlation * (1.0 - self.adaptation_rate) + 
                                        outcome_value * self.adaptation_rate;
            
            // Update confidence impact based on prediction accuracy
            let prediction_error = (outcome.predicted_confidence - outcome_value).abs();
            let confidence_adjustment = if prediction_error > 0.2 {
                // If we consistently mis-predict with this pattern, adjust confidence
                if outcome.actual_success && outcome.predicted_confidence < 0.7 {
                    0.05 // This pattern might indicate higher success
                } else if !outcome.actual_success && outcome.predicted_confidence > 0.7 {
                    -0.05 // This pattern might indicate lower success
                } else {
                    0.0
                }
            } else {
                0.0
            };
            
            pattern.confidence_impact = pattern.confidence_impact * 0.9 + confidence_adjustment * 0.1;
        }
        
        // Remove patterns that haven't been seen recently and have low occurrence
        self.cleanup_old_patterns();
        
        Ok(())
    }
    
    /// Extract meaningful phrases from user input for pattern learning
    fn extract_meaningful_phrases(&self, input: &str) -> Vec<String> {
        let input_lower = input.to_lowercase();
        let words: Vec<&str> = input_lower
            .split_whitespace()
            .filter(|word| word.len() > 2) // Filter out very short words
            .collect();
        
        let mut phrases = Vec::new();
        
        // Extract single words
        for word in &words {
            if word.len() > 3 { // Focus on longer words
                phrases.push(word.to_string());
            }
        }
        
        // Extract 2-word phrases
        for window in words.windows(2) {
            if window[0].len() > 2 && window[1].len() > 2 {
                phrases.push(format!("{} {}", window[0], window[1]));
            }
        }
        
        // Extract 3-word phrases for very specific patterns
        for window in words.windows(3) {
            if window.iter().all(|w| w.len() > 2) {
                phrases.push(format!("{} {} {}", window[0], window[1], window[2]));
            }
        }
        
        phrases
    }
    
    /// Analyze patterns in successful vs failed interactions
    fn analyze_success_patterns(&mut self) -> Result<()> {
        if self.interaction_log.len() < self.min_interactions_for_learning as usize {
            return Ok(());
        }
        
        // Group interactions by task type
        let mut task_outcomes: HashMap<TaskType, Vec<&InteractionOutcome>> = HashMap::new();
        for outcome in &self.interaction_log {
            task_outcomes.entry(outcome.predicted_task_type).or_default().push(outcome);
        }
        
        // Analyze each task type for success patterns
        for (task_type, outcomes) in task_outcomes {
            self.analyze_task_success_patterns(task_type, &outcomes)?;
        }
        
        Ok(())
    }
    
    /// Analyze success patterns for a specific task type
    fn analyze_task_success_patterns(&mut self, task_type: TaskType, outcomes: &[&InteractionOutcome]) -> Result<()> {
        if outcomes.len() < 3 {
            return Ok(()); // Need minimum interactions for analysis
        }
        
        let successful_outcomes: Vec<_> = outcomes.iter().filter(|o| o.actual_success).collect();
        let failed_outcomes: Vec<_> = outcomes.iter().filter(|o| !o.actual_success).collect();
        
        // Find common patterns in successful interactions
        if !successful_outcomes.is_empty() {
            self.identify_success_indicators(task_type, &successful_outcomes)?;
        }
        
        // Find common patterns in failed interactions
        if !failed_outcomes.is_empty() {
            self.identify_failure_indicators(task_type, &failed_outcomes)?;
        }
        
        Ok(())
    }
    
    /// Identify patterns that correlate with success
    fn identify_success_indicators(&mut self, _task_type: TaskType, successful_outcomes: &[&InteractionOutcome]) -> Result<()> {
        // Analyze common words/phrases in successful interactions
        let mut word_frequency: HashMap<String, u32> = HashMap::new();
        
        for outcome in successful_outcomes {
            let words = outcome.user_input.to_lowercase()
                .split_whitespace()
                .filter(|w| w.len() > 3)
                .map(|w| w.to_string());
            
            for word in words {
                *word_frequency.entry(word).or_insert(0) += 1;
            }
        }
        
        // Find words that appear frequently in successful interactions
        let min_frequency = (successful_outcomes.len() as f32 * 0.3) as u32; // At least 30% of interactions
        for (word, frequency) in word_frequency {
            if frequency >= min_frequency && frequency > 1 {
                // This word might be a success indicator
                // Note: In a full implementation, we'd create new patterns here
                // For now, just log the discovery
                tracing::info!("Discovered potential success indicator: '{}' (frequency: {})", word, frequency);
            }
        }
        
        Ok(())
    }
    
    /// Identify patterns that correlate with failure
    fn identify_failure_indicators(&mut self, _task_type: TaskType, failed_outcomes: &[&InteractionOutcome]) -> Result<()> {
        // Analyze common error types and patterns in failed interactions
        let mut error_patterns: HashMap<String, u32> = HashMap::new();
        
        for outcome in failed_outcomes {
            if let Some(error_type) = &outcome.error_type {
                *error_patterns.entry(error_type.clone()).or_insert(0) += 1;
            }
            
            // Also analyze context that leads to failures
            let context_words = outcome.context.to_lowercase()
                .split_whitespace()
                .filter(|w| w.len() > 3)
                .map(|w| w.to_string());
            
            for word in context_words {
                *error_patterns.entry(format!("context:{}", word)).or_insert(0) += 1;
            }
        }
        
        // Identify frequent failure patterns
        let min_frequency = (failed_outcomes.len() as f32 * 0.4) as u32; // At least 40% of failures
        for (pattern, frequency) in error_patterns {
            if frequency >= min_frequency && frequency > 1 {
                tracing::warn!("Discovered potential failure indicator: '{}' (frequency: {})", pattern, frequency);
            }
        }
        
        Ok(())
    }
    
    /// Update adaptation strategies based on recent learning
    fn update_adaptation_strategies(&mut self, outcome: &InteractionOutcome) -> Result<()> {
        // Adjust adaptation rate based on recent prediction accuracy
        let recent_interactions = self.interaction_log.iter()
            .rev()
            .take(20)
            .collect::<Vec<_>>();
        
        if recent_interactions.len() >= 10 {
            let accurate_predictions = recent_interactions
                .iter()
                .filter(|i| {
                    let predicted_success = i.predicted_confidence > 0.5;
                    predicted_success == i.actual_success
                })
                .count();
            
            let accuracy = accurate_predictions as f32 / recent_interactions.len() as f32;
            
            // Adjust adaptation rate based on accuracy
            if accuracy > 0.8 {
                // High accuracy, reduce adaptation rate to maintain stability
                self.adaptation_rate = (self.adaptation_rate * 0.95).max(0.05);
            } else if accuracy < 0.6 {
                // Low accuracy, increase adaptation rate to learn faster
                self.adaptation_rate = (self.adaptation_rate * 1.05).min(0.2);
            }
        }
        
        // Learn from user feedback if available
        if let Some(feedback) = &outcome.user_feedback {
            self.learn_from_user_feedback(outcome, feedback)?;
        }
        
        Ok(())
    }
    
    /// Learn from explicit user feedback
    fn learn_from_user_feedback(&mut self, outcome: &InteractionOutcome, feedback: &UserFeedback) -> Result<()> {
        if !feedback.correct_interpretation {
            // User indicated our interpretation was wrong
            // Reduce confidence in patterns that led to this prediction
            let phrases = self.extract_meaningful_phrases(&outcome.user_input);
            
            for phrase in phrases {
                let pattern_key = format!("{}_{:?}", phrase, outcome.predicted_task_type);
                if let Some(pattern) = self.learning_patterns.get_mut(&pattern_key) {
                    pattern.confidence_impact -= 0.1; // Reduce confidence impact
                    pattern.success_correlation *= 0.8; // Reduce success correlation
                }
            }
        }
        
        if let Some(suggestion) = &feedback.suggestion {
            // User provided a suggestion - learn from it
            let suggestion_phrases = self.extract_meaningful_phrases(suggestion);
            
            // Create or strengthen patterns from the suggestion
            for phrase in suggestion_phrases {
                let pattern_key = format!("{}_{:?}", phrase, outcome.predicted_task_type);
                let pattern = self.learning_patterns.entry(pattern_key).or_insert_with(|| {
                    LearningPattern {
                        pattern_text: phrase.clone(),
                        associated_task_type: outcome.predicted_task_type,
                        success_correlation: 0.8, // Start high for user suggestions
                        occurrence_count: 0,
                        last_seen: Utc::now(),
                        confidence_impact: 0.1, // Positive confidence impact
                    }
                });
                
                pattern.success_correlation = (pattern.success_correlation + 0.2).min(1.0);
                pattern.confidence_impact = (pattern.confidence_impact + 0.05).min(0.2);
            }
        }
        
        Ok(())
    }
    
    /// Clean up old patterns that are no longer relevant
    fn cleanup_old_patterns(&mut self) {
        let cutoff_date = Utc::now() - chrono::Duration::days(30);
        
        self.learning_patterns.retain(|_, pattern| {
            // Keep patterns that are either:
            // 1. Recent (seen in last 30 days)
            // 2. Frequent (seen at least 10 times)
            // 3. High impact (strong confidence impact)
            pattern.last_seen > cutoff_date || 
            pattern.occurrence_count >= 10 || 
            pattern.confidence_impact.abs() > 0.1
        });
    }
    
    /// Get learning insights for monitoring and debugging
    pub fn get_learning_insights(&self) -> LearningInsights {
        let total_interactions = self.interaction_log.len();
        let successful_interactions = self.interaction_log.iter().filter(|i| i.actual_success).count();
        
        let success_rate = if total_interactions > 0 {
            successful_interactions as f32 / total_interactions as f32
        } else {
            0.0
        };
        
        // Analyze recent trend (last 20 interactions)
        let recent_interactions = self.interaction_log.iter().rev().take(20).collect::<Vec<_>>();
        let recent_successes = recent_interactions.iter().filter(|i| i.actual_success).count();
        let recent_success_rate = if recent_interactions.len() > 0 {
            recent_successes as f32 / recent_interactions.len() as f32
        } else {
            0.0
        };
        
        // Count patterns by strength
        let strong_patterns = self.learning_patterns.values()
            .filter(|p| p.confidence_impact.abs() > 0.1)
            .count();
        
        let total_patterns = self.learning_patterns.len();
        
        LearningInsights {
            total_interactions,
            success_rate,
            recent_success_rate,
            total_learned_patterns: total_patterns,
            strong_patterns,
            adaptation_rate: self.adaptation_rate,
            top_patterns: self.get_top_patterns(5),
        }
    }
    
    /// Get top learning patterns by impact
    fn get_top_patterns(&self, limit: usize) -> Vec<PatternInsight> {
        let mut patterns: Vec<_> = self.learning_patterns.values()
            .map(|p| PatternInsight {
                pattern_text: p.pattern_text.clone(),
                associated_task_type: p.associated_task_type,
                success_correlation: p.success_correlation,
                confidence_impact: p.confidence_impact,
                occurrence_count: p.occurrence_count,
            })
            .collect();
        
        // Sort by confidence impact (absolute value)
        patterns.sort_by(|a, b| b.confidence_impact.abs().partial_cmp(&a.confidence_impact.abs()).unwrap());
        
        patterns.into_iter().take(limit).collect()
    }
    
    /// Get patterns that might help with a specific input
    pub fn get_relevant_patterns(&self, input: &str, task_type: TaskType) -> Vec<&LearningPattern> {
        let input_lower = input.to_lowercase();
        
        self.learning_patterns.values()
            .filter(|pattern| {
                pattern.associated_task_type == task_type &&
                input_lower.contains(&pattern.pattern_text) &&
                pattern.confidence_impact.abs() > 0.02 // Only meaningful patterns
            })
            .collect()
    }
}

/// Learning insights for monitoring and debugging
#[derive(Debug, Serialize, Deserialize)]
pub struct LearningInsights {
    pub total_interactions: usize,
    pub success_rate: f32,
    pub recent_success_rate: f32,
    pub total_learned_patterns: usize,
    pub strong_patterns: usize,
    pub adaptation_rate: f32,
    pub top_patterns: Vec<PatternInsight>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PatternInsight {
    pub pattern_text: String,
    pub associated_task_type: TaskType,
    pub success_correlation: f32,
    pub confidence_impact: f32,
    pub occurrence_count: u32,
}

impl Default for LearningEngine {
    fn default() -> Self {
        Self::new()
    }
}