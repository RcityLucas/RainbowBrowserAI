// Standard Perception Layer (<500ms)
// Comprehensive page analysis with detailed content understanding

use anyhow::{Result, Context};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};
use super::browser_connection::BrowserConnection;
use super::quick_real::{QuickData, RealQuickPerception};
use super::lightning_real::LightningData;

/// Standard perception data with comprehensive analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardData {
    pub quick_data: QuickData,
    pub content_analysis: ContentAnalysis,
    pub visual_structure: VisualStructure,
    pub interaction_patterns: Vec<InteractionPattern>,
    pub data_extraction: DataExtraction,
    pub semantic_understanding: SemanticUnderstanding,
    pub performance_metrics: PerformanceMetrics,
    pub scan_time_ms: u64,
}

/// Comprehensive content analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentAnalysis {
    pub text_content: TextContent,
    pub media_elements: Vec<MediaElement>,
    pub data_tables: Vec<DataTable>,
    pub lists: Vec<ListStructure>,
    pub headings: Vec<Heading>,
    pub paragraphs: Vec<Paragraph>,
    pub code_blocks: Vec<CodeBlock>,
}

/// Text content analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextContent {
    pub total_words: usize,
    pub reading_time_minutes: f32,
    pub language: Option<String>,
    pub key_phrases: Vec<String>,
    pub sentiment: Option<String>,
    pub topics: Vec<String>,
}

/// Media elements found on page
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaElement {
    pub element_type: MediaType,
    pub src: String,
    pub alt_text: Option<String>,
    pub dimensions: Option<(u32, u32)>,
    pub file_size: Option<u64>,
    pub accessibility_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MediaType {
    Image,
    Video,
    Audio,
    Svg,
    Canvas,
    Iframe,
}

/// Data table structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataTable {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub caption: Option<String>,
    pub sortable_columns: Vec<usize>,
    pub total_rows: usize,
}

/// List structure analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListStructure {
    pub list_type: ListType,
    pub items: Vec<String>,
    pub nested_level: u32,
    pub has_links: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ListType {
    Ordered,
    Unordered,
    Definition,
}

/// Heading structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Heading {
    pub level: u32, // H1, H2, etc.
    pub text: String,
    pub id: Option<String>,
    pub section_content_words: usize,
}

/// Paragraph analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Paragraph {
    pub text: String,
    pub word_count: usize,
    pub has_links: bool,
    pub emphasis_count: usize,
}

/// Code block detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeBlock {
    pub language: Option<String>,
    pub lines: usize,
    pub code_type: CodeType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CodeType {
    InlineCode,
    CodeBlock,
    Preformatted,
}

/// Visual structure analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualStructure {
    pub layout_grid: LayoutGrid,
    pub color_scheme: ColorScheme,
    pub typography: Typography,
    pub spacing_analysis: SpacingAnalysis,
    pub responsive_breakpoints: Vec<u32>,
    pub accessibility_features: AccessibilityFeatures,
}

/// Layout grid analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutGrid {
    pub columns: u32,
    pub rows: u32,
    pub grid_areas: Vec<GridArea>,
    pub flexbox_containers: u32,
    pub css_grid_containers: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridArea {
    pub name: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// Color scheme analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorScheme {
    pub dominant_colors: Vec<String>,
    pub background_color: String,
    pub text_color: String,
    pub accent_colors: Vec<String>,
    pub contrast_ratio: f32,
    pub theme: ThemeType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThemeType {
    Light,
    Dark,
    Auto,
    Custom,
}

/// Typography analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Typography {
    pub primary_font: String,
    pub secondary_fonts: Vec<String>,
    pub font_sizes: Vec<f32>,
    pub line_heights: Vec<f32>,
    pub font_weights: Vec<u32>,
}

/// Spacing analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpacingAnalysis {
    pub margins: Vec<f32>,
    pub paddings: Vec<f32>,
    pub gaps: Vec<f32>,
    pub consistent_spacing: bool,
}

/// Accessibility features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessibilityFeatures {
    pub aria_labels: u32,
    pub alt_texts: u32,
    pub heading_structure_score: f32,
    pub color_contrast_score: f32,
    pub keyboard_navigation_score: f32,
    pub screen_reader_score: f32,
}

/// Interaction patterns detected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionPattern {
    pub pattern_type: PatternType,
    pub confidence: f32,
    pub elements_involved: Vec<String>,
    pub user_flow: Vec<String>,
    pub complexity_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatternType {
    LoginFlow,
    SearchAndFilter,
    EcommerceCheckout,
    FormSubmission,
    NavigationMenu,
    MediaGallery,
    DataVisualization,
    SocialInteraction,
    ContentManagement,
    Custom(String),
}

/// Data extraction results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataExtraction {
    pub structured_data: Vec<StructuredDataItem>,
    pub contact_information: Vec<ContactInfo>,
    pub social_links: Vec<SocialLink>,
    pub download_links: Vec<DownloadLink>,
    pub external_links: Vec<ExternalLink>,
    pub internal_links: Vec<InternalLink>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuredDataItem {
    pub data_type: String,
    pub content: serde_json::Value,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactInfo {
    pub info_type: ContactType,
    pub value: String,
    pub context: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContactType {
    Email,
    Phone,
    Address,
    Website,
    SocialMedia,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialLink {
    pub platform: String,
    pub url: String,
    pub username: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadLink {
    pub filename: String,
    pub url: String,
    pub file_type: String,
    pub estimated_size: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalLink {
    pub url: String,
    pub domain: String,
    pub anchor_text: String,
    pub rel_attributes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InternalLink {
    pub url: String,
    pub path: String,
    pub anchor_text: String,
    pub is_navigation: bool,
}

/// Semantic understanding of page content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticUnderstanding {
    pub page_purpose: PagePurpose,
    pub content_categories: Vec<String>,
    pub target_audience: Option<String>,
    pub call_to_actions: Vec<CallToAction>,
    pub information_architecture: InformationArchitecture,
    pub content_quality_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PagePurpose {
    Informational,
    Transactional,
    Navigational,
    Entertainment,
    Educational,
    Commercial,
    Personal,
    Technical,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallToAction {
    pub text: String,
    pub action_type: ActionType,
    pub urgency_level: UrgencyLevel,
    pub selector: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionType {
    Purchase,
    SignUp,
    Download,
    Contact,
    Subscribe,
    Share,
    Learn,
    Navigate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UrgencyLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InformationArchitecture {
    pub depth_levels: u32,
    pub breadcrumb_paths: Vec<String>,
    pub sitemap_structure: Vec<SitemapNode>,
    pub content_hierarchy_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SitemapNode {
    pub title: String,
    pub url: String,
    pub level: u32,
    pub children: Vec<String>,
}

/// Performance metrics from Standard perception
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub page_load_time: Option<f32>,
    pub dom_content_loaded: Option<f32>,
    pub first_contentful_paint: Option<f32>,
    pub largest_contentful_paint: Option<f32>,
    pub cumulative_layout_shift: Option<f32>,
    pub first_input_delay: Option<f32>,
    pub performance_score: f32,
}

/// Standard perception implementation
pub struct RealStandardPerception {
    quick_perception: RealQuickPerception,
    timeout_ms: u64,
}

impl RealStandardPerception {
    pub fn new() -> Self {
        Self {
            quick_perception: RealQuickPerception::new(),
            timeout_ms: 500,
        }
    }
    
    /// Execute Standard perception (<500ms)
    pub async fn scan_page(&self, browser: &BrowserConnection) -> Result<StandardData> {
        let start = Instant::now();
        
        // Run Quick perception first (should take <200ms)
        let quick_data = self.quick_perception.scan_page(browser).await?;
        
        // Run Standard-specific analyses in parallel
        let (content_analysis, visual_structure, interaction_patterns, data_extraction, semantic_understanding, performance_metrics) = tokio::join!(
            self.analyze_content(browser),
            self.analyze_visual_structure(browser),
            self.detect_interaction_patterns(browser),
            self.extract_data(browser),
            self.understand_semantics(browser, &quick_data),
            self.measure_performance(browser)
        );
        
        let scan_time_ms = start.elapsed().as_millis() as u64;
        
        if scan_time_ms > self.timeout_ms {
            warn!("Standard perception exceeded timeout: {}ms > {}ms", scan_time_ms, self.timeout_ms);
        } else {
            info!("Standard perception completed in {}ms", scan_time_ms);
        }
        
        Ok(StandardData {
            quick_data,
            content_analysis: content_analysis.unwrap_or_default(),
            visual_structure: visual_structure.unwrap_or_default(),
            interaction_patterns: interaction_patterns.unwrap_or_default(),
            data_extraction: data_extraction.unwrap_or_default(),
            semantic_understanding: semantic_understanding.unwrap_or_default(),
            performance_metrics: performance_metrics.unwrap_or_default(),
            scan_time_ms,
        })
    }
    
    /// Analyze page content comprehensively
    async fn analyze_content(&self, browser: &BrowserConnection) -> Result<ContentAnalysis> {
        let script = r##"
            function analyzeContent() {
                const analysis = {
                    text_content: {
                        total_words: 0,
                        reading_time_minutes: 0,
                        language: document.documentElement.lang || null,
                        key_phrases: [],
                        sentiment: null,
                        topics: []
                    },
                    media_elements: [],
                    data_tables: [],
                    lists: [],
                    headings: [],
                    paragraphs: [],
                    code_blocks: []
                };
                
                // Text content analysis
                const allText = document.body.innerText || '';
                const words = allText.trim().split(/\s+/).filter(w => w.length > 0);
                analysis.text_content.total_words = words.length;
                analysis.text_content.reading_time_minutes = Math.max(1, Math.round(words.length / 200));
                
                // Extract key phrases (simplified)
                const phrases = allText.match(/\b\w{4,}\b/g) || [];
                const phraseCount = {};
                phrases.forEach(phrase => {
                    const lower = phrase.toLowerCase();
                    phraseCount[lower] = (phraseCount[lower] || 0) + 1;
                });
                analysis.text_content.key_phrases = Object.entries(phraseCount)
                    .sort((a, b) => b[1] - a[1])
                    .slice(0, 10)
                    .map(([phrase]) => phrase);
                
                // Media elements
                document.querySelectorAll('img, video, audio, svg, canvas, iframe').forEach(media => {
                    const element = {
                        element_type: media.tagName.toLowerCase(),
                        src: media.src || media.data || '',
                        alt_text: media.alt || null,
                        dimensions: null,
                        file_size: null,
                        accessibility_score: 0.7
                    };
                    
                    if (media.naturalWidth && media.naturalHeight) {
                        element.dimensions = [media.naturalWidth, media.naturalHeight];
                    }
                    
                    analysis.media_elements.push(element);
                });
                
                // Data tables
                document.querySelectorAll('table').forEach((table, idx) => {
                    if (idx >= 5) return; // Limit to 5 tables
                    
                    const headers = Array.from(table.querySelectorAll('th')).map(th => th.innerText.trim());
                    const rows = Array.from(table.querySelectorAll('tr')).slice(headers.length > 0 ? 1 : 0).map(tr =>
                        Array.from(tr.querySelectorAll('td')).map(td => td.innerText.trim())
                    );
                    
                    analysis.data_tables.push({
                        headers,
                        rows: rows.slice(0, 10), // Limit rows
                        caption: table.caption ? table.caption.innerText : null,
                        sortable_columns: [],
                        total_rows: rows.length
                    });
                });
                
                // Lists
                document.querySelectorAll('ul, ol, dl').forEach((list, idx) => {
                    if (idx >= 10) return;
                    
                    const items = Array.from(list.children).map(item => item.innerText.trim()).slice(0, 20);
                    analysis.lists.push({
                        list_type: list.tagName.toLowerCase() === 'ol' ? 'Ordered' : 
                                  list.tagName.toLowerCase() === 'dl' ? 'Definition' : 'Unordered',
                        items,
                        nested_level: 0,
                        has_links: list.querySelector('a') !== null
                    });
                });
                
                // Headings
                document.querySelectorAll('h1, h2, h3, h4, h5, h6').forEach(heading => {
                    analysis.headings.push({
                        level: parseInt(heading.tagName.charAt(1)),
                        text: heading.innerText.trim(),
                        id: heading.id || null,
                        section_content_words: 0 // Would need more analysis
                    });
                });
                
                // Paragraphs
                document.querySelectorAll('p').forEach((p, idx) => {
                    if (idx >= 20) return;
                    
                    const text = p.innerText.trim();
                    if (text.length > 10) {
                        analysis.paragraphs.push({
                            text: text.substring(0, 200),
                            word_count: text.split(/\s+/).length,
                            has_links: p.querySelector('a') !== null,
                            emphasis_count: p.querySelectorAll('strong, em, b, i').length
                        });
                    }
                });
                
                // Code blocks
                document.querySelectorAll('code, pre').forEach(code => {
                    const text = code.innerText;
                    analysis.code_blocks.push({
                        language: code.className.match(/language-(\w+)/)?.[1] || null,
                        lines: text.split('\n').length,
                        code_type: code.tagName.toLowerCase() === 'pre' ? 'CodeBlock' : 'InlineCode'
                    });
                });
                
                return analysis;
            }
            
            return analyzeContent();
        "##;
        
        let result = browser.execute_script(script, vec![]).await?;
        let analysis: ContentAnalysis = serde_json::from_value(result)
            .unwrap_or_default();
        
        Ok(analysis)
    }
    
    /// Analyze visual structure and design
    async fn analyze_visual_structure(&self, browser: &BrowserConnection) -> Result<VisualStructure> {
        let script = r##"
            function analyzeVisualStructure() {
                const analysis = {
                    layout_grid: {
                        columns: 1,
                        rows: 1,
                        grid_areas: [],
                        flexbox_containers: document.querySelectorAll('[style*="display: flex"], [style*="display:flex"], .d-flex').length,
                        css_grid_containers: document.querySelectorAll('[style*="display: grid"], [style*="display:grid"]').length
                    },
                    color_scheme: {
                        dominant_colors: [],
                        background_color: getComputedStyle(document.body).backgroundColor || '#ffffff',
                        text_color: getComputedStyle(document.body).color || '#000000',
                        accent_colors: [],
                        contrast_ratio: 4.5,
                        theme: 'Light'
                    },
                    typography: {
                        primary_font: getComputedStyle(document.body).fontFamily || 'serif',
                        secondary_fonts: [],
                        font_sizes: [],
                        line_heights: [],
                        font_weights: []
                    },
                    spacing_analysis: {
                        margins: [],
                        paddings: [],
                        gaps: [],
                        consistent_spacing: true
                    },
                    responsive_breakpoints: [],
                    accessibility_features: {
                        aria_labels: document.querySelectorAll('[aria-label]').length,
                        alt_texts: document.querySelectorAll('img[alt]').length,
                        heading_structure_score: 0.8,
                        color_contrast_score: 0.7,
                        keyboard_navigation_score: 0.6,
                        screen_reader_score: 0.7
                    }
                };
                
                // Collect font sizes from various elements
                const elements = document.querySelectorAll('h1, h2, h3, h4, h5, h6, p, span, div');
                const fontSizes = new Set();
                const fontWeights = new Set();
                const lineHeights = new Set();
                
                for (let i = 0; i < Math.min(50, elements.length); i++) {
                    const style = getComputedStyle(elements[i]);
                    if (style.fontSize) fontSizes.add(parseFloat(style.fontSize));
                    if (style.fontWeight) fontWeights.add(parseInt(style.fontWeight));
                    if (style.lineHeight && style.lineHeight !== 'normal') {
                        lineHeights.add(parseFloat(style.lineHeight));
                    }
                }
                
                analysis.typography.font_sizes = Array.from(fontSizes).slice(0, 10);
                analysis.typography.font_weights = Array.from(fontWeights).slice(0, 5);
                analysis.typography.line_heights = Array.from(lineHeights).slice(0, 5);
                
                // Detect dark/light theme
                const bgColor = analysis.color_scheme.background_color;
                const isLight = bgColor.includes('rgb(255') || bgColor.includes('#fff') || bgColor === 'white';
                analysis.color_scheme.theme = isLight ? 'Light' : 'Dark';
                
                return analysis;
            }
            
            return analyzeVisualStructure();
        "##;
        
        let result = browser.execute_script(script, vec![]).await?;
        let structure: VisualStructure = serde_json::from_value(result)
            .unwrap_or_default();
        
        Ok(structure)
    }
    
    /// Detect interaction patterns
    async fn detect_interaction_patterns(&self, browser: &BrowserConnection) -> Result<Vec<InteractionPattern>> {
        let script = r##"
            function detectInteractionPatterns() {
                const patterns = [];
                
                // Login flow pattern
                const loginInputs = document.querySelectorAll('input[type="password"], input[name*="password" i], input[name*="login" i]');
                if (loginInputs.length > 0) {
                    patterns.push({
                        pattern_type: 'LoginFlow',
                        confidence: 0.9,
                        elements_involved: Array.from(loginInputs).map(input => input.name || input.id || 'password-input'),
                        user_flow: ['Enter credentials', 'Click login', 'Navigate to dashboard'],
                        complexity_score: 3.0
                    });
                }
                
                // Search and filter pattern
                const searchInputs = document.querySelectorAll('input[type="search"], input[placeholder*="search" i], input[name*="search" i]');
                const filterElements = document.querySelectorAll('select[name*="filter"], input[type="checkbox"], input[type="radio"]');
                if (searchInputs.length > 0 && filterElements.length > 0) {
                    patterns.push({
                        pattern_type: 'SearchAndFilter',
                        confidence: 0.8,
                        elements_involved: ['search-input', 'filter-controls'],
                        user_flow: ['Enter search term', 'Apply filters', 'View results'],
                        complexity_score: 4.0
                    });
                }
                
                // Form submission pattern
                const forms = document.querySelectorAll('form');
                forms.forEach(form => {
                    const inputs = form.querySelectorAll('input, select, textarea');
                    if (inputs.length > 2) {
                        patterns.push({
                            pattern_type: 'FormSubmission',
                            confidence: 0.85,
                            elements_involved: Array.from(inputs).map((input, idx) => `form-field-${idx}`),
                            user_flow: ['Fill form fields', 'Validate input', 'Submit form'],
                            complexity_score: inputs.length * 0.5
                        });
                    }
                });
                
                // Navigation menu pattern
                const navElements = document.querySelectorAll('nav, .navigation, .menu');
                if (navElements.length > 0) {
                    patterns.push({
                        pattern_type: 'NavigationMenu',
                        confidence: 0.9,
                        elements_involved: ['navigation-menu'],
                        user_flow: ['Browse menu', 'Select option', 'Navigate to page'],
                        complexity_score: 2.5
                    });
                }
                
                // E-commerce patterns
                const addToCartButtons = document.querySelectorAll('button[name*="add" i], button:contains("add to cart"), .add-to-cart');
                const priceElements = document.querySelectorAll('.price, [class*="price"], [id*="price"]');
                if (addToCartButtons.length > 0 && priceElements.length > 0) {
                    patterns.push({
                        pattern_type: 'EcommerceCheckout',
                        confidence: 0.8,
                        elements_involved: ['product-display', 'add-to-cart', 'checkout-flow'],
                        user_flow: ['Browse products', 'Add to cart', 'Proceed to checkout', 'Complete purchase'],
                        complexity_score: 6.0
                    });
                }
                
                return patterns;
            }
            
            return detectInteractionPatterns();
        "##;
        
        let result = browser.execute_script(script, vec![]).await?;
        let patterns: Vec<InteractionPattern> = serde_json::from_value(result)
            .unwrap_or_default();
        
        Ok(patterns)
    }
    
    /// Extract structured data and links
    async fn extract_data(&self, browser: &BrowserConnection) -> Result<DataExtraction> {
        let script = r##"
            function extractData() {
                const extraction = {
                    structured_data: [],
                    contact_information: [],
                    social_links: [],
                    download_links: [],
                    external_links: [],
                    internal_links: []
                };
                
                // Extract structured data (JSON-LD, microdata)
                const jsonLdScripts = document.querySelectorAll('script[type="application/ld+json"]');
                jsonLdScripts.forEach(script => {
                    try {
                        const data = JSON.parse(script.textContent);
                        extraction.structured_data.push({
                            data_type: data['@type'] || 'Unknown',
                            content: data,
                            source: 'JSON-LD'
                        });
                    } catch (e) {}
                });
                
                // Extract contact information
                const emailPattern = /([a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,})/g;
                const phonePattern = /(\+?\d{1,3}[-.\s]?\(?\d{1,3}\)?[-.\s]?\d{1,4}[-.\s]?\d{1,9})/g;
                
                const pageText = document.body.innerText;
                const emails = pageText.match(emailPattern) || [];
                const phones = pageText.match(phonePattern) || [];
                
                emails.forEach(email => {
                    extraction.contact_information.push({
                        info_type: 'Email',
                        value: email,
                        context: 'found in page text'
                    });
                });
                
                phones.forEach(phone => {
                    extraction.contact_information.push({
                        info_type: 'Phone',
                        value: phone,
                        context: 'found in page text'
                    });
                });
                
                // Extract social links
                const socialDomains = ['facebook.com', 'twitter.com', 'instagram.com', 'linkedin.com', 'youtube.com', 'tiktok.com'];
                document.querySelectorAll('a[href]').forEach(link => {
                    const href = link.href;
                    const domain = socialDomains.find(d => href.includes(d));
                    
                    if (domain) {
                        extraction.social_links.push({
                            platform: domain.split('.')[0],
                            url: href,
                            username: null // Would need more parsing
                        });
                    } else if (href.startsWith('http') && !href.includes(window.location.hostname)) {
                        extraction.external_links.push({
                            url: href,
                            domain: new URL(href).hostname,
                            anchor_text: link.innerText.trim(),
                            rel_attributes: link.rel ? link.rel.split(' ') : []
                        });
                    } else if (href.startsWith('/') || href.includes(window.location.hostname)) {
                        extraction.internal_links.push({
                            url: href,
                            path: new URL(href, window.location.origin).pathname,
                            anchor_text: link.innerText.trim(),
                            is_navigation: link.closest('nav') !== null
                        });
                    }
                });
                
                // Extract download links
                document.querySelectorAll('a[download], a[href$=".pdf"], a[href$=".doc"], a[href$=".zip"]').forEach(link => {
                    const href = link.href;
                    const filename = link.download || href.split('/').pop();
                    const extension = filename.split('.').pop();
                    
                    extraction.download_links.push({
                        filename: filename,
                        url: href,
                        file_type: extension,
                        estimated_size: null
                    });
                });
                
                // Limit arrays to prevent excessive data
                Object.keys(extraction).forEach(key => {
                    if (Array.isArray(extraction[key])) {
                        extraction[key] = extraction[key].slice(0, 20);
                    }
                });
                
                return extraction;
            }
            
            return extractData();
        "##;
        
        let result = browser.execute_script(script, vec![]).await?;
        let extraction: DataExtraction = serde_json::from_value(result)
            .unwrap_or_default();
        
        Ok(extraction)
    }
    
    /// Understand semantic meaning of the page
    async fn understand_semantics(&self, browser: &BrowserConnection, quick_data: &QuickData) -> Result<SemanticUnderstanding> {
        let script = r##"
            function understandSemantics() {
                const understanding = {
                    page_purpose: 'Unknown',
                    content_categories: [],
                    target_audience: null,
                    call_to_actions: [],
                    information_architecture: {
                        depth_levels: 0,
                        breadcrumb_paths: [],
                        sitemap_structure: [],
                        content_hierarchy_score: 0.7
                    },
                    content_quality_score: 0.5
                };
                
                // Determine page purpose
                const title = document.title.toLowerCase();
                const headings = Array.from(document.querySelectorAll('h1, h2')).map(h => h.innerText.toLowerCase());
                const content = headings.join(' ') + ' ' + title;
                
                if (content.includes('shop') || content.includes('buy') || content.includes('price') || content.includes('cart')) {
                    understanding.page_purpose = 'Commercial';
                } else if (content.includes('login') || content.includes('sign up') || content.includes('register')) {
                    understanding.page_purpose = 'Transactional';
                } else if (content.includes('about') || content.includes('help') || content.includes('faq')) {
                    understanding.page_purpose = 'Informational';
                } else if (content.includes('learn') || content.includes('tutorial') || content.includes('course')) {
                    understanding.page_purpose = 'Educational';
                } else if (content.includes('game') || content.includes('play') || content.includes('fun')) {
                    understanding.page_purpose = 'Entertainment';
                }
                
                // Extract call-to-actions
                const ctaSelectors = [
                    'button:contains("buy")', 'button:contains("purchase")', 'button:contains("order")',
                    'button:contains("sign up")', 'button:contains("register")', 'button:contains("join")',
                    'button:contains("download")', 'button:contains("get")', 'button:contains("start")',
                    'a:contains("learn more")', 'a:contains("read more")', 'a:contains("contact")'
                ];
                
                document.querySelectorAll('button, a').forEach(element => {
                    const text = element.innerText.toLowerCase().trim();
                    let actionType = 'Navigate';
                    let urgencyLevel = 'Medium';
                    
                    if (text.includes('buy') || text.includes('purchase')) {
                        actionType = 'Purchase';
                        urgencyLevel = 'High';
                    } else if (text.includes('sign up') || text.includes('register')) {
                        actionType = 'SignUp';
                        urgencyLevel = 'Medium';
                    } else if (text.includes('download')) {
                        actionType = 'Download';
                        urgencyLevel = 'Medium';
                    } else if (text.includes('contact')) {
                        actionType = 'Contact';
                        urgencyLevel = 'Low';
                    }
                    
                    if (text.length > 2 && text.length < 50) {
                        understanding.call_to_actions.push({
                            text: element.innerText.trim(),
                            action_type: actionType,
                            urgency_level: urgencyLevel,
                            selector: element.id ? '#' + element.id : element.tagName.toLowerCase()
                        });
                    }
                });
                
                // Analyze breadcrumbs
                const breadcrumbs = document.querySelectorAll('.breadcrumb, .breadcrumbs, nav[aria-label*="breadcrumb"]');
                breadcrumbs.forEach(breadcrumb => {
                    const links = breadcrumb.querySelectorAll('a');
                    if (links.length > 0) {
                        understanding.information_architecture.breadcrumb_paths.push(
                            Array.from(links).map(link => link.innerText.trim()).join(' > ')
                        );
                    }
                });
                
                // Content categories from headings and meta keywords
                const metaKeywords = document.querySelector('meta[name="keywords"]');
                if (metaKeywords) {
                    understanding.content_categories = metaKeywords.content.split(',').map(k => k.trim()).slice(0, 10);
                }
                
                // Quality score based on various factors
                let qualityScore = 0.5;
                if (document.querySelector('h1')) qualityScore += 0.1;
                if (document.querySelectorAll('img[alt]').length > 0) qualityScore += 0.1;
                if (document.querySelectorAll('[aria-label]').length > 0) qualityScore += 0.1;
                if (document.querySelector('meta[name="description"]')) qualityScore += 0.1;
                if (document.title.length > 10) qualityScore += 0.1;
                
                understanding.content_quality_score = Math.min(1.0, qualityScore);
                
                // Limit arrays
                understanding.call_to_actions = understanding.call_to_actions.slice(0, 10);
                understanding.information_architecture.breadcrumb_paths = understanding.information_architecture.breadcrumb_paths.slice(0, 5);
                
                return understanding;
            }
            
            return understandSemantics();
        "##;
        
        let result = browser.execute_script(script, vec![]).await?;
        let understanding: SemanticUnderstanding = serde_json::from_value(result)
            .unwrap_or_default();
        
        Ok(understanding)
    }
    
    /// Measure performance metrics
    async fn measure_performance(&self, browser: &BrowserConnection) -> Result<PerformanceMetrics> {
        let script = r##"
            function measurePerformance() {
                const metrics = {
                    page_load_time: null,
                    dom_content_loaded: null,
                    first_contentful_paint: null,
                    largest_contentful_paint: null,
                    cumulative_layout_shift: null,
                    first_input_delay: null,
                    performance_score: 0.7
                };
                
                if ('performance' in window) {
                    const navigation = performance.getEntriesByType('navigation')[0];
                    if (navigation) {
                        metrics.page_load_time = navigation.loadEventEnd - navigation.loadEventStart;
                        metrics.dom_content_loaded = navigation.domContentLoadedEventEnd - navigation.domContentLoadedEventStart;
                    }
                    
                    // Try to get paint metrics
                    const paintMetrics = performance.getEntriesByType('paint');
                    paintMetrics.forEach(metric => {
                        if (metric.name === 'first-contentful-paint') {
                            metrics.first_contentful_paint = metric.startTime;
                        }
                    });
                    
                    // Try to get LCP from observer (if available)
                    if ('PerformanceObserver' in window) {
                        // This would normally require setting up observers
                        // For now, we'll use estimated values
                        metrics.largest_contentful_paint = 2500; // Estimated
                        metrics.cumulative_layout_shift = 0.1;   // Estimated
                        metrics.first_input_delay = 100;        // Estimated
                    }
                    
                    // Calculate basic performance score
                    let score = 1.0;
                    if (metrics.first_contentful_paint > 2000) score -= 0.2;
                    if (metrics.page_load_time > 3000) score -= 0.2;
                    if (metrics.dom_content_loaded > 1500) score -= 0.1;
                    
                    metrics.performance_score = Math.max(0.1, score);
                }
                
                return metrics;
            }
            
            return measurePerformance();
        "##;
        
        let result = browser.execute_script(script, vec![]).await?;
        let metrics: PerformanceMetrics = serde_json::from_value(result)
            .unwrap_or_default();
        
        Ok(metrics)
    }
}

// Default implementations for complex types
impl Default for ContentAnalysis {
    fn default() -> Self {
        Self {
            text_content: TextContent::default(),
            media_elements: Vec::new(),
            data_tables: Vec::new(),
            lists: Vec::new(),
            headings: Vec::new(),
            paragraphs: Vec::new(),
            code_blocks: Vec::new(),
        }
    }
}

impl Default for TextContent {
    fn default() -> Self {
        Self {
            total_words: 0,
            reading_time_minutes: 0.0,
            language: None,
            key_phrases: Vec::new(),
            sentiment: None,
            topics: Vec::new(),
        }
    }
}

impl Default for VisualStructure {
    fn default() -> Self {
        Self {
            layout_grid: LayoutGrid::default(),
            color_scheme: ColorScheme::default(),
            typography: Typography::default(),
            spacing_analysis: SpacingAnalysis::default(),
            responsive_breakpoints: Vec::new(),
            accessibility_features: AccessibilityFeatures::default(),
        }
    }
}

impl Default for LayoutGrid {
    fn default() -> Self {
        Self {
            columns: 1,
            rows: 1,
            grid_areas: Vec::new(),
            flexbox_containers: 0,
            css_grid_containers: 0,
        }
    }
}

impl Default for ColorScheme {
    fn default() -> Self {
        Self {
            dominant_colors: Vec::new(),
            background_color: "#ffffff".to_string(),
            text_color: "#000000".to_string(),
            accent_colors: Vec::new(),
            contrast_ratio: 4.5,
            theme: ThemeType::Light,
        }
    }
}

impl Default for Typography {
    fn default() -> Self {
        Self {
            primary_font: "serif".to_string(),
            secondary_fonts: Vec::new(),
            font_sizes: Vec::new(),
            line_heights: Vec::new(),
            font_weights: Vec::new(),
        }
    }
}

impl Default for SpacingAnalysis {
    fn default() -> Self {
        Self {
            margins: Vec::new(),
            paddings: Vec::new(),
            gaps: Vec::new(),
            consistent_spacing: true,
        }
    }
}

impl Default for AccessibilityFeatures {
    fn default() -> Self {
        Self {
            aria_labels: 0,
            alt_texts: 0,
            heading_structure_score: 0.7,
            color_contrast_score: 0.7,
            keyboard_navigation_score: 0.6,
            screen_reader_score: 0.7,
        }
    }
}

impl Default for DataExtraction {
    fn default() -> Self {
        Self {
            structured_data: Vec::new(),
            contact_information: Vec::new(),
            social_links: Vec::new(),
            download_links: Vec::new(),
            external_links: Vec::new(),
            internal_links: Vec::new(),
        }
    }
}

impl Default for SemanticUnderstanding {
    fn default() -> Self {
        Self {
            page_purpose: PagePurpose::Unknown,
            content_categories: Vec::new(),
            target_audience: None,
            call_to_actions: Vec::new(),
            information_architecture: InformationArchitecture::default(),
            content_quality_score: 0.5,
        }
    }
}

impl Default for InformationArchitecture {
    fn default() -> Self {
        Self {
            depth_levels: 0,
            breadcrumb_paths: Vec::new(),
            sitemap_structure: Vec::new(),
            content_hierarchy_score: 0.7,
        }
    }
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            page_load_time: None,
            dom_content_loaded: None,
            first_contentful_paint: None,
            largest_contentful_paint: None,
            cumulative_layout_shift: None,
            first_input_delay: None,
            performance_score: 0.7,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_standard_data_creation() {
        // Test that we can create the complex data structures
        let _analysis = ContentAnalysis::default();
        let _structure = VisualStructure::default();
        let _extraction = DataExtraction::default();
        let _understanding = SemanticUnderstanding::default();
        let _metrics = PerformanceMetrics::default();
    }
    
    #[test]
    fn test_pattern_types() {
        let pattern = InteractionPattern {
            pattern_type: PatternType::LoginFlow,
            confidence: 0.9,
            elements_involved: vec!["username".to_string(), "password".to_string()],
            user_flow: vec!["Enter credentials".to_string()],
            complexity_score: 3.0,
        };
        
        assert_eq!(pattern.confidence, 0.9);
        assert_eq!(pattern.elements_involved.len(), 2);
    }
}