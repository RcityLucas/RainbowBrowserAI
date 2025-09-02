//! Simple Memory System for RainbowBrowserAI
//! 
//! A lightweight memory system using file-based storage for persistent learning
//! across sessions. This provides the foundation for adaptive intelligence.

use anyhow::Result;
use chrono::{DateTime, Utc};
use lru::LruCache;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::num::NonZeroUsize;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use tracing::{info, warn, error};
use uuid::Uuid;

use crate::intelligence::core::llm_service::llm_service_enhanced::TaskType;

/// Simple memory configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleMemoryConfig {
    pub data_dir: PathBuf,
    pub cache_size: usize,
    pub pattern_retention_days: u32,
    pub min_pattern_usage: u32,
    pub learning_rate: f32,
}

impl Default for SimpleMemoryConfig {
    fn default() -> Self {
        Self {
            data_dir: PathBuf::from("data/memory"),
            cache_size: 500,
            pattern_retention_days: 90,
            min_pattern_usage: 2,
            learning_rate: 0.1,
        }
    }
}

/// Represents an interaction outcome for learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionRecord {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub user_input: String,
    pub classified_task: TaskType,
    pub confidence: f32,
    pub execution_success: bool,
    pub execution_time_ms: u64,
    pub context_markers: Vec<String>,
}

/// Learned pattern with adaptive weights
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnedPattern {
    pub id: Uuid,
    pub pattern_text: String,
    pub associated_task: TaskType,
    pub weight: f32,
    pub success_rate: f32,
    pub usage_count: u32,
    pub last_used: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub context_tags: Vec<String>,
}

/// Simple Memory System for persistent learning
pub struct SimpleMemory {
    config: SimpleMemoryConfig,
    patterns_cache: Arc<RwLock<LruCache<String, LearnedPattern>>>,
    interactions_cache: Arc<RwLock<LruCache<Uuid, InteractionRecord>>>,
    patterns_file: PathBuf,
    interactions_file: PathBuf,
}

impl SimpleMemory {
    /// Initialize the simple memory system
    pub async fn new(config: SimpleMemoryConfig) -> Result<Self> {
        // Create data directory if it doesn't exist
        fs::create_dir_all(&config.data_dir)?;

        let patterns_file = config.data_dir.join("patterns.json");
        let interactions_file = config.data_dir.join("interactions.json");

        // Initialize caches
        let cache_size = NonZeroUsize::new(config.cache_size).unwrap_or(NonZeroUsize::new(500).unwrap());
        let patterns_cache = Arc::new(RwLock::new(LruCache::new(cache_size)));
        let interactions_cache = Arc::new(RwLock::new(LruCache::new(cache_size)));

        let memory = Self {
            config,
            patterns_cache,
            interactions_cache,
            patterns_file,
            interactions_file,
        };

        // Load existing data
        memory.load_patterns_from_file().await?;
        memory.load_interactions_from_file().await?;
        
        info!("🧠 Simple Memory System initialized with {} cache size", memory.config.cache_size);
        Ok(memory)
    }

    /// Record a new interaction for learning
    pub async fn record_interaction(&self, interaction: InteractionRecord) -> Result<()> {
        // Update cache
        {
            let mut cache = self.interactions_cache.write().unwrap();
            cache.put(interaction.id, interaction.clone());
        }

        // Learn from this interaction
        self.learn_from_interaction(&interaction).await?;

        // Persist to file (background operation)
        if let Err(e) = self.save_interactions_to_file().await {
            warn!("Failed to save interactions to file: {}", e);
        }

        info!("🧠 Recorded interaction: {} -> {:?} (confidence: {:.2})", 
              interaction.user_input, interaction.classified_task, interaction.confidence);

        Ok(())
    }

    /// Learn and adapt from interaction outcome
    async fn learn_from_interaction(&self, interaction: &InteractionRecord) -> Result<()> {
        // Extract patterns from the user input
        let extracted_patterns = self.extract_patterns_from_input(&interaction.user_input);

        for pattern_text in extracted_patterns {
            // Update or create learned pattern
            if let Some(mut pattern) = self.get_pattern(&pattern_text).await? {
                // Update existing pattern
                pattern.usage_count += 1;
                pattern.last_used = interaction.timestamp;

                // Update success rate using exponential moving average
                let outcome_score = if interaction.execution_success { 1.0 } else { 0.0 };
                pattern.success_rate = pattern.success_rate * (1.0 - self.config.learning_rate) + 
                                     outcome_score * self.config.learning_rate;

                // Adjust weight based on success
                let old_weight = pattern.weight;
                if interaction.execution_success && interaction.confidence > 0.7 {
                    pattern.weight = (pattern.weight * 1.05).min(2.0);
                } else if !interaction.execution_success {
                    pattern.weight = (pattern.weight * 0.95).max(0.1);
                }

                let new_weight = pattern.weight;
                let success_rate = pattern.success_rate;
                self.update_pattern(pattern).await?;
                
                info!("📈 Updated pattern '{}': weight {:.2} -> {:.2}, success rate: {:.2}", 
                      pattern_text, old_weight, new_weight, success_rate);
            } else {
                // Create new pattern
                let new_pattern = LearnedPattern {
                    id: Uuid::new_v4(),
                    pattern_text: pattern_text.clone(),
                    associated_task: interaction.classified_task,
                    weight: 0.5, // Start with neutral weight
                    success_rate: if interaction.execution_success { 1.0 } else { 0.0 },
                    usage_count: 1,
                    last_used: interaction.timestamp,
                    created_at: interaction.timestamp,
                    context_tags: interaction.context_markers.clone(),
                };

                self.create_pattern(new_pattern).await?;
                info!("🌱 Created new pattern: '{}' -> {:?}", pattern_text, interaction.classified_task);
            }
        }

        Ok(())
    }

    /// Extract patterns from user input using multiple strategies
    fn extract_patterns_from_input(&self, input: &str) -> Vec<String> {
        let mut patterns = Vec::new();
        let input_lower = input.to_lowercase();

        // Extract full phrases (2-4 words)
        let words: Vec<&str> = input_lower.split_whitespace().collect();
        
        // Extract bigrams and trigrams
        for window_size in 2..=3 {
            for window in words.windows(window_size) {
                if window.len() == window_size {
                    patterns.push(window.join(" "));
                }
            }
        }

        // Extract significant single words (not stop words)
        let stop_words = ["the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for", "of", "with", "by", "me", "my", "i"];
        for word in &words {
            if word.len() > 3 && !stop_words.contains(word) {
                patterns.push(word.to_string());
            }
        }

        patterns.into_iter().take(8).collect() // Limit to prevent explosion
    }

    /// Retrieve pattern from cache or file
    async fn get_pattern(&self, pattern_text: &str) -> Result<Option<LearnedPattern>> {
        // Check cache first
        {
            let cache = self.patterns_cache.read().unwrap();
            if let Some(pattern) = cache.peek(pattern_text) {
                return Ok(Some(pattern.clone()));
            }
        }

        // Pattern not in cache, would need to be loaded from file
        // For simplicity, return None here
        Ok(None)
    }

    /// Create new learned pattern
    async fn create_pattern(&self, pattern: LearnedPattern) -> Result<()> {
        // Update cache
        {
            let mut cache = self.patterns_cache.write().unwrap();
            cache.put(pattern.pattern_text.clone(), pattern.clone());
        }

        // Save to file (background operation)
        if let Err(e) = self.save_patterns_to_file().await {
            warn!("Failed to save patterns to file: {}", e);
        }

        Ok(())
    }

    /// Update existing learned pattern
    async fn update_pattern(&self, pattern: LearnedPattern) -> Result<()> {
        // Update cache
        {
            let mut cache = self.patterns_cache.write().unwrap();
            cache.put(pattern.pattern_text.clone(), pattern.clone());
        }

        // Save to file (background operation)
        if let Err(e) = self.save_patterns_to_file().await {
            warn!("Failed to save patterns to file: {}", e);
        }

        Ok(())
    }

    /// Get enhanced patterns for classification with memory-based weights
    pub async fn get_enhanced_patterns_for_task(&self, task_type: TaskType) -> Result<Vec<LearnedPattern>> {
        let patterns: Vec<LearnedPattern> = {
            let cache = self.patterns_cache.read().unwrap();
            cache
                .iter()
                .filter(|(_, pattern)| pattern.associated_task == task_type)
                .map(|(_, pattern)| pattern.clone())
                .collect()
        };

        // Sort by weight and success rate
        let mut sorted_patterns = patterns;
        sorted_patterns.sort_by(|a, b| {
            let score_a = a.weight * a.success_rate;
            let score_b = b.weight * b.success_rate;
            score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(sorted_patterns.into_iter().take(20).collect())
    }

    /// Get memory-enhanced classification for input
    pub async fn enhance_classification(&self, input: &str, base_confidence: f32, task_type: TaskType) -> Result<(f32, Vec<String>)> {
        let input_patterns = self.extract_patterns_from_input(input);
        let mut confidence_boost = 0.0;
        let mut evidence = Vec::new();

        // Get relevant patterns for this task type
        let task_patterns = self.get_enhanced_patterns_for_task(task_type).await?;

        // Calculate confidence boost based on matching patterns
        for input_pattern in &input_patterns {
            for task_pattern in &task_patterns {
                if task_pattern.pattern_text.contains(input_pattern) || input_pattern.contains(&task_pattern.pattern_text) {
                    let pattern_boost = task_pattern.weight * task_pattern.success_rate * 0.05;
                    confidence_boost += pattern_boost;
                    evidence.push(format!("Pattern '{}' matches (weight: {:.2}, success: {:.2})", 
                                         task_pattern.pattern_text, task_pattern.weight, task_pattern.success_rate));
                }
            }
        }

        // Clamp confidence boost
        confidence_boost = confidence_boost.clamp(-0.2, 0.3);
        let enhanced_confidence = (base_confidence + confidence_boost).clamp(0.0, 1.0);

        Ok((enhanced_confidence, evidence))
    }

    /// Load patterns from file
    async fn load_patterns_from_file(&self) -> Result<()> {
        if !self.patterns_file.exists() {
            return Ok(()); // No file to load
        }

        let content = fs::read_to_string(&self.patterns_file)?;
        let patterns: Vec<LearnedPattern> = serde_json::from_str(&content).unwrap_or_default();

        {
            let mut cache = self.patterns_cache.write().unwrap();
            for pattern in patterns {
                cache.put(pattern.pattern_text.clone(), pattern);
            }
        }

        info!("📚 Loaded {} patterns from file", 
              self.patterns_cache.read().unwrap().len());
        Ok(())
    }

    /// Save patterns to file
    async fn save_patterns_to_file(&self) -> Result<()> {
        let patterns: Vec<LearnedPattern> = {
            let cache = self.patterns_cache.read().unwrap();
            cache.iter().map(|(_, pattern)| pattern.clone()).collect()
        };

        let content = serde_json::to_string_pretty(&patterns)?;
        fs::write(&self.patterns_file, content)?;

        Ok(())
    }

    /// Load interactions from file
    async fn load_interactions_from_file(&self) -> Result<()> {
        if !self.interactions_file.exists() {
            return Ok(()); // No file to load
        }

        let content = fs::read_to_string(&self.interactions_file)?;
        let interactions: Vec<InteractionRecord> = serde_json::from_str(&content).unwrap_or_default();

        {
            let mut cache = self.interactions_cache.write().unwrap();
            for interaction in interactions {
                cache.put(interaction.id, interaction);
            }
        }

        info!("📚 Loaded {} interactions from file", 
              self.interactions_cache.read().unwrap().len());
        Ok(())
    }

    /// Save interactions to file
    async fn save_interactions_to_file(&self) -> Result<()> {
        let interactions: Vec<InteractionRecord> = {
            let cache = self.interactions_cache.read().unwrap();
            cache.iter().map(|(_, interaction)| interaction.clone()).collect()
        };

        let content = serde_json::to_string_pretty(&interactions)?;
        fs::write(&self.interactions_file, content)?;

        Ok(())
    }

    /// Clean up old patterns and interactions
    pub async fn cleanup_old_memories(&self) -> Result<()> {
        let cutoff_date = Utc::now() - chrono::Duration::days(self.config.pattern_retention_days as i64);

        // Clean patterns
        {
            let mut cache = self.patterns_cache.write().unwrap();
            let keys_to_remove: Vec<String> = cache
                .iter()
                .filter(|(_, pattern)| pattern.usage_count < self.config.min_pattern_usage && pattern.last_used < cutoff_date)
                .map(|(key, _)| key.clone())
                .collect();

            for key in keys_to_remove {
                cache.pop(&key);
            }
        }

        // Clean interactions
        {
            let mut cache = self.interactions_cache.write().unwrap();
            let keys_to_remove: Vec<Uuid> = cache
                .iter()
                .filter(|(_, interaction)| interaction.timestamp < cutoff_date)
                .map(|(key, _)| *key)
                .collect();

            for key in keys_to_remove {
                cache.pop(&key);
            }
        }

        // Save cleaned data
        self.save_patterns_to_file().await?;
        self.save_interactions_to_file().await?;

        info!("🧹 Cleaned up old memories (cutoff: {})", cutoff_date.format("%Y-%m-%d"));
        Ok(())
    }

    /// Get memory statistics
    pub async fn get_memory_stats(&self) -> SimpleMemoryStats {
        let patterns_count = self.patterns_cache.read().unwrap().len();
        let interactions_count = self.interactions_cache.read().unwrap().len();

        let avg_success_rate = {
            let cache = self.patterns_cache.read().unwrap();
            let rates: Vec<f32> = cache.iter().map(|(_, pattern)| pattern.success_rate).collect();
            if rates.is_empty() {
                0.0
            } else {
                rates.iter().sum::<f32>() / rates.len() as f32
            }
        };

        SimpleMemoryStats {
            total_interactions: interactions_count,
            total_patterns: patterns_count,
            average_success_rate: avg_success_rate,
        }
    }
}

/// Memory system statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleMemoryStats {
    pub total_interactions: usize,
    pub total_patterns: usize,
    pub average_success_rate: f32,
}

/// Create simple memory system from environment configuration
pub async fn create_simple_memory() -> Result<SimpleMemory> {
    let config = SimpleMemoryConfig::default();
    SimpleMemory::new(config).await
}