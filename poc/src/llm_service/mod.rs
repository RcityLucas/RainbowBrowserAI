// LLM Service Module - Intelligent Language Understanding
//
// This module provides both legacy and organic LLM services for gradual migration
// from hardcoded rules to adaptive, learning-based intelligence.

pub mod legacy_service;
pub mod llm_service_enhanced;
pub use legacy_service::{LLMService, ParsedCommand, CommandParams};
pub use llm_service_enhanced::{
    TaskType, TaskUnderstanding, MockTaskUnderstanding, TaskPlan, ActionStep, Entity, IntelligentCommand
};