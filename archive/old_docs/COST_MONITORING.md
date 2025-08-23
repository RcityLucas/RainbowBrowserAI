# LLM Cost Monitoring Framework 💰

## Overview
Comprehensive cost tracking system to prevent budget overruns and optimize LLM usage throughout development phases.

## Cost Structure Analysis

### OpenAI API Pricing (Current)
```yaml
GPT-4 Turbo:
  input: $0.01 per 1K tokens
  output: $0.03 per 1K tokens
  context_window: 128K tokens

GPT-3.5 Turbo:
  input: $0.0005 per 1K tokens
  output: $0.0015 per 1K tokens
  context_window: 16K tokens
```

### Expected Token Usage Per Operation
```yaml
Simple Navigation Command:
  prompt: ~200 tokens
  response: ~50 tokens
  total: ~250 tokens
  cost: $0.003 (GPT-4) / $0.0004 (GPT-3.5)

Complex Workflow Analysis:
  prompt: ~1000 tokens
  response: ~300 tokens
  total: ~1300 tokens
  cost: $0.049 (GPT-4) / $0.0065 (GPT-3.5)
```

## Phase-Based Cost Budgets

### Phase 0: Proof of Concept (2 weeks)
```yaml
Total Budget: $5
Daily Budget: $0.50
Operations Budget: 100 operations @ $0.05 each
Safety Margin: 20%

Model Selection: GPT-3.5 Turbo (cost-optimized)
Expected Usage:
  - Development/testing: 200 calls
  - Demo operations: 50 calls
  - Buffer for debugging: 50 calls
```

### Phase 1: MVP (6-8 weeks)
```yaml
Total Budget: $50/month
Daily Budget: $1.67
Operations Budget: 1000 operations/month @ $0.05 each
Safety Margin: 30%

Model Mix:
  - 80% GPT-3.5 Turbo (routine operations)
  - 20% GPT-4 Turbo (complex reasoning)
```

### Phase 2: Beta (4-6 months)
```yaml
Total Budget: $500/month
Daily Budget: $16.67
User Operations: 10,000 operations/month
Cost per User: $50/month (10 beta users)
```

## Cost Tracking Implementation

### Core Cost Tracker
```rust
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostTracker {
    pub daily_costs: HashMap<String, DailyCost>,
    pub monthly_budgets: HashMap<String, f64>,
    pub total_spent: f64,
    pub last_reset: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyCost {
    pub date: String,
    pub total_tokens: u64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub api_calls: u64,
    pub total_cost: f64,
    pub operations: Vec<Operation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operation {
    pub timestamp: DateTime<Utc>,
    pub operation_type: String,
    pub model: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cost: f64,
    pub success: bool,
}

impl CostTracker {
    pub fn new() -> Self {
        Self {
            daily_costs: HashMap::new(),
            monthly_budgets: HashMap::new(),
            total_spent: 0.0,
            last_reset: Utc::now(),
        }
    }

    pub fn record_operation(&mut self, op: Operation) -> Result<(), CostError> {
        let today = op.timestamp.format("%Y-%m-%d").to_string();
        
        // Check daily budget before recording
        if let Some(daily_budget) = self.get_daily_budget(&today) {
            let current_daily_cost = self.get_daily_cost(&today);
            if current_daily_cost + op.cost > daily_budget {
                return Err(CostError::DailyBudgetExceeded {
                    current: current_daily_cost,
                    limit: daily_budget,
                    attempted: op.cost,
                });
            }
        }

        // Record the operation
        let daily_cost = self.daily_costs.entry(today.clone()).or_insert_with(|| {
            DailyCost {
                date: today,
                total_tokens: 0,
                input_tokens: 0,
                output_tokens: 0,
                api_calls: 0,
                total_cost: 0.0,
                operations: Vec::new(),
            }
        });

        daily_cost.total_tokens += op.input_tokens + op.output_tokens;
        daily_cost.input_tokens += op.input_tokens;
        daily_cost.output_tokens += op.output_tokens;
        daily_cost.api_calls += 1;
        daily_cost.total_cost += op.cost;
        daily_cost.operations.push(op);

        self.total_spent += op.cost;
        Ok(())
    }

    pub fn estimate_operation_cost(&self, prompt: &str, model: &str) -> f64 {
        let input_tokens = estimate_tokens(prompt);
        let output_tokens = estimate_output_tokens(prompt); // Based on operation type
        
        match model {
            "gpt-4-turbo" => {
                (input_tokens as f64 / 1000.0) * 0.01 + 
                (output_tokens as f64 / 1000.0) * 0.03
            },
            "gpt-3.5-turbo" => {
                (input_tokens as f64 / 1000.0) * 0.0005 + 
                (output_tokens as f64 / 1000.0) * 0.0015
            },
            _ => 0.0,
        }
    }

    pub fn check_budget_status(&self) -> BudgetStatus {
        let today = Utc::now().format("%Y-%m-%d").to_string();
        let daily_cost = self.get_daily_cost(&today);
        let daily_budget = self.get_daily_budget(&today).unwrap_or(f64::MAX);
        
        let monthly_cost = self.get_monthly_cost();
        let monthly_budget = self.get_monthly_budget().unwrap_or(f64::MAX);

        BudgetStatus {
            daily_spent: daily_cost,
            daily_budget,
            daily_remaining: daily_budget - daily_cost,
            monthly_spent: monthly_cost,
            monthly_budget,
            monthly_remaining: monthly_budget - monthly_cost,
            warning_level: self.get_warning_level(daily_cost, daily_budget, monthly_cost, monthly_budget),
        }
    }

    fn get_warning_level(&self, daily_cost: f64, daily_budget: f64, monthly_cost: f64, monthly_budget: f64) -> WarningLevel {
        let daily_usage = daily_cost / daily_budget;
        let monthly_usage = monthly_cost / monthly_budget;
        
        if daily_usage >= 1.0 || monthly_usage >= 1.0 {
            WarningLevel::Critical
        } else if daily_usage >= 0.8 || monthly_usage >= 0.8 {
            WarningLevel::High
        } else if daily_usage >= 0.6 || monthly_usage >= 0.6 {
            WarningLevel::Medium
        } else {
            WarningLevel::Normal
        }
    }

    pub fn generate_report(&self) -> CostReport {
        let last_7_days = self.get_last_n_days(7);
        let last_30_days = self.get_last_n_days(30);
        
        CostReport {
            total_spent: self.total_spent,
            last_7_days_cost: last_7_days.iter().map(|d| d.total_cost).sum(),
            last_30_days_cost: last_30_days.iter().map(|d| d.total_cost).sum(),
            total_operations: last_30_days.iter().map(|d| d.api_calls).sum(),
            average_cost_per_operation: self.calculate_average_cost_per_operation(),
            most_expensive_operations: self.get_most_expensive_operations(10),
            cost_by_model: self.get_cost_by_model(),
            daily_trends: last_30_days,
        }
    }
}

#[derive(Debug, Clone)]
pub enum CostError {
    DailyBudgetExceeded { current: f64, limit: f64, attempted: f64 },
    MonthlyBudgetExceeded { current: f64, limit: f64, attempted: f64 },
    InvalidModel(String),
    TokenEstimationFailed,
}

#[derive(Debug, Clone)]
pub struct BudgetStatus {
    pub daily_spent: f64,
    pub daily_budget: f64,
    pub daily_remaining: f64,
    pub monthly_spent: f64,
    pub monthly_budget: f64,
    pub monthly_remaining: f64,
    pub warning_level: WarningLevel,
}

#[derive(Debug, Clone)]
pub enum WarningLevel {
    Normal,    // <60% of budget
    Medium,    // 60-80% of budget
    High,      // 80-100% of budget
    Critical,  // >100% of budget
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostReport {
    pub total_spent: f64,
    pub last_7_days_cost: f64,
    pub last_30_days_cost: f64,
    pub total_operations: u64,
    pub average_cost_per_operation: f64,
    pub most_expensive_operations: Vec<Operation>,
    pub cost_by_model: HashMap<String, f64>,
    pub daily_trends: Vec<DailyCost>,
}

// Token estimation utilities
fn estimate_tokens(text: &str) -> u64 {
    // Rough estimation: ~4 characters per token for English
    (text.len() as f64 / 4.0).ceil() as u64
}

fn estimate_output_tokens(prompt: &str) -> u64 {
    // Estimate based on operation type
    if prompt.contains("navigate") || prompt.contains("click") {
        50 // Simple action responses
    } else if prompt.contains("analyze") || prompt.contains("explain") {
        200 // Detailed analysis
    } else if prompt.contains("workflow") || prompt.contains("complex") {
        500 // Complex workflow responses
    } else {
        100 // Default estimate
    }
}
```

### Budget Configuration
```rust
impl CostTracker {
    pub fn set_phase_budget(&mut self, phase: Phase) {
        match phase {
            Phase::ProofOfConcept => {
                self.set_daily_budget(0.50);
                self.set_monthly_budget(5.0);
            },
            Phase::MVP => {
                self.set_daily_budget(1.67);
                self.set_monthly_budget(50.0);
            },
            Phase::Beta => {
                self.set_daily_budget(16.67);
                self.set_monthly_budget(500.0);
            },
            Phase::Production => {
                self.set_daily_budget(100.0);
                self.set_monthly_budget(3000.0);
            },
        }
    }
}

#[derive(Debug, Clone)]
pub enum Phase {
    ProofOfConcept,
    MVP,
    Beta,
    Production,
}
```

## Cost Optimization Strategies

### Prompt Optimization
```rust
pub struct PromptOptimizer {
    templates: HashMap<String, String>,
    compression_ratios: HashMap<String, f64>,
}

impl PromptOptimizer {
    pub fn optimize_navigation_prompt(&self, url: &str) -> String {
        // Use minimal template for navigation
        format!("Navigate to: {}", url)
    }

    pub fn optimize_analysis_prompt(&self, content: &str) -> String {
        // Compress content while preserving key information
        let compressed = self.compress_content(content, 0.7);
        format!("Analyze this content:\n{}", compressed)
    }

    fn compress_content(&self, content: &str, ratio: f64) -> String {
        // Implement content compression logic
        // Remove redundant information, summarize long sections
        content.chars().take((content.len() as f64 * ratio) as usize).collect()
    }
}
```

### Model Selection Strategy
```rust
pub struct ModelSelector {
    cost_tracker: CostTracker,
}

impl ModelSelector {
    pub fn select_model_for_operation(&self, operation_type: &str, complexity: f64) -> String {
        match operation_type {
            "navigate" | "click" | "type" => "gpt-3.5-turbo".to_string(),
            "analyze" | "explain" if complexity < 0.7 => "gpt-3.5-turbo".to_string(),
            "analyze" | "explain" if complexity >= 0.7 => "gpt-4-turbo".to_string(),
            "workflow" | "complex" => "gpt-4-turbo".to_string(),
            _ => "gpt-3.5-turbo".to_string(), // Default to cheaper model
        }
    }
}
```

### Caching Strategy
```rust
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

pub struct ResponseCache {
    cache: HashMap<String, CachedResponse>,
    max_size: usize,
    ttl_seconds: u64,
}

#[derive(Debug, Clone)]
struct CachedResponse {
    response: String,
    timestamp: DateTime<Utc>,
    cost_saved: f64,
}

impl ResponseCache {
    pub fn get_or_compute<F>(&mut self, prompt: &str, compute_fn: F) -> Result<String, CostError>
    where
        F: Fn(&str) -> Result<(String, f64), CostError>,
    {
        let cache_key = self.hash_prompt(prompt);
        
        // Check if cached response exists and is fresh
        if let Some(cached) = self.cache.get(&cache_key) {
            if self.is_fresh(&cached.timestamp) {
                return Ok(cached.response.clone());
            }
        }

        // Compute new response
        let (response, cost) = compute_fn(prompt)?;
        
        // Cache the response
        self.cache.insert(cache_key, CachedResponse {
            response: response.clone(),
            timestamp: Utc::now(),
            cost_saved: cost,
        });

        Ok(response)
    }

    fn hash_prompt(&self, prompt: &str) -> String {
        use std::hash::DefaultHasher;
        let mut hasher = DefaultHasher::new();
        prompt.hash(&mut hasher);
        hasher.finish().to_string()
    }

    fn is_fresh(&self, timestamp: &DateTime<Utc>) -> bool {
        let now = Utc::now();
        let age = now.signed_duration_since(*timestamp);
        age.num_seconds() < self.ttl_seconds as i64
    }
}
```

## Monitoring Dashboard

### Real-time Cost Monitoring
```rust
pub struct CostDashboard {
    tracker: CostTracker,
    alerts: Vec<CostAlert>,
}

impl CostDashboard {
    pub fn check_and_alert(&mut self) {
        let status = self.tracker.check_budget_status();
        
        match status.warning_level {
            WarningLevel::Medium => {
                self.send_alert(CostAlert::BudgetWarning {
                    percentage: 60.0,
                    daily_spent: status.daily_spent,
                    daily_budget: status.daily_budget,
                });
            },
            WarningLevel::High => {
                self.send_alert(CostAlert::BudgetCritical {
                    percentage: 80.0,
                    remaining: status.daily_remaining,
                });
            },
            WarningLevel::Critical => {
                self.send_alert(CostAlert::BudgetExceeded {
                    overage: status.daily_spent - status.daily_budget,
                });
            },
            _ => {},
        }
    }

    pub fn generate_daily_report(&self) -> String {
        let report = self.tracker.generate_report();
        format!(
            "Daily Cost Report:\n\
             - Total Spent: ${:.4}\n\
             - Operations: {}\n\
             - Avg Cost/Op: ${:.4}\n\
             - Budget Status: {:?}\n\
             - Top Models: {:?}",
            report.last_7_days_cost / 7.0,
            report.total_operations,
            report.average_cost_per_operation,
            self.tracker.check_budget_status().warning_level,
            report.cost_by_model
        )
    }
}

#[derive(Debug, Clone)]
pub enum CostAlert {
    BudgetWarning { percentage: f64, daily_spent: f64, daily_budget: f64 },
    BudgetCritical { percentage: f64, remaining: f64 },
    BudgetExceeded { overage: f64 },
    UnusualSpike { increase_percentage: f64 },
    ModelCostAnomaly { model: String, cost: f64, average: f64 },
}
```

## Integration with Main Application

### Pre-flight Cost Check
```rust
impl LLMService {
    pub async fn execute_with_cost_check(&mut self, prompt: &str) -> Result<String, CostError> {
        // Estimate cost before execution
        let estimated_cost = self.cost_tracker.estimate_operation_cost(prompt, &self.current_model);
        
        // Check budget
        let status = self.cost_tracker.check_budget_status();
        if status.daily_remaining < estimated_cost {
            return Err(CostError::DailyBudgetExceeded {
                current: status.daily_spent,
                limit: status.daily_budget,
                attempted: estimated_cost,
            });
        }

        // Execute operation
        let start_time = Utc::now();
        let response = self.call_llm_api(prompt).await?;
        
        // Record actual cost
        let actual_cost = self.calculate_actual_cost(&response);
        self.cost_tracker.record_operation(Operation {
            timestamp: start_time,
            operation_type: "llm_call".to_string(),
            model: self.current_model.clone(),
            input_tokens: estimate_tokens(prompt),
            output_tokens: estimate_tokens(&response),
            cost: actual_cost,
            success: true,
        })?;

        Ok(response)
    }
}
```

### Phase Transition Checks
```rust
impl PhaseManager {
    pub fn can_proceed_to_next_phase(&self) -> Result<bool, String> {
        let report = self.cost_tracker.generate_report();
        
        match self.current_phase {
            Phase::ProofOfConcept => {
                if report.total_spent > 5.0 {
                    return Err("PoC budget exceeded. Cannot proceed to MVP.".to_string());
                }
                if report.average_cost_per_operation > 0.05 {
                    return Err("Cost per operation too high for MVP viability.".to_string());
                }
            },
            Phase::MVP => {
                if report.last_30_days_cost > 50.0 {
                    return Err("MVP monthly budget exceeded.".to_string());
                }
            },
            _ => {},
        }
        
        Ok(true)
    }
}
```

## Cost Reporting

### Weekly Cost Report
```rust
impl CostReporter {
    pub fn generate_weekly_report(&self) -> WeeklyReport {
        let last_week = self.cost_tracker.get_last_n_days(7);
        
        WeeklyReport {
            total_cost: last_week.iter().map(|d| d.total_cost).sum(),
            total_operations: last_week.iter().map(|d| d.api_calls).sum(),
            cost_by_day: last_week.clone(),
            cost_by_model: self.calculate_model_costs(&last_week),
            efficiency_metrics: self.calculate_efficiency_metrics(&last_week),
            budget_variance: self.calculate_budget_variance(&last_week),
            recommendations: self.generate_cost_recommendations(&last_week),
        }
    }
}
```

This comprehensive cost monitoring framework ensures budget control throughout all development phases while providing actionable insights for optimization.