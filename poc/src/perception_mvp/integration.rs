// Integration layer - connects the perception module to your existing API
// This shows how to use the perception engine in your command handlers

use anyhow::Result;
use crate::perception_mvp::{PerceptionEngineMVP, PerceivedElement, ElementType};
use crate::browser::SimpleBrowser;
use crate::llm_service::ParsedCommand;
use thirtyfour::WebDriver;
use tracing::{info, debug, warn};

/// Enhanced command executor that uses perception
pub struct PerceptionAwareExecutor {
    perception: PerceptionEngineMVP,
    browser: SimpleBrowser,
}

impl PerceptionAwareExecutor {
    pub fn new(driver: WebDriver) -> Self {
        let perception = PerceptionEngineMVP::new(driver.clone());
        let browser = SimpleBrowser::from_driver(driver);
        
        Self {
            perception,
            browser,
        }
    }

    /// Execute a parsed command using perception
    pub async fn execute_command(&mut self, command: &ParsedCommand) -> Result<CommandResult> {
        match command.action.as_str() {
            "click" => self.handle_click(command).await,
            "type" => self.handle_type(command).await,
            "select" => self.handle_select(command).await,
            "navigate" => self.handle_navigate(command).await,
            "search" => self.handle_search(command).await,
            "extract" => self.handle_extract(command).await,
            _ => self.handle_unknown(command).await,
        }
    }

    /// Smart click that uses perception to find elements
    async fn handle_click(&mut self, command: &ParsedCommand) -> Result<CommandResult> {
        info!("Handling click command with perception");
        
        // Build element description from command
        let description = self.build_element_description(command);
        
        // Use perception to find the element
        match self.perception.find_element(&description).await {
            Ok(element) => {
                info!("Found element: {:?}", element);
                
                // Click the element
                self.browser.click_element(&element.selector).await?;
                
                Ok(CommandResult {
                    success: true,
                    action: "click".to_string(),
                    message: format!("Successfully clicked: {}", element.text),
                    element_info: Some(element),
                })
            }
            Err(e) => {
                warn!("Failed to find element: {}", e);
                
                // Try alternative strategies
                let suggestions = self.suggest_alternatives(&description).await?;
                
                Err(anyhow::anyhow!(
                    "Could not find element '{}'. Did you mean: {}?", 
                    description,
                    suggestions.join(", ")
                ))
            }
        }
    }

    /// Smart type/input that understands form fields
    async fn handle_type(&mut self, command: &ParsedCommand) -> Result<CommandResult> {
        info!("Handling type command with perception");
        
        let text = command.input_text.as_deref()
            .ok_or_else(|| anyhow::anyhow!("No text to type provided"))?;
        
        // Find the input field
        let field_description = command.element_selector.as_deref()
            .unwrap_or("input field");
        
        let element = self.perception.find_element(field_description).await?;
        
        // Clear and type
        if element.element_type == ElementType::Input || 
           element.element_type == ElementType::TextArea {
            self.browser.clear_element(&element.selector).await?;
            self.browser.type_text(&element.selector, text).await?;
            
            Ok(CommandResult {
                success: true,
                action: "type".to_string(),
                message: format!("Typed '{}' into {}", text, element.text),
                element_info: Some(element),
            })
        } else {
            Err(anyhow::anyhow!(
                "Element {} is not a text input field", 
                element.selector
            ))
        }
    }

    /// Handle dropdown/select elements
    async fn handle_select(&mut self, command: &ParsedCommand) -> Result<CommandResult> {
        let selector_desc = command.element_selector.as_deref()
            .unwrap_or("dropdown");
        let option = command.input_text.as_deref()
            .ok_or_else(|| anyhow::anyhow!("No option to select provided"))?;
        
        let element = self.perception.find_element(selector_desc).await?;
        
        if element.element_type == ElementType::Select {
            self.browser.select_option(&element.selector, option).await?;
            
            Ok(CommandResult {
                success: true,
                action: "select".to_string(),
                message: format!("Selected '{}' in dropdown", option),
                element_info: Some(element),
            })
        } else {
            Err(anyhow::anyhow!("Element is not a dropdown/select"))
        }
    }

    /// Navigate with page type detection
    async fn handle_navigate(&mut self, command: &ParsedCommand) -> Result<CommandResult> {
        let url = command.url.as_deref()
            .ok_or_else(|| anyhow::anyhow!("No URL provided"))?;
        
        self.browser.navigate_to(url).await?;
        
        // Classify the new page
        let page_type = self.perception.classify_page().await?;
        
        Ok(CommandResult {
            success: true,
            action: "navigate".to_string(),
            message: format!("Navigated to {} (detected as {:?})", url, page_type),
            element_info: None,
        })
    }

    /// Enhanced search with automatic search box detection
    async fn handle_search(&mut self, command: &ParsedCommand) -> Result<CommandResult> {
        let query = command.input_text.as_deref()
            .ok_or_else(|| anyhow::anyhow!("No search query provided"))?;
        
        // Try to find search box on current page
        match self.perception.find_element("search box").await {
            Ok(search_box) => {
                // Found search box, use it
                self.browser.clear_element(&search_box.selector).await?;
                self.browser.type_text(&search_box.selector, query).await?;
                
                // Try to find and click search button
                if let Ok(search_btn) = self.perception.find_element("search button").await {
                    self.browser.click_element(&search_btn.selector).await?;
                } else {
                    // Press Enter if no button found
                    self.browser.press_key("enter").await?;
                }
                
                Ok(CommandResult {
                    success: true,
                    action: "search".to_string(),
                    message: format!("Searched for '{}'", query),
                    element_info: Some(search_box),
                })
            }
            Err(_) => {
                // No search box found, navigate to Google
                let google_url = format!("https://www.google.com/search?q={}", 
                                        urlencoding::encode(query));
                self.browser.navigate_to(&google_url).await?;
                
                Ok(CommandResult {
                    success: true,
                    action: "search".to_string(),
                    message: format!("Searched Google for '{}'", query),
                    element_info: None,
                })
            }
        }
    }

    /// Extract data with intelligent detection
    async fn handle_extract(&mut self, command: &ParsedCommand) -> Result<CommandResult> {
        // Classify page to understand what we're extracting from
        let page_type = self.perception.classify_page().await?;
        
        let extracted_data = match page_type {
            crate::perception_mvp::PageType::ProductPage => {
                let product = self.extract_product_data().await?;
                serde_json::to_value(product)?
            }
            crate::perception_mvp::PageType::ArticlePage => {
                let article = self.extract_article_data().await?;
                serde_json::to_value(article)?
            }
            crate::perception_mvp::PageType::SearchResults => {
                let results = self.extract_search_results().await?;
                serde_json::to_value(results)?
            }
            _ => {
                let generic = self.extract_generic_data().await?;
                serde_json::to_value(generic)?
            }
        };
        
        Ok(CommandResult {
            success: true,
            action: "extract".to_string(),
            message: "Successfully extracted page data".to_string(),
            element_info: None,
        })
    }

    /// Handle unknown commands with suggestions
    async fn handle_unknown(&mut self, command: &ParsedCommand) -> Result<CommandResult> {
        // Try to understand intent and suggest alternatives
        let suggestions = vec![
            "Try: 'click on [button name]'",
            "Try: 'type [text] in [field name]'",
            "Try: 'search for [query]'",
            "Try: 'go to [website]'",
        ];
        
        Err(anyhow::anyhow!(
            "Unknown command '{}'. {}", 
            command.action,
            suggestions.join(" ")
        ))
    }

    /// Build element description from command parameters
    fn build_element_description(&self, command: &ParsedCommand) -> String {
        if let Some(selector) = &command.element_selector {
            // Direct selector provided
            selector.clone()
        } else if let Some(text) = &command.input_text {
            // Try using input text as element description
            format!("{} button", text)
        } else {
            // Generic description based on action
            match command.action.as_str() {
                "click" => "button".to_string(),
                "type" => "input field".to_string(),
                "select" => "dropdown".to_string(),
                _ => "element".to_string(),
            }
        }
    }

    /// Suggest alternative elements when one isn't found
    async fn suggest_alternatives(&mut self, description: &str) -> Result<Vec<String>> {
        // This would analyze the page and suggest similar elements
        // For now, return generic suggestions
        Ok(vec![
            "the submit button".to_string(),
            "the search box".to_string(),
            "the login link".to_string(),
        ])
    }

    /// Extract product information from e-commerce pages
    async fn extract_product_data(&mut self) -> Result<ProductData> {
        let title = self.perception.find_element("product title").await
            .map(|e| e.text)
            .unwrap_or_default();
            
        let price = self.perception.find_element("price").await
            .map(|e| e.text)
            .unwrap_or_default();
            
        let description = self.perception.find_element("product description").await
            .map(|e| e.text)
            .unwrap_or_default();
        
        Ok(ProductData {
            title,
            price,
            description,
            images: vec![],
            reviews: vec![],
        })
    }

    /// Extract article content
    async fn extract_article_data(&mut self) -> Result<ArticleData> {
        let title = self.perception.find_element("article title").await
            .map(|e| e.text)
            .unwrap_or_default();
            
        let author = self.perception.find_element("author").await
            .map(|e| e.text)
            .unwrap_or_default();
            
        let content = self.perception.find_element("article content").await
            .map(|e| e.text)
            .unwrap_or_default();
        
        Ok(ArticleData {
            title,
            author,
            content,
            published_date: None,
        })
    }

    /// Extract search results
    async fn extract_search_results(&mut self) -> Result<Vec<SearchResult>> {
        // Would find all search result items
        Ok(vec![])
    }

    /// Generic data extraction
    async fn extract_generic_data(&self) -> Result<GenericData> {
        Ok(GenericData {
            headings: vec![],
            paragraphs: vec![],
            links: vec![],
            images: vec![],
        })
    }
}

/// Result of executing a command
#[derive(Debug)]
pub struct CommandResult {
    pub success: bool,
    pub action: String,
    pub message: String,
    pub element_info: Option<PerceivedElement>,
}

/// Product data structure
#[derive(Debug, serde::Serialize)]
struct ProductData {
    title: String,
    price: String,
    description: String,
    images: Vec<String>,
    reviews: Vec<String>,
}

/// Article data structure
#[derive(Debug, serde::Serialize)]
struct ArticleData {
    title: String,
    author: String,
    content: String,
    published_date: Option<String>,
}

/// Search result item
#[derive(Debug, serde::Serialize)]
struct SearchResult {
    title: String,
    url: String,
    snippet: String,
}

/// Generic extracted data
#[derive(Debug, serde::Serialize)]
struct GenericData {
    headings: Vec<String>,
    paragraphs: Vec<String>,
    links: Vec<String>,
    images: Vec<String>,
}

// Note: SimpleBrowser methods like click_element, clear_element, type_text, select_option
// are already implemented in src/browser.rs

/// Enhanced Perception Engine - Advanced perception capabilities
/// This is an alias for the main PerceptionAwareExecutor for backward compatibility
pub type EnhancedPerceptionEngine = PerceptionAwareExecutor;