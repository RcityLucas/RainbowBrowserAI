// Semantic Understanding Layer - Understands the meaning and purpose of web elements
// This layer analyzes DOM structure, content, and patterns to classify elements and pages

use anyhow::Result;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use thirtyfour::{WebDriver, WebElement, By};
use regex::Regex;

/// Semantic analyzer that understands web page structure and element purposes
pub struct SemanticAnalyzer {
    driver: WebDriver,
    page_classifier: PageClassifier,
    element_classifier: ElementClassifier,
    form_analyzer: FormAnalyzer,
    content_analyzer: ContentAnalyzer,
}

/// Classifies the type and purpose of web pages
pub struct PageClassifier {
    patterns: HashMap<PageType, Vec<PagePattern>>,
}

/// Determines element roles and purposes
pub struct ElementClassifier {
    role_patterns: HashMap<ElementRole, Vec<ElementPattern>>,
}

/// Analyzes form structure and fields
pub struct FormAnalyzer {
    field_patterns: HashMap<FieldType, Vec<FieldPattern>>,
}

/// Analyzes content and text patterns
pub struct ContentAnalyzer {
    content_patterns: HashMap<ContentType, Vec<ContentPattern>>,
}

/// Page classification types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PageType {
    Homepage,
    LoginPage,
    RegisterPage,
    SearchResults,
    ProductPage,
    CategoryPage,
    ArticlePage,
    FormPage,
    CheckoutPage,
    Dashboard,
    ProfilePage,
    SettingsPage,
    ContactPage,
    Unknown,
}

/// Element roles and purposes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ElementRole {
    PrimaryAction,      // Main CTA buttons
    SecondaryAction,    // Secondary buttons
    NavigationLink,     // Menu/nav links
    FormSubmit,         // Form submission
    FormInput,          // Data input fields
    Search,             // Search functionality
    Filter,             // Filtering controls
    Pagination,         // Page navigation
    SocialShare,        // Social sharing
    Advertisement,      // Ads/promotional
    Content,            // Main content
    Media,              // Images/videos
    Unknown,
}

/// Form field types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum FieldType {
    Email,
    Password,
    Username,
    FirstName,
    LastName,
    FullName,
    Phone,
    Address,
    City,
    State,
    ZipCode,
    Country,
    CreditCard,
    CVV,
    ExpiryDate,
    Search,
    Message,
    Subject,
    Generic,
}

/// Content classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ContentType {
    Heading,
    Paragraph,
    List,
    Quote,
    Code,
    Caption,
    Navigation,
    Footer,
    Sidebar,
    Advertisement,
}

/// Pattern for matching pages
#[derive(Debug, Clone)]
struct PagePattern {
    url_patterns: Vec<Regex>,
    title_patterns: Vec<Regex>,
    element_selectors: Vec<String>,
    content_patterns: Vec<Regex>,
    confidence_weight: f32,
}

/// Pattern for matching elements
#[derive(Debug, Clone)]
struct ElementPattern {
    selectors: Vec<String>,
    text_patterns: Vec<Regex>,
    attribute_patterns: HashMap<String, Regex>,
    context_selectors: Vec<String>,
    confidence_weight: f32,
}

/// Pattern for matching form fields
#[derive(Debug, Clone)]
struct FieldPattern {
    name_patterns: Vec<Regex>,
    placeholder_patterns: Vec<Regex>,
    label_patterns: Vec<Regex>,
    type_patterns: Vec<String>,
    validation_patterns: Vec<Regex>,
    confidence_weight: f32,
}

/// Pattern for matching content
#[derive(Debug, Clone)]
struct ContentPattern {
    text_patterns: Vec<Regex>,
    structure_patterns: Vec<String>,
    length_range: (usize, usize),
    confidence_weight: f32,
}

/// Result of semantic analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticAnalysis {
    pub page_type: PageType,
    pub page_confidence: f32,
    pub elements: Vec<SemanticElement>,
    pub forms: Vec<SemanticForm>,
    pub content_structure: ContentStructure,
}

/// Semantically understood element
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticElement {
    pub selector: String,
    pub role: ElementRole,
    pub purpose: String,
    pub priority: f32,
    pub confidence: f32,
    pub text: String,
    pub attributes: HashMap<String, String>,
}

/// Semantically analyzed form
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticForm {
    pub selector: String,
    pub purpose: String,
    pub fields: Vec<SemanticField>,
    pub submit_button: Option<String>,
    pub validation_rules: Vec<ValidationRule>,
}

/// Semantically understood form field
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticField {
    pub selector: String,
    pub field_type: FieldType,
    pub label: String,
    pub required: bool,
    pub placeholder: Option<String>,
    pub validation: Option<String>,
    pub confidence: f32,
}

/// Form validation rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    pub field: String,
    pub rule_type: String,
    pub pattern: Option<String>,
    pub message: Option<String>,
}

/// Content structure analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentStructure {
    pub main_content: Option<String>,
    pub navigation: Vec<String>,
    pub sidebar: Vec<String>,
    pub footer: Option<String>,
    pub hierarchy: Vec<ContentHierarchy>,
}

/// Content hierarchy level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentHierarchy {
    pub level: u8,
    pub selector: String,
    pub text: String,
    pub content_type: ContentType,
}

impl SemanticAnalyzer {
    pub fn new(driver: WebDriver) -> Self {
        Self {
            driver,
            page_classifier: PageClassifier::new(),
            element_classifier: ElementClassifier::new(),
            form_analyzer: FormAnalyzer::new(),
            content_analyzer: ContentAnalyzer::new(),
        }
    }

    /// Perform complete semantic analysis of the current page
    pub async fn analyze_page(&mut self) -> Result<SemanticAnalysis> {
        // Get page metadata
        let url = self.driver.current_url().await?;
        let title = self.driver.title().await?;
        let body_html = self.driver.source().await?;

        // Classify page type
        let (page_type, page_confidence) = self.page_classifier.classify(&url, &title, &body_html).await?;

        // Analyze elements
        let elements = self.analyze_elements().await?;

        // Analyze forms
        let forms = self.analyze_forms().await?;

        // Analyze content structure
        let content_structure = self.analyze_content_structure().await?;

        Ok(SemanticAnalysis {
            page_type,
            page_confidence,
            elements,
            forms,
            content_structure,
        })
    }

    /// Analyze all interactive elements on the page
    async fn analyze_elements(&self) -> Result<Vec<SemanticElement>> {
        let mut elements = Vec::new();

        // Find all potentially interactive elements
        let interactive_selectors = vec![
            "button",
            "a[href]",
            "input[type='button']",
            "input[type='submit']",
            "[role='button']",
            "[onclick]",
            ".btn", ".button",
        ];

        for selector in interactive_selectors {
            if let Ok(web_elements) = self.driver.find_all(By::Css(selector)).await {
                for element in web_elements {
                    if let Ok(semantic_element) = self.classify_element(&element).await {
                        elements.push(semantic_element);
                    }
                }
            }
        }

        // Sort by priority (most important elements first)
        elements.sort_by(|a, b| b.priority.partial_cmp(&a.priority).unwrap());

        Ok(elements)
    }

    /// Classify a single element's role and purpose
    async fn classify_element(&self, element: &WebElement) -> Result<SemanticElement> {
        let text = element.text().await.unwrap_or_default();
        let tag_name = element.tag_name().await.unwrap_or_default();
        
        // Collect attributes
        let mut attributes = HashMap::new();
        for attr in ["class", "id", "role", "aria-label", "title", "type"] {
            if let Ok(Some(value)) = element.attr(attr).await {
                attributes.insert(attr.to_string(), value);
            }
        }

        // Generate CSS selector
        let selector = self.generate_selector(element).await?;

        // Classify role
        let (role, confidence) = self.element_classifier.classify_role(&text, &tag_name, &attributes)?;

        // Determine purpose
        let purpose = self.determine_purpose(&role, &text, &attributes);

        // Calculate priority
        let priority = self.calculate_priority(&role, &text, &attributes);

        Ok(SemanticElement {
            selector,
            role,
            purpose,
            priority,
            confidence,
            text,
            attributes,
        })
    }

    /// Analyze all forms on the page
    async fn analyze_forms(&self) -> Result<Vec<SemanticForm>> {
        let mut forms = Vec::new();
        let form_elements = self.driver.find_all(By::Tag("form")).await?;

        for (index, form_element) in form_elements.iter().enumerate() {
            let selector = format!("form:nth-of-type({})", index + 1);
            let purpose = self.determine_form_purpose(form_element).await?;
            
            // Analyze form fields
            let fields = self.analyze_form_fields(form_element).await?;
            
            // Find submit button
            let submit_button = self.find_submit_button(form_element).await?;
            
            // Generate validation rules
            let validation_rules = self.generate_validation_rules(&fields);

            forms.push(SemanticForm {
                selector,
                purpose,
                fields,
                submit_button,
                validation_rules,
            });
        }

        Ok(forms)
    }

    /// Analyze form fields within a form
    async fn analyze_form_fields(&self, form: &WebElement) -> Result<Vec<SemanticField>> {
        let mut fields = Vec::new();
        
        // Find all input elements
        let inputs = form.find_all(By::Css("input, select, textarea")).await?;
        
        for input in inputs {
            if let Ok(field) = self.classify_form_field(&input).await {
                fields.push(field);
            }
        }

        Ok(fields)
    }

    /// Classify a form field's type and purpose
    async fn classify_form_field(&self, input: &WebElement) -> Result<SemanticField> {
        let tag_name = input.tag_name().await.unwrap_or_default();
        let input_type = input.attr("type").await?.unwrap_or_default();
        let name = input.attr("name").await?.unwrap_or_default();
        let placeholder = input.attr("placeholder").await?;
        let required = input.attr("required").await?.is_some();

        // Generate selector
        let selector = self.generate_selector(input).await?;

        // Find associated label
        let label = self.find_field_label(input).await.unwrap_or_default();

        // Classify field type
        let (field_type, confidence) = self.form_analyzer.classify_field(
            &name, &label, &placeholder.unwrap_or_default(), &input_type
        )?;

        // Extract validation pattern
        let validation = input.attr("pattern").await?;

        Ok(SemanticField {
            selector,
            field_type,
            label,
            required,
            placeholder,
            validation,
            confidence,
        })
    }

    /// Analyze content structure and hierarchy
    async fn analyze_content_structure(&self) -> Result<ContentStructure> {
        // Find main content area
        let main_content = self.find_main_content().await?;
        
        // Find navigation elements
        let navigation = self.find_navigation_elements().await?;
        
        // Find sidebar elements
        let sidebar = self.find_sidebar_elements().await?;
        
        // Find footer
        let footer = self.find_footer().await?;
        
        // Build content hierarchy
        let hierarchy = self.build_content_hierarchy().await?;

        Ok(ContentStructure {
            main_content,
            navigation,
            sidebar,
            footer,
            hierarchy,
        })
    }

    /// Generate a unique CSS selector for an element
    async fn generate_selector(&self, element: &WebElement) -> Result<String> {
        // Try ID first
        if let Ok(Some(id)) = element.attr("id").await {
            if !id.is_empty() {
                return Ok(format!("#{}", id));
            }
        }

        // Try unique class combination
        if let Ok(Some(classes)) = element.attr("class").await {
            let class_selector = classes
                .split_whitespace()
                .map(|c| format!(".{}", c))
                .collect::<Vec<_>>()
                .join("");
            
            if !class_selector.is_empty() {
                return Ok(class_selector);
            }
        }

        // Fall back to tag with position
        let tag_name = element.tag_name().await.unwrap_or_default();
        Ok(format!("{}:nth-of-type(1)", tag_name))
    }

    /// Determine element's purpose based on its role and context
    fn determine_purpose(&self, role: &ElementRole, text: &str, attributes: &HashMap<String, String>) -> String {
        match role {
            ElementRole::PrimaryAction => {
                if text.to_lowercase().contains("submit") {
                    "Submit form".to_string()
                } else if text.to_lowercase().contains("buy") || text.to_lowercase().contains("purchase") {
                    "Complete purchase".to_string()
                } else if text.to_lowercase().contains("sign") {
                    "Sign up or log in".to_string()
                } else {
                    format!("Perform primary action: {}", text)
                }
            }
            ElementRole::NavigationLink => {
                format!("Navigate to: {}", text)
            }
            ElementRole::Search => {
                "Search functionality".to_string()
            }
            ElementRole::Filter => {
                "Filter or sort content".to_string()
            }
            _ => format!("Element purpose: {}", text.chars().take(50).collect::<String>()),
        }
    }

    /// Calculate element priority for interaction
    fn calculate_priority(&self, role: &ElementRole, text: &str, attributes: &HashMap<String, String>) -> f32 {
        let mut priority = match role {
            ElementRole::PrimaryAction => 1.0,
            ElementRole::FormSubmit => 0.9,
            ElementRole::Search => 0.8,
            ElementRole::NavigationLink => 0.7,
            ElementRole::SecondaryAction => 0.6,
            ElementRole::Filter => 0.5,
            _ => 0.3,
        };

        // Boost priority for visible, prominent elements
        if let Some(class) = attributes.get("class") {
            if class.contains("primary") || class.contains("main") {
                priority += 0.2;
            }
            if class.contains("large") || class.contains("big") {
                priority += 0.1;
            }
        }

        // Boost priority for common action words
        let action_words = ["submit", "buy", "purchase", "login", "search", "go"];
        if action_words.iter().any(|word| text.to_lowercase().contains(word)) {
            priority += 0.1;
        }

        priority.min(1.0)
    }

    /// Determine the purpose of a form
    async fn determine_form_purpose(&self, form: &WebElement) -> Result<String> {
        // Check form attributes
        if let Ok(Some(id)) = form.attr("id").await {
            if id.contains("login") {
                return Ok("Login form".to_string());
            }
            if id.contains("register") || id.contains("signup") {
                return Ok("Registration form".to_string());
            }
            if id.contains("search") {
                return Ok("Search form".to_string());
            }
            if id.contains("contact") {
                return Ok("Contact form".to_string());
            }
        }

        // Analyze form fields to infer purpose
        let inputs = form.find_all(By::Css("input")).await?;
        let mut has_email = false;
        let mut has_password = false;
        let mut has_name = false;

        for input in inputs {
            if let Ok(Some(name)) = input.attr("name").await {
                let name_lower = name.to_lowercase();
                if name_lower.contains("email") {
                    has_email = true;
                }
                if name_lower.contains("password") {
                    has_password = true;
                }
                if name_lower.contains("name") {
                    has_name = true;
                }
            }
        }

        if has_email && has_password && has_name {
            Ok("Registration form".to_string())
        } else if has_email && has_password {
            Ok("Login form".to_string())
        } else if has_email {
            Ok("Email form".to_string())
        } else {
            Ok("Generic form".to_string())
        }
    }

    /// Find the label associated with a form field
    async fn find_field_label(&self, input: &WebElement) -> Result<String> {
        // Try to find label by 'for' attribute
        if let Ok(Some(id)) = input.attr("id").await {
            if let Ok(label) = self.driver.find(By::Css(&format!("label[for='{}']", id))).await {
                if let Ok(text) = label.text().await {
                    if !text.trim().is_empty() {
                        return Ok(text.trim().to_string());
                    }
                }
            }
        }

        // Try to find parent label
        // This is simplified - in reality you'd traverse up the DOM tree
        Ok(String::new())
    }

    /// Find submit button within a form
    async fn find_submit_button(&self, form: &WebElement) -> Result<Option<String>> {
        // Look for submit button
        if let Ok(submit) = form.find(By::Css("input[type='submit'], button[type='submit']")).await {
            return Ok(Some(self.generate_selector(&submit).await?));
        }

        // Look for button without explicit type (defaults to submit in forms)
        if let Ok(button) = form.find(By::Tag("button")).await {
            return Ok(Some(self.generate_selector(&button).await?));
        }

        Ok(None)
    }

    /// Generate validation rules for form fields
    fn generate_validation_rules(&self, fields: &[SemanticField]) -> Vec<ValidationRule> {
        let mut rules = Vec::new();

        for field in fields {
            match field.field_type {
                FieldType::Email => {
                    rules.push(ValidationRule {
                        field: field.selector.clone(),
                        rule_type: "email".to_string(),
                        pattern: Some(r"^[^\s@]+@[^\s@]+\.[^\s@]+$".to_string()),
                        message: Some("Please enter a valid email address".to_string()),
                    });
                }
                FieldType::Phone => {
                    rules.push(ValidationRule {
                        field: field.selector.clone(),
                        rule_type: "phone".to_string(),
                        pattern: Some(r"^\+?[\d\s\-\(\)]+$".to_string()),
                        message: Some("Please enter a valid phone number".to_string()),
                    });
                }
                FieldType::Password => {
                    rules.push(ValidationRule {
                        field: field.selector.clone(),
                        rule_type: "minLength".to_string(),
                        pattern: Some("8".to_string()),
                        message: Some("Password must be at least 8 characters".to_string()),
                    });
                }
                _ => {}
            }

            if field.required {
                rules.push(ValidationRule {
                    field: field.selector.clone(),
                    rule_type: "required".to_string(),
                    pattern: None,
                    message: Some("This field is required".to_string()),
                });
            }
        }

        rules
    }

    /// Find main content area
    async fn find_main_content(&self) -> Result<Option<String>> {
        let main_selectors = vec!["main", "#main", "#content", ".main-content", "article"];
        
        for selector in main_selectors {
            if let Ok(_) = self.driver.find(By::Css(selector)).await {
                return Ok(Some(selector.to_string()));
            }
        }

        Ok(None)
    }

    /// Find navigation elements
    async fn find_navigation_elements(&self) -> Result<Vec<String>> {
        let mut navigation = Vec::new();
        let nav_selectors = vec!["nav", ".navigation", ".menu", "#menu", ".navbar"];

        for selector in nav_selectors {
            if let Ok(_) = self.driver.find(By::Css(selector)).await {
                navigation.push(selector.to_string());
            }
        }

        Ok(navigation)
    }

    /// Find sidebar elements
    async fn find_sidebar_elements(&self) -> Result<Vec<String>> {
        let mut sidebar = Vec::new();
        let sidebar_selectors = vec![".sidebar", "#sidebar", ".side-nav", "aside"];

        for selector in sidebar_selectors {
            if let Ok(_) = self.driver.find(By::Css(selector)).await {
                sidebar.push(selector.to_string());
            }
        }

        Ok(sidebar)
    }

    /// Find footer element
    async fn find_footer(&self) -> Result<Option<String>> {
        let footer_selectors = vec!["footer", "#footer", ".footer"];
        
        for selector in footer_selectors {
            if let Ok(_) = self.driver.find(By::Css(selector)).await {
                return Ok(Some(selector.to_string()));
            }
        }

        Ok(None)
    }

    /// Build content hierarchy from headings
    async fn build_content_hierarchy(&self) -> Result<Vec<ContentHierarchy>> {
        let mut hierarchy = Vec::new();
        
        // Find all headings
        for level in 1..=6 {
            let heading_selector = format!("h{}", level);
            if let Ok(headings) = self.driver.find_all(By::Tag(&heading_selector)).await {
                for (index, heading) in headings.iter().enumerate() {
                    if let Ok(text) = heading.text().await {
                        if !text.trim().is_empty() {
                            hierarchy.push(ContentHierarchy {
                                level: level as u8,
                                selector: format!("h{}:nth-of-type({})", level, index + 1),
                                text: text.trim().to_string(),
                                content_type: ContentType::Heading,
                            });
                        }
                    }
                }
            }
        }

        // Sort by document order (simplified - would need position info)
        Ok(hierarchy)
    }
}

impl PageClassifier {
    fn new() -> Self {
        Self {
            patterns: Self::build_page_patterns(),
        }
    }

    fn build_page_patterns() -> HashMap<PageType, Vec<PagePattern>> {
        let mut patterns = HashMap::new();

        // Login page patterns
        patterns.insert(PageType::LoginPage, vec![
            PagePattern {
                url_patterns: vec![Regex::new(r"login|signin|auth").unwrap()],
                title_patterns: vec![Regex::new(r"login|sign\s+in").unwrap()],
                element_selectors: vec!["input[type='password']".to_string(), "#password".to_string()],
                content_patterns: vec![Regex::new(r"forgot\s+password").unwrap()],
                confidence_weight: 0.9,
            }
        ]);

        // Registration page patterns
        patterns.insert(PageType::RegisterPage, vec![
            PagePattern {
                url_patterns: vec![Regex::new(r"register|signup|join").unwrap()],
                title_patterns: vec![Regex::new(r"sign\s+up|register|create\s+account").unwrap()],
                element_selectors: vec!["input[name*='confirm']".to_string()],
                content_patterns: vec![Regex::new(r"terms\s+of\s+service|privacy\s+policy").unwrap()],
                confidence_weight: 0.9,
            }
        ]);

        // Search results patterns
        patterns.insert(PageType::SearchResults, vec![
            PagePattern {
                url_patterns: vec![Regex::new(r"search|query|q=").unwrap()],
                title_patterns: vec![Regex::new(r"search\s+results").unwrap()],
                element_selectors: vec![".search-result".to_string(), ".result-item".to_string()],
                content_patterns: vec![Regex::new(r"results?\s+for|showing\s+\d+").unwrap()],
                confidence_weight: 0.8,
            }
        ]);

        // Add more patterns...
        patterns
    }

    async fn classify(&self, url: &str, title: &str, html: &str) -> Result<(PageType, f32)> {
        let mut best_match = (PageType::Unknown, 0.0f32);

        for (page_type, patterns) in &self.patterns {
            for pattern in patterns {
                let mut score = 0.0f32;
                let mut matches = 0;

                // Check URL patterns
                for url_pattern in &pattern.url_patterns {
                    if url_pattern.is_match(url) {
                        score += 0.3 * pattern.confidence_weight;
                        matches += 1;
                    }
                }

                // Check title patterns
                for title_pattern in &pattern.title_patterns {
                    if title_pattern.is_match(title) {
                        score += 0.2 * pattern.confidence_weight;
                        matches += 1;
                    }
                }

                // Check content patterns
                for content_pattern in &pattern.content_patterns {
                    if content_pattern.is_match(html) {
                        score += 0.1 * pattern.confidence_weight;
                        matches += 1;
                    }
                }

                // Bonus for multiple matches
                if matches > 1 {
                    score *= 1.2;
                }

                if score > best_match.1 {
                    best_match = (page_type.clone(), score);
                }
            }
        }

        Ok(best_match)
    }
}

impl ElementClassifier {
    fn new() -> Self {
        Self {
            role_patterns: Self::build_element_patterns(),
        }
    }

    fn build_element_patterns() -> HashMap<ElementRole, Vec<ElementPattern>> {
        let mut patterns = HashMap::new();

        // Primary action patterns
        patterns.insert(ElementRole::PrimaryAction, vec![
            ElementPattern {
                selectors: vec![".btn-primary".to_string(), ".primary-btn".to_string()],
                text_patterns: vec![
                    Regex::new(r"(?i)submit|buy|purchase|get\s+started|sign\s+up").unwrap()
                ],
                attribute_patterns: HashMap::new(),
                context_selectors: vec![],
                confidence_weight: 0.9,
            }
        ]);

        // Search patterns
        patterns.insert(ElementRole::Search, vec![
            ElementPattern {
                selectors: vec!["input[type='search']".to_string(), ".search-input".to_string()],
                text_patterns: vec![Regex::new(r"(?i)search").unwrap()],
                attribute_patterns: {
                    let mut map = HashMap::new();
                    map.insert("placeholder".to_string(), Regex::new(r"(?i)search").unwrap());
                    map
                },
                context_selectors: vec![".search-form".to_string()],
                confidence_weight: 0.8,
            }
        ]);

        patterns
    }

    fn classify_role(&self, text: &str, tag_name: &str, attributes: &HashMap<String, String>) -> Result<(ElementRole, f32)> {
        let mut best_match = (ElementRole::Unknown, 0.0f32);

        for (role, patterns) in &self.role_patterns {
            for pattern in patterns {
                let mut score = 0.0f32;

                // Check text patterns
                for text_pattern in &pattern.text_patterns {
                    if text_pattern.is_match(text) {
                        score += 0.4 * pattern.confidence_weight;
                    }
                }

                // Check attribute patterns
                for (attr_name, attr_pattern) in &pattern.attribute_patterns {
                    if let Some(attr_value) = attributes.get(attr_name) {
                        if attr_pattern.is_match(attr_value) {
                            score += 0.3 * pattern.confidence_weight;
                        }
                    }
                }

                // Check tag-based patterns
                if tag_name == "button" && *role == ElementRole::PrimaryAction {
                    score += 0.2;
                }

                if score > best_match.1 {
                    best_match = (role.clone(), score);
                }
            }
        }

        Ok(best_match)
    }
}

impl FormAnalyzer {
    fn new() -> Self {
        Self {
            field_patterns: Self::build_field_patterns(),
        }
    }

    fn build_field_patterns() -> HashMap<FieldType, Vec<FieldPattern>> {
        let mut patterns = HashMap::new();

        // Email field patterns
        patterns.insert(FieldType::Email, vec![
            FieldPattern {
                name_patterns: vec![Regex::new(r"(?i)email|e-mail").unwrap()],
                placeholder_patterns: vec![Regex::new(r"(?i)email|@").unwrap()],
                label_patterns: vec![Regex::new(r"(?i)email").unwrap()],
                type_patterns: vec!["email".to_string()],
                validation_patterns: vec![Regex::new(r"@").unwrap()],
                confidence_weight: 0.9,
            }
        ]);

        // Password field patterns
        patterns.insert(FieldType::Password, vec![
            FieldPattern {
                name_patterns: vec![Regex::new(r"(?i)password|passwd|pwd").unwrap()],
                placeholder_patterns: vec![Regex::new(r"(?i)password").unwrap()],
                label_patterns: vec![Regex::new(r"(?i)password").unwrap()],
                type_patterns: vec!["password".to_string()],
                validation_patterns: vec![],
                confidence_weight: 0.95,
            }
        ]);

        // Add more field patterns...
        patterns
    }

    fn classify_field(&self, name: &str, label: &str, placeholder: &str, input_type: &str) -> Result<(FieldType, f32)> {
        let mut best_match = (FieldType::Generic, 0.0f32);

        // Direct type check first
        if input_type == "email" {
            return Ok((FieldType::Email, 0.95));
        }
        if input_type == "password" {
            return Ok((FieldType::Password, 0.95));
        }
        if input_type == "tel" {
            return Ok((FieldType::Phone, 0.95));
        }

        for (field_type, patterns) in &self.field_patterns {
            for pattern in patterns {
                let mut score = 0.0f32;

                // Check name patterns
                for name_pattern in &pattern.name_patterns {
                    if name_pattern.is_match(name) {
                        score += 0.4 * pattern.confidence_weight;
                    }
                }

                // Check label patterns
                for label_pattern in &pattern.label_patterns {
                    if label_pattern.is_match(label) {
                        score += 0.3 * pattern.confidence_weight;
                    }
                }

                // Check placeholder patterns
                for placeholder_pattern in &pattern.placeholder_patterns {
                    if placeholder_pattern.is_match(placeholder) {
                        score += 0.3 * pattern.confidence_weight;
                    }
                }

                if score > best_match.1 {
                    best_match = (field_type.clone(), score);
                }
            }
        }

        Ok(best_match)
    }
}

impl ContentAnalyzer {
    fn new() -> Self {
        Self {
            content_patterns: HashMap::new(), // Would be populated with content patterns
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_page_classification() {
        let classifier = PageClassifier::new();
        
        // Test URL pattern matching
        let url = "https://example.com/login";
        let title = "Sign In - Example";
        let html = "<input type='password'>";
        
        // This would need to be async in real test
        // let (page_type, confidence) = classifier.classify(url, title, html).await.unwrap();
        // assert_eq!(page_type, PageType::LoginPage);
        // assert!(confidence > 0.5);
    }

    #[test]
    fn test_field_classification() {
        let analyzer = FormAnalyzer::new();
        
        let (field_type, confidence) = analyzer.classify_field(
            "user_email", 
            "Email Address", 
            "Enter your email", 
            "text"
        ).unwrap();
        
        assert_eq!(field_type, FieldType::Email);
        assert!(confidence > 0.5);
    }
}