// Quick Perception - Real Implementation (<200ms)
// Builds on Lightning perception with more detailed analysis

use anyhow::{Result, Context};
use serde::{Serialize, Deserialize};
use std::time::{Duration, Instant};
use std::collections::HashMap;
use tracing::{debug, warn, info};
use super::browser_connection::BrowserConnection;
use super::lightning_real::{LightningData, RealLightningPerception};
use super::ElementType;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuickData {
    pub lightning_data: LightningData,
    pub interaction_elements: Vec<InteractionElement>,
    pub layout_structure: LayoutInfo,
    pub navigation_paths: Vec<NavPath>,
    pub form_analysis: Vec<FormInfo>,
    pub scan_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionElement {
    pub selector: String,
    pub element_type: ElementType,
    pub text: String,
    pub attributes: HashMap<String, String>,
    pub interaction_type: InteractionType,
    pub confidence: f32,
    pub bounding_box: BoundingBox,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InteractionType {
    Clickable,
    Typeable,
    Selectable,
    Draggable,
    Scrollable,
    Hoverable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundingBox {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutInfo {
    pub has_header: bool,
    pub has_navigation: bool,
    pub has_sidebar: bool,
    pub has_footer: bool,
    pub main_content_selector: Option<String>,
    pub layout_type: LayoutType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LayoutType {
    SingleColumn,
    TwoColumn,
    ThreeColumn,
    Grid,
    Dashboard,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavPath {
    pub text: String,
    pub url: String,
    pub depth: u32,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormInfo {
    pub form_selector: String,
    pub fields: Vec<FormField>,
    pub submit_button: Option<String>,
    pub method: String,
    pub action: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormField {
    pub name: String,
    pub field_type: String,
    pub label: Option<String>,
    pub required: bool,
    pub current_value: Option<String>,
}

pub struct RealQuickPerception {
    lightning_perception: RealLightningPerception,
    timeout_ms: u64,
}

impl RealQuickPerception {
    pub fn new() -> Self {
        Self {
            lightning_perception: RealLightningPerception::new(),
            timeout_ms: 200,
        }
    }
    
    /// Execute Quick perception (<200ms)
    pub async fn scan_page(&self, browser: &BrowserConnection) -> Result<QuickData> {
        let start = Instant::now();
        
        // Run Lightning perception first (should take <50ms)
        let lightning_data = self.lightning_perception.scan_page(browser).await?;
        
        // Run additional Quick analyses in parallel
        let (interaction_elements, layout_structure, navigation_paths, form_analysis) = tokio::join!(
            self.analyze_interactions(browser),
            self.analyze_layout(browser),
            self.extract_navigation(browser),
            self.analyze_forms(browser)
        );
        
        let scan_time_ms = start.elapsed().as_millis() as u64;
        
        if scan_time_ms > self.timeout_ms {
            warn!("Quick perception exceeded timeout: {}ms > {}ms", scan_time_ms, self.timeout_ms);
        } else {
            info!("Quick perception completed in {}ms", scan_time_ms);
        }
        
        Ok(QuickData {
            lightning_data,
            interaction_elements: interaction_elements.unwrap_or_default(),
            layout_structure: layout_structure.unwrap_or(LayoutInfo {
                has_header: false,
                has_navigation: false,
                has_sidebar: false,
                has_footer: false,
                main_content_selector: None,
                layout_type: LayoutType::Unknown,
            }),
            navigation_paths: navigation_paths.unwrap_or_default(),
            form_analysis: form_analysis.unwrap_or_default(),
            scan_time_ms,
        })
    }
    
    /// Analyze interactive elements with more detail
    async fn analyze_interactions(&self, browser: &BrowserConnection) -> Result<Vec<InteractionElement>> {
        let script = r##"
            function analyzeInteractions() {
                const elements = [];
                const interactiveSelectors = [
                    'button',
                    'a[href]',
                    'input',
                    'select',
                    'textarea',
                    '[onclick]',
                    '[role="button"]',
                    '[role="link"]',
                    '[draggable="true"]',
                    '[contenteditable="true"]'
                ];
                
                let count = 0;
                const maxElements = 30;
                
                for (const selector of interactiveSelectors) {
                    if (count >= maxElements) break;
                    
                    const found = document.querySelectorAll(selector);
                    for (const el of found) {
                        if (count >= maxElements) break;
                        
                        const rect = el.getBoundingClientRect();
                        if (rect.width === 0 || rect.height === 0) continue;
                        
                        // Determine interaction type
                        let interactionType = 'Hoverable';
                        if (el.matches('input, textarea, select, [contenteditable="true"]')) {
                            interactionType = el.matches('select') ? 'Selectable' : 'Typeable';
                        } else if (el.matches('button, a, [onclick], [role="button"], [role="link"]')) {
                            interactionType = 'Clickable';
                        } else if (el.draggable) {
                            interactionType = 'Draggable';
                        }
                        
                        // Generate unique selector
                        let uniqueSelector = '';
                        if (el.id) {
                            uniqueSelector = '#' + el.id;
                        } else if (el.className) {
                            uniqueSelector = el.tagName.toLowerCase() + '.' + 
                                           el.className.split(' ').filter(c => c).join('.');
                        } else {
                            uniqueSelector = el.tagName.toLowerCase();
                            if (el.name) uniqueSelector += '[name="' + el.name + '"]';
                        }
                        
                        // Collect attributes
                        const attributes = {};
                        ['href', 'type', 'name', 'placeholder', 'aria-label', 'title'].forEach(attr => {
                            if (el.hasAttribute(attr)) {
                                attributes[attr] = el.getAttribute(attr);
                            }
                        });
                        
                        elements.push({
                            selector: uniqueSelector,
                            element_type: el.tagName.toLowerCase(),
                            text: (el.innerText || el.value || '').substring(0, 100),
                            attributes: attributes,
                            interaction_type: interactionType,
                            confidence: 0.8,
                            bounding_box: {
                                x: rect.x,
                                y: rect.y,
                                width: rect.width,
                                height: rect.height
                            }
                        });
                        
                        count++;
                    }
                }
                
                return elements;
            }
            
            return analyzeInteractions();
        "##;
        
        let result = browser.execute_script(script, vec![]).await?;
        let elements: Vec<InteractionElement> = serde_json::from_value(result)
            .unwrap_or_else(|_| Vec::new());
        
        Ok(elements)
    }
    
    /// Analyze page layout structure
    async fn analyze_layout(&self, browser: &BrowserConnection) -> Result<LayoutInfo> {
        let script = r##"
            function analyzeLayout() {
                // Common layout selectors
                const hasHeader = document.querySelector('header, [role="banner"], .header, #header') !== null;
                const hasNavigation = document.querySelector('nav, [role="navigation"], .navigation, .nav') !== null;
                const hasSidebar = document.querySelector('aside, [role="complementary"], .sidebar, #sidebar') !== null;
                const hasFooter = document.querySelector('footer, [role="contentinfo"], .footer, #footer') !== null;
                
                // Find main content area
                let mainContentSelector = null;
                const mainSelectors = ['main', '[role="main"]', '#main', '.main', '#content', '.content'];
                for (const selector of mainSelectors) {
                    if (document.querySelector(selector)) {
                        mainContentSelector = selector;
                        break;
                    }
                }
                
                // Determine layout type
                let layoutType = 'Unknown';
                const bodyWidth = document.body.offsetWidth;
                
                // Check for grid layout
                const gridContainers = document.querySelectorAll('[style*="display: grid"], [style*="display:grid"]');
                if (gridContainers.length > 0) {
                    layoutType = 'Grid';
                } else if (hasSidebar) {
                    // Check for multi-column layouts
                    const sidebarCount = document.querySelectorAll('aside, .sidebar').length;
                    if (sidebarCount >= 2) {
                        layoutType = 'ThreeColumn';
                    } else {
                        layoutType = 'TwoColumn';
                    }
                } else if (document.querySelectorAll('.dashboard, .panel, .widget').length > 3) {
                    layoutType = 'Dashboard';
                } else {
                    layoutType = 'SingleColumn';
                }
                
                return {
                    has_header: hasHeader,
                    has_navigation: hasNavigation,
                    has_sidebar: hasSidebar,
                    has_footer: hasFooter,
                    main_content_selector: mainContentSelector,
                    layout_type: layoutType
                };
            }
            
            return analyzeLayout();
        "##;
        
        let result = browser.execute_script(script, vec![]).await?;
        let layout: LayoutInfo = serde_json::from_value(result)
            .unwrap_or(LayoutInfo {
                has_header: false,
                has_navigation: false,
                has_sidebar: false,
                has_footer: false,
                main_content_selector: None,
                layout_type: LayoutType::Unknown,
            });
        
        Ok(layout)
    }
    
    /// Extract navigation paths
    async fn extract_navigation(&self, browser: &BrowserConnection) -> Result<Vec<NavPath>> {
        let script = r##"
            function extractNavigation() {
                const paths = [];
                const navLinks = document.querySelectorAll('nav a, .navigation a, .menu a, .nav a');
                
                navLinks.forEach((link, index) => {
                    if (index >= 20) return; // Limit to 20 nav items
                    
                    const rect = link.getBoundingClientRect();
                    if (rect.width === 0 || rect.height === 0) return;
                    
                    paths.push({
                        text: (link.innerText || link.textContent || '').trim(),
                        url: link.href || '',
                        depth: link.closest('ul') ? 
                            (link.closest('ul').querySelectorAll('ul').length + 1) : 0,
                        is_active: link.classList.contains('active') || 
                                  link.classList.contains('current') ||
                                  link.getAttribute('aria-current') === 'page'
                    });
                });
                
                return paths;
            }
            
            return extractNavigation();
        "##;
        
        let result = browser.execute_script(script, vec![]).await?;
        let paths: Vec<NavPath> = serde_json::from_value(result)
            .unwrap_or_else(|_| Vec::new());
        
        Ok(paths)
    }
    
    /// Analyze forms on the page
    async fn analyze_forms(&self, browser: &BrowserConnection) -> Result<Vec<FormInfo>> {
        let script = r##"
            function analyzeForms() {
                const forms = [];
                const formElements = document.querySelectorAll('form');
                
                formElements.forEach((form, index) => {
                    if (index >= 5) return; // Limit to 5 forms
                    
                    const fields = [];
                    const inputs = form.querySelectorAll('input, select, textarea');
                    
                    inputs.forEach(input => {
                        // Find associated label
                        let label = null;
                        if (input.id) {
                            const labelEl = document.querySelector(`label[for="${input.id}"]`);
                            if (labelEl) label = labelEl.innerText;
                        }
                        if (!label && input.closest('label')) {
                            label = input.closest('label').innerText;
                        }
                        
                        fields.push({
                            name: input.name || input.id || '',
                            field_type: input.type || input.tagName.toLowerCase(),
                            label: label,
                            required: input.required || input.hasAttribute('required'),
                            current_value: input.value || null
                        });
                    });
                    
                    // Find submit button
                    let submitButton = null;
                    const submitBtn = form.querySelector('button[type="submit"], input[type="submit"], button:not([type])');
                    if (submitBtn) {
                        if (submitBtn.id) {
                            submitButton = '#' + submitBtn.id;
                        } else {
                            submitButton = submitBtn.tagName.toLowerCase() + '[type="submit"]';
                        }
                    }
                    
                    forms.push({
                        form_selector: form.id ? '#' + form.id : 'form',
                        fields: fields,
                        submit_button: submitButton,
                        method: form.method || 'get',
                        action: form.action || null
                    });
                });
                
                return forms;
            }
            
            return analyzeForms();
        "##;
        
        let result = browser.execute_script(script, vec![]).await?;
        let forms: Vec<FormInfo> = serde_json::from_value(result)
            .unwrap_or_else(|_| Vec::new());
        
        Ok(forms)
    }
}

/// Cached Quick perception
pub struct CachedQuickPerception {
    perception: RealQuickPerception,
    cache: tokio::sync::RwLock<HashMap<String, (QuickData, Instant)>>,
    cache_ttl: Duration,
}

impl CachedQuickPerception {
    pub fn new() -> Self {
        Self {
            perception: RealQuickPerception::new(),
            cache: tokio::sync::RwLock::new(HashMap::new()),
            cache_ttl: Duration::from_secs(10),
        }
    }
    
    pub async fn scan_page(&self, browser: &BrowserConnection) -> Result<QuickData> {
        let url = browser.current_url().await?;
        
        // Check cache
        {
            let cache = self.cache.read().await;
            if let Some((data, timestamp)) = cache.get(&url) {
                if timestamp.elapsed() < self.cache_ttl {
                    debug!("Quick perception cache hit for {}", url);
                    return Ok(data.clone());
                }
            }
        }
        
        // Perform scan
        let data = self.perception.scan_page(browser).await?;
        
        // Update cache
        {
            let mut cache = self.cache.write().await;
            cache.insert(url, (data.clone(), Instant::now()));
            
            // Clean old entries
            cache.retain(|_, (_, timestamp)| timestamp.elapsed() < self.cache_ttl * 2);
        }
        
        Ok(data)
    }
}