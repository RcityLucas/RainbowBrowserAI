//! Tool Mapping - MVP Phase 1
//! 
//! Maps perception results to available tools

use serde::{Deserialize, Serialize};
use super::ElementInfo;

/// Tool suggestion from perception
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSuggestion {
    pub tool_name: String,
    pub element_id: String,
    pub parameters: serde_json::Value,
    pub confidence: f32,
    pub description: String,
}

/// Maps elements to appropriate tools
pub struct ToolMapper {
    available_tools: Vec<String>,
}

impl ToolMapper {
    pub fn new() -> Self {
        Self {
            available_tools: vec![
                "navigate".to_string(),
                "click".to_string(),
                "type_text".to_string(),
                "take_screenshot".to_string(),
                "wait_for_element".to_string(),
                "scroll_page".to_string(),
            ],
        }
    }
    
    /// Suggest tools based on detected elements
    pub fn suggest_tools(&self, elements: &[ElementInfo]) -> Vec<ToolSuggestion> {
        let mut suggestions = Vec::new();
        
        for element in elements {
            if let Some(suggestion) = self.map_element_to_tool(element) {
                suggestions.push(suggestion);
            }
        }
        
        // Sort by confidence
        suggestions.sort_by(|a, b| {
            b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        suggestions
    }
    
    /// Map a single element to appropriate tool
    fn map_element_to_tool(&self, element: &ElementInfo) -> Option<ToolSuggestion> {
        match element.element_type.as_str() {
            "button" => Some(ToolSuggestion {
                tool_name: "click".to_string(),
                element_id: element.id.clone(),
                parameters: serde_json::json!({
                    "selector": element.selector.clone(),
                    "wait_after": 500,
                }),
                confidence: element.confidence * 0.9,
                description: format!("Click on button: {}", element.text.as_ref().unwrap_or(&"Button".to_string())),
            }),
            
            "link" => Some(ToolSuggestion {
                tool_name: "click".to_string(),
                element_id: element.id.clone(),
                parameters: serde_json::json!({
                    "selector": element.selector.clone(),
                    "wait_for_navigation": true,
                }),
                confidence: element.confidence * 0.85,
                description: format!("Click on link: {}", element.text.as_ref().unwrap_or(&"Link".to_string())),
            }),
            
            "input" | "textarea" => Some(ToolSuggestion {
                tool_name: "type_text".to_string(),
                element_id: element.id.clone(),
                parameters: serde_json::json!({
                    "selector": element.selector.clone(),
                    "clear_first": true,
                }),
                confidence: element.confidence * 0.9,
                description: format!("Type text in: {}", element.text.as_ref().unwrap_or(&"Input field".to_string())),
            }),
            
            "select" => Some(ToolSuggestion {
                tool_name: "select_option".to_string(),
                element_id: element.id.clone(),
                parameters: serde_json::json!({
                    "selector": element.selector.clone(),
                }),
                confidence: element.confidence * 0.8,
                description: "Select dropdown option".to_string(),
            }),
            
            _ => None,
        }
    }
    
    /// Get tool suggestions for a specific action
    pub fn suggest_for_action(&self, action: &str, elements: &[ElementInfo]) -> Vec<ToolSuggestion> {
        match action {
            "submit_form" => {
                // Find submit button or form
                elements.iter()
                    .filter(|e| {
                        e.element_type == "button" 
                        && e.text.as_ref().map_or(false, |t| {
                            t.to_lowercase().contains("submit") 
                            || t.to_lowercase().contains("send")
                            || t.to_lowercase().contains("ok")
                        })
                    })
                    .filter_map(|e| self.map_element_to_tool(e))
                    .collect()
            },
            
            "fill_form" => {
                // Find all input fields
                elements.iter()
                    .filter(|e| e.element_type == "input" || e.element_type == "textarea")
                    .filter_map(|e| self.map_element_to_tool(e))
                    .collect()
            },
            
            "navigate_menu" => {
                // Find navigation links
                elements.iter()
                    .filter(|e| e.element_type == "link")
                    .filter_map(|e| self.map_element_to_tool(e))
                    .collect()
            },
            
            _ => self.suggest_tools(elements),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_tool_mapping() {
        let mapper = ToolMapper::new();
        
        let elements = vec![
            ElementInfo {
                id: "btn1".to_string(),
                selector: "button#btn1".to_string(),
                element_type: "button".to_string(),
                text: Some("Submit".to_string()),
                is_visible: true,
                is_clickable: true,
                confidence: 0.9,
            },
            ElementInfo {
                id: "input1".to_string(),
                selector: "input#input1".to_string(),
                element_type: "input".to_string(),
                text: Some("Email".to_string()),
                is_visible: true,
                is_clickable: false,
                confidence: 0.85,
            },
        ];
        
        let suggestions = mapper.suggest_tools(&elements);
        
        assert_eq!(suggestions.len(), 2);
        assert_eq!(suggestions[0].tool_name, "click");
        assert_eq!(suggestions[1].tool_name, "type_text");
    }
    
    #[test]
    fn test_action_specific_suggestions() {
        let mapper = ToolMapper::new();
        
        let elements = vec![
            ElementInfo {
                id: "submit".to_string(),
                selector: "button#submit".to_string(),
                element_type: "button".to_string(),
                text: Some("Submit Form".to_string()),
                is_visible: true,
                is_clickable: true,
                confidence: 0.95,
            },
            ElementInfo {
                id: "cancel".to_string(),
                selector: "button#cancel".to_string(),
                element_type: "button".to_string(),
                text: Some("Cancel".to_string()),
                is_visible: true,
                is_clickable: true,
                confidence: 0.9,
            },
        ];
        
        let suggestions = mapper.suggest_for_action("submit_form", &elements);
        
        assert_eq!(suggestions.len(), 1);
        assert_eq!(suggestions[0].element_id, "submit");
    }
}