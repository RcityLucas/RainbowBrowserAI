// Natural Language Element Finding
// Converts human descriptions to precise element selectors

use anyhow::{Result, Context};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use regex::Regex;
use tracing::{debug, info, warn};
use super::browser_connection::BrowserConnection;
use super::quick_real::{QuickData, InteractionElement};

/// Natural language element finder
pub struct NaturalLanguageFinder {
    /// Pattern matchers for common phrases
    patterns: Vec<PatternMatcher>,
    
    /// Context from previous interactions
    context: InteractionContext,
    
    /// Cached element mappings
    element_cache: HashMap<String, ElementMapping>,
}

/// Maps a natural language pattern to element finding strategy
#[derive(Clone)]
struct PatternMatcher {
    pattern: Regex,
    strategy: FindStrategy,
    priority: u8,
}

/// Strategy for finding elements
#[derive(Clone, Debug)]
enum FindStrategy {
    ButtonText(String),
    LinkText(String),
    InputPlaceholder(String),
    InputLabel(String),
    AriaLabel(String),
    ElementRole(String),
    FormField(String),
    Navigation(String),
    CommonPattern(CommonUIPattern),
    Composite(Vec<FindStrategy>),
}

/// Common UI patterns that users refer to
#[derive(Clone, Debug)]
enum CommonUIPattern {
    SearchBox,
    LoginButton,
    SignUpButton,
    SubmitButton,
    NavigationMenu,
    Logo,
    ShoppingCart,
    UserProfile,
    Settings,
    HelpButton,
}

/// Context from previous interactions
#[derive(Default)]
struct InteractionContext {
    last_clicked: Option<String>,
    last_typed_field: Option<String>,
    named_elements: HashMap<String, String>,
    page_type: Option<String>,
}

/// Cached element mapping
#[derive(Clone)]
struct ElementMapping {
    selector: String,
    confidence: f32,
    last_used: std::time::Instant,
}

/// Result of natural language element finding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NaturalLanguageResult {
    pub selector: String,
    pub element_type: String,
    pub confidence: f32,
    pub alternatives: Vec<AlternativeElement>,
    pub interpretation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlternativeElement {
    pub selector: String,
    pub confidence: f32,
    pub reason: String,
}

impl NaturalLanguageFinder {
    pub fn new() -> Self {
        Self {
            patterns: Self::build_patterns(),
            context: InteractionContext::default(),
            element_cache: HashMap::new(),
        }
    }
    
    /// Build pattern matchers for common phrases
    fn build_patterns() -> Vec<PatternMatcher> {
        let mut patterns = Vec::new();
        
        // Button patterns
        patterns.push(PatternMatcher {
            pattern: Regex::new(r"(?i)(?:click|press|tap|hit)\s+(?:the\s+)?(.+?)\s+button").unwrap(),
            strategy: FindStrategy::ButtonText("$1".to_string()),
            priority: 10,
        });
        
        patterns.push(PatternMatcher {
            pattern: Regex::new(r#"(?i)(?:click|press|tap)\s+(?:on\s+)?['"](.+?)['"]"#).unwrap(),
            strategy: FindStrategy::ButtonText("$1".to_string()),
            priority: 9,
        });
        
        // Link patterns
        patterns.push(PatternMatcher {
            pattern: Regex::new(r"(?i)(?:click|go to|navigate to|open)\s+(?:the\s+)?(.+?)\s+link").unwrap(),
            strategy: FindStrategy::LinkText("$1".to_string()),
            priority: 10,
        });
        
        // Input field patterns
        patterns.push(PatternMatcher {
            pattern: Regex::new(r"(?i)(?:type|enter|fill|input)\s+.*?\s+in(?:to)?\s+(?:the\s+)?(.+?)(?:\s+field)?").unwrap(),
            strategy: FindStrategy::InputLabel("$1".to_string()),
            priority: 8,
        });
        
        patterns.push(PatternMatcher {
            pattern: Regex::new(r"(?i)(?:the\s+)?(.+?)\s+(?:input|field|box|textbox)").unwrap(),
            strategy: FindStrategy::InputLabel("$1".to_string()),
            priority: 7,
        });
        
        // Search patterns
        patterns.push(PatternMatcher {
            pattern: Regex::new(r"(?i)search\s+(?:box|field|bar|input)").unwrap(),
            strategy: FindStrategy::CommonPattern(CommonUIPattern::SearchBox),
            priority: 10,
        });
        
        // Login/Sign up patterns
        patterns.push(PatternMatcher {
            pattern: Regex::new(r"(?i)(?:login|log\s+in|sign\s+in)\s+button").unwrap(),
            strategy: FindStrategy::CommonPattern(CommonUIPattern::LoginButton),
            priority: 10,
        });
        
        patterns.push(PatternMatcher {
            pattern: Regex::new(r"(?i)(?:signup|sign\s+up|register)\s+button").unwrap(),
            strategy: FindStrategy::CommonPattern(CommonUIPattern::SignUpButton),
            priority: 10,
        });
        
        // Submit patterns
        patterns.push(PatternMatcher {
            pattern: Regex::new(r"(?i)submit\s+(?:button|form)?").unwrap(),
            strategy: FindStrategy::CommonPattern(CommonUIPattern::SubmitButton),
            priority: 9,
        });
        
        // Navigation patterns
        patterns.push(PatternMatcher {
            pattern: Regex::new(r"(?i)(?:navigation|nav)\s+menu").unwrap(),
            strategy: FindStrategy::CommonPattern(CommonUIPattern::NavigationMenu),
            priority: 8,
        });
        
        // Common UI elements
        patterns.push(PatternMatcher {
            pattern: Regex::new(r"(?i)(?:the\s+)?logo").unwrap(),
            strategy: FindStrategy::CommonPattern(CommonUIPattern::Logo),
            priority: 7,
        });
        
        patterns.push(PatternMatcher {
            pattern: Regex::new(r"(?i)shopping\s+cart").unwrap(),
            strategy: FindStrategy::CommonPattern(CommonUIPattern::ShoppingCart),
            priority: 8,
        });
        
        patterns.push(PatternMatcher {
            pattern: Regex::new(r"(?i)(?:user\s+)?profile").unwrap(),
            strategy: FindStrategy::CommonPattern(CommonUIPattern::UserProfile),
            priority: 7,
        });
        
        patterns.push(PatternMatcher {
            pattern: Regex::new(r"(?i)settings").unwrap(),
            strategy: FindStrategy::CommonPattern(CommonUIPattern::Settings),
            priority: 7,
        });
        
        // Sort by priority (higher priority first)
        patterns.sort_by(|a, b| b.priority.cmp(&a.priority));
        
        patterns
    }
    
    /// Find element using natural language description
    pub async fn find_element(
        &mut self,
        description: &str,
        browser: &BrowserConnection,
        quick_data: Option<&QuickData>,
    ) -> Result<NaturalLanguageResult> {
        info!("Finding element: \"{}\"", description);
        
        // Check cache first
        if let Some(cached) = self.get_cached(description) {
            return Ok(NaturalLanguageResult {
                selector: cached.selector.clone(),
                element_type: "cached".to_string(),
                confidence: cached.confidence,
                alternatives: vec![],
                interpretation: format!("Using cached selector for '{}'", description),
            });
        }
        
        // Parse the description
        let strategies = self.parse_description(description);
        
        if strategies.is_empty() {
            // Fallback to fuzzy matching
            return self.fuzzy_find(description, browser, quick_data).await;
        }
        
        // Execute strategies
        let mut candidates = Vec::new();
        for strategy in strategies {
            if let Ok(results) = self.execute_strategy(&strategy, browser, quick_data).await {
                candidates.extend(results);
            }
        }
        
        // Rank and select best candidate
        let best = self.select_best_candidate(candidates, description)?;
        
        // Cache the result
        self.cache_element(description, &best);
        
        Ok(best)
    }
    
    /// Parse natural language description into strategies
    fn parse_description(&self, description: &str) -> Vec<FindStrategy> {
        let mut strategies = Vec::new();
        let desc_lower = description.to_lowercase();
        
        // Check against patterns
        for matcher in &self.patterns {
            if matcher.pattern.is_match(&desc_lower) {
                strategies.push(matcher.strategy.clone());
            }
        }
        
        // Check for references to previous elements
        if desc_lower == "it" || desc_lower == "that" || desc_lower == "the same" {
            if let Some(last) = &self.context.last_clicked {
                strategies.push(FindStrategy::ButtonText(last.clone()));
            }
        }
        
        // Check named elements
        for (name, selector) in &self.context.named_elements {
            if desc_lower.contains(name) {
                strategies.push(FindStrategy::AriaLabel(selector.clone()));
            }
        }
        
        strategies
    }
    
    /// Execute a finding strategy
    async fn execute_strategy(
        &self,
        strategy: &FindStrategy,
        browser: &BrowserConnection,
        quick_data: Option<&QuickData>,
    ) -> Result<Vec<NaturalLanguageResult>> {
        match strategy {
            FindStrategy::ButtonText(text) => {
                self.find_button_by_text(text, browser).await
            }
            FindStrategy::LinkText(text) => {
                self.find_link_by_text(text, browser).await
            }
            FindStrategy::InputPlaceholder(text) => {
                self.find_input_by_placeholder(text, browser).await
            }
            FindStrategy::InputLabel(text) => {
                self.find_input_by_label(text, browser).await
            }
            FindStrategy::AriaLabel(label) => {
                self.find_by_aria_label(label, browser).await
            }
            FindStrategy::ElementRole(role) => {
                self.find_by_role(role, browser).await
            }
            FindStrategy::CommonPattern(pattern) => {
                self.find_common_pattern(pattern, browser).await
            }
            FindStrategy::Composite(strategies) => {
                let mut results = Vec::new();
                for s in strategies {
                    if let Ok(r) = self.execute_strategy(s, browser, quick_data).await {
                        results.extend(r);
                    }
                }
                Ok(results)
            }
            _ => Ok(vec![]),
        }
    }
    
    /// Find button by text content
    async fn find_button_by_text(&self, text: &str, browser: &BrowserConnection) -> Result<Vec<NaturalLanguageResult>> {
        let script = format!(r##"
            function findButtonByText(searchText) {{
                const results = [];
                const cleanText = searchText.toLowerCase().trim();
                
                // Check actual buttons
                document.querySelectorAll('button').forEach(btn => {{
                    const btnText = (btn.innerText || '').toLowerCase().trim();
                    if (btnText.includes(cleanText) || cleanText.includes(btnText)) {{
                        results.push({{
                            selector: btn.id ? '#' + btn.id : 'button:contains("' + btn.innerText + '")',
                            element_type: 'button',
                            confidence: btnText === cleanText ? 1.0 : 0.8,
                            text: btn.innerText
                        }});
                    }}
                }});
                
                // Check input buttons
                document.querySelectorAll('input[type="button"], input[type="submit"]').forEach(input => {{
                    const value = (input.value || '').toLowerCase().trim();
                    if (value.includes(cleanText) || cleanText.includes(value)) {{
                        results.push({{
                            selector: input.id ? '#' + input.id : 'input[value="' + input.value + '"]',
                            element_type: 'input_button',
                            confidence: value === cleanText ? 0.9 : 0.7,
                            text: input.value
                        }});
                    }}
                }});
                
                // Check links styled as buttons
                document.querySelectorAll('a.button, a.btn, a[role="button"]').forEach(link => {{
                    const linkText = (link.innerText || '').toLowerCase().trim();
                    if (linkText.includes(cleanText) || cleanText.includes(linkText)) {{
                        results.push({{
                            selector: link.id ? '#' + link.id : 'a.button:contains("' + link.innerText + '")',
                            element_type: 'link_button',
                            confidence: 0.7,
                            text: link.innerText
                        }});
                    }}
                }});
                
                return results;
            }}
            
            return findButtonByText("{}");
        "##, text);
        
        let result = browser.execute_script(&script, vec![]).await?;
        let elements: Vec<serde_json::Value> = serde_json::from_value(result).unwrap_or_default();
        
        Ok(elements.into_iter().map(|e| NaturalLanguageResult {
            selector: e["selector"].as_str().unwrap_or("").to_string(),
            element_type: e["element_type"].as_str().unwrap_or("").to_string(),
            confidence: e["confidence"].as_f64().unwrap_or(0.5) as f32,
            alternatives: vec![],
            interpretation: format!("Button with text '{}'", text),
        }).collect())
    }
    
    /// Find link by text content
    async fn find_link_by_text(&self, text: &str, browser: &BrowserConnection) -> Result<Vec<NaturalLanguageResult>> {
        let script = format!(r##"
            function findLinkByText(searchText) {{
                const results = [];
                const cleanText = searchText.toLowerCase().trim();
                
                document.querySelectorAll('a').forEach(link => {{
                    const linkText = (link.innerText || '').toLowerCase().trim();
                    if (linkText.includes(cleanText) || cleanText.includes(linkText)) {{
                        results.push({{
                            selector: link.id ? '#' + link.id : 'a:contains("' + link.innerText + '")',
                            element_type: 'link',
                            confidence: linkText === cleanText ? 1.0 : 0.7,
                            text: link.innerText,
                            href: link.href
                        }});
                    }}
                }});
                
                return results;
            }}
            
            return findLinkByText("{}");
        "##, text);
        
        let result = browser.execute_script(&script, vec![]).await?;
        let elements: Vec<serde_json::Value> = serde_json::from_value(result).unwrap_or_default();
        
        Ok(elements.into_iter().map(|e| NaturalLanguageResult {
            selector: e["selector"].as_str().unwrap_or("").to_string(),
            element_type: "link".to_string(),
            confidence: e["confidence"].as_f64().unwrap_or(0.5) as f32,
            alternatives: vec![],
            interpretation: format!("Link with text '{}'", text),
        }).collect())
    }
    
    /// Find input by placeholder text
    async fn find_input_by_placeholder(&self, text: &str, browser: &BrowserConnection) -> Result<Vec<NaturalLanguageResult>> {
        let script = format!(r##"
            function findInputByPlaceholder(searchText) {{
                const results = [];
                const cleanText = searchText.toLowerCase().trim();
                
                document.querySelectorAll('input[placeholder], textarea[placeholder]').forEach(input => {{
                    const placeholder = (input.placeholder || '').toLowerCase().trim();
                    if (placeholder.includes(cleanText) || cleanText.includes(placeholder)) {{
                        results.push({{
                            selector: input.id ? '#' + input.id : 
                                     input.name ? 'input[name="' + input.name + '"]' :
                                     'input[placeholder="' + input.placeholder + '"]',
                            element_type: input.tagName.toLowerCase(),
                            confidence: placeholder === cleanText ? 1.0 : 0.7,
                            placeholder: input.placeholder
                        }});
                    }}
                }});
                
                return results;
            }}
            
            return findInputByPlaceholder("{}");
        "##, text);
        
        let result = browser.execute_script(&script, vec![]).await?;
        let elements: Vec<serde_json::Value> = serde_json::from_value(result).unwrap_or_default();
        
        Ok(elements.into_iter().map(|e| NaturalLanguageResult {
            selector: e["selector"].as_str().unwrap_or("").to_string(),
            element_type: e["element_type"].as_str().unwrap_or("input").to_string(),
            confidence: e["confidence"].as_f64().unwrap_or(0.5) as f32,
            alternatives: vec![],
            interpretation: format!("Input with placeholder '{}'", text),
        }).collect())
    }
    
    /// Find input by label text
    async fn find_input_by_label(&self, text: &str, browser: &BrowserConnection) -> Result<Vec<NaturalLanguageResult>> {
        let script = format!(r##"
            function findInputByLabel(searchText) {{
                const results = [];
                const cleanText = searchText.toLowerCase().trim();
                
                // Find labels matching text
                document.querySelectorAll('label').forEach(label => {{
                    const labelText = (label.innerText || '').toLowerCase().trim();
                    if (labelText.includes(cleanText) || cleanText.includes(labelText)) {{
                        // Find associated input
                        let input = null;
                        if (label.htmlFor) {{
                            input = document.getElementById(label.htmlFor);
                        }} else {{
                            input = label.querySelector('input, select, textarea');
                        }}
                        
                        if (input) {{
                            results.push({{
                                selector: input.id ? '#' + input.id : 
                                         input.name ? input.tagName.toLowerCase() + '[name="' + input.name + '"]' :
                                         input.tagName.toLowerCase(),
                                element_type: input.tagName.toLowerCase(),
                                confidence: labelText === cleanText ? 1.0 : 0.8,
                                label: label.innerText
                            }});
                        }}
                    }}
                }});
                
                return results;
            }}
            
            return findInputByLabel("{}");
        "##, text);
        
        let result = browser.execute_script(&script, vec![]).await?;
        let elements: Vec<serde_json::Value> = serde_json::from_value(result).unwrap_or_default();
        
        Ok(elements.into_iter().map(|e| NaturalLanguageResult {
            selector: e["selector"].as_str().unwrap_or("").to_string(),
            element_type: e["element_type"].as_str().unwrap_or("input").to_string(),
            confidence: e["confidence"].as_f64().unwrap_or(0.5) as f32,
            alternatives: vec![],
            interpretation: format!("Input field labeled '{}'", text),
        }).collect())
    }
    
    /// Find element by aria-label
    async fn find_by_aria_label(&self, label: &str, browser: &BrowserConnection) -> Result<Vec<NaturalLanguageResult>> {
        let script = format!(r##"
            function findByAriaLabel(searchLabel) {{
                const results = [];
                const cleanLabel = searchLabel.toLowerCase().trim();
                
                document.querySelectorAll('[aria-label]').forEach(elem => {{
                    const ariaLabel = (elem.getAttribute('aria-label') || '').toLowerCase().trim();
                    if (ariaLabel.includes(cleanLabel) || cleanLabel.includes(ariaLabel)) {{
                        results.push({{
                            selector: elem.id ? '#' + elem.id : 
                                     elem.className ? elem.tagName.toLowerCase() + '.' + elem.className.split(' ')[0] :
                                     elem.tagName.toLowerCase() + '[aria-label="' + elem.getAttribute('aria-label') + '"]',
                            element_type: elem.tagName.toLowerCase(),
                            confidence: ariaLabel === cleanLabel ? 1.0 : 0.7,
                            aria_label: elem.getAttribute('aria-label')
                        }});
                    }}
                }});
                
                return results;
            }}
            
            return findByAriaLabel("{}");
        "##, label);
        
        let result = browser.execute_script(&script, vec![]).await?;
        let elements: Vec<serde_json::Value> = serde_json::from_value(result).unwrap_or_default();
        
        Ok(elements.into_iter().map(|e| NaturalLanguageResult {
            selector: e["selector"].as_str().unwrap_or("").to_string(),
            element_type: e["element_type"].as_str().unwrap_or("").to_string(),
            confidence: e["confidence"].as_f64().unwrap_or(0.5) as f32,
            alternatives: vec![],
            interpretation: format!("Element with aria-label '{}'", label),
        }).collect())
    }
    
    /// Find element by role
    async fn find_by_role(&self, role: &str, browser: &BrowserConnection) -> Result<Vec<NaturalLanguageResult>> {
        let script = format!(r##"
            function findByRole(searchRole) {{
                const results = [];
                
                document.querySelectorAll('[role="{}"]').forEach(elem => {{
                    results.push({{
                        selector: elem.id ? '#' + elem.id : 
                                 elem.className ? elem.tagName.toLowerCase() + '.' + elem.className.split(' ')[0] :
                                 '[role="{}"]',
                        element_type: elem.tagName.toLowerCase(),
                        confidence: 0.8,
                        role: '{}'
                    }});
                }});
                
                return results;
            }}
            
            return findByRole();
        "##, role, role, role);
        
        let result = browser.execute_script(&script, vec![]).await?;
        let elements: Vec<serde_json::Value> = serde_json::from_value(result).unwrap_or_default();
        
        Ok(elements.into_iter().map(|e| NaturalLanguageResult {
            selector: e["selector"].as_str().unwrap_or("").to_string(),
            element_type: e["element_type"].as_str().unwrap_or("").to_string(),
            confidence: e["confidence"].as_f64().unwrap_or(0.5) as f32,
            alternatives: vec![],
            interpretation: format!("Element with role '{}'", role),
        }).collect())
    }
    
    /// Find common UI patterns
    async fn find_common_pattern(&self, pattern: &CommonUIPattern, browser: &BrowserConnection) -> Result<Vec<NaturalLanguageResult>> {
        let selectors = match pattern {
            CommonUIPattern::SearchBox => vec![
                ("input[type='search']", 1.0),
                ("input[placeholder*='search' i]", 0.9),
                ("input[name*='search' i]", 0.8),
                ("#search", 0.8),
                (".search-input", 0.7),
                ("input[aria-label*='search' i]", 0.8),
            ],
            CommonUIPattern::LoginButton => vec![
                ("button:contains('Login')", 1.0),
                ("button:contains('Sign in')", 1.0),
                ("a:contains('Login')", 0.8),
                ("a:contains('Sign in')", 0.8),
                ("#login-button", 0.9),
                (".login-button", 0.8),
            ],
            CommonUIPattern::SignUpButton => vec![
                ("button:contains('Sign up')", 1.0),
                ("button:contains('Register')", 1.0),
                ("a:contains('Sign up')", 0.8),
                ("a:contains('Register')", 0.8),
                ("#signup-button", 0.9),
                (".signup-button", 0.8),
            ],
            CommonUIPattern::SubmitButton => vec![
                ("button[type='submit']", 1.0),
                ("input[type='submit']", 0.9),
                ("button:contains('Submit')", 0.8),
                ("button:contains('Send')", 0.7),
            ],
            CommonUIPattern::NavigationMenu => vec![
                ("nav", 1.0),
                ("[role='navigation']", 0.9),
                (".navigation", 0.8),
                (".nav-menu", 0.8),
                ("#navigation", 0.8),
            ],
            CommonUIPattern::Logo => vec![
                (".logo", 1.0),
                ("#logo", 1.0),
                ("img.logo", 0.9),
                ("a.logo", 0.8),
                ("[aria-label*='logo' i]", 0.7),
            ],
            CommonUIPattern::ShoppingCart => vec![
                (".cart", 1.0),
                ("#cart", 1.0),
                ("[aria-label*='cart' i]", 0.9),
                ("button:contains('Cart')", 0.8),
                ("a:contains('Cart')", 0.8),
            ],
            CommonUIPattern::UserProfile => vec![
                (".profile", 1.0),
                ("#profile", 1.0),
                ("[aria-label*='profile' i]", 0.9),
                (".user-profile", 0.9),
                ("button:contains('Profile')", 0.8),
            ],
            CommonUIPattern::Settings => vec![
                (".settings", 1.0),
                ("#settings", 1.0),
                ("[aria-label*='settings' i]", 0.9),
                ("button:contains('Settings')", 0.8),
                ("a:contains('Settings')", 0.8),
            ],
            CommonUIPattern::HelpButton => vec![
                ("button:contains('Help')", 1.0),
                ("a:contains('Help')", 0.9),
                (".help-button", 0.9),
                ("[aria-label*='help' i]", 0.8),
            ],
        };
        
        let mut results = Vec::new();
        
        for (selector, confidence) in selectors {
            let script = format!(r##"
                const elem = document.querySelector("{}");
                if (elem && elem.offsetWidth > 0 && elem.offsetHeight > 0) {{
                    return {{
                        found: true,
                        selector: "{}",
                        element_type: elem.tagName.toLowerCase(),
                        text: elem.innerText || elem.value || ""
                    }};
                }}
                return {{ found: false }};
            "##, selector, selector);
            
            if let Ok(result) = browser.execute_script(&script, vec![]).await {
                if let Ok(data) = serde_json::from_value::<serde_json::Value>(result) {
                    if data["found"].as_bool().unwrap_or(false) {
                        results.push(NaturalLanguageResult {
                            selector: data["selector"].as_str().unwrap_or(selector).to_string(),
                            element_type: data["element_type"].as_str().unwrap_or("").to_string(),
                            confidence: confidence as f32,
                            alternatives: vec![],
                            interpretation: format!("{:?} element", pattern),
                        });
                        break; // Use first matching selector
                    }
                }
            }
        }
        
        Ok(results)
    }
    
    /// Fuzzy find when no patterns match
    async fn fuzzy_find(
        &self,
        description: &str,
        browser: &BrowserConnection,
        quick_data: Option<&QuickData>,
    ) -> Result<NaturalLanguageResult> {
        // Use Quick perception data if available
        if let Some(data) = quick_data {
            for element in &data.interaction_elements {
                if Self::fuzzy_match(&element.text, description) {
                    return Ok(NaturalLanguageResult {
                        selector: element.selector.clone(),
                        element_type: format!("{:?}", element.element_type),
                        confidence: 0.6,
                        alternatives: vec![],
                        interpretation: format!("Fuzzy match for '{}'", description),
                    });
                }
            }
        }
        
        // Fallback to generic search
        let script = format!(r##"
            function fuzzyFind(searchText) {{
                const cleanText = searchText.toLowerCase().trim();
                const allElements = document.querySelectorAll('button, a, input, select, [role="button"], [onclick]');
                
                for (const elem of allElements) {{
                    const text = (elem.innerText || elem.value || elem.placeholder || '').toLowerCase();
                    if (text.includes(cleanText) || cleanText.includes(text)) {{
                        return {{
                            found: true,
                            selector: elem.id ? '#' + elem.id : elem.tagName.toLowerCase(),
                            element_type: elem.tagName.toLowerCase(),
                            confidence: 0.5
                        }};
                    }}
                }}
                
                return {{ found: false }};
            }}
            
            return fuzzyFind("{}");
        "##, description);
        
        let result = browser.execute_script(&script, vec![]).await?;
        let data: serde_json::Value = serde_json::from_value(result)?;
        
        if data["found"].as_bool().unwrap_or(false) {
            Ok(NaturalLanguageResult {
                selector: data["selector"].as_str().unwrap_or("").to_string(),
                element_type: data["element_type"].as_str().unwrap_or("").to_string(),
                confidence: data["confidence"].as_f64().unwrap_or(0.3) as f32,
                alternatives: vec![],
                interpretation: format!("Fuzzy match for '{}'", description),
            })
        } else {
            Err(anyhow::anyhow!("No element found matching: {}", description))
        }
    }
    
    /// Fuzzy string matching
    fn fuzzy_match(text1: &str, text2: &str) -> bool {
        let t1 = text1.to_lowercase();
        let t2 = text2.to_lowercase();
        
        // Direct containment
        if t1.contains(&t2) || t2.contains(&t1) {
            return true;
        }
        
        // Word overlap
        let words1: Vec<&str> = t1.split_whitespace().collect();
        let words2: Vec<&str> = t2.split_whitespace().collect();
        
        for w1 in &words1 {
            for w2 in &words2 {
                if w1 == w2 && w1.len() > 2 {
                    return true;
                }
            }
        }
        
        false
    }
    
    /// Select best candidate from multiple options
    fn select_best_candidate(&self, mut candidates: Vec<NaturalLanguageResult>, description: &str) -> Result<NaturalLanguageResult> {
        if candidates.is_empty() {
            return Err(anyhow::anyhow!("No elements found matching: {}", description));
        }
        
        // Sort by confidence
        candidates.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
        
        // Take top result and add others as alternatives
        let mut best = candidates.remove(0);
        best.alternatives = candidates.into_iter()
            .take(3)
            .map(|c| AlternativeElement {
                selector: c.selector,
                confidence: c.confidence,
                reason: c.interpretation,
            })
            .collect();
        
        Ok(best)
    }
    
    /// Get cached element mapping
    fn get_cached(&self, description: &str) -> Option<&ElementMapping> {
        self.element_cache.get(description)
            .filter(|m| m.last_used.elapsed() < std::time::Duration::from_secs(60))
    }
    
    /// Cache element mapping
    fn cache_element(&mut self, description: &str, result: &NaturalLanguageResult) {
        self.element_cache.insert(
            description.to_string(),
            ElementMapping {
                selector: result.selector.clone(),
                confidence: result.confidence,
                last_used: std::time::Instant::now(),
            }
        );
        
        // Update context
        if description.contains("button") {
            self.context.last_clicked = Some(result.selector.clone());
        } else if description.contains("field") || description.contains("input") {
            self.context.last_typed_field = Some(result.selector.clone());
        }
        
        // Store named element
        if description.starts_with("the ") {
            self.context.named_elements.insert(
                description.to_string(),
                result.selector.clone()
            );
        }
    }
    
    /// Update context with page type
    pub fn set_page_type(&mut self, page_type: String) {
        self.context.page_type = Some(page_type);
    }
    
    /// Clear context (for new page)
    pub fn clear_context(&mut self) {
        self.context = InteractionContext::default();
        self.element_cache.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_pattern_matching() {
        let finder = NaturalLanguageFinder::new();
        
        // Test button patterns
        let strategies = finder.parse_description("click the submit button");
        assert!(!strategies.is_empty());
        
        // Test input patterns
        let strategies = finder.parse_description("type in the email field");
        assert!(!strategies.is_empty());
        
        // Test search pattern
        let strategies = finder.parse_description("search box");
        assert!(!strategies.is_empty());
    }
    
    #[test]
    fn test_fuzzy_matching() {
        assert!(NaturalLanguageFinder::fuzzy_match("Submit Form", "submit"));
        assert!(NaturalLanguageFinder::fuzzy_match("Login", "log in"));
        assert!(NaturalLanguageFinder::fuzzy_match("Search Products", "search"));
    }
}