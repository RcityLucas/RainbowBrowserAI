// Context-aware perception - understanding user intent and workflow context

use anyhow::Result;
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, VecDeque};

/// Context-aware perception engine that learns from user interactions
pub struct ContextAwarePerception {
    interaction_history: VecDeque<InteractionRecord>,
    user_patterns: HashMap<String, UserPattern>,
    workflow_context: WorkflowContext,
    max_history: usize,
}

/// Record of user interaction with context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionRecord {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub action: String,
    pub target_description: String,
    pub target_selector: String,
    pub page_url: String,
    pub page_type: String,
    pub success: bool,
    pub context: InteractionContext,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionContext {
    pub preceding_actions: Vec<String>,
    pub page_state: HashMap<String, String>,
    pub user_intent: Option<String>,
    pub workflow_step: Option<String>,
}

/// User behavior patterns learned over time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPattern {
    pub pattern_type: PatternType,
    pub frequency: u32,
    pub success_rate: f32,
    pub typical_sequence: Vec<String>,
    pub contextual_cues: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatternType {
    LoginSequence,
    SearchWorkflow,
    FormFilling,
    Navigation,
    DataExtraction,
    Shopping,
}

/// Current workflow context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowContext {
    pub current_goal: Option<String>,
    pub current_step: Option<String>,
    pub completed_steps: Vec<String>,
    pub context_data: HashMap<String, String>,
    pub confidence: f32,
}

impl ContextAwarePerception {
    pub fn new() -> Self {
        Self {
            interaction_history: VecDeque::new(),
            user_patterns: HashMap::new(),
            workflow_context: WorkflowContext {
                current_goal: None,
                current_step: None,
                completed_steps: Vec::new(),
                context_data: HashMap::new(),
                confidence: 0.0,
            },
            max_history: 1000,
        }
    }

    /// Record a user interaction for learning
    pub fn record_interaction(&mut self, interaction: InteractionRecord) {
        // Add to history
        self.interaction_history.push_back(interaction.clone());
        
        // Maintain history size
        while self.interaction_history.len() > self.max_history {
            self.interaction_history.pop_front();
        }
        
        // Update patterns
        self.update_patterns(&interaction);
        
        // Update workflow context
        self.update_workflow_context(&interaction);
    }

    /// Predict the next likely action based on context
    pub fn predict_next_action(&self, current_context: &InteractionContext) -> Result<Vec<ActionPrediction>> {
        let mut predictions = Vec::new();
        
        // Analyze patterns to predict next actions
        for (_, pattern) in &self.user_patterns {
            if let Some(prediction) = self.analyze_pattern_match(pattern, current_context) {
                predictions.push(prediction);
            }
        }
        
        // Sort by confidence
        predictions.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
        
        Ok(predictions)
    }

    /// Suggest element descriptions based on context
    pub fn suggest_element_descriptions(&self, page_type: &str, action: &str) -> Vec<String> {
        let mut suggestions: Vec<String> = Vec::new();
        
        // Get suggestions based on page type and action
        match (page_type, action) {
            ("LoginPage", "click") => {
                suggestions.extend([
                    "login button".to_string(),
                    "sign in button".to_string(),
                    "submit button".to_string(),
                    "continue button".to_string(),
                ]);
            }
            ("LoginPage", "type") => {
                suggestions.extend([
                    "username field".to_string(),
                    "email field".to_string(), 
                    "password field".to_string(),
                    "login field".to_string(),
                ]);
            }
            ("SearchResults", "click") => {
                suggestions.extend([
                    "first result".to_string(),
                    "next page".to_string(),
                    "search result link".to_string(),
                ]);
            }
            ("ProductPage", "click") => {
                suggestions.extend([
                    "add to cart button".to_string(),
                    "buy now button".to_string(),
                    "add to wishlist".to_string(),
                ]);
            }
            ("FormPage", "type") => {
                suggestions.extend([
                    "name field".to_string(),
                    "email field".to_string(),
                    "phone field".to_string(),
                    "address field".to_string(),
                ]);
            }
            _ => {
                // Generic suggestions
                suggestions.extend([
                    "submit button".to_string(),
                    "search box".to_string(),
                    "menu button".to_string(),
                ]);
            }
        }
        
        // Add suggestions from learned patterns
        for pattern in self.user_patterns.values() {
            if pattern.success_rate > 0.7 {
                suggestions.extend(pattern.typical_sequence.clone());
            }
        }
        
        // Remove duplicates and return
        suggestions.sort();
        suggestions.dedup();
        suggestions
    }

    /// Get contextual information for better element finding
    pub fn get_contextual_hints(&self, description: &str) -> ContextualHints {
        let mut hints = ContextualHints {
            alternative_descriptions: Vec::new(),
            expected_location: None,
            timing_hints: None,
            prerequisite_actions: Vec::new(),
        };

        // Add alternative descriptions based on patterns
        if let Some(pattern) = self.find_matching_pattern(description) {
            hints.alternative_descriptions = pattern.contextual_cues.clone();
        }

        // Add location hints based on page type
        if let Some(ref goal) = self.workflow_context.current_goal {
            hints.expected_location = self.predict_element_location(description, goal);
        }

        // Add timing hints
        hints.timing_hints = self.analyze_timing_patterns(description);

        hints
    }

    /// Update workflow context based on interaction
    fn update_workflow_context(&mut self, interaction: &InteractionRecord) {
        // Detect workflow transitions
        if interaction.success {
            if let Some(ref current_step) = self.workflow_context.current_step {
                self.workflow_context.completed_steps.push(current_step.clone());
            }
            
            self.workflow_context.current_step = Some(interaction.action.clone());
            
            // Update context data
            self.workflow_context.context_data.insert(
                format!("last_{}", interaction.action),
                interaction.target_selector.clone()
            );
        }

        // Detect workflow completion or reset
        if self.is_workflow_complete(&interaction) {
            self.reset_workflow_context();
        }
    }

    /// Update user patterns based on new interaction
    fn update_patterns(&mut self, interaction: &InteractionRecord) {
        let pattern_key = format!("{}_{}", interaction.page_type, interaction.action);
        
        // Call classify_pattern_type before the mutable borrow
        let pattern_type = self.classify_pattern_type(&interaction.action);
        
        let pattern = self.user_patterns.entry(pattern_key).or_insert_with(|| {
            UserPattern {
                pattern_type,
                frequency: 0,
                success_rate: 0.0,
                typical_sequence: Vec::new(),
                contextual_cues: Vec::new(),
            }
        });

        // Update frequency and success rate
        pattern.frequency += 1;
        let success_count = if interaction.success { 1.0 } else { 0.0 };
        pattern.success_rate = (pattern.success_rate * (pattern.frequency - 1) as f32 + success_count) / pattern.frequency as f32;

        // Update typical sequence
        if !pattern.typical_sequence.contains(&interaction.target_description) {
            pattern.typical_sequence.push(interaction.target_description.clone());
        }

        // Update contextual cues
        for action in &interaction.context.preceding_actions {
            if !pattern.contextual_cues.contains(action) {
                pattern.contextual_cues.push(action.clone());
            }
        }
    }

    fn classify_pattern_type(&self, action: &str) -> PatternType {
        match action {
            "click" if action.contains("login") || action.contains("sign") => PatternType::LoginSequence,
            "search" => PatternType::SearchWorkflow,
            "type" => PatternType::FormFilling,
            "navigate" => PatternType::Navigation,
            "extract" => PatternType::DataExtraction,
            _ => PatternType::Navigation, // Default
        }
    }

    fn analyze_pattern_match(&self, pattern: &UserPattern, context: &InteractionContext) -> Option<ActionPrediction> {
        // Simple pattern matching - could be enhanced with ML
        let mut confidence = pattern.success_rate * 0.5;
        
        // Boost confidence if context matches
        for cue in &pattern.contextual_cues {
            if context.preceding_actions.contains(cue) {
                confidence += 0.1;
            }
        }
        
        if confidence > 0.3 {
            Some(ActionPrediction {
                action: "click".to_string(), // Simplified
                target_description: pattern.typical_sequence.first()?.clone(),
                confidence,
                reasoning: format!("Based on pattern with {:.1}% success rate", pattern.success_rate * 100.0),
            })
        } else {
            None
        }
    }

    fn find_matching_pattern(&self, description: &str) -> Option<&UserPattern> {
        self.user_patterns.values()
            .find(|p| p.typical_sequence.iter().any(|seq| seq.contains(description)))
    }

    fn predict_element_location(&self, _description: &str, _goal: &str) -> Option<ElementLocation> {
        // TODO: Implement location prediction based on patterns
        None
    }

    fn analyze_timing_patterns(&self, _description: &str) -> Option<TimingHints> {
        // TODO: Implement timing analysis
        None
    }

    fn is_workflow_complete(&self, _interaction: &InteractionRecord) -> bool {
        // TODO: Implement workflow completion detection
        false
    }

    fn reset_workflow_context(&mut self) {
        self.workflow_context = WorkflowContext {
            current_goal: None,
            current_step: None,
            completed_steps: Vec::new(),
            context_data: HashMap::new(),
            confidence: 0.0,
        };
    }

    /// Export patterns for analysis or persistence
    pub fn export_patterns(&self) -> HashMap<String, UserPattern> {
        self.user_patterns.clone()
    }

    /// Import patterns from previous sessions
    pub fn import_patterns(&mut self, patterns: HashMap<String, UserPattern>) {
        for (key, pattern) in patterns {
            self.user_patterns.insert(key, pattern);
        }
    }

    /// Get interaction history summary
    pub fn get_history_summary(&self) -> InteractionSummary {
        let total_interactions = self.interaction_history.len();
        let successful_interactions = self.interaction_history.iter()
            .filter(|i| i.success)
            .count();
        
        let success_rate = if total_interactions > 0 {
            successful_interactions as f32 / total_interactions as f32
        } else {
            0.0
        };

        InteractionSummary {
            total_interactions,
            successful_interactions,
            success_rate,
            most_common_actions: self.get_most_common_actions(),
            active_patterns: self.user_patterns.len(),
        }
    }

    fn get_most_common_actions(&self) -> Vec<(String, u32)> {
        let mut action_counts: HashMap<String, u32> = HashMap::new();
        
        for interaction in &self.interaction_history {
            *action_counts.entry(interaction.action.clone()).or_insert(0) += 1;
        }
        
        let mut actions: Vec<_> = action_counts.into_iter().collect();
        actions.sort_by(|a, b| b.1.cmp(&a.1));
        actions.into_iter().take(10).collect()
    }
}

#[derive(Debug, Serialize)]
pub struct ActionPrediction {
    pub action: String,
    pub target_description: String,
    pub confidence: f32,
    pub reasoning: String,
}

#[derive(Debug)]
pub struct ContextualHints {
    pub alternative_descriptions: Vec<String>,
    pub expected_location: Option<ElementLocation>,
    pub timing_hints: Option<TimingHints>,
    pub prerequisite_actions: Vec<String>,
}

#[derive(Debug)]
pub struct ElementLocation {
    pub region: String, // "top", "bottom", "left", "right", "center"
    pub confidence: f32,
}

#[derive(Debug)]
pub struct TimingHints {
    pub wait_before: Option<u64>, // milliseconds
    pub wait_after: Option<u64>,
    pub retry_intervals: Vec<u64>,
}

#[derive(Debug, Serialize)]
pub struct InteractionSummary {
    pub total_interactions: usize,
    pub successful_interactions: usize,
    pub success_rate: f32,
    pub most_common_actions: Vec<(String, u32)>,
    pub active_patterns: usize,
}