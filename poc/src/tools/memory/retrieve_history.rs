//! Retrieve History tool implementation
//! 
//! Retrieves browser history, navigation history, and action logs from the current session,
//! with filtering and pagination capabilities.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use anyhow::{Result, Context};
use tracing::{info, debug, warn};
use serde_json::json;
use chrono::{DateTime, Utc, Duration};

use crate::tools::{Tool, DynamicTool};
use crate::browser::Browser;

/// Parameters for retrieve_history tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrieveHistoryParams {
    /// Type of history to retrieve
    #[serde(default = "default_history_type")]
    pub history_type: HistoryType,
    
    /// Optional filter criteria
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<HistoryFilter>,
    
    /// Pagination options
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pagination: Option<PaginationOptions>,
    
    /// Include detailed information
    #[serde(default)]
    pub include_details: bool,
}

fn default_history_type() -> HistoryType {
    HistoryType::Navigation
}

/// Type of history to retrieve
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HistoryType {
    /// Browser navigation history (URLs visited)
    Navigation,
    /// Action history (tools executed)
    Actions,
    /// Combined history
    Combined,
    /// Form interactions
    Forms,
    /// Downloads
    Downloads,
    /// Errors and failures
    Errors,
}

/// Filter criteria for history retrieval
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryFilter {
    /// Filter by time range
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_range: Option<TimeRange>,
    
    /// Filter by URL pattern
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url_pattern: Option<String>,
    
    /// Filter by domain
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
    
    /// Filter by action type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action_type: Option<String>,
    
    /// Filter by success status
    #[serde(skip_serializing_if = "Option::is_none")]
    pub success_only: Option<bool>,
    
    /// Search query
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,
    
    /// Tags to filter by
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Time range for filtering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start: Option<DateTime<Utc>>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<DateTime<Utc>>,
    
    /// Relative time (e.g., "last_hour", "last_day", "last_week")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relative: Option<RelativeTime>,
}

/// Relative time periods
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelativeTime {
    LastHour,
    LastDay,
    LastWeek,
    LastMonth,
    LastSession,
    Current,
}

/// Pagination options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationOptions {
    #[serde(default = "default_page_size")]
    pub page_size: usize,
    
    #[serde(default)]
    pub page: usize,
    
    #[serde(default = "default_sort_order")]
    pub sort_order: SortOrder,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

fn default_page_size() -> usize {
    50
}

fn default_sort_order() -> SortOrder {
    SortOrder::Descending
}

/// Sort order for results
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SortOrder {
    Ascending,
    Descending,
}

/// History retrieval result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryResult {
    /// History entries
    pub entries: Vec<HistoryEntry>,
    
    /// Total number of entries (before pagination)
    pub total_count: usize,
    
    /// Number of entries returned
    pub returned_count: usize,
    
    /// Pagination information
    pub pagination: PaginationInfo,
    
    /// Summary statistics
    pub summary: HistorySummary,
    
    /// Applied filters
    pub applied_filters: AppliedFilters,
}

/// Individual history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    /// Unique ID for this entry
    pub id: String,
    
    /// Timestamp of the event
    pub timestamp: DateTime<Utc>,
    
    /// Type of entry
    pub entry_type: EntryType,
    
    /// URL associated with this entry
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    
    /// Title of the page
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    
    /// Action performed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<ActionInfo>,
    
    /// Duration in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
    
    /// Success status
    pub success: bool,
    
    /// Error message if failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    
    /// Additional metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    
    /// Tags associated with this entry
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Type of history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntryType {
    Navigation,
    Click,
    Type,
    Submit,
    Screenshot,
    Download,
    Error,
    Custom(String),
}

/// Information about an action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionInfo {
    pub tool_name: String,
    pub parameters: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
}

/// Pagination information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationInfo {
    pub current_page: usize,
    pub total_pages: usize,
    pub has_next: bool,
    pub has_previous: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_cursor: Option<String>,
}

/// Summary statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistorySummary {
    pub total_navigations: usize,
    pub total_actions: usize,
    pub unique_domains: usize,
    pub success_rate: f32,
    pub average_duration_ms: f64,
    pub time_span: TimeSpan,
    pub most_visited: Vec<MostVisited>,
    pub most_used_tools: Vec<ToolUsage>,
}

/// Time span information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSpan {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub duration_minutes: f64,
}

/// Most visited sites
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MostVisited {
    pub domain: String,
    pub count: usize,
    pub average_time_ms: f64,
}

/// Tool usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolUsage {
    pub tool_name: String,
    pub count: usize,
    pub success_rate: f32,
    pub average_duration_ms: f64,
}

/// Applied filters information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppliedFilters {
    pub history_type: HistoryType,
    pub filter_count: usize,
    pub filters: Vec<String>,
}

/// Retrieve History tool
pub struct RetrieveHistory {
    browser: Arc<Browser>,
}

impl RetrieveHistory {
    /// Create a new RetrieveHistory tool
    pub fn new(browser: Arc<Browser>) -> Self {
        Self { browser }
    }
    
    /// Retrieve history based on parameters
    async fn fetch_history(&self, params: &RetrieveHistoryParams) -> Result<HistoryResult> {
        // In mock mode, return simulated history
        if std::env::var("RAINBOW_MOCK_MODE").unwrap_or_default() == "true" {
            return Ok(self.create_mock_history(params));
        }
        
        // Real implementation would interact with browser
        // For now, return mock data
        Ok(self.create_mock_history(params))
    }
    
    /// Create mock history for testing
    fn create_mock_history(&self, params: &RetrieveHistoryParams) -> HistoryResult {
        let now = Utc::now();
        let page_size = params.pagination.as_ref().map(|p| p.page_size).unwrap_or(50);
        let current_page = params.pagination.as_ref().map(|p| p.page).unwrap_or(0);
        
        // Create mock entries based on history type
        let mut entries = Vec::new();
        
        match params.history_type {
            HistoryType::Navigation => {
                entries.push(HistoryEntry {
                    id: "nav-1".to_string(),
                    timestamp: now - Duration::minutes(5),
                    entry_type: EntryType::Navigation,
                    url: Some("https://example.com".to_string()),
                    title: Some("Example Domain".to_string()),
                    action: None,
                    duration_ms: Some(1250),
                    success: true,
                    error: None,
                    metadata: if params.include_details {
                        Some(json!({
                            "status_code": 200,
                            "content_type": "text/html",
                            "referrer": "https://google.com"
                        }))
                    } else {
                        None
                    },
                    tags: vec!["navigation".to_string()],
                });
                
                entries.push(HistoryEntry {
                    id: "nav-2".to_string(),
                    timestamp: now - Duration::minutes(3),
                    entry_type: EntryType::Navigation,
                    url: Some("https://example.com/about".to_string()),
                    title: Some("About - Example Domain".to_string()),
                    action: None,
                    duration_ms: Some(800),
                    success: true,
                    error: None,
                    metadata: None,
                    tags: vec!["navigation".to_string()],
                });
            },
            HistoryType::Actions => {
                entries.push(HistoryEntry {
                    id: "action-1".to_string(),
                    timestamp: now - Duration::minutes(4),
                    entry_type: EntryType::Click,
                    url: Some("https://example.com".to_string()),
                    title: None,
                    action: Some(ActionInfo {
                        tool_name: "click_element".to_string(),
                        parameters: json!({
                            "selector": "#submit-button"
                        }),
                        result: if params.include_details {
                            Some(json!({
                                "success": true,
                                "element_found": true
                            }))
                        } else {
                            None
                        },
                    }),
                    duration_ms: Some(150),
                    success: true,
                    error: None,
                    metadata: None,
                    tags: vec!["interaction".to_string()],
                });
                
                entries.push(HistoryEntry {
                    id: "action-2".to_string(),
                    timestamp: now - Duration::minutes(2),
                    entry_type: EntryType::Type,
                    url: Some("https://example.com/form".to_string()),
                    title: None,
                    action: Some(ActionInfo {
                        tool_name: "type_text".to_string(),
                        parameters: json!({
                            "selector": "#username",
                            "text": "testuser"
                        }),
                        result: None,
                    }),
                    duration_ms: Some(500),
                    success: true,
                    error: None,
                    metadata: None,
                    tags: vec!["form".to_string(), "input".to_string()],
                });
            },
            HistoryType::Combined => {
                // Add both navigation and action entries
                entries.push(HistoryEntry {
                    id: "combined-1".to_string(),
                    timestamp: now - Duration::minutes(10),
                    entry_type: EntryType::Navigation,
                    url: Some("https://example.com".to_string()),
                    title: Some("Example Domain".to_string()),
                    action: None,
                    duration_ms: Some(1000),
                    success: true,
                    error: None,
                    metadata: None,
                    tags: vec!["navigation".to_string()],
                });
                
                entries.push(HistoryEntry {
                    id: "combined-2".to_string(),
                    timestamp: now - Duration::minutes(8),
                    entry_type: EntryType::Screenshot,
                    url: Some("https://example.com".to_string()),
                    title: None,
                    action: Some(ActionInfo {
                        tool_name: "take_screenshot".to_string(),
                        parameters: json!({
                            "format": "png"
                        }),
                        result: None,
                    }),
                    duration_ms: Some(200),
                    success: true,
                    error: None,
                    metadata: None,
                    tags: vec!["capture".to_string()],
                });
            },
            _ => {
                // For other types, return sample entries
                entries.push(HistoryEntry {
                    id: format!("{:?}-1", params.history_type),
                    timestamp: now - Duration::minutes(1),
                    entry_type: EntryType::Custom(format!("{:?}", params.history_type)),
                    url: Some("https://example.com".to_string()),
                    title: Some("Sample Entry".to_string()),
                    action: None,
                    duration_ms: Some(100),
                    success: true,
                    error: None,
                    metadata: None,
                    tags: vec![],
                });
            }
        }
        
        // Apply filters if provided
        if let Some(filter) = &params.filter {
            // Mock filtering logic
            if let Some(domain) = &filter.domain {
                entries.retain(|e| {
                    e.url.as_ref()
                        .map(|u| u.contains(domain))
                        .unwrap_or(false)
                });
            }
            
            if let Some(true) = filter.success_only {
                entries.retain(|e| e.success);
            }
        }
        
        let total_count = entries.len();
        let total_pages = (total_count + page_size - 1) / page_size;
        
        // Apply pagination
        let start = current_page * page_size;
        let end = std::cmp::min(start + page_size, total_count);
        let paginated_entries: Vec<_> = entries.into_iter().skip(start).take(page_size).collect();
        let returned_count = paginated_entries.len();
        
        HistoryResult {
            entries: paginated_entries,
            total_count,
            returned_count,
            pagination: PaginationInfo {
                current_page,
                total_pages,
                has_next: current_page < total_pages - 1,
                has_previous: current_page > 0,
                next_cursor: if current_page < total_pages - 1 {
                    Some(format!("page_{}", current_page + 1))
                } else {
                    None
                },
                previous_cursor: if current_page > 0 {
                    Some(format!("page_{}", current_page - 1))
                } else {
                    None
                },
            },
            summary: HistorySummary {
                total_navigations: 2,
                total_actions: 3,
                unique_domains: 1,
                success_rate: 1.0,
                average_duration_ms: 570.0,
                time_span: TimeSpan {
                    start: now - Duration::minutes(10),
                    end: now,
                    duration_minutes: 10.0,
                },
                most_visited: vec![
                    MostVisited {
                        domain: "example.com".to_string(),
                        count: 5,
                        average_time_ms: 900.0,
                    }
                ],
                most_used_tools: vec![
                    ToolUsage {
                        tool_name: "navigate_to_url".to_string(),
                        count: 2,
                        success_rate: 1.0,
                        average_duration_ms: 1025.0,
                    },
                    ToolUsage {
                        tool_name: "click_element".to_string(),
                        count: 1,
                        success_rate: 1.0,
                        average_duration_ms: 150.0,
                    },
                ],
            },
            applied_filters: AppliedFilters {
                history_type: params.history_type.clone(),
                filter_count: if params.filter.is_some() { 1 } else { 0 },
                filters: if params.filter.is_some() {
                    vec!["custom_filter".to_string()]
                } else {
                    vec![]
                },
            },
        }
    }
}

#[async_trait]
impl Tool for RetrieveHistory {
    type Input = RetrieveHistoryParams;
    type Output = HistoryResult;
    
    fn name(&self) -> &str {
        "retrieve_history"
    }
    
    fn description(&self) -> &str {
        "Retrieve browser history, navigation history, and action logs with filtering and pagination"
    }
    
    async fn execute(&self, params: Self::Input) -> Result<Self::Output> {
        debug!("Retrieving history with type: {:?}", params.history_type);
        let start = std::time::Instant::now();
        
        let result = self.fetch_history(&params).await?;
        
        let duration = start.elapsed();
        info!(
            "Retrieved {} history entries (total: {}) in {:?}",
            result.returned_count,
            result.total_count,
            duration
        );
        
        Ok(result)
    }
    
    fn validate_input(&self, params: &Self::Input) -> Result<()> {
        if let Some(pagination) = &params.pagination {
            if pagination.page_size == 0 {
                return Err(anyhow::anyhow!("Page size must be greater than 0"));
            }
            if pagination.page_size > 1000 {
                return Err(anyhow::anyhow!("Page size cannot exceed 1000"));
            }
        }
        
        if let Some(filter) = &params.filter {
            if let Some(time_range) = &filter.time_range {
                if let (Some(start), Some(end)) = (&time_range.start, &time_range.end) {
                    if start > end {
                        return Err(anyhow::anyhow!("Start time must be before end time"));
                    }
                }
            }
        }
        
        Ok(())
    }
    
    fn input_schema(&self) -> serde_json::Value {
        json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object",
            "properties": {
                "history_type": {
                    "type": "string",
                    "enum": ["navigation", "actions", "combined", "forms", "downloads", "errors"],
                    "default": "navigation"
                },
                "filter": {
                    "type": "object",
                    "properties": {
                        "time_range": {
                            "type": "object",
                            "properties": {
                                "start": {"type": "string", "format": "date-time"},
                                "end": {"type": "string", "format": "date-time"},
                                "relative": {
                                    "type": "string",
                                    "enum": ["last_hour", "last_day", "last_week", "last_month", "last_session", "current"]
                                }
                            }
                        },
                        "url_pattern": {"type": "string"},
                        "domain": {"type": "string"},
                        "action_type": {"type": "string"},
                        "success_only": {"type": "boolean"},
                        "query": {"type": "string"},
                        "tags": {
                            "type": "array",
                            "items": {"type": "string"}
                        }
                    }
                },
                "pagination": {
                    "type": "object",
                    "properties": {
                        "page_size": {
                            "type": "integer",
                            "minimum": 1,
                            "maximum": 1000,
                            "default": 50
                        },
                        "page": {
                            "type": "integer",
                            "minimum": 0,
                            "default": 0
                        },
                        "sort_order": {
                            "type": "string",
                            "enum": ["ascending", "descending"],
                            "default": "descending"
                        },
                        "cursor": {"type": "string"}
                    }
                },
                "include_details": {
                    "type": "boolean",
                    "default": false
                }
            }
        })
    }
    
    fn output_schema(&self) -> serde_json::Value {
        json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object",
            "properties": {
                "entries": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "id": {"type": "string"},
                            "timestamp": {"type": "string", "format": "date-time"},
                            "entry_type": {"type": "string"},
                            "url": {"type": ["string", "null"]},
                            "title": {"type": ["string", "null"]},
                            "success": {"type": "boolean"}
                        }
                    }
                },
                "total_count": {"type": "integer"},
                "returned_count": {"type": "integer"},
                "pagination": {
                    "type": "object",
                    "properties": {
                        "current_page": {"type": "integer"},
                        "total_pages": {"type": "integer"},
                        "has_next": {"type": "boolean"},
                        "has_previous": {"type": "boolean"}
                    }
                },
                "summary": {
                    "type": "object",
                    "properties": {
                        "total_navigations": {"type": "integer"},
                        "total_actions": {"type": "integer"},
                        "success_rate": {"type": "number"}
                    }
                }
            },
            "required": ["entries", "total_count", "returned_count", "pagination", "summary"]
        })
    }
}

// Implement DynamicTool for runtime dispatch
#[async_trait]
impl DynamicTool for RetrieveHistory {
    fn name(&self) -> &str {
        Tool::name(self)
    }
    
    async fn execute_json(&self, params: serde_json::Value) -> Result<serde_json::Value> {
        let input: RetrieveHistoryParams = serde_json::from_value(params)
            .context("Failed to parse RetrieveHistory parameters")?;
        let output = self.execute(input).await?;
        Ok(serde_json::to_value(output)?)
    }
    
    fn input_schema(&self) -> serde_json::Value {
        Tool::input_schema(self)
    }
    
    fn output_schema(&self) -> serde_json::Value {
        Tool::output_schema(self)
    }
}