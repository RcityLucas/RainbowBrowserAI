// Strategy pattern for perception modes - removes hard-coded enum dispatch
use async_trait::async_trait;
use anyhow::Result;
use std::sync::Arc;
use std::collections::HashMap;

use super::{
    PerceptionMode, PerceptionResult, PerceptionData,
    LightningPerception, QuickPerception, StandardPerception, DeepPerception,
    LightningData, QuickData, StandardData, DeepData,
};

/// Perception strategy trait
#[async_trait]
pub trait PerceptionStrategy: Send + Sync {
    async fn perceive(&self, url: &str) -> Result<PerceptionData>;
    fn max_duration_ms(&self) -> u64;
    fn mode(&self) -> PerceptionMode;
}

/// Lightning perception strategy
pub struct LightningStrategy {
    perception: Arc<LightningPerception>,
}

impl LightningStrategy {
    pub fn new(perception: Arc<LightningPerception>) -> Self {
        Self { perception }
    }
}

#[async_trait]
impl PerceptionStrategy for LightningStrategy {
    async fn perceive(&self, url: &str) -> Result<PerceptionData> {
        let result = self.perception.perceive(url).await?;
        Ok(PerceptionData::Lightning(result))
    }

    fn max_duration_ms(&self) -> u64 {
        50
    }

    fn mode(&self) -> PerceptionMode {
        PerceptionMode::Lightning
    }
}

/// Quick perception strategy
pub struct QuickStrategy {
    perception: Arc<QuickPerception>,
}

impl QuickStrategy {
    pub fn new(perception: Arc<QuickPerception>) -> Self {
        Self { perception }
    }
}

#[async_trait]
impl PerceptionStrategy for QuickStrategy {
    async fn perceive(&self, url: &str) -> Result<PerceptionData> {
        let result = self.perception.perceive(url).await?;
        Ok(PerceptionData::Quick(result))
    }

    fn max_duration_ms(&self) -> u64 {
        200
    }

    fn mode(&self) -> PerceptionMode {
        PerceptionMode::Quick
    }
}

/// Standard perception strategy
pub struct StandardStrategy {
    perception: Arc<StandardPerception>,
}

impl StandardStrategy {
    pub fn new(perception: Arc<StandardPerception>) -> Self {
        Self { perception }
    }
}

#[async_trait]
impl PerceptionStrategy for StandardStrategy {
    async fn perceive(&self, url: &str) -> Result<PerceptionData> {
        let result = self.perception.perceive(url).await?;
        Ok(PerceptionData::Standard(result))
    }

    fn max_duration_ms(&self) -> u64 {
        500
    }

    fn mode(&self) -> PerceptionMode {
        PerceptionMode::Standard
    }
}

/// Deep perception strategy
pub struct DeepStrategy {
    perception: Arc<DeepPerception>,
}

impl DeepStrategy {
    pub fn new(perception: Arc<DeepPerception>) -> Self {
        Self { perception }
    }
}

#[async_trait]
impl PerceptionStrategy for DeepStrategy {
    async fn perceive(&self, url: &str) -> Result<PerceptionData> {
        let result = self.perception.perceive(url).await?;
        Ok(PerceptionData::Deep(result))
    }

    fn max_duration_ms(&self) -> u64 {
        1000
    }

    fn mode(&self) -> PerceptionMode {
        PerceptionMode::Deep
    }
}

/// Strategy factory for creating perception strategies
pub struct PerceptionStrategyFactory {
    strategies: HashMap<PerceptionMode, Arc<dyn PerceptionStrategy>>,
}

impl PerceptionStrategyFactory {
    pub async fn new() -> Result<Self> {
        let lightning = Arc::new(LightningPerception::new().await?);
        let quick = Arc::new(QuickPerception::new().await?);
        let standard = Arc::new(StandardPerception::new().await?);
        let deep = Arc::new(DeepPerception::new().await?);

        let mut strategies: HashMap<PerceptionMode, Arc<dyn PerceptionStrategy>> = HashMap::new();
        
        strategies.insert(
            PerceptionMode::Lightning,
            Arc::new(LightningStrategy::new(lightning)),
        );
        strategies.insert(
            PerceptionMode::Quick,
            Arc::new(QuickStrategy::new(quick)),
        );
        strategies.insert(
            PerceptionMode::Standard,
            Arc::new(StandardStrategy::new(standard)),
        );
        strategies.insert(
            PerceptionMode::Deep,
            Arc::new(DeepStrategy::new(deep)),
        );

        Ok(Self { strategies })
    }

    pub fn get_strategy(&self, mode: PerceptionMode) -> Option<Arc<dyn PerceptionStrategy>> {
        self.strategies.get(&mode).cloned()
    }

    /// Register a custom strategy (extensibility point)
    pub fn register_strategy(&mut self, mode: PerceptionMode, strategy: Arc<dyn PerceptionStrategy>) {
        self.strategies.insert(mode, strategy);
    }
}