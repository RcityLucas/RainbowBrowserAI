//! Element Detection - MVP Phase 1
//! 
//! Simple but effective element detection for common web patterns

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Basic element information for MVP
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementInfo {
    pub id: String,
    pub selector: String,
    pub element_type: String,
    pub text: Option<String>,
    pub is_visible: bool,
    pub is_clickable: bool,
    pub confidence: f32,
}

impl ElementInfo {
    /// Create a simple clickable button element
    pub fn button(id: &str, text: &str) -> Self {
        Self {
            id: id.to_string(),
            selector: format!("button#{}", id),
            element_type: "button".to_string(),
            text: Some(text.to_string()),
            is_visible: true,
            is_clickable: true,
            confidence: 0.9,
        }
    }
    
    /// Create a simple input field element
    pub fn input(id: &str, placeholder: Option<&str>) -> Self {
        Self {
            id: id.to_string(),
            selector: format!("input#{}", id),
            element_type: "input".to_string(),
            text: placeholder.map(|s| s.to_string()),
            is_visible: true,
            is_clickable: false,
            confidence: 0.9,
        }
    }
    
    /// Create a simple link element
    pub fn link(id: &str, text: &str) -> Self {
        Self {
            id: id.to_string(),
            selector: format!("a#{}", id),
            element_type: "link".to_string(),
            text: Some(text.to_string()),
            is_visible: true,
            is_clickable: true,
            confidence: 0.85,
        }
    }
}

/// Simple element detector for MVP
pub struct ElementDetector {
    min_confidence: f32,
}

impl ElementDetector {
    pub fn new() -> Self {
        Self {
            min_confidence: 0.6,
        }
    }
    
    /// Detect essential elements on a page
    pub async fn detect_essential_elements(&self, _page_url: &str) -> Result<Vec<ElementInfo>> {
        // For MVP, return mock elements for testing
        // In production, this would interact with the browser
        
        let mut elements = vec![
            ElementInfo::button("submit-btn", "Submit"),
            ElementInfo::button("cancel-btn", "Cancel"),
            ElementInfo::input("search-field", Some("Search...")),
            ElementInfo::input("email-field", Some("Email")),
            ElementInfo::link("home-link", "Home"),
            ElementInfo::link("about-link", "About"),
        ];
        
        // Filter by confidence
        elements.retain(|e| e.confidence >= self.min_confidence);
        
        // Sort by confidence (highest first)
        elements.sort_by(|a, b| {
            b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        Ok(elements)
    }
    
    /// Find elements by type
    pub fn find_by_type(&self, elements: &[ElementInfo], element_type: &str) -> Vec<ElementInfo> {
        elements
            .iter()
            .filter(|e| e.element_type == element_type)
            .cloned()
            .collect()
    }
    
    /// Find clickable elements
    pub fn find_clickable(&self, elements: &[ElementInfo]) -> Vec<ElementInfo> {
        elements
            .iter()
            .filter(|e| e.is_clickable && e.is_visible)
            .cloned()
            .collect()
    }
    
    /// Find form inputs
    pub fn find_inputs(&self, elements: &[ElementInfo]) -> Vec<ElementInfo> {
        elements
            .iter()
            .filter(|e| {
                e.element_type == "input" 
                || e.element_type == "textarea" 
                || e.element_type == "select"
            })
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_element_detection() {
        let detector = ElementDetector::new();
        let elements = detector.detect_essential_elements("test").await.unwrap();
        
        assert!(!elements.is_empty());
        assert!(elements.iter().all(|e| e.confidence >= 0.6));
    }
    
    #[test]
    fn test_element_filtering() {
        let detector = ElementDetector::new();
        let elements = vec![
            ElementInfo::button("btn1", "Click"),
            ElementInfo::input("input1", None),
            ElementInfo::link("link1", "Home"),
        ];
        
        let clickable = detector.find_clickable(&elements);
        assert_eq!(clickable.len(), 2); // button and link
        
        let inputs = detector.find_inputs(&elements);
        assert_eq!(inputs.len(), 1); // input field
    }
}