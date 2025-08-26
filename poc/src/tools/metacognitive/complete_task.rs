//! Complete Task tool implementation
//! 
//! Marks a task or objective as complete with results and learnings,
//! providing closure and knowledge capture for AI agents.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::collections::HashMap;
use anyhow::{Result, Context};
use tracing::{info, debug, warn};
use serde_json::{json, Value};
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;

use crate::tools::{Tool, DynamicTool};
use crate::browser::Browser;
use super::CompletionStrategy;

/// Parameters for complete_task tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompleteTaskParams {
    /// Task identifier or description
    pub task_id: String,
    
    /// Task completion status
    pub status: CompletionStatus,
    
    /// Results achieved
    #[serde(skip_serializing_if = "Option::is_none")]
    pub results: Option<TaskResults>,
    
    /// Metrics and measurements
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metrics: Option<TaskMetrics>,
    
    /// Learnings captured
    #[serde(default)]
    pub learnings: Vec<Learning>,
    
    /// Next steps or follow-up tasks
    #[serde(default)]
    pub next_steps: Vec<NextStep>,
    
    /// Artifacts produced
    #[serde(default)]
    pub artifacts: Vec<Artifact>,
    
    /// Summary or notes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    
    /// Tags for categorization
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Task completion status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompletionStatus {
    /// Successfully completed
    Success,
    /// Partially completed
    Partial,
    /// Failed to complete
    Failed,
    /// Cancelled or abandoned
    Cancelled,
    /// Blocked by dependencies
    Blocked,
    /// Deferred to later
    Deferred,
}

/// Task results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResults {
    /// Primary outcome
    pub outcome: String,
    
    /// Success criteria met
    #[serde(default)]
    pub criteria_met: Vec<CriteriaMet>,
    
    /// Deliverables produced
    #[serde(default)]
    pub deliverables: Vec<Deliverable>,
    
    /// Quality score (0.0 to 1.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality_score: Option<f32>,
    
    /// Performance data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub performance: Option<PerformanceData>,
}

/// Criteria that was met
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CriteriaMet {
    pub criterion: String,
    pub met: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence: Option<String>,
}

/// Deliverable produced
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deliverable {
    pub name: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    pub deliverable_type: DeliverableType,
}

/// Type of deliverable
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeliverableType {
    File,
    Data,
    Report,
    Code,
    Documentation,
    Model,
    Other(String),
}

/// Performance data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceData {
    pub execution_time_ms: u64,
    pub cpu_usage_percent: Option<f32>,
    pub memory_usage_mb: Option<f32>,
    pub network_calls: Option<u32>,
    pub cache_hits: Option<u32>,
    pub error_count: u32,
}

/// Task metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskMetrics {
    /// Numeric metrics
    #[serde(default)]
    pub numeric: HashMap<String, f64>,
    
    /// Count metrics
    #[serde(default)]
    pub counts: HashMap<String, u64>,
    
    /// Duration metrics
    #[serde(default)]
    pub durations: HashMap<String, u64>,
    
    /// Boolean flags
    #[serde(default)]
    pub flags: HashMap<String, bool>,
    
    /// Percentage metrics
    #[serde(default)]
    pub percentages: HashMap<String, f32>,
}

/// Learning captured
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Learning {
    /// The learning or insight
    pub insight: String,
    
    /// Category of learning
    pub category: LearningCategory,
    
    /// How to apply this learning
    #[serde(skip_serializing_if = "Option::is_none")]
    pub application: Option<String>,
    
    /// Confidence in this learning
    pub confidence: f32,
}

/// Category of learning
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LearningCategory {
    Technical,
    Process,
    Performance,
    UserBehavior,
    SystemBehavior,
    ErrorHandling,
    Optimization,
    BestPractice,
}

/// Next step or follow-up
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NextStep {
    /// Description of the step
    pub description: String,
    
    /// Priority level
    pub priority: StepPriority,
    
    /// Estimated effort
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_effort: Option<String>,
    
    /// Dependencies
    #[serde(default)]
    pub dependencies: Vec<String>,
    
    /// Due date if applicable
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_date: Option<DateTime<Utc>>,
}

/// Priority of next step
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StepPriority {
    Critical,
    High,
    Medium,
    Low,
    Optional,
}

/// Artifact produced
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    /// Name of the artifact
    pub name: String,
    
    /// Type of artifact
    pub artifact_type: ArtifactType,
    
    /// Location or reference
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    
    /// Size in bytes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_bytes: Option<u64>,
    
    /// Checksum or hash
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checksum: Option<String>,
    
    /// Metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
}

/// Type of artifact
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactType {
    Screenshot,
    Log,
    Data,
    Report,
    Export,
    Backup,
    Configuration,
    Other(String),
}

/// Result of task completion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskCompletionResult {
    /// Unique completion ID
    pub completion_id: String,
    
    /// Task that was completed
    pub task_id: String,
    
    /// Completion timestamp
    pub completed_at: DateTime<Utc>,
    
    /// Status of completion
    pub status: CompletionStatus,
    
    /// Completion strategy used
    pub strategy: CompletionStrategy,
    
    /// Execution summary
    pub execution_summary: ExecutionSummary,
    
    /// Knowledge captured
    pub knowledge_capture: KnowledgeCapture,
    
    /// Recommendations
    pub recommendations: Vec<String>,
    
    /// Quality assessment
    pub quality_assessment: QualityAssessment,
    
    /// Completion confirmation
    pub confirmation: CompletionConfirmation,
}

/// Execution summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionSummary {
    /// Start time
    pub started_at: DateTime<Utc>,
    
    /// End time
    pub ended_at: DateTime<Utc>,
    
    /// Total duration
    pub duration_seconds: f64,
    
    /// Steps executed
    pub steps_executed: u32,
    
    /// Tools used
    pub tools_used: Vec<String>,
    
    /// Resources consumed
    pub resources: ResourceConsumption,
    
    /// Errors encountered
    pub error_count: u32,
    
    /// Retry count
    pub retry_count: u32,
}

/// Resource consumption
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceConsumption {
    pub api_calls: u32,
    pub data_processed_mb: f32,
    pub compute_time_seconds: f64,
    pub memory_peak_mb: f32,
}

/// Knowledge captured
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeCapture {
    /// Number of learnings
    pub learning_count: usize,
    
    /// Patterns identified
    pub patterns: Vec<String>,
    
    /// Best practices discovered
    pub best_practices: Vec<String>,
    
    /// Pitfalls to avoid
    pub pitfalls: Vec<String>,
    
    /// Reusable components
    pub reusable_components: Vec<String>,
}

/// Quality assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityAssessment {
    /// Overall quality score
    pub overall_score: f32,
    
    /// Completeness
    pub completeness: f32,
    
    /// Correctness
    pub correctness: f32,
    
    /// Efficiency
    pub efficiency: f32,
    
    /// Reliability
    pub reliability: f32,
    
    /// Issues identified
    pub issues: Vec<String>,
    
    /// Improvements suggested
    pub improvements: Vec<String>,
}

/// Completion confirmation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionConfirmation {
    /// Confirmation message
    pub message: String,
    
    /// Whether task is fully complete
    pub is_complete: bool,
    
    /// Whether follow-up is needed
    pub follow_up_required: bool,
    
    /// Closure notes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub closure_notes: Option<String>,
}

/// Complete Task tool
pub struct CompleteTask {
    browser: Arc<Browser>,
}

impl CompleteTask {
    /// Create a new CompleteTask tool
    pub fn new(browser: Arc<Browser>) -> Self {
        Self { browser }
    }
    
    /// Process task completion
    async fn process_completion(&self, params: &CompleteTaskParams) -> Result<TaskCompletionResult> {
        // In mock mode, return simulated completion
        if std::env::var("RAINBOW_MOCK_MODE").unwrap_or_default() == "true" {
            return Ok(self.create_mock_completion(params));
        }
        
        // Real implementation would process the task completion
        // For now, return mock data
        Ok(self.create_mock_completion(params))
    }
    
    /// Create mock completion result for testing
    fn create_mock_completion(&self, params: &CompleteTaskParams) -> TaskCompletionResult {
        let completion_id = Uuid::new_v4().to_string();
        let completed_at = Utc::now();
        let started_at = completed_at - Duration::minutes(30);
        
        // Determine strategy based on task characteristics
        let strategy = if params.next_steps.len() > 3 {
            CompletionStrategy::Iterative
        } else if params.artifacts.len() > 1 {
            CompletionStrategy::Parallel
        } else {
            CompletionStrategy::Sequential
        };
        
        // Create execution summary
        let execution_summary = ExecutionSummary {
            started_at,
            ended_at: completed_at,
            duration_seconds: 1800.0,
            steps_executed: 12,
            tools_used: vec![
                "navigate_to_url".to_string(),
                "click_element".to_string(),
                "type_text".to_string(),
                "take_screenshot".to_string(),
            ],
            resources: ResourceConsumption {
                api_calls: 5,
                data_processed_mb: 2.5,
                compute_time_seconds: 150.0,
                memory_peak_mb: 128.0,
            },
            error_count: if matches!(params.status, CompletionStatus::Failed) { 3 } else { 0 },
            retry_count: 1,
        };
        
        // Capture knowledge
        let knowledge_capture = KnowledgeCapture {
            learning_count: params.learnings.len(),
            patterns: if params.learnings.len() > 2 {
                vec![
                    "User interaction pattern identified".to_string(),
                    "Optimal timing for element loading".to_string(),
                ]
            } else {
                vec![]
            },
            best_practices: if matches!(params.status, CompletionStatus::Success) {
                vec![
                    "Wait for element visibility before interaction".to_string(),
                    "Use explicit waits instead of sleep".to_string(),
                ]
            } else {
                vec![]
            },
            pitfalls: if matches!(params.status, CompletionStatus::Failed | CompletionStatus::Partial) {
                vec![
                    "Dynamic content loading issues".to_string(),
                    "Race conditions in async operations".to_string(),
                ]
            } else {
                vec![]
            },
            reusable_components: if !params.artifacts.is_empty() {
                vec!["Element selection strategy".to_string()]
            } else {
                vec![]
            },
        };
        
        // Generate recommendations
        let mut recommendations = Vec::new();
        if matches!(params.status, CompletionStatus::Partial) {
            recommendations.push("Complete remaining subtasks for full completion".to_string());
        }
        if params.metrics.is_some() {
            recommendations.push("Monitor performance metrics for optimization opportunities".to_string());
        }
        if !params.next_steps.is_empty() {
            recommendations.push(format!(
                "Prioritize {} follow-up tasks identified",
                params.next_steps.len()
            ));
        }
        
        // Assess quality
        let quality_assessment = QualityAssessment {
            overall_score: match params.status {
                CompletionStatus::Success => 0.95,
                CompletionStatus::Partial => 0.70,
                CompletionStatus::Failed => 0.30,
                _ => 0.50,
            },
            completeness: match params.status {
                CompletionStatus::Success => 1.0,
                CompletionStatus::Partial => 0.6,
                _ => 0.3,
            },
            correctness: if matches!(params.status, CompletionStatus::Success) {
                0.98
            } else {
                0.75
            },
            efficiency: 0.85,
            reliability: 0.90,
            issues: if matches!(params.status, CompletionStatus::Failed | CompletionStatus::Blocked) {
                vec!["Task incomplete due to errors".to_string()]
            } else {
                vec![]
            },
            improvements: vec![
                "Consider adding more comprehensive error handling".to_string(),
                "Implement retry logic for transient failures".to_string(),
            ],
        };
        
        // Create confirmation
        let confirmation = CompletionConfirmation {
            message: format!(
                "Task '{}' has been marked as {:?}",
                params.task_id, params.status
            ),
            is_complete: matches!(params.status, CompletionStatus::Success),
            follow_up_required: !params.next_steps.is_empty(),
            closure_notes: params.summary.clone(),
        };
        
        TaskCompletionResult {
            completion_id,
            task_id: params.task_id.clone(),
            completed_at,
            status: params.status.clone(),
            strategy,
            execution_summary,
            knowledge_capture,
            recommendations,
            quality_assessment,
            confirmation,
        }
    }
}

#[async_trait]
impl Tool for CompleteTask {
    type Input = CompleteTaskParams;
    type Output = TaskCompletionResult;
    
    fn name(&self) -> &str {
        "complete_task"
    }
    
    fn description(&self) -> &str {
        "Mark a task or objective as complete with results and learnings"
    }
    
    async fn execute(&self, params: Self::Input) -> Result<Self::Output> {
        debug!(
            "Completing task: {} with status {:?}",
            params.task_id, params.status
        );
        
        let start = std::time::Instant::now();
        
        let result = self.process_completion(&params).await?;
        
        let duration = start.elapsed();
        info!(
            "Task {} completed with ID {} in {:?} (quality: {:.2})",
            result.task_id,
            result.completion_id,
            duration,
            result.quality_assessment.overall_score
        );
        
        Ok(result)
    }
    
    fn validate_input(&self, params: &Self::Input) -> Result<()> {
        if params.task_id.trim().is_empty() {
            return Err(anyhow::anyhow!("Task ID cannot be empty"));
        }
        
        if params.task_id.len() > 1000 {
            return Err(anyhow::anyhow!("Task ID cannot exceed 1000 characters"));
        }
        
        if let Some(summary) = &params.summary {
            if summary.len() > 10000 {
                return Err(anyhow::anyhow!("Summary cannot exceed 10000 characters"));
            }
        }
        
        if let Some(results) = &params.results {
            if let Some(quality) = results.quality_score {
                if quality < 0.0 || quality > 1.0 {
                    return Err(anyhow::anyhow!("Quality score must be between 0.0 and 1.0"));
                }
            }
        }
        
        for learning in &params.learnings {
            if learning.confidence < 0.0 || learning.confidence > 1.0 {
                return Err(anyhow::anyhow!("Learning confidence must be between 0.0 and 1.0"));
            }
        }
        
        Ok(())
    }
    
    fn input_schema(&self) -> serde_json::Value {
        json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object",
            "properties": {
                "task_id": {
                    "type": "string",
                    "description": "Task identifier or description",
                    "maxLength": 1000
                },
                "status": {
                    "type": "string",
                    "enum": ["success", "partial", "failed", "cancelled", "blocked", "deferred"]
                },
                "results": {
                    "type": "object",
                    "properties": {
                        "outcome": {"type": "string"},
                        "criteria_met": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "properties": {
                                    "criterion": {"type": "string"},
                                    "met": {"type": "boolean"},
                                    "evidence": {"type": "string"}
                                }
                            }
                        },
                        "deliverables": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "properties": {
                                    "name": {"type": "string"},
                                    "description": {"type": "string"},
                                    "location": {"type": "string"},
                                    "deliverable_type": {"type": "string"}
                                }
                            }
                        },
                        "quality_score": {
                            "type": "number",
                            "minimum": 0.0,
                            "maximum": 1.0
                        }
                    }
                },
                "metrics": {
                    "type": "object",
                    "properties": {
                        "numeric": {
                            "type": "object",
                            "additionalProperties": {"type": "number"}
                        },
                        "counts": {
                            "type": "object",
                            "additionalProperties": {"type": "integer"}
                        },
                        "durations": {
                            "type": "object",
                            "additionalProperties": {"type": "integer"}
                        },
                        "flags": {
                            "type": "object",
                            "additionalProperties": {"type": "boolean"}
                        },
                        "percentages": {
                            "type": "object",
                            "additionalProperties": {"type": "number"}
                        }
                    }
                },
                "learnings": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "insight": {"type": "string"},
                            "category": {
                                "type": "string",
                                "enum": ["technical", "process", "performance", "user_behavior", "system_behavior", "error_handling", "optimization", "best_practice"]
                            },
                            "application": {"type": "string"},
                            "confidence": {
                                "type": "number",
                                "minimum": 0.0,
                                "maximum": 1.0
                            }
                        },
                        "required": ["insight", "category", "confidence"]
                    }
                },
                "next_steps": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "description": {"type": "string"},
                            "priority": {
                                "type": "string",
                                "enum": ["critical", "high", "medium", "low", "optional"]
                            },
                            "estimated_effort": {"type": "string"},
                            "dependencies": {
                                "type": "array",
                                "items": {"type": "string"}
                            }
                        },
                        "required": ["description", "priority"]
                    }
                },
                "artifacts": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "name": {"type": "string"},
                            "artifact_type": {"type": "string"},
                            "location": {"type": "string"},
                            "size_bytes": {"type": "integer"}
                        },
                        "required": ["name", "artifact_type"]
                    }
                },
                "summary": {
                    "type": "string",
                    "maxLength": 10000
                },
                "tags": {
                    "type": "array",
                    "items": {"type": "string"}
                }
            },
            "required": ["task_id", "status"]
        })
    }
    
    fn output_schema(&self) -> serde_json::Value {
        json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object",
            "properties": {
                "completion_id": {"type": "string"},
                "task_id": {"type": "string"},
                "completed_at": {"type": "string", "format": "date-time"},
                "status": {"type": "string"},
                "strategy": {"type": "string"},
                "execution_summary": {
                    "type": "object",
                    "properties": {
                        "started_at": {"type": "string", "format": "date-time"},
                        "ended_at": {"type": "string", "format": "date-time"},
                        "duration_seconds": {"type": "number"},
                        "steps_executed": {"type": "integer"},
                        "tools_used": {
                            "type": "array",
                            "items": {"type": "string"}
                        },
                        "error_count": {"type": "integer"},
                        "retry_count": {"type": "integer"}
                    }
                },
                "knowledge_capture": {
                    "type": "object",
                    "properties": {
                        "learning_count": {"type": "integer"},
                        "patterns": {
                            "type": "array",
                            "items": {"type": "string"}
                        },
                        "best_practices": {
                            "type": "array",
                            "items": {"type": "string"}
                        }
                    }
                },
                "recommendations": {
                    "type": "array",
                    "items": {"type": "string"}
                },
                "quality_assessment": {
                    "type": "object",
                    "properties": {
                        "overall_score": {"type": "number"},
                        "completeness": {"type": "number"},
                        "correctness": {"type": "number"},
                        "efficiency": {"type": "number"},
                        "reliability": {"type": "number"}
                    }
                },
                "confirmation": {
                    "type": "object",
                    "properties": {
                        "message": {"type": "string"},
                        "is_complete": {"type": "boolean"},
                        "follow_up_required": {"type": "boolean"}
                    }
                }
            },
            "required": ["completion_id", "task_id", "completed_at", "status"]
        })
    }
}

// Implement DynamicTool for runtime dispatch
#[async_trait]
impl DynamicTool for CompleteTask {
    fn name(&self) -> &str {
        Tool::name(self)
    }
    
    async fn execute_json(&self, params: serde_json::Value) -> Result<serde_json::Value> {
        let input: CompleteTaskParams = serde_json::from_value(params)
            .context("Failed to parse CompleteTask parameters")?;
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