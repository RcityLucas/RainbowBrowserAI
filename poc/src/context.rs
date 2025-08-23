use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use chrono::{DateTime, Utc};
use crate::ParsedCommand;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationContext {
    pub conversation_id: String,
    pub created_at: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
    pub command_history: Vec<HistoryEntry>,
    pub preferences: UserPreferences,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub timestamp: DateTime<Utc>,
    pub user_input: String,
    pub parsed_command: ParsedCommand,
    pub execution_result: ExecutionResult,
    pub cost: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub success: bool,
    pub duration_ms: u64,
    pub error_message: Option<String>,
    pub output_summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    pub default_screenshot: bool,
    pub preferred_viewport: (u32, u32),
    pub favorite_sites: Vec<String>,
    pub default_retries: u32,
    pub default_timeout: u64,
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            default_screenshot: false,
            preferred_viewport: (1920, 1080),
            favorite_sites: Vec::new(),
            default_retries: 3,
            default_timeout: 30,
        }
    }
}

impl ConversationContext {
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            conversation_id: format!("conv_{}", now.format("%Y%m%d_%H%M%S")),
            created_at: now,
            last_updated: now,
            command_history: Vec::new(),
            preferences: UserPreferences::default(),
        }
    }

    pub fn load_from_file() -> Result<Self> {
        match fs::read_to_string("conversation_context.json") {
            Ok(data) => {
                let context: ConversationContext = serde_json::from_str(&data)?;
                Ok(context)
            }
            Err(_) => {
                // File doesn't exist, create new context
                Ok(Self::new())
            }
        }
    }

    pub fn save_to_file(&self) -> Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write("conversation_context.json", json)?;
        Ok(())
    }

    pub fn add_history_entry(&mut self, entry: HistoryEntry) -> Result<()> {
        self.command_history.push(entry);
        self.last_updated = Utc::now();
        
        // Keep only last 50 entries to prevent file from growing too large
        if self.command_history.len() > 50 {
            self.command_history.drain(0..self.command_history.len() - 50);
        }
        
        // Update preferences based on usage patterns
        self.update_preferences_from_history();
        
        self.save_to_file()
    }

    fn update_preferences_from_history(&mut self) {
        if self.command_history.len() < 3 {
            return; // Need some history to infer preferences
        }

        let recent_entries = &self.command_history[self.command_history.len().saturating_sub(10)..];
        
        // Update default screenshot preference based on recent usage
        let screenshot_usage = recent_entries
            .iter()
            .filter(|entry| entry.parsed_command.screenshot)
            .count();
        
        if screenshot_usage > recent_entries.len() / 2 {
            self.preferences.default_screenshot = true;
        }

        // Track frequently visited sites
        for entry in recent_entries {
            if let Some(ref url) = entry.parsed_command.url {
                if !self.preferences.favorite_sites.contains(url) {
                    self.preferences.favorite_sites.push(url.clone());
                }
            }
            for url in &entry.parsed_command.urls {
                if !self.preferences.favorite_sites.contains(url) {
                    self.preferences.favorite_sites.push(url.clone());
                }
            }
        }

        // Keep only top 10 favorite sites
        if self.preferences.favorite_sites.len() > 10 {
            self.preferences.favorite_sites.truncate(10);
        }
    }

    pub fn get_context_for_llm(&self) -> String {
        let mut context = String::new();
        
        // Add recent command history
        if !self.command_history.is_empty() {
            context.push_str("Recent commands:\n");
            let recent = &self.command_history[self.command_history.len().saturating_sub(5)..];
            for entry in recent {
                context.push_str(&format!(
                    "- \"{}\": {} ({})\n",
                    entry.user_input,
                    entry.execution_result.output_summary,
                    if entry.execution_result.success { "success" } else { "failed" }
                ));
            }
            context.push('\n');
        }

        // Add user preferences
        context.push_str(&format!(
            "User preferences:\n\
             - Default screenshot: {}\n\
             - Preferred viewport: {}x{}\n\
             - Default retries: {}\n\
             - Default timeout: {}s\n",
            self.preferences.default_screenshot,
            self.preferences.preferred_viewport.0,
            self.preferences.preferred_viewport.1,
            self.preferences.default_retries,
            self.preferences.default_timeout
        ));

        if !self.preferences.favorite_sites.is_empty() {
            context.push_str(&format!(
                "- Favorite sites: {}\n",
                self.preferences.favorite_sites.join(", ")
            ));
        }

        context
    }

    pub fn enhance_command_with_context(&self, command: &mut ParsedCommand) {
        // Apply user preferences to fill in missing values
        if command.viewport_width.is_none() {
            command.viewport_width = Some(self.preferences.preferred_viewport.0);
        }
        
        if command.viewport_height.is_none() {
            command.viewport_height = Some(self.preferences.preferred_viewport.1);
        }
        
        if command.retries.is_none() {
            command.retries = Some(self.preferences.default_retries);
        }
        
        if command.timeout.is_none() {
            command.timeout = Some(self.preferences.default_timeout);
        }

        // If no explicit screenshot preference, use user default
        if !command.screenshot && self.preferences.default_screenshot {
            // Only auto-enable screenshots for navigate commands, not test commands
            if command.action == "navigate" {
                command.screenshot = true;
            }
        }
    }

    pub fn get_similar_commands(&self, current_command: &str) -> Vec<&HistoryEntry> {
        self.command_history
            .iter()
            .filter(|entry| {
                // Simple similarity check - contains similar words
                let current_lower = current_command.to_lowercase();
                let current_words: Vec<&str> = current_lower.split_whitespace().collect();
                let entry_lower = entry.user_input.to_lowercase();
                let entry_words: Vec<&str> = entry_lower.split_whitespace().collect();
                
                let common_words = current_words.iter()
                    .filter(|word| entry_words.contains(word))
                    .count();
                
                common_words > 0 && common_words as f32 / current_words.len() as f32 > 0.3
            })
            .take(3) // Return top 3 similar commands
            .collect()
    }

    pub fn get_stats(&self) -> ContextStats {
        let total_commands = self.command_history.len();
        let successful_commands = self.command_history
            .iter()
            .filter(|entry| entry.execution_result.success)
            .count();
        
        let total_cost: f64 = self.command_history
            .iter()
            .map(|entry| entry.cost)
            .sum();

        let avg_duration = if total_commands > 0 {
            self.command_history
                .iter()
                .map(|entry| entry.execution_result.duration_ms)
                .sum::<u64>() / total_commands as u64
        } else {
            0
        };

        ContextStats {
            total_commands,
            successful_commands,
            success_rate: if total_commands > 0 {
                successful_commands as f32 / total_commands as f32
            } else {
                0.0
            },
            total_cost,
            average_duration_ms: avg_duration,
            favorite_sites: self.preferences.favorite_sites.clone(),
        }
    }
}

#[derive(Debug)]
pub struct ContextStats {
    pub total_commands: usize,
    pub successful_commands: usize,
    pub success_rate: f32,
    pub total_cost: f64,
    pub average_duration_ms: u64,
    pub favorite_sites: Vec<String>,
}