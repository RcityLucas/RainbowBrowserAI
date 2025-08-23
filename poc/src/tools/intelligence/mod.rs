// Intelligence tools for AI-driven insights and task management
//
// This module contains meta-cognitive tools that provide AI-driven
// pattern recognition, insights, and task completion tracking.

// TODO: Implement these modules in Phase 3
// pub mod report_insight;
// pub mod complete_task;

// Re-export tools for easy access (commented out until implementation)
// pub use report_insight::{ReportInsight, ReportInsightInput, ReportInsightOutput};
// pub use complete_task::{CompleteTask, CompleteTaskInput, CompleteTaskOutput};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// Types of insights that can be reported
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InsightType {
    /// Detected behavioral pattern
    Pattern,
    /// Unusual or unexpected behavior
    Anomaly,
    /// Performance improvement opportunity
    Optimization,
    /// Potential issue or risk
    Warning,
    /// New discovery or finding
    Discovery,
    /// Predictive insight about future behavior
    Prediction,
}

/// Severity level of an insight
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum InsightSeverity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

/// Evidence supporting an insight
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    pub evidence_type: EvidenceType,
    pub description: String,
    pub value: serde_json::Value,
    pub source: String,
    pub timestamp: DateTime<Utc>,
}

/// Types of evidence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvidenceType {
    Screenshot,
    Data,
    Observation,
    Metric,
    Log,
}

/// Recommendation based on insight
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    pub action: String,
    pub reason: String,
    pub priority: u8, // 1-5 scale
    pub estimated_impact: String,
    pub implementation_difficulty: u8, // 1-5 scale
}

/// Task completion status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskStatus {
    Success,
    Partial,
    Failure,
    Cancelled,
    Timeout,
}

/// Task execution metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskMetrics {
    pub duration_ms: u64,
    pub actions_count: u32,
    pub perceptions_count: u32,
    pub retries: u32,
    
    pub resource_usage: ResourceUsage,
    pub performance_score: Option<f32>, // 0-100
    pub efficiency_score: Option<f32>,  // 0-100
    pub quality_score: Option<f32>,     // 0-100
}

/// Resource usage during task execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub cpu_percent: f32,
    pub memory_mb: f32,
    pub network_kb: f32,
}

/// Task artifacts (outputs produced)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskArtifact {
    pub artifact_type: ArtifactType,
    pub name: String,
    pub content: serde_json::Value,
    pub mime_type: Option<String>,
    pub size_bytes: u64,
}

/// Types of artifacts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArtifactType {
    Screenshot,
    Data,
    Report,
    File,
    Log,
}

/// Follow-up task information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FollowUpTask {
    pub task_type: String,
    pub description: String,
    pub priority: u8, // 1-5 scale
    pub deadline: Option<DateTime<Utc>>,
    pub dependencies: Vec<String>,
}