// Intelligence Module - Organic AI Perception and Learning System
//
// This module implements the organic intelligence described in the design documents,
// replacing hardcoded rule-based systems with learning, adaptive intelligence.

pub mod perception;
pub mod patterns;
pub mod confidence;
pub mod learning;

pub use perception::{OrganicPerception, IntentUnderstanding, Context, PerceptionMode, IntelligenceStats};
pub use patterns::{Pattern, PatternMatcher, PatternType};
pub use confidence::{ConfidenceCalibrator, ConfidenceScore};
pub use learning::{LearningEngine, InteractionOutcome};