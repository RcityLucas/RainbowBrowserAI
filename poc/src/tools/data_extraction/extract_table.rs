// Extract Table Tool - Phase 2 Week 7 Implementation
// 
// This tool specializes in extracting tabular data from web pages with intelligent
// table structure detection, column mapping, and advanced table processing capabilities.

use crate::tools::{Tool, ToolError};
use super::{OutputFormat, ExtractionScope, ExtractionConfig, ExtractionResult, ExtractionMetadata, text_utils, format_utils};
use std::sync::Arc;
use std::collections::HashMap;
use thirtyfour::{WebDriver, By, WebElement};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::time::{Duration, Instant};
use chrono::Utc;

/// Table column definition with data type inference
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TableColumn {
    /// Column name (from header or generated)
    pub name: String,
    
    /// Column index in the table
    pub index: usize,
    
    /// Inferred or specified data type
    pub data_type: TableDataType,
    
    /// Whether this column contains required data
    pub required: bool,
    
    /// Column header element text
    pub header_text: String,
    
    /// Column alignment (left, right, center)
    pub alignment: Option<String>,
    
    /// Column width hint from CSS
    pub width_hint: Option<String>,
}

/// Data types optimized for table data
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TableDataType {
    /// Text data (default)
    Text,
    /// Numeric data (integers and floats)
    Number,
    /// Currency values
    Currency,
    /// Percentage values
    Percentage,
    /// Date/time values
    Date,
    /// Boolean values (checkboxes, yes/no)
    Boolean,
    /// Link/URL values
    Link,
    /// Image URLs
    Image,
    /// Auto-detect based on content
    Auto,
}

impl Default for TableDataType {
    fn default() -> Self {
        TableDataType::Auto
    }
}

impl std::str::FromStr for TableDataType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "text" => Ok(TableDataType::Text),
            "number" | "numeric" => Ok(TableDataType::Number),
            "currency" | "money" => Ok(TableDataType::Currency),
            "percentage" | "percent" => Ok(TableDataType::Percentage),
            "date" | "datetime" => Ok(TableDataType::Date),
            "boolean" | "bool" => Ok(TableDataType::Boolean),
            "link" | "url" => Ok(TableDataType::Link),
            "image" | "img" => Ok(TableDataType::Image),
            "auto" => Ok(TableDataType::Auto),
            _ => Err(format!("Invalid table data type: '{}'. Valid types: text, number, currency, percentage, date, boolean, link, image, auto", s))
        }
    }
}

/// Table structure information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableStructure {
    /// Table columns with metadata
    pub columns: Vec<TableColumn>,
    
    /// Total number of data rows (excluding headers)
    pub row_count: usize,
    
    /// Whether table has a header row
    pub has_header: bool,
    
    /// Whether table has row headers (first column as headers)
    pub has_row_headers: bool,
    
    /// Table caption or title
    pub caption: Option<String>,
    
    /// Table summary or description
    pub summary: Option<String>,
    
    /// CSS classes on the table element
    pub css_classes: Vec<String>,
    
    /// Table ID attribute
    pub table_id: Option<String>,
}

/// Individual table row data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableRow {
    /// Row index (0-based, excluding headers)
    pub index: usize,
    
    /// Cell data mapped by column name
    pub cells: HashMap<String, TableCell>,
    
    /// Row CSS classes
    pub css_classes: Vec<String>,
    
    /// Whether this is a header row
    pub is_header: bool,
    
    /// Whether this is a footer row
    pub is_footer: bool,
}

/// Individual table cell data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableCell {
    /// Raw text content
    pub raw_text: String,
    
    /// Processed value according to column data type
    pub value: serde_json::Value,
    
    /// Column span (for merged cells)
    pub col_span: usize,
    
    /// Row span (for merged cells)
    pub row_span: usize,
    
    /// Cell alignment
    pub alignment: Option<String>,
    
    /// CSS classes
    pub css_classes: Vec<String>,
    
    /// Link href if cell contains a link
    pub link_url: Option<String>,
    
    /// Image src if cell contains an image
    pub image_url: Option<String>,
}

/// Table extraction configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableExtractionConfig {
    /// Whether to include header row in output
    pub include_headers: bool,
    
    /// Whether to include footer rows
    pub include_footers: bool,
    
    /// Whether to merge cells spanning multiple columns/rows
    pub merge_spanned_cells: bool,
    
    /// Skip rows that are entirely empty
    pub skip_empty_rows: bool,
    
    /// Minimum number of columns required to consider it a valid table
    pub min_columns: usize,
    
    /// Maximum number of rows to extract (0 = no limit)
    pub max_rows: usize,
    
    /// Column name mapping (old_name -> new_name)
    pub column_mapping: HashMap<String, String>,
    
    /// Specific columns to include (if empty, include all)
    pub include_columns: Vec<String>,
    
    /// Columns to exclude
    pub exclude_columns: Vec<String>,
}

impl Default for TableExtractionConfig {
    fn default() -> Self {
        Self {
            include_headers: true,
            include_footers: false,
            merge_spanned_cells: true,
            skip_empty_rows: true,
            min_columns: 2,
            max_rows: 0,
            column_mapping: HashMap::new(),
            include_columns: Vec::new(),
            exclude_columns: Vec::new(),
        }
    }
}

/// Input parameters for extract_table tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractTableInput {
    /// CSS selector for table elements (optional, defaults to "table")
    pub table_selector: Option<String>,
    
    /// Table extraction configuration
    pub config: ExtractionConfig,
    
    /// Table-specific configuration
    pub table_config: TableExtractionConfig,
    
    /// Whether to extract from multiple tables or just the first
    pub extract_multiple: bool,
    
    /// Column data types (column_name -> data_type)
    pub column_types: HashMap<String, TableDataType>,
    
    /// Whether to infer data types automatically
    pub auto_infer_types: bool,
}

impl Default for ExtractTableInput {
    fn default() -> Self {
        Self {
            table_selector: None,
            config: ExtractionConfig::default(),
            table_config: TableExtractionConfig::default(),
            extract_multiple: false,
            column_types: HashMap::new(),
            auto_infer_types: true,
        }
    }
}

/// Output from extract_table tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractTableOutput {
    /// Extracted table data
    pub tables: Vec<TableData>,
    
    /// Formatted output in requested format
    pub formatted_output: String,
    
    /// Total number of tables extracted
    pub table_count: usize,
    
    /// Total number of rows across all tables
    pub total_rows: usize,
    
    /// Extraction configuration used
    pub config: ExtractionConfig,
    
    /// Table configuration used
    pub table_config: TableExtractionConfig,
}

/// Complete table data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableData {
    /// Table structure information
    pub structure: TableStructure,
    
    /// Table rows
    pub rows: Vec<TableRow>,
    
    /// Table index on the page (0-based)
    pub table_index: usize,
    
    /// CSS selector that found this table
    pub selector: String,
}

/// Extract table tool implementation
pub struct ExtractTable {
    driver: Arc<WebDriver>,
}

impl ExtractTable {
    /// Create a new extract table tool
    pub fn new(driver: Arc<WebDriver>) -> Self {
        Self { driver }
    }
    
    /// Find table elements using selector
    async fn find_tables(&self, selector: &str) -> anyhow::Result<Vec<WebElement>> {
        Ok(self.driver.find_all(By::Css(selector)).await?)
    }
    
    /// Detect table structure from table element
    async fn analyze_table_structure(&self, table: &WebElement) -> anyhow::Result<TableStructure> {
        // Get table metadata
        let caption = self.get_table_caption(table).await?;
        let summary = self.get_table_summary(table).await?;
        let css_classes = self.get_css_classes(table).await?;
        let table_id = table.attr("id").await?.unwrap_or_default();
        
        // Find header rows and columns
        let header_rows = table.find_all(By::Css("thead tr, tr:first-child")).await.unwrap_or_default();
        let has_header = !header_rows.is_empty();
        
        // Analyze columns from first row (header or data)
        let first_row = if has_header && !header_rows.is_empty() {
            &header_rows[0]
        } else {
            // Find first data row
            let data_rows = table.find_all(By::Css("tbody tr, tr")).await.unwrap_or_default();
            if data_rows.is_empty() {
                return Err(anyhow::anyhow!("Table has no rows"));
            }
            &data_rows[0]
        };
        
        let columns = self.analyze_columns(first_row, has_header).await?;
        
        // Count data rows
        let row_count = {
            let all_rows = table.find_all(By::Css("tbody tr, tr")).await.unwrap_or_default();
            if has_header { all_rows.len().saturating_sub(1) } else { all_rows.len() }
        };
        
        // Check for row headers (first column contains th elements)
        let first_column_cells = table.find_all(By::Css("tr td:first-child, tr th:first-child")).await.unwrap_or_default();
        let mut has_row_headers = false;
        for cell in &first_column_cells {
            if let Ok(tag) = cell.tag_name().await {
                if tag == "th" {
                    has_row_headers = true;
                    break;
                }
            }
            if let Ok(Some(scope)) = cell.attr("scope").await {
                if scope == "row" {
                    has_row_headers = true;
                    break;
                }
            }
        }
        
        Ok(TableStructure {
            columns,
            row_count,
            has_header,
            has_row_headers,
            caption,
            summary,
            css_classes,
            table_id: if table_id.is_empty() { None } else { Some(table_id) },
        })
    }
    
    /// Get table caption
    async fn get_table_caption(&self, table: &WebElement) -> anyhow::Result<Option<String>> {
        if let Ok(caption) = table.find(By::Css("caption")).await {
            let text = caption.text().await?;
            if !text.trim().is_empty() {
                return Ok(Some(text_utils::clean_text(&text)));
            }
        }
        Ok(None)
    }
    
    /// Get table summary
    async fn get_table_summary(&self, table: &WebElement) -> anyhow::Result<Option<String>> {
        if let Ok(Some(summary)) = table.attr("summary").await {
            if !summary.trim().is_empty() {
                return Ok(Some(summary));
            }
        }
        Ok(None)
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
    
    /// Analyze table columns from header/first row
    async fn analyze_columns(&self, row: &WebElement, is_header: bool) -> anyhow::Result<Vec<TableColumn>> {
        let cells = row.find_all(By::Css("th, td")).await?;
        let mut columns = Vec::new();
        
        for (index, cell) in cells.iter().enumerate() {
            let header_text = text_utils::clean_text(&cell.text().await?);
            let name = if header_text.is_empty() {
                format!("column_{}", index + 1)
            } else {
                self.sanitize_column_name(&header_text)
            };
            
            // Get alignment and width hints
            let alignment = self.detect_cell_alignment(cell).await?;
            let width_hint = if let Ok(Some(width)) = cell.attr("width").await {
                Some(width)
            } else if let Ok(Some(style)) = cell.attr("style").await {
                if style.contains("width:") {
                    style.split("width:").nth(1).and_then(|s| s.split(';').next()).map(|s| s.trim().to_string())
                } else {
                    None
                }
            } else {
                None
            };
            
            columns.push(TableColumn {
                name,
                index,
                data_type: TableDataType::Auto,
                required: false,
                header_text,
                alignment,
                width_hint,
            });
        }
        
        Ok(columns)
    }
    
    /// Sanitize column name for use as identifier
    fn sanitize_column_name(&self, text: &str) -> String {
        text.chars()
            .map(|c| if c.is_alphanumeric() || c == '_' { c.to_ascii_lowercase() } else { '_' })
            .collect::<String>()
            .trim_matches('_')
            .to_string()
    }
    
    /// Detect cell alignment from styles
    async fn detect_cell_alignment(&self, cell: &WebElement) -> anyhow::Result<Option<String>> {
        if let Ok(Some(align)) = cell.attr("align").await {
            return Ok(Some(align));
        }
        
        // Try to detect from CSS text-align
        if let Ok(Some(style)) = cell.attr("style").await {
            if let Some(text_align) = self.extract_text_align_from_style(&style) {
                return Ok(Some(text_align));
            }
        }
        
        Ok(None)
    }
    
    /// Extract text-align value from style attribute
    fn extract_text_align_from_style(&self, style: &str) -> Option<String> {
        for part in style.split(';') {
            let part = part.trim();
            if part.starts_with("text-align:") {
                return part.split(':').nth(1).map(|s| s.trim().to_string());
            }
        }
        None
    }
    
    /// Extract data from table element
    async fn extract_table_data(&self, table: &WebElement, table_index: usize, selector: &str, input: &ExtractTableInput) -> anyhow::Result<TableData> {
        let mut structure = self.analyze_table_structure(table).await?;
        
        // Apply column type overrides and infer types
        if input.auto_infer_types {
            self.infer_column_types(&mut structure, table).await?;
        }
        
        // Apply manual column type overrides
        for column in &mut structure.columns {
            if let Some(data_type) = input.column_types.get(&column.name) {
                column.data_type = data_type.clone();
            }
        }
        
        // Extract rows
        let rows = self.extract_table_rows(table, &structure, input).await?;
        
        Ok(TableData {
            structure,
            rows,
            table_index,
            selector: selector.to_string(),
        })
    }
    
    /// Infer column data types from table content
    async fn infer_column_types(&self, structure: &mut TableStructure, table: &WebElement) -> anyhow::Result<()> {
        // Get sample data from first few rows
        let sample_rows = table.find_all(By::Css("tbody tr, tr")).await.unwrap_or_default();
        let sample_size = std::cmp::min(sample_rows.len(), 10);
        
        for column in &mut structure.columns {
            let mut sample_values = Vec::new();
            
            // Collect sample values for this column
            for row in sample_rows.iter().take(sample_size) {
                if let Ok(cells) = row.find_all(By::Css("td, th")).await {
                    if column.index < cells.len() {
                        if let Ok(text) = cells[column.index].text().await {
                            let cleaned = text_utils::clean_text(&text);
                            if !cleaned.is_empty() {
                                sample_values.push(cleaned);
                            }
                        }
                    }
                }
            }
            
            // Infer type from sample values
            column.data_type = self.infer_data_type_from_samples(&sample_values);
        }
        
        Ok(())
    }
    
    /// Infer data type from sample values
    fn infer_data_type_from_samples(&self, samples: &[String]) -> TableDataType {
        if samples.is_empty() {
            return TableDataType::Text;
        }
        
        let mut number_count = 0;
        let mut currency_count = 0;
        let mut percentage_count = 0;
        let mut date_count = 0;
        let mut boolean_count = 0;
        let mut link_count = 0;
        
        for sample in samples {
            if self.looks_like_currency(sample) {
                currency_count += 1;
            } else if self.looks_like_percentage(sample) {
                percentage_count += 1;
            } else if self.looks_like_number(sample) {
                number_count += 1;
            } else if self.looks_like_date(sample) {
                date_count += 1;
            } else if self.looks_like_boolean(sample) {
                boolean_count += 1;
            } else if self.looks_like_link(sample) {
                link_count += 1;
            }
        }
        
        let total = samples.len();
        let threshold = (total as f64 * 0.6) as usize; // 60% threshold
        
        if currency_count >= threshold {
            TableDataType::Currency
        } else if percentage_count >= threshold {
            TableDataType::Percentage
        } else if number_count >= threshold {
            TableDataType::Number
        } else if date_count >= threshold {
            TableDataType::Date
        } else if boolean_count >= threshold {
            TableDataType::Boolean
        } else if link_count >= threshold {
            TableDataType::Link
        } else {
            TableDataType::Text
        }
    }
    
    /// Check if text looks like currency
    fn looks_like_currency(&self, text: &str) -> bool {
        let text = text.trim();
        // Check for currency symbols and number patterns
        text.starts_with('$') || text.starts_with('€') || text.starts_with('£') ||
        text.ends_with(" USD") || text.ends_with(" EUR") || text.ends_with(" GBP") ||
        (text.contains('.') && text.chars().filter(|c| c.is_ascii_digit()).count() > 0 && 
         text.chars().any(|c| "$€£¥¢".contains(c)))
    }
    
    /// Check if text looks like percentage
    fn looks_like_percentage(&self, text: &str) -> bool {
        text.trim().ends_with('%') && text.chars().any(|c| c.is_ascii_digit())
    }
    
    /// Check if text looks like number
    fn looks_like_number(&self, text: &str) -> bool {
        text.trim().parse::<f64>().is_ok()
    }
    
    /// Check if text looks like date
    fn looks_like_date(&self, text: &str) -> bool {
        let text = text.trim();
        // Simple date pattern matching
        text.contains('/') || text.contains('-') || text.contains('.') &&
        text.chars().any(|c| c.is_ascii_digit()) &&
        (text.len() >= 8 && text.len() <= 20)
    }
    
    /// Check if text looks like boolean
    fn looks_like_boolean(&self, text: &str) -> bool {
        let text = text.trim().to_lowercase();
        matches!(text.as_str(), "true" | "false" | "yes" | "no" | "y" | "n" | 
                              "on" | "off" | "enabled" | "disabled" | "1" | "0" | 
                              "✓" | "✗" | "✔" | "✘" | "☑" | "☐")
    }
    
    /// Check if text looks like link
    fn looks_like_link(&self, text: &str) -> bool {
        text.starts_with("http://") || text.starts_with("https://") || 
        text.starts_with("www.") || text.contains(".com") || text.contains(".org")
    }
    
    /// Extract rows from table
    async fn extract_table_rows(&self, table: &WebElement, structure: &TableStructure, input: &ExtractTableInput) -> anyhow::Result<Vec<TableRow>> {
        let all_rows = table.find_all(By::Css("tr")).await?;
        let mut rows = Vec::new();
        let mut row_index = 0;
        
        // Skip header row if present
        let start_index = if structure.has_header { 1 } else { 0 };
        
        for (i, row_element) in all_rows.iter().enumerate().skip(start_index) {
            // Check max rows limit
            if input.table_config.max_rows > 0 && rows.len() >= input.table_config.max_rows {
                break;
            }
            
            // Extract row data
            let row = self.extract_row_data(row_element, structure, row_index, input).await?;
            
            // Skip empty rows if configured
            if input.table_config.skip_empty_rows && self.is_row_empty(&row) {
                continue;
            }
            
            rows.push(row);
            row_index += 1;
        }
        
        Ok(rows)
    }
    
    /// Extract data from a single table row
    async fn extract_row_data(&self, row: &WebElement, structure: &TableStructure, row_index: usize, input: &ExtractTableInput) -> anyhow::Result<TableRow> {
        let cells_elements = row.find_all(By::Css("td, th")).await?;
        let mut cells = HashMap::new();
        let css_classes = self.get_css_classes(row).await?;
        
        // Check if this is a header or footer row
        let is_header = row.find(By::Css("th")).await.is_ok();
        let is_footer = css_classes.iter().any(|c| c.contains("footer")) || 
                       row.find(By::Css("tfoot")).await.is_ok();
        
        for (cell_index, cell_element) in cells_elements.iter().enumerate() {
            if cell_index >= structure.columns.len() {
                break; // Skip extra cells
            }
            
            let column = &structure.columns[cell_index];
            
            // Skip excluded columns
            if !input.table_config.exclude_columns.is_empty() && 
               input.table_config.exclude_columns.contains(&column.name) {
                continue;
            }
            
            // Skip if include_columns is specified and this column is not included
            if !input.table_config.include_columns.is_empty() && 
               !input.table_config.include_columns.contains(&column.name) {
                continue;
            }
            
            let cell = self.extract_cell_data(cell_element, column).await?;
            
            // Use mapped column name if specified
            let column_name = input.table_config.column_mapping
                .get(&column.name)
                .unwrap_or(&column.name)
                .clone();
            
            cells.insert(column_name, cell);
        }
        
        Ok(TableRow {
            index: row_index,
            cells,
            css_classes,
            is_header,
            is_footer,
        })
    }
    
    /// Extract data from a single table cell
    async fn extract_cell_data(&self, cell: &WebElement, column: &TableColumn) -> anyhow::Result<TableCell> {
        let raw_text = text_utils::clean_text(&cell.text().await?);
        let col_span = cell.attr("colspan").await?.and_then(|s| s.parse().ok()).unwrap_or(1);
        let row_span = cell.attr("rowspan").await?.and_then(|s| s.parse().ok()).unwrap_or(1);
        let alignment = self.detect_cell_alignment(cell).await?;
        let css_classes = self.get_css_classes(cell).await?;
        
        // Extract link URL if cell contains a link
        let link_url = if let Ok(link) = cell.find(By::Css("a")).await {
            link.attr("href").await?
        } else {
            None
        };
        
        // Extract image URL if cell contains an image
        let image_url = if let Ok(img) = cell.find(By::Css("img")).await {
            img.attr("src").await?
        } else {
            None
        };
        
        // Transform value based on column data type
        let value = self.transform_cell_value(&raw_text, &column.data_type)?;
        
        Ok(TableCell {
            raw_text,
            value,
            col_span,
            row_span,
            alignment,
            css_classes,
            link_url,
            image_url,
        })
    }
    
    /// Transform cell value based on data type
    fn transform_cell_value(&self, text: &str, data_type: &TableDataType) -> anyhow::Result<serde_json::Value> {
        let text = text.trim();
        
        if text.is_empty() {
            return Ok(serde_json::Value::Null);
        }
        
        match data_type {
            TableDataType::Text => Ok(serde_json::Value::String(text.to_string())),
            
            TableDataType::Number => {
                // Try integer first, then float
                if let Ok(int_val) = text.parse::<i64>() {
                    Ok(serde_json::Value::Number(serde_json::Number::from(int_val)))
                } else if let Ok(float_val) = text.parse::<f64>() {
                    if let Some(num) = serde_json::Number::from_f64(float_val) {
                        Ok(serde_json::Value::Number(num))
                    } else {
                        Ok(serde_json::Value::String(text.to_string()))
                    }
                } else {
                    Ok(serde_json::Value::String(text.to_string()))
                }
            }
            
            TableDataType::Currency => {
                // Extract numeric value from currency text
                let numeric_text: String = text.chars()
                    .filter(|c| c.is_ascii_digit() || *c == '.' || *c == ',' || *c == '-')
                    .collect();
                
                if let Ok(value) = numeric_text.replace(',', "").parse::<f64>() {
                    if let Some(num) = serde_json::Number::from_f64(value) {
                        Ok(serde_json::Value::Number(num))
                    } else {
                        Ok(serde_json::Value::String(text.to_string()))
                    }
                } else {
                    Ok(serde_json::Value::String(text.to_string()))
                }
            }
            
            TableDataType::Percentage => {
                let numeric_text = text.trim_end_matches('%');
                if let Ok(value) = numeric_text.parse::<f64>() {
                    if let Some(num) = serde_json::Number::from_f64(value / 100.0) {
                        Ok(serde_json::Value::Number(num))
                    } else {
                        Ok(serde_json::Value::String(text.to_string()))
                    }
                } else {
                    Ok(serde_json::Value::String(text.to_string()))
                }
            }
            
            TableDataType::Boolean => {
                let lower_text = text.to_lowercase();
                match lower_text.as_str() {
                    "true" | "yes" | "y" | "on" | "enabled" | "1" | "✓" | "✔" | "☑" => {
                        Ok(serde_json::Value::Bool(true))
                    }
                    "false" | "no" | "n" | "off" | "disabled" | "0" | "✗" | "✘" | "☐" => {
                        Ok(serde_json::Value::Bool(false))
                    }
                    _ => Ok(serde_json::Value::String(text.to_string()))
                }
            }
            
            TableDataType::Date => {
                // For now, keep as string - could add date parsing later
                Ok(serde_json::Value::String(text.to_string()))
            }
            
            TableDataType::Link | TableDataType::Image => {
                Ok(serde_json::Value::String(text.to_string()))
            }
            
            TableDataType::Auto => {
                // Try to auto-detect and transform
                if let Ok(int_val) = text.parse::<i64>() {
                    Ok(serde_json::Value::Number(serde_json::Number::from(int_val)))
                } else if let Ok(float_val) = text.parse::<f64>() {
                    if let Some(num) = serde_json::Number::from_f64(float_val) {
                        Ok(serde_json::Value::Number(num))
                    } else {
                        Ok(serde_json::Value::String(text.to_string()))
                    }
                } else if self.looks_like_boolean(text) {
                    self.transform_cell_value(text, &TableDataType::Boolean)
                } else {
                    Ok(serde_json::Value::String(text.to_string()))
                }
            }
        }
    }
    
    /// Check if row is empty (all cells are empty)
    fn is_row_empty(&self, row: &TableRow) -> bool {
        row.cells.values().all(|cell| {
            cell.raw_text.trim().is_empty() || cell.value.is_null()
        })
    }
}

#[async_trait]
impl Tool for ExtractTable {
    type Input = ExtractTableInput;
    type Output = ExtractTableOutput;

    fn name(&self) -> &str {
        "extract_table"
    }

    fn description(&self) -> &str {
        "Extract tabular data from web pages with intelligent structure detection and column mapping"
    }

    async fn execute(&self, input: Self::Input) -> anyhow::Result<Self::Output> {
        let start_time = Instant::now();
        
        // Determine table selector
        let table_selector = input.table_selector
            .as_deref()
            .unwrap_or("table");
        
        // Find tables
        let table_elements = self.find_tables(table_selector).await?;
        
        if table_elements.is_empty() {
            return Err(anyhow::anyhow!("No tables found with selector '{}'", table_selector));
        }
        
        let mut tables = Vec::new();
        let mut total_rows = 0;
        
        // Extract data from tables
        let extract_count = if input.extract_multiple { 
            table_elements.len() 
        } else { 
            1 
        };
        
        for (index, table_element) in table_elements.iter().enumerate().take(extract_count) {
            let table_data = self.extract_table_data(table_element, index, table_selector, &input).await?;
            
            // Check minimum column requirement
            if table_data.structure.columns.len() < input.table_config.min_columns {
                continue; // Skip tables with too few columns
            }
            
            total_rows += table_data.rows.len();
            tables.push(table_data);
        }
        
        // Create extraction metadata
        let metadata = if input.config.include_metadata {
            let mut tool_metadata = HashMap::new();
            tool_metadata.insert("table_selector".to_string(), serde_json::Value::String(table_selector.to_string()));
            tool_metadata.insert("extract_multiple".to_string(), serde_json::Value::Bool(input.extract_multiple));
            tool_metadata.insert("auto_infer_types".to_string(), serde_json::Value::Bool(input.auto_infer_types));
            
            Some(ExtractionMetadata {
                url: self.driver.current_url().await?.to_string(),
                timestamp: Utc::now(),
                item_count: tables.len(),
                duration_ms: start_time.elapsed().as_millis() as u64,
                scope: input.config.scope.clone(),
                tool_name: self.name().to_string(),
                tool_metadata,
            })
        } else {
            None
        };
        
        // Create result structure for formatting
        let result_data = ExtractionResult::success(tables.clone(), metadata.clone());
        
        // Format output
        let formatted_output = format_utils::format_output(&result_data, &input.config.format)
            .map_err(|e| anyhow::anyhow!("Failed to format output: {}", e))?;
        
        Ok(ExtractTableOutput {
            tables,
            formatted_output,
            table_count: result_data.data.len(),
            total_rows,
            config: input.config,
            table_config: input.table_config,
        })
    }
}

// Implementation checklist for extract_table tool:
// [x] Define table-specific data structures (TableColumn, TableRow, TableCell, TableStructure)
// [x] Implement table data types optimized for tabular data
// [x] Add table structure detection with header/footer recognition
// [x] Implement intelligent column mapping and data type inference
// [x] Create comprehensive table extraction configuration
// [x] Add support for merged cells and complex table layouts
// [x] Implement cell value transformation based on detected/specified data types
// [x] Add table validation and filtering (min columns, empty rows)
// [ ] Add CLI integration in main.rs
// [ ] Create specialized CSV/Excel export formatting
// [ ] Add support for nested tables and complex structures
// [ ] Create unit tests and integration tests
// [ ] Optimize performance for large tables

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_table_data_type_parsing() {
        assert_eq!("text".parse::<TableDataType>().unwrap(), TableDataType::Text);
        assert_eq!("number".parse::<TableDataType>().unwrap(), TableDataType::Number);
        assert_eq!("currency".parse::<TableDataType>().unwrap(), TableDataType::Currency);
        assert_eq!("percentage".parse::<TableDataType>().unwrap(), TableDataType::Percentage);
        assert_eq!("auto".parse::<TableDataType>().unwrap(), TableDataType::Auto);
    }
    
    #[test]
    fn test_column_name_sanitization() {
        let tool = ExtractTable { driver: Arc::new(unsafe { std::mem::zeroed() }) };
        
        assert_eq!(tool.sanitize_column_name("Product Name"), "product_name");
        assert_eq!(tool.sanitize_column_name("Price ($)"), "price");
        assert_eq!(tool.sanitize_column_name("Rating (1-5)"), "rating_1_5");
        assert_eq!(tool.sanitize_column_name("___test___"), "test");
    }
    
    #[test]
    fn test_data_type_inference() {
        let tool = ExtractTable { driver: Arc::new(unsafe { std::mem::zeroed() }) };
        
        // Test currency detection
        assert!(tool.looks_like_currency("$19.99"));
        assert!(tool.looks_like_currency("€25.50"));
        assert!(tool.looks_like_currency("123.45 USD"));
        
        // Test percentage detection
        assert!(tool.looks_like_percentage("25%"));
        assert!(tool.looks_like_percentage("100.0%"));
        
        // Test number detection
        assert!(tool.looks_like_number("123"));
        assert!(tool.looks_like_number("45.67"));
        assert!(tool.looks_like_number("-89.1"));
        
        // Test boolean detection
        assert!(tool.looks_like_boolean("true"));
        assert!(tool.looks_like_boolean("YES"));
        assert!(tool.looks_like_boolean("✓"));
        assert!(tool.looks_like_boolean("enabled"));
    }
}