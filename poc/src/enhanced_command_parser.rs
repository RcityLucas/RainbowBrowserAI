// Enhanced Command Parser - Improves Natural Language Understanding
// Specifically addresses the "fill" command issue and enhances form interactions

use std::collections::HashMap;
use regex::Regex;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedCommand {
    pub action: String,
    pub target: Option<String>,
    pub value: Option<String>,
    pub selector: Option<String>,
    pub confidence: f32,
    pub metadata: HashMap<String, String>,
}

/// Enhanced Command Parser with improved pattern matching
pub struct EnhancedCommandParser {
    patterns: Vec<CommandPattern>,
}

struct CommandPattern {
    pattern: Regex,
    action: String,
    confidence_boost: f32,
    extractor: Box<dyn Fn(&regex::Captures) -> ParsedCommand>,
}

impl EnhancedCommandParser {
    pub fn new() -> Self {
        Self {
            patterns: Self::initialize_patterns(),
        }
    }

    fn initialize_patterns() -> Vec<CommandPattern> {
        let mut patterns = Vec::new();

        // Fill command patterns (addressing the failing test)
        patterns.push(CommandPattern {
            pattern: Regex::new(r"(?i)fill\s+(?:in\s+)?([^\s]+(?:\s+[^\s]+)*?)\s+in(?:\s+the)?\s+(\w+)\s+field").unwrap(),
            action: "type".to_string(),
            confidence_boost: 0.95,
            extractor: Box::new(|caps| {
                ParsedCommand {
                    action: "type".to_string(),
                    value: Some(caps.get(1).map_or("", |m| m.as_str()).to_string()),
                    target: Some(caps.get(2).map_or("", |m| m.as_str()).to_string()),
                    selector: Some(format!("input[name*='{}'], input[placeholder*='{}'], input[type='{}']", 
                        caps.get(2).map_or("", |m| m.as_str()),
                        caps.get(2).map_or("", |m| m.as_str()),
                        caps.get(2).map_or("", |m| m.as_str()))),
                    confidence: 0.95,
                    metadata: HashMap::from([
                        ("field_type".to_string(), caps.get(2).map_or("", |m| m.as_str()).to_string()),
                        ("value".to_string(), caps.get(1).map_or("", |m| m.as_str()).to_string()),
                    ]),
                }
            }),
        });

        // Type command patterns
        patterns.push(CommandPattern {
            pattern: Regex::new(r"(?i)type\s+([^\s]+(?:\s+[^\s]+)*?)\s+in(?:\s+the)?\s+(\w+)").unwrap(),
            action: "type".to_string(),
            confidence_boost: 0.9,
            extractor: Box::new(|caps| {
                ParsedCommand {
                    action: "type".to_string(),
                    value: Some(caps.get(1).map_or("", |m| m.as_str()).to_string()),
                    target: Some(caps.get(2).map_or("", |m| m.as_str()).to_string()),
                    selector: None,
                    confidence: 0.9,
                    metadata: HashMap::new(),
                }
            }),
        });

        // Enter/Input patterns
        patterns.push(CommandPattern {
            pattern: Regex::new(r"(?i)(?:enter|input)\s+([^\s]+(?:\s+[^\s]+)*?)").unwrap(),
            action: "type".to_string(),
            confidence_boost: 0.85,
            extractor: Box::new(|caps| {
                ParsedCommand {
                    action: "type".to_string(),
                    value: Some(caps.get(1).map_or("", |m| m.as_str()).to_string()),
                    target: None,
                    selector: None,
                    confidence: 0.85,
                    metadata: HashMap::new(),
                }
            }),
        });

        // Click patterns
        patterns.push(CommandPattern {
            pattern: Regex::new(r"(?i)click\s+(?:on\s+)?(?:the\s+)?(.+?)(?:\s+button|\s+link|\s+element)?$").unwrap(),
            action: "click".to_string(),
            confidence_boost: 0.9,
            extractor: Box::new(|caps| {
                ParsedCommand {
                    action: "click".to_string(),
                    target: Some(caps.get(1).map_or("", |m| m.as_str()).to_string()),
                    value: None,
                    selector: None,
                    confidence: 0.9,
                    metadata: HashMap::new(),
                }
            }),
        });

        // Navigate patterns
        patterns.push(CommandPattern {
            pattern: Regex::new(r"(?i)(?:go\s+to|navigate\s+to|open)\s+(.+)").unwrap(),
            action: "navigate".to_string(),
            confidence_boost: 0.95,
            extractor: Box::new(|caps| {
                ParsedCommand {
                    action: "navigate".to_string(),
                    target: Some(caps.get(1).map_or("", |m| m.as_str()).to_string()),
                    value: None,
                    selector: None,
                    confidence: 0.95,
                    metadata: HashMap::new(),
                }
            }),
        });

        // Search patterns
        patterns.push(CommandPattern {
            pattern: Regex::new(r"(?i)search\s+(?:for\s+)?(.+)").unwrap(),
            action: "search".to_string(),
            confidence_boost: 0.9,
            extractor: Box::new(|caps| {
                ParsedCommand {
                    action: "search".to_string(),
                    value: Some(caps.get(1).map_or("", |m| m.as_str()).to_string()),
                    target: None,
                    selector: None,
                    confidence: 0.9,
                    metadata: HashMap::new(),
                }
            }),
        });

        // Select dropdown patterns
        patterns.push(CommandPattern {
            pattern: Regex::new(r"(?i)select\s+(.+?)\s+from(?:\s+the)?\s+(\w+)").unwrap(),
            action: "select".to_string(),
            confidence_boost: 0.9,
            extractor: Box::new(|caps| {
                ParsedCommand {
                    action: "select".to_string(),
                    value: Some(caps.get(1).map_or("", |m| m.as_str()).to_string()),
                    target: Some(caps.get(2).map_or("", |m| m.as_str()).to_string()),
                    selector: None,
                    confidence: 0.9,
                    metadata: HashMap::new(),
                }
            }),
        });

        // Scroll patterns
        patterns.push(CommandPattern {
            pattern: Regex::new(r"(?i)scroll\s+(?:down|up|to)?\s*(.*)").unwrap(),
            action: "scroll".to_string(),
            confidence_boost: 0.9,
            extractor: Box::new(|caps| {
                ParsedCommand {
                    action: "scroll".to_string(),
                    target: caps.get(1).map(|m| m.as_str().to_string()),
                    value: None,
                    selector: None,
                    confidence: 0.9,
                    metadata: HashMap::new(),
                }
            }),
        });

        // Screenshot patterns
        patterns.push(CommandPattern {
            pattern: Regex::new(r"(?i)(?:take\s+a?\s*)?screenshot").unwrap(),
            action: "take_screenshot".to_string(),
            confidence_boost: 0.95,
            extractor: Box::new(|_| {
                ParsedCommand {
                    action: "take_screenshot".to_string(),
                    target: None,
                    value: None,
                    selector: None,
                    confidence: 0.95,
                    metadata: HashMap::new(),
                }
            }),
        });

        // Extract patterns
        patterns.push(CommandPattern {
            pattern: Regex::new(r"(?i)extract\s+(?:all\s+)?(.+?)(?:\s+from\s+(?:the\s+)?page)?").unwrap(),
            action: "extract".to_string(),
            confidence_boost: 0.85,
            extractor: Box::new(|caps| {
                ParsedCommand {
                    action: "extract".to_string(),
                    target: Some(caps.get(1).map_or("text", |m| m.as_str()).to_string()),
                    value: None,
                    selector: None,
                    confidence: 0.85,
                    metadata: HashMap::new(),
                }
            }),
        });

        // Find patterns
        patterns.push(CommandPattern {
            pattern: Regex::new(r"(?i)find\s+(?:all\s+)?(.+?)(?:\s+on\s+(?:the\s+)?page)?").unwrap(),
            action: "find".to_string(),
            confidence_boost: 0.9,
            extractor: Box::new(|caps| {
                ParsedCommand {
                    action: "find".to_string(),
                    target: Some(caps.get(1).map_or("", |m| m.as_str()).to_string()),
                    value: None,
                    selector: None,
                    confidence: 0.9,
                    metadata: HashMap::new(),
                }
            }),
        });

        // Wait patterns
        patterns.push(CommandPattern {
            pattern: Regex::new(r"(?i)wait\s+(?:for\s+)?(.+)").unwrap(),
            action: "wait".to_string(),
            confidence_boost: 0.9,
            extractor: Box::new(|caps| {
                ParsedCommand {
                    action: "wait".to_string(),
                    target: Some(caps.get(1).map_or("", |m| m.as_str()).to_string()),
                    value: None,
                    selector: None,
                    confidence: 0.9,
                    metadata: HashMap::new(),
                }
            }),
        });

        patterns
    }

    /// Parse a natural language command
    pub fn parse(&self, command: &str) -> ParsedCommand {
        // Try each pattern in order
        for pattern_def in &self.patterns {
            if let Some(captures) = pattern_def.pattern.captures(command) {
                return (pattern_def.extractor)(&captures);
            }
        }

        // If no pattern matches, try to intelligently guess the action
        self.intelligent_fallback(command)
    }

    /// Intelligent fallback when no pattern matches
    fn intelligent_fallback(&self, command: &str) -> ParsedCommand {
        let command_lower = command.to_lowercase();
        
        // Check for action keywords
        let action = if command_lower.contains("fill") || command_lower.contains("type") || command_lower.contains("enter") {
            "type"
        } else if command_lower.contains("click") || command_lower.contains("press") || command_lower.contains("tap") {
            "click"
        } else if command_lower.contains("go") || command_lower.contains("navigate") || command_lower.contains("open") {
            "navigate"
        } else if command_lower.contains("search") || command_lower.contains("find") || command_lower.contains("look") {
            "search"
        } else if command_lower.contains("select") || command_lower.contains("choose") || command_lower.contains("pick") {
            "select"
        } else if command_lower.contains("scroll") || command_lower.contains("swipe") {
            "scroll"
        } else if command_lower.contains("screenshot") || command_lower.contains("capture") {
            "take_screenshot"
        } else if command_lower.contains("extract") || command_lower.contains("get") || command_lower.contains("read") {
            "extract"
        } else if command_lower.contains("wait") || command_lower.contains("pause") {
            "wait"
        } else {
            "unknown"
        };

        ParsedCommand {
            action: action.to_string(),
            target: Some(command.to_string()),
            value: None,
            selector: None,
            confidence: if action == "unknown" { 0.3 } else { 0.6 },
            metadata: HashMap::from([
                ("fallback".to_string(), "true".to_string()),
                ("original_command".to_string(), command.to_string()),
            ]),
        }
    }

    /// Check if a command is a multi-step workflow
    pub fn is_multi_step(&self, command: &str) -> bool {
        // Check for conjunctions that indicate multiple steps
        command.contains(" and ") || 
        command.contains(", then ") || 
        command.contains(", ") ||
        command.contains(" then ")
    }

    /// Split a multi-step command into individual steps
    pub fn split_multi_step(&self, command: &str) -> Vec<String> {
        let mut steps = Vec::new();
        
        // Split by various conjunctions
        let parts: Vec<&str> = if command.contains(" and ") {
            command.split(" and ").collect()
        } else if command.contains(", then ") {
            command.split(", then ").collect()
        } else if command.contains(" then ") {
            command.split(" then ").collect()
        } else if command.contains(", ") {
            command.split(", ").collect()
        } else {
            vec![command]
        };

        for part in parts {
            let trimmed = part.trim();
            if !trimmed.is_empty() {
                steps.push(trimmed.to_string());
            }
        }

        steps
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fill_command_parsing() {
        let parser = EnhancedCommandParser::new();
        
        // Test the failing command from integration test
        let result = parser.parse("fill email@example.com in the email field");
        assert_eq!(result.action, "type");
        assert_eq!(result.value, Some("email@example.com".to_string()));
        assert_eq!(result.target, Some("email".to_string()));
        assert!(result.confidence >= 0.9);
        
        // Test variations
        let result2 = parser.parse("fill in john.doe@test.com in email field");
        assert_eq!(result2.action, "type");
        assert_eq!(result2.value, Some("john.doe@test.com".to_string()));
        
        let result3 = parser.parse("fill password123 in the password field");
        assert_eq!(result3.action, "type");
        assert_eq!(result3.target, Some("password".to_string()));
    }

    #[test]
    fn test_type_command_variations() {
        let parser = EnhancedCommandParser::new();
        
        let result = parser.parse("type hello world in search");
        assert_eq!(result.action, "type");
        assert_eq!(result.value, Some("hello world".to_string()));
        
        let result2 = parser.parse("enter test data");
        assert_eq!(result2.action, "type");
        assert_eq!(result2.value, Some("test data".to_string()));
    }

    #[test]
    fn test_multi_step_detection() {
        let parser = EnhancedCommandParser::new();
        
        assert!(parser.is_multi_step("go to google and search for AI"));
        assert!(parser.is_multi_step("navigate to example.com, fill the form, and submit"));
        assert!(!parser.is_multi_step("click the button"));
        
        let steps = parser.split_multi_step("go to google and search for AI");
        assert_eq!(steps.len(), 2);
        assert_eq!(steps[0], "go to google");
        assert_eq!(steps[1], "search for AI");
    }

    #[test]
    fn test_intelligent_fallback() {
        let parser = EnhancedCommandParser::new();
        
        // Test unmatched but guessable commands
        let result = parser.parse("please fill out the form with my details");
        assert_eq!(result.action, "type");
        assert!(result.confidence >= 0.6);
        
        let result2 = parser.parse("tap on the next button");
        assert_eq!(result2.action, "click");
        
        let result3 = parser.parse("look for products");
        assert_eq!(result3.action, "search");
    }
}