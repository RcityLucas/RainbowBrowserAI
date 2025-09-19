// Unified State Management System
// Provides centralized state management with event-driven updates

use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tracing::info;

use super::cache::CacheCoordinator;
use super::events::{Event, EventBus};
use crate::perception::{ElementType, PageType, PerceivedElement};

/// Unified state manager for all modules
pub struct UnifiedStateManager {
    event_bus: Arc<EventBus>,
    browser_state: Arc<RwLock<BrowserState>>,
    perception_state: Arc<RwLock<PerceptionState>>,
    tool_state: Arc<RwLock<ToolState>>,
    intelligence_state: Arc<RwLock<IntelligenceState>>,
    cache_coordinator: Arc<CacheCoordinator>,
    state_version: Arc<RwLock<u64>>,
}

/// Browser state information
#[derive(Debug, Clone)]
pub struct BrowserState {
    pub current_url: String,
    pub page_title: String,
    pub page_load_state: PageLoadState,
    pub viewport: ViewportInfo,
    pub navigation_history: Vec<NavigationEvent>,
    pub cookies: HashMap<String, String>,
    pub local_storage: HashMap<String, String>,
    pub last_screenshot: Option<ScreenshotMetadata>,
    pub last_updated: Instant,
}

#[derive(Debug, Clone)]
pub enum PageLoadState {
    Loading,
    DomContentLoaded,
    Complete,
    Failed(String),
}

#[derive(Debug, Clone)]
pub struct ViewportInfo {
    pub width: u32,
    pub height: u32,
    pub device_scale_factor: f64,
    pub is_mobile: bool,
}

#[derive(Debug, Clone)]
pub struct NavigationEvent {
    pub url: String,
    pub timestamp: Instant,
    pub load_time_ms: Option<u64>,
    pub status_code: Option<u16>,
}

#[derive(Debug, Clone)]
pub struct ScreenshotMetadata {
    pub timestamp: Instant,
    pub size_bytes: usize,
    pub format: String,
    pub full_page: bool,
}

/// Perception module state
#[derive(Debug, Clone, Default)]
pub struct PerceptionState {
    pub current_analysis: Option<PageAnalysis>,
    pub element_cache: ElementCache,
    pub page_classification: Option<PageClassification>,
    pub context_stack: Vec<PerceptionContext>,
    pub confidence_scores: HashMap<String, f64>,
    pub last_analysis_time: Option<Instant>,
}

#[derive(Debug, Clone)]
pub struct PageAnalysis {
    pub timestamp: Instant,
    pub analysis_type: String,
    pub duration_ms: u64,
    pub elements_found: usize,
    pub interactive_elements: Vec<InteractiveElement>,
    pub forms: Vec<FormInfo>,
    pub semantic_structure: SemanticStructure,
}

#[derive(Debug, Clone)]
pub struct InteractiveElement {
    pub selector: String,
    pub element_type: ElementType,
    pub text: String,
    pub is_visible: bool,
    pub is_enabled: bool,
}

#[derive(Debug, Clone)]
pub struct FormInfo {
    pub selector: String,
    pub fields: Vec<FormField>,
    pub action: Option<String>,
    pub method: String,
}

#[derive(Debug, Clone)]
pub struct FormField {
    pub name: String,
    pub field_type: String,
    pub value: Option<String>,
    pub required: bool,
}

#[derive(Debug, Clone)]
pub struct SemanticStructure {
    pub headings: Vec<String>,
    pub main_content: Option<String>,
    pub navigation: Vec<String>,
    pub footer: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PageClassification {
    pub page_type: PageType,
    pub confidence: f64,
    pub features: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ElementCache {
    _elements: HashMap<String, CachedElement>,
    _last_cleanup: Instant,
}

impl Default for ElementCache {
    fn default() -> Self {
        Self {
            _elements: HashMap::new(),
            _last_cleanup: Instant::now(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CachedElement {
    pub element: PerceivedElement,
    pub cached_at: Instant,
    pub access_count: usize,
}

#[derive(Debug, Clone)]
pub struct PerceptionContext {
    pub context_id: String,
    pub created_at: Instant,
    pub last_action: Option<String>,
    pub named_elements: HashMap<String, String>,
    pub form_state: HashMap<String, String>,
}

/// Tool execution state
#[derive(Debug, Clone, Default)]
pub struct ToolState {
    pub execution_history: Vec<ToolExecution>,
    pub active_workflows: HashMap<String, WorkflowState>,
    pub dependency_graph: DependencyGraph,
    pub performance_metrics: ToolPerformanceMetrics,
    pub tool_cache_stats: HashMap<String, CacheStats>,
}

#[derive(Debug, Clone, Default)]
pub struct IntelligenceState {
    pub decisions_made: u64,
    pub successful_decisions: u64,
    pub learning_rate: f64,
    pub last_decision_time: Option<Instant>,
}

#[derive(Debug, Clone)]
pub struct ToolExecution {
    pub execution_id: String,
    pub tool_name: String,
    pub started_at: Instant,
    pub completed_at: Option<Instant>,
    pub success: bool,
    pub error: Option<String>,
    pub input_size: usize,
    pub output_size: usize,
}

#[derive(Debug, Clone)]
pub struct WorkflowState {
    pub workflow_id: String,
    pub name: String,
    pub status: WorkflowStatus,
    pub current_step: usize,
    pub total_steps: usize,
    pub started_at: Instant,
    pub updated_at: Instant,
}

#[derive(Debug, Clone)]
pub enum WorkflowStatus {
    Running,
    Paused,
    Completed,
    Failed(String),
    Cancelled,
}

#[derive(Debug, Clone, Default)]
pub struct DependencyGraph {
    pub nodes: HashMap<String, DependencyNode>,
    pub edges: Vec<DependencyEdge>,
}

#[derive(Debug, Clone)]
pub struct DependencyNode {
    pub tool_name: String,
    pub category: String,
}

#[derive(Debug, Clone)]
pub struct DependencyEdge {
    pub from: String,
    pub to: String,
    pub dependency_type: String,
}

#[derive(Debug, Clone, Default)]
pub struct ToolPerformanceMetrics {
    pub total_executions: u64,
    pub successful_executions: u64,
    pub failed_executions: u64,
    pub average_duration_ms: f64,
    pub total_duration_ms: u64,
}

#[derive(Debug, Clone)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub entries: usize,
    pub size_bytes: usize,
}

impl UnifiedStateManager {
    pub async fn new(event_bus: Arc<EventBus>) -> Result<Self> {
        let cache_coordinator = Arc::new(CacheCoordinator::new(event_bus.clone()).await);

        let manager = Self {
            event_bus: event_bus.clone(),
            browser_state: Arc::new(RwLock::new(BrowserState::default())),
            perception_state: Arc::new(RwLock::new(PerceptionState::default())),
            tool_state: Arc::new(RwLock::new(ToolState::default())),
            intelligence_state: Arc::new(RwLock::new(IntelligenceState::default())),
            cache_coordinator,
            state_version: Arc::new(RwLock::new(0)),
        };

        // Subscribe to events that affect state
        manager.setup_event_subscriptions().await;

        Ok(manager)
    }

    async fn setup_event_subscriptions(&self) {
        // TODO: Implement proper event subscriptions
        // The current EventHandler trait requires async handlers which
        // are complex to implement with closures. We need to create
        // proper handler structs that implement the trait.

        // For now, state updates will be handled directly by the
        // modules that emit the events
    }

    /// Update browser state with a function
    pub async fn update_browser_state<F>(&self, updater: F) -> Result<()>
    where
        F: FnOnce(&mut BrowserState) -> Result<()>,
    {
        let mut state = self.browser_state.write().await;
        updater(&mut state)?;
        state.last_updated = Instant::now();

        // Increment version
        let mut version = self.state_version.write().await;
        *version += 1;

        // Emit state change event
        self.event_bus
            .emit(Event::PageContentChanged {
                session_id: String::new(), // Will be filled by session context
                change_type: super::events::ContentChangeType::Unknown,
                timestamp: Instant::now(),
            })
            .await?;

        Ok(())
    }

    /// Update perception state
    pub async fn update_perception_state<F>(&self, updater: F) -> Result<()>
    where
        F: FnOnce(&mut PerceptionState) -> Result<()>,
    {
        let mut state = self.perception_state.write().await;
        updater(&mut state)?;
        state.last_analysis_time = Some(Instant::now());

        // Increment version
        let mut version = self.state_version.write().await;
        *version += 1;

        Ok(())
    }

    /// Update tool state
    pub async fn update_tool_state<F>(&self, updater: F) -> Result<()>
    where
        F: FnOnce(&mut ToolState) -> Result<()>,
    {
        let mut state = self.tool_state.write().await;
        updater(&mut state)?;

        // Increment version
        let mut version = self.state_version.write().await;
        *version += 1;

        Ok(())
    }

    pub async fn update_intelligence_state<F>(&self, updater: F) -> Result<()>
    where
        F: FnOnce(&mut IntelligenceState) -> Result<()>,
    {
        let mut state = self.intelligence_state.write().await;
        updater(&mut state)?;

        // Increment version
        let mut version = self.state_version.write().await;
        *version += 1;

        Ok(())
    }

    /// Get current browser state (read-only)
    pub async fn get_browser_state(&self) -> BrowserState {
        self.browser_state.read().await.clone()
    }

    /// Get current perception state (read-only)
    pub async fn get_perception_state(&self) -> PerceptionState {
        self.perception_state.read().await.clone()
    }

    /// Get current tool state (read-only)
    pub async fn get_tool_state(&self) -> ToolState {
        self.tool_state.read().await.clone()
    }

    /// Get current state version
    pub async fn get_state_version(&self) -> u64 {
        *self.state_version.read().await
    }

    /// Invalidate caches for navigation
    pub async fn invalidate_caches_for_navigation(&self, new_url: &str) -> Result<()> {
        info!("Invalidating caches for navigation to: {}", new_url);

        // Clear perception element cache
        self.update_perception_state(|state| {
            state.element_cache = ElementCache::default();
            state.current_analysis = None;
            state.page_classification = None;
            Ok(())
        })
        .await?;

        // Notify cache coordinator
        self.cache_coordinator
            .invalidate_navigation_sensitive(new_url)
            .await?;

        // Emit cache invalidation event
        self.event_bus
            .emit(Event::CacheInvalidated {
                cache_type: "navigation".to_string(),
                reason: "page_navigation".to_string(),
                keys_affected: vec![],
                timestamp: Instant::now(),
            })
            .await?;

        Ok(())
    }

    /// Get a snapshot of all state
    pub async fn get_state_snapshot(&self) -> StateSnapshot {
        StateSnapshot {
            browser_state: self.browser_state.read().await.clone(),
            perception_state: self.perception_state.read().await.clone(),
            tool_state: self.tool_state.read().await.clone(),
            version: *self.state_version.read().await,
            timestamp: Instant::now(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct StateSnapshot {
    pub browser_state: BrowserState,
    pub perception_state: PerceptionState,
    pub tool_state: ToolState,
    pub version: u64,
    pub timestamp: Instant,
}

// Default implementations
impl Default for BrowserState {
    fn default() -> Self {
        Self {
            current_url: String::new(),
            page_title: String::new(),
            page_load_state: PageLoadState::Loading,
            viewport: ViewportInfo {
                width: 1920,
                height: 1080,
                device_scale_factor: 1.0,
                is_mobile: false,
            },
            navigation_history: Vec::new(),
            cookies: HashMap::new(),
            local_storage: HashMap::new(),
            last_screenshot: None,
            last_updated: Instant::now(),
        }
    }
}

// Default for PerceptionState and ToolState derived above
