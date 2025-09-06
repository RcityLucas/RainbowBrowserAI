use super::traits::{Tool, ToolCategory};
use crate::browser::Browser;
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use std::sync::Arc;
use std::collections::HashMap;
use tracing::info;

// ============================================================================
// Extract Text Tool
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct ExtractTextInput {
    pub selector: String,
    #[serde(default)]
    pub include_hidden: bool,
    #[serde(default)]
    pub trim: bool,
}

#[derive(Debug, Serialize)]
pub struct ExtractTextOutput {
    pub success: bool,
    pub text: String,
    pub element_count: usize,
}

pub struct ExtractTextTool {
    browser: Arc<Browser>,
}

impl ExtractTextTool {
    pub fn new(browser: Arc<Browser>) -> Self {
        Self { browser }
    }
}

#[async_trait]
impl Tool for ExtractTextTool {
    type Input = ExtractTextInput;
    type Output = ExtractTextOutput;
    
    fn name(&self) -> &str {
        "extract_text"
    }
    
    fn description(&self) -> &str {
        "Extract text content from elements"
    }
    
    fn category(&self) -> ToolCategory {
        ToolCategory::DataExtraction
    }
    
    async fn execute(&self, input: Self::Input) -> Result<Self::Output> {
        info!("Extracting text from: {}", input.selector);
        
        let script = format!(
            r#"
            (function() {{
                const elements = document.querySelectorAll('{}');
                const texts = [];
                let count = 0;
                
                elements.forEach(el => {{
                    const isVisible = {} || (
                        el.offsetParent !== null && 
                        window.getComputedStyle(el).display !== 'none' &&
                        window.getComputedStyle(el).visibility !== 'hidden'
                    );
                    
                    if (isVisible) {{
                        const text = el.textContent || el.innerText || '';
                        texts.push({} ? text.trim() : text);
                        count++;
                    }}
                }});
                
                return {{
                    text: texts.join(' '),
                    count: count
                }};
            }})()"#,
            input.selector,
            input.include_hidden,
            input.trim
        );
        
        let result = self.browser.execute_script(&script).await?;
        
        Ok(ExtractTextOutput {
            success: true,
            text: result["text"].as_str().unwrap_or("").to_string(),
            element_count: result["count"].as_u64().unwrap_or(0) as usize,
        })
    }
    
    async fn validate_input(&self, input: &Self::Input) -> Result<()> {
        if input.selector.is_empty() {
            return Err(anyhow!("Selector cannot be empty"));
        }
        Ok(())
    }
}

// ============================================================================
// Extract Links Tool
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct ExtractLinksInput {
    #[serde(default)]
    pub selector: Option<String>,
    #[serde(default)]
    pub include_external: bool,
    #[serde(default)]
    pub include_internal: bool,
    #[serde(default)]
    pub absolute_urls: bool,
}

#[derive(Debug, Serialize)]
pub struct ExtractLinksOutput {
    pub success: bool,
    pub links: Vec<LinkInfo>,
    pub total_count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LinkInfo {
    pub href: String,
    pub text: String,
    pub title: Option<String>,
    pub is_external: bool,
}

pub struct ExtractLinksTool {
    browser: Arc<Browser>,
}

impl ExtractLinksTool {
    pub fn new(browser: Arc<Browser>) -> Self {
        Self { browser }
    }
}

impl Default for ExtractLinksInput {
    fn default() -> Self {
        Self {
            selector: None,
            include_external: true,
            include_internal: true,
            absolute_urls: true,
        }
    }
}

#[async_trait]
impl Tool for ExtractLinksTool {
    type Input = ExtractLinksInput;
    type Output = ExtractLinksOutput;
    
    fn name(&self) -> &str {
        "extract_links"
    }
    
    fn description(&self) -> &str {
        "Extract all links from the page or specific elements"
    }
    
    fn category(&self) -> ToolCategory {
        ToolCategory::DataExtraction
    }
    
    async fn execute(&self, input: Self::Input) -> Result<Self::Output> {
        let selector = input.selector.as_deref().unwrap_or("a");
        info!("Extracting links from: {}", selector);
        
        let script = format!(
            r#"
            (function() {{
                const links = document.querySelectorAll('{}');
                const currentHost = window.location.host;
                const results = [];
                
                links.forEach(link => {{
                    const href = link.href || link.getAttribute('href') || '';
                    if (!href) return;
                    
                    let absoluteHref = href;
                    if ({} && !href.startsWith('http')) {{
                        const url = new URL(href, window.location.origin);
                        absoluteHref = url.href;
                    }}
                    
                    const isExternal = absoluteHref.startsWith('http') && 
                                       !absoluteHref.includes(currentHost);
                    
                    if (({} && isExternal) || ({} && !isExternal)) {{
                        results.push({{
                            href: absoluteHref,
                            text: link.textContent?.trim() || '',
                            title: link.title || null,
                            is_external: isExternal
                        }});
                    }}
                }});
                
                return results;
            }})()"#,
            selector,
            input.absolute_urls,
            input.include_external,
            input.include_internal
        );
        
        let result = self.browser.execute_script(&script).await?;
        
        let links: Vec<LinkInfo> = serde_json::from_value(result)?;
        let total_count = links.len();
        
        Ok(ExtractLinksOutput {
            success: true,
            links,
            total_count,
        })
    }
}

// ============================================================================
// Extract Data Tool
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct ExtractDataInput {
    pub selector: String,
    pub attributes: Vec<String>,
    #[serde(default)]
    pub include_text: bool,
    #[serde(default)]
    pub include_html: bool,
}

#[derive(Debug, Serialize)]
pub struct ExtractDataOutput {
    pub success: bool,
    pub data: Vec<ElementData>,
    pub total_count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ElementData {
    pub text: Option<String>,
    pub html: Option<String>,
    pub attributes: HashMap<String, String>,
}

pub struct ExtractDataTool {
    browser: Arc<Browser>,
}

impl ExtractDataTool {
    pub fn new(browser: Arc<Browser>) -> Self {
        Self { browser }
    }
}

#[async_trait]
impl Tool for ExtractDataTool {
    type Input = ExtractDataInput;
    type Output = ExtractDataOutput;
    
    fn name(&self) -> &str {
        "extract_data"
    }
    
    fn description(&self) -> &str {
        "Extract structured data from elements"
    }
    
    fn category(&self) -> ToolCategory {
        ToolCategory::DataExtraction
    }
    
    async fn execute(&self, input: Self::Input) -> Result<Self::Output> {
        info!("Extracting data from: {}", input.selector);
        
        let attributes_json = serde_json::to_string(&input.attributes)?;
        
        let script = format!(
            r#"
            (function() {{
                const elements = document.querySelectorAll('{}');
                const attributes = {};
                const includeText = {};
                const includeHtml = {};
                const results = [];
                
                elements.forEach(el => {{
                    const data = {{
                        text: includeText ? (el.textContent?.trim() || null) : null,
                        html: includeHtml ? el.innerHTML : null,
                        attributes: {{}}
                    }};
                    
                    attributes.forEach(attr => {{
                        data.attributes[attr] = el.getAttribute(attr) || '';
                    }});
                    
                    results.push(data);
                }});
                
                return results;
            }})()"#,
            input.selector,
            attributes_json,
            input.include_text,
            input.include_html
        );
        
        let result = self.browser.execute_script(&script).await?;
        let data: Vec<ElementData> = serde_json::from_value(result)?;
        let total_count = data.len();
        
        Ok(ExtractDataOutput {
            success: true,
            data,
            total_count,
        })
    }
    
    async fn validate_input(&self, input: &Self::Input) -> Result<()> {
        if input.selector.is_empty() {
            return Err(anyhow!("Selector cannot be empty"));
        }
        Ok(())
    }
}

// ============================================================================
// Extract Table Tool
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct ExtractTableInput {
    pub selector: String,
    #[serde(default)]
    pub include_headers: bool,
    #[serde(default)]
    pub as_objects: bool,
}

#[derive(Debug, Serialize)]
pub struct ExtractTableOutput {
    pub success: bool,
    pub headers: Vec<String>,
    pub rows: Vec<TableRow>,
    pub total_rows: usize,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum TableRow {
    Array(Vec<String>),
    Object(HashMap<String, String>),
}

pub struct ExtractTableTool {
    browser: Arc<Browser>,
}

impl ExtractTableTool {
    pub fn new(browser: Arc<Browser>) -> Self {
        Self { browser }
    }
}

#[async_trait]
impl Tool for ExtractTableTool {
    type Input = ExtractTableInput;
    type Output = ExtractTableOutput;
    
    fn name(&self) -> &str {
        "extract_table"
    }
    
    fn description(&self) -> &str {
        "Extract data from HTML tables"
    }
    
    fn category(&self) -> ToolCategory {
        ToolCategory::DataExtraction
    }
    
    async fn execute(&self, input: Self::Input) -> Result<Self::Output> {
        info!("Extracting table from: {}", input.selector);
        
        let script = format!(
            r#"
            (function() {{
                const table = document.querySelector('{}');
                if (!table) return {{ headers: [], rows: [] }};
                
                const headers = [];
                const rows = [];
                
                // Extract headers
                const headerCells = table.querySelectorAll('thead th, thead td');
                headerCells.forEach(cell => {{
                    headers.push(cell.textContent?.trim() || '');
                }});
                
                // If no thead, try first row
                if (headers.length === 0 && {}) {{
                    const firstRow = table.querySelector('tr');
                    if (firstRow) {{
                        firstRow.querySelectorAll('th, td').forEach(cell => {{
                            headers.push(cell.textContent?.trim() || '');
                        }});
                    }}
                }}
                
                // Extract rows
                const dataRows = table.querySelectorAll('tbody tr, tr');
                const startIndex = (headers.length > 0 && {}) ? 1 : 0;
                
                for (let i = startIndex; i < dataRows.length; i++) {{
                    const row = dataRows[i];
                    const cells = row.querySelectorAll('td, th');
                    const rowData = [];
                    
                    cells.forEach(cell => {{
                        rowData.push(cell.textContent?.trim() || '');
                    }});
                    
                    if (rowData.length > 0) {{
                        if ({} && headers.length > 0) {{
                            const obj = {{}};
                            headers.forEach((header, index) => {{
                                obj[header] = rowData[index] || '';
                            }});
                            rows.push(obj);
                        }} else {{
                            rows.push(rowData);
                        }}
                    }}
                }}
                
                return {{ headers, rows }};
            }})()"#,
            input.selector,
            input.include_headers,
            input.include_headers,
            input.as_objects
        );
        
        let result = self.browser.execute_script(&script).await?;
        
        let headers: Vec<String> = serde_json::from_value(result["headers"].clone())?;
        let rows: Vec<TableRow> = if input.as_objects {
            result["rows"].as_array()
                .unwrap_or(&Vec::new())
                .iter()
                .map(|r| TableRow::Object(serde_json::from_value(r.clone()).unwrap_or_default()))
                .collect()
        } else {
            result["rows"].as_array()
                .unwrap_or(&Vec::new())
                .iter()
                .map(|r| TableRow::Array(serde_json::from_value(r.clone()).unwrap_or_default()))
                .collect()
        };
        
        let total_rows = rows.len();
        
        Ok(ExtractTableOutput {
            success: true,
            headers,
            rows,
            total_rows,
        })
    }
    
    async fn validate_input(&self, input: &Self::Input) -> Result<()> {
        if input.selector.is_empty() {
            return Err(anyhow!("Selector cannot be empty"));
        }
        Ok(())
    }
}

// ============================================================================
// Extract Form Tool
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct ExtractFormInput {
    pub selector: String,
    #[serde(default)]
    pub include_values: bool,
    #[serde(default)]
    pub include_options: bool,
}

#[derive(Debug, Serialize)]
pub struct ExtractFormOutput {
    pub success: bool,
    pub form_data: FormData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FormData {
    pub action: Option<String>,
    pub method: Option<String>,
    pub fields: Vec<FormField>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FormField {
    pub name: String,
    pub field_type: String,
    pub value: Option<String>,
    pub required: bool,
    pub options: Option<Vec<String>>,
    pub label: Option<String>,
}

pub struct ExtractFormTool {
    browser: Arc<Browser>,
}

impl ExtractFormTool {
    pub fn new(browser: Arc<Browser>) -> Self {
        Self { browser }
    }
}

#[async_trait]
impl Tool for ExtractFormTool {
    type Input = ExtractFormInput;
    type Output = ExtractFormOutput;
    
    fn name(&self) -> &str {
        "extract_form"
    }
    
    fn description(&self) -> &str {
        "Extract form structure and data"
    }
    
    fn category(&self) -> ToolCategory {
        ToolCategory::DataExtraction
    }
    
    async fn execute(&self, input: Self::Input) -> Result<Self::Output> {
        info!("Extracting form from: {}", input.selector);
        
        let script = format!(
            r#"
            (function() {{
                const form = document.querySelector('{}');
                if (!form) return null;
                
                const formData = {{
                    action: form.action || null,
                    method: form.method || 'GET',
                    fields: []
                }};
                
                const inputs = form.querySelectorAll('input, textarea, select');
                
                inputs.forEach(field => {{
                    const fieldData = {{
                        name: field.name || field.id || '',
                        field_type: field.type || field.tagName.toLowerCase(),
                        value: {} ? field.value : null,
                        required: field.required || false,
                        options: null,
                        label: null
                    }};
                    
                    // Find associated label
                    if (field.id) {{
                        const label = form.querySelector(`label[for="${{field.id}}"]`);
                        if (label) {{
                            fieldData.label = label.textContent?.trim() || null;
                        }}
                    }}
                    
                    // Get options for select elements
                    if (field.tagName === 'SELECT' && {}) {{
                        fieldData.options = Array.from(field.options).map(o => o.text);
                    }}
                    
                    formData.fields.push(fieldData);
                }});
                
                return formData;
            }})()"#,
            input.selector,
            input.include_values,
            input.include_options
        );
        
        let result = self.browser.execute_script(&script).await?;
        let form_data: FormData = serde_json::from_value(result)?;
        
        Ok(ExtractFormOutput {
            success: true,
            form_data,
        })
    }
    
    async fn validate_input(&self, input: &Self::Input) -> Result<()> {
        if input.selector.is_empty() {
            return Err(anyhow!("Selector cannot be empty"));
        }
        Ok(())
    }
}