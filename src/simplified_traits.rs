// Simplified trait definitions to reduce Arc and async complexity
use anyhow::Result;
use uuid::Uuid;

// For simple, synchronous operations that don't need async
pub mod sync_traits {
    use super::*;
    
    /// Simple synchronous perception for lightweight operations
    pub trait SyncPerception {
        fn quick_check(&self, url: &str) -> Result<bool>;
        fn get_page_title(&self, url: &str) -> Result<String>;
        fn count_elements(&self, url: &str, selector: &str) -> Result<usize>;
    }
    
    /// Simple synchronous validation
    pub trait Validator {
        fn validate_config(&self, config: &str) -> Result<()>;
        fn is_valid_url(&self, url: &str) -> bool;
        fn check_health(&self) -> bool;
    }
}

// Simplified async traits without excessive Arc usage
pub mod lightweight {
    use super::*;
    use async_trait::async_trait;
    
    /// Lightweight session manager without Arc overhead
    #[async_trait]
    pub trait LightweightSession {
        async fn start(&mut self) -> Result<Uuid>;
        async fn stop(&mut self, id: Uuid) -> Result<()>;
        async fn is_active(&self, id: Uuid) -> bool;
    }
    
    /// Simplified action executor
    #[async_trait]
    pub trait SimpleExecutor {
        async fn click(&self, selector: &str) -> Result<()>;
        async fn type_text(&self, selector: &str, text: &str) -> Result<()>;
        async fn navigate(&self, url: &str) -> Result<()>;
        async fn wait(&self, ms: u64) -> Result<()>;
    }
}

// Helper to reduce Arc usage - use references where possible
pub mod ref_based {
    use super::*;
    use crate::{
        layered_perception::PerceptionResult,
        intelligent_action::ActionResult,
    };
    
    /// Reference-based workflow to avoid Arc cloning
    pub struct RefWorkflow<'a> {
        perception: &'a dyn PerceptionRef,
        action: &'a dyn ActionRef,
        persistence: &'a dyn PersistenceRef,
    }
    
    pub trait PerceptionRef {
        fn perceive_ref(&self, url: &str) -> Result<PerceptionResult>;
    }
    
    pub trait ActionRef {
        fn execute_ref(&self, action: &str) -> Result<ActionResult>;
    }
    
    pub trait PersistenceRef {
        fn store_ref(&self, data: &[u8]) -> Result<()>;
    }
    
    impl<'a> RefWorkflow<'a> {
        pub fn new(
            perception: &'a dyn PerceptionRef,
            action: &'a dyn ActionRef,
            persistence: &'a dyn PersistenceRef,
        ) -> Self {
            Self { perception, action, persistence }
        }
        
        pub fn execute(&self, url: &str, action_cmd: &str) -> Result<()> {
            // No Arc cloning needed
            let perception_result = self.perception.perceive_ref(url)?;
            let action_result = self.action.execute_ref(action_cmd)?;
            
            // Store results
            let data = format!("{:?}|{:?}", perception_result, action_result);
            self.persistence.store_ref(data.as_bytes())?;
            
            Ok(())
        }
    }
}

// Simplified builder pattern without excessive generics
pub struct SimpleEngineBuilder {
    config: SimpleConfig,
}

#[derive(Default)]
pub struct SimpleConfig {
    pub enable_cache: bool,
    pub timeout_ms: u64,
    pub max_retries: u8,
}

impl SimpleEngineBuilder {
    pub fn new() -> Self {
        Self {
            config: SimpleConfig::default(),
        }
    }
    
    pub fn with_cache(mut self) -> Self {
        self.config.enable_cache = true;
        self
    }
    
    pub fn with_timeout(mut self, ms: u64) -> Self {
        self.config.timeout_ms = ms;
        self
    }
    
    pub fn with_retries(mut self, retries: u8) -> Self {
        self.config.max_retries = retries;
        self
    }
    
    pub fn build(self) -> SimpleEngine {
        SimpleEngine {
            config: self.config,
        }
    }
}

pub struct SimpleEngine {
    config: SimpleConfig,
}

impl SimpleEngine {
    /// Direct method calls without async overhead for simple operations
    pub fn get_config(&self) -> &SimpleConfig {
        &self.config
    }
    
    pub fn is_cache_enabled(&self) -> bool {
        self.config.enable_cache
    }
}

// Utility to convert between Arc-heavy and lightweight implementations
pub mod converters {
    use std::sync::Arc;
    use super::*;
    
    /// Convert Arc-based trait to reference-based
    pub fn arc_to_ref<T>(arc: Arc<T>) -> impl AsRef<T> {
        arc
    }
    
    /// Wrapper to use Arc types with reference-based traits
    pub struct ArcWrapper<T> {
        inner: Arc<T>,
    }
    
    impl<T> ArcWrapper<T> {
        pub fn new(inner: Arc<T>) -> Self {
            Self { inner }
        }
        
        pub fn as_ref(&self) -> &T {
            &self.inner
        }
    }
    
    impl<T> AsRef<T> for ArcWrapper<T> {
        fn as_ref(&self) -> &T {
            &self.inner
        }
    }
}