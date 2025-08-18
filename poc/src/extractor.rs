use anyhow::Result;
use serde::{Deserialize, Serialize};
use thirtyfour::{By, WebDriver};
use tracing::info;

/// Data extraction module for web scraping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedData {
    pub url: String,
    pub title: Option<String>,
    pub meta_description: Option<String>,
    pub headings: Vec<String>,
    pub links: Vec<LinkData>,
    pub images: Vec<ImageData>,
    pub tables: Vec<TableData>,
    pub text_content: Vec<String>,
    pub form_fields: Vec<FormField>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkData {
    pub text: String,
    pub href: String,
    pub is_external: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageData {
    pub src: String,
    pub alt: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableData {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormField {
    pub name: Option<String>,
    pub field_type: String,
    pub label: Option<String>,
    pub required: bool,
}

pub struct DataExtractor<'a> {
    driver: &'a WebDriver,
}

impl<'a> DataExtractor<'a> {
    pub fn new(driver: &'a WebDriver) -> Self {
        Self { driver }
    }

    /// Extract all data from the current page
    pub async fn extract_all(&self) -> Result<ExtractedData> {
        let url = self.driver.current_url().await?;
        info!("Extracting data from: {}", url);

        let data = ExtractedData {
            url: url.to_string(),
            title: self.extract_title().await.ok(),
            meta_description: self.extract_meta_description().await.ok(),
            headings: self.extract_headings().await.unwrap_or_default(),
            links: self.extract_links(&url).await.unwrap_or_default(),
            images: self.extract_images().await.unwrap_or_default(),
            tables: self.extract_tables().await.unwrap_or_default(),
            text_content: self.extract_text_paragraphs().await.unwrap_or_default(),
            form_fields: self.extract_forms().await.unwrap_or_default(),
        };

        info!("Extraction complete: {} links, {} images, {} tables", 
              data.links.len(), data.images.len(), data.tables.len());

        Ok(data)
    }

    /// Extract page title
    async fn extract_title(&self) -> Result<String> {
        self.driver.title().await.map_err(Into::into)
    }

    /// Extract meta description
    async fn extract_meta_description(&self) -> Result<String> {
        let element = self.driver
            .find(By::Css("meta[name='description']"))
            .await?;
        element.attr("content")
            .await?
            .ok_or_else(|| anyhow::anyhow!("No description found"))
    }

    /// Extract all headings (h1-h6)
    async fn extract_headings(&self) -> Result<Vec<String>> {
        let mut headings = Vec::new();
        
        for level in 1..=6 {
            let selector = format!("h{}", level);
            if let Ok(elements) = self.driver.find_all(By::Css(&selector)).await {
                for element in elements {
                    if let Ok(text) = element.text().await {
                        if !text.trim().is_empty() {
                            headings.push(text.trim().to_string());
                        }
                    }
                }
            }
        }
        
        Ok(headings)
    }

    /// Extract all links
    async fn extract_links(&self, current_url: &url::Url) -> Result<Vec<LinkData>> {
        let elements = self.driver.find_all(By::Css("a[href]")).await?;
        let mut links = Vec::new();
        
        for element in elements.iter().take(100) { // Limit to 100 links
            if let Ok(href) = element.attr("href").await {
                if let Some(href) = href {
                    let text = element.text().await.unwrap_or_default();
                    let is_external = href.starts_with("http") && 
                        !href.contains(current_url.host_str().unwrap_or(""));
                    
                    links.push(LinkData {
                        text: text.trim().to_string(),
                        href: href.to_string(),
                        is_external,
                    });
                }
            }
        }
        
        Ok(links)
    }

    /// Extract all images
    async fn extract_images(&self) -> Result<Vec<ImageData>> {
        let elements = self.driver.find_all(By::Css("img")).await?;
        let mut images = Vec::new();
        
        for element in elements.iter().take(50) { // Limit to 50 images
            if let Ok(src) = element.attr("src").await {
                if let Some(src) = src {
                    let alt = element.attr("alt").await.ok().flatten();
                    let width = element.attr("width").await.ok()
                        .flatten()
                        .and_then(|w| w.parse().ok());
                    let height = element.attr("height").await.ok()
                        .flatten()
                        .and_then(|h| h.parse().ok());
                    
                    images.push(ImageData {
                        src: src.to_string(),
                        alt,
                        width,
                        height,
                    });
                }
            }
        }
        
        Ok(images)
    }

    /// Extract tables
    async fn extract_tables(&self) -> Result<Vec<TableData>> {
        let tables = self.driver.find_all(By::Css("table")).await?;
        let mut table_data = Vec::new();
        
        for table in tables.iter().take(10) { // Limit to 10 tables
            let mut headers = Vec::new();
            let mut rows = Vec::new();
            
            // Extract headers
            if let Ok(header_elements) = table.find_all(By::Css("th")).await {
                for header in header_elements {
                    if let Ok(text) = header.text().await {
                        headers.push(text.trim().to_string());
                    }
                }
            }
            
            // Extract rows
            if let Ok(row_elements) = table.find_all(By::Css("tr")).await {
                for row in row_elements {
                    if let Ok(cells) = row.find_all(By::Css("td")).await {
                        let mut row_data = Vec::new();
                        for cell in cells {
                            if let Ok(text) = cell.text().await {
                                row_data.push(text.trim().to_string());
                            }
                        }
                        if !row_data.is_empty() {
                            rows.push(row_data);
                        }
                    }
                }
            }
            
            if !headers.is_empty() || !rows.is_empty() {
                table_data.push(TableData { headers, rows });
            }
        }
        
        Ok(table_data)
    }

    /// Extract text paragraphs
    async fn extract_text_paragraphs(&self) -> Result<Vec<String>> {
        let elements = self.driver.find_all(By::Css("p")).await?;
        let mut paragraphs = Vec::new();
        
        for element in elements.iter().take(50) { // Limit to 50 paragraphs
            if let Ok(text) = element.text().await {
                let text = text.trim();
                if !text.is_empty() && text.len() > 20 { // Filter out short snippets
                    paragraphs.push(text.to_string());
                }
            }
        }
        
        Ok(paragraphs)
    }

    /// Extract form fields
    async fn extract_forms(&self) -> Result<Vec<FormField>> {
        let mut fields = Vec::new();
        
        // Extract input fields
        if let Ok(inputs) = self.driver.find_all(By::Css("input")).await {
            for input in inputs.iter().take(30) { // Limit to 30 fields
                let name = input.attr("name").await.ok().flatten();
                let field_type = input.attr("type").await.ok()
                    .flatten()
                    .unwrap_or_else(|| "text".to_string());
                let required = input.attr("required").await.ok().flatten().is_some();
                
                // Try to find associated label
                let label = if let Some(id) = input.attr("id").await.ok().flatten() {
                    if let Ok(label_elem) = self.driver
                        .find(By::Css(&format!("label[for='{}']", id)))
                        .await {
                        label_elem.text().await.ok()
                    } else {
                        None
                    }
                } else {
                    None
                };
                
                fields.push(FormField {
                    name,
                    field_type,
                    label,
                    required,
                });
            }
        }
        
        // Extract select fields
        if let Ok(selects) = self.driver.find_all(By::Css("select")).await {
            for select in selects.iter().take(10) {
                let name = select.attr("name").await.ok().flatten();
                let required = select.attr("required").await.ok().flatten().is_some();
                
                fields.push(FormField {
                    name,
                    field_type: "select".to_string(),
                    label: None,
                    required,
                });
            }
        }
        
        // Extract textareas
        if let Ok(textareas) = self.driver.find_all(By::Css("textarea")).await {
            for textarea in textareas.iter().take(10) {
                let name = textarea.attr("name").await.ok().flatten();
                let required = textarea.attr("required").await.ok().flatten().is_some();
                
                fields.push(FormField {
                    name,
                    field_type: "textarea".to_string(),
                    label: None,
                    required,
                });
            }
        }
        
        Ok(fields)
    }

    /// Extract specific data using CSS selector
    pub async fn extract_by_selector(&self, selector: &str) -> Result<Vec<String>> {
        let elements = self.driver.find_all(By::Css(selector)).await?;
        let mut results = Vec::new();
        
        for element in elements {
            if let Ok(text) = element.text().await {
                if !text.trim().is_empty() {
                    results.push(text.trim().to_string());
                }
            }
        }
        
        Ok(results)
    }

    /// Extract data as JSON
    pub async fn extract_as_json(&self) -> Result<String> {
        let data = self.extract_all().await?;
        serde_json::to_string_pretty(&data).map_err(Into::into)
    }

    /// Extract data as CSV (simplified - just links for now)
    pub async fn extract_links_as_csv(&self) -> Result<String> {
        let url = self.driver.current_url().await?;
        let links = self.extract_links(&url).await?;
        
        let mut csv = String::from("text,href,is_external\n");
        for link in links {
            csv.push_str(&format!(
                "\"{}\",\"{}\",{}\n",
                link.text.replace("\"", "\"\""),
                link.href.replace("\"", "\"\""),
                link.is_external
            ));
        }
        
        Ok(csv)
    }
}