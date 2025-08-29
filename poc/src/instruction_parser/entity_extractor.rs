//! Entity Extraction Module
//! 
//! Extracts entities (selectors, URLs, text, etc.) from natural language

use anyhow::Result;
use serde::{Deserialize, Serialize};
use regex::Regex;
use lazy_static::lazy_static;

use super::intent_recognizer::Intent;

lazy_static! {
    // URL patterns
    static ref URL_REGEX: Regex = Regex::new(
        r"(?i)(https?://)?([a-z0-9]+(-[a-z0-9]+)*\.)+[a-z]{2,}(/[^\s]*)?"
    ).unwrap();
    
    // CSS selector patterns
    static ref ID_REGEX: Regex = Regex::new(r"#[\w-]+").unwrap();
    static ref CLASS_REGEX: Regex = Regex::new(r"\.[\w-]+").unwrap();
    
    // Number patterns
    static ref NUMBER_REGEX: Regex = Regex::new(r"\b\d+(\.\d+)?\b").unwrap();
    static ref PRICE_REGEX: Regex = Regex::new(r"\$\d+(\.\d{2})?").unwrap();
    
    // Time patterns
    static ref TIME_REGEX: Regex = Regex::new(r"\b\d{1,2}:\d{2}(:\d{2})?\s?(am|pm|AM|PM)?\b").unwrap();
    static ref DATE_REGEX: Regex = Regex::new(r"\b\d{1,2}/\d{1,2}/\d{2,4}\b").unwrap();
}

/// Extracted entity from user instruction
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Entity {
    pub entity_type: EntityType,
    pub value: String,
    pub confidence: f32,
    pub position: (usize, usize), // Start and end position in text
    pub suggested_value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EntityType {
    // Web elements
    Selector,
    ElementDescription,
    
    // Navigation
    Url,
    Domain,
    Path,
    
    // Content
    Text,
    Number,
    Price,
    
    // Time
    Date,
    Time,
    Duration,
    
    // Actions
    ActionModifier,
    Direction,
    Position,
    
    // Data
    DataType,
    FieldName,
    ComparisonOperator,
}

/// Entity extractor
pub struct EntityExtractor {
    domain_list: Vec<String>,
    element_descriptors: Vec<String>,
    action_modifiers: Vec<String>,
}

impl EntityExtractor {
    pub fn new() -> Self {
        Self {
            domain_list: vec![
                "google.com".to_string(),
                "amazon.com".to_string(),
                "github.com".to_string(),
                "stackoverflow.com".to_string(),
                "wikipedia.org".to_string(),
            ],
            element_descriptors: vec![
                "button".to_string(),
                "link".to_string(),
                "input".to_string(),
                "form".to_string(),
                "image".to_string(),
                "video".to_string(),
                "table".to_string(),
                "menu".to_string(),
                "dropdown".to_string(),
                "checkbox".to_string(),
                "radio".to_string(),
            ],
            action_modifiers: vec![
                "quickly".to_string(),
                "slowly".to_string(),
                "carefully".to_string(),
                "repeatedly".to_string(),
                "all".to_string(),
                "first".to_string(),
                "last".to_string(),
                "next".to_string(),
                "previous".to_string(),
            ],
        }
    }
    
    pub fn extract(&self, text: &str, intent: &Intent) -> Vec<Entity> {
        let mut entities = Vec::new();
        
        // Extract based on intent type
        match intent {
            Intent::Navigate { .. } => {
                entities.extend(self.extract_urls(text));
                entities.extend(self.extract_domains(text));
            }
            Intent::Click { .. } | Intent::Type { .. } => {
                entities.extend(self.extract_selectors(text));
                entities.extend(self.extract_element_descriptions(text));
            }
            Intent::Extract { .. } => {
                entities.extend(self.extract_data_types(text));
                entities.extend(self.extract_field_names(text));
            }
            Intent::Search { .. } => {
                entities.extend(self.extract_search_terms(text));
            }
            _ => {
                // General entity extraction
                entities.extend(self.extract_all(text));
            }
        }
        
        // Sort by position and remove duplicates
        entities.sort_by_key(|e| e.position.0);
        self.deduplicate_entities(entities)
    }
    
    fn extract_urls(&self, text: &str) -> Vec<Entity> {
        let mut entities = Vec::new();
        
        for capture in URL_REGEX.find_iter(text) {
            let url = capture.as_str();
            let full_url = if !url.starts_with("http") {
                format!("https://{}", url)
            } else {
                url.to_string()
            };
            
            entities.push(Entity {
                entity_type: EntityType::Url,
                value: full_url,
                confidence: 0.95,
                position: (capture.start(), capture.end()),
                suggested_value: None,
            });
        }
        
        entities
    }
    
    fn extract_domains(&self, text: &str) -> Vec<Entity> {
        let mut entities = Vec::new();
        let lower_text = text.to_lowercase();
        
        for domain in &self.domain_list {
            if let Some(pos) = lower_text.find(domain) {
                entities.push(Entity {
                    entity_type: EntityType::Domain,
                    value: domain.clone(),
                    confidence: 0.9,
                    position: (pos, pos + domain.len()),
                    suggested_value: Some(format!("https://{}", domain)),
                });
            }
        }
        
        entities
    }
    
    fn extract_selectors(&self, text: &str) -> Vec<Entity> {
        let mut entities = Vec::new();
        
        // Extract ID selectors
        for capture in ID_REGEX.find_iter(text) {
            entities.push(Entity {
                entity_type: EntityType::Selector,
                value: capture.as_str().to_string(),
                confidence: 0.95,
                position: (capture.start(), capture.end()),
                suggested_value: None,
            });
        }
        
        // Extract class selectors
        for capture in CLASS_REGEX.find_iter(text) {
            entities.push(Entity {
                entity_type: EntityType::Selector,
                value: capture.as_str().to_string(),
                confidence: 0.9,
                position: (capture.start(), capture.end()),
                suggested_value: None,
            });
        }
        
        entities
    }
    
    fn extract_element_descriptions(&self, text: &str) -> Vec<Entity> {
        let mut entities = Vec::new();
        let lower_text = text.to_lowercase();
        
        for descriptor in &self.element_descriptors {
            if let Some(pos) = lower_text.find(descriptor) {
                // Try to find more context around the element
                let context = self.extract_element_context(text, pos, descriptor);
                
                entities.push(Entity {
                    entity_type: EntityType::ElementDescription,
                    value: context.clone(),
                    confidence: 0.8,
                    position: (pos, pos + context.len()),
                    suggested_value: Some(self.suggest_selector(&context)),
                });
            }
        }
        
        entities
    }
    
    fn extract_element_context(&self, text: &str, pos: usize, element: &str) -> String {
        // Extract context around element (e.g., "submit button", "search input")
        let words: Vec<&str> = text.split_whitespace().collect();
        let mut context = element.to_string();
        
        // Find word index
        let mut word_pos = 0;
        let mut char_count = 0;
        for (i, word) in words.iter().enumerate() {
            if char_count >= pos && word.to_lowercase().contains(element) {
                word_pos = i;
                break;
            }
            char_count += word.len() + 1; // +1 for space
        }
        
        // Add preceding word if it's descriptive
        if word_pos > 0 {
            let prev_word = words[word_pos - 1].to_lowercase();
            if self.is_descriptive(&prev_word) {
                context = format!("{} {}", prev_word, context);
            }
        }
        
        // Add following word if it's descriptive
        if word_pos < words.len() - 1 {
            let next_word = words[word_pos + 1].to_lowercase();
            if self.is_descriptive(&next_word) {
                context = format!("{} {}", context, next_word);
            }
        }
        
        context
    }
    
    fn is_descriptive(&self, word: &str) -> bool {
        // Check if word is descriptive (color, size, position, action)
        let descriptive_words = [
            "red", "blue", "green", "big", "small", "large", "tiny",
            "first", "last", "next", "previous", "main", "sidebar",
            "submit", "cancel", "save", "delete", "edit", "search",
            "login", "logout", "register", "signup", "download", "upload"
        ];
        
        descriptive_words.contains(&word)
    }
    
    fn suggest_selector(&self, description: &str) -> String {
        // Suggest CSS selector based on description
        let lower = description.to_lowercase();
        
        if lower.contains("submit") {
            "button[type='submit']".to_string()
        } else if lower.contains("search") && lower.contains("input") {
            "input[type='search'], input[placeholder*='search']".to_string()
        } else if lower.contains("button") {
            "button".to_string()
        } else if lower.contains("link") {
            "a".to_string()
        } else {
            format!("[aria-label*='{}']", description)
        }
    }
    
    fn extract_data_types(&self, text: &str) -> Vec<Entity> {
        let mut entities = Vec::new();
        let data_keywords = vec![
            ("price", EntityType::DataType),
            ("cost", EntityType::DataType),
            ("review", EntityType::DataType),
            ("rating", EntityType::DataType),
            ("description", EntityType::DataType),
            ("title", EntityType::DataType),
            ("image", EntityType::DataType),
            ("link", EntityType::DataType),
            ("email", EntityType::DataType),
            ("phone", EntityType::DataType),
            ("address", EntityType::DataType),
        ];
        
        let lower_text = text.to_lowercase();
        for (keyword, entity_type) in data_keywords {
            if let Some(pos) = lower_text.find(keyword) {
                entities.push(Entity {
                    entity_type: entity_type.clone(),
                    value: keyword.to_string(),
                    confidence: 0.85,
                    position: (pos, pos + keyword.len()),
                    suggested_value: None,
                });
            }
        }
        
        entities
    }
    
    fn extract_field_names(&self, text: &str) -> Vec<Entity> {
        let mut entities = Vec::new();
        
        // Look for quoted field names
        let quoted_regex = Regex::new(r#"["']([^"']+)["']"#).unwrap();
        for capture in quoted_regex.captures_iter(text) {
            if let Some(field) = capture.get(1) {
                entities.push(Entity {
                    entity_type: EntityType::FieldName,
                    value: field.as_str().to_string(),
                    confidence: 0.9,
                    position: (field.start(), field.end()),
                    suggested_value: None,
                });
            }
        }
        
        entities
    }
    
    fn extract_search_terms(&self, text: &str) -> Vec<Entity> {
        let mut entities = Vec::new();
        
        // Extract text after "for" or "about"
        let search_markers = vec!["for ", "about ", "regarding "];
        for marker in search_markers {
            if let Some(pos) = text.find(marker) {
                let term_start = pos + marker.len();
                let term = &text[term_start..];
                
                // Take until end of sentence or line
                let end = term.find(|c: char| c == '.' || c == ',' || c == '\n')
                    .unwrap_or(term.len());
                
                entities.push(Entity {
                    entity_type: EntityType::Text,
                    value: term[..end].trim().to_string(),
                    confidence: 0.85,
                    position: (term_start, term_start + end),
                    suggested_value: None,
                });
            }
        }
        
        entities
    }
    
    fn extract_all(&self, text: &str) -> Vec<Entity> {
        let mut entities = Vec::new();
        
        // Extract all types of entities
        entities.extend(self.extract_urls(text));
        entities.extend(self.extract_selectors(text));
        entities.extend(self.extract_numbers(text));
        entities.extend(self.extract_prices(text));
        entities.extend(self.extract_dates(text));
        entities.extend(self.extract_times(text));
        
        entities
    }
    
    fn extract_numbers(&self, text: &str) -> Vec<Entity> {
        let mut entities = Vec::new();
        
        for capture in NUMBER_REGEX.find_iter(text) {
            entities.push(Entity {
                entity_type: EntityType::Number,
                value: capture.as_str().to_string(),
                confidence: 0.9,
                position: (capture.start(), capture.end()),
                suggested_value: None,
            });
        }
        
        entities
    }
    
    fn extract_prices(&self, text: &str) -> Vec<Entity> {
        let mut entities = Vec::new();
        
        for capture in PRICE_REGEX.find_iter(text) {
            entities.push(Entity {
                entity_type: EntityType::Price,
                value: capture.as_str().to_string(),
                confidence: 0.95,
                position: (capture.start(), capture.end()),
                suggested_value: None,
            });
        }
        
        entities
    }
    
    fn extract_dates(&self, text: &str) -> Vec<Entity> {
        let mut entities = Vec::new();
        
        for capture in DATE_REGEX.find_iter(text) {
            entities.push(Entity {
                entity_type: EntityType::Date,
                value: capture.as_str().to_string(),
                confidence: 0.9,
                position: (capture.start(), capture.end()),
                suggested_value: None,
            });
        }
        
        entities
    }
    
    fn extract_times(&self, text: &str) -> Vec<Entity> {
        let mut entities = Vec::new();
        
        for capture in TIME_REGEX.find_iter(text) {
            entities.push(Entity {
                entity_type: EntityType::Time,
                value: capture.as_str().to_string(),
                confidence: 0.9,
                position: (capture.start(), capture.end()),
                suggested_value: None,
            });
        }
        
        entities
    }
    
    fn deduplicate_entities(&self, entities: Vec<Entity>) -> Vec<Entity> {
        let mut deduped = Vec::new();
        let mut seen_positions = Vec::new();
        
        for entity in entities {
            // Check if this position overlaps with any seen position
            let overlaps = seen_positions.iter().any(|&(start, end)| {
                entity.position.0 < end && entity.position.1 > start
            });
            
            if !overlaps {
                seen_positions.push(entity.position);
                deduped.push(entity);
            }
        }
        
        deduped
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instruction_parser::intent_recognizer::NavigationTarget;
    
    #[test]
    fn test_extract_urls() {
        let extractor = EntityExtractor::new();
        let entities = extractor.extract_urls("go to google.com and then visit https://github.com");
        
        assert_eq!(entities.len(), 2);
        assert_eq!(entities[0].value, "https://google.com");
        assert_eq!(entities[1].value, "https://github.com");
    }
    
    #[test]
    fn test_extract_selectors() {
        let extractor = EntityExtractor::new();
        let entities = extractor.extract_selectors("click #submit-btn and then .close-button");
        
        assert_eq!(entities.len(), 2);
        assert_eq!(entities[0].value, "#submit-btn");
        assert_eq!(entities[1].value, ".close-button");
    }
    
    #[test]
    fn test_extract_element_descriptions() {
        let extractor = EntityExtractor::new();
        let entities = extractor.extract_element_descriptions("click the submit button");
        
        assert!(!entities.is_empty());
        assert!(entities[0].value.contains("submit"));
        assert!(entities[0].suggested_value.is_some());
    }
    
    #[test]
    fn test_extract_prices() {
        let extractor = EntityExtractor::new();
        let entities = extractor.extract_prices("find products under $50.99");
        
        assert_eq!(entities.len(), 1);
        assert_eq!(entities[0].value, "$50.99");
    }
}