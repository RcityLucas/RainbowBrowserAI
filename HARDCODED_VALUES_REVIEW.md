# Code Review: Hardcoded Values Analysis

## Summary
**Status**: ⚠️ MULTIPLE HARDCODED VALUES FOUND - Requires Configuration Refactoring

The code contains numerous hardcoded values that should be moved to configuration files for better maintainability, flexibility, and customization.

## Critical Hardcoded Values Found

### 1. Website URLs (HIGH PRIORITY)
**Files**: `task_executor.rs`, `llm_service_enhanced.rs`, `llm_service.rs`

**Hardcoded URLs**:
```rust
// task_executor.rs
"https://google.com" (fallback URL)

// llm_service_enhanced.rs  
"https://www.google.com/search?q=travel+guide"
"https://www.google.com/flights"
"https://www.booking.com"
"https://www.tripadvisor.com"
"https://www.weather.com"
"https://www.google.com"

// llm_service.rs
["google.com", "github.com", "stackoverflow.com", "reddit.com", 
 "youtube.com", "twitter.com", "facebook.com", "linkedin.com",
 "amazon.com", "wikipedia.org"]
```

**Risk**: URLs may become invalid, different regions need different sites, business requirements change.

### 2. Business Logic Keywords (HIGH PRIORITY)  
**File**: `llm_service_enhanced.rs`

**Hardcoded Intent Classification**:
```rust
// Planning keywords
["plan", "itinerary", "trip", "vacation"]

// Search keywords  
["search", "find", "look for"]

// Analysis keywords
["analyze", "review", "evaluate"]

// Extract keywords
["extract", "scrape", "collect"]

// Monitor keywords
["monitor", "watch", "track"]

// Test keywords
["test"] + ["sites", "websites"]

// Location entities
["paris", "tokyo", "new york", "london", "rome", "barcelona", 
 "berlin", "amsterdam", "prague", "vienna"]

// Time entities
["tomorrow", "next week"]
```

**Risk**: Different languages, regional variations, business domain changes.

### 3. Timeout and Timing Values (MEDIUM PRIORITY)
**Files**: `task_executor.rs`, `llm_service_enhanced.rs`

**Hardcoded Timings**:
```rust
// task_executor.rs
max_attempts: 3
delay_seconds: 2  
timeout: Some(30) // 30 seconds per step
estimated_remaining: 15000 // 15s per step estimate
tokio::time::sleep(Duration::from_millis(500)) // demo delay

// llm_service_enhanced.rs
estimated_duration = 60 * steps.len() // 60 seconds per step
```

**Risk**: Different environments need different timeouts, performance requirements vary.

### 4. Confidence Scores (MEDIUM PRIORITY)
**Files**: `llm_service_enhanced.rs`, `llm_service.rs`

**Hardcoded Confidence Values**:
```rust
confidence: 0.9    // Location entities
confidence: 0.95   // Date/website entities  
confidence: 0.85   // Search categories
confidence: 0.85   // Planning tasks
confidence: 0.8    // Search/Analysis tasks
confidence: 0.3    // Unknown tasks
```

**Risk**: Machine learning models may need different thresholds, A/B testing requirements.

### 5. Display Formatting (LOW PRIORITY)
**Files**: `task_executor.rs`

**Hardcoded Display Logic**:
```rust
if i < 5 { // Show first 5 URLs
} else if i == 5 {
    println!("... and {} more", total - 5);
}

println!("⏱️  Total Duration: {:.1}s", duration);
println!("💰 Cost: ${:.4}", cost);
```

**Risk**: UI requirements change, different number formats needed.

### 6. File Naming Patterns (LOW PRIORITY)
**Files**: `task_executor.rs`

**Hardcoded Patterns**:
```rust
format!("task_step_{}_{}.png", step_number, timestamp)
format!("workflow_{}.png", timestamp)
```

**Risk**: File organization requirements change.

## Recommended Configuration Structure

### 1. Create Configuration Files

```yaml
# config/task_execution.yaml
task_execution:
  timeouts:
    step_timeout_seconds: 30
    max_retries: 3
    retry_delay_seconds: 2
    step_estimate_seconds: 60
    
  confidence_thresholds:
    planning_task: 0.85
    search_task: 0.8
    analysis_task: 0.8
    unknown_threshold: 0.3
    high_entity_confidence: 0.95
    medium_entity_confidence: 0.9
    
  display:
    max_urls_shown: 5
    duration_decimal_places: 1
    cost_decimal_places: 4

# config/intent_classification.yaml  
intent_classification:
  planning_keywords: ["plan", "itinerary", "trip", "vacation", "journey"]
  search_keywords: ["search", "find", "look for", "discover"]
  analysis_keywords: ["analyze", "review", "evaluate", "assess"]
  extract_keywords: ["extract", "scrape", "collect", "gather"]
  monitor_keywords: ["monitor", "watch", "track", "observe"]
  test_keywords: ["test"]
  test_modifiers: ["sites", "websites", "pages"]

# config/travel_urls.yaml
travel_urls:
  search_engine: "https://www.google.com/search?q=travel+guide"
  flights: "https://www.google.com/flights"
  hotels: "https://www.booking.com"
  attractions: "https://www.tripadvisor.com"
  weather: "https://www.weather.com"
  
known_sites:
  - name: "google"
    url: "google.com"
  - name: "github" 
    url: "github.com"
  # ... etc

# config/entities.yaml
entities:
  locations: ["paris", "tokyo", "new york", "london", "rome"]
  time_expressions: ["tomorrow", "next week", "next month"]
```

### 2. Configuration Loading Infrastructure

```rust
// config/mod.rs
#[derive(Debug, Deserialize)]
pub struct TaskExecutionConfig {
    pub timeouts: TimeoutConfig,
    pub confidence_thresholds: ConfidenceConfig,
    pub display: DisplayConfig,
}

#[derive(Debug, Deserialize)]
pub struct IntentConfig {
    pub planning_keywords: Vec<String>,
    pub search_keywords: Vec<String>,
    // ... etc
}

impl Config {
    pub fn load_task_execution() -> Result<TaskExecutionConfig> {
        // Load from config/task_execution.yaml
    }
    
    pub fn load_intent_classification() -> Result<IntentConfig> {
        // Load from config/intent_classification.yaml  
    }
}
```

### 3. Refactored Code Structure

```rust
// task_executor.rs
impl TaskExecutor {
    pub fn new(cost_tracker: CostTracker, config: TaskExecutionConfig) -> Self {
        Self { 
            cost_tracker, 
            config,
            execution_log: Vec::new() 
        }
    }
    
    fn convert_action_step_to_workflow_step(&self, step: &ActionStep) -> Result<WorkflowStep> {
        // Use self.config.timeouts.step_timeout_seconds instead of hardcoded 30
        // Use self.config.timeouts.max_retries instead of hardcoded 3
    }
}

// llm_service_enhanced.rs  
impl MockTaskUnderstanding {
    pub fn new(config: IntentConfig) -> Self {
        Self { config }
    }
    
    fn classify_intent(&self, input: &str) -> Result<TaskType> {
        // Use self.config.planning_keywords instead of hardcoded array
        if self.config.planning_keywords.iter().any(|k| input.contains(k)) {
            return Ok(TaskType::Planning);
        }
        // ... etc
    }
}
```

## Implementation Priority

### High Priority (Security/Reliability)
1. **Website URLs** - Move to travel_urls.yaml
2. **Intent Keywords** - Move to intent_classification.yaml  
3. **Timeout Values** - Move to task_execution.yaml

### Medium Priority (Flexibility)
4. **Confidence Scores** - Move to task_execution.yaml
5. **Entity Lists** - Move to entities.yaml

### Low Priority (Polish)
6. **Display Formatting** - Move to task_execution.yaml
7. **File Naming** - Move to task_execution.yaml

## Benefits of Configuration Refactoring

1. **Customization**: Users can modify behavior without code changes
2. **Internationalization**: Different keyword sets for different languages
3. **Environment-Specific**: Different URLs/timeouts for dev/staging/prod
4. **A/B Testing**: Easy to modify confidence thresholds
5. **Business Rules**: Domain experts can modify classification logic
6. **Maintenance**: Changes don't require recompilation

## Migration Strategy

1. Create configuration loading infrastructure
2. Gradually move hardcoded values to config files
3. Maintain backward compatibility during transition
4. Add validation for configuration values
5. Document configuration options

This refactoring would significantly improve the code's maintainability and make it production-ready for different environments and use cases.