//! Enhanced User Instruction Parser
//! 
//! Transforms natural language instructions into structured, executable intents
//! with context awareness and learning capabilities.

use anyhow::Result;
use serde::{Deserialize, Serialize};

pub mod intent_recognizer;
pub mod entity_extractor;
pub mod context_manager;
pub mod patterns;
pub mod workflow_parser;

pub use intent_recognizer::{IntentRecognizer, Intent};
pub use entity_extractor::{EntityExtractor, Entity};
pub use context_manager::{ContextManager, InstructionContext};
pub use patterns::{InstructionPattern, PatternMatcher};
pub use workflow_parser::WorkflowParser;

/// Main instruction parser that coordinates all components
pub struct InstructionParser {
    intent_recognizer: IntentRecognizer,
    entity_extractor: EntityExtractor,
    context_manager: ContextManager,
    pattern_matcher: PatternMatcher,
}

impl InstructionParser {
    pub fn new() -> Self {
        Self {
            intent_recognizer: IntentRecognizer::new(),
            entity_extractor: EntityExtractor::new(),
            context_manager: ContextManager::new(),
            pattern_matcher: PatternMatcher::new(),
        }
    }
    
    /// Parse user instruction into structured format
    pub fn parse(&mut self, input: &str) -> Result<UserInstruction> {
        // Normalize input
        let normalized = self.normalize_input(input);
        
        // Check for known patterns first (fast path)
        if let Some(pattern_match) = self.pattern_matcher.match_pattern(&normalized) {
            return Ok(self.create_from_pattern(pattern_match, input));
        }
        
        // Extract intent
        let intent = self.intent_recognizer.recognize(&normalized)?;
        
        // Extract entities
        let entities = self.entity_extractor.extract(&normalized, &intent);
        
        // Add context
        let context = self.context_manager.get_current_context();
        
        // Calculate confidence
        let confidence = self.calculate_confidence(&intent, &entities);
        
        // Build instruction
        let instruction = UserInstruction {
            raw_text: input.to_string(),
            normalized_text: normalized.clone(),
            intent,
            entities,
            context,
            confidence,
            suggestions: self.generate_suggestions(&normalized),
        };
        
        // Update context with this instruction
        self.context_manager.add_instruction(&instruction);
        
        Ok(instruction)
    }
    
    /// Parse with context hints
    pub fn parse_with_context(&mut self, input: &str, hints: ContextHints) -> Result<UserInstruction> {
        self.context_manager.apply_hints(hints);
        self.parse(input)
    }
    
    /// Learn from user feedback
    pub fn learn_from_feedback(&mut self, instruction: &UserInstruction, feedback: Feedback) {
        match feedback {
            Feedback::Success => {
                self.pattern_matcher.record_success(instruction);
                self.intent_recognizer.reinforce_positive(&instruction.intent);
            }
            Feedback::Failure { corrected_intent } => {
                self.intent_recognizer.learn_correction(&instruction.normalized_text, corrected_intent);
            }
            Feedback::PartialSuccess { improvements } => {
                self.apply_improvements(instruction, improvements);
            }
        }
    }
    
    fn normalize_input(&self, input: &str) -> String {
        input
            .to_lowercase()
            .trim()
            .replace("please", "")
            .replace("could you", "")
            .replace("can you", "")
            .trim()
            .to_string()
    }
    
    fn create_from_pattern(&self, pattern: InstructionPattern, raw: &str) -> UserInstruction {
        UserInstruction {
            raw_text: raw.to_string(),
            normalized_text: pattern.template.clone(),
            intent: pattern.intent,
            entities: pattern.entities,
            context: self.context_manager.get_current_context(),
            confidence: pattern.confidence,
            suggestions: vec![],
        }
    }
    
    fn calculate_confidence(&self, intent: &Intent, entities: &[Entity]) -> f32 {
        let intent_confidence = intent.confidence();
        let entity_confidence = if entities.is_empty() {
            0.5
        } else {
            entities.iter().map(|e| e.confidence).sum::<f32>() / entities.len() as f32
        };
        
        (intent_confidence * 0.7 + entity_confidence * 0.3).min(1.0)
    }
    
    fn generate_suggestions(&self, input: &str) -> Vec<String> {
        let mut suggestions = vec![];
        
        // Suggest more specific instructions if too vague
        if input.split_whitespace().count() < 3 {
            suggestions.push("Try being more specific, e.g., 'click the submit button'".to_string());
        }
        
        // Suggest selector if clicking without target
        if input.contains("click") && !input.contains("#") && !input.contains(".") {
            suggestions.push("You can specify elements by ID (#id) or class (.class)".to_string());
        }
        
        suggestions
    }
    
    fn apply_improvements(&mut self, instruction: &UserInstruction, improvements: Vec<String>) {
        for improvement in improvements {
            tracing::info!("Learning improvement: {}", improvement);
            // Store improvements for future pattern matching
            self.pattern_matcher.add_improvement_pattern(&instruction.normalized_text, &improvement);
        }
    }
}

/// Structured user instruction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInstruction {
    pub raw_text: String,
    pub normalized_text: String,
    pub intent: Intent,
    pub entities: Vec<Entity>,
    pub context: InstructionContext,
    pub confidence: f32,
    pub suggestions: Vec<String>,
}

impl UserInstruction {
    /// Check if instruction needs clarification
    pub fn needs_clarification(&self) -> bool {
        self.confidence < 0.6 || self.intent.is_ambiguous()
    }
    
    /// Get clarification questions
    pub fn get_clarification_questions(&self) -> Vec<String> {
        let mut questions = vec![];
        
        if self.intent.is_ambiguous() {
            questions.push("What would you like me to do? Navigate, click, or extract information?".to_string());
        }
        
        for entity in &self.entities {
            if entity.confidence < 0.5 {
                questions.push(format!("Did you mean '{}' when you said '{}'?", 
                    entity.suggested_value.as_ref().unwrap_or(&entity.value),
                    entity.value
                ));
            }
        }
        
        questions
    }
}

/// Context hints for better parsing
#[derive(Debug, Clone)]
pub struct ContextHints {
    pub current_page_type: Option<PageType>,
    pub previous_action: Option<String>,
    pub user_goal: Option<String>,
}

/// User feedback on instruction execution
#[derive(Debug, Clone)]
pub enum Feedback {
    Success,
    Failure { corrected_intent: Intent },
    PartialSuccess { improvements: Vec<String> },
}

/// Page type for context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PageType {
    Homepage,
    SearchResults,
    ProductPage,
    ArticlePage,
    FormPage,
    Dashboard,
    Unknown,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_simple_instruction() {
        let mut parser = InstructionParser::new();
        
        let instruction = parser.parse("click the submit button").unwrap();
        assert!(matches!(instruction.intent, Intent::Click { .. }));
        assert!(!instruction.entities.is_empty());
    }
    
    #[test]
    fn test_parse_complex_instruction() {
        let mut parser = InstructionParser::new();
        
        let instruction = parser.parse(
            "go to amazon.com and search for wireless headphones under $100"
        ).unwrap();
        
        assert!(instruction.confidence > 0.5);
        assert!(instruction.entities.len() >= 2); // URL and search term
    }
    
    #[test]
    fn test_context_awareness() {
        let mut parser = InstructionParser::new();
        
        // First instruction
        parser.parse("go to google.com").unwrap();
        
        // Second instruction uses context
        let instruction = parser.parse("search for rust programming").unwrap();
        assert!(matches!(instruction.intent, Intent::Search { .. }));
    }
}