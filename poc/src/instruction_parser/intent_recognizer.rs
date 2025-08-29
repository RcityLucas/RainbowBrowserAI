//! Intent Recognition Module
//! 
//! Identifies user intent from natural language instructions

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Recognized user intents
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Intent {
    // Navigation intents
    Navigate { 
        target: NavigationTarget,
        wait_for: Option<String>,
    },
    
    // Interaction intents
    Click { 
        target_description: String,
        modifier_keys: Vec<ModifierKey>,
    },
    
    Type { 
        text: String,
        target: Option<String>,
        clear_first: bool,
    },
    
    Select {
        option: String,
        dropdown: Option<String>,
    },
    
    // Information extraction intents
    Extract {
        data_type: DataType,
        filters: Vec<Filter>,
    },
    
    Search {
        query: String,
        scope: SearchScope,
    },
    
    // Complex intents
    Workflow {
        steps: Vec<WorkflowStep>,
        goal: String,
    },
    
    Compare {
        items: Vec<String>,
        criteria: Vec<String>,
    },
    
    // Utility intents
    Wait {
        condition: WaitCondition,
        timeout: Option<u64>,
    },
    
    Screenshot {
        area: ScreenshotArea,
        filename: Option<String>,
    },
    
    Scroll {
        direction: ScrollDirection,
        amount: Option<i32>,
    },
    
    // Meta intents
    Help,
    Clarify { original_text: String },
    Unknown { text: String },
}

impl Intent {
    pub fn confidence(&self) -> f32 {
        match self {
            Intent::Navigate { .. } => 0.9,
            Intent::Click { .. } => 0.85,
            Intent::Type { .. } => 0.85,
            Intent::Extract { .. } => 0.8,
            Intent::Search { .. } => 0.85,
            Intent::Unknown { .. } => 0.3,
            _ => 0.75,
        }
    }
    
    pub fn is_ambiguous(&self) -> bool {
        matches!(self, Intent::Unknown { .. } | Intent::Clarify { .. })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NavigationTarget {
    Url(String),
    Back,
    Forward,
    Refresh,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ModifierKey {
    Ctrl,
    Shift,
    Alt,
    Meta,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DataType {
    Text,
    Links,
    Images,
    Tables,
    Forms,
    Prices,
    Dates,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Filter {
    pub field: String,
    pub operator: FilterOperator,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FilterOperator {
    Equals,
    Contains,
    LessThan,
    GreaterThan,
    Between,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SearchScope {
    CurrentPage,
    Website,
    Global,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkflowStep {
    pub action: String,
    pub parameters: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WaitCondition {
    ElementVisible(String),
    ElementClickable(String),
    TextPresent(String),
    UrlContains(String),
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ScreenshotArea {
    FullPage,
    Viewport,
    Element(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ScrollDirection {
    Up,
    Down,
    Left,
    Right,
    ToElement(String),
}

/// Intent recognizer
pub struct IntentRecognizer {
    patterns: HashMap<String, Intent>,
    learned_patterns: HashMap<String, Intent>,
}

impl IntentRecognizer {
    pub fn new() -> Self {
        let mut patterns = HashMap::new();
        
        // Navigation patterns
        patterns.insert("navigate".to_string(), Intent::Navigate { 
            target: NavigationTarget::Url(String::new()),
            wait_for: None,
        });
        patterns.insert("go to".to_string(), Intent::Navigate { 
            target: NavigationTarget::Url(String::new()),
            wait_for: None,
        });
        patterns.insert("open".to_string(), Intent::Navigate { 
            target: NavigationTarget::Url(String::new()),
            wait_for: None,
        });
        patterns.insert("visit".to_string(), Intent::Navigate { 
            target: NavigationTarget::Url(String::new()),
            wait_for: None,
        });
        
        // Click patterns
        patterns.insert("click".to_string(), Intent::Click { 
            target_description: String::new(),
            modifier_keys: vec![],
        });
        patterns.insert("press".to_string(), Intent::Click { 
            target_description: String::new(),
            modifier_keys: vec![],
        });
        patterns.insert("tap".to_string(), Intent::Click { 
            target_description: String::new(),
            modifier_keys: vec![],
        });
        
        // Type patterns
        patterns.insert("type".to_string(), Intent::Type { 
            text: String::new(),
            target: None,
            clear_first: true,
        });
        patterns.insert("enter".to_string(), Intent::Type { 
            text: String::new(),
            target: None,
            clear_first: false,
        });
        patterns.insert("fill".to_string(), Intent::Type { 
            text: String::new(),
            target: None,
            clear_first: true,
        });
        
        // Search patterns
        patterns.insert("search".to_string(), Intent::Search { 
            query: String::new(),
            scope: SearchScope::CurrentPage,
        });
        patterns.insert("find".to_string(), Intent::Search { 
            query: String::new(),
            scope: SearchScope::CurrentPage,
        });
        patterns.insert("look for".to_string(), Intent::Search { 
            query: String::new(),
            scope: SearchScope::CurrentPage,
        });
        
        // Extract patterns
        patterns.insert("extract".to_string(), Intent::Extract { 
            data_type: DataType::Text,
            filters: vec![],
        });
        patterns.insert("get".to_string(), Intent::Extract { 
            data_type: DataType::Text,
            filters: vec![],
        });
        patterns.insert("scrape".to_string(), Intent::Extract { 
            data_type: DataType::Text,
            filters: vec![],
        });
        
        Self {
            patterns,
            learned_patterns: HashMap::new(),
        }
    }
    
    pub fn recognize(&self, input: &str) -> Result<Intent> {
        // Check learned patterns first
        if let Some(intent) = self.match_learned_pattern(input) {
            return Ok(intent);
        }
        
        // Check built-in patterns
        if let Some(intent) = self.match_builtin_pattern(input) {
            return Ok(self.enhance_intent(intent, input));
        }
        
        // Try to infer from context
        if let Some(intent) = self.infer_intent(input) {
            return Ok(intent);
        }
        
        // Unknown intent
        Ok(Intent::Unknown { text: input.to_string() })
    }
    
    pub fn reinforce_positive(&mut self, intent: &Intent) {
        // Increase confidence for this intent pattern
        tracing::debug!("Reinforcing positive feedback for intent: {:?}", intent);
    }
    
    pub fn learn_correction(&mut self, input: &str, correct_intent: Intent) {
        self.learned_patterns.insert(input.to_string(), correct_intent);
        tracing::info!("Learned new pattern: '{}' -> intent", input);
    }
    
    fn match_learned_pattern(&self, input: &str) -> Option<Intent> {
        self.learned_patterns.get(input).cloned()
    }
    
    fn match_builtin_pattern(&self, input: &str) -> Option<Intent> {
        for (pattern, intent) in &self.patterns {
            if input.contains(pattern) {
                return Some(intent.clone());
            }
        }
        None
    }
    
    fn enhance_intent(&self, base_intent: Intent, input: &str) -> Intent {
        match base_intent {
            Intent::Navigate { .. } => {
                if let Some(url) = self.extract_url(input) {
                    Intent::Navigate { 
                        target: NavigationTarget::Url(url),
                        wait_for: None,
                    }
                } else {
                    base_intent
                }
            }
            Intent::Click { .. } => {
                let target = self.extract_click_target(input);
                let modifiers = self.extract_modifiers(input);
                Intent::Click { 
                    target_description: target,
                    modifier_keys: modifiers,
                }
            }
            Intent::Type { .. } => {
                let text = self.extract_text_to_type(input);
                let target = self.extract_type_target(input);
                Intent::Type { 
                    text,
                    target,
                    clear_first: !input.contains("append"),
                }
            }
            Intent::Search { .. } => {
                let query = self.extract_search_query(input);
                Intent::Search { 
                    query,
                    scope: SearchScope::CurrentPage,
                }
            }
            _ => base_intent,
        }
    }
    
    fn infer_intent(&self, input: &str) -> Option<Intent> {
        // URL patterns
        if input.contains("http") || input.contains(".com") || input.contains(".org") {
            return Some(Intent::Navigate { 
                target: NavigationTarget::Url(input.to_string()),
                wait_for: None,
            });
        }
        
        // Question patterns
        if input.starts_with("what") || input.starts_with("how") || input.starts_with("where") {
            return Some(Intent::Extract { 
                data_type: DataType::Text,
                filters: vec![],
            });
        }
        
        None
    }
    
    fn extract_url(&self, input: &str) -> Option<String> {
        // Look for URL patterns
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
    
    fn extract_click_target(&self, input: &str) -> String {
        // Extract what comes after "click"
        if let Some(pos) = input.find("click") {
            let after = &input[pos + 5..].trim();
            if let Some(end) = after.find(" and ") {
                after[..end].to_string()
            } else {
                after.to_string()
            }
        } else {
            input.to_string()
        }
    }
    
    fn extract_modifiers(&self, input: &str) -> Vec<ModifierKey> {
        let mut modifiers = vec![];
        if input.contains("ctrl") || input.contains("control") {
            modifiers.push(ModifierKey::Ctrl);
        }
        if input.contains("shift") {
            modifiers.push(ModifierKey::Shift);
        }
        if input.contains("alt") {
            modifiers.push(ModifierKey::Alt);
        }
        modifiers
    }
    
    fn extract_text_to_type(&self, input: &str) -> String {
        // Extract text between quotes if present
        if let Some(start) = input.find('"') {
            if let Some(end) = input[start + 1..].find('"') {
                return input[start + 1..start + 1 + end].to_string();
            }
        }
        
        // Otherwise, take everything after "type"
        if let Some(pos) = input.find("type") {
            input[pos + 4..].trim().to_string()
        } else {
            input.to_string()
        }
    }
    
    fn extract_type_target(&self, input: &str) -> Option<String> {
        if input.contains(" in ") || input.contains(" into ") {
            let marker = if input.contains(" in ") { " in " } else { " into " };
            if let Some(pos) = input.find(marker) {
                let after = &input[pos + marker.len()..];
                if let Some(end) = after.find(' ') {
                    Some(after[..end].to_string())
                } else {
                    Some(after.to_string())
                }
            } else {
                None
            }
        } else {
            None
        }
    }
    
    fn extract_search_query(&self, input: &str) -> String {
        if input.contains(" for ") {
            if let Some(pos) = input.find(" for ") {
                input[pos + 5..].trim().to_string()
            } else {
                input.to_string()
            }
        } else {
            input.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_recognize_navigation() {
        let recognizer = IntentRecognizer::new();
        
        let intent = recognizer.recognize("go to google.com").unwrap();
        match intent {
            Intent::Navigate { target, .. } => {
                assert_eq!(target, NavigationTarget::Url("https://google.com".to_string()));
            }
            _ => panic!("Expected Navigate intent"),
        }
    }
    
    #[test]
    fn test_recognize_click() {
        let recognizer = IntentRecognizer::new();
        
        let intent = recognizer.recognize("click the submit button").unwrap();
        match intent {
            Intent::Click { target_description, .. } => {
                assert!(target_description.contains("submit"));
            }
            _ => panic!("Expected Click intent"),
        }
    }
    
    #[test]
    fn test_recognize_type() {
        let recognizer = IntentRecognizer::new();
        
        let intent = recognizer.recognize("type \"hello world\" in search box").unwrap();
        match intent {
            Intent::Type { text, target, .. } => {
                assert_eq!(text, "hello world");
                assert_eq!(target, Some("search box".to_string()));
            }
            _ => panic!("Expected Type intent"),
        }
    }
}