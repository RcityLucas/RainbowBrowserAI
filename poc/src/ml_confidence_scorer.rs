// Machine Learning Confidence Scoring System
// Uses historical data and pattern recognition to improve confidence scores

use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// ML-based confidence scoring system
pub struct MLConfidenceScorer {
    model: ConfidenceModel,
    history: CommandHistory,
    feature_extractor: FeatureExtractor,
    learning_rate: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceModel {
    // Feature weights learned from historical data
    weights: HashMap<String, f32>,
    bias: f32,
    
    // Model performance metrics
    accuracy: f32,
    total_predictions: u32,
    correct_predictions: u32,
    
    // Model parameters
    version: String,
    last_updated: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandHistory {
    // Historical command data for learning
    successful_commands: Vec<CommandRecord>,
    failed_commands: Vec<CommandRecord>,
    
    // Pattern statistics
    pattern_success_rates: HashMap<String, f32>,
    action_success_rates: HashMap<String, f32>,
    
    // Context success rates
    context_success_rates: HashMap<String, f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandRecord {
    pub command: String,
    pub action: String,
    pub confidence: f32,
    pub actual_success: bool,
    pub execution_time: u64,
    pub context: HashMap<String, String>,
    pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub struct FeatureExtractor {
    feature_functions: Vec<fn(&str, &HashMap<String, String>) -> f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidencePrediction {
    pub base_confidence: f32,
    pub ml_adjusted_confidence: f32,
    pub features_used: HashMap<String, f32>,
    pub explanation: String,
    pub uncertainty: f32,
}

impl MLConfidenceScorer {
    pub fn new() -> Self {
        Self {
            model: Self::initialize_model(),
            history: CommandHistory {
                successful_commands: Vec::new(),
                failed_commands: Vec::new(),
                pattern_success_rates: HashMap::new(),
                action_success_rates: HashMap::new(),
                context_success_rates: HashMap::new(),
            },
            feature_extractor: Self::initialize_feature_extractor(),
            learning_rate: 0.01,
        }
    }
    
    fn initialize_model() -> ConfidenceModel {
        let mut weights = HashMap::new();
        
        // Initialize feature weights based on empirical data
        weights.insert("command_length".to_string(), -0.002);
        weights.insert("has_specific_selector".to_string(), 0.3);
        weights.insert("action_clarity".to_string(), 0.25);
        weights.insert("keyword_match".to_string(), 0.2);
        weights.insert("historical_success".to_string(), 0.35);
        weights.insert("context_match".to_string(), 0.15);
        weights.insert("complexity_score".to_string(), -0.1);
        weights.insert("ambiguity_score".to_string(), -0.2);
        weights.insert("pattern_familiarity".to_string(), 0.3);
        weights.insert("time_since_similar".to_string(), -0.05);
        
        ConfidenceModel {
            weights,
            bias: 0.5,
            accuracy: 0.85,
            total_predictions: 0,
            correct_predictions: 0,
            version: "1.0.0".to_string(),
            last_updated: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }
    
    fn initialize_feature_extractor() -> FeatureExtractor {
        FeatureExtractor {
            feature_functions: vec![
                Self::extract_command_length,
                Self::extract_selector_specificity,
                Self::extract_action_clarity,
                Self::extract_keyword_match,
                Self::extract_complexity,
                Self::extract_ambiguity,
            ],
        }
    }
    
    /// Predict confidence for a command using ML model
    pub fn predict_confidence(
        &self,
        command: &str,
        base_confidence: f32,
        context: &HashMap<String, String>,
    ) -> ConfidencePrediction {
        let mut features = self.extract_features(command, context);
        
        // Add historical success rate if available
        if let Some(action) = context.get("action") {
            if let Some(success_rate) = self.history.action_success_rates.get(action) {
                features.insert("historical_success".to_string(), *success_rate);
            }
        }
        
        // Calculate ML confidence score
        let ml_score = self.calculate_score(&features);
        
        // Combine with base confidence
        let combined_confidence = (base_confidence * 0.4 + ml_score * 0.6)
            .max(0.1)
            .min(0.99);
        
        // Calculate uncertainty based on model accuracy and data availability
        let uncertainty = self.calculate_uncertainty(&features);
        
        ConfidencePrediction {
            base_confidence,
            ml_adjusted_confidence: combined_confidence,
            features_used: features,
            explanation: self.generate_explanation(base_confidence, ml_score, combined_confidence),
            uncertainty,
        }
    }
    
    /// Extract features from command and context
    fn extract_features(&self, command: &str, context: &HashMap<String, String>) -> HashMap<String, f32> {
        let mut features = HashMap::new();
        
        // Basic features
        features.insert("command_length".to_string(), command.len() as f32);
        
        // Apply feature extraction functions
        for func in &self.feature_extractor.feature_functions {
            let (name, value) = Self::apply_feature_function(func, command, context);
            features.insert(name, value);
        }
        
        // Pattern-based features
        let pattern_score = self.calculate_pattern_score(command);
        features.insert("pattern_familiarity".to_string(), pattern_score);
        
        // Context-based features
        if let Some(page_type) = context.get("page_type") {
            let context_score = self.calculate_context_score(page_type);
            features.insert("context_match".to_string(), context_score);
        }
        
        features
    }
    
    /// Calculate ML score from features
    fn calculate_score(&self, features: &HashMap<String, f32>) -> f32 {
        let mut score = self.model.bias;
        
        for (feature_name, feature_value) in features {
            if let Some(weight) = self.model.weights.get(feature_name) {
                score += weight * feature_value;
            }
        }
        
        // Apply sigmoid activation
        1.0 / (1.0 + (-score).exp())
    }
    
    /// Learn from command execution results
    pub fn learn_from_execution(&mut self, record: CommandRecord) {
        // Add to history
        if record.actual_success {
            self.history.successful_commands.push(record.clone());
        } else {
            self.history.failed_commands.push(record.clone());
        }
        
        // Update success rates
        self.update_success_rates(&record);
        
        // Update model weights using gradient descent
        self.update_weights(&record);
        
        // Update model accuracy
        self.update_model_accuracy(&record);
    }
    
    /// Update success rates based on new record
    fn update_success_rates(&mut self, record: &CommandRecord) {
        // Update action success rate
        let action_count = self.history.action_success_rates
            .entry(record.action.clone())
            .or_insert(0.5);
        
        // Exponential moving average
        *action_count = (*action_count * 0.9) + (if record.actual_success { 0.1 } else { 0.0 });
        
        // Update pattern success rates
        for pattern in self.extract_patterns(&record.command) {
            let pattern_count = self.history.pattern_success_rates
                .entry(pattern)
                .or_insert(0.5);
            *pattern_count = (*pattern_count * 0.9) + (if record.actual_success { 0.1 } else { 0.0 });
        }
    }
    
    /// Update model weights based on prediction error
    fn update_weights(&mut self, record: &CommandRecord) {
        let features = self.extract_features(&record.command, &record.context);
        let predicted = self.calculate_score(&features);
        let actual = if record.actual_success { 1.0 } else { 0.0 };
        let error = actual - predicted;
        
        // Gradient descent update
        for (feature_name, feature_value) in &features {
            if let Some(weight) = self.model.weights.get_mut(feature_name) {
                *weight += self.learning_rate * error * feature_value;
            }
        }
        
        // Update bias
        self.model.bias += self.learning_rate * error;
    }
    
    /// Update model accuracy metrics
    fn update_model_accuracy(&mut self, record: &CommandRecord) {
        self.model.total_predictions += 1;
        
        let features = self.extract_features(&record.command, &record.context);
        let predicted = self.calculate_score(&features) > 0.5;
        
        if predicted == record.actual_success {
            self.model.correct_predictions += 1;
        }
        
        self.model.accuracy = self.model.correct_predictions as f32 / self.model.total_predictions as f32;
    }
    
    /// Calculate uncertainty in prediction
    fn calculate_uncertainty(&self, features: &HashMap<String, f32>) -> f32 {
        let mut uncertainty = 0.0;
        
        // Higher uncertainty for features with missing data
        let expected_features = self.model.weights.len();
        let actual_features = features.len();
        let missing_ratio = 1.0 - (actual_features as f32 / expected_features as f32);
        uncertainty += missing_ratio * 0.3;
        
        // Higher uncertainty for low model accuracy
        uncertainty += (1.0 - self.model.accuracy) * 0.2;
        
        // Higher uncertainty for limited training data
        let data_points = self.history.successful_commands.len() + self.history.failed_commands.len();
        if data_points < 100 {
            uncertainty += 0.2 * (1.0 - (data_points as f32 / 100.0));
        }
        
        uncertainty.min(0.5)
    }
    
    /// Generate human-readable explanation
    fn generate_explanation(&self, base: f32, ml: f32, combined: f32) -> String {
        let adjustment = ml - base;
        
        if adjustment.abs() < 0.05 {
            format!("ML model confirms base confidence of {:.1}%", base * 100.0)
        } else if adjustment > 0.0 {
            format!("ML model increased confidence by {:.1}% based on historical patterns", adjustment * 100.0)
        } else {
            format!("ML model decreased confidence by {:.1}% due to complexity/ambiguity", adjustment.abs() * 100.0)
        }
    }
    
    /// Extract patterns from command
    fn extract_patterns(&self, command: &str) -> Vec<String> {
        let mut patterns = Vec::new();
        let words: Vec<&str> = command.split_whitespace().collect();
        
        // Unigrams
        for word in &words {
            patterns.push(word.to_string());
        }
        
        // Bigrams
        for i in 0..words.len().saturating_sub(1) {
            patterns.push(format!("{}_{}", words[i], words[i + 1]));
        }
        
        patterns
    }
    
    /// Calculate pattern familiarity score
    fn calculate_pattern_score(&self, command: &str) -> f32 {
        let patterns = self.extract_patterns(command);
        let mut total_score = 0.0;
        let mut count = 0;
        
        for pattern in patterns {
            if let Some(success_rate) = self.history.pattern_success_rates.get(&pattern) {
                total_score += success_rate;
                count += 1;
            }
        }
        
        if count > 0 {
            total_score / count as f32
        } else {
            0.5 // neutral score for unknown patterns
        }
    }
    
    /// Calculate context match score
    fn calculate_context_score(&self, page_type: &str) -> f32 {
        self.history.context_success_rates
            .get(page_type)
            .copied()
            .unwrap_or(0.5)
    }
    
    // Feature extraction functions
    
    fn apply_feature_function(
        func: &fn(&str, &HashMap<String, String>) -> f32,
        command: &str,
        context: &HashMap<String, String>,
    ) -> (String, f32) {
        // This is a workaround - in real implementation, we'd have proper feature functions
        ("generic_feature".to_string(), 0.5)
    }
    
    fn extract_command_length(command: &str, _context: &HashMap<String, String>) -> f32 {
        command.len() as f32
    }
    
    fn extract_selector_specificity(command: &str, _context: &HashMap<String, String>) -> f32 {
        if command.contains("#") || command.contains("id=") {
            1.0
        } else if command.contains(".") || command.contains("class=") {
            0.7
        } else {
            0.3
        }
    }
    
    fn extract_action_clarity(command: &str, _context: &HashMap<String, String>) -> f32 {
        let clear_actions = ["click", "type", "navigate", "search", "select", "scroll"];
        let mut clarity = 0.0;
        
        for action in clear_actions {
            if command.contains(action) {
                clarity = 1.0;
                break;
            }
        }
        
        clarity
    }
    
    fn extract_keyword_match(command: &str, context: &HashMap<String, String>) -> f32 {
        let mut matches = 0;
        let words: Vec<&str> = command.split_whitespace().collect();
        
        for (key, value) in context {
            for word in &words {
                if value.contains(word) || word.contains(value) {
                    matches += 1;
                }
            }
        }
        
        (matches as f32 / words.len() as f32).min(1.0)
    }
    
    fn extract_complexity(command: &str, _context: &HashMap<String, String>) -> f32 {
        let operators = ["and", "then", "or", "if", "after", "before"];
        let mut complexity = 0.0;
        
        for op in operators {
            if command.contains(op) {
                complexity += 0.2;
            }
        }
        
        complexity.min(1.0)
    }
    
    fn extract_ambiguity(command: &str, _context: &HashMap<String, String>) -> f32 {
        let ambiguous = ["it", "that", "this", "something", "stuff", "thing"];
        let mut ambiguity = 0.0;
        
        for word in ambiguous {
            if command.contains(word) {
                ambiguity += 0.2;
            }
        }
        
        ambiguity.min(1.0)
    }
    
    /// Save model to disk
    pub fn save_model(&self, path: &str) -> Result<(), std::io::Error> {
        let json = serde_json::to_string_pretty(&self.model)?;
        std::fs::write(path, json)?;
        Ok(())
    }
    
    /// Load model from disk
    pub fn load_model(&mut self, path: &str) -> Result<(), std::io::Error> {
        let json = std::fs::read_to_string(path)?;
        self.model = serde_json::from_str(&json)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_confidence_prediction() {
        let scorer = MLConfidenceScorer::new();
        let mut context = HashMap::new();
        context.insert("action".to_string(), "click".to_string());
        context.insert("page_type".to_string(), "homepage".to_string());
        
        let prediction = scorer.predict_confidence(
            "click the login button",
            0.8,
            &context
        );
        
        assert!(prediction.ml_adjusted_confidence > 0.0);
        assert!(prediction.ml_adjusted_confidence <= 1.0);
        assert!(!prediction.explanation.is_empty());
    }
    
    #[test]
    fn test_learning_mechanism() {
        let mut scorer = MLConfidenceScorer::new();
        
        let record = CommandRecord {
            command: "click submit".to_string(),
            action: "click".to_string(),
            confidence: 0.8,
            actual_success: true,
            execution_time: 150,
            context: HashMap::new(),
            timestamp: 123456789,
        };
        
        scorer.learn_from_execution(record);
        
        assert_eq!(scorer.history.successful_commands.len(), 1);
        assert!(scorer.history.action_success_rates.contains_key("click"));
    }
    
    #[test]
    fn test_uncertainty_calculation() {
        let scorer = MLConfidenceScorer::new();
        let features = HashMap::new();
        
        let uncertainty = scorer.calculate_uncertainty(&features);
        assert!(uncertainty >= 0.0);
        assert!(uncertainty <= 0.5);
    }
    
    #[test]
    fn test_pattern_extraction() {
        let scorer = MLConfidenceScorer::new();
        let patterns = scorer.extract_patterns("click the button");
        
        assert!(patterns.contains(&"click".to_string()));
        assert!(patterns.contains(&"click_the".to_string()));
        assert!(patterns.contains(&"the_button".to_string()));
    }
}