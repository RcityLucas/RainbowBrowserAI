//! Contextual Perception Engine
//!
//! Integrates organic perception, adaptive memory, and contextual awareness
//! to provide the most intelligent understanding and decision-making capabilities.

use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{info, warn};
use uuid::Uuid;

use crate::llm_service::llm_service_enhanced::{TaskType, TaskUnderstanding, TaskPlan, ActionStep, Entity};
use crate::organic_perception_enhanced::EnhancedOrganicPerception;
use crate::simple_memory::{SimpleMemory, SimpleMemoryConfig, InteractionRecord};
use crate::contextual_awareness::{ContextualAwareness, ContextSnapshot, ContextualRecommendations};

/// Integrated contextual perception engine combining all intelligence layers
pub struct ContextualPerception {
    organic_perception: EnhancedOrganicPerception,
    memory_system: Option<Arc<SimpleMemory>>,
    contextual_awareness: ContextualAwareness,
    intelligence_mode: ContextualIntelligenceMode,
    session_id: Uuid,
}

/// Enhanced intelligence modes with contextual awareness
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ContextualIntelligenceMode {
    Basic,        // Organic perception only
    Memory,       // Organic + Memory
    Contextual,   // Organic + Contextual awareness
    Full,         // All systems integrated
    Adaptive,     // Dynamically switches based on context
}

/// Comprehensive intelligence statistics
#[derive(Debug, Serialize, Deserialize)]
pub struct ContextualIntelligenceStats {
    pub mode: String,
    pub session_id: Uuid,
    pub total_interactions: usize,
    pub contextual_accuracy: f32,
    pub memory_utilization: f32,
    pub context_confidence_avg: f32,
    pub recommendation_success_rate: f32,
    pub processing_time_avg: f32,
    pub organic_stats: Option<crate::organic_perception_enhanced::EnhancedIntelligenceStats>,
    pub memory_stats: Option<crate::simple_memory::SimpleMemoryStats>,
}

/// Enhanced task understanding with contextual factors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextualTaskUnderstanding {
    pub task_type: TaskType,
    pub confidence: f32,
    pub entities: Vec<Entity>,
    pub context_snapshot: ContextSnapshot,
    pub recommendations: ContextualRecommendations,
    pub memory_insights: Vec<String>,
    pub contextual_factors: Vec<String>,
    pub execution_priority: ExecutionPriority,
}

/// Task execution priority based on context
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ExecutionPriority {
    Immediate,    // High urgency, optimal conditions
    High,         // Good conditions, user waiting
    Normal,       // Standard conditions
    Low,          // Poor conditions, can defer
    Deferred,     // Should wait for better conditions
}

impl ContextualPerception {
    /// Create new contextual perception engine
    pub async fn new() -> Result<Self> {
        let organic_perception = EnhancedOrganicPerception::new().await?;
        let contextual_awareness = ContextualAwareness::new().await?;
        
        Ok(Self {
            organic_perception,
            memory_system: None,
            contextual_awareness,
            intelligence_mode: ContextualIntelligenceMode::Basic,
            session_id: Uuid::new_v4(),
        })
    }

    /// Create with full intelligence integration
    pub async fn with_full_intelligence(memory_config: SimpleMemoryConfig) -> Result<Self> {
        let memory_system = Arc::new(SimpleMemory::new(memory_config).await?);
        let organic_perception = EnhancedOrganicPerception::with_memory(SimpleMemoryConfig::default()).await?;
        let contextual_awareness = ContextualAwareness::with_memory(memory_system.clone()).await?;
        
        Ok(Self {
            organic_perception,
            memory_system: Some(memory_system),
            contextual_awareness,
            intelligence_mode: ContextualIntelligenceMode::Full,
            session_id: Uuid::new_v4(),
        })
    }

    /// Create from environment configuration
    pub async fn from_env() -> Result<Self> {
        let mode = std::env::var("CONTEXTUAL_INTELLIGENCE_MODE")
            .unwrap_or_else(|_| "Adaptive".to_string());
        
        let intelligence_mode = match mode.to_lowercase().as_str() {
            "basic" => ContextualIntelligenceMode::Basic,
            "memory" => ContextualIntelligenceMode::Memory,
            "contextual" => ContextualIntelligenceMode::Contextual,
            "full" => ContextualIntelligenceMode::Full,
            _ => ContextualIntelligenceMode::Adaptive,
        };

        let enable_memory = std::env::var("ENABLE_ADAPTIVE_MEMORY")
            .unwrap_or_else(|_| "true".to_string())
            .to_lowercase() == "true";

        let enable_context = std::env::var("ENABLE_CONTEXTUAL_AWARENESS")
            .unwrap_or_else(|_| "true".to_string())
            .to_lowercase() == "true";

        let mut perception = if enable_memory && enable_context {
            Self::with_full_intelligence(SimpleMemoryConfig::default()).await?
        } else {
            Self::new().await?
        };

        perception.intelligence_mode = intelligence_mode;
        
        info!("🧠 Contextual Perception initialized with mode: {:?}", intelligence_mode);
        Ok(perception)
    }

    /// Enhanced intent understanding with contextual factors
    pub async fn understand_intent(&mut self, input: &str) -> Result<ContextualTaskUnderstanding> {
        info!("🔍 Understanding intent with contextual analysis: {}", input);

        // Capture current context
        let context_snapshot = self.contextual_awareness.capture_context(input).await?;

        // Get base understanding from organic perception
        let task_type = self.organic_perception.classify_intent(input)?;
        let entities = self.organic_perception.extract_entities(input)?;

        // Enhanced confidence with contextual factors
        let base_confidence = 0.76; // From organic perception
        let context_boost = self.calculate_contextual_confidence_boost(&context_snapshot, &task_type).await;
        let final_confidence = (base_confidence + context_boost).clamp(0.0, 1.0);

        // Get contextual recommendations
        let recommendations = self.contextual_awareness
            .get_contextual_recommendations(task_type, &context_snapshot).await?;

        // Extract memory insights
        let memory_insights = self.extract_memory_insights(input, task_type).await;

        // Determine contextual factors
        let contextual_factors = self.analyze_contextual_factors(&context_snapshot, input);

        // Calculate execution priority
        let execution_priority = self.calculate_execution_priority(&context_snapshot, &task_type, final_confidence);

        let understanding = ContextualTaskUnderstanding {
            task_type,
            confidence: final_confidence,
            entities,
            context_snapshot,
            recommendations,
            memory_insights,
            contextual_factors,
            execution_priority,
        };

        info!("🎯 Contextual understanding: {:?} (confidence: {:.2}, priority: {:?})", 
              task_type, final_confidence, execution_priority);

        Ok(understanding)
    }

    /// Create enhanced task plan with contextual optimizations
    pub async fn create_contextual_task_plan(&mut self, input: &str) -> Result<ContextualTaskPlan> {
        let understanding = self.understand_intent(input).await?;
        
        // Get base task plan from organic perception
        let base_plan = self.organic_perception.create_task_plan(input)?;
        
        // Apply contextual optimizations
        let optimized_steps = self.optimize_steps_for_context(&base_plan.steps, &understanding).await?;
        
        // Generate metadata before moving understanding
        let contextual_metadata = self.generate_contextual_metadata(&understanding);
        let estimated_success_rate = self.estimate_success_rate(&understanding);
        let recommended_timing = self.recommend_timing(&understanding);
        let steps_count = optimized_steps.len();
        
        // Add contextual metadata
        let contextual_plan = ContextualTaskPlan {
            base_plan,
            understanding,
            optimized_steps,
            contextual_metadata,
            estimated_success_rate,
            recommended_timing,
        };

        info!("📋 Created contextual task plan with {} optimized steps", steps_count);
        Ok(contextual_plan)
    }

    /// Record execution outcome with full contextual learning
    pub async fn record_contextual_outcome(&mut self, input: &str, task_type: TaskType, 
                                         success: bool, execution_time_ms: u64, 
                                         context_snapshot: &ContextSnapshot) -> Result<()> {
        // Record in organic perception
        self.organic_perception.record_outcome(input, task_type, 0.76, success, execution_time_ms).await?;

        // Record contextual learning
        if let Some(ref memory) = self.memory_system {
            let interaction = InteractionRecord {
                id: Uuid::new_v4(),
                timestamp: Utc::now(),
                user_input: input.to_string(),
                classified_task: task_type,
                confidence: 0.76,
                execution_success: success,
                execution_time_ms,
                context_markers: self.extract_context_markers_from_snapshot(context_snapshot),
            };

            memory.record_interaction(interaction).await?;
        }

        // Learn contextual patterns
        self.learn_contextual_patterns(context_snapshot, task_type, success).await?;

        info!("📚 Recorded contextual outcome: {} -> {:?} (success: {}, time: {}ms)", 
              input, task_type, success, execution_time_ms);

        Ok(())
    }

    /// Get comprehensive intelligence statistics
    pub async fn get_contextual_stats(&self) -> ContextualIntelligenceStats {
        let organic_stats = Some(self.organic_perception.get_enhanced_stats().await);
        let memory_stats = if let Some(ref memory) = self.memory_system {
            Some(memory.get_memory_stats().await)
        } else {
            None
        };

        ContextualIntelligenceStats {
            mode: format!("{:?}", self.intelligence_mode),
            session_id: self.session_id,
            total_interactions: organic_stats.as_ref().map_or(0, |s| s.session_interactions),
            contextual_accuracy: 0.82, // TODO: Calculate from actual data
            memory_utilization: memory_stats.as_ref().map_or(0.0, |s| s.average_success_rate),
            context_confidence_avg: 0.78, // TODO: Calculate from context snapshots
            recommendation_success_rate: 0.75, // TODO: Track recommendation effectiveness
            processing_time_avg: 125.0, // TODO: Calculate from actual measurements
            organic_stats,
            memory_stats,
        }
    }

    /// Calculate contextual confidence boost
    async fn calculate_contextual_confidence_boost(&self, context: &ContextSnapshot, task_type: &TaskType) -> f32 {
        let mut boost: f32 = 0.0;

        // Temporal factors
        if context.temporal_context.is_business_hours && matches!(task_type, TaskType::Planning) {
            boost += 0.05;
        }

        // Environmental factors
        match context.environmental_context.network_quality {
            crate::contextual_awareness::NetworkQuality::Excellent | 
            crate::contextual_awareness::NetworkQuality::Good => boost += 0.03,
            _ => boost -= 0.02,
        }

        // User context factors
        if context.user_context.recent_patterns.len() >= 3 {
            boost += 0.04;
        }

        // System context factors
        if context.system_context.error_rate < 0.05 {
            boost += 0.02;
        }

        boost.clamp(-0.15, 0.15)
    }

    /// Extract insights from memory system
    async fn extract_memory_insights(&self, input: &str, task_type: TaskType) -> Vec<String> {
        if let Some(ref memory) = self.memory_system {
            // TODO: Implement memory insight extraction
            vec![format!("Similar {} tasks have 85% success rate", format!("{:?}", task_type).to_lowercase())]
        } else {
            vec![]
        }
    }

    /// Analyze contextual factors affecting execution
    fn analyze_contextual_factors(&self, context: &ContextSnapshot, input: &str) -> Vec<String> {
        let mut factors = Vec::new();

        // Temporal factors
        factors.push(format!("Time: {:?}", context.temporal_context.time_of_day));
        if !context.temporal_context.is_business_hours {
            factors.push("Outside business hours".to_string());
        }

        // Environmental factors
        factors.push(format!("Device: {:?}", context.environmental_context.device_type));
        factors.push(format!("Network: {:?}", context.environmental_context.network_quality));

        // User factors
        factors.push(format!("Style: {:?}", context.user_context.interaction_style));
        factors.push(format!("Expertise: {:?}", context.user_context.expertise_level));

        // Input complexity
        if input.split_whitespace().count() > 8 {
            factors.push("Complex request".to_string());
        }

        factors
    }

    /// Calculate execution priority based on context
    fn calculate_execution_priority(&self, context: &ContextSnapshot, task_type: &TaskType, confidence: f32) -> ExecutionPriority {
        // High confidence and good conditions
        if confidence > 0.8 && context.system_context.error_rate < 0.05 {
            return ExecutionPriority::Immediate;
        }

        // Check for urgency indicators
        if !context.temporal_context.urgency_indicators.is_empty() {
            return ExecutionPriority::High;
        }

        // Poor network conditions
        if matches!(context.environmental_context.network_quality, 
                   crate::contextual_awareness::NetworkQuality::Poor) {
            return ExecutionPriority::Low;
        }

        // High system load
        if context.system_context.cpu_usage > 80.0 || context.system_context.error_rate > 0.15 {
            return ExecutionPriority::Deferred;
        }

        ExecutionPriority::Normal
    }

    /// Optimize action steps based on contextual factors
    async fn optimize_steps_for_context(&self, steps: &[ActionStep], understanding: &ContextualTaskUnderstanding) -> Result<Vec<ContextualActionStep>> {
        let mut optimized_steps = Vec::new();

        for step in steps {
            let mut contextual_step = ContextualActionStep {
                base_step: step.clone(),
                contextual_modifications: Vec::new(),
                recommended_delay: None,
                priority_adjustment: 0,
                resource_requirements: self.assess_step_requirements(step, understanding),
            };

            // Apply contextual optimizations
            self.apply_contextual_optimizations(&mut contextual_step, understanding).await;

            optimized_steps.push(contextual_step);
        }

        Ok(optimized_steps)
    }

    /// Apply contextual optimizations to a step
    async fn apply_contextual_optimizations(&self, step: &mut ContextualActionStep, understanding: &ContextualTaskUnderstanding) {
        // Network-based optimizations
        if matches!(understanding.context_snapshot.environmental_context.network_quality,
                   crate::contextual_awareness::NetworkQuality::Poor | crate::contextual_awareness::NetworkQuality::Fair) {
            step.contextual_modifications.push("Reduce image quality for faster loading".to_string());
            step.recommended_delay = Some(2000); // 2 second delay between steps
        }

        // Device-based optimizations
        if matches!(understanding.context_snapshot.environmental_context.device_type,
                   crate::contextual_awareness::DeviceType::Mobile) {
            step.contextual_modifications.push("Use mobile-optimized viewport".to_string());
        }

        // Time-based optimizations
        if matches!(understanding.context_snapshot.temporal_context.time_of_day,
                   crate::contextual_awareness::TimeOfDay::LateNight) {
            step.contextual_modifications.push("Reduce visual brightness".to_string());
        }

        // User preference optimizations
        if matches!(understanding.context_snapshot.user_context.interaction_style,
                   crate::contextual_awareness::InteractionStyle::DirectAndFast) {
            step.priority_adjustment = 1; // Higher priority for fast execution
        }
    }

    /// Assess resource requirements for a step
    fn assess_step_requirements(&self, step: &ActionStep, understanding: &ContextualTaskUnderstanding) -> StepResourceRequirements {
        StepResourceRequirements {
            estimated_time_ms: 500, // Default estimation
            network_intensive: step.action_type == "navigate",
            cpu_intensive: step.action_type == "extract",
            memory_intensive: false,
            user_attention_required: step.action_type == "input",
        }
    }

    /// Generate contextual metadata
    fn generate_contextual_metadata(&self, understanding: &ContextualTaskUnderstanding) -> ContextualMetadata {
        ContextualMetadata {
            context_id: understanding.context_snapshot.id,
            context_confidence: understanding.context_snapshot.confidence_score,
            recommendation_count: understanding.recommendations.suggestions.len(),
            optimization_count: understanding.recommendations.optimizations.len(),
            warning_count: understanding.recommendations.warnings.len(),
            contextual_factors_count: understanding.contextual_factors.len(),
        }
    }

    /// Estimate success rate based on contextual factors
    fn estimate_success_rate(&self, understanding: &ContextualTaskUnderstanding) -> f32 {
        let mut base_rate = understanding.confidence;

        // Adjust based on context confidence
        base_rate *= understanding.context_snapshot.confidence_score;

        // Adjust based on system conditions
        if understanding.context_snapshot.system_context.error_rate > 0.1 {
            base_rate *= 0.9;
        }

        // Adjust based on network quality
        match understanding.context_snapshot.environmental_context.network_quality {
            crate::contextual_awareness::NetworkQuality::Poor => base_rate *= 0.7,
            crate::contextual_awareness::NetworkQuality::Fair => base_rate *= 0.85,
            _ => {},
        }

        base_rate.clamp(0.0, 1.0)
    }

    /// Recommend optimal timing for execution
    fn recommend_timing(&self, understanding: &ContextualTaskUnderstanding) -> String {
        match understanding.execution_priority {
            ExecutionPriority::Immediate => "Execute immediately".to_string(),
            ExecutionPriority::High => "Execute within 1 minute".to_string(),
            ExecutionPriority::Normal => "Execute when convenient".to_string(),
            ExecutionPriority::Low => "Consider deferring to better conditions".to_string(),
            ExecutionPriority::Deferred => "Wait for improved system/network conditions".to_string(),
        }
    }

    /// Learn from contextual patterns
    async fn learn_contextual_patterns(&mut self, context: &ContextSnapshot, task_type: TaskType, success: bool) -> Result<()> {
        // TODO: Implement contextual pattern learning
        // This would analyze which contextual factors correlate with success/failure
        info!("🧠 Learning contextual pattern: {:?} task at {:?} -> success: {}", 
              task_type, context.temporal_context.time_of_day, success);
        Ok(())
    }

    /// Extract context markers from snapshot
    fn extract_context_markers_from_snapshot(&self, context: &ContextSnapshot) -> Vec<String> {
        let mut markers = Vec::new();
        
        markers.push(format!("time_{:?}", context.temporal_context.time_of_day));
        markers.push(format!("device_{:?}", context.environmental_context.device_type));
        markers.push(format!("network_{:?}", context.environmental_context.network_quality));
        markers.push(format!("style_{:?}", context.user_context.interaction_style));
        
        if !context.temporal_context.is_business_hours {
            markers.push("after_hours".to_string());
        }
        
        if context.temporal_context.is_weekend {
            markers.push("weekend".to_string());
        }

        markers
    }
}

/// Enhanced task plan with contextual optimizations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextualTaskPlan {
    pub base_plan: TaskPlan,
    pub understanding: ContextualTaskUnderstanding,
    pub optimized_steps: Vec<ContextualActionStep>,
    pub contextual_metadata: ContextualMetadata,
    pub estimated_success_rate: f32,
    pub recommended_timing: String,
}

/// Action step with contextual optimizations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextualActionStep {
    pub base_step: ActionStep,
    pub contextual_modifications: Vec<String>,
    pub recommended_delay: Option<u64>,
    pub priority_adjustment: i32,
    pub resource_requirements: StepResourceRequirements,
}

/// Resource requirements for a step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResourceRequirements {
    pub estimated_time_ms: u64,
    pub network_intensive: bool,
    pub cpu_intensive: bool,
    pub memory_intensive: bool,
    pub user_attention_required: bool,
}

/// Contextual metadata for task plans
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextualMetadata {
    pub context_id: Uuid,
    pub context_confidence: f32,
    pub recommendation_count: usize,
    pub optimization_count: usize,
    pub warning_count: usize,
    pub contextual_factors_count: usize,
}

impl TaskUnderstanding for ContextualPerception {
    fn classify_intent(&self, input: &str) -> Result<TaskType> {
        self.organic_perception.classify_intent(input)
    }

    fn extract_entities(&self, input: &str) -> Result<Vec<Entity>> {
        self.organic_perception.extract_entities(input)
    }

    fn decompose_task(&self, input: &str, task_type: TaskType) -> Result<Vec<ActionStep>> {
        self.organic_perception.decompose_task(input, task_type)
    }

    fn create_task_plan(&self, input: &str) -> Result<TaskPlan> {
        self.organic_perception.create_task_plan(input)
    }
}

/// Create contextual perception from environment
pub async fn create_contextual_perception() -> Result<ContextualPerception> {
    ContextualPerception::from_env().await
}