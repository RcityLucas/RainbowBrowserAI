// AI and intelligence module
pub mod core; // Core intelligence components

// Re-export main types from core
pub use core::{
    ai_decision_engine::*,
    contextual_awareness::*,
    llm_integration::*,
    ml_confidence_scorer::*,
};

// Re-export LLM service
pub use core::llm_service::{LLMService, TaskType, TaskUnderstanding};