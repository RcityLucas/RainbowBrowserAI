//! HistoryTracker Tool - V8.0 Standard Tool #10
//! 
//! Tracks and manages operation history for browser automation workflows.
//! Enables replay, analysis, and learning from past actions.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc, Duration as ChronoDuration};

use crate::browser::Browser;
use crate::tools::{Tool, Result};
use crate::tools::errors::ToolError;
use super::{MemoryTool, MemoryStats, ActionRecord};

/// Input parameters for HistoryTracker operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryTrackerInput {
    pub operation: HistoryOperation,
}

/// History operations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum HistoryOperation {
    Record {
        action: ActionRecord,
    },
    Search {
        query: SearchQuery,
    },
    Replay {
        from_id: Option<Uuid>,
        to_id: Option<Uuid>,
        filter: Option<ReplayFilter>,
    },
    Analyze {
        time_range: Option<TimeRange>,
        group_by: Option<GroupBy>,
    },
    Export {
        format: ExportFormat,
        time_range: Option<TimeRange>,
    },
    Clear {
        older_than: Option<i64>, // seconds
    },
    GetStats,
}

/// Search query for history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub tool_name: Option<String>,
    pub action_type: Option<String>,
    pub success: Option<bool>,
    pub time_range: Option<TimeRange>,
    pub limit: Option<usize>,
}

/// Time range for filtering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    pub start: Option<DateTime<Utc>>,
    pub end: Option<DateTime<Utc>>,
}

/// Replay filter options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayFilter {
    pub tool_names: Option<Vec<String>>,
    pub skip_failed: bool,
    pub speed_multiplier: f32,
}

/// Grouping options for analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GroupBy {
    Tool,
    ActionType,
    Hour,
    Day,
    Success,
}

/// Export format options
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExportFormat {
    Json,
    Csv,
    Timeline,
    Summary,
}

/// Output from HistoryTracker operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryTrackerOutput {
    pub success: bool,
    pub operation: String,
    pub result: Option<Value>,
    pub count: usize,
    pub timestamp: DateTime<Utc>,
}

/// History storage with compression
struct HistoryStorage {
    actions: VecDeque<ActionRecord>,
    max_history: usize,
    compression_enabled: bool,
    index_by_tool: std::collections::HashMap<String, Vec<usize>>,
    index_by_time: std::collections::BTreeMap<i64, Vec<usize>>,
}

/// HistoryTracker tool implementation
pub struct HistoryTracker {
    browser: Arc<Browser>,
    storage: Arc<RwLock<HistoryStorage>>,
}

impl HistoryTracker {
    /// Create a new HistoryTracker instance
    pub fn new(browser: Arc<Browser>) -> Self {
        Self::with_max_history(browser, 10000)
    }

    /// Create with custom max history size
    pub fn with_max_history(browser: Arc<Browser>, max_history: usize) -> Self {
        let storage = HistoryStorage {
            actions: VecDeque::with_capacity(max_history),
            max_history,
            compression_enabled: true,
            index_by_tool: std::collections::HashMap::new(),
            index_by_time: std::collections::BTreeMap::new(),
        };

        Self {
            browser,
            storage: Arc::new(RwLock::new(storage)),
        }
    }

    /// Record a new action
    async fn record_action(&self, mut action: ActionRecord) -> Result<()> {
        let mut storage = self.storage.write().await;
        
        // Ensure action has an ID
        if action.id == Uuid::nil() {
            action.id = Uuid::new_v4();
        }
        
        // Check if we need to remove old entries
        if storage.actions.len() >= storage.max_history {
            if let Some(old_action) = storage.actions.pop_front() {
                // Update indices
                self.remove_from_indices(&mut storage, &old_action);
            }
        }
        
        // Add to storage
        let index = storage.actions.len();
        storage.actions.push_back(action.clone());
        
        // Update indices
        storage.index_by_tool
            .entry(action.tool_name.clone())
            .or_insert_with(Vec::new)
            .push(index);
        
        storage.index_by_time
            .entry(action.timestamp.timestamp())
            .or_insert_with(Vec::new)
            .push(index);
        
        Ok(())
    }

    /// Remove action from indices
    fn remove_from_indices(&self, storage: &mut HistoryStorage, action: &ActionRecord) {
        // Remove from tool index
        if let Some(indices) = storage.index_by_tool.get_mut(&action.tool_name) {
            indices.retain(|&i| i > 0); // Shift indices down
            for idx in indices.iter_mut() {
                *idx -= 1;
            }
        }
        
        // Remove from time index
        if let Some(indices) = storage.index_by_time.get_mut(&action.timestamp.timestamp()) {
            indices.retain(|&i| i > 0);
            for idx in indices.iter_mut() {
                *idx -= 1;
            }
        }
    }

    /// Search history with query
    async fn search_history(&self, query: SearchQuery) -> Result<Vec<ActionRecord>> {
        let storage = self.storage.read().await;
        let mut results = Vec::new();
        let limit = query.limit.unwrap_or(100);
        
        for action in &storage.actions {
            if results.len() >= limit {
                break;
            }
            
            // Apply filters
            if let Some(ref tool_name) = query.tool_name {
                if &action.tool_name != tool_name {
                    continue;
                }
            }
            
            if let Some(ref action_type) = query.action_type {
                if &action.action_type != action_type {
                    continue;
                }
            }
            
            if let Some(success) = query.success {
                if action.success != success {
                    continue;
                }
            }
            
            if let Some(ref time_range) = query.time_range {
                if let Some(start) = time_range.start {
                    if action.timestamp < start {
                        continue;
                    }
                }
                if let Some(end) = time_range.end {
                    if action.timestamp > end {
                        continue;
                    }
                }
            }
            
            results.push(action.clone());
        }
        
        Ok(results)
    }

    /// Replay a sequence of actions
    async fn replay_history(
        &self,
        from_id: Option<Uuid>,
        to_id: Option<Uuid>,
        filter: Option<ReplayFilter>,
    ) -> Result<Vec<ActionRecord>> {
        let storage = self.storage.read().await;
        let mut replay_list = Vec::new();
        
        let mut in_range = from_id.is_none();
        
        for action in &storage.actions {
            // Check if we've reached the start
            if let Some(from) = from_id {
                if action.id == from {
                    in_range = true;
                }
            }
            
            if !in_range {
                continue;
            }
            
            // Apply filters
            if let Some(ref f) = filter {
                if f.skip_failed && !action.success {
                    continue;
                }
                
                if let Some(ref tools) = f.tool_names {
                    if !tools.contains(&action.tool_name) {
                        continue;
                    }
                }
            }
            
            replay_list.push(action.clone());
            
            // Check if we've reached the end
            if let Some(to) = to_id {
                if action.id == to {
                    break;
                }
            }
        }
        
        Ok(replay_list)
    }

    /// Analyze history with grouping
    async fn analyze_history(
        &self,
        time_range: Option<TimeRange>,
        group_by: Option<GroupBy>,
    ) -> Result<Value> {
        let storage = self.storage.read().await;
        
        let mut stats = serde_json::json!({
            "total_actions": 0,
            "successful_actions": 0,
            "failed_actions": 0,
            "average_duration_ms": 0.0,
            "tools_used": {},
            "timeline": [],
        });
        
        let mut total_duration: u64 = 0;
        let mut tool_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        let mut timeline_data: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();
        
        for action in &storage.actions {
            // Apply time range filter
            if let Some(ref range) = time_range {
                if let Some(start) = range.start {
                    if action.timestamp < start {
                        continue;
                    }
                }
                if let Some(end) = range.end {
                    if action.timestamp > end {
                        continue;
                    }
                }
            }
            
            stats["total_actions"] = serde_json::json!(stats["total_actions"].as_u64().unwrap_or(0) + 1);
            
            if action.success {
                stats["successful_actions"] = serde_json::json!(stats["successful_actions"].as_u64().unwrap_or(0) + 1);
            } else {
                stats["failed_actions"] = serde_json::json!(stats["failed_actions"].as_u64().unwrap_or(0) + 1);
            }
            
            total_duration += action.duration_ms;
            *tool_counts.entry(action.tool_name.clone()).or_insert(0) += 1;
            
            // Group by timeline if requested
            if let Some(ref grouping) = group_by {
                let key = match grouping {
                    GroupBy::Tool => action.tool_name.clone(),
                    GroupBy::ActionType => action.action_type.clone(),
                    GroupBy::Hour => action.timestamp.format("%Y-%m-%d %H:00").to_string(),
                    GroupBy::Day => action.timestamp.format("%Y-%m-%d").to_string(),
                    GroupBy::Success => action.success.to_string(),
                };
                *timeline_data.entry(key).or_insert(0) += 1;
            }
        }
        
        let total = stats["total_actions"].as_u64().unwrap_or(1).max(1);
        stats["average_duration_ms"] = serde_json::json!(total_duration as f64 / total as f64);
        stats["tools_used"] = serde_json::json!(tool_counts);
        
        if !timeline_data.is_empty() {
            stats["timeline"] = serde_json::json!(timeline_data);
        }
        
        Ok(stats)
    }

    /// Export history in specified format
    async fn export_history(
        &self,
        format: ExportFormat,
        time_range: Option<TimeRange>,
    ) -> Result<Value> {
        let storage = self.storage.read().await;
        
        let mut actions: Vec<&ActionRecord> = storage.actions.iter().collect();
        
        // Apply time range filter
        if let Some(ref range) = time_range {
            actions.retain(|action| {
                let mut keep = true;
                if let Some(start) = range.start {
                    keep &= action.timestamp >= start;
                }
                if let Some(end) = range.end {
                    keep &= action.timestamp <= end;
                }
                keep
            });
        }
        
        match format {
            ExportFormat::Json => {
                Ok(serde_json::json!(actions))
            }
            ExportFormat::Csv => {
                let mut csv = String::from("id,timestamp,tool_name,action_type,success,duration_ms,error\n");
                for action in actions {
                    csv.push_str(&format!(
                        "{},{},{},{},{},{},{}\n",
                        action.id,
                        action.timestamp.to_rfc3339(),
                        action.tool_name,
                        action.action_type,
                        action.success,
                        action.duration_ms,
                        action.error.as_deref().unwrap_or("")
                    ));
                }
                Ok(serde_json::json!(csv))
            }
            ExportFormat::Timeline => {
                let timeline: Vec<_> = actions.iter().map(|a| {
                    serde_json::json!({
                        "time": a.timestamp.to_rfc3339(),
                        "event": format!("{} - {}", a.tool_name, a.action_type),
                        "success": a.success,
                        "duration": a.duration_ms,
                    })
                }).collect();
                Ok(serde_json::json!(timeline))
            }
            ExportFormat::Summary => {
                let total = actions.len();
                let successful = actions.iter().filter(|a| a.success).count();
                let failed = total - successful;
                let avg_duration = if total > 0 {
                    actions.iter().map(|a| a.duration_ms).sum::<u64>() / total as u64
                } else {
                    0
                };
                
                Ok(serde_json::json!({
                    "total_actions": total,
                    "successful": successful,
                    "failed": failed,
                    "success_rate": (successful as f64 / total.max(1) as f64) * 100.0,
                    "average_duration_ms": avg_duration,
                    "time_range": {
                        "start": actions.first().map(|a| a.timestamp.to_rfc3339()),
                        "end": actions.last().map(|a| a.timestamp.to_rfc3339()),
                    }
                }))
            }
        }
    }

    /// Clear old history entries
    async fn clear_old_history(&self, older_than_seconds: Option<i64>) -> Result<usize> {
        let mut storage = self.storage.write().await;
        
        if let Some(seconds) = older_than_seconds {
            let cutoff = Utc::now() - ChronoDuration::seconds(seconds);
            let original_len = storage.actions.len();
            
            storage.actions.retain(|action| action.timestamp >= cutoff);
            
            // Rebuild indices
            storage.index_by_tool.clear();
            storage.index_by_time.clear();
            
            // Collect data to avoid borrowing conflicts
            let actions_data: Vec<(usize, String, i64)> = storage.actions.iter().enumerate()
                .map(|(index, action)| (index, action.tool_name.clone(), action.timestamp.timestamp()))
                .collect();
            
            for (index, tool_name, timestamp) in actions_data {
                storage.index_by_tool
                    .entry(tool_name)
                    .or_insert_with(Vec::new)
                    .push(index);
                
                storage.index_by_time
                    .entry(timestamp)
                    .or_insert_with(Vec::new)
                    .push(index);
            }
            
            Ok(original_len - storage.actions.len())
        } else {
            let count = storage.actions.len();
            storage.actions.clear();
            storage.index_by_tool.clear();
            storage.index_by_time.clear();
            Ok(count)
        }
    }
}

#[async_trait]
impl Tool for HistoryTracker {
    type Input = HistoryTrackerInput;
    type Output = HistoryTrackerOutput;

    fn name(&self) -> &str {
        "history_tracker"
    }

    fn description(&self) -> &str {
        "Tracks and manages operation history for analysis and replay"
    }

    async fn execute(&self, input: Self::Input) -> Result<Self::Output> {
        let timestamp = Utc::now();
        
        let (operation_name, result, count) = match input.operation {
            HistoryOperation::Record { action } => {
                self.record_action(action).await?;
                ("record", None, 1)
            }
            HistoryOperation::Search { query } => {
                let results = self.search_history(query).await?;
                let count = results.len();
                ("search", Some(serde_json::json!(results)), count)
            }
            HistoryOperation::Replay { from_id, to_id, filter } => {
                let replay_list = self.replay_history(from_id, to_id, filter).await?;
                let count = replay_list.len();
                ("replay", Some(serde_json::json!(replay_list)), count)
            }
            HistoryOperation::Analyze { time_range, group_by } => {
                let analysis = self.analyze_history(time_range, group_by).await?;
                ("analyze", Some(analysis), 0)
            }
            HistoryOperation::Export { format, time_range } => {
                let exported = self.export_history(format, time_range).await?;
                ("export", Some(exported), 0)
            }
            HistoryOperation::Clear { older_than } => {
                let cleared = self.clear_old_history(older_than).await?;
                ("clear", Some(serde_json::json!(cleared)), cleared)
            }
            HistoryOperation::GetStats => {
                let stats = self.stats().await?;
                ("get_stats", Some(serde_json::to_value(stats)?), 0)
            }
        };
        
        Ok(HistoryTrackerOutput {
            success: true,
            operation: operation_name.to_string(),
            result,
            count,
            timestamp,
        })
    }
}

#[async_trait]
impl MemoryTool for HistoryTracker {
    async fn store(&self, key: String, value: Value) -> Result<()> {
        // Store as an action record
        let action = ActionRecord {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            action_type: "store".to_string(),
            tool_name: "history_tracker".to_string(),
            input: serde_json::json!({ "key": key }),
            output: Some(value),
            duration_ms: 0,
            success: true,
            error: None,
        };
        
        self.record_action(action).await
    }

    async fn retrieve(&self, key: &str) -> Result<Option<Value>> {
        // Search for the most recent store action with this key
        let query = SearchQuery {
            tool_name: Some("history_tracker".to_string()),
            action_type: Some("store".to_string()),
            success: Some(true),
            time_range: None,
            limit: Some(1),
        };
        
        let results = self.search_history(query).await?;
        
        for action in results {
            if let Some(input) = action.input.as_object() {
                if let Some(stored_key) = input.get("key").and_then(|v| v.as_str()) {
                    if stored_key == key {
                        return Ok(action.output);
                    }
                }
            }
        }
        
        Ok(None)
    }

    async fn clear(&self) -> Result<()> {
        self.clear_old_history(None).await?;
        Ok(())
    }

    async fn stats(&self) -> Result<MemoryStats> {
        let storage = self.storage.read().await;
        
        let total_entries = storage.actions.len();
        let memory_bytes = std::mem::size_of::<ActionRecord>() * total_entries;
        
        let last_accessed = storage.actions
            .back()
            .map(|a| a.timestamp);
        
        let successful = storage.actions.iter().filter(|a| a.success).count();
        let hit_rate = if total_entries > 0 {
            (successful as f32 / total_entries as f32) * 100.0
        } else {
            0.0
        };
        
        Ok(MemoryStats {
            total_entries,
            memory_bytes,
            last_accessed,
            hit_rate,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_history_tracker_operations() {
        // Test will be implemented with mock browser
    }
}