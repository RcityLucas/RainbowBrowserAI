// Comprehensive tests for Week 2 Interaction Tools
// Tests for click, type_text, and select_option tools

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::browser::MockBrowser;
    use crate::tools::{Tool, ToolError};
    use serde_json::json;
    use std::sync::Arc;
    use std::time::Instant;
    
    // ============================================================================
    // CLICK TOOL TESTS
    // ============================================================================
    
    mod click_tests {
        use super::*;
        
        #[tokio::test]
        async fn test_click_basic() {
            let mock_browser = Arc::new(MockBrowser::new());
            let tool = Click::new(mock_browser.clone());
            
            let params = ClickParams {
                selector: "#submit-button".to_string(),
                options: Default::default(),
            };
            
            let result = tool.execute(params).await;
            assert!(result.is_ok());
            
            let output = result.unwrap();
            assert!(output.success);
            assert_eq!(output.element.tag_name, "button");
        }
        
        #[tokio::test]
        async fn test_click_with_modifiers() {
            let mock_browser = Arc::new(MockBrowser::new());
            let tool = Click::new(mock_browser.clone());
            
            let params = ClickParams {
                selector: "#link".to_string(),
                options: ClickOptions {
                    modifiers: vec![ModifierKey::Control, ModifierKey::Shift],
                    ..Default::default()
                },
            };
            
            let result = tool.execute(params).await;
            assert!(result.is_ok());
            
            let output = result.unwrap();
            assert!(output.events_triggered.contains(&"click".to_string()));
        }
        
        #[tokio::test]
        async fn test_click_double_click() {
            let mock_browser = Arc::new(MockBrowser::new());
            let tool = Click::new(mock_browser.clone());
            
            let params = ClickParams {
                selector: ".editable".to_string(),
                options: ClickOptions {
                    click_count: 2,
                    delay: Some(100),
                    ..Default::default()
                },
            };
            
            let result = tool.execute(params).await;
            assert!(result.is_ok());
        }
        
        #[tokio::test]
        async fn test_click_right_button() {
            let mock_browser = Arc::new(MockBrowser::new());
            let tool = Click::new(mock_browser.clone());
            
            let params = ClickParams {
                selector: "#context-menu-target".to_string(),
                options: ClickOptions {
                    button: MouseButton::Right,
                    ..Default::default()
                },
            };
            
            let result = tool.execute(params).await;
            assert!(result.is_ok());
        }
        
        #[tokio::test]
        async fn test_click_with_offset() {
            let mock_browser = Arc::new(MockBrowser::new());
            let tool = Click::new(mock_browser.clone());
            
            let params = ClickParams {
                selector: "#canvas".to_string(),
                options: ClickOptions {
                    offset: Some(Position { x: 100.0, y: 50.0 }),
                    ..Default::default()
                },
            };
            
            let result = tool.execute(params).await;
            assert!(result.is_ok());
        }
        
        #[tokio::test]
        async fn test_click_force_mode() {
            let mock_browser = Arc::new(MockBrowser::new());
            let tool = Click::new(mock_browser.clone());
            
            let params = ClickParams {
                selector: ".hidden-element".to_string(),
                options: ClickOptions {
                    force: true,
                    ..Default::default()
                },
            };
            
            let result = tool.execute(params).await;
            assert!(result.is_ok());
        }
        
        #[tokio::test]
        async fn test_click_wait_for_element() {
            let mock_browser = Arc::new(MockBrowser::new());
            let tool = Click::new(mock_browser.clone());
            
            let params = ClickParams {
                selector: "#dynamic-button".to_string(),
                options: ClickOptions {
                    wait_for: Some(WaitOptions {
                        visible: true,
                        enabled: true,
                        stable: false,
                        timeout: 5000,
                    }),
                    ..Default::default()
                },
            };
            
            let result = tool.execute(params).await;
            assert!(result.is_ok());
        }
        
        #[tokio::test]
        async fn test_click_invalid_selector() {
            let mock_browser = Arc::new(MockBrowser::new());
            let tool = Click::new(mock_browser.clone());
            
            let params = ClickParams {
                selector: "".to_string(),
                options: Default::default(),
            };
            
            let validation = tool.validate_input(&params);
            assert!(validation.is_err());
            assert!(matches!(validation.unwrap_err(), ToolError::InvalidInput(_)));
        }
        
        #[tokio::test]
        async fn test_click_invalid_click_count() {
            let mock_browser = Arc::new(MockBrowser::new());
            let tool = Click::new(mock_browser.clone());
            
            let params = ClickParams {
                selector: "#button".to_string(),
                options: ClickOptions {
                    click_count: 4, // Invalid: max is 3
                    ..Default::default()
                },
            };
            
            let validation = tool.validate_input(&params);
            assert!(validation.is_err());
        }
        
        #[tokio::test]
        async fn test_click_schema_validation() {
            let tool = Click::new(Arc::new(MockBrowser::new()));
            
            let input_schema = tool.input_schema();
            assert_eq!(input_schema["type"], "object");
            assert!(input_schema["required"].as_array().unwrap().contains(&json!("selector")));
            
            let output_schema = tool.output_schema();
            assert_eq!(output_schema["type"], "object");
            assert!(output_schema["properties"]["success"].is_object());
        }
    }
    
    // ============================================================================
    // TYPE_TEXT TOOL TESTS
    // ============================================================================
    
    mod type_text_tests {
        use super::*;
        
        #[tokio::test]
        async fn test_type_text_basic() {
            let mock_browser = Arc::new(MockBrowser::new());
            let tool = TypeText::new(mock_browser.clone());
            
            let params = TypeTextParams {
                selector: "#username".to_string(),
                text: "test_user".to_string(),
                options: Default::default(),
            };
            
            let result = tool.execute(params).await;
            assert!(result.is_ok());
            
            let output = result.unwrap();
            assert!(output.success);
            assert_eq!(output.input_element.final_value, "test_user");
        }
        
        #[tokio::test]
        async fn test_type_text_clear_first() {
            let mock_browser = Arc::new(MockBrowser::new());
            let tool = TypeText::new(mock_browser.clone());
            
            let params = TypeTextParams {
                selector: "#email".to_string(),
                text: "new@example.com".to_string(),
                options: TypeTextOptions {
                    clear_first: true,
                    ..Default::default()
                },
            };
            
            let result = tool.execute(params).await;
            assert!(result.is_ok());
        }
        
        #[tokio::test]
        async fn test_type_text_with_delay() {
            let mock_browser = Arc::new(MockBrowser::new());
            let tool = TypeText::new(mock_browser.clone());
            
            let params = TypeTextParams {
                selector: "#search".to_string(),
                text: "slow typing".to_string(),
                options: TypeTextOptions {
                    delay: 100,
                    ..Default::default()
                },
            };
            
            let result = tool.execute(params).await;
            assert!(result.is_ok());
            
            let output = result.unwrap();
            assert!(output.timing.typing > 1000); // Should take at least 1 second for 11 chars
        }
        
        #[tokio::test]
        async fn test_type_text_paste_mode() {
            let mock_browser = Arc::new(MockBrowser::new());
            let tool = TypeText::new(mock_browser.clone());
            
            let params = TypeTextParams {
                selector: "#description".to_string(),
                text: "This is a long text that should be pasted instantly".to_string(),
                options: TypeTextOptions {
                    paste: true,
                    ..Default::default()
                },
            };
            
            let result = tool.execute(params).await;
            assert!(result.is_ok());
            
            let output = result.unwrap();
            assert!(output.timing.typing < 100); // Paste should be nearly instant
        }
        
        #[tokio::test]
        async fn test_type_text_press_enter() {
            let mock_browser = Arc::new(MockBrowser::new());
            let tool = TypeText::new(mock_browser.clone());
            
            let params = TypeTextParams {
                selector: "#search-box".to_string(),
                text: "search query".to_string(),
                options: TypeTextOptions {
                    press_enter: true,
                    ..Default::default()
                },
            };
            
            let result = tool.execute(params).await;
            assert!(result.is_ok());
            
            let output = result.unwrap();
            assert!(output.events_triggered.contains(&"keydown:Enter".to_string()));
        }
        
        #[tokio::test]
        async fn test_type_text_append() {
            let mock_browser = Arc::new(MockBrowser::new());
            let tool = TypeText::new(mock_browser.clone());
            
            let params = TypeTextParams {
                selector: "#notes".to_string(),
                text: " additional text".to_string(),
                options: TypeTextOptions {
                    append: true,
                    ..Default::default()
                },
            };
            
            let result = tool.execute(params).await;
            assert!(result.is_ok());
        }
        
        #[tokio::test]
        async fn test_type_text_validation_pattern() {
            let mock_browser = Arc::new(MockBrowser::new());
            let tool = TypeText::new(mock_browser.clone());
            
            let params = TypeTextParams {
                selector: "#phone".to_string(),
                text: "123-456-7890".to_string(),
                options: TypeTextOptions {
                    validate: Some(ValidationRule::Pattern {
                        regex: r"^\d{3}-\d{3}-\d{4}$".to_string(),
                    }),
                    ..Default::default()
                },
            };
            
            let result = tool.execute(params).await;
            assert!(result.is_ok());
            
            let output = result.unwrap();
            assert!(output.validation.unwrap().passed);
        }
        
        #[tokio::test]
        async fn test_type_text_validation_email() {
            let mock_browser = Arc::new(MockBrowser::new());
            let tool = TypeText::new(mock_browser.clone());
            
            let params = TypeTextParams {
                selector: "#email".to_string(),
                text: "user@example.com".to_string(),
                options: TypeTextOptions {
                    validate: Some(ValidationRule::Email),
                    ..Default::default()
                },
            };
            
            let result = tool.execute(params).await;
            assert!(result.is_ok());
            
            let output = result.unwrap();
            assert!(output.validation.unwrap().passed);
        }
        
        #[tokio::test]
        async fn test_type_text_validation_length() {
            let mock_browser = Arc::new(MockBrowser::new());
            let tool = TypeText::new(mock_browser.clone());
            
            let params = TypeTextParams {
                selector: "#password".to_string(),
                text: "short".to_string(),
                options: TypeTextOptions {
                    validate: Some(ValidationRule::Length {
                        min: Some(8),
                        max: Some(20),
                    }),
                    ..Default::default()
                },
            };
            
            let result = tool.execute(params).await;
            assert!(result.is_ok());
            
            let output = result.unwrap();
            assert!(!output.validation.unwrap().passed); // Too short
        }
        
        #[tokio::test]
        async fn test_type_text_invalid_params() {
            let tool = TypeText::new(Arc::new(MockBrowser::new()));
            
            // Empty selector
            let params = TypeTextParams {
                selector: "".to_string(),
                text: "test".to_string(),
                options: Default::default(),
            };
            assert!(tool.validate_input(&params).is_err());
            
            // Both append and prepend
            let params = TypeTextParams {
                selector: "#input".to_string(),
                text: "test".to_string(),
                options: TypeTextOptions {
                    append: true,
                    prepend: true,
                    ..Default::default()
                },
            };
            assert!(tool.validate_input(&params).is_err());
        }
    }
    
    // ============================================================================
    // SELECT_OPTION TOOL TESTS
    // ============================================================================
    
    mod select_option_tests {
        use super::*;
        
        #[tokio::test]
        async fn test_select_single_by_value() {
            let mock_browser = Arc::new(MockBrowser::new());
            let tool = SelectOption::new(mock_browser.clone());
            
            let params = SelectOptionParams {
                selector: "#country".to_string(),
                value: SelectValue::Single("US".to_string()),
                options: Default::default(),
            };
            
            let result = tool.execute(params).await;
            assert!(result.is_ok());
            
            let output = result.unwrap();
            assert!(output.success);
            assert_eq!(output.selected.len(), 1);
            assert_eq!(output.selected[0].value, "US");
        }
        
        #[tokio::test]
        async fn test_select_multiple_values() {
            let mock_browser = Arc::new(MockBrowser::new());
            let tool = SelectOption::new(mock_browser.clone());
            
            let params = SelectOptionParams {
                selector: "#languages".to_string(),
                value: SelectValue::Multiple(vec![
                    "en".to_string(),
                    "es".to_string(),
                    "fr".to_string(),
                ]),
                options: Default::default(),
            };
            
            let result = tool.execute(params).await;
            assert!(result.is_ok());
            
            let output = result.unwrap();
            assert_eq!(output.selected.len(), 3);
        }
        
        #[tokio::test]
        async fn test_select_by_text() {
            let mock_browser = Arc::new(MockBrowser::new());
            let tool = SelectOption::new(mock_browser.clone());
            
            let params = SelectOptionParams {
                selector: "#size".to_string(),
                value: SelectValue::Single("Large".to_string()),
                options: SelectOptions {
                    by: SelectBy::Text,
                    ..Default::default()
                },
            };
            
            let result = tool.execute(params).await;
            assert!(result.is_ok());
        }
        
        #[tokio::test]
        async fn test_select_by_index() {
            let mock_browser = Arc::new(MockBrowser::new());
            let tool = SelectOption::new(mock_browser.clone());
            
            let params = SelectOptionParams {
                selector: "#priority".to_string(),
                value: SelectValue::Single("2".to_string()),
                options: SelectOptions {
                    by: SelectBy::Index,
                    ..Default::default()
                },
            };
            
            let result = tool.execute(params).await;
            assert!(result.is_ok());
        }
        
        #[tokio::test]
        async fn test_select_by_partial_text() {
            let mock_browser = Arc::new(MockBrowser::new());
            let tool = SelectOption::new(mock_browser.clone());
            
            let params = SelectOptionParams {
                selector: "#state".to_string(),
                value: SelectValue::Single("Calif".to_string()),
                options: SelectOptions {
                    by: SelectBy::PartialText,
                    ..Default::default()
                },
            };
            
            let result = tool.execute(params).await;
            assert!(result.is_ok());
        }
        
        #[tokio::test]
        async fn test_select_deselect_others() {
            let mock_browser = Arc::new(MockBrowser::new());
            let tool = SelectOption::new(mock_browser.clone());
            
            let params = SelectOptionParams {
                selector: "#multi-select".to_string(),
                value: SelectValue::Single("option1".to_string()),
                options: SelectOptions {
                    deselect_others: true,
                    ..Default::default()
                },
            };
            
            let result = tool.execute(params).await;
            assert!(result.is_ok());
            
            let output = result.unwrap();
            assert_eq!(output.selected.len(), 1);
            assert!(!output.deselected.is_empty());
        }
        
        #[tokio::test]
        async fn test_select_wait_for_options() {
            let mock_browser = Arc::new(MockBrowser::new());
            let tool = SelectOption::new(mock_browser.clone());
            
            let params = SelectOptionParams {
                selector: "#dynamic-select".to_string(),
                value: SelectValue::Single("delayed-option".to_string()),
                options: SelectOptions {
                    wait_for_options: true,
                    min_options: 5,
                    timeout: 3000,
                    ..Default::default()
                },
            };
            
            let result = tool.execute(params).await;
            assert!(result.is_ok());
        }
        
        #[tokio::test]
        async fn test_select_force_disabled() {
            let mock_browser = Arc::new(MockBrowser::new());
            let tool = SelectOption::new(mock_browser.clone());
            
            let params = SelectOptionParams {
                selector: "#status".to_string(),
                value: SelectValue::Single("disabled-option".to_string()),
                options: SelectOptions {
                    force: true,
                    ..Default::default()
                },
            };
            
            let result = tool.execute(params).await;
            assert!(result.is_ok());
        }
        
        #[tokio::test]
        async fn test_select_invalid_params() {
            let tool = SelectOption::new(Arc::new(MockBrowser::new()));
            
            // Empty selector
            let params = SelectOptionParams {
                selector: "".to_string(),
                value: SelectValue::Single("value".to_string()),
                options: Default::default(),
            };
            assert!(tool.validate_input(&params).is_err());
            
            // Empty value
            let params = SelectOptionParams {
                selector: "#select".to_string(),
                value: SelectValue::Single("".to_string()),
                options: Default::default(),
            };
            assert!(tool.validate_input(&params).is_err());
            
            // Invalid index
            let params = SelectOptionParams {
                selector: "#select".to_string(),
                value: SelectValue::Single("not-a-number".to_string()),
                options: SelectOptions {
                    by: SelectBy::Index,
                    ..Default::default()
                },
            };
            assert!(tool.validate_input(&params).is_err());
        }
        
        #[tokio::test]
        async fn test_select_schema_validation() {
            let tool = SelectOption::new(Arc::new(MockBrowser::new()));
            
            let input_schema = tool.input_schema();
            assert_eq!(input_schema["type"], "object");
            assert!(input_schema["required"].as_array().unwrap().contains(&json!("selector")));
            assert!(input_schema["required"].as_array().unwrap().contains(&json!("value")));
            
            let output_schema = tool.output_schema();
            assert_eq!(output_schema["type"], "object");
            assert!(output_schema["properties"]["selected"].is_object());
        }
    }
    
    // ============================================================================
    // INTEGRATION TESTS
    // ============================================================================
    
    mod integration_tests {
        use super::*;
        
        #[tokio::test]
        async fn test_form_filling_workflow() {
            let browser = Arc::new(MockBrowser::new());
            
            // Type username
            let type_tool = TypeText::new(browser.clone());
            let type_params = TypeTextParams {
                selector: "#username".to_string(),
                text: "john_doe".to_string(),
                options: Default::default(),
            };
            let result = type_tool.execute(type_params).await;
            assert!(result.is_ok());
            
            // Type password
            let type_params = TypeTextParams {
                selector: "#password".to_string(),
                text: "secure_password".to_string(),
                options: Default::default(),
            };
            let result = type_tool.execute(type_params).await;
            assert!(result.is_ok());
            
            // Select country
            let select_tool = SelectOption::new(browser.clone());
            let select_params = SelectOptionParams {
                selector: "#country".to_string(),
                value: SelectValue::Single("US".to_string()),
                options: Default::default(),
            };
            let result = select_tool.execute(select_params).await;
            assert!(result.is_ok());
            
            // Click submit
            let click_tool = Click::new(browser.clone());
            let click_params = ClickParams {
                selector: "#submit".to_string(),
                options: Default::default(),
            };
            let result = click_tool.execute(click_params).await;
            assert!(result.is_ok());
        }
        
        #[tokio::test]
        async fn test_search_workflow() {
            let browser = Arc::new(MockBrowser::new());
            
            // Click search box to focus
            let click_tool = Click::new(browser.clone());
            let click_params = ClickParams {
                selector: "#search-input".to_string(),
                options: Default::default(),
            };
            click_tool.execute(click_params).await.unwrap();
            
            // Type search query
            let type_tool = TypeText::new(browser.clone());
            let type_params = TypeTextParams {
                selector: "#search-input".to_string(),
                text: "rust programming".to_string(),
                options: TypeTextOptions {
                    press_enter: true,
                    ..Default::default()
                },
            };
            let result = type_tool.execute(type_params).await;
            assert!(result.is_ok());
            assert!(result.unwrap().events_triggered.contains(&"keydown:Enter".to_string()));
        }
        
        #[tokio::test]
        async fn test_multi_select_workflow() {
            let browser = Arc::new(MockBrowser::new());
            
            // Select multiple options with Ctrl+Click
            let select_tool = SelectOption::new(browser.clone());
            
            // First, deselect all
            let params = SelectOptionParams {
                selector: "#skills".to_string(),
                value: SelectValue::Multiple(vec![
                    "javascript".to_string(),
                    "rust".to_string(),
                    "python".to_string(),
                ]),
                options: SelectOptions {
                    deselect_others: true,
                    ..Default::default()
                },
            };
            
            let result = select_tool.execute(params).await;
            assert!(result.is_ok());
            
            let output = result.unwrap();
            assert_eq!(output.selected.len(), 3);
        }
    }
}