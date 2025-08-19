// Confidence Calibration System - Adaptive Confidence Based on Real Outcomes
//
// This module implements confidence calibration that learns from actual success rates,
// replacing hardcoded confidence scores with adaptive, evidence-based confidence.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use anyhow::Result;

use crate::llm_service::llm_service_enhanced::TaskType;

/// Intelligent confidence score that adapts based on outcomes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceScore {
    pub base_confidence: f32,
    pub context_adjusted: f32,
    pub experience_factor: f32,
    pub final_confidence: f32,
    pub reasoning: Vec<String>,
}

/// Adaptive confidence calibrator that learns from real outcomes
pub struct ConfidenceCalibrator {
    task_success_rates: HashMap<TaskType, SuccessTracker>,
    context_modifiers: HashMap<String, f32>,
    global_success_rate: f32,
    total_interactions: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SuccessTracker {
    successes: u32,
    total_attempts: u32,
    recent_performance: Vec<bool>, // Last 20 outcomes for trend analysis
    confidence_history: Vec<(f32, bool)>, // (predicted_confidence, actual_outcome)
}

impl SuccessTracker {
    fn new() -> Self {
        Self {
            successes: 0,
            total_attempts: 0,
            recent_performance: Vec::new(),
            confidence_history: Vec::new(),
        }
    }
    
    fn success_rate(&self) -> f32 {
        if self.total_attempts == 0 {
            0.5 // Start with neutral confidence
        } else {
            self.successes as f32 / self.total_attempts as f32
        }
    }
    
    fn recent_trend(&self) -> f32 {
        if self.recent_performance.len() < 3 {
            return self.success_rate();
        }
        
        let recent_successes = self.recent_performance.iter().filter(|&&x| x).count();
        recent_successes as f32 / self.recent_performance.len() as f32
    }
    
    fn confidence_calibration_error(&self) -> f32 {
        if self.confidence_history.is_empty() {
            return 0.0;
        }
        
        // Calculate how well our predicted confidence matches actual outcomes
        let mut total_error = 0.0;
        for (predicted_confidence, actual_outcome) in &self.confidence_history {
            let actual_value = if *actual_outcome { 1.0 } else { 0.0 };
            total_error += (predicted_confidence - actual_value).abs();
        }
        
        total_error / self.confidence_history.len() as f32
    }
}

impl ConfidenceCalibrator {
    pub fn new() -> Self {
        Self {
            task_success_rates: HashMap::new(),
            context_modifiers: HashMap::new(),
            global_success_rate: 0.5,
            total_interactions: 0,
        }
    }
    
    /// Calculate intelligent confidence based on multiple factors
    pub fn calculate_confidence(
        &self, 
        task_type: &TaskType, 
        pattern_score: f32,
        context: &str,
        user_history: Option<&UserHistory>
    ) -> ConfidenceScore {
        let mut reasoning = Vec::new();
        
        // Base confidence from pattern matching strength
        let base_confidence = pattern_score;
        reasoning.push(format!("Pattern match strength: {:.2}", pattern_score));
        
        // Historical success rate for this task type
        let task_success_rate = self.task_success_rates
            .get(task_type)
            .map(|tracker| tracker.success_rate())
            .unwrap_or(0.5);
        
        reasoning.push(format!("Historical success rate for {:?}: {:.2}", task_type, task_success_rate));
        
        // Recent performance trend
        let recent_trend = self.task_success_rates
            .get(task_type)
            .map(|tracker| tracker.recent_trend())
            .unwrap_or(0.5);
        
        if (recent_trend - task_success_rate).abs() > 0.1 {
            reasoning.push(format!("Recent trend: {:.2} (vs overall {:.2})", recent_trend, task_success_rate));
        }
        
        // Context adjustments
        let context_adjustment = self.calculate_context_adjustment(context);
        reasoning.push(format!("Context adjustment: {:+.2}", context_adjustment));
        
        // User experience factor
        let experience_factor = user_history
            .map(|h| self.calculate_experience_factor(h, task_type))
            .unwrap_or(1.0);
        
        if experience_factor != 1.0 {
            reasoning.push(format!("User experience factor: {:.2}", experience_factor));
        }
        
        // Combine factors intelligently
        let context_adjusted = (base_confidence * 0.4 + task_success_rate * 0.3 + recent_trend * 0.3)
            .max(0.0).min(1.0);
        
        let context_modified = (context_adjusted + context_adjustment)
            .max(0.0).min(1.0);
        
        let final_confidence = (context_modified * experience_factor)
            .max(0.05).min(0.98); // Keep within reasonable bounds
        
        // Apply confidence calibration correction
        let calibrated_confidence = self.apply_calibration_correction(task_type, final_confidence);
        
        ConfidenceScore {
            base_confidence,
            context_adjusted: context_modified,
            experience_factor,
            final_confidence: calibrated_confidence,
            reasoning,
        }
    }
    
    /// Calculate context-based adjustments to confidence
    fn calculate_context_adjustment(&self, context: &str) -> f32 {
        let mut adjustment = 0.0;
        
        // Check for complexity indicators that might reduce confidence
        if context.contains("complex") || context.contains("complicated") {
            adjustment -= 0.1;
        }
        
        // Check for urgency indicators that might affect confidence
        if context.contains("urgent") || context.contains("asap") {
            adjustment -= 0.05; // Urgency might lead to more errors
        }
        
        // Check for clarity indicators that might increase confidence
        if context.contains("clear") || context.contains("specific") {
            adjustment += 0.05;
        }
        
        // Check for uncertainty indicators
        if context.contains("maybe") || context.contains("might") || context.contains("not sure") {
            adjustment -= 0.15;
        }
        
        // Apply learned context modifiers
        for (context_key, modifier) in &self.context_modifiers {
            if context.contains(context_key) {
                adjustment += modifier;
            }
        }
        
        adjustment.max(-0.3).min(0.3) // Limit context adjustment range
    }
    
    /// Calculate experience factor based on user history
    fn calculate_experience_factor(&self, user_history: &UserHistory, task_type: &TaskType) -> f32 {
        // Users with more experience in a task type get higher confidence
        let task_experience = user_history.task_experience.get(task_type).unwrap_or(&0);
        let experience_boost = (*task_experience as f32 * 0.01).min(0.2);
        
        // Users with high overall success rate get confidence boost
        let success_boost = if user_history.overall_success_rate > 0.8 {
            0.1
        } else if user_history.overall_success_rate < 0.5 {
            -0.1
        } else {
            0.0
        };
        
        1.0 + experience_boost + success_boost
    }
    
    /// Apply calibration correction based on historical prediction accuracy
    fn apply_calibration_correction(&self, task_type: &TaskType, predicted_confidence: f32) -> f32 {
        if let Some(tracker) = self.task_success_rates.get(task_type) {
            let calibration_error = tracker.confidence_calibration_error();
            
            // If we consistently over-predict, reduce confidence
            // If we consistently under-predict, increase confidence
            if calibration_error > 0.2 {
                // We're not well calibrated, be more conservative
                predicted_confidence * 0.9
            } else {
                predicted_confidence
            }
        } else {
            predicted_confidence
        }
    }
    
    /// Learn from actual outcomes to improve confidence calibration
    pub fn learn_from_outcome(
        &mut self, 
        task_type: TaskType, 
        predicted_confidence: f32,
        actual_success: bool,
        context: &str
    ) -> Result<()> {
        // Update task-specific success tracking
        let tracker = self.task_success_rates.entry(task_type).or_insert_with(SuccessTracker::new);
        
        tracker.total_attempts += 1;
        if actual_success {
            tracker.successes += 1;
        }
        
        // Update recent performance (keep last 20)
        tracker.recent_performance.push(actual_success);
        if tracker.recent_performance.len() > 20 {
            tracker.recent_performance.remove(0);
        }
        
        // Update confidence history (keep last 50)
        tracker.confidence_history.push((predicted_confidence, actual_success));
        if tracker.confidence_history.len() > 50 {
            tracker.confidence_history.remove(0);
        }
        
        // Update global statistics
        self.total_interactions += 1;
        if actual_success {
            self.global_success_rate = (self.global_success_rate * (self.total_interactions - 1) as f32 + 1.0) 
                / self.total_interactions as f32;
        } else {
            self.global_success_rate = (self.global_success_rate * (self.total_interactions - 1) as f32) 
                / self.total_interactions as f32;
        }
        
        // Learn context modifiers
        self.learn_context_modifiers(context, actual_success, predicted_confidence)?;
        
        Ok(())
    }
    
    /// Learn how different context words affect success rates
    fn learn_context_modifiers(&mut self, context: &str, success: bool, confidence: f32) -> Result<()> {
        let words: Vec<&str> = context.split_whitespace()
            .filter(|word| word.len() > 3) // Ignore short words
            .collect();
        
        for word in words {
            let word = word.to_lowercase();
            
            // Calculate how this word correlates with success
            let outcome_value = if success { 1.0 } else { 0.0 };
            let prediction_error = (confidence - outcome_value).abs();
            
            // If this word appears in contexts where we consistently over/under predict,
            // learn to adjust for it
            let current_modifier = self.context_modifiers.get(&word).unwrap_or(&0.0);
            
            // Update modifier using exponential moving average
            let learning_rate = 0.05;
            let adjustment = if success && confidence < 0.7 {
                0.02 // This word might indicate higher success than we predict
            } else if !success && confidence > 0.7 {
                -0.02 // This word might indicate lower success than we predict
            } else {
                0.0
            };
            
            let new_modifier = current_modifier * (1.0 - learning_rate) + adjustment * learning_rate;
            
            // Only keep modifiers that have meaningful impact
            if new_modifier.abs() > 0.01 {
                self.context_modifiers.insert(word, new_modifier);
            } else {
                self.context_modifiers.remove(&word);
            }
        }
        
        Ok(())
    }
    
    /// Get calibration statistics for monitoring and debugging
    pub fn get_calibration_stats(&self) -> CalibrationStats {
        let mut total_attempts = 0;
        let mut total_successes = 0;
        let mut total_calibration_error = 0.0;
        let mut task_stats = HashMap::new();
        
        for (task_type, tracker) in &self.task_success_rates {
            total_attempts += tracker.total_attempts;
            total_successes += tracker.successes;
            total_calibration_error += tracker.confidence_calibration_error();
            
            task_stats.insert(*task_type, TaskCalibrationStats {
                success_rate: tracker.success_rate(),
                total_attempts: tracker.total_attempts,
                recent_trend: tracker.recent_trend(),
                calibration_error: tracker.confidence_calibration_error(),
            });
        }
        
        CalibrationStats {
            global_success_rate: self.global_success_rate,
            total_interactions: self.total_interactions,
            average_calibration_error: if !self.task_success_rates.is_empty() {
                total_calibration_error / self.task_success_rates.len() as f32
            } else {
                0.0
            },
            task_stats,
            learned_context_modifiers: self.context_modifiers.len(),
        }
    }
}

/// User history for experience-based confidence adjustment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserHistory {
    pub task_experience: HashMap<TaskType, u32>,
    pub overall_success_rate: f32,
    pub total_interactions: u32,
}

/// Statistics about confidence calibration performance
#[derive(Debug, Serialize, Deserialize)]
pub struct CalibrationStats {
    pub global_success_rate: f32,
    pub total_interactions: u32,
    pub average_calibration_error: f32,
    pub task_stats: HashMap<TaskType, TaskCalibrationStats>,
    pub learned_context_modifiers: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskCalibrationStats {
    pub success_rate: f32,
    pub total_attempts: u32,
    pub recent_trend: f32,
    pub calibration_error: f32,
}

impl Default for ConfidenceCalibrator {
    fn default() -> Self {
        Self::new()
    }
}