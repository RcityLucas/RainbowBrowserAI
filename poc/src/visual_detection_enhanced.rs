// Enhanced Visual Detection Module
// Improves visual element recognition and logo/icon detection

use std::collections::HashMap;
use anyhow::Result;
use serde::{Serialize, Deserialize};

/// Enhanced visual detection system
pub struct VisualDetectionEnhanced {
    patterns: HashMap<String, VisualPattern>,
    confidence_threshold: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualPattern {
    pub name: String,
    pub selectors: Vec<String>,
    pub attributes: Vec<String>,
    pub confidence_boost: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualDetectionResult {
    pub element_selector: String,
    pub element_type: String,
    pub confidence: f32,
    pub reasoning: String,
}

impl VisualDetectionEnhanced {
    pub fn new() -> Self {
        let mut patterns = HashMap::new();
        
        // Logo patterns
        patterns.insert("logo".to_string(), VisualPattern {
            name: "Logo".to_string(),
            selectors: vec![
                "img[class*='logo']".to_string(),
                "img[id*='logo']".to_string(),
                "img[alt*='logo']".to_string(),
                "a[class*='logo'] img".to_string(),
                "a[href='/'] img".to_string(),
                ".logo img".to_string(),
                "#logo img".to_string(),
                "header img".to_string(),
                "nav img".to_string(),
                "img[src*='logo']".to_string(),
                "svg[class*='logo']".to_string(),
                ".brand img".to_string(),
                ".navbar-brand img".to_string(),
            ],
            attributes: vec!["logo".to_string(), "brand".to_string(), "home".to_string()],
            confidence_boost: 0.95,
        });
        
        // Icon patterns
        patterns.insert("icon".to_string(), VisualPattern {
            name: "Icon".to_string(),
            selectors: vec![
                "i[class*='icon']".to_string(),
                "span[class*='icon']".to_string(),
                "svg[class*='icon']".to_string(),
                ".icon".to_string(),
                ".fa".to_string(),
                ".fas".to_string(),
                ".material-icons".to_string(),
                "button i".to_string(),
                "button svg".to_string(),
            ],
            attributes: vec!["icon".to_string(), "glyph".to_string()],
            confidence_boost: 0.85,
        });
        
        // Shopping cart patterns
        patterns.insert("cart".to_string(), VisualPattern {
            name: "Shopping Cart".to_string(),
            selectors: vec![
                "[class*='cart']".to_string(),
                "[id*='cart']".to_string(),
                "[aria-label*='cart']".to_string(),
                "button[class*='cart']".to_string(),
                "a[href*='cart']".to_string(),
                ".shopping-cart".to_string(),
                "#shopping-cart".to_string(),
                "[data-test*='cart']".to_string(),
            ],
            attributes: vec!["cart".to_string(), "basket".to_string(), "bag".to_string()],
            confidence_boost: 0.9,
        });
        
        // Button patterns
        patterns.insert("button".to_string(), VisualPattern {
            name: "Button".to_string(),
            selectors: vec![
                "button".to_string(),
                "input[type='button']".to_string(),
                "input[type='submit']".to_string(),
                "a.button".to_string(),
                "a.btn".to_string(),
                "[role='button']".to_string(),
                ".button".to_string(),
                ".btn".to_string(),
            ],
            attributes: vec!["button".to_string(), "btn".to_string(), "submit".to_string()],
            confidence_boost: 0.8,
        });
        
        // Color-based patterns
        patterns.insert("green_button".to_string(), VisualPattern {
            name: "Green Button".to_string(),
            selectors: vec![
                "button.green".to_string(),
                "button.btn-green".to_string(),
                "button.btn-success".to_string(),
                "button[style*='green']".to_string(),
                "button[style*='#00ff00']".to_string(),
                "button[style*='#008000']".to_string(),
                ".green-button".to_string(),
                ".success-button".to_string(),
            ],
            attributes: vec!["green".to_string(), "success".to_string(), "go".to_string()],
            confidence_boost: 0.85,
        });
        
        // Image patterns
        patterns.insert("image".to_string(), VisualPattern {
            name: "Image".to_string(),
            selectors: vec![
                "img".to_string(),
                "picture img".to_string(),
                "figure img".to_string(),
                ".image img".to_string(),
                "[role='img']".to_string(),
            ],
            attributes: vec!["image".to_string(), "photo".to_string(), "picture".to_string()],
            confidence_boost: 0.75,
        });
        
        Self {
            patterns,
            confidence_threshold: 0.5,
        }
    }
    
    /// Detect visual element based on description
    pub fn detect_element(&self, description: &str) -> Result<VisualDetectionResult> {
        let description_lower = description.to_lowercase();
        
        // Check for specific pattern matches
        for (key, pattern) in &self.patterns {
            if description_lower.contains(key) || 
               pattern.attributes.iter().any(|attr| description_lower.contains(attr)) {
                
                // Find the best selector
                for selector in &pattern.selectors {
                    // In real implementation, this would check if element exists
                    // For now, return the most likely selector
                    if self.is_high_confidence_selector(selector) {
                        return Ok(VisualDetectionResult {
                            element_selector: selector.clone(),
                            element_type: pattern.name.clone(),
                            confidence: pattern.confidence_boost,
                            reasoning: format!("Detected {} pattern in '{}'", pattern.name, description),
                        });
                    }
                }
                
                // Return first selector if no high confidence one found
                if let Some(first_selector) = pattern.selectors.first() {
                    return Ok(VisualDetectionResult {
                        element_selector: first_selector.clone(),
                        element_type: pattern.name.clone(),
                        confidence: pattern.confidence_boost * 0.8,
                        reasoning: format!("Best match for {} pattern", pattern.name),
                    });
                }
            }
        }
        
        // Fallback detection strategies
        self.fallback_detection(description)
    }
    
    /// Check if selector is high confidence
    fn is_high_confidence_selector(&self, selector: &str) -> bool {
        // Prioritize specific selectors over generic ones
        selector.contains("id=") || 
        selector.contains("#") ||
        selector.contains("[aria-label") ||
        selector.contains("[data-test")
    }
    
    /// Fallback detection when no pattern matches
    fn fallback_detection(&self, description: &str) -> Result<VisualDetectionResult> {
        let description_lower = description.to_lowercase();
        
        // Try to construct a selector based on the description
        let mut selectors = Vec::new();
        let mut element_type = "Unknown";
        
        if description_lower.contains("button") {
            selectors.push("button".to_string());
            selectors.push("input[type='button']".to_string());
            element_type = "Button";
        } else if description_lower.contains("link") {
            selectors.push("a".to_string());
            element_type = "Link";
        } else if description_lower.contains("image") || description_lower.contains("img") {
            selectors.push("img".to_string());
            element_type = "Image";
        } else if description_lower.contains("text") || description_lower.contains("input") {
            selectors.push("input[type='text']".to_string());
            element_type = "Input";
        }
        
        // Add attribute-based selectors
        let words: Vec<&str> = description_lower.split_whitespace().collect();
        for word in words {
            if word.len() > 3 {
                selectors.push(format!("[class*='{}']", word));
                selectors.push(format!("[id*='{}']", word));
                selectors.push(format!("[aria-label*='{}']", word));
            }
        }
        
        if !selectors.is_empty() {
            Ok(VisualDetectionResult {
                element_selector: selectors.join(", "),
                element_type: element_type.to_string(),
                confidence: 0.6,
                reasoning: format!("Fallback detection for '{}'", description),
            })
        } else {
            Ok(VisualDetectionResult {
                element_selector: "*".to_string(),
                element_type: "Unknown".to_string(),
                confidence: 0.3,
                reasoning: "No specific pattern detected".to_string(),
            })
        }
    }
    
    /// Detect elements by color
    pub fn detect_by_color(&self, color: &str) -> Vec<String> {
        let color_lower = color.to_lowercase();
        let mut selectors = Vec::new();
        
        match color_lower.as_str() {
            "green" => {
                selectors.push(".green".to_string());
                selectors.push(".btn-green".to_string());
                selectors.push(".btn-success".to_string());
                selectors.push("[style*='green']".to_string());
                selectors.push("[style*='#00ff00']".to_string());
                selectors.push("[style*='#008000']".to_string());
                selectors.push("[class*='success']".to_string());
            },
            "red" => {
                selectors.push(".red".to_string());
                selectors.push(".btn-red".to_string());
                selectors.push(".btn-danger".to_string());
                selectors.push("[style*='red']".to_string());
                selectors.push("[style*='#ff0000']".to_string());
                selectors.push("[class*='danger']".to_string());
                selectors.push("[class*='error']".to_string());
            },
            "blue" => {
                selectors.push(".blue".to_string());
                selectors.push(".btn-blue".to_string());
                selectors.push(".btn-primary".to_string());
                selectors.push("[style*='blue']".to_string());
                selectors.push("[style*='#0000ff']".to_string());
                selectors.push("[class*='primary']".to_string());
                selectors.push("[class*='info']".to_string());
            },
            _ => {
                selectors.push(format!(".{}", color_lower));
                selectors.push(format!("[class*='{}']", color_lower));
                selectors.push(format!("[style*='{}']", color_lower));
            }
        }
        
        selectors
    }
    
    /// Enhance command with visual detection
    pub fn enhance_command(&self, command: &str) -> String {
        let command_lower = command.to_lowercase();
        
        // Check if command contains visual references
        if command_lower.contains("logo") {
            if let Ok(result) = self.detect_element("logo") {
                return command.replace("logo", &format!("element with selector {}", result.element_selector));
            }
        }
        
        if command_lower.contains("icon") {
            if let Ok(result) = self.detect_element(&command_lower) {
                return command.replace("icon", &format!("element with selector {}", result.element_selector));
            }
        }
        
        // Check for color-based commands
        let colors = vec!["green", "red", "blue", "yellow", "orange", "purple"];
        for color in colors {
            if command_lower.contains(color) {
                let selectors = self.detect_by_color(color);
                if !selectors.is_empty() {
                    return command.replace(
                        &format!("{} button", color),
                        &format!("button with selector {}", selectors.join(" or "))
                    );
                }
            }
        }
        
        command.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_logo_detection() {
        let detector = VisualDetectionEnhanced::new();
        
        let result = detector.detect_element("click on the logo").unwrap();
        assert_eq!(result.element_type, "Logo");
        assert!(result.confidence >= 0.8);
        assert!(result.element_selector.contains("logo"));
    }
    
    #[test]
    fn test_color_detection() {
        let detector = VisualDetectionEnhanced::new();
        
        let selectors = detector.detect_by_color("green");
        assert!(!selectors.is_empty());
        assert!(selectors.iter().any(|s| s.contains("green") || s.contains("success")));
    }
    
    #[test]
    fn test_cart_detection() {
        let detector = VisualDetectionEnhanced::new();
        
        let result = detector.detect_element("click shopping cart icon").unwrap();
        assert_eq!(result.element_type, "Shopping Cart");
        assert!(result.confidence >= 0.8);
    }
    
    #[test]
    fn test_fallback_detection() {
        let detector = VisualDetectionEnhanced::new();
        
        let result = detector.detect_element("random element xyz").unwrap();
        assert!(result.confidence <= 0.6);
        assert_eq!(result.element_type, "Unknown");
    }
    
    #[test]
    fn test_command_enhancement() {
        let detector = VisualDetectionEnhanced::new();
        
        let enhanced = detector.enhance_command("click on the logo");
        assert!(enhanced.contains("element with selector"));
        assert!(enhanced.contains("logo"));
        
        let enhanced2 = detector.enhance_command("click the green button");
        assert!(enhanced2.contains("button with selector"));
    }
}