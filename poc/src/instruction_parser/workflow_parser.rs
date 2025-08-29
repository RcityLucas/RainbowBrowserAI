use anyhow::Result;
use tracing::info;

use super::intent_recognizer::{Intent, IntentRecognizer};
use super::{UserInstruction, Entity, InstructionContext};
use super::entity_extractor::EntityType;

/// Breaks down complex instructions into multiple workflow steps
pub struct WorkflowParser {
    intent_recognizer: IntentRecognizer,
}

impl WorkflowParser {
    pub fn new() -> Self {
        Self {
            intent_recognizer: IntentRecognizer::new(),
        }
    }
    
    /// Parse a complex instruction into multiple steps
    pub fn parse_workflow(&self, input: &str) -> Vec<UserInstruction> {
        info!("Parsing workflow from: {}", input);
        
        // Split instruction by common conjunctions and action delimiters
        let steps = self.split_into_steps(input);
        let mut instructions = Vec::new();
        
        for (i, step) in steps.iter().enumerate() {
            info!("Processing step {}: {}", i + 1, step);
            
            // Recognize intent for this step
            let intent_result = self.intent_recognizer.recognize(step);
            let (intent, confidence) = match intent_result {
                Ok(recognized_intent) => {
                    // The recognize method returns a Result<Intent>, we need to calculate confidence
                    (recognized_intent, 0.75) // Default confidence for workflow steps
                }
                Err(_) => {
                    // If recognition fails, create an unknown intent
                    (Intent::Unknown { text: step.clone() }, 0.3)
                }
            };
            
            // Create instruction for this step
            let instruction = UserInstruction {
                raw_text: step.clone(),
                normalized_text: step.to_lowercase().trim().to_string(),
                intent: intent.clone(),
                confidence,
                entities: self.extract_entities_as_entity_type(step, &intent),
                context: InstructionContext::new(),
                suggestions: vec![],
            };
            
            instructions.push(instruction);
        }
        
        // Post-process to ensure logical flow
        self.optimize_workflow(&mut instructions);
        
        info!("Parsed {} workflow steps", instructions.len());
        instructions
    }
    
    /// Split complex instruction into atomic steps
    fn split_into_steps(&self, input: &str) -> Vec<String> {
        let mut steps = Vec::new();
        let mut current_step = String::new();
        let mut in_quotes = false;
        
        // Common action keywords that indicate a new step
        let action_keywords = vec![
            "navigate", "go to", "open", "visit",
            "click", "press", "tap",
            "type", "enter", "input", "fill",
            "search", "find", "look for",
            "wait", "pause",
            "screenshot", "capture", "take",
            "extract", "get", "copy",
            "scroll", "move"
        ];
        
        // Split by common conjunctions
        let conjunctions = vec![", and ", " and ", ", then ", " then ", ", ", ". "];
        
        // First, try to split by explicit conjunctions
        let mut remaining = input.to_string();
        
        for conjunction in &conjunctions {
            if remaining.contains(conjunction) {
                let parts: Vec<&str> = remaining.split(conjunction).collect();
                if parts.len() > 1 {
                    // Found a conjunction split
                    for (i, part) in parts.iter().enumerate() {
                        if i == 0 {
                            steps.push(part.to_string());
                        } else {
                            // Check if this part starts with an action or is a continuation
                            let trimmed = part.trim().to_lowercase();
                            let is_new_action = action_keywords.iter()
                                .any(|kw| trimmed.starts_with(kw));
                            
                            if is_new_action {
                                steps.push(part.to_string());
                            } else {
                                // This is likely a continuation of the previous action
                                // For example: "search for" followed by "OpenAI"
                                if let Some(last) = steps.last_mut() {
                                    last.push_str(conjunction);
                                    last.push_str(part);
                                } else {
                                    steps.push(part.to_string());
                                }
                            }
                        }
                    }
                    return steps;
                }
            }
        }
        
        // If no conjunctions found, check if there are multiple actions in sequence
        let words: Vec<&str> = input.split_whitespace().collect();
        let mut i = 0;
        
        while i < words.len() {
            let word = words[i].to_lowercase();
            
            // Check if this word starts a new action
            let is_action_start = action_keywords.iter()
                .any(|kw| {
                    if kw.contains(' ') {
                        // Multi-word keyword like "go to"
                        let kw_words: Vec<&str> = kw.split_whitespace().collect();
                        if i + kw_words.len() <= words.len() {
                            let phrase = words[i..i + kw_words.len()]
                                .join(" ")
                                .to_lowercase();
                            phrase == *kw
                        } else {
                            false
                        }
                    } else {
                        word == *kw
                    }
                });
            
            if is_action_start && !current_step.is_empty() {
                // Start of a new action, save current step
                steps.push(current_step.trim().to_string());
                current_step = String::new();
            }
            
            // Add word to current step
            if !current_step.is_empty() {
                current_step.push(' ');
            }
            current_step.push_str(words[i]);
            
            i += 1;
        }
        
        // Add the last step
        if !current_step.is_empty() {
            steps.push(current_step.trim().to_string());
        }
        
        // If still no steps found, return the whole input as a single step
        if steps.is_empty() {
            steps.push(input.to_string());
        }
        
        steps
    }
    
    /// Extract entities as Entity type
    fn extract_entities_as_entity_type(&self, step: &str, intent: &Intent) -> Vec<Entity> {
        let raw_entities = self.extract_entities(step, intent);
        raw_entities.into_iter().map(|(key, value)| {
            let entity_type = match key.as_str() {
                "url" => EntityType::Url,
                "target" => EntityType::ElementDescription,
                "text" => EntityType::Text,
                "query" => EntityType::Text,
                _ => EntityType::Text, // Default to Text for unknown types
            };
            Entity {
                entity_type,
                value: value.clone(),
                confidence: 0.8,
                position: (0, value.len()), // Approximate position
                suggested_value: Some(value),
            }
        }).collect()
    }
    
    /// Extract entities relevant to the intent
    fn extract_entities(&self, step: &str, intent: &Intent) -> Vec<(String, String)> {
        let mut entities = Vec::new();
        
        match intent {
            Intent::Navigate { .. } => {
                // Extract URL
                if let Some(url) = self.extract_url(step) {
                    entities.push(("url".to_string(), url));
                }
            }
            Intent::Click { .. } => {
                // Extract element description
                if let Some(target) = self.extract_click_target(step) {
                    entities.push(("target".to_string(), target));
                }
            }
            Intent::Type { .. } => {
                // Extract text to type
                if let Some(text) = self.extract_quoted_text(step) {
                    entities.push(("text".to_string(), text));
                } else if let Some(text) = self.extract_after_keyword(step, "type") {
                    entities.push(("text".to_string(), text));
                }
            }
            Intent::Search { .. } => {
                // Extract search query
                if let Some(query) = self.extract_after_keyword(step, "search for") {
                    entities.push(("query".to_string(), query));
                } else if let Some(query) = self.extract_after_keyword(step, "find") {
                    entities.push(("query".to_string(), query));
                }
            }
            _ => {}
        }
        
        entities
    }
    
    /// Optimize workflow for logical execution
    fn optimize_workflow(&self, instructions: &mut Vec<UserInstruction>) {
        // Remove duplicate navigations
        let mut i = 0;
        while i < instructions.len() - 1 {
            if matches!(instructions[i].intent, Intent::Navigate { .. }) &&
               matches!(instructions[i + 1].intent, Intent::Navigate { .. }) {
                // Keep only the second navigation
                instructions.remove(i);
            } else {
                i += 1;
            }
        }
        
        // Ensure search actions have proper context
        for i in 0..instructions.len() {
            if matches!(instructions[i].intent, Intent::Search { .. }) {
                // If searching and no prior navigation, might need to be on a search engine
                if i == 0 || !matches!(instructions[i - 1].intent, Intent::Navigate { .. }) {
                    // This search might need a search engine context
                    // We'll let the executor handle this
                }
            }
        }
    }
    
    // Helper methods
    
    fn extract_url(&self, input: &str) -> Option<String> {
        for word in input.split_whitespace() {
            if word.contains("http") || word.contains(".com") || word.contains(".org") {
                return Some(if word.starts_with("http") {
                    word.to_string()
                } else {
                    format!("https://{}", word)
                });
            }
        }
        None
    }
    
    fn extract_click_target(&self, input: &str) -> Option<String> {
        if let Some(pos) = input.find("click") {
            let after = &input[pos + 5..].trim();
            if !after.is_empty() {
                // Remove "on" if it starts with it
                let cleaned = if after.starts_with("on ") {
                    &after[3..]
                } else {
                    after
                };
                return Some(cleaned.to_string());
            }
        }
        None
    }
    
    fn extract_quoted_text(&self, input: &str) -> Option<String> {
        // Look for text in quotes
        if let Some(start) = input.find('"') {
            if let Some(end) = input[start + 1..].find('"') {
                return Some(input[start + 1..start + 1 + end].to_string());
            }
        } else if let Some(start) = input.find('\'') {
            if let Some(end) = input[start + 1..].find('\'') {
                return Some(input[start + 1..start + 1 + end].to_string());
            }
        }
        None
    }
    
    fn extract_after_keyword(&self, input: &str, keyword: &str) -> Option<String> {
        let lower = input.to_lowercase();
        if let Some(pos) = lower.find(keyword) {
            let after = input[pos + keyword.len()..].trim();
            if !after.is_empty() {
                return Some(after.to_string());
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_simple_workflow() {
        let parser = WorkflowParser::new();
        let steps = parser.parse_workflow("Go to google.com and search for OpenAI");
        assert_eq!(steps.len(), 2);
        assert!(matches!(steps[0].intent, Intent::Navigate { .. }));
        assert!(matches!(steps[1].intent, Intent::Search { .. }));
    }
    
    #[test]
    fn test_complex_workflow() {
        let parser = WorkflowParser::new();
        let steps = parser.parse_workflow(
            "Navigate to github.com, click on the search box, type rust, and press enter"
        );
        assert_eq!(steps.len(), 4);
    }
    
    #[test]
    fn test_screenshot_workflow() {
        let parser = WorkflowParser::new();
        let steps = parser.parse_workflow("Open example.com and take a screenshot");
        assert_eq!(steps.len(), 2);
        assert!(matches!(steps[0].intent, Intent::Navigate { .. }));
        assert!(matches!(steps[1].intent, Intent::Screenshot { .. }));
    }
}