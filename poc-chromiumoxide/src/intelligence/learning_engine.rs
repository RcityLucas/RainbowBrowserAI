// Learning Engine
// Learns from user interactions and automation results to improve future performance

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use tracing::{info, debug};

/// Learning engine that improves automation through experience
#[derive(Debug)]
pub struct LearningEngine {
    learning_data: VecDeque<LearningData>,
    action_patterns: HashMap<String, ActionPattern>,
    performance_metrics: PerformanceMetrics,
    learning_config: LearningConfig,
}

/// Configuration for learning behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningConfig {
    pub max_learning_samples: usize,
    pub learning_rate: f64,
    pub success_threshold: f64,
    pub pattern_confidence_threshold: f64,
    pub enable_automatic_adaptation: bool,
    pub retention_period_days: u32,
}

impl Default for LearningConfig {
    fn default() -> Self {
        Self {
            max_learning_samples: 10000,
            learning_rate: 0.1,
            success_threshold: 0.8,
            pattern_confidence_threshold: 0.7,
            enable_automatic_adaptation: true,
            retention_period_days: 30,
        }
    }
}

/// Individual learning data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningData {
    pub action_type: String,
    pub parameters: HashMap<String, serde_json::Value>,
    pub expected_outcome: String,
    pub actual_outcome: String,
    pub success: bool,
    pub execution_time_ms: u64,
    pub confidence: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Pattern learned from repeated actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionPattern {
    pub pattern_id: String,
    pub action_type: String,
    pub common_parameters: HashMap<String, serde_json::Value>,
    pub success_rate: f64,
    pub total_attempts: u32,
    pub successful_attempts: u32,
    pub average_execution_time: f64,
    pub confidence_score: f64,
    pub last_updated: chrono::DateTime<chrono::Utc>,
    pub domain_contexts: Vec<String>, // Domains where this pattern works
    pub improvement_suggestions: Vec<String>,
}

/// Performance metrics for the learning system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub total_samples: usize,
    pub success_rate: f64,
    pub average_confidence: f64,
    pub improvement_over_time: f64, // Percentage improvement
    pub patterns_learned: usize,
    pub domains_encountered: usize,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            total_samples: 0,
            success_rate: 0.0,
            average_confidence: 0.0,
            improvement_over_time: 0.0,
            patterns_learned: 0,
            domains_encountered: 0,
            last_updated: chrono::Utc::now(),
        }
    }
}

/// Learning statistics for analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningStatistics {
    pub total_samples: usize,
    pub success_rate: f64,
    pub average_confidence: f64,
    pub top_patterns: Vec<ActionPattern>,
    pub improvement_trends: Vec<ImprovementTrend>,
    pub domain_performance: HashMap<String, DomainPerformance>,
}

/// Improvement trend over time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImprovementTrend {
    pub period: String,
    pub success_rate: f64,
    pub average_execution_time: f64,
    pub confidence_improvement: f64,
}

/// Performance metrics for specific domain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainPerformance {
    pub domain: String,
    pub success_rate: f64,
    pub total_attempts: u32,
    pub average_response_time: f64,
    pub top_failing_actions: Vec<String>,
    pub recommended_improvements: Vec<String>,
}

/// Learning insight that can improve future actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningInsight {
    pub insight_type: String,
    pub description: String,
    pub confidence: f64,
    pub applicable_contexts: Vec<String>,
    pub suggested_action: String,
}

impl LearningEngine {
    /// Create new learning engine
    pub fn new() -> Self {
        Self::with_config(LearningConfig::default())
    }
    
    /// Create learning engine with custom configuration
    pub fn with_config(config: LearningConfig) -> Self {
        info!("Initializing Learning Engine with {} max samples", config.max_learning_samples);
        
        Self {
            learning_data: VecDeque::with_capacity(config.max_learning_samples),
            action_patterns: HashMap::new(),
            performance_metrics: PerformanceMetrics::default(),
            learning_config: config,
        }
    }
    
    /// Record new learning data
    pub async fn record_learning_data(&mut self, data: LearningData) -> Result<()> {
        debug!("Recording learning data: {} success={}", data.action_type, data.success);
        
        // Add to learning data queue
        if self.learning_data.len() >= self.learning_config.max_learning_samples {
            self.learning_data.pop_front(); // Remove oldest data
        }
        self.learning_data.push_back(data.clone());
        
        // Update action patterns
        self.update_action_pattern(&data).await?;
        
        // Update performance metrics
        self.update_performance_metrics().await;
        
        // Generate insights if automatic adaptation is enabled
        if self.learning_config.enable_automatic_adaptation {
            let _insights = self.generate_insights(&data).await;
        }
        
        Ok(())
    }
    
    /// Update action pattern based on new data
    async fn update_action_pattern(&mut self, data: &LearningData) -> Result<()> {
        let pattern_key = self.create_pattern_key(&data.action_type, &data.parameters);
        
        let pattern = self.action_patterns
            .entry(pattern_key.clone())
            .or_insert_with(|| ActionPattern {
                pattern_id: pattern_key,
                action_type: data.action_type.clone(),
                common_parameters: data.parameters.clone(),
                success_rate: 0.0,
                total_attempts: 0,
                successful_attempts: 0,
                average_execution_time: 0.0,
                confidence_score: 0.0,
                last_updated: chrono::Utc::now(),
                domain_contexts: Vec::new(),
                improvement_suggestions: Vec::new(),
            });
        
        // Update pattern statistics
        pattern.total_attempts += 1;
        if data.success {
            pattern.successful_attempts += 1;
        }
        
        pattern.success_rate = pattern.successful_attempts as f64 / pattern.total_attempts as f64;
        
        // Update average execution time using exponential moving average
        let alpha = self.learning_config.learning_rate;
        pattern.average_execution_time = (1.0 - alpha) * pattern.average_execution_time + 
                                        alpha * data.execution_time_ms as f64;
        
        // Update confidence score
        pattern.confidence_score = pattern.success_rate;
        pattern.last_updated = chrono::Utc::now();
        
        // Generate improvement suggestions based on pattern stats
        pattern.improvement_suggestions = if pattern.success_rate < 0.7 {
            vec!["Consider alternative selectors".to_string()]
        } else {
            vec![]
        };
        
        debug!("Updated pattern {}: success_rate={:.2}, confidence={:.2}", 
            pattern.action_type, pattern.success_rate, pattern.confidence_score);
        
        Ok(())
    }
    
    /// Create unique key for action pattern
    fn create_pattern_key(&self, action_type: &str, parameters: &HashMap<String, serde_json::Value>) -> String {
        let mut key = action_type.to_string();
        
        // Add significant parameters to the key
        let mut param_keys: Vec<_> = parameters.keys().collect();
        param_keys.sort();
        
        for param_key in param_keys {
            if let Some(value) = parameters.get(param_key) {
                // Only include parameters that affect action behavior
                if self.is_significant_parameter(param_key, value) {
                    key.push_str(&format!("_{}:{}", param_key, value));
                }
            }
        }
        
        key
    }
    
    /// Check if parameter significantly affects action behavior
    fn is_significant_parameter(&self, key: &str, _value: &serde_json::Value) -> bool {
        // Parameters that typically affect action success
        matches!(key, "selector" | "text" | "url" | "element_type" | "wait_time" | "timeout")
    }
    
    /// Calculate confidence score for a pattern
    fn calculate_pattern_confidence(&self, pattern: &ActionPattern) -> f64 {
        let base_confidence = pattern.success_rate;
        
        // Adjust based on sample size (more data = higher confidence)
        let sample_confidence = if pattern.total_attempts >= 10 {
            1.0
        } else {
            pattern.total_attempts as f64 / 10.0
        };
        
        // Adjust based on consistency
        let consistency_bonus = if pattern.success_rate > 0.9 || pattern.success_rate < 0.1 {
            0.1 // Very consistent patterns get bonus
        } else {
            0.0
        };
        
        (base_confidence * sample_confidence + consistency_bonus).min(1.0)
    }
    
    /// Generate improvement suggestions for a pattern
    fn generate_pattern_improvements(&self, pattern: &ActionPattern) -> Vec<String> {
        let mut suggestions = Vec::new();
        
        if pattern.success_rate < self.learning_config.success_threshold {
            suggestions.push("Success rate below threshold - consider alternative selectors".to_string());
        }
        
        if pattern.average_execution_time > 5000.0 {
            suggestions.push("Execution time high - consider optimizing wait conditions".to_string());
        }
        
        if pattern.total_attempts > 20 && pattern.success_rate < 0.5 {
            suggestions.push("Frequent failures detected - review element identification strategy".to_string());
        }
        
        suggestions
    }
    
    /// Update overall performance metrics
    async fn update_performance_metrics(&mut self) {
        let total_samples = self.learning_data.len();
        if total_samples == 0 {
            return;
        }
        
        let successful_samples = self.learning_data.iter().filter(|d| d.success).count();
        let success_rate = successful_samples as f64 / total_samples as f64;
        
        let average_confidence = self.learning_data.iter()
            .map(|d| d.confidence)
            .sum::<f64>() / total_samples as f64;
        
        // Calculate improvement over time (compare recent vs older data)
        let improvement_over_time = self.calculate_improvement_trend();
        
        self.performance_metrics = PerformanceMetrics {
            total_samples,
            success_rate,
            average_confidence,
            improvement_over_time,
            patterns_learned: self.action_patterns.len(),
            domains_encountered: self.count_unique_domains(),
            last_updated: chrono::Utc::now(),
        };
        
        debug!("Updated performance metrics: success_rate={:.2}, patterns={}", 
            success_rate, self.action_patterns.len());
    }
    
    /// Calculate improvement trend over time
    fn calculate_improvement_trend(&self) -> f64 {
        if self.learning_data.len() < 20 {
            return 0.0; // Need enough data for trend analysis
        }
        
        let half_point = self.learning_data.len() / 2;
        
        let older_success = self.learning_data.iter()
            .take(half_point)
            .filter(|d| d.success)
            .count() as f64 / half_point as f64;
            
        let recent_success = self.learning_data.iter()
            .skip(half_point)
            .filter(|d| d.success)
            .count() as f64 / (self.learning_data.len() - half_point) as f64;
        
        ((recent_success - older_success) / older_success.max(0.01)) * 100.0
    }
    
    /// Count unique domains in learning data
    fn count_unique_domains(&self) -> usize {
        // This would require domain information in learning data
        // For now, estimate based on action patterns
        self.action_patterns.values()
            .flat_map(|p| &p.domain_contexts)
            .collect::<std::collections::HashSet<_>>()
            .len()
    }
    
    /// Generate insights from learning data
    async fn generate_insights(&self, data: &LearningData) -> Vec<LearningInsight> {
        let mut insights = Vec::new();
        
        // Insight: Repeated failures for same action type
        if !data.success {
            let recent_failures = self.learning_data.iter()
                .rev()
                .take(10)
                .filter(|d| d.action_type == data.action_type && !d.success)
                .count();
                
            if recent_failures >= 3 {
                insights.push(LearningInsight {
                    insight_type: "repeated_failure".to_string(),
                    description: format!("Action '{}' failing repeatedly - may need strategy change", data.action_type),
                    confidence: 0.8,
                    applicable_contexts: vec![data.action_type.clone()],
                    suggested_action: "Review element selectors and wait conditions".to_string(),
                });
            }
        }
        
        // Insight: Performance degradation
        if data.execution_time_ms > 10000 { // Slow execution
            insights.push(LearningInsight {
                insight_type: "performance_issue".to_string(),
                description: "Action execution time unusually high".to_string(),
                confidence: 0.7,
                applicable_contexts: vec![data.action_type.clone()],
                suggested_action: "Optimize wait conditions and timeouts".to_string(),
            });
        }
        
        insights
    }
    
    /// Get best action pattern for given action type
    pub async fn get_best_pattern(&self, action_type: &str) -> Option<&ActionPattern> {
        self.action_patterns.values()
            .filter(|p| p.action_type == action_type)
            .max_by(|a, b| a.confidence_score.partial_cmp(&b.confidence_score).unwrap())
    }
    
    /// Get recommendations for improving action success
    pub async fn get_improvement_recommendations(&self, action_type: &str) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        // Find patterns for this action type
        let patterns: Vec<_> = self.action_patterns.values()
            .filter(|p| p.action_type == action_type)
            .collect();
        
        if patterns.is_empty() {
            recommendations.push("No learning data available for this action type".to_string());
            return recommendations;
        }
        
        // Analyze patterns and generate recommendations
        let avg_success_rate = patterns.iter().map(|p| p.success_rate).sum::<f64>() / patterns.len() as f64;
        
        if avg_success_rate < 0.7 {
            recommendations.push("Consider using more reliable selectors".to_string());
            recommendations.push("Increase wait times before actions".to_string());
        }
        
        let high_performers: Vec<_> = patterns.iter()
            .filter(|p| p.success_rate > 0.8)
            .collect();
            
        if !high_performers.is_empty() {
            recommendations.push(format!("Use successful patterns: {} high-performing variants found", 
                high_performers.len()));
        }
        
        recommendations
    }
    
    /// Get learning statistics
    pub async fn get_statistics(&self) -> LearningStatistics {
        // Get top performing patterns
        let mut top_patterns: Vec<_> = self.action_patterns.values().cloned().collect();
        top_patterns.sort_by(|a, b| b.confidence_score.partial_cmp(&a.confidence_score).unwrap());
        top_patterns.truncate(10); // Top 10
        
        // Generate improvement trends (simplified)
        let improvement_trends = self.generate_improvement_trends();
        
        // Generate domain performance (simplified)
        let domain_performance = HashMap::new(); // Would be implemented based on domain data
        
        LearningStatistics {
            total_samples: self.learning_data.len(),
            success_rate: self.performance_metrics.success_rate,
            average_confidence: self.performance_metrics.average_confidence,
            top_patterns,
            improvement_trends,
            domain_performance,
        }
    }
    
    /// Generate improvement trends over time periods
    fn generate_improvement_trends(&self) -> Vec<ImprovementTrend> {
        // Simplified trend generation
        // In a real implementation, this would analyze data by time periods
        vec![
            ImprovementTrend {
                period: "last_7_days".to_string(),
                success_rate: self.performance_metrics.success_rate,
                average_execution_time: 3000.0, // Would be calculated from actual data
                confidence_improvement: self.performance_metrics.improvement_over_time,
            }
        ]
    }
    
    /// Export learning data for analysis
    pub fn export_learning_data(&self) -> Result<String> {
        let json_data = serde_json::to_string_pretty(&self.learning_data)?;
        Ok(json_data)
    }
    
    /// Import learning data from previous sessions
    pub fn import_learning_data(&mut self, json_data: &str) -> Result<usize> {
        let imported_data: VecDeque<LearningData> = serde_json::from_str(json_data)?;
        let imported_count = imported_data.len();
        
        // Merge with existing data
        for data in imported_data {
            if self.learning_data.len() >= self.learning_config.max_learning_samples {
                self.learning_data.pop_front();
            }
            self.learning_data.push_back(data);
        }
        
        info!("Imported {} learning data points", imported_count);
        Ok(imported_count)
    }
    
    /// Clean up old learning data
    pub async fn cleanup_old_data(&mut self) -> Result<usize> {
        let cutoff_date = chrono::Utc::now() - 
            chrono::Duration::days(self.learning_config.retention_period_days as i64);
        
        let initial_count = self.learning_data.len();
        
        // Remove old data
        self.learning_data.retain(|data| data.timestamp > cutoff_date);
        
        // Clean up old patterns
        self.action_patterns.retain(|_, pattern| pattern.last_updated > cutoff_date);
        
        let removed_count = initial_count - self.learning_data.len();
        
        if removed_count > 0 {
            info!("Cleaned up {} old learning data points", removed_count);
        }
        
        Ok(removed_count)
    }
}

impl Default for LearningEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_learning_engine_creation() {
        let engine = LearningEngine::new();
        assert_eq!(engine.learning_data.len(), 0);
        assert_eq!(engine.action_patterns.len(), 0);
    }
    
    #[tokio::test]
    async fn test_record_learning_data() {
        let mut engine = LearningEngine::new();
        
        let data = LearningData {
            action_type: "click".to_string(),
            parameters: {
                let mut params = HashMap::new();
                params.insert("selector".to_string(), serde_json::Value::String("button.submit".to_string()));
                params
            },
            expected_outcome: "Form submission".to_string(),
            actual_outcome: "Form submitted successfully".to_string(),
            success: true,
            execution_time_ms: 1500,
            confidence: 0.9,
            timestamp: chrono::Utc::now(),
        };
        
        engine.record_learning_data(data).await.unwrap();
        
        assert_eq!(engine.learning_data.len(), 1);
        assert_eq!(engine.action_patterns.len(), 1);
        
        let stats = engine.get_statistics().await;
        assert_eq!(stats.total_samples, 1);
        assert_eq!(stats.success_rate, 1.0);
    }
    
    #[tokio::test]
    async fn test_pattern_confidence_calculation() {
        let mut engine = LearningEngine::new();
        
        // Record multiple successful attempts for same pattern
        for i in 0..15 {
            let data = LearningData {
                action_type: "click".to_string(),
                parameters: {
                    let mut params = HashMap::new();
                    params.insert("selector".to_string(), serde_json::Value::String("button.test".to_string()));
                    params
                },
                expected_outcome: "Click".to_string(),
                actual_outcome: "Clicked".to_string(),
                success: true,
                execution_time_ms: 1000 + i * 100,
                confidence: 0.8,
                timestamp: chrono::Utc::now(),
            };
            
            engine.record_learning_data(data).await.unwrap();
        }
        
        let pattern = engine.get_best_pattern("click").await.unwrap();
        assert!(pattern.confidence_score > 0.9); // Should be high confidence
        assert_eq!(pattern.success_rate, 1.0);
        assert_eq!(pattern.total_attempts, 15);
    }
    
    #[tokio::test]
    async fn test_improvement_recommendations() {
        let mut engine = LearningEngine::new();
        
        // Record some failures
        for _ in 0..5 {
            let data = LearningData {
                action_type: "click".to_string(),
                parameters: HashMap::new(),
                expected_outcome: "Click".to_string(),
                actual_outcome: "Failed".to_string(),
                success: false,
                execution_time_ms: 5000,
                confidence: 0.3,
                timestamp: chrono::Utc::now(),
            };
            
            engine.record_learning_data(data).await.unwrap();
        }
        
        let recommendations = engine.get_improvement_recommendations("click").await;
        assert!(!recommendations.is_empty());
        assert!(recommendations.iter().any(|r| r.contains("reliable selectors")));
    }
}