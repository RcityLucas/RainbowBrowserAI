// Organic Perception - The Digital Life Form's Intelligent Understanding
//
// This module implements the four-layer consciousness described in the design documents:
// Lightning (50ms) → Quick (200ms) → Standard (500ms) → Deep (1000ms)
//
// It replaces hardcoded keyword matching with organic, learning-based understanding.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::time::Instant;
use tracing::{info, debug};

use crate::llm_service::llm_service_enhanced::TaskType;
use super::{
    patterns::{PatternMatcher, PatternStats},
    confidence::{ConfidenceCalibrator, ConfidenceScore, UserHistory, CalibrationStats},
    learning::{LearningEngine, InteractionOutcome, LearningInsights, UserFeedback},
};

/// Organic perception system with four consciousness layers
pub struct OrganicPerception {
    pattern_matcher: PatternMatcher,
    confidence_calibrator: ConfidenceCalibrator,
    learning_engine: LearningEngine,
    perception_mode: PerceptionMode,
}

/// Four-layer consciousness as described in design documents
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum PerceptionMode {
    Lightning, // <50ms - 本能反应 (Instinctive reaction)
    Quick,     // <200ms - 感官知觉 (Sensory perception)
    Standard,  // <500ms - 认知理解 (Cognitive understanding)
    Deep,      // <1000ms - 智慧洞察 (Wisdom insight)
}

/// Complete understanding result from organic perception
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentUnderstanding {
    pub task_type: TaskType,
    pub confidence: ConfidenceScore,
    pub context_analysis: ContextAnalysis,
    pub processing_time_ms: u64,
    pub perception_layer: PerceptionMode,
    pub learning_applied: bool,
    pub reasoning: Vec<String>,
}

/// Context analysis for situational understanding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextAnalysis {
    pub complexity_level: f32,
    pub urgency_indicators: Vec<String>,
    pub clarity_score: f32,
    pub user_expertise_estimate: f32,
    pub environment_factors: Vec<String>,
}

/// Context for perception understanding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Context {
    pub user_input: String,
    pub environment: String,
    pub user_history: Option<UserHistory>,
    pub time_constraints: Option<u64>, // milliseconds
    pub complexity_preference: Option<PerceptionMode>,
}

impl OrganicPerception {
    pub fn new() -> Self {
        Self {
            pattern_matcher: PatternMatcher::new(),
            confidence_calibrator: ConfidenceCalibrator::new(),
            learning_engine: LearningEngine::new(),
            perception_mode: PerceptionMode::Standard,
        }
    }
    
    /// Main perception function - understand intent with organic intelligence
    pub async fn understand_intent(&mut self, context: &Context) -> Result<IntentUnderstanding> {
        let start_time = Instant::now();
        
        // Determine optimal perception layer based on context
        let perception_layer = self.select_perception_layer(context);
        
        info!("🧠 Organic perception starting: {:?} layer", perception_layer);
        
        // Lightning Layer: Instinctive pattern matching (<50ms)
        let pattern_matches = self.lightning_perception(&context.user_input, &context.environment)?;
        
        if matches!(perception_layer, PerceptionMode::Lightning) {
            return self.create_lightning_understanding(context, pattern_matches, start_time);
        }
        
        // Quick Layer: Context-aware understanding (<200ms)
        let context_analysis = self.quick_perception(context, &pattern_matches)?;
        
        if matches!(perception_layer, PerceptionMode::Quick) {
            return self.create_quick_understanding(context, pattern_matches, context_analysis, start_time);
        }
        
        // Standard Layer: Intelligent confidence calibration (<500ms)
        let confidence_analysis = self.standard_perception(context, &pattern_matches, &context_analysis)?;
        
        if matches!(perception_layer, PerceptionMode::Standard) {
            return self.create_standard_understanding(context, pattern_matches, context_analysis, confidence_analysis, start_time);
        }
        
        // Deep Layer: Learning-enhanced wisdom (<1000ms)
        let wisdom_insights = self.deep_perception(context, &pattern_matches, &confidence_analysis).await?;
        
        self.create_deep_understanding(context, pattern_matches, context_analysis, confidence_analysis, wisdom_insights, start_time)
    }
    
    /// Select optimal perception layer based on context and constraints
    fn select_perception_layer(&self, context: &Context) -> PerceptionMode {
        // User preference takes priority
        if let Some(preference) = context.complexity_preference {
            return preference;
        }
        
        // Time constraints influence layer selection
        if let Some(time_limit) = context.time_constraints {
            return match time_limit {
                0..=50 => PerceptionMode::Lightning,
                51..=200 => PerceptionMode::Quick,
                201..=500 => PerceptionMode::Standard,
                _ => PerceptionMode::Deep,
            };
        }
        
        // Analyze input complexity to choose appropriate layer
        let complexity = self.estimate_input_complexity(&context.user_input);
        
        match complexity {
            0.0..=0.3 => PerceptionMode::Lightning,  // Simple, clear requests
            0.3..=0.6 => PerceptionMode::Quick,     // Moderate complexity
            0.6..=0.8 => PerceptionMode::Standard,  // Complex requests
            _ => PerceptionMode::Deep,              // Very complex or novel requests
        }
    }
    
    /// Lightning Layer: Instinctive pattern matching (<50ms)
    fn lightning_perception(&self, input: &str, environment: &str) -> Result<Vec<(TaskType, f32)>> {
        debug!("⚡ Lightning perception: Instinctive pattern matching");
        
        // Fast pattern matching with minimal processing
        let matches = self.pattern_matcher.match_patterns(input, environment);
        
        // Return top 3 matches for speed
        Ok(matches.into_iter().take(3).collect())
    }
    
    /// Quick Layer: Context-aware understanding (<200ms)
    fn quick_perception(&self, context: &Context, pattern_matches: &[(TaskType, f32)]) -> Result<ContextAnalysis> {
        debug!("🔍 Quick perception: Environmental sensing");
        
        let context_analysis = ContextAnalysis {
            complexity_level: self.estimate_input_complexity(&context.user_input),
            urgency_indicators: self.detect_urgency_indicators(&context.user_input),
            clarity_score: self.calculate_clarity_score(&context.user_input),
            user_expertise_estimate: self.estimate_user_expertise(context),
            environment_factors: self.analyze_environment_factors(&context.environment),
        };
        
        Ok(context_analysis)
    }
    
    /// Standard Layer: Intelligent confidence calibration (<500ms)
    fn standard_perception(
        &self, 
        context: &Context, 
        pattern_matches: &[(TaskType, f32)],
        context_analysis: &ContextAnalysis
    ) -> Result<ConfidenceScore> {
        debug!("🎯 Standard perception: Cognitive understanding");
        
        if let Some((best_task_type, pattern_score)) = pattern_matches.first() {
            let confidence = self.confidence_calibrator.calculate_confidence(
                best_task_type,
                *pattern_score,
                &context.environment,
                context.user_history.as_ref(),
            );
            
            Ok(confidence)
        } else {
            // No patterns matched - return unknown with low confidence
            Ok(ConfidenceScore {
                base_confidence: 0.0,
                context_adjusted: 0.0,
                experience_factor: 1.0,
                final_confidence: 0.1,
                reasoning: vec!["No patterns matched input".to_string()],
            })
        }
    }
    
    /// Deep Layer: Learning-enhanced wisdom (<1000ms)
    async fn deep_perception(
        &self,
        context: &Context,
        pattern_matches: &[(TaskType, f32)],
        confidence_analysis: &ConfidenceScore
    ) -> Result<WisdomInsights> {
        debug!("🧘 Deep perception: Wisdom synthesis");
        
        let mut insights = WisdomInsights {
            learned_patterns_applied: Vec::new(),
            novel_pattern_discovered: false,
            user_intent_refined: false,
            alternative_interpretations: Vec::new(),
            confidence_adjustments: Vec::new(),
        };
        
        // Apply learned patterns from previous interactions
        if let Some((best_task_type, _)) = pattern_matches.first() {
            let relevant_patterns = self.learning_engine.get_relevant_patterns(&context.user_input, *best_task_type);
            
            for pattern in relevant_patterns {
                insights.learned_patterns_applied.push(format!(
                    "Pattern '{}' (impact: {:+.2})", 
                    pattern.pattern_text, 
                    pattern.confidence_impact
                ));
            }
        }
        
        // Analyze for alternative interpretations
        if pattern_matches.len() > 1 {
            for (task_type, score) in pattern_matches.iter().skip(1).take(2) {
                if *score > 0.3 { // Significant alternative
                    insights.alternative_interpretations.push(format!(
                        "{:?} (score: {:.2})", 
                        task_type, 
                        score
                    ));
                }
            }
        }
        
        // Check for novel patterns that we haven't seen before
        let input_words: Vec<&str> = context.user_input.split_whitespace().collect();
        if input_words.len() > 2 && confidence_analysis.final_confidence < 0.5 {
            insights.novel_pattern_discovered = true;
            insights.confidence_adjustments.push("Novel pattern detected - reducing confidence".to_string());
        }
        
        Ok(insights)
    }
    
    /// Create lightning-speed understanding result
    fn create_lightning_understanding(
        &self,
        context: &Context,
        pattern_matches: Vec<(TaskType, f32)>,
        start_time: Instant,
    ) -> Result<IntentUnderstanding> {
        let (task_type, pattern_score) = pattern_matches
            .first()
            .copied()
            .unwrap_or((TaskType::Unknown, 0.1));
        
        let confidence = ConfidenceScore {
            base_confidence: pattern_score,
            context_adjusted: pattern_score,
            experience_factor: 1.0,
            final_confidence: pattern_score * 0.8, // Reduce confidence for lightning mode
            reasoning: vec!["Lightning-fast pattern match".to_string()],
        };
        
        Ok(IntentUnderstanding {
            task_type,
            confidence,
            context_analysis: ContextAnalysis {
                complexity_level: 0.3,
                urgency_indicators: vec!["lightning mode".to_string()],
                clarity_score: 0.5,
                user_expertise_estimate: 0.5,
                environment_factors: vec![],
            },
            processing_time_ms: start_time.elapsed().as_millis() as u64,
            perception_layer: PerceptionMode::Lightning,
            learning_applied: false,
            reasoning: vec!["Instinctive pattern recognition".to_string()],
        })
    }
    
    /// Create quick understanding result
    fn create_quick_understanding(
        &self,
        context: &Context,
        pattern_matches: Vec<(TaskType, f32)>,
        context_analysis: ContextAnalysis,
        start_time: Instant,
    ) -> Result<IntentUnderstanding> {
        let (task_type, pattern_score) = pattern_matches
            .first()
            .copied()
            .unwrap_or((TaskType::Unknown, 0.1));
        
        // Quick confidence calculation
        let confidence_boost = context_analysis.clarity_score * 0.1;
        let final_confidence = (pattern_score + confidence_boost).min(0.9);
        
        let confidence = ConfidenceScore {
            base_confidence: pattern_score,
            context_adjusted: final_confidence,
            experience_factor: 1.0,
            final_confidence,
            reasoning: vec![
                format!("Pattern match: {:.2}", pattern_score),
                format!("Clarity boost: {:+.2}", confidence_boost),
            ],
        };
        
        Ok(IntentUnderstanding {
            task_type,
            confidence,
            context_analysis,
            processing_time_ms: start_time.elapsed().as_millis() as u64,
            perception_layer: PerceptionMode::Quick,
            learning_applied: false,
            reasoning: vec!["Context-aware understanding".to_string()],
        })
    }
    
    /// Create standard understanding result
    fn create_standard_understanding(
        &self,
        context: &Context,
        pattern_matches: Vec<(TaskType, f32)>,
        context_analysis: ContextAnalysis,
        confidence_analysis: ConfidenceScore,
        start_time: Instant,
    ) -> Result<IntentUnderstanding> {
        let task_type = pattern_matches
            .first()
            .map(|(tt, _)| *tt)
            .unwrap_or(TaskType::Unknown);
        
        Ok(IntentUnderstanding {
            task_type,
            confidence: confidence_analysis,
            context_analysis,
            processing_time_ms: start_time.elapsed().as_millis() as u64,
            perception_layer: PerceptionMode::Standard,
            learning_applied: false,
            reasoning: vec!["Cognitive understanding with calibrated confidence".to_string()],
        })
    }
    
    /// Create deep understanding result with wisdom
    fn create_deep_understanding(
        &self,
        context: &Context,
        pattern_matches: Vec<(TaskType, f32)>,
        context_analysis: ContextAnalysis,
        mut confidence_analysis: ConfidenceScore,
        wisdom_insights: WisdomInsights,
        start_time: Instant,
    ) -> Result<IntentUnderstanding> {
        let task_type = pattern_matches
            .first()
            .map(|(tt, _)| *tt)
            .unwrap_or(TaskType::Unknown);
        
        // Apply wisdom insights to adjust confidence
        if wisdom_insights.novel_pattern_discovered {
            confidence_analysis.final_confidence *= 0.8; // Reduce confidence for novel patterns
        }
        
        let mut reasoning = vec!["Deep wisdom synthesis".to_string()];
        reasoning.extend(confidence_analysis.reasoning.clone());
        reasoning.extend(wisdom_insights.confidence_adjustments);
        
        if !wisdom_insights.learned_patterns_applied.is_empty() {
            reasoning.push(format!("Applied {} learned patterns", wisdom_insights.learned_patterns_applied.len()));
        }
        
        if !wisdom_insights.alternative_interpretations.is_empty() {
            reasoning.push(format!("Found {} alternative interpretations", wisdom_insights.alternative_interpretations.len()));
        }
        
        Ok(IntentUnderstanding {
            task_type,
            confidence: confidence_analysis,
            context_analysis,
            processing_time_ms: start_time.elapsed().as_millis() as u64,
            perception_layer: PerceptionMode::Deep,
            learning_applied: !wisdom_insights.learned_patterns_applied.is_empty(),
            reasoning,
        })
    }
    
    /// Learn from interaction outcome to improve future understanding
    pub async fn learn_from_outcome(
        &mut self,
        understanding: &IntentUnderstanding,
        actual_success: bool,
        execution_time_ms: u64,
        error_type: Option<String>,
        user_feedback: Option<UserFeedback>,
    ) -> Result<()> {
        info!("📚 Learning from outcome: success={}, confidence={:.2}", 
               actual_success, understanding.confidence.final_confidence);
        
        // Learn in pattern matcher
        self.pattern_matcher.learn_from_outcome(
            &understanding.context_analysis.environment_factors.join(" "),
            understanding.task_type,
            actual_success,
        )?;
        
        // Learn in confidence calibrator
        self.confidence_calibrator.learn_from_outcome(
            understanding.task_type,
            understanding.confidence.final_confidence,
            actual_success,
            &understanding.context_analysis.environment_factors.join(" "),
        )?;
        
        // Learn in learning engine
        let interaction_outcome = InteractionOutcome {
            timestamp: chrono::Utc::now(),
            user_input: understanding.context_analysis.environment_factors.join(" "), // Approximation
            predicted_task_type: understanding.task_type,
            predicted_confidence: understanding.confidence.final_confidence,
            actual_success,
            execution_time_ms,
            error_type,
            context: understanding.context_analysis.environment_factors.join(" "),
            user_feedback,
        };
        
        self.learning_engine.observe_interaction(interaction_outcome)?;
        
        Ok(())
    }
    
    /// Get intelligence statistics for monitoring
    pub fn get_intelligence_stats(&self) -> IntelligenceStats {
        IntelligenceStats {
            pattern_stats: self.pattern_matcher.get_learning_stats(),
            calibration_stats: self.confidence_calibrator.get_calibration_stats(),
            learning_insights: self.learning_engine.get_learning_insights(),
        }
    }
    
    // Helper methods for analysis
    
    fn estimate_input_complexity(&self, input: &str) -> f32 {
        let word_count = input.split_whitespace().count();
        let unique_words = input.split_whitespace().collect::<std::collections::HashSet<_>>().len();
        let avg_word_length = input.split_whitespace()
            .map(|w| w.len())
            .sum::<usize>() as f32 / word_count.max(1) as f32;
        
        // Complexity factors
        let length_factor = (word_count as f32 / 20.0).min(1.0);
        let vocabulary_factor = (unique_words as f32 / word_count.max(1) as f32);
        let sophistication_factor = (avg_word_length / 10.0).min(1.0);
        
        (length_factor * 0.4 + vocabulary_factor * 0.3 + sophistication_factor * 0.3).min(1.0)
    }
    
    fn detect_urgency_indicators(&self, input: &str) -> Vec<String> {
        let urgency_words = ["urgent", "asap", "immediately", "quickly", "fast", "now", "emergency"];
        let input_lower = input.to_lowercase();
        
        urgency_words.iter()
            .filter(|&&word| input_lower.contains(word))
            .map(|&word| word.to_string())
            .collect()
    }
    
    fn calculate_clarity_score(&self, input: &str) -> f32 {
        let clear_indicators = ["specific", "exactly", "precisely", "clearly"];
        let unclear_indicators = ["maybe", "perhaps", "might", "not sure", "unclear"];
        
        let input_lower = input.to_lowercase();
        
        let clear_count = clear_indicators.iter().filter(|&&word| input_lower.contains(word)).count();
        let unclear_count = unclear_indicators.iter().filter(|&&word| input_lower.contains(word)).count();
        
        let base_clarity = 0.5;
        let clarity_boost = clear_count as f32 * 0.2;
        let clarity_penalty = unclear_count as f32 * 0.3;
        
        (base_clarity + clarity_boost - clarity_penalty).max(0.0).min(1.0)
    }
    
    fn estimate_user_expertise(&self, context: &Context) -> f32 {
        // In a full implementation, this would use user history
        // For now, estimate based on input sophistication
        let technical_terms = ["API", "database", "server", "configuration", "parameters"];
        let input_lower = context.user_input.to_lowercase();
        
        let technical_count = technical_terms.iter()
            .filter(|&&term| input_lower.contains(&term.to_lowercase()))
            .count();
        
        (0.3 + technical_count as f32 * 0.2).min(1.0)
    }
    
    fn analyze_environment_factors(&self, environment: &str) -> Vec<String> {
        // Extract environment context indicators
        let mut factors = Vec::new();
        
        if environment.contains("production") {
            factors.push("production-environment".to_string());
        }
        if environment.contains("development") {
            factors.push("development-environment".to_string());
        }
        if environment.contains("test") {
            factors.push("test-environment".to_string());
        }
        
        factors
    }
}

#[derive(Debug)]
struct WisdomInsights {
    learned_patterns_applied: Vec<String>,
    novel_pattern_discovered: bool,
    user_intent_refined: bool,
    alternative_interpretations: Vec<String>,
    confidence_adjustments: Vec<String>,
}

/// Comprehensive intelligence statistics
#[derive(Debug, Serialize, Deserialize)]
pub struct IntelligenceStats {
    pub pattern_stats: PatternStats,
    pub calibration_stats: CalibrationStats,
    pub learning_insights: LearningInsights,
}

impl Default for OrganicPerception {
    fn default() -> Self {
        Self::new()
    }
}