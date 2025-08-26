//! Report Insight tool implementation
//! 
//! Reports patterns, learnings, or optimization opportunities discovered during execution,
//! enabling AI agents to share meta-cognitive observations.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use anyhow::{Result, Context};
use tracing::{info, debug, warn};
use serde_json::{json, Value};
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::tools::{Tool, DynamicTool};
use crate::browser::Browser;
use super::{InsightCategory, CognitionLevel};

/// Parameters for report_insight tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportInsightParams {
    /// The insight being reported
    pub insight: String,
    
    /// Category of the insight
    pub category: InsightCategory,
    
    /// Confidence level (0.0 to 1.0)
    pub confidence: f32,
    
    /// Supporting evidence or data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence: Option<Evidence>,
    
    /// Recommended actions based on the insight
    #[serde(default)]
    pub recommendations: Vec<Recommendation>,
    
    /// Priority level (1-5, 5 being highest)
    #[serde(default = "default_priority")]
    pub priority: u8,
    
    /// Tags for categorization
    #[serde(default)]
    pub tags: Vec<String>,
    
    /// Context information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<InsightContext>,
}

fn default_priority() -> u8 {
    3
}

/// Evidence supporting the insight
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    /// Type of evidence
    pub evidence_type: EvidenceType,
    
    /// Evidence data
    pub data: Value,
    
    /// Source of the evidence
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    
    /// Metrics if applicable
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metrics: Option<HashMap<String, f64>>,
    
    /// Sample data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub samples: Option<Vec<Value>>,
}

/// Type of evidence
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceType {
    /// Performance metrics
    Metrics,
    /// Pattern observation
    Pattern,
    /// Statistical analysis
    Statistical,
    /// User behavior
    Behavioral,
    /// System logs
    Logs,
    /// Comparison data
    Comparative,
    /// Experimental results
    Experimental,
}

/// Recommendation based on insight
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    /// Action to take
    pub action: String,
    
    /// Expected impact
    pub expected_impact: String,
    
    /// Implementation complexity (low, medium, high)
    pub complexity: ComplexityLevel,
    
    /// Estimated effort in hours
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effort_hours: Option<f32>,
    
    /// Dependencies for this recommendation
    #[serde(default)]
    pub dependencies: Vec<String>,
}

/// Complexity level
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ComplexityLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Context for the insight
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsightContext {
    /// Current URL or location
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    
    /// Current workflow or task
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workflow: Option<String>,
    
    /// Session information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    
    /// Related tools used
    #[serde(default)]
    pub tools_used: Vec<String>,
    
    /// Time range of observation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_range: Option<TimeRange>,
    
    /// Environmental factors
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment: Option<HashMap<String, String>>,
}

/// Time range for observations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub duration_seconds: f64,
}

/// Result of reporting an insight
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsightReport {
    /// Unique ID for this insight
    pub id: String,
    
    /// Timestamp when reported
    pub timestamp: DateTime<Utc>,
    
    /// The insight that was reported
    pub insight: String,
    
    /// Category
    pub category: InsightCategory,
    
    /// Confidence level
    pub confidence: f32,
    
    /// Priority
    pub priority: u8,
    
    /// Cognition level demonstrated
    pub cognition_level: CognitionLevel,
    
    /// Whether the insight was stored
    pub stored: bool,
    
    /// Whether the insight triggered any actions
    pub actions_triggered: Vec<TriggeredAction>,
    
    /// Related insights
    pub related_insights: Vec<RelatedInsight>,
    
    /// Impact assessment
    pub impact_assessment: ImpactAssessment,
    
    /// Acknowledgment
    pub acknowledgment: Acknowledgment,
}

/// Action triggered by the insight
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggeredAction {
    pub action_type: String,
    pub description: String,
    pub status: ActionStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
}

/// Status of a triggered action
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Skipped,
}

/// Related insight reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelatedInsight {
    pub id: String,
    pub category: InsightCategory,
    pub similarity_score: f32,
    pub relationship_type: RelationshipType,
}

/// Type of relationship between insights
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationshipType {
    Reinforces,
    Contradicts,
    Extends,
    Precedes,
    Follows,
    Related,
}

/// Impact assessment of the insight
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactAssessment {
    /// Immediate impact
    pub immediate: ImpactLevel,
    
    /// Long-term impact
    pub long_term: ImpactLevel,
    
    /// Scope of impact
    pub scope: ImpactScope,
    
    /// Estimated value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_value: Option<f64>,
    
    /// Risk mitigation value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub risk_mitigation: Option<f64>,
}

/// Level of impact
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImpactLevel {
    None,
    Low,
    Medium,
    High,
    Critical,
}

/// Scope of impact
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImpactScope {
    Local,
    Module,
    System,
    Global,
}

/// Acknowledgment of the insight
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Acknowledgment {
    pub message: String,
    pub action_required: bool,
    pub follow_up: Option<String>,
}

use std::collections::HashMap;

/// Report Insight tool
pub struct ReportInsight {
    browser: Arc<Browser>,
}

impl ReportInsight {
    /// Create a new ReportInsight tool
    pub fn new(browser: Arc<Browser>) -> Self {
        Self { browser }
    }
    
    /// Process and store the insight
    async fn process_insight(&self, params: &ReportInsightParams) -> Result<InsightReport> {
        // In mock mode, return simulated report
        if std::env::var("RAINBOW_MOCK_MODE").unwrap_or_default() == "true" {
            return Ok(self.create_mock_report(params));
        }
        
        // Real implementation would process and store the insight
        // For now, return mock data
        Ok(self.create_mock_report(params))
    }
    
    /// Create mock insight report for testing
    fn create_mock_report(&self, params: &ReportInsightParams) -> InsightReport {
        let id = Uuid::new_v4().to_string();
        let timestamp = Utc::now();
        
        // Determine cognition level based on insight complexity
        let cognition_level = match params.category {
            InsightCategory::Strategic => CognitionLevel::Strategic,
            InsightCategory::Pattern => CognitionLevel::Pattern,
            InsightCategory::Performance | InsightCategory::SystemOptimization => {
                if params.confidence > 0.8 {
                    CognitionLevel::Adaptive
                } else {
                    CognitionLevel::Pattern
                }
            },
            _ => CognitionLevel::Pattern,
        };
        
        // Create triggered actions based on priority and category
        let mut actions_triggered = Vec::new();
        if params.priority >= 4 {
            actions_triggered.push(TriggeredAction {
                action_type: "notification".to_string(),
                description: "High-priority insight notification sent".to_string(),
                status: ActionStatus::Completed,
                result: Some(json!({
                    "notified": true,
                    "channels": ["system", "log"]
                })),
            });
        }
        
        if matches!(params.category, InsightCategory::Risk) {
            actions_triggered.push(TriggeredAction {
                action_type: "risk_assessment".to_string(),
                description: "Automated risk assessment initiated".to_string(),
                status: ActionStatus::InProgress,
                result: None,
            });
        }
        
        // Find related insights (mock)
        let related_insights = if params.confidence > 0.7 {
            vec![RelatedInsight {
                id: Uuid::new_v4().to_string(),
                category: params.category.clone(),
                similarity_score: 0.85,
                relationship_type: RelationshipType::Related,
            }]
        } else {
            vec![]
        };
        
        // Assess impact
        let impact_assessment = ImpactAssessment {
            immediate: match params.priority {
                5 => ImpactLevel::Critical,
                4 => ImpactLevel::High,
                3 => ImpactLevel::Medium,
                2 => ImpactLevel::Low,
                _ => ImpactLevel::None,
            },
            long_term: if matches!(params.category, InsightCategory::Strategic | InsightCategory::Learning) {
                ImpactLevel::High
            } else {
                ImpactLevel::Medium
            },
            scope: match params.category {
                InsightCategory::SystemOptimization => ImpactScope::System,
                InsightCategory::Strategic => ImpactScope::Global,
                InsightCategory::Pattern | InsightCategory::UserBehavior => ImpactScope::Module,
                _ => ImpactScope::Local,
            },
            estimated_value: if params.confidence > 0.8 {
                Some(params.priority as f64 * 1000.0)
            } else {
                None
            },
            risk_mitigation: if matches!(params.category, InsightCategory::Risk) {
                Some(params.confidence * 5000.0)
            } else {
                None
            },
        };
        
        // Create acknowledgment
        let acknowledgment = Acknowledgment {
            message: format!(
                "Insight '{}' has been recorded and processed.",
                params.insight.chars().take(50).collect::<String>()
            ),
            action_required: params.priority >= 4,
            follow_up: if !params.recommendations.is_empty() {
                Some(format!(
                    "Review {} recommendation(s) for implementation.",
                    params.recommendations.len()
                ))
            } else {
                None
            },
        };
        
        InsightReport {
            id,
            timestamp,
            insight: params.insight.clone(),
            category: params.category.clone(),
            confidence: params.confidence,
            priority: params.priority,
            cognition_level,
            stored: true,
            actions_triggered,
            related_insights,
            impact_assessment,
            acknowledgment,
        }
    }
}

#[async_trait]
impl Tool for ReportInsight {
    type Input = ReportInsightParams;
    type Output = InsightReport;
    
    fn name(&self) -> &str {
        "report_insight"
    }
    
    fn description(&self) -> &str {
        "Report patterns, learnings, or optimization opportunities discovered during execution"
    }
    
    async fn execute(&self, params: Self::Input) -> Result<Self::Output> {
        debug!(
            "Reporting insight: {} (category: {:?}, confidence: {})",
            params.insight, params.category, params.confidence
        );
        
        let start = std::time::Instant::now();
        
        let report = self.process_insight(&params).await?;
        
        let duration = start.elapsed();
        info!(
            "Insight {} reported successfully in {:?} (cognition level: {:?})",
            report.id,
            duration,
            report.cognition_level
        );
        
        Ok(report)
    }
    
    fn validate_input(&self, params: &Self::Input) -> Result<()> {
        if params.insight.trim().is_empty() {
            return Err(anyhow::anyhow!("Insight cannot be empty"));
        }
        
        if params.confidence < 0.0 || params.confidence > 1.0 {
            return Err(anyhow::anyhow!("Confidence must be between 0.0 and 1.0"));
        }
        
        if params.priority < 1 || params.priority > 5 {
            return Err(anyhow::anyhow!("Priority must be between 1 and 5"));
        }
        
        if params.insight.len() > 5000 {
            return Err(anyhow::anyhow!("Insight text cannot exceed 5000 characters"));
        }
        
        Ok(())
    }
    
    fn input_schema(&self) -> serde_json::Value {
        json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object",
            "properties": {
                "insight": {
                    "type": "string",
                    "description": "The insight being reported",
                    "maxLength": 5000
                },
                "category": {
                    "type": "string",
                    "enum": [
                        "performance", "pattern", "risk", "learning",
                        "strategic", "user_behavior", "system_optimization"
                    ]
                },
                "confidence": {
                    "type": "number",
                    "minimum": 0.0,
                    "maximum": 1.0
                },
                "evidence": {
                    "type": "object",
                    "properties": {
                        "evidence_type": {
                            "type": "string",
                            "enum": ["metrics", "pattern", "statistical", "behavioral", "logs", "comparative", "experimental"]
                        },
                        "data": {"type": "object"},
                        "source": {"type": "string"},
                        "metrics": {
                            "type": "object",
                            "additionalProperties": {"type": "number"}
                        },
                        "samples": {
                            "type": "array",
                            "items": {"type": "object"}
                        }
                    },
                    "required": ["evidence_type", "data"]
                },
                "recommendations": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "action": {"type": "string"},
                            "expected_impact": {"type": "string"},
                            "complexity": {
                                "type": "string",
                                "enum": ["low", "medium", "high", "critical"]
                            },
                            "effort_hours": {"type": "number"},
                            "dependencies": {
                                "type": "array",
                                "items": {"type": "string"}
                            }
                        },
                        "required": ["action", "expected_impact", "complexity"]
                    }
                },
                "priority": {
                    "type": "integer",
                    "minimum": 1,
                    "maximum": 5,
                    "default": 3
                },
                "tags": {
                    "type": "array",
                    "items": {"type": "string"}
                }
            },
            "required": ["insight", "category", "confidence"]
        })
    }
    
    fn output_schema(&self) -> serde_json::Value {
        json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object",
            "properties": {
                "id": {"type": "string"},
                "timestamp": {"type": "string", "format": "date-time"},
                "insight": {"type": "string"},
                "category": {"type": "string"},
                "confidence": {"type": "number"},
                "priority": {"type": "integer"},
                "cognition_level": {"type": "string"},
                "stored": {"type": "boolean"},
                "actions_triggered": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "action_type": {"type": "string"},
                            "description": {"type": "string"},
                            "status": {"type": "string"}
                        }
                    }
                },
                "impact_assessment": {
                    "type": "object",
                    "properties": {
                        "immediate": {"type": "string"},
                        "long_term": {"type": "string"},
                        "scope": {"type": "string"}
                    }
                },
                "acknowledgment": {
                    "type": "object",
                    "properties": {
                        "message": {"type": "string"},
                        "action_required": {"type": "boolean"},
                        "follow_up": {"type": ["string", "null"]}
                    }
                }
            },
            "required": ["id", "timestamp", "insight", "category", "confidence", "stored"]
        })
    }
}

// Implement DynamicTool for runtime dispatch
#[async_trait]
impl DynamicTool for ReportInsight {
    fn name(&self) -> &str {
        Tool::name(self)
    }
    
    async fn execute_json(&self, params: serde_json::Value) -> Result<serde_json::Value> {
        let input: ReportInsightParams = serde_json::from_value(params)
            .context("Failed to parse ReportInsight parameters")?;
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