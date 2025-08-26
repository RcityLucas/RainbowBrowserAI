//! Meta-cognitive Tools Module - V8.0 Standard Compliance
//! 
//! Provides self-awareness and intelligence amplification capabilities for AI agents.
//! Part of the V8.0 standard's Meta-cognitive category (元认知类).

pub mod report_insight;
pub mod complete_task;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::tools::{Tool, Result};

/// Meta-cognitive capability levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CognitionLevel {
    /// Basic pattern recognition
    Pattern,
    /// Strategic thinking
    Strategic,
    /// Creative problem solving
    Creative,
    /// Self-improvement
    Adaptive,
}

/// Insight category
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InsightCategory {
    /// Performance optimization opportunity
    Performance,
    /// Pattern or trend identified
    Pattern,
    /// Potential issue or risk
    Risk,
    /// Learning or improvement
    Learning,
    /// Strategic recommendation
    Strategic,
    /// User behavior insight
    UserBehavior,
    /// System optimization
    SystemOptimization,
}

/// Task completion strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompletionStrategy {
    /// Sequential execution
    Sequential,
    /// Parallel execution
    Parallel,
    /// Adaptive based on context
    Adaptive,
    /// Iterative refinement
    Iterative,
}

/// Re-export for convenience
pub use self::{
    report_insight::{ReportInsight, ReportInsightParams, InsightReport},
    complete_task::{CompleteTask, CompleteTaskParams, TaskCompletionResult},
};