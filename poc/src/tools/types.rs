//! Common types used across all tools

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

// ============================================================================
// Navigation Types
// ============================================================================

/// Wait strategies for page loading
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum WaitUntil {
    /// Wait for the 'load' event
    Load,
    /// Wait for the 'DOMContentLoaded' event
    DomContentLoaded,
    /// Wait until there are no network connections for at least 500ms
    NetworkIdle0,
    /// Wait until there are 2 or fewer network connections for at least 500ms
    NetworkIdle2,
}

impl Default for WaitUntil {
    fn default() -> Self {
        Self::Load
    }
}

/// Information about a redirect
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedirectInfo {
    pub from: String,
    pub to: String,
    pub status_code: u16,
}

/// Performance metrics for navigation
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PerformanceMetrics {
    pub dns_lookup: u64,
    pub tcp_connect: u64,
    pub request_sent: u64,
    pub response_received: u64,
    pub dom_loaded: u64,
    pub page_loaded: u64,
}

/// Scroll direction options
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ScrollDirection {
    /// Basic directions
    Simple(SimpleScrollDirection),
    /// Scroll to a specific element
    ToElement { selector: String },
    /// Scroll to a specific position
    ToPosition { x: i32, y: i32 },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SimpleScrollDirection {
    Up,
    Down,
    Left,
    Right,
    Top,
    Bottom,
}

/// Position in the document
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

/// Size dimensions
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

/// Viewport information
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct Viewport {
    pub width: u32,
    pub height: u32,
}

/// Document dimensions
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct DocumentSize {
    pub width: u32,
    pub height: u32,
}

/// Boundary reached status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct ReachedBoundary {
    pub top: bool,
    pub bottom: bool,
    pub left: bool,
    pub right: bool,
}

// ============================================================================
// Interaction Types
// ============================================================================

/// Mouse button types
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

impl Default for MouseButton {
    fn default() -> Self {
        Self::Left
    }
}

/// Keyboard modifier keys
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum KeyModifier {
    Alt,
    Control,
    Meta,
    Shift,
}

/// Offset from element position
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Offset {
    pub x: i32,
    pub y: i32,
}

/// Element bounding box
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct BoundingBox {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
    pub left: f64,
}

/// Click effects analysis
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ClickEffects {
    pub navigation_triggered: bool,
    pub form_submitted: bool,
    pub popup_opened: bool,
    pub dom_changed: bool,
}

/// Timing information for click operation
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ClickTiming {
    pub element_found: u64,
    pub click_executed: u64,
    pub total: u64,
}

/// Select by strategy
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SelectBy {
    Value,
    Text,
    Index,
}

impl Default for SelectBy {
    fn default() -> Self {
        Self::Value
    }
}

// ============================================================================
// Synchronization Types
// ============================================================================

/// Element state for waiting
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ElementState {
    Attached,
    Detached,
    Visible,
    Hidden,
}

/// State change record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateChange {
    pub timestamp: DateTime<Utc>,
    pub from_state: ElementState,
    pub to_state: ElementState,
}

/// Wait condition types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum WaitCondition {
    /// Wait for URL to contain a string
    UrlContains { url_contains: String },
    /// Wait for URL to equal a string
    UrlEquals { url_equals: String },
    /// Wait for URL to match a pattern
    UrlMatches { url_matches: String },
    /// Wait for title to contain a string
    TitleContains { title_contains: String },
    /// Wait for title to equal a string
    TitleEquals { title_equals: String },
    /// Wait for specific element count
    ElementCount { 
        selector: String,
        count: usize,
    },
    /// Custom JavaScript condition
    Custom { 
        script: String,
        args: Option<Vec<serde_json::Value>>,
    },
}

/// Evaluation record for condition checking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationRecord {
    pub timestamp: DateTime<Utc>,
    pub value: serde_json::Value,
    pub met: bool,
}

// ============================================================================
// Memory Types
// ============================================================================

/// Basic element information
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ElementInfo {
    // Basic information
    pub tag_name: String,
    pub text_content: String,
    pub inner_text: String,
    pub inner_html: String,
    pub outer_html: String,
    
    // Identification
    pub id: String,
    pub class_list: Vec<String>,
    pub unique_id: String,
    
    // Position and size
    pub bounding_box: BoundingBox,
    
    // State
    pub is_visible: bool,
    pub is_enabled: bool,
    pub is_selected: bool,
    pub is_focused: bool,
    pub is_in_viewport: bool,
    
    // Optional detailed information
    pub attributes: Option<HashMap<String, String>>,
    pub dataset: Option<HashMap<String, String>>,
    pub computed_style: Option<HashMap<String, String>>,
    
    // Element-specific
    pub input_type: Option<String>,
    pub input_value: Option<String>,
    pub href: Option<String>,
    pub src: Option<String>,
    
    // Relationships
    pub parent: Option<Box<ElementInfo>>,
    pub children: Option<Vec<ElementInfo>>,
    
    // Screenshot
    pub screenshot: Option<ElementScreenshot>,
}

/// Element screenshot data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementScreenshot {
    pub format: String,
    pub data: String, // Base64 encoded
}

/// Screenshot type options
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ScreenshotType {
    Viewport,
    FullPage,
    Element,
}

impl Default for ScreenshotType {
    fn default() -> Self {
        Self::Viewport
    }
}

/// Image format options
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ImageFormat {
    Png,
    Jpeg,
    Webp,
}

impl Default for ImageFormat {
    fn default() -> Self {
        Self::Png
    }
}

impl ImageFormat {
    pub fn to_string(&self) -> &str {
        match self {
            Self::Png => "png",
            Self::Jpeg => "jpeg",
            Self::Webp => "webp",
        }
    }
}

/// Rectangle for clipping
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Rectangle {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// Highlight style for screenshots
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Highlight {
    pub selector: String,
    pub style: Option<HighlightStyle>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighlightStyle {
    pub border: Option<String>,
    pub background: Option<String>,
    pub opacity: Option<f32>,
}

/// Screenshot metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenshotMetadata {
    pub timestamp: DateTime<Utc>,
    pub url: String,
    pub device_pixel_ratio: f64,
    pub color_space: String,
}

/// History entry type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum HistoryType {
    Navigation,
    Action,
    Perception,
    All,
}

impl Default for HistoryType {
    fn default() -> Self {
        Self::All
    }
}

/// Action type for history
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ActionType {
    Click,
    TypeText,
    SelectOption,
    Navigate,
    Scroll,
    Screenshot,
    Wait,
}

// ============================================================================
// Meta-cognitive Types
// ============================================================================

/// Insight type classification
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum InsightType {
    Pattern,
    Anomaly,
    Optimization,
    Prediction,
    Discovery,
    Warning,
}

/// Insight severity levels
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd)]
#[serde(rename_all = "snake_case")]
pub enum InsightSeverity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

/// Evidence for insights
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    #[serde(rename = "type")]
    pub evidence_type: EvidenceType,
    pub description: String,
    pub value: serde_json::Value,
    pub source: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceType {
    Screenshot,
    Data,
    Observation,
    Metric,
}

/// Recommendation for insights
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    pub action: String,
    pub reason: String,
    pub priority: u8, // 1-5
    pub estimated_impact: String,
}

/// Task completion status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Success,
    Partial,
    Failure,
    Cancelled,
    Timeout,
}

/// Task error information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskError {
    pub code: String,
    pub message: String,
    pub recoverable: bool,
    pub retry_after: Option<u64>, // seconds
}

/// Task artifact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskArtifact {
    #[serde(rename = "type")]
    pub artifact_type: ArtifactType,
    pub name: String,
    pub content: serde_json::Value,
    pub mime_type: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactType {
    Screenshot,
    Data,
    Report,
    File,
}

/// Follow-up task information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FollowUpTask {
    #[serde(rename = "type")]
    pub task_type: String,
    pub description: String,
    pub priority: u8,
    pub deadline: Option<DateTime<Utc>>,
}