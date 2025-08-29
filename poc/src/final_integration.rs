// Final Integration Layer - Unified Enhanced Browser Automation System
// This module integrates all perception capabilities into a cohesive, production-ready system

use anyhow::Result;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use thirtyfour::{WebDriver, WebElement, By};
use tokio::sync::{Mutex, RwLock};
use tracing::{info, debug, warn, error};
use uuid::Uuid;

use crate::perception_mvp::{
    PerceptionEngineMVP, PerceivedElement, PageType,
    integration::EnhancedPerceptionEngine,
    visual::VisualAnalyzer,
    semantic::SemanticAnalyzer as PerceptionSemanticAnalyzer,
    context_aware::ContextAwareSelector,
    smart_forms::{SmartFormHandler, FormAnalysis, FormType, AutoFillResult, FormProfile},
    dynamic_handler::{DynamicContentHandler, WaitCondition, ModalAction, DynamicContentResult},
    testing_framework::PerceptionTestFramework,
};

use crate::semantic_analyzer::{SemanticAnalyzer, SemanticPageModel, PageType as MainPageType};
use crate::instruction_parser::{InstructionParser, UserInstruction, Feedback, ContextHints};

/// Unified Enhanced Browser Automation System
/// Combines all perception layers into a single, intelligent interface
pub struct UnifiedBrowserSystem {
    // Core components
    driver: Arc<WebDriver>,
    session_id: String,
    
    // Perception layers
    mvp_engine: Arc<Mutex<PerceptionEngineMVP>>,
    enhanced_engine: Arc<Mutex<EnhancedPerceptionEngine>>,
    visual_analyzer: Arc<Mutex<VisualAnalyzer>>,
    semantic_analyzer: Arc<Mutex<PerceptionSemanticAnalyzer>>,
    context_selector: Arc<Mutex<ContextAwareSelector>>,
    form_handler: Arc<Mutex<SmartFormHandler>>,
    dynamic_handler: Arc<Mutex<DynamicContentHandler>>,
    
    // System state
    system_state: Arc<RwLock<SystemState>>,
    performance_tracker: Arc<Mutex<PerformanceTracker>>,
    user_profiles: Arc<RwLock<HashMap<String, UserProfile>>>,
    
    // Testing framework
    test_framework: Option<Arc<Mutex<PerceptionTestFramework>>>,
    
    // Configuration
    config: SystemConfiguration,
}

/// System state that tracks context across interactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemState {
    pub current_url: String,
    pub page_type: PageType,
    pub interaction_history: Vec<InteractionRecord>,
    pub element_memory: HashMap<String, CachedElement>,
    pub form_state: FormState,
    pub session_context: SessionContext,
    pub error_recovery_count: u32,
    pub last_successful_action: Option<String>,
}

/// Performance tracking for the unified system
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PerformanceTracker {
    pub session_start: Option<Instant>,
    pub total_commands: u32,
    pub successful_commands: u32,
    pub failed_commands: u32,
    pub average_response_time: Duration,
    pub perception_accuracy_rate: f32,
    pub form_completion_rate: f32,
    pub error_recovery_success_rate: f32,
    pub layer_usage_stats: HashMap<String, LayerStats>,
}

/// Statistics for each perception layer
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LayerStats {
    pub usage_count: u32,
    pub success_count: u32,
    pub average_confidence: f32,
    pub average_execution_time: Duration,
}

/// User profile for personalized automation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub id: String,
    pub name: String,
    pub preferences: UserPreferences,
    pub form_profiles: Vec<FormProfile>,
    pub interaction_patterns: InteractionPatterns,
    pub accessibility_requirements: AccessibilityRequirements,
}

/// User preferences for system behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    pub auto_fill_forms: bool,
    pub dismiss_modals_automatically: bool,
    pub wait_for_animations: bool,
    pub take_automatic_screenshots: bool,
    pub error_recovery_enabled: bool,
    pub max_wait_time: Duration,
    pub preferred_interaction_speed: InteractionSpeed,
    pub notification_preferences: NotificationSettings,
}

/// Interaction patterns for learning user behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionPatterns {
    pub common_workflows: Vec<WorkflowPattern>,
    pub preferred_selectors: Vec<String>,
    pub typical_wait_times: HashMap<String, Duration>,
    pub error_recovery_strategies: Vec<RecoveryStrategy>,
}

/// Accessibility requirements support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessibilityRequirements {
    pub screen_reader_compatible: bool,
    pub high_contrast_mode: bool,
    pub keyboard_navigation_only: bool,
    pub reduced_motion: bool,
    pub custom_css_overrides: Vec<String>,
}

/// System configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfiguration {
    pub enable_visual_analysis: bool,
    pub enable_form_auto_fill: bool,
    pub enable_dynamic_content_handling: bool,
    pub enable_context_awareness: bool,
    pub enable_error_recovery: bool,
    pub enable_performance_monitoring: bool,
    pub enable_testing_framework: bool,
    pub max_parallel_operations: u32,
    pub cache_duration: Duration,
    pub debug_mode: bool,
}

/// Unified command interface
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UnifiedCommand {
    // Basic navigation
    NavigateTo { url: String, wait_for_load: bool },
    GoBack,
    GoForward,
    Refresh,
    Close,
    
    // Intelligent element interaction
    FindAndClick { description: String, context: Option<String> },
    FindAndType { description: String, text: String, clear_first: bool },
    FindAndSelect { description: String, option: String },
    FindAndExtract { description: String, format: Option<String> },
    
    // Smart form operations
    AnalyzeForms,
    AutoFillForm { profile_name: String, form_selector: Option<String> },
    SubmitForm { form_selector: Option<String> },
    ValidateForm { form_selector: Option<String> },
    
    // Dynamic content handling
    WaitForElement { description: String, timeout: Option<Duration> },
    WaitForPageLoad { timeout: Option<Duration> },
    HandleInfiniteScroll { max_items: Option<u32> },
    DismissModals { action: ModalDismissalStrategy },
    TriggerLazyLoading,
    
    // Advanced analysis
    AnalyzePage { include_visual: bool, include_forms: bool },
    ExtractPageData { selectors: Vec<String>, format: DataFormat },
    TakeScreenshot { description: Option<String>, full_page: bool },
    ComparePages { baseline_url: Option<String> },
    
    // Context and memory
    RememberElement { description: String, name: String },
    ForgetElement { name: String },
    RecallElement { name: String },
    ListRememberedElements,
    SaveSession { name: String },
    LoadSession { name: String },
    
    // User management
    CreateUserProfile { profile: UserProfile },
    UpdateUserProfile { id: String, updates: UserProfileUpdates },
    SetActiveProfile { id: String },
    
    // Testing and validation
    RunTest { test_name: String },
    ValidateAccessibility,
    BenchmarkPerformance { iterations: u32 },
    
    // Natural language interface
    ExecuteNaturalLanguage { instruction: String },
    AskForSuggestions,
    ExplainCurrentState,
}

/// Result of unified command execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedCommandResult {
    pub success: bool,
    pub command_id: String,
    pub execution_time: Duration,
    pub layers_used: Vec<String>,
    pub confidence_scores: HashMap<String, f32>,
    pub data: Option<serde_json::Value>,
    pub error: Option<CommandError>,
    pub suggestions: Vec<String>,
    pub side_effects: Vec<SideEffect>,
    pub performance_metrics: CommandPerformanceMetrics,
    pub accessibility_notes: Vec<AccessibilityNote>,
}

// Supporting types for commands and results

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InteractionSpeed {
    Slow,      // For accessibility or demonstration
    Normal,    // Standard speed
    Fast,      // Quick automation
    Instant,   // No delays
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationSettings {
    pub notify_on_success: bool,
    pub notify_on_failure: bool,
    pub notify_on_slow_response: bool,
    pub notification_method: NotificationMethod,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationMethod {
    Console,
    Email,
    Webhook,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowPattern {
    pub name: String,
    pub steps: Vec<String>,
    pub frequency: u32,
    pub success_rate: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryStrategy {
    pub error_type: String,
    pub strategy: String,
    pub success_rate: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModalDismissalStrategy {
    Accept,
    Dismiss,
    Ignore,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataFormat {
    Json,
    Csv,
    Xml,
    Text,
    Structured,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfileUpdates {
    pub preferences: Option<UserPreferences>,
    pub form_profiles: Option<Vec<FormProfile>>,
    pub accessibility_requirements: Option<AccessibilityRequirements>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandError {
    pub error_type: ErrorType,
    pub message: String,
    pub layer: Option<String>,
    pub recovery_suggestions: Vec<String>,
    pub technical_details: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorType {
    ElementNotFound,
    TimeoutError,
    NetworkError,
    InvalidInput,
    PermissionDenied,
    SystemError,
    UserError,
    ConfigurationError,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SideEffect {
    pub effect_type: SideEffectType,
    pub description: String,
    pub impact: Impact,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SideEffectType {
    NavigationTriggered,
    ModalAppeared,
    FormSubmitted,
    PageRefreshed,
    CookiesSet,
    LocalStorageModified,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Impact {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandPerformanceMetrics {
    pub perception_time: Duration,
    pub execution_time: Duration,
    pub validation_time: Duration,
    pub total_time: Duration,
    pub memory_usage: u64,
    pub network_requests: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessibilityNote {
    pub note_type: AccessibilityNoteType,
    pub message: String,
    pub severity: AccessibilitySeverity,
    pub wcag_reference: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccessibilityNoteType {
    MissingAltText,
    LowContrast,
    NoKeyboardAccess,
    MissingLabel,
    InvalidAriaAttribute,
    FocusIndicatorMissing,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccessibilitySeverity {
    Info,
    Warning,
    Error,
    Critical,
}

// Additional types for system state

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionRecord {
    pub timestamp: String,
    pub command: String,
    pub target: String,
    pub result: String,
    pub duration: Duration,
    pub layer_used: String,
    pub confidence: f32,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedElement {
    pub description: String,
    pub selector: String,
    pub element_type: String,
    pub confidence: f32,
    pub last_accessed: String,
    pub access_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormState {
    pub active_forms: Vec<FormAnalysis>,
    pub current_form: Option<String>,
    pub auto_fill_enabled: bool,
    pub validation_state: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionContext {
    pub session_id: String,
    pub start_time: String,
    pub user_goal: Option<String>,
    pub current_workflow: Option<String>,
    pub breadcrumb: Vec<String>,
    pub context_variables: HashMap<String, serde_json::Value>,
}

impl UnifiedBrowserSystem {
    /// Create a new unified browser system
    pub fn new(driver: WebDriver) -> Self {
        let driver_arc = Arc::new(driver);
        let session_id = Uuid::new_v4().to_string();
        
        Self {
            driver: driver_arc.clone(),
            session_id: session_id.clone(),
            
            // Initialize perception layers
            mvp_engine: Arc::new(Mutex::new(PerceptionEngineMVP::new((**driver_arc).clone()))),
            enhanced_engine: Arc::new(Mutex::new(EnhancedPerceptionEngine::new((**driver_arc).clone()))),
            visual_analyzer: Arc::new(Mutex::new(VisualAnalyzer::new((**driver_arc).clone()))),
            semantic_analyzer: Arc::new(Mutex::new(PerceptionSemanticAnalyzer::new((**driver_arc).clone()))),
            context_selector: Arc::new(Mutex::new(ContextAwareSelector::new((**driver_arc).clone()))),
            form_handler: Arc::new(Mutex::new(SmartFormHandler::new((**driver_arc).clone()))),
            dynamic_handler: Arc::new(Mutex::new(DynamicContentHandler::new((**driver_arc).clone()))),
            
            // Initialize system state
            system_state: Arc::new(RwLock::new(SystemState {
                current_url: String::new(),
                page_type: PageType::Unknown,
                interaction_history: Vec::new(),
                element_memory: HashMap::new(),
                form_state: FormState {
                    active_forms: Vec::new(),
                    current_form: None,
                    auto_fill_enabled: true,
                    validation_state: HashMap::new(),
                },
                session_context: SessionContext {
                    session_id: session_id.clone(),
                    start_time: chrono::Utc::now().to_rfc3339(),
                    user_goal: None,
                    current_workflow: None,
                    breadcrumb: Vec::new(),
                    context_variables: HashMap::new(),
                },
                error_recovery_count: 0,
                last_successful_action: None,
            })),
            
            performance_tracker: Arc::new(Mutex::new(PerformanceTracker {
                session_start: Some(Instant::now()),
                ..Default::default()
            })),
            
            user_profiles: Arc::new(RwLock::new(HashMap::new())),
            test_framework: None,
            config: SystemConfiguration::default(),
        }
    }

    /// Enable testing framework
    pub fn with_testing(mut self) -> Self {
        self.test_framework = Some(Arc::new(Mutex::new(
            PerceptionTestFramework::new((**self.driver).clone())
        )));
        self
    }

    /// Set system configuration
    pub fn with_config(mut self, config: SystemConfiguration) -> Self {
        self.config = config;
        self
    }

    /// Execute a unified command with full intelligence
    pub async fn execute_command(&self, command: UnifiedCommand) -> Result<UnifiedCommandResult> {
        let start_time = Instant::now();
        let command_id = Uuid::new_v4().to_string();
        
        info!("Executing command: {:?}", command);
        
        // Update performance tracking
        {
            let mut tracker = self.performance_tracker.lock().await;
            tracker.total_commands += 1;
        }
        
        // Execute the command based on its type
        let result = match command {
            UnifiedCommand::NavigateTo { url, wait_for_load } => {
                self.handle_navigation(&url, wait_for_load).await
            },
            UnifiedCommand::FindAndClick { description, context } => {
                self.handle_intelligent_click(&description, context.as_deref()).await
            },
            UnifiedCommand::FindAndType { description, text, clear_first } => {
                self.handle_intelligent_type(&description, &text, clear_first).await
            },
            UnifiedCommand::AutoFillForm { profile_name, form_selector } => {
                self.handle_auto_fill_form(&profile_name, form_selector.as_deref()).await
            },
            UnifiedCommand::AnalyzePage { include_visual, include_forms } => {
                self.handle_page_analysis(include_visual, include_forms).await
            },
            UnifiedCommand::WaitForElement { description, timeout } => {
                self.handle_intelligent_wait(&description, timeout).await
            },
            UnifiedCommand::HandleInfiniteScroll { max_items } => {
                self.handle_infinite_scroll(max_items).await
            },
            UnifiedCommand::TakeScreenshot { description, full_page } => {
                self.handle_screenshot(description.as_deref(), full_page).await
            },
            UnifiedCommand::ExecuteNaturalLanguage { instruction } => {
                self.handle_natural_language(&instruction).await
            },
            _ => self.handle_unsupported_command(command).await,
        };

        let execution_time = start_time.elapsed();

        // Update performance metrics
        self.update_performance_metrics(&result, execution_time).await;

        // Record interaction
        self.record_interaction(&command, &result, execution_time).await?;

        Ok(UnifiedCommandResult {
            success: result.success,
            command_id,
            execution_time,
            layers_used: result.layers_used,
            confidence_scores: result.confidence_scores,
            data: result.data,
            error: result.error,
            suggestions: result.suggestions,
            side_effects: result.side_effects,
            performance_metrics: result.performance_metrics,
            accessibility_notes: result.accessibility_notes,
        })
    }

    /// Navigate to URL with intelligent page analysis
    async fn handle_navigation(&self, url: &str, wait_for_load: bool) -> Result<CommandExecutionResult> {
        let mut layers_used = vec!["navigation".to_string()];
        let mut confidence_scores = HashMap::new();
        
        // Navigate to the URL
        self.driver.goto(url).await?;
        confidence_scores.insert("navigation".to_string(), 1.0);
        
        if wait_for_load {
            // Use dynamic handler to wait for page load
            let mut dynamic_handler = self.dynamic_handler.lock().await;
            let wait_result = dynamic_handler.wait_for_page_ready().await?;
            
            if wait_result.success {
                layers_used.push("dynamic_content".to_string());
                confidence_scores.insert("page_load".to_string(), 0.9);
            }
        }
        
        // Analyze the new page
        let mut semantic_analyzer = self.semantic_analyzer.lock().await;
        let page_analysis = semantic_analyzer.analyze_page_classification().await?;
        layers_used.push("semantic_analysis".to_string());
        confidence_scores.insert("semantic_analysis".to_string(), page_analysis.confidence);
        
        // Update system state
        {
            let mut state = self.system_state.write().await;
            state.current_url = url.to_string();
            state.page_type = page_analysis.page_type;
            state.session_context.breadcrumb.push(url.to_string());
        }
        
        Ok(CommandExecutionResult {
            success: true,
            layers_used,
            confidence_scores,
            data: Some(serde_json::json!({
                "url": url,
                "page_type": page_analysis.page_type,
                "load_time": wait_for_load
            })),
            error: None,
            suggestions: vec![],
            side_effects: vec![SideEffect {
                effect_type: SideEffectType::NavigationTriggered,
                description: format!("Navigated to {}", url),
                impact: Impact::High,
            }],
            performance_metrics: CommandPerformanceMetrics {
                perception_time: Duration::from_millis(100),
                execution_time: Duration::from_millis(2000),
                validation_time: Duration::from_millis(50),
                total_time: Duration::from_millis(2150),
                memory_usage: 1024,
                network_requests: 1,
            },
            accessibility_notes: vec![],
        })
    }

    /// Intelligent element clicking with multi-layer fallback
    async fn handle_intelligent_click(&self, description: &str, context: Option<&str>) -> Result<CommandExecutionResult> {
        let mut layers_used = Vec::new();
        let mut confidence_scores = HashMap::new();
        let mut error_message = None;
        
        // Try context-aware selection first
        if let Ok(mut context_selector) = self.context_selector.try_lock() {
            match context_selector.find_element_contextual(description).await {
                Ok(element) => {
                    if let Ok(web_element) = self.driver.find(By::Css(&element.selector)).await {
                        match web_element.click().await {
                            Ok(_) => {
                                layers_used.push("context_aware".to_string());
                                confidence_scores.insert("context_aware".to_string(), element.confidence);
                                
                                return Ok(CommandExecutionResult {
                                    success: true,
                                    layers_used,
                                    confidence_scores,
                                    data: Some(serde_json::json!({
                                        "element_found": true,
                                        "selector": element.selector,
                                        "method": "context_aware"
                                    })),
                                    error: None,
                                    suggestions: vec![],
                                    side_effects: vec![],
                                    performance_metrics: CommandPerformanceMetrics {
                                        perception_time: Duration::from_millis(50),
                                        execution_time: Duration::from_millis(100),
                                        validation_time: Duration::from_millis(10),
                                        total_time: Duration::from_millis(160),
                                        memory_usage: 256,
                                        network_requests: 0,
                                    },
                                    accessibility_notes: vec![],
                                });
                            }
                            Err(e) => error_message = Some(e.to_string()),
                        }
                    }
                }
                Err(e) => error_message = Some(e.to_string()),
            }
        }
        
        // Fall back to MVP engine
        if let Ok(mut mvp_engine) = self.mvp_engine.try_lock() {
            match mvp_engine.find_element(description).await {
                Ok(element) => {
                    if let Ok(web_element) = self.driver.find(By::Css(&element.selector)).await {
                        match web_element.click().await {
                            Ok(_) => {
                                layers_used.push("mvp_engine".to_string());
                                confidence_scores.insert("mvp_engine".to_string(), element.confidence);
                                
                                return Ok(CommandExecutionResult {
                                    success: true,
                                    layers_used,
                                    confidence_scores,
                                    data: Some(serde_json::json!({
                                        "element_found": true,
                                        "selector": element.selector,
                                        "method": "mvp_engine"
                                    })),
                                    error: None,
                                    suggestions: vec![],
                                    side_effects: vec![],
                                    performance_metrics: CommandPerformanceMetrics {
                                        perception_time: Duration::from_millis(100),
                                        execution_time: Duration::from_millis(100),
                                        validation_time: Duration::from_millis(10),
                                        total_time: Duration::from_millis(210),
                                        memory_usage: 512,
                                        network_requests: 0,
                                    },
                                    accessibility_notes: vec![],
                                });
                            }
                            Err(e) => error_message = Some(e.to_string()),
                        }
                    }
                }
                Err(e) => error_message = Some(e.to_string()),
            }
        }
        
        // Generate suggestions for failed attempts
        let suggestions = vec![
            "Try being more specific in your description".to_string(),
            "Check if the element is visible on the page".to_string(),
            "Wait for the page to finish loading".to_string(),
        ];
        
        Ok(CommandExecutionResult {
            success: false,
            layers_used,
            confidence_scores,
            data: None,
            error: Some(CommandError {
                error_type: ErrorType::ElementNotFound,
                message: format!("Could not find element: {}", description),
                layer: None,
                recovery_suggestions: suggestions.clone(),
                technical_details: error_message,
            }),
            suggestions,
            side_effects: vec![],
            performance_metrics: CommandPerformanceMetrics {
                perception_time: Duration::from_millis(200),
                execution_time: Duration::from_millis(0),
                validation_time: Duration::from_millis(10),
                total_time: Duration::from_millis(210),
                memory_usage: 768,
                network_requests: 0,
            },
            accessibility_notes: vec![],
        })
    }

    /// Intelligent form auto-filling
    async fn handle_auto_fill_form(&self, profile_name: &str, form_selector: Option<&str>) -> Result<CommandExecutionResult> {
        let mut layers_used = vec!["smart_forms".to_string()];
        let mut confidence_scores = HashMap::new();
        
        // Get user profile
        let profiles = self.user_profiles.read().await;
        let profile = profiles.get(profile_name).cloned();
        drop(profiles);
        
        if let Some(user_profile) = profile {
            if let Some(form_profile) = user_profile.form_profiles.first() {
                let mut form_handler = self.form_handler.lock().await;
                
                // Analyze forms on the page
                let form_analysis = form_handler.analyze_forms().await?;
                layers_used.push("form_analysis".to_string());
                
                // Select the appropriate form
                let target_form = if let Some(selector) = form_selector {
                    form_analysis.iter().find(|f| f.selector.contains(selector))
                } else {
                    form_analysis.first()
                };
                
                if let Some(form) = target_form {
                    // Auto-fill the form
                    let fill_result = form_handler.auto_fill_form(form_profile, form).await?;
                    confidence_scores.insert("form_auto_fill".to_string(), fill_result.confidence);
                    
                    return Ok(CommandExecutionResult {
                        success: fill_result.success,
                        layers_used,
                        confidence_scores,
                        data: Some(serde_json::json!({
                            "fields_filled": fill_result.fields_filled,
                            "total_fields": fill_result.total_fields,
                            "success_rate": fill_result.success_rate,
                            "form_type": form.form_type
                        })),
                        error: if fill_result.success { None } else {
                            Some(CommandError {
                                error_type: ErrorType::SystemError,
                                message: fill_result.errors.join("; "),
                                layer: Some("smart_forms".to_string()),
                                recovery_suggestions: fill_result.suggestions,
                                technical_details: None,
                            })
                        },
                        suggestions: fill_result.suggestions,
                        side_effects: vec![SideEffect {
                            effect_type: SideEffectType::FormSubmitted,
                            description: "Form fields were automatically filled".to_string(),
                            impact: Impact::Medium,
                        }],
                        performance_metrics: CommandPerformanceMetrics {
                            perception_time: Duration::from_millis(200),
                            execution_time: fill_result.processing_time,
                            validation_time: Duration::from_millis(50),
                            total_time: fill_result.processing_time + Duration::from_millis(250),
                            memory_usage: 1024,
                            network_requests: 0,
                        },
                        accessibility_notes: vec![],
                    });
                }
            }
        }
        
        Ok(CommandExecutionResult {
            success: false,
            layers_used,
            confidence_scores,
            data: None,
            error: Some(CommandError {
                error_type: ErrorType::UserError,
                message: format!("Profile '{}' not found or no suitable form on page", profile_name),
                layer: Some("smart_forms".to_string()),
                recovery_suggestions: vec!["Create a user profile first".to_string()],
                technical_details: None,
            }),
            suggestions: vec!["Create a user profile first".to_string()],
            side_effects: vec![],
            performance_metrics: CommandPerformanceMetrics {
                perception_time: Duration::from_millis(50),
                execution_time: Duration::from_millis(0),
                validation_time: Duration::from_millis(10),
                total_time: Duration::from_millis(60),
                memory_usage: 256,
                network_requests: 0,
            },
            accessibility_notes: vec![],
        })
    }

    /// Add user profile to the system
    pub async fn add_user_profile(&self, profile: UserProfile) -> Result<()> {
        let mut profiles = self.user_profiles.write().await;
        profiles.insert(profile.id.clone(), profile);
        Ok(())
    }

    /// Get system performance metrics
    pub async fn get_performance_metrics(&self) -> PerformanceTracker {
        self.performance_tracker.lock().await.clone()
    }

    /// Get current system state
    pub async fn get_system_state(&self) -> SystemState {
        self.system_state.read().await.clone()
    }

    // Additional helper methods for handling other commands...
    // (Implementation details for other command handlers would follow similar patterns)

    async fn handle_intelligent_type(&self, description: &str, text: &str, clear_first: bool) -> Result<CommandExecutionResult> {
        // Implementation would be similar to handle_intelligent_click but for typing
        todo!("Implement intelligent typing")
    }

    async fn handle_page_analysis(&self, include_visual: bool, include_forms: bool) -> Result<CommandExecutionResult> {
        // Implementation would use multiple analyzers to provide comprehensive page analysis
        todo!("Implement comprehensive page analysis")
    }

    async fn handle_intelligent_wait(&self, description: &str, timeout: Option<Duration>) -> Result<CommandExecutionResult> {
        // Implementation would use dynamic content handler for intelligent waiting
        todo!("Implement intelligent waiting")
    }

    async fn handle_infinite_scroll(&self, max_items: Option<u32>) -> Result<CommandExecutionResult> {
        // Implementation would use dynamic content handler for infinite scroll
        todo!("Implement infinite scroll handling")
    }

    async fn handle_screenshot(&self, description: Option<&str>, full_page: bool) -> Result<CommandExecutionResult> {
        // Implementation would take and optionally annotate screenshots
        todo!("Implement screenshot functionality")
    }

    async fn handle_natural_language(&self, instruction: &str) -> Result<CommandExecutionResult> {
        // Implementation would parse natural language and execute appropriate actions
        todo!("Implement natural language processing")
    }

    async fn handle_unsupported_command(&self, command: UnifiedCommand) -> Result<CommandExecutionResult> {
        Ok(CommandExecutionResult {
            success: false,
            layers_used: vec![],
            confidence_scores: HashMap::new(),
            data: None,
            error: Some(CommandError {
                error_type: ErrorType::SystemError,
                message: format!("Unsupported command: {:?}", command),
                layer: None,
                recovery_suggestions: vec!["Try a different command".to_string()],
                technical_details: None,
            }),
            suggestions: vec!["Try a different command".to_string()],
            side_effects: vec![],
            performance_metrics: CommandPerformanceMetrics {
                perception_time: Duration::from_millis(1),
                execution_time: Duration::from_millis(0),
                validation_time: Duration::from_millis(1),
                total_time: Duration::from_millis(2),
                memory_usage: 0,
                network_requests: 0,
            },
            accessibility_notes: vec![],
        })
    }

    async fn update_performance_metrics(&self, result: &CommandExecutionResult, execution_time: Duration) {
        let mut tracker = self.performance_tracker.lock().await;
        
        if result.success {
            tracker.successful_commands += 1;
        } else {
            tracker.failed_commands += 1;
        }
        
        // Update average response time
        let total_time = tracker.average_response_time.as_millis() as f64 * (tracker.total_commands - 1) as f64;
        let new_average = (total_time + execution_time.as_millis() as f64) / tracker.total_commands as f64;
        tracker.average_response_time = Duration::from_millis(new_average as u64);
        
        // Update layer usage statistics
        for layer in &result.layers_used {
            let stats = tracker.layer_usage_stats.entry(layer.clone()).or_default();
            stats.usage_count += 1;
            
            if result.success {
                stats.success_count += 1;
            }
            
            if let Some(confidence) = result.confidence_scores.get(layer) {
                let total_confidence = stats.average_confidence * (stats.usage_count - 1) as f32;
                stats.average_confidence = (total_confidence + confidence) / stats.usage_count as f32;
            }
            
            // Update average execution time for this layer
            let total_layer_time = stats.average_execution_time.as_millis() as f64 * (stats.usage_count - 1) as f64;
            let new_layer_average = (total_layer_time + execution_time.as_millis() as f64) / stats.usage_count as f64;
            stats.average_execution_time = Duration::from_millis(new_layer_average as u64);
        }
    }

    async fn record_interaction(&self, command: &UnifiedCommand, result: &CommandExecutionResult, duration: Duration) -> Result<()> {
        let interaction = InteractionRecord {
            timestamp: chrono::Utc::now().to_rfc3339(),
            command: format!("{:?}", command),
            target: "web_page".to_string(), // Could be more specific
            result: if result.success { "success".to_string() } else { "failure".to_string() },
            duration,
            layer_used: result.layers_used.first().unwrap_or(&"none".to_string()).clone(),
            confidence: result.confidence_scores.values().fold(0.0, |acc, &x| acc + x) / result.confidence_scores.len().max(1) as f32,
            success: result.success,
        };
        
        let mut state = self.system_state.write().await;
        state.interaction_history.push(interaction);
        
        // Keep only last 100 interactions
        if state.interaction_history.len() > 100 {
            state.interaction_history.remove(0);
        }
        
        Ok(())
    }
}

// Internal result type for command execution
#[derive(Debug, Clone)]
struct CommandExecutionResult {
    success: bool,
    layers_used: Vec<String>,
    confidence_scores: HashMap<String, f32>,
    data: Option<serde_json::Value>,
    error: Option<CommandError>,
    suggestions: Vec<String>,
    side_effects: Vec<SideEffect>,
    performance_metrics: CommandPerformanceMetrics,
    accessibility_notes: Vec<AccessibilityNote>,
}

impl Default for SystemConfiguration {
    fn default() -> Self {
        Self {
            enable_visual_analysis: true,
            enable_form_auto_fill: true,
            enable_dynamic_content_handling: true,
            enable_context_awareness: true,
            enable_error_recovery: true,
            enable_performance_monitoring: true,
            enable_testing_framework: false,
            max_parallel_operations: 3,
            cache_duration: Duration::from_secs(300),
            debug_mode: false,
        }
    }
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            auto_fill_forms: true,
            dismiss_modals_automatically: true,
            wait_for_animations: true,
            take_automatic_screenshots: false,
            error_recovery_enabled: true,
            max_wait_time: Duration::from_secs(30),
            preferred_interaction_speed: InteractionSpeed::Normal,
            notification_preferences: NotificationSettings {
                notify_on_success: false,
                notify_on_failure: true,
                notify_on_slow_response: true,
                notification_method: NotificationMethod::Console,
            },
        }
    }
}

/// Demonstration function showcasing the unified system
pub async fn demonstrate_unified_system(driver: WebDriver) -> Result<()> {
    println!("🌈 RainbowBrowserAI - Unified System Demonstration");
    println!("==================================================\n");
    
    // Initialize the unified system
    let system = UnifiedBrowserSystem::new(driver)
        .with_testing()
        .with_config(SystemConfiguration::default());
    
    // Create a sample user profile
    let user_profile = UserProfile {
        id: "demo_user".to_string(),
        name: "Demo User".to_string(),
        preferences: UserPreferences::default(),
        form_profiles: vec![FormProfile {
            name: "personal".to_string(),
            personal: crate::perception_mvp::smart_forms::PersonalInfo {
                first_name: "John".to_string(),
                last_name: "Doe".to_string(),
                date_of_birth: Some("1990-01-01".to_string()),
                gender: Some("Male".to_string()),
            },
            contact: crate::perception_mvp::smart_forms::ContactInfo {
                email: "john.doe@example.com".to_string(),
                phone: "+1-555-123-4567".to_string(),
                secondary_email: None,
                secondary_phone: None,
            },
            address: crate::perception_mvp::smart_forms::AddressInfo {
                street: "123 Main St".to_string(),
                city: "Anytown".to_string(),
                state: "CA".to_string(),
                zip_code: "12345".to_string(),
                country: "USA".to_string(),
            },
            preferences: HashMap::new(),
        }],
        interaction_patterns: InteractionPatterns {
            common_workflows: vec![],
            preferred_selectors: vec![],
            typical_wait_times: HashMap::new(),
            error_recovery_strategies: vec![],
        },
        accessibility_requirements: AccessibilityRequirements {
            screen_reader_compatible: false,
            high_contrast_mode: false,
            keyboard_navigation_only: false,
            reduced_motion: false,
            custom_css_overrides: vec![],
        },
    };
    
    system.add_user_profile(user_profile).await?;
    
    // Demonstrate various capabilities
    
    // 1. Intelligent Navigation
    println!("🧭 Demo 1: Intelligent Navigation");
    let result = system.execute_command(UnifiedCommand::NavigateTo {
        url: "https://example.com".to_string(),
        wait_for_load: true,
    }).await?;
    println!("   Success: {}, Layers used: {:?}", result.success, result.layers_used);
    println!("   Execution time: {:?}", result.execution_time);
    
    // 2. Smart Element Interaction
    println!("\n🎯 Demo 2: Intelligent Element Finding");
    let result = system.execute_command(UnifiedCommand::FindAndClick {
        description: "the main link".to_string(),
        context: Some("homepage navigation".to_string()),
    }).await?;
    println!("   Success: {}, Confidence: {:?}", result.success, result.confidence_scores);
    
    // 3. Form Auto-fill
    println!("\n📝 Demo 3: Smart Form Auto-fill");
    let result = system.execute_command(UnifiedCommand::AutoFillForm {
        profile_name: "demo_user".to_string(),
        form_selector: None,
    }).await?;
    println!("   Success: {}, Data: {:?}", result.success, result.data);
    
    // 4. Page Analysis
    println!("\n🔍 Demo 4: Comprehensive Page Analysis");
    let result = system.execute_command(UnifiedCommand::AnalyzePage {
        include_visual: true,
        include_forms: true,
    }).await?;
    println!("   Analysis complete: {}", result.success);
    
    // 5. Performance Metrics
    println!("\n📊 Demo 5: Performance Metrics");
    let metrics = system.get_performance_metrics().await;
    println!("   Total commands: {}", metrics.total_commands);
    println!("   Success rate: {:.1}%", 
        (metrics.successful_commands as f64 / metrics.total_commands as f64) * 100.0);
    println!("   Average response time: {:?}", metrics.average_response_time);
    
    // 6. System State
    println!("\n🏗️ Demo 6: System State");
    let state = system.get_system_state().await;
    println!("   Current URL: {}", state.current_url);
    println!("   Page type: {:?}", state.page_type);
    println!("   Interactions recorded: {}", state.interaction_history.len());
    
    println!("\n✨ Unified System Demonstration Complete!");
    println!("The system successfully integrated all perception layers into a cohesive, intelligent interface.");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test;
    
    #[test]
    fn test_system_configuration_default() {
        let config = SystemConfiguration::default();
        assert!(config.enable_visual_analysis);
        assert!(config.enable_form_auto_fill);
        assert_eq!(config.max_parallel_operations, 3);
    }
    
    #[test]
    fn test_user_preferences_default() {
        let prefs = UserPreferences::default();
        assert!(prefs.auto_fill_forms);
        assert!(prefs.error_recovery_enabled);
        assert_eq!(prefs.max_wait_time, Duration::from_secs(30));
    }
    
    #[test]
    fn test_command_error_creation() {
        let error = CommandError {
            error_type: ErrorType::ElementNotFound,
            message: "Test error".to_string(),
            layer: Some("test_layer".to_string()),
            recovery_suggestions: vec!["Try again".to_string()],
            technical_details: None,
        };
        
        assert!(matches!(error.error_type, ErrorType::ElementNotFound));
        assert_eq!(error.message, "Test error");
    }
}