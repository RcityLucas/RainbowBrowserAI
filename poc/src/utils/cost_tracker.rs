use anyhow::Result;
use chrono::{DateTime, Utc, NaiveDate};
use serde::{Deserialize, Serialize};
use std::fs;
use tracing::{info, warn};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CostTracker {
    pub total_spent: f64,
    pub daily_budget: f64,
    pub operations: Vec<Operation>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Operation {
    pub timestamp: DateTime<Utc>,
    pub operation_type: String,
    pub description: String,
    pub cost: f64,
    pub success: bool,
}

#[derive(Debug)]
pub struct BudgetStatus {
    pub daily_spent: f64,
    pub daily_remaining: f64,
    pub daily_budget: f64,
    pub percentage_used: f64,
    pub can_proceed: bool,
}

impl CostTracker {
    /// Create a new cost tracker with specified daily budget
    pub fn new(daily_budget: f64) -> Self {
        Self {
            total_spent: 0.0,
            daily_budget,
            operations: Vec::new(),
            created_at: Utc::now(),
        }
    }

    /// Check if we can afford a specific operation
    pub fn can_afford(&self, estimated_cost: f64) -> bool {
        let today_spent = self.get_today_spent();
        let remaining = self.daily_budget - today_spent;
        
        if remaining >= estimated_cost {
            true
        } else {
            warn!(
                "Cannot afford operation: ${:.4} (remaining: ${:.4})", 
                estimated_cost, remaining
            );
            false
        }
    }

    /// Record a completed operation
    pub fn record_operation(
        &mut self, 
        op_type: String, 
        description: String, 
        cost: f64, 
        success: bool
    ) -> Result<()> {
        let operation = Operation {
            timestamp: Utc::now(),
            operation_type: op_type.clone(),
            description: description.clone(),
            cost,
            success,
        };

        self.total_spent += cost;
        self.operations.push(operation);

        info!(
            "Recorded operation: {} - {} (${:.4}) - {}", 
            op_type, description, cost, if success { "SUCCESS" } else { "FAILED" }
        );

        // Save to file after each operation
        self.save_to_file()?;

        // Check budget status
        let status = self.get_budget_status();
        if status.percentage_used > 80.0 {
            warn!(
                "High budget usage: {:.1}% (${:.4} / ${:.4})", 
                status.percentage_used, status.daily_spent, status.daily_budget
            );
        }

        Ok(())
    }

    /// Get today's spending
    pub fn get_today_spent(&self) -> f64 {
        let today = Utc::now().date_naive();
        self.operations
            .iter()
            .filter(|op| op.timestamp.date_naive() == today)
            .map(|op| op.cost)
            .sum()
    }
    
    /// Alias for get_today_spent for API compatibility
    pub fn get_daily_total(&self) -> f64 {
        self.get_today_spent()
    }

    /// Get budget status for today
    pub fn get_budget_status(&self) -> BudgetStatus {
        let daily_spent = self.get_today_spent();
        let daily_remaining = (self.daily_budget - daily_spent).max(0.0);
        let percentage_used = if self.daily_budget > 0.0 {
            (daily_spent / self.daily_budget) * 100.0
        } else {
            0.0
        };

        BudgetStatus {
            daily_spent,
            daily_remaining,
            daily_budget: self.daily_budget,
            percentage_used,
            can_proceed: daily_remaining > 0.01, // At least 1 cent remaining
        }
    }

    /// Get operations for a specific date
    pub fn get_operations_for_date(&self, date: NaiveDate) -> Vec<&Operation> {
        self.operations
            .iter()
            .filter(|op| op.timestamp.date_naive() == date)
            .collect()
    }

    /// Get total number of operations today
    pub fn get_today_operation_count(&self) -> usize {
        let today = Utc::now().date_naive();
        self.operations
            .iter()
            .filter(|op| op.timestamp.date_naive() == today)
            .count()
    }

    /// Get success rate for today
    pub fn get_today_success_rate(&self) -> f64 {
        let today = Utc::now().date_naive();
        let today_ops: Vec<_> = self.operations
            .iter()
            .filter(|op| op.timestamp.date_naive() == today)
            .collect();

        if today_ops.is_empty() {
            return 100.0;
        }

        let successful = today_ops.iter().filter(|op| op.success).count();
        (successful as f64 / today_ops.len() as f64) * 100.0
    }

    /// Save cost tracker to file
    fn save_to_file(&self) -> Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write("cost_tracker.json", json)?;
        Ok(())
    }

    /// Load cost tracker from file
    pub fn load_from_file() -> Result<Self> {
        let data = fs::read_to_string("cost_tracker.json")?;
        let tracker: CostTracker = serde_json::from_str(&data)?;
        info!(
            "Loaded cost tracker: ${:.4} total spent, {} operations", 
            tracker.total_spent, tracker.operations.len()
        );
        Ok(tracker)
    }

    /// Generate a daily report
    pub fn generate_daily_report(&self) -> String {
        let status = self.get_budget_status();
        let operation_count = self.get_today_operation_count();
        let success_rate = self.get_today_success_rate();

        format!(
            "=== Daily Cost Report ===\n\
             Date: {}\n\
             Budget: ${:.4}\n\
             Spent: ${:.4} ({:.1}%)\n\
             Remaining: ${:.4}\n\
             Operations: {}\n\
             Success Rate: {:.1}%\n\
             Total All-Time: ${:.4}\n\
             ========================",
            Utc::now().format("%Y-%m-%d"),
            status.daily_budget,
            status.daily_spent,
            status.percentage_used,
            status.daily_remaining,
            operation_count,
            success_rate,
            self.total_spent
        )
    }

    /// Estimate cost for a browser operation
    pub fn estimate_browser_operation_cost(&self) -> f64 {
        0.01 // Simple browser operations cost $0.01 in PoC
    }

    /// Estimate cost for an LLM operation
    pub fn estimate_llm_operation_cost(&self, prompt_length: usize) -> f64 {
        // Rough estimate: ~4 chars per token, GPT-3.5 pricing
        let input_tokens = (prompt_length as f64 / 4.0).ceil();
        let output_tokens = 50.0; // Estimated response length
        
        // GPT-3.5 pricing: $0.0005 input, $0.0015 output per 1K tokens
        let input_cost = (input_tokens / 1000.0) * 0.0005;
        let output_cost = (output_tokens / 1000.0) * 0.0015;
        
        input_cost + output_cost
    }
}