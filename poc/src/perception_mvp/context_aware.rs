// Context-Aware Element Selection - Maintains context and understands references
// This module handles "it", "that button", form state, and multi-step interactions

use anyhow::Result;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::time::{Instant, Duration};
use thirtyfour::{WebDriver, WebElement, By};
use crate::perception_mvp::{PerceivedElement, ElementType, PageType};
use crate::perception_mvp::semantic::{SemanticAnalyzer, SemanticElement, SemanticForm};

/// Context-aware element selector that maintains interaction history and state
pub struct ContextAwareSelector {
    driver: WebDriver,
    semantic_analyzer: SemanticAnalyzer,
    context: InteractionContext,
    element_memory: ElementMemory,
    form_tracker: FormTracker,
}

/// Maintains context across user interactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionContext {
    pub current_page: PageContext,
    pub interaction_history: Vec<InteractionEvent>,
    pub named_elements: HashMap<String, ElementReference>,
    pub last_focused_element: Option<ElementReference>,
    pub conversation_state: ConversationState,
}

/// Context about the current page
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageContext {
    pub url: String,
    pub title: String,
    pub page_type: PageType,
    pub timestamp: u64,
    pub dominant_elements: Vec<String>, // Most important elements on page
    pub available_actions: Vec<String>, // What user can do on this page
}

/// Record of user interactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionEvent {
    pub timestamp: u64,
    pub action: String,
    pub element: ElementReference,
    pub success: bool,
    pub context_before: String,
    pub context_after: String,
}

/// Reference to a specific element
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementReference {
    pub selector: String,
    pub text: String,
    pub element_type: ElementType,
    pub nickname: Option<String>, // User-given name like "the red button"
    pub confidence: f32,
    pub last_seen: u64,
}

/// Current conversation state for reference resolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationState {
    pub last_mentioned_elements: Vec<String>,
    pub active_form: Option<String>,
    pub pending_action: Option<String>,
    pub user_intent: UserIntent,
}

/// Understanding of what the user wants to accomplish
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserIntent {
    Navigation,       // Going somewhere
    FormFilling,      // Completing a form
    Search,           // Finding information
    Purchase,         // Buying something
    Reading,          // Consuming content
    Unknown,
}

/// Tracks element memory and learning
pub struct ElementMemory {
    elements: HashMap<String, MemorizedElement>,
    usage_patterns: HashMap<String, UsagePattern>,
    success_rates: HashMap<String, f32>,
}

/// Element with usage history
#[derive(Debug, Clone)]
struct MemorizedElement {
    reference: ElementReference,
    usage_count: u32,
    success_count: u32,
    last_used: Instant,
    alternative_selectors: Vec<String>,
    user_descriptions: Vec<String>,
}

/// Pattern of how user interacts with elements
#[derive(Debug, Clone)]
struct UsagePattern {
    element_type: ElementType,
    common_descriptions: Vec<String>,
    success_indicators: Vec<String>,
    failure_patterns: Vec<String>,
}

/// Tracks form state and completion
pub struct FormTracker {
    active_forms: HashMap<String, FormState>,
    form_templates: HashMap<String, FormTemplate>,
    auto_fill_data: HashMap<String, String>,
}

/// Current state of a form
#[derive(Debug, Clone)]
struct FormState {
    selector: String,
    fields: HashMap<String, FieldState>,
    completed_fields: Vec<String>,
    remaining_fields: Vec<String>,
    validation_errors: Vec<ValidationError>,
    last_interaction: Instant,
}

/// State of an individual form field
#[derive(Debug, Clone)]
struct FieldState {
    selector: String,
    value: Option<String>,
    validated: bool,
    error_message: Option<String>,
    user_focused: bool,
}

/// Template for common form types
#[derive(Debug, Clone)]
struct FormTemplate {
    form_type: String,
    expected_fields: Vec<String>,
    field_order: Vec<String>,
    validation_rules: HashMap<String, String>,
    completion_indicators: Vec<String>,
}

/// Validation error for forms
#[derive(Debug, Clone)]
struct ValidationError {
    field: String,
    message: String,
    error_type: String,
}

/// Result of context-aware element selection
#[derive(Debug, Clone)]
pub struct ContextualElement {
    pub element: PerceivedElement,
    pub context_score: f32,
    pub reference_type: ReferenceType,
    pub confidence_factors: Vec<String>,
    pub suggested_action: String,
}

/// Type of reference used to find the element
#[derive(Debug, Clone)]
pub enum ReferenceType {
    Direct,           // "the submit button"
    Pronoun,          // "it", "that"
    Positional,       // "the first", "next"
    Descriptive,      // "the red button", "large text field"
    Contextual,       // "the login button" (inferred from page context)
    Historical,       // "the button I clicked before"
}

impl ContextAwareSelector {
    pub fn new(driver: WebDriver) -> Self {
        let semantic_analyzer = SemanticAnalyzer::new(driver.clone());
        
        Self {
            driver,
            semantic_analyzer,
            context: InteractionContext::new(),
            element_memory: ElementMemory::new(),
            form_tracker: FormTracker::new(),
        }
    }

    /// Main entry point: Find element using natural language with context
    pub async fn find_element_with_context(&mut self, description: &str) -> Result<ContextualElement> {
        // Update page context if needed
        self.update_page_context().await?;
        
        // Analyze user intent from description
        let intent = self.analyze_user_intent(description);
        self.context.conversation_state.user_intent = intent;

        // Try different resolution strategies in priority order
        let strategies = vec![
            self.resolve_pronoun_reference(description).await,
            self.resolve_positional_reference(description).await,
            self.resolve_descriptive_reference(description).await,
            self.resolve_contextual_reference(description).await,
            self.resolve_semantic_reference(description).await,
        ];

        // Find the best match from all strategies
        let mut best_match = None;
        let mut best_score = 0.0f32;

        for strategy_result in strategies {
            if let Ok(element) = strategy_result {
                if element.context_score > best_score {
                    best_score = element.context_score;
                    best_match = Some(element);
                }
            }
        }

        match best_match {
            Some(element) => {
                // Record successful selection
                self.record_interaction(description, &element, true).await?;
                
                // Update element memory
                self.element_memory.record_usage(&element.element);
                
                Ok(element)
            }
            None => {
                // Record failure and suggest alternatives
                let suggestions = self.suggest_alternatives(description).await?;
                anyhow::bail!("Could not find element '{}'. Try: {}", description, suggestions.join(", "))
            }
        }
    }

    /// Resolve pronoun references like "it", "that", "this"
    async fn resolve_pronoun_reference(&self, description: &str) -> Result<ContextualElement> {
        let desc_lower = description.to_lowercase();
        
        if matches!(desc_lower.as_str(), "it" | "that" | "this" | "that button" | "this link") {
            if let Some(last_element) = &self.context.last_focused_element {
                // Try to find the element again
                if let Ok(web_element) = self.driver.find(By::Css(&last_element.selector)).await {
                    let perceived = self.web_element_to_perceived(web_element, last_element.element_type.clone()).await?;
                    
                    return Ok(ContextualElement {
                        element: perceived,
                        context_score: 0.95,
                        reference_type: ReferenceType::Pronoun,
                        confidence_factors: vec!["Recent interaction history".to_string()],
                        suggested_action: "Continue with last element".to_string(),
                    });
                }
            }
        }

        anyhow::bail!("No pronoun reference available")
    }

    /// Resolve positional references like "first", "next", "previous"
    async fn resolve_positional_reference(&self, description: &str) -> Result<ContextualElement> {
        let desc_lower = description.to_lowercase();
        
        if desc_lower.contains("first") {
            // Find first clickable element
            if let Ok(buttons) = self.driver.find_all(By::Css("button, a, input[type='button'], input[type='submit']")).await {
                if let Some(first_button) = buttons.first() {
                    let perceived = self.web_element_to_perceived(first_button.clone(), ElementType::Button).await?;
                    
                    return Ok(ContextualElement {
                        element: perceived,
                        context_score: 0.8,
                        reference_type: ReferenceType::Positional,
                        confidence_factors: vec!["Positional reference - first element".to_string()],
                        suggested_action: "Interact with first element".to_string(),
                    });
                }
            }
        }

        if desc_lower.contains("next") {
            // Find next element based on current context
            if let Some(current) = &self.context.last_focused_element {
                // This would need more complex logic to find the "next" element
                // For now, return error
            }
        }

        anyhow::bail!("No positional reference found")
    }

    /// Resolve descriptive references like "red button", "large text field"
    async fn resolve_descriptive_reference(&self, description: &str) -> Result<ContextualElement> {
        // Extract visual descriptors
        let visual_descriptors = self.extract_visual_descriptors(description);
        
        // Extract element type hints
        let element_hints = self.extract_element_hints(description);
        
        // Find candidate elements
        let mut candidates = Vec::new();
        
        for hint in &element_hints {
            match hint.as_str() {
                "button" => {
                    if let Ok(buttons) = self.driver.find_all(By::Css("button, input[type='button'], input[type='submit']")).await {
                        for button in buttons {
                            candidates.push((button, ElementType::Button));
                        }
                    }
                }
                "link" => {
                    if let Ok(links) = self.driver.find_all(By::Css("a[href]")).await {
                        for link in links {
                            candidates.push((link, ElementType::Link));
                        }
                    }
                }
                "field" | "input" => {
                    if let Ok(inputs) = self.driver.find_all(By::Css("input, textarea, select")).await {
                        for input in inputs {
                            candidates.push((input, ElementType::Input));
                        }
                    }
                }
                _ => {}
            }
        }

        // Score candidates based on visual descriptors
        for (candidate, elem_type) in candidates {
            let score = self.score_visual_match(&candidate, &visual_descriptors).await?;
            if score > 0.6 {
                let perceived = self.web_element_to_perceived(candidate, elem_type).await?;
                
                return Ok(ContextualElement {
                    element: perceived,
                    context_score: score,
                    reference_type: ReferenceType::Descriptive,
                    confidence_factors: vec![format!("Visual descriptors: {:?}", visual_descriptors)],
                    suggested_action: "Interact with visually described element".to_string(),
                });
            }
        }

        anyhow::bail!("No descriptive match found")
    }

    /// Resolve contextual references using page understanding
    async fn resolve_contextual_reference(&self, description: &str) -> Result<ContextualElement> {
        // Use page context to understand what user might want
        match self.context.current_page.page_type {
            PageType::LoginPage => {
                if description.to_lowercase().contains("login") || description.to_lowercase().contains("sign in") {
                    // Look for login button
                    let selectors = vec!["button[type='submit']", "input[type='submit']", "button:contains('Login')", "a:contains('Sign in')"];
                    
                    for selector in selectors {
                        if let Ok(element) = self.driver.find(By::Css(selector)).await {
                            let perceived = self.web_element_to_perceived(element, ElementType::Button).await?;
                            
                            return Ok(ContextualElement {
                                element: perceived,
                                context_score: 0.9,
                                reference_type: ReferenceType::Contextual,
                                confidence_factors: vec!["Login page context".to_string()],
                                suggested_action: "Submit login form".to_string(),
                            });
                        }
                    }
                }
            }
            PageType::SearchResults => {
                if description.to_lowercase().contains("search") {
                    // Look for search box
                    let selectors = vec!["input[type='search']", "input[name*='search']", "#search", ".search-input"];
                    
                    for selector in selectors {
                        if let Ok(element) = self.driver.find(By::Css(selector)).await {
                            let perceived = self.web_element_to_perceived(element, ElementType::Input).await?;
                            
                            return Ok(ContextualElement {
                                element: perceived,
                                context_score: 0.85,
                                reference_type: ReferenceType::Contextual,
                                confidence_factors: vec!["Search page context".to_string()],
                                suggested_action: "Perform search".to_string(),
                            });
                        }
                    }
                }
            }
            _ => {}
        }

        anyhow::bail!("No contextual match found")
    }

    /// Resolve references using semantic analysis
    async fn resolve_semantic_reference(&mut self, description: &str) -> Result<ContextualElement> {
        // Use semantic analyzer to understand the page
        let semantic_analysis = self.semantic_analyzer.analyze_page().await?;
        
        // Find elements that match the description semantically
        for semantic_element in semantic_analysis.elements {
            let similarity_score = self.calculate_semantic_similarity(description, &semantic_element);
            
            if similarity_score > 0.7 {
                // Convert semantic element to perceived element
                if let Ok(web_element) = self.driver.find(By::Css(&semantic_element.selector)).await {
                    let element_type = self.semantic_role_to_element_type(&semantic_element.role);
                    let perceived = self.web_element_to_perceived(web_element, element_type).await?;
                    
                    return Ok(ContextualElement {
                        element: perceived,
                        context_score: similarity_score,
                        reference_type: ReferenceType::Contextual,
                        confidence_factors: vec![format!("Semantic role: {:?}", semantic_element.role)],
                        suggested_action: semantic_element.purpose,
                    });
                }
            }
        }

        anyhow::bail!("No semantic match found")
    }

    /// Update the current page context
    async fn update_page_context(&mut self) -> Result<()> {
        let url = self.driver.current_url().await?;
        let title = self.driver.title().await?;
        
        // Only update if URL changed
        if url != self.context.current_page.url {
            // Analyze the new page
            let page_type = self.classify_page_type(&url, &title).await?;
            let dominant_elements = self.find_dominant_elements().await?;
            let available_actions = self.determine_available_actions(&page_type).await?;
            
            self.context.current_page = PageContext {
                url,
                title,
                page_type,
                timestamp: self.current_timestamp(),
                dominant_elements,
                available_actions,
            };
            
            // Clear element-specific context when page changes
            self.context.last_focused_element = None;
            self.context.conversation_state.active_form = None;
        }

        Ok(())
    }

    /// Analyze what the user intends to do
    fn analyze_user_intent(&self, description: &str) -> UserIntent {
        let desc_lower = description.to_lowercase();
        
        if desc_lower.contains("go to") || desc_lower.contains("navigate") || desc_lower.contains("visit") {
            UserIntent::Navigation
        } else if desc_lower.contains("fill") || desc_lower.contains("type") || desc_lower.contains("enter") {
            UserIntent::FormFilling
        } else if desc_lower.contains("search") || desc_lower.contains("find") {
            UserIntent::Search
        } else if desc_lower.contains("buy") || desc_lower.contains("purchase") || desc_lower.contains("order") {
            UserIntent::Purchase
        } else if desc_lower.contains("read") || desc_lower.contains("view") || desc_lower.contains("show") {
            UserIntent::Reading
        } else {
            UserIntent::Unknown
        }
    }

    /// Record an interaction for context building
    async fn record_interaction(&mut self, description: &str, element: &ContextualElement, success: bool) -> Result<()> {
        let event = InteractionEvent {
            timestamp: self.current_timestamp(),
            action: description.to_string(),
            element: ElementReference {
                selector: element.element.selector.clone(),
                text: element.element.text.clone(),
                element_type: element.element.element_type.clone(),
                nickname: None,
                confidence: element.element.confidence,
                last_seen: self.current_timestamp(),
            },
            success,
            context_before: self.context.current_page.url.clone(),
            context_after: self.driver.current_url().await?,
        };

        self.context.interaction_history.push(event);
        self.context.last_focused_element = Some(ElementReference {
            selector: element.element.selector.clone(),
            text: element.element.text.clone(),
            element_type: element.element.element_type.clone(),
            nickname: None,
            confidence: element.element.confidence,
            last_seen: self.current_timestamp(),
        });

        // Keep history manageable
        if self.context.interaction_history.len() > 50 {
            self.context.interaction_history.drain(0..10);
        }

        Ok(())
    }

    /// Suggest alternatives when element not found
    async fn suggest_alternatives(&self, description: &str) -> Result<Vec<String>> {
        let mut suggestions = Vec::new();

        // Suggest based on page context
        match self.context.current_page.page_type {
            PageType::LoginPage => {
                suggestions.extend(vec![
                    "the username field".to_string(),
                    "the password field".to_string(),
                    "the login button".to_string(),
                    "the sign in link".to_string(),
                ]);
            }
            PageType::SearchResults => {
                suggestions.extend(vec![
                    "the search box".to_string(),
                    "the first result".to_string(),
                    "the next page button".to_string(),
                ]);
            }
            _ => {
                suggestions.extend(vec![
                    "the submit button".to_string(),
                    "the main link".to_string(),
                    "the search box".to_string(),
                ]);
            }
        }

        // Add elements currently visible on page
        if let Ok(buttons) = self.driver.find_all(By::Css("button")).await {
            for (i, button) in buttons.iter().take(3).enumerate() {
                if let Ok(text) = button.text().await {
                    if !text.trim().is_empty() {
                        suggestions.push(format!("\"{}\"", text.trim()));
                    }
                }
            }
        }

        Ok(suggestions)
    }

    /// Extract visual descriptors from description
    fn extract_visual_descriptors(&self, description: &str) -> Vec<String> {
        let mut descriptors = Vec::new();
        let desc_lower = description.to_lowercase();

        // Colors
        let colors = vec!["red", "blue", "green", "yellow", "orange", "purple", "pink", "black", "white", "gray"];
        for color in colors {
            if desc_lower.contains(color) {
                descriptors.push(color.to_string());
            }
        }

        // Sizes
        let sizes = vec!["large", "big", "small", "tiny", "huge"];
        for size in sizes {
            if desc_lower.contains(size) {
                descriptors.push(size.to_string());
            }
        }

        // Positions
        let positions = vec!["top", "bottom", "left", "right", "center", "middle"];
        for position in positions {
            if desc_lower.contains(position) {
                descriptors.push(position.to_string());
            }
        }

        descriptors
    }

    /// Extract element type hints from description
    fn extract_element_hints(&self, description: &str) -> Vec<String> {
        let mut hints = Vec::new();
        let desc_lower = description.to_lowercase();

        let element_words = vec![
            "button", "link", "field", "input", "box", "dropdown", "menu", 
            "form", "text", "image", "video", "table", "list"
        ];

        for word in element_words {
            if desc_lower.contains(word) {
                hints.push(word.to_string());
            }
        }

        hints
    }

    /// Score how well an element matches visual descriptors
    async fn score_visual_match(&self, element: &WebElement, descriptors: &[String]) -> Result<f32> {
        // This is simplified - real implementation would analyze CSS styles, colors, sizes
        let mut score = 0.0f32;
        
        // Check element text for descriptor matches
        if let Ok(text) = element.text().await {
            for descriptor in descriptors {
                if text.to_lowercase().contains(descriptor) {
                    score += 0.3;
                }
            }
        }

        // Check class names for descriptor matches
        if let Ok(Some(class)) = element.attr("class").await {
            for descriptor in descriptors {
                if class.to_lowercase().contains(descriptor) {
                    score += 0.4;
                }
            }
        }

        Ok(score.min(1.0))
    }

    /// Calculate semantic similarity between description and semantic element
    fn calculate_semantic_similarity(&self, description: &str, semantic_element: &SemanticElement) -> f32 {
        let desc_lower = description.to_lowercase();
        let element_text = semantic_element.text.to_lowercase();
        let purpose_text = semantic_element.purpose.to_lowercase();

        let mut score = 0.0f32;

        // Direct text match
        if element_text.contains(&desc_lower) || desc_lower.contains(&element_text) {
            score += 0.5;
        }

        // Purpose match
        if purpose_text.contains(&desc_lower) || desc_lower.contains(&purpose_text) {
            score += 0.3;
        }

        // Word overlap
        let desc_words: Vec<&str> = desc_lower.split_whitespace().collect();
        let element_words: Vec<&str> = element_text.split_whitespace().collect();
        
        let matching_words = desc_words.iter()
            .filter(|word| element_words.contains(word))
            .count();

        if desc_words.len() > 0 {
            score += (matching_words as f32 / desc_words.len() as f32) * 0.2;
        }

        score.min(1.0)
    }

    /// Convert semantic role to element type
    fn semantic_role_to_element_type(&self, role: &crate::perception_mvp::semantic::ElementRole) -> ElementType {
        match role {
            crate::perception_mvp::semantic::ElementRole::PrimaryAction |
            crate::perception_mvp::semantic::ElementRole::SecondaryAction |
            crate::perception_mvp::semantic::ElementRole::FormSubmit => ElementType::Button,
            crate::perception_mvp::semantic::ElementRole::NavigationLink => ElementType::Link,
            crate::perception_mvp::semantic::ElementRole::FormInput |
            crate::perception_mvp::semantic::ElementRole::Search => ElementType::Input,
            _ => ElementType::Unknown,
        }
    }

    /// Helper methods
    async fn classify_page_type(&self, url: &str, title: &str) -> Result<PageType> {
        if url.contains("login") || title.to_lowercase().contains("login") {
            Ok(PageType::LoginPage)
        } else if url.contains("search") || title.to_lowercase().contains("search") {
            Ok(PageType::SearchResults)
        } else {
            Ok(PageType::Unknown)
        }
    }

    async fn find_dominant_elements(&self) -> Result<Vec<String>> {
        let mut elements = Vec::new();
        
        // Find the most prominent buttons
        if let Ok(buttons) = self.driver.find_all(By::Css("button")).await {
            for button in buttons.iter().take(3) {
                if let Ok(text) = button.text().await {
                    if !text.trim().is_empty() {
                        elements.push(text.trim().to_string());
                    }
                }
            }
        }

        Ok(elements)
    }

    async fn determine_available_actions(&self, page_type: &PageType) -> Result<Vec<String>> {
        match page_type {
            PageType::LoginPage => Ok(vec![
                "type username".to_string(),
                "type password".to_string(),
                "click login button".to_string(),
            ]),
            PageType::SearchResults => Ok(vec![
                "click on result".to_string(),
                "search again".to_string(),
                "go to next page".to_string(),
            ]),
            _ => Ok(vec![
                "click button".to_string(),
                "fill form".to_string(),
                "navigate".to_string(),
            ]),
        }
    }

    fn current_timestamp(&self) -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    /// Convert WebElement to PerceivedElement
    async fn web_element_to_perceived(&self, element: WebElement, element_type: ElementType) -> Result<PerceivedElement> {
        let text = element.text().await.unwrap_or_default();
        let is_displayed = element.is_displayed().await.unwrap_or(false);
        let is_enabled = element.is_enabled().await.unwrap_or(false);
        
        // Generate selector
        let id = element.attr("id").await?.unwrap_or_default();
        let selector = if !id.is_empty() {
            format!("#{}", id)
        } else {
            format!("{}:nth-of-type(1)", element.tag_name().await?)
        };

        // Collect attributes
        let mut attributes = HashMap::new();
        for attr in ["class", "name", "type", "placeholder", "aria-label"] {
            if let Ok(Some(value)) = element.attr(attr).await {
                attributes.insert(attr.to_string(), value);
            }
        }

        Ok(PerceivedElement {
            selector,
            text,
            element_type,
            clickable: is_enabled && is_displayed,
            visible: is_displayed,
            confidence: 0.8,
            attributes,
        })
    }
}

impl InteractionContext {
    fn new() -> Self {
        Self {
            current_page: PageContext {
                url: String::new(),
                title: String::new(),
                page_type: PageType::Unknown,
                timestamp: 0,
                dominant_elements: Vec::new(),
                available_actions: Vec::new(),
            },
            interaction_history: Vec::new(),
            named_elements: HashMap::new(),
            last_focused_element: None,
            conversation_state: ConversationState {
                last_mentioned_elements: Vec::new(),
                active_form: None,
                pending_action: None,
                user_intent: UserIntent::Unknown,
            },
        }
    }
}

impl ElementMemory {
    fn new() -> Self {
        Self {
            elements: HashMap::new(),
            usage_patterns: HashMap::new(),
            success_rates: HashMap::new(),
        }
    }

    fn record_usage(&mut self, element: &PerceivedElement) {
        let key = element.selector.clone();
        
        let memorized = self.elements.entry(key.clone()).or_insert_with(|| MemorizedElement {
            reference: ElementReference {
                selector: element.selector.clone(),
                text: element.text.clone(),
                element_type: element.element_type.clone(),
                nickname: None,
                confidence: element.confidence,
                last_seen: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
            },
            usage_count: 0,
            success_count: 0,
            last_used: Instant::now(),
            alternative_selectors: Vec::new(),
            user_descriptions: Vec::new(),
        });

        memorized.usage_count += 1;
        memorized.success_count += 1;
        memorized.last_used = Instant::now();
    }
}

impl FormTracker {
    fn new() -> Self {
        Self {
            active_forms: HashMap::new(),
            form_templates: HashMap::new(),
            auto_fill_data: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_visual_descriptors() {
        let selector = ContextAwareSelector::new(
            // This would need a mock WebDriver for testing
            // WebDriver::mock()
        );
        
        let descriptors = selector.extract_visual_descriptors("click the red button");
        assert!(descriptors.contains(&"red".to_string()));
        
        let descriptors = selector.extract_visual_descriptors("type in the large text field");
        assert!(descriptors.contains(&"large".to_string()));
    }

    #[test]
    fn test_extract_element_hints() {
        let selector = ContextAwareSelector::new(
            // Mock WebDriver needed
        );
        
        let hints = selector.extract_element_hints("click the submit button");
        assert!(hints.contains(&"button".to_string()));
        
        let hints = selector.extract_element_hints("fill the email field");
        assert!(hints.contains(&"field".to_string()));
    }
}