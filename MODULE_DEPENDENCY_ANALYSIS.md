# Module Dependency Analysis 📊

## 1. Project Structure Overview

### POC Modules (Currently Working)
```
poc/src/
├── llm_service.rs       # LLM integration (Mock & Real)
├── browser.rs           # Browser automation
├── browser_pool.rs      # Browser session management
├── api.rs              # HTTP API endpoints
├── workflow.rs         # Workflow execution
├── context.rs          # Context management
├── cache.rs            # Caching layer
├── cost_tracker.rs     # Cost tracking
├── metrics.rs          # Performance metrics
├── config.rs           # Configuration
├── security.rs         # Security features
├── extractor.rs        # Content extraction
├── plugins/            # Plugin system
└── main.rs            # Entry point
```

### Six-Organ Architecture Modules (Partially Implemented)
```
src/
├── unified_kernel/      # Central orchestration ⚠️ Stub
├── layered_perception/  # Multi-speed perception ⚠️ Stub
├── intelligent_action/  # Command execution ⚠️ Stub
├── optimized_persistence/ # Memory systems ⚠️ Stub
├── performance_engine/  # Performance optimization ⚠️ Stub
├── stability_engine/    # Error recovery ⚠️ Stub
└── [other legacy modules]
```

### Shared Services (New Bridge - Module 1 Complete ✅)
```
shared/
├── services/
│   ├── browser_service.rs  # WebDriver implementation ✅
│   └── llm_service.rs      # Mock LLM only ⚠️
├── traits.rs              # Service interfaces ✅
└── utils.rs              # Utilities ✅
```

## 2. Current Problem Analysis

### Issue: "give me a travel plan" Returns Unknown Action
**Root Cause**: The POC's `llm_service.rs` only handles basic commands in mock mode:
- ✅ Can parse: "go to [URL]", "take screenshot"
- ❌ Cannot parse: "give me a travel plan", "search for flights", "book a hotel"

### Why It Fails
1. **Mock Mode Limitation**: `parse_command_mock()` uses simple regex patterns
2. **No Task Understanding**: Cannot decompose complex requests
3. **No Action Planning**: Cannot create multi-step workflows
4. **No Real AI**: Not using actual LLM APIs (OpenAI/Claude)

## 3. Module Dependencies

### Dependency Graph
```
┌─────────────────────────────────────────┐
│         POC llm_service.rs              │ <- MODIFY THIS FIRST
│  (parse_command, generate_response)     │    (Most Independent)
└─────────────────┬───────────────────────┘
                  │ uses
┌─────────────────▼───────────────────────┐
│         POC workflow.rs                 │ <- MODIFY SECOND
│  (execute_workflow, handle_command)     │    (Depends on LLM)
└─────────────────┬───────────────────────┘
                  │ calls
┌─────────────────▼───────────────────────┐
│         POC api.rs                      │ <- MODIFY THIRD
│  (HTTP endpoints, handle_request)       │    (Depends on Workflow)
└─────────────────────────────────────────┘
```

## 4. Modules That Need Modification

### Priority 1: LLM Service (Most Independent)
**File**: `poc/src/llm_service.rs`
**Dependencies**: None (only external APIs)
**Changes Needed**:
- Add intelligent command classification
- Implement task decomposition
- Add context-aware parsing
- Support complex queries

### Priority 2: Workflow Engine
**File**: `poc/src/workflow.rs`
**Dependencies**: LLM Service
**Changes Needed**:
- Handle multi-step workflows
- Add task planning
- Support conditional execution
- Implement feedback loops

### Priority 3: API Layer
**File**: `poc/src/api.rs`
**Dependencies**: Workflow, LLM Service
**Changes Needed**:
- Support streaming responses
- Add conversation context
- Handle complex requests
- Return structured task plans

## 5. Implementation Strategy

### Step 1: Enhance LLM Service (Independent Module)
```rust
// Add to poc/src/llm_service.rs
enum TaskType {
    Navigation,      // go to URL
    Screenshot,      // take screenshot
    Search,         // search for information
    Planning,       // create plans (travel, shopping, etc.)
    Analysis,       // analyze content
    Execution,      // execute multi-step tasks
}

struct IntelligentCommand {
    task_type: TaskType,
    intent: String,
    entities: Vec<Entity>,
    steps: Vec<ActionStep>,
    context: Context,
}
```

### Step 2: Add Task Decomposition
```rust
impl LLMService {
    pub fn decompose_task(&self, input: &str) -> Result<Vec<ActionStep>> {
        match self.classify_intent(input)? {
            TaskType::Planning => self.create_plan(input),
            TaskType::Search => self.create_search_workflow(input),
            // ... other task types
        }
    }
}
```

### Step 3: Implement Intent Classification
```rust
impl LLMService {
    pub fn classify_intent(&self, input: &str) -> Result<TaskType> {
        // Use patterns or actual LLM to classify
        if input.contains("plan") || input.contains("itinerary") {
            return Ok(TaskType::Planning);
        }
        if input.contains("search") || input.contains("find") {
            return Ok(TaskType::Search);
        }
        // ... more classifications
    }
}
```

## 6. Module Independence Analysis

### Most Independent (Start Here)
1. **llm_service.rs** - No internal dependencies
2. **cost_tracker.rs** - Standalone utility
3. **metrics.rs** - Standalone monitoring

### Moderate Dependencies
4. **workflow.rs** - Depends on LLM
5. **browser.rs** - Depends on WebDriver
6. **context.rs** - Used by multiple modules

### Most Dependent (Modify Last)
7. **api.rs** - Depends on all services
8. **main.rs** - Entry point, depends on everything

## 7. Verification Plan

### After Each Module Change:
1. Run unit tests: `cargo test --lib [module_name]`
2. Run integration tests: `cargo test --test integration_tests`
3. Test specific functionality: `cargo run -- --mock-mode`
4. Verify no regression: Check existing commands still work

## 8. Next Actions

### Immediate Action: Enhance LLM Service
1. [ ] Add TaskType enum
2. [ ] Implement intent classification
3. [ ] Add task decomposition
4. [ ] Create planning functions
5. [ ] Test with "give me a travel plan"

### Success Criteria
- "give me a travel plan" returns structured steps
- Confidence score > 0.8 for complex queries
- Backward compatibility maintained
- All existing tests pass

---

**Recommendation**: Start with `poc/src/llm_service.rs` as it's the most independent module and directly addresses the intelligent task understanding problem.