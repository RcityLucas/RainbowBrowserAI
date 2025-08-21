// Extract Links Tool - Phase 2 Week 8 Implementation
// 
// This tool specializes in extracting and analyzing links from web pages with comprehensive
// link categorization, validation, and detailed metadata extraction.

use crate::tools::{Tool, ToolError};
use super::{OutputFormat, ExtractionScope, ExtractionConfig, ExtractionResult, ExtractionMetadata, text_utils, format_utils};
use std::sync::Arc;
use std::collections::HashMap;
use thirtyfour::{WebDriver, By, WebElement};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::time::{Duration, Instant};
use chrono::Utc;
use url::{Url, ParseError};

/// Comprehensive link information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LinkInfo {
    /// Link URL (href attribute)
    pub url: String,
    
    /// Link text content
    pub text: String,
    
    /// Link title attribute
    pub title: Option<String>,
    
    /// Link target attribute (_blank, _self, etc.)
    pub target: Option<String>,
    
    /// Link relationship (rel attribute)
    pub rel: Option<String>,
    
    /// Link type/category
    pub link_type: LinkType,
    
    /// Link validation status
    pub validation: LinkValidation,
    
    /// Link position on page
    pub position: usize,
    
    /// CSS classes
    pub css_classes: Vec<String>,
    
    /// Link ID attribute
    pub id: Option<String>,
    
    /// Parent element context
    pub parent_context: Option<String>,
    
    /// Whether link has download attribute
    pub is_download: bool,
    
    /// Download filename if specified
    pub download_filename: Option<String>,
    
    /// Link bounding box
    pub bounding_box: Option<LinkBounds>,
    
    /// Additional attributes
    pub attributes: HashMap<String, String>,
}

/// Link type categories
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LinkType {
    /// Internal link (same domain)
    Internal,
    
    /// External link (different domain)
    External,
    
    /// Anchor link (same page)
    Anchor,
    
    /// Email link
    Email,
    
    /// Phone link
    Phone,
    
    /// Download link
    Download,
    
    /// Social media link
    Social,
    
    /// Navigation link
    Navigation,
    
    /// Footer link
    Footer,
    
    /// Image link
    Image,
    
    /// JavaScript link
    JavaScript,
    
    /// Unknown/other
    Unknown,
}

/// Link validation information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LinkValidation {
    /// URL validation status
    pub is_valid_url: bool,
    
    /// URL parse error if invalid
    pub url_error: Option<String>,
    
    /// Whether URL is absolute
    pub is_absolute: bool,
    
    /// Whether URL is relative
    pub is_relative: bool,
    
    /// Parsed URL components
    pub url_components: Option<UrlComponents>,
    
    /// Link accessibility status
    pub accessibility: LinkAccessibility,
}

/// Parsed URL components
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UrlComponents {
    /// URL scheme (http, https, ftp, etc.)
    pub scheme: String,
    
    /// Domain/host
    pub host: Option<String>,
    
    /// Port number
    pub port: Option<u16>,
    
    /// URL path
    pub path: String,
    
    /// Query parameters
    pub query: Option<String>,
    
    /// Fragment/anchor
    pub fragment: Option<String>,
}

/// Link accessibility information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LinkAccessibility {
    /// Whether link has accessible text
    pub has_accessible_text: bool,
    
    /// aria-label attribute
    pub aria_label: Option<String>,
    
    /// aria-describedby attribute
    pub aria_describedby: Option<String>,
    
    /// Whether link opens in new window/tab
    pub opens_new_window: bool,
    
    /// Link role attribute
    pub role: Option<String>,
}

/// Link bounding box
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LinkBounds {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// Link extraction configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkExtractionConfig {
    /// Whether to include anchor links (#fragment)
    pub include_anchors: bool,
    
    /// Whether to include email links (mailto:)
    pub include_email: bool,
    
    /// Whether to include phone links (tel:)
    pub include_phone: bool,
    
    /// Whether to include JavaScript links (javascript:)
    pub include_javascript: bool,
    
    /// Whether to validate URLs
    pub validate_urls: bool,
    
    /// Whether to categorize links
    pub categorize_links: bool,
    
    /// Whether to extract positioning information
    pub include_positioning: bool,
    
    /// Whether to analyze accessibility
    pub analyze_accessibility: bool,
    
    /// Minimum link text length to include
    pub min_text_length: usize,
    
    /// Maximum number of links to extract (0 = no limit)
    pub max_links: usize,
    
    /// Domains to consider as internal (in addition to current domain)
    pub internal_domains: Vec<String>,
    
    /// Link text patterns to exclude
    pub exclude_patterns: Vec<String>,
}

impl Default for LinkExtractionConfig {
    fn default() -> Self {
        Self {
            include_anchors: true,
            include_email: true,
            include_phone: true,
            include_javascript: false,
            validate_urls: true,
            categorize_links: true,
            include_positioning: false,
            analyze_accessibility: true,
            min_text_length: 0,
            max_links: 0,
            internal_domains: Vec::new(),
            exclude_patterns: Vec::new(),
        }
    }
}

/// Input parameters for extract_links tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractLinksInput {
    /// CSS selector for link elements (optional, defaults to "a[href]")
    pub link_selector: Option<String>,
    
    /// Extraction configuration
    pub config: ExtractionConfig,
    
    /// Link-specific configuration
    pub link_config: LinkExtractionConfig,
}

impl Default for ExtractLinksInput {
    fn default() -> Self {
        Self {
            link_selector: None,
            config: ExtractionConfig::default(),
            link_config: LinkExtractionConfig::default(),
        }
    }
}

/// Output from extract_links tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractLinksOutput {
    /// Extracted link data
    pub links: Vec<LinkInfo>,
    
    /// Formatted output in requested format
    pub formatted_output: String,
    
    /// Total number of links extracted
    pub link_count: usize,
    
    /// Link statistics
    pub statistics: LinkStatistics,
    
    /// Extraction configuration used
    pub config: ExtractionConfig,
    
    /// Link configuration used
    pub link_config: LinkExtractionConfig,
}

/// Link extraction statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkStatistics {
    /// Count by link type
    pub by_type: HashMap<String, usize>,
    
    /// Count of internal vs external links
    pub internal_count: usize,
    pub external_count: usize,
    
    /// Count of valid vs invalid URLs
    pub valid_urls: usize,
    pub invalid_urls: usize,
    
    /// Count of accessible vs inaccessible links
    pub accessible_links: usize,
    pub inaccessible_links: usize,
    
    /// Unique domains found
    pub unique_domains: Vec<String>,
}

/// Extract links tool implementation
pub struct ExtractLinks {
    driver: Arc<WebDriver>,
}

impl ExtractLinks {
    /// Create a new extract links tool
    pub fn new(driver: Arc<WebDriver>) -> Self {
        Self { driver }
    }
    
    /// Find link elements using selector
    async fn find_links(&self, selector: &str) -> anyhow::Result<Vec<WebElement>> {
        Ok(self.driver.find_all(By::Css(selector)).await?)
    }
    
    /// Extract information from a single link
    async fn extract_link_info(&self, element: &WebElement, position: usize, base_url: &str, input: &ExtractLinksInput) -> anyhow::Result<Option<LinkInfo>> {
        let url = match element.attr("href").await? {
            Some(href) => href,
            None => return Ok(None), // Skip links without href
        };
        
        let text = text_utils::clean_text(&element.text().await?);
        let title = element.attr("title").await?;
        let target = element.attr("target").await?;
        let rel = element.attr("rel").await?;
        let id = element.attr("id").await?;
        let is_download = element.attr("download").await?.is_some();
        let download_filename = element.attr("download").await?;
        let css_classes = self.get_css_classes(element).await?;
        
        // Check exclusion patterns
        if self.should_exclude_link(&text, &url, input) {
            return Ok(None);
        }
        
        // Check minimum text length
        if input.link_config.min_text_length > 0 && text.len() < input.link_config.min_text_length {
            return Ok(None);
        }
        
        // Validate and categorize URL
        let validation = if input.link_config.validate_urls {
            self.validate_url(&url, base_url)?
        } else {
            LinkValidation::default()
        };
        
        let link_type = if input.link_config.categorize_links {
            self.categorize_link(&url, &text, element, &validation, base_url).await?
        } else {
            LinkType::Unknown
        };
        
        // Analyze accessibility
        let accessibility = if input.link_config.analyze_accessibility {
            self.analyze_link_accessibility(element, &text).await?
        } else {
            LinkAccessibility::default()
        };
        
        let validation = LinkValidation {
            accessibility,
            ..validation
        };
        
        // Get parent context
        let parent_context = self.get_parent_context(element).await?;
        
        // Get bounding box if requested
        let bounding_box = if input.link_config.include_positioning {
            self.get_link_bounds(element).await?
        } else {
            None
        };
        
        // Extract additional attributes
        let attributes = self.extract_additional_attributes(element).await?;
        
        Ok(Some(LinkInfo {
            url,
            text,
            title,
            target,
            rel,
            link_type,
            validation,
            position,
            css_classes,
            id,
            parent_context,
            is_download,
            download_filename,
            bounding_box,
            attributes,
        }))
    }
    
    /// Validate URL and extract components
    fn validate_url(&self, url: &str, base_url: &str) -> anyhow::Result<LinkValidation> {
        let mut validation = LinkValidation::default();
        
        // Try to parse the URL
        match Url::parse(url) {
            Ok(parsed_url) => {
                validation.is_valid_url = true;
                validation.is_absolute = true;
                validation.is_relative = false;
                validation.url_components = Some(UrlComponents {
                    scheme: parsed_url.scheme().to_string(),
                    host: parsed_url.host_str().map(|s| s.to_string()),
                    port: parsed_url.port(),
                    path: parsed_url.path().to_string(),
                    query: parsed_url.query().map(|s| s.to_string()),
                    fragment: parsed_url.fragment().map(|s| s.to_string()),
                });
            }
            Err(_) => {
                // Try to parse as relative URL
                if let Ok(base) = Url::parse(base_url) {
                    match base.join(url) {
                        Ok(joined_url) => {
                            validation.is_valid_url = true;
                            validation.is_absolute = false;
                            validation.is_relative = true;
                            validation.url_components = Some(UrlComponents {
                                scheme: joined_url.scheme().to_string(),
                                host: joined_url.host_str().map(|s| s.to_string()),
                                port: joined_url.port(),
                                path: joined_url.path().to_string(),
                                query: joined_url.query().map(|s| s.to_string()),
                                fragment: joined_url.fragment().map(|s| s.to_string()),
                            });
                        }
                        Err(e) => {
                            validation.is_valid_url = false;
                            validation.url_error = Some(e.to_string());
                        }
                    }
                } else {
                    validation.is_valid_url = false;
                    validation.url_error = Some("Invalid base URL".to_string());
                }
            }
        }
        
        Ok(validation)
    }
    
    /// Categorize link based on URL and context
    async fn categorize_link(&self, url: &str, text: &str, element: &WebElement, validation: &LinkValidation, base_url: &str) -> anyhow::Result<LinkType> {
        // Check for specific protocols
        if url.starts_with("mailto:") {
            return Ok(LinkType::Email);
        }
        if url.starts_with("tel:") {
            return Ok(LinkType::Phone);
        }
        if url.starts_with("javascript:") {
            return Ok(LinkType::JavaScript);
        }
        
        // Check for anchor links
        if url.starts_with('#') {
            return Ok(LinkType::Anchor);
        }
        
        // Check for download links
        if element.attr("download").await?.is_some() {
            return Ok(LinkType::Download);
        }
        
        // Check if link contains an image
        if element.find(By::Css("img")).await.is_ok() {
            return Ok(LinkType::Image);
        }
        
        // Check for social media links
        if self.is_social_media_link(url) {
            return Ok(LinkType::Social);
        }
        
        // Check context-based categorization
        if let Ok(parent) = element.find(By::XPath("ancestor::nav[1]")).await {
            return Ok(LinkType::Navigation);
        }
        
        if let Ok(parent) = element.find(By::XPath("ancestor::footer[1]")).await {
            return Ok(LinkType::Footer);
        }
        
        // Determine if internal or external
        if let Some(components) = &validation.url_components {
            if let Some(host) = &components.host {
                if let Ok(parsed_base_url) = Url::parse(base_url) {
                    if let Some(base_host) = parsed_base_url.host_str() {
                        if host == base_host {
                            return Ok(LinkType::Internal);
                        } else {
                            return Ok(LinkType::External);
                        }
                    }
                }
            }
        }
        
        Ok(LinkType::Unknown)
    }
    
    /// Check if URL is a social media link
    fn is_social_media_link(&self, url: &str) -> bool {
        let social_domains = [
            "facebook.com", "twitter.com", "x.com", "instagram.com", "linkedin.com",
            "youtube.com", "tiktok.com", "snapchat.com", "pinterest.com", "reddit.com",
            "discord.com", "whatsapp.com", "telegram.org", "github.com"
        ];
        
        social_domains.iter().any(|domain| url.contains(domain))
    }
    
    /// Analyze link accessibility
    async fn analyze_link_accessibility(&self, element: &WebElement, text: &str) -> anyhow::Result<LinkAccessibility> {
        let aria_label = element.attr("aria-label").await?;
        let aria_describedby = element.attr("aria-describedby").await?;
        let role = element.attr("role").await?;
        let target = element.attr("target").await?;
        
        let has_accessible_text = !text.trim().is_empty() || aria_label.is_some();
        let opens_new_window = target.as_deref() == Some("_blank");
        
        Ok(LinkAccessibility {
            has_accessible_text,
            aria_label,
            aria_describedby,
            opens_new_window,
            role,
        })
    }
    
    /// Get parent element context
    async fn get_parent_context(&self, element: &WebElement) -> anyhow::Result<Option<String>> {
        // Try to get meaningful parent context
        let parent_selectors = [
            "nav", "header", "footer", "main", "article", "section", "div[class]", "ul", "ol"
        ];
        
        for selector in &parent_selectors {
            if let Ok(parent) = element.find(By::XPath(&format!("ancestor::{}[1]", selector))).await {
                if let Ok(tag_name) = parent.tag_name().await {
                    let mut context = tag_name;
                    if let Ok(Some(class)) = parent.attr("class").await {
                        context.push_str(&format!(".{}", class.split_whitespace().next().unwrap_or("")));
                    }
                    if let Ok(Some(id)) = parent.attr("id").await {
                        context.push_str(&format!("#{}", id));
                    }
                    return Ok(Some(context));
                }
            }
        }
        
        Ok(None)
    }
    
    /// Get link bounding box
    async fn get_link_bounds(&self, element: &WebElement) -> anyhow::Result<Option<LinkBounds>> {
        if let Ok(rect) = element.rect().await {
            Ok(Some(LinkBounds {
                x: rect.x,
                y: rect.y,
                width: rect.width,
                height: rect.height,
            }))
        } else {
            Ok(None)
        }
    }
    
    /// Extract additional attributes
    async fn extract_additional_attributes(&self, element: &WebElement) -> anyhow::Result<HashMap<String, String>> {
        let mut attributes = HashMap::new();
        
        // Common link attributes to extract
        let attribute_names = [
            "data-*", "hreflang", "type", "ping", "referrerpolicy", "crossorigin"
        ];
        
        // Note: WebDriver doesn't have a direct way to get all attributes
        // This is a simplified version - in a real implementation, you might
        // use JavaScript to get all attributes
        
        Ok(attributes)
    }
    
    /// Get CSS classes from element
    async fn get_css_classes(&self, element: &WebElement) -> anyhow::Result<Vec<String>> {
        let class_attr = element.attr("class").await?.unwrap_or_default();
        if class_attr.trim().is_empty() {
            Ok(Vec::new())
        } else {
            Ok(class_attr.split_whitespace().map(|s| s.to_string()).collect())
        }
    }
    
    /// Check if link should be excluded based on patterns
    fn should_exclude_link(&self, text: &str, url: &str, input: &ExtractLinksInput) -> bool {
        // Check URL protocol exclusions
        if !input.link_config.include_anchors && url.starts_with('#') {
            return true;
        }
        if !input.link_config.include_email && url.starts_with("mailto:") {
            return true;
        }
        if !input.link_config.include_phone && url.starts_with("tel:") {
            return true;
        }
        if !input.link_config.include_javascript && url.starts_with("javascript:") {
            return true;
        }
        
        // Check text exclusion patterns
        for pattern in &input.link_config.exclude_patterns {
            if text.contains(pattern) || url.contains(pattern) {
                return true;
            }
        }
        
        false
    }
    
    /// Generate link statistics
    fn generate_statistics(&self, links: &[LinkInfo]) -> LinkStatistics {
        let mut by_type = HashMap::new();
        let mut unique_domains = std::collections::HashSet::new();
        let mut internal_count = 0;
        let mut external_count = 0;
        let mut valid_urls = 0;
        let mut invalid_urls = 0;
        let mut accessible_links = 0;
        let mut inaccessible_links = 0;
        
        for link in links {
            // Count by type
            let type_name = format!("{:?}", link.link_type).to_lowercase();
            *by_type.entry(type_name).or_insert(0) += 1;
            
            // Count internal/external
            match link.link_type {
                LinkType::Internal => internal_count += 1,
                LinkType::External => external_count += 1,
                _ => {}
            }
            
            // Count valid/invalid URLs
            if link.validation.is_valid_url {
                valid_urls += 1;
            } else {
                invalid_urls += 1;
            }
            
            // Count accessible links
            if link.validation.accessibility.has_accessible_text {
                accessible_links += 1;
            } else {
                inaccessible_links += 1;
            }
            
            // Collect unique domains
            if let Some(components) = &link.validation.url_components {
                if let Some(host) = &components.host {
                    unique_domains.insert(host.clone());
                }
            }
        }
        
        LinkStatistics {
            by_type,
            internal_count,
            external_count,
            valid_urls,
            invalid_urls,
            accessible_links,
            inaccessible_links,
            unique_domains: unique_domains.into_iter().collect(),
        }
    }
}

impl Default for LinkValidation {
    fn default() -> Self {
        Self {
            is_valid_url: false,
            url_error: None,
            is_absolute: false,
            is_relative: false,
            url_components: None,
            accessibility: LinkAccessibility::default(),
        }
    }
}

impl Default for LinkAccessibility {
    fn default() -> Self {
        Self {
            has_accessible_text: false,
            aria_label: None,
            aria_describedby: None,
            opens_new_window: false,
            role: None,
        }
    }
}

#[async_trait]
impl Tool for ExtractLinks {
    type Input = ExtractLinksInput;
    type Output = ExtractLinksOutput;

    fn name(&self) -> &str {
        "extract_links"
    }

    fn description(&self) -> &str {
        "Extract and analyze links from web pages with comprehensive categorization and validation"
    }

    async fn execute(&self, input: Self::Input) -> anyhow::Result<Self::Output> {
        let start_time = Instant::now();
        
        // Get current URL for relative link resolution
        let base_url = self.driver.current_url().await?.to_string();
        
        // Determine link selector
        let link_selector = input.link_selector
            .as_deref()
            .unwrap_or("a[href]");
        
        // Find links
        let link_elements = self.find_links(link_selector).await?;
        
        if link_elements.is_empty() {
            return Err(anyhow::anyhow!("No links found with selector '{}'", link_selector));
        }
        
        let mut links = Vec::new();
        
        // Extract data from links
        let max_links = if input.link_config.max_links > 0 {
            input.link_config.max_links
        } else {
            link_elements.len()
        };
        
        for (position, link_element) in link_elements.iter().enumerate().take(max_links) {
            if let Some(link_info) = self.extract_link_info(link_element, position, &base_url, &input).await? {
                links.push(link_info);
            }
        }
        
        // Generate statistics
        let statistics = self.generate_statistics(&links);
        
        // Create extraction metadata
        let metadata = if input.config.include_metadata {
            let mut tool_metadata = HashMap::new();
            tool_metadata.insert("link_selector".to_string(), serde_json::Value::String(link_selector.to_string()));
            tool_metadata.insert("base_url".to_string(), serde_json::Value::String(base_url.clone()));
            tool_metadata.insert("validate_urls".to_string(), serde_json::Value::Bool(input.link_config.validate_urls));
            tool_metadata.insert("categorize_links".to_string(), serde_json::Value::Bool(input.link_config.categorize_links));
            
            Some(ExtractionMetadata {
                url: base_url,
                timestamp: Utc::now(),
                item_count: links.len(),
                duration_ms: start_time.elapsed().as_millis() as u64,
                scope: input.config.scope.clone(),
                tool_name: self.name().to_string(),
                tool_metadata,
            })
        } else {
            None
        };
        
        // Create result structure for formatting
        let result_data = ExtractionResult::success(links.clone(), metadata.clone());
        
        // Format output
        let formatted_output = format_utils::format_output(&result_data, &input.config.format)
            .map_err(|e| anyhow::anyhow!("Failed to format output: {}", e))?;
        
        Ok(ExtractLinksOutput {
            links,
            formatted_output,
            link_count: result_data.data.len(),
            statistics,
            config: input.config,
            link_config: input.link_config,
        })
    }
}

// Implementation checklist for extract_links tool:
// [x] Define comprehensive link data structures (LinkInfo, LinkValidation, LinkAccessibility)
// [x] Implement link type categorization with intelligent detection
// [x] Add URL validation and component parsing
// [x] Implement link accessibility analysis
// [x] Create parent context detection for link categorization
// [x] Add link positioning and bounding box information
// [x] Implement comprehensive link filtering and exclusion patterns
// [x] Add social media link detection
// [x] Create link statistics generation and analysis
// [ ] Add CLI integration in main.rs
// [ ] Create specialized link analysis report formatting
// [ ] Add broken link detection (HTTP status checking)
// [ ] Create unit tests and integration tests
// [ ] Add link graph analysis (internal link structure)

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_link_validation_url_parsing() {
        let tool = ExtractLinks { driver: Arc::new(unsafe { std::mem::zeroed() }) };
        let base_url = "https://example.com";
        
        // Test valid absolute URL
        let validation = tool.validate_url("https://google.com", base_url).unwrap();
        assert!(validation.is_valid_url);
        assert!(validation.is_absolute);
        assert!(!validation.is_relative);
        
        // Test valid relative URL
        let validation = tool.validate_url("/path/to/page", base_url).unwrap();
        assert!(validation.is_valid_url);
        assert!(!validation.is_absolute);
        assert!(validation.is_relative);
    }
    
    #[test]
    fn test_social_media_detection() {
        let tool = ExtractLinks { driver: Arc::new(unsafe { std::mem::zeroed() }) };
        
        assert!(tool.is_social_media_link("https://facebook.com/profile"));
        assert!(tool.is_social_media_link("https://twitter.com/user"));
        assert!(tool.is_social_media_link("https://github.com/repo"));
        assert!(!tool.is_social_media_link("https://google.com"));
    }
    
    #[test]
    fn test_link_exclusion_logic() {
        let tool = ExtractLinks { driver: Arc::new(unsafe { std::mem::zeroed() }) };
        let mut input = ExtractLinksInput::default();
        input.link_config.include_anchors = false;
        input.link_config.include_email = false;
        input.link_config.exclude_patterns = vec!["admin".to_string()];
        
        assert!(tool.should_exclude_link("", "#section", &input));
        assert!(tool.should_exclude_link("", "mailto:test@example.com", &input));
        assert!(tool.should_exclude_link("Admin Panel", "https://example.com/admin", &input));
        assert!(!tool.should_exclude_link("Home", "https://example.com", &input));
    }
    
    #[test]
    fn test_link_extraction_config_default() {
        let config = LinkExtractionConfig::default();
        assert!(config.include_anchors);
        assert!(config.include_email);
        assert!(config.validate_urls);
        assert!(config.categorize_links);
        assert_eq!(config.min_text_length, 0);
        assert_eq!(config.max_links, 0);
    }
}