// Cost Tracking Module
// Monitors and limits LLM API usage costs

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc, Datelike, Timelike};
use tracing::{info, warn};

use super::LLMResponse;

/// Tracks LLM usage and costs
#[derive(Debug, Clone)]
pub struct CostTracker {
    metrics: UsageMetrics,
    pricing: HashMap<String, ModelPricing>,
    limits: CostLimits,
    usage_history: Vec<UsageRecord>,
}

/// Current usage metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageMetrics {
    pub total_requests: u64,
    pub total_prompt_tokens: u64,
    pub total_completion_tokens: u64,
    pub total_tokens: u64,
    pub total_cost_usd: f64,
    pub requests_by_model: HashMap<String, u64>,
    pub tokens_by_model: HashMap<String, u64>,
    pub cost_by_model: HashMap<String, f64>,
    pub last_updated: DateTime<Utc>,
}

/// Pricing information for different models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPricing {
    pub model_name: String,
    pub prompt_token_cost: f64,    // Cost per 1K prompt tokens
    pub completion_token_cost: f64, // Cost per 1K completion tokens
    pub request_cost: f64,         // Fixed cost per request (if any)
}

/// Cost limits and controls
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostLimits {
    pub daily_limit_usd: Option<f64>,
    pub monthly_limit_usd: Option<f64>,
    pub total_limit_usd: Option<f64>,
    pub per_request_limit_tokens: Option<u64>,
    pub rate_limit_requests_per_minute: Option<u64>,
    pub alert_thresholds: Vec<f64>, // Alert at these cost percentages
}

/// Individual usage record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageRecord {
    pub timestamp: DateTime<Utc>,
    pub model: String,
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub total_tokens: u64,
    pub estimated_cost: f64,
    pub request_id: Option<String>,
}

/// Cost analysis report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostReport {
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub total_cost: f64,
    pub total_requests: u64,
    pub total_tokens: u64,
    pub average_cost_per_request: f64,
    pub average_tokens_per_request: f64,
    pub cost_by_model: HashMap<String, f64>,
    pub usage_by_hour: HashMap<String, f64>, // Hour -> cost
    pub projection_daily: f64,
    pub projection_monthly: f64,
}

impl CostTracker {
    /// Create new cost tracker with default pricing
    pub fn new() -> Self {
        let mut pricing = HashMap::new();
        
        // OpenAI GPT-4 pricing (as of 2024)
        pricing.insert("gpt-4".to_string(), ModelPricing {
            model_name: "gpt-4".to_string(),
            prompt_token_cost: 0.03,     // $0.03 per 1K tokens
            completion_token_cost: 0.06, // $0.06 per 1K tokens
            request_cost: 0.0,
        });
        
        pricing.insert("gpt-4-turbo".to_string(), ModelPricing {
            model_name: "gpt-4-turbo".to_string(),
            prompt_token_cost: 0.01,
            completion_token_cost: 0.03,
            request_cost: 0.0,
        });
        
        pricing.insert("gpt-3.5-turbo".to_string(), ModelPricing {
            model_name: "gpt-3.5-turbo".to_string(),
            prompt_token_cost: 0.0015,   // $0.0015 per 1K tokens
            completion_token_cost: 0.002, // $0.002 per 1K tokens
            request_cost: 0.0,
        });
        
        // Claude pricing (Anthropic)
        pricing.insert("claude-3-sonnet-20240229".to_string(), ModelPricing {
            model_name: "claude-3-sonnet-20240229".to_string(),
            prompt_token_cost: 0.003,    // $0.003 per 1K tokens
            completion_token_cost: 0.015, // $0.015 per 1K tokens
            request_cost: 0.0,
        });
        
        pricing.insert("claude-3-opus-20240229".to_string(), ModelPricing {
            model_name: "claude-3-opus-20240229".to_string(),
            prompt_token_cost: 0.015,
            completion_token_cost: 0.075,
            request_cost: 0.0,
        });
        
        Self {
            metrics: UsageMetrics {
                total_requests: 0,
                total_prompt_tokens: 0,
                total_completion_tokens: 0,
                total_tokens: 0,
                total_cost_usd: 0.0,
                requests_by_model: HashMap::new(),
                tokens_by_model: HashMap::new(),
                cost_by_model: HashMap::new(),
                last_updated: Utc::now(),
            },
            pricing,
            limits: CostLimits {
                daily_limit_usd: Some(10.0),   // $10 daily limit
                monthly_limit_usd: Some(100.0), // $100 monthly limit
                total_limit_usd: Some(500.0),  // $500 total limit
                per_request_limit_tokens: Some(8000),
                rate_limit_requests_per_minute: Some(60),
                alert_thresholds: vec![0.5, 0.8, 0.9], // Alert at 50%, 80%, 90%
            },
            usage_history: Vec::new(),
        }
    }
    
    /// Create cost tracker with custom limits
    pub fn with_limits(limits: CostLimits) -> Self {
        let mut tracker = Self::new();
        tracker.limits = limits;
        tracker
    }
    
    /// Track usage from an LLM response
    pub fn track_usage(&mut self, response: &LLMResponse) {
        let timestamp = Utc::now();
        let model = &response.model;
        let usage = &response.usage;
        
        // Calculate cost
        let cost = self.calculate_cost(model, usage.prompt_tokens, usage.completion_tokens);
        
        // Update metrics
        self.metrics.total_requests += 1;
        self.metrics.total_prompt_tokens += usage.prompt_tokens as u64;
        self.metrics.total_completion_tokens += usage.completion_tokens as u64;
        self.metrics.total_tokens += usage.total_tokens as u64;
        self.metrics.total_cost_usd += cost;
        self.metrics.last_updated = timestamp;
        
        // Update per-model metrics
        *self.metrics.requests_by_model.entry(model.clone()).or_insert(0) += 1;
        *self.metrics.tokens_by_model.entry(model.clone()).or_insert(0) += usage.total_tokens as u64;
        *self.metrics.cost_by_model.entry(model.clone()).or_insert(0.0) += cost;
        
        // Add to history
        let record = UsageRecord {
            timestamp,
            model: model.clone(),
            prompt_tokens: usage.prompt_tokens as u64,
            completion_tokens: usage.completion_tokens as u64,
            total_tokens: usage.total_tokens as u64,
            estimated_cost: cost,
            request_id: None, // Could be added later
        };
        
        self.usage_history.push(record);
        
        // Check limits and alerts
        self.check_limits();
        
        info!("Tracked usage: {} tokens, ${:.4} cost for model {}", 
            usage.total_tokens, cost, model);
    }
    
    /// Calculate cost for a request
    pub fn calculate_cost(&self, model: &str, prompt_tokens: u32, completion_tokens: u32) -> f64 {
        if let Some(pricing) = self.pricing.get(model) {
            let prompt_cost = (prompt_tokens as f64 / 1000.0) * pricing.prompt_token_cost;
            let completion_cost = (completion_tokens as f64 / 1000.0) * pricing.completion_token_cost;
            prompt_cost + completion_cost + pricing.request_cost
        } else {
            warn!("No pricing information for model: {}", model);
            // Default fallback pricing
            let prompt_cost = (prompt_tokens as f64 / 1000.0) * 0.01;
            let completion_cost = (completion_tokens as f64 / 1000.0) * 0.02;
            prompt_cost + completion_cost
        }
    }
    
    /// Check if usage is within limits
    pub fn is_within_limits(&self) -> bool {
        // Check total limit
        if let Some(total_limit) = self.limits.total_limit_usd {
            if self.metrics.total_cost_usd >= total_limit {
                return false;
            }
        }
        
        // Check daily limit
        if let Some(daily_limit) = self.limits.daily_limit_usd {
            let daily_cost = self.get_daily_cost();
            if daily_cost >= daily_limit {
                return false;
            }
        }
        
        // Check monthly limit
        if let Some(monthly_limit) = self.limits.monthly_limit_usd {
            let monthly_cost = self.get_monthly_cost();
            if monthly_cost >= monthly_limit {
                return false;
            }
        }
        
        true
    }
    
    /// Check limits and issue warnings
    fn check_limits(&self) {
        for threshold in &self.limits.alert_thresholds {
            if let Some(daily_limit) = self.limits.daily_limit_usd {
                let daily_cost = self.get_daily_cost();
                let usage_percent = daily_cost / daily_limit;
                
                if usage_percent >= *threshold {
                    warn!("Daily cost alert: ${:.2} ({:.1}% of ${:.2} limit)", 
                        daily_cost, usage_percent * 100.0, daily_limit);
                }
            }
            
            if let Some(monthly_limit) = self.limits.monthly_limit_usd {
                let monthly_cost = self.get_monthly_cost();
                let usage_percent = monthly_cost / monthly_limit;
                
                if usage_percent >= *threshold {
                    warn!("Monthly cost alert: ${:.2} ({:.1}% of ${:.2} limit)", 
                        monthly_cost, usage_percent * 100.0, monthly_limit);
                }
            }
        }
    }
    
    /// Get today's cost
    pub fn get_daily_cost(&self) -> f64 {
        let today = Utc::now().date_naive();
        self.usage_history.iter()
            .filter(|record| record.timestamp.date_naive() == today)
            .map(|record| record.estimated_cost)
            .sum()
    }
    
    /// Get this month's cost
    pub fn get_monthly_cost(&self) -> f64 {
        let now = Utc::now();
        let month_start = now.with_day(1).unwrap()
            .with_hour(0).unwrap()
            .with_minute(0).unwrap()
            .with_second(0).unwrap()
            .with_nanosecond(0).unwrap();
        
        self.usage_history.iter()
            .filter(|record| record.timestamp >= month_start)
            .map(|record| record.estimated_cost)
            .sum()
    }
    
    /// Get usage metrics
    pub fn get_metrics(&self) -> &UsageMetrics {
        &self.metrics
    }
    
    /// Generate cost report for a period
    pub fn generate_report(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> CostReport {
        let period_records: Vec<_> = self.usage_history.iter()
            .filter(|record| record.timestamp >= start && record.timestamp <= end)
            .collect();
            
        let total_cost: f64 = period_records.iter().map(|r| r.estimated_cost).sum();
        let total_requests = period_records.len() as u64;
        let total_tokens: u64 = period_records.iter().map(|r| r.total_tokens).sum();
        
        let average_cost_per_request = if total_requests > 0 {
            total_cost / total_requests as f64
        } else {
            0.0
        };
        
        let average_tokens_per_request = if total_requests > 0 {
            total_tokens as f64 / total_requests as f64
        } else {
            0.0
        };
        
        // Cost by model
        let mut cost_by_model = HashMap::new();
        for record in &period_records {
            *cost_by_model.entry(record.model.clone()).or_insert(0.0) += record.estimated_cost;
        }
        
        // Usage by hour
        let mut usage_by_hour = HashMap::new();
        for record in &period_records {
            let hour = record.timestamp.format("%H").to_string();
            *usage_by_hour.entry(hour).or_insert(0.0) += record.estimated_cost;
        }
        
        // Projections
        let period_duration = (end - start).num_seconds() as f64;
        let daily_projection = if period_duration > 0.0 {
            total_cost * (86400.0 / period_duration) // 86400 seconds in a day
        } else {
            0.0
        };
        
        let monthly_projection = daily_projection * 30.0;
        
        CostReport {
            period_start: start,
            period_end: end,
            total_cost,
            total_requests,
            total_tokens,
            average_cost_per_request,
            average_tokens_per_request,
            cost_by_model,
            usage_by_hour,
            projection_daily: daily_projection,
            projection_monthly: monthly_projection,
        }
    }
    
    /// Reset metrics (useful for testing)
    pub fn reset(&mut self) {
        self.metrics = UsageMetrics {
            total_requests: 0,
            total_prompt_tokens: 0,
            total_completion_tokens: 0,
            total_tokens: 0,
            total_cost_usd: 0.0,
            requests_by_model: HashMap::new(),
            tokens_by_model: HashMap::new(),
            cost_by_model: HashMap::new(),
            last_updated: Utc::now(),
        };
        self.usage_history.clear();
    }
    
    /// Add custom model pricing
    pub fn add_model_pricing(&mut self, pricing: ModelPricing) {
        self.pricing.insert(pricing.model_name.clone(), pricing);
    }
    
    /// Export usage history as CSV
    pub fn export_csv(&self) -> String {
        let mut csv = String::from("timestamp,model,prompt_tokens,completion_tokens,total_tokens,estimated_cost\n");
        
        for record in &self.usage_history {
            csv.push_str(&format!(
                "{},{},{},{},{},{:.6}\n",
                record.timestamp.to_rfc3339(),
                record.model,
                record.prompt_tokens,
                record.completion_tokens,
                record.total_tokens,
                record.estimated_cost
            ));
        }
        
        csv
    }
}

impl Default for CostTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::{LLMResponse, TokenUsage};
    
    #[test]
    fn test_cost_calculation() {
        let tracker = CostTracker::new();
        
        // Test GPT-4 pricing
        let cost = tracker.calculate_cost("gpt-4", 1000, 500);
        assert!((cost - 0.06).abs() < 0.001); // $0.03 + $0.03 = $0.06
        
        // Test GPT-3.5 pricing
        let cost = tracker.calculate_cost("gpt-3.5-turbo", 1000, 1000);
        assert!((cost - 0.0035).abs() < 0.001); // $0.0015 + $0.002 = $0.0035
    }
    
    #[test]
    fn test_usage_tracking() {
        let mut tracker = CostTracker::new();
        
        let response = LLMResponse {
            content: "Test response".to_string(),
            model: "gpt-4".to_string(),
            usage: TokenUsage {
                prompt_tokens: 100,
                completion_tokens: 50,
                total_tokens: 150,
            },
            finish_reason: "stop".to_string(),
            timestamp: Utc::now(),
        };
        
        tracker.track_usage(&response);
        
        assert_eq!(tracker.metrics.total_requests, 1);
        assert_eq!(tracker.metrics.total_tokens, 150);
        assert!(tracker.metrics.total_cost_usd > 0.0);
        assert_eq!(tracker.usage_history.len(), 1);
    }
    
    #[test]
    fn test_limits() {
        let limits = CostLimits {
            daily_limit_usd: Some(1.0),
            monthly_limit_usd: Some(10.0),
            total_limit_usd: Some(100.0),
            per_request_limit_tokens: Some(1000),
            rate_limit_requests_per_minute: Some(10),
            alert_thresholds: vec![0.8, 0.9],
        };
        
        let mut tracker = CostTracker::with_limits(limits);
        
        // Should be within limits initially
        assert!(tracker.is_within_limits());
        
        // Add usage that exceeds daily limit
        for _ in 0..50 {
            let response = LLMResponse {
                content: "Test".to_string(),
                model: "gpt-4".to_string(),
                usage: TokenUsage {
                    prompt_tokens: 1000,
                    completion_tokens: 1000,
                    total_tokens: 2000,
                },
                finish_reason: "stop".to_string(),
                timestamp: Utc::now(),
            };
            tracker.track_usage(&response);
        }
        
        // Should exceed limits now
        assert!(!tracker.is_within_limits());
    }
    
    #[test]
    fn test_report_generation() {
        let mut tracker = CostTracker::new();
        
        let start = Utc::now() - chrono::Duration::hours(1);
        let end = Utc::now();
        
        // Add some usage
        let response = LLMResponse {
            content: "Test".to_string(),
            model: "gpt-4".to_string(),
            usage: TokenUsage {
                prompt_tokens: 100,
                completion_tokens: 50,
                total_tokens: 150,
            },
            finish_reason: "stop".to_string(),
            timestamp: Utc::now(),
        };
        
        tracker.track_usage(&response);
        
        let report = tracker.generate_report(start, end);
        
        assert_eq!(report.total_requests, 1);
        assert_eq!(report.total_tokens, 150);
        assert!(report.total_cost > 0.0);
    }
}