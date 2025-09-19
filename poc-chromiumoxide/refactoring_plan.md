# RainbowBrowserAI Refactoring Plan

## Phase 1: Preparation (Day 1)

### 1.1 Create Backup
```bash
# Create a backup branch
git checkout -b backup-before-refactor
git add .
git commit -m "Backup before major refactoring"
git checkout -b refactoring-work
```

### 1.2 Document Current API
Create a file documenting all current API endpoints and their actual usage:
- Which endpoints are being called by the frontend
- Which endpoints are never used
- Which endpoints duplicate functionality

### 1.3 Identify Core Features
Mark features as:
- ✅ **Essential**: Currently used and needed
- ⚠️ **Redundant**: Duplicate implementation
- ❌ **Dead**: Never used, safe to remove

## Phase 2: Remove Dead Code (Day 1-2)

### 2.1 Clean Unused Imports and Variables
```bash
# Auto-fix what we can
cargo fix --allow-dirty --allow-staged
cargo clippy --fix --allow-dirty --allow-staged
```

### 2.2 Remove Unused Error Variants
Search for error types with #[allow(dead_code)] and remove unused variants:
- `PerceptionError`: Keep only InvalidMode, ValidationError
- `LLMApiError`: Keep only the ones actually thrown
- Remove entire unused error types

### 2.3 Delete Never-Constructed Structs
Remove structs that are never instantiated:
- `UnifiedErrorHandler`
- `CircuitBreaker`
- Many in coordination module

## Phase 3: Consolidate Modules (Day 2-3)

### 3.1 Choose Primary Implementations

**Browser Management:**
- Keep: `src/browser/` (primary implementation)
- Remove: Browser management from coordination

**Perception:**
- Keep: `src/perception/` (original, working implementation)
- Remove: `src/coordination/perception_impl.rs`

**Tools:**
- Keep: `src/tools/registry.rs` (comprehensive, working)
- Remove: `src/coordination/tools_impl.rs`
- Remove: `CoordinatedToolRegistry`

**Intelligence:**
- Keep: `src/intelligence/` (if used)
- Remove: `src/coordination/intelligence_impl.rs`

### 3.2 Merge Coordination Features (if any unique ones exist)
Before deleting coordination module, check for unique features:
- Session management → Move to `src/session/`
- Event system → Move to `src/events/` if needed
- Caching → Keep central cache in `src/cache/`

## Phase 4: Simplify API Layer (Day 3)

### 4.1 Consolidate Endpoints

**Before:** Multiple perception endpoints
```
/api/perception/analyze-page
/api/perception/perceive  
/api/perception/navigate-and-perceive
/api/perception/quick-scan
```

**After:** Single unified endpoint
```
POST /api/perception
{
    "action": "analyze|navigate|scan",
    "mode": "lightning|quick|standard|deep",
    "url": "optional",
    "session_id": "optional"
}
```

### 4.2 Remove Duplicate Handlers
Keep only one handler per feature type.

## Phase 5: Restructure Project (Day 4)

### 5.1 Target Structure
```
src/
├── main.rs                 # Entry point
├── lib.rs                  # Public API
├── api/                    # HTTP handlers (thin layer)
│   ├── mod.rs
│   ├── routes.rs          # Route definitions
│   └── handlers.rs        # Consolidated handlers
├── browser/               # Browser management
│   ├── mod.rs
│   ├── pool.rs           # Browser pool
│   └── session.rs         # Session management (moved from coordination)
├── perception/            # Perception engine
│   ├── mod.rs
│   ├── engine.rs
│   ├── layered.rs
│   └── modes.rs
├── tools/                 # Tool system
│   ├── mod.rs
│   ├── registry.rs        # Single registry
│   ├── traits.rs
│   └── [individual tools]
├── intelligence/          # AI features (if needed)
│   ├── mod.rs
│   └── engine.rs
├── error.rs               # Consolidated error types
└── utils/                 # Shared utilities
```

### 5.2 Delete Entire Coordination Module
```bash
# After extracting any unique features
rm -rf src/coordination/
```

## Phase 6: Implementation Steps (Day 4-5)

### 6.1 Create New Session Module
```rust
// src/session/mod.rs
pub struct SessionManager {
    sessions: Arc<RwLock<HashMap<String, Session>>>,
    browser_pool: Arc<BrowserPool>,
}

pub struct Session {
    pub id: String,
    pub browser: Arc<Browser>,
    pub created_at: Instant,
    pub last_accessed: Instant,
}
```

### 6.2 Simplify Browser Pool
```rust
// src/browser/pool.rs
pub struct BrowserPool {
    browsers: Vec<Arc<Browser>>,
    available: Arc<Mutex<VecDeque<usize>>>,
}
```

### 6.3 Single Tool Registry
```rust
// src/tools/registry.rs - Keep existing, remove duplicates
impl ToolRegistry {
    pub fn new(browser: Arc<Browser>) -> Self { ... }
    pub fn execute(&self, name: &str, params: Value) -> Result<Value> { ... }
}
```

## Phase 7: Update API Handlers (Day 5)

### 7.1 Consolidate Handlers
```rust
// src/api/handlers.rs
pub async fn perception_handler(
    State(state): State<AppState>,
    Json(req): Json<PerceptionRequest>,
) -> Result<Json<PerceptionResponse>> {
    // Single handler for all perception operations
    match req.action {
        Action::Analyze => analyze(state, req).await,
        Action::Navigate => navigate_and_analyze(state, req).await,
        Action::Scan => quick_scan(state, req).await,
    }
}
```

### 7.2 Simplify State
```rust
pub struct AppState {
    browser_pool: Arc<BrowserPool>,
    session_manager: Arc<SessionManager>,
    tool_registry: Arc<ToolRegistry>,
    // Remove duplicate managers
}
```

## Phase 8: Testing and Validation (Day 6)

### 8.1 Test Core Functions
```bash
# Run existing tests
cargo test

# Test each endpoint
curl -X POST http://localhost:3001/api/perception \
  -H "Content-Type: application/json" \
  -d '{"action": "analyze", "mode": "quick", "url": "https://example.com"}'
```

### 8.2 Verify No Functionality Lost
- Compare API responses before/after
- Ensure all used features still work
- Check performance improvements

## Phase 9: Cleanup (Day 6)

### 9.1 Final Cleanup
```bash
# Remove unused dependencies from Cargo.toml
cargo machete

# Format code
cargo fmt

# Final clippy check
cargo clippy -- -W clippy::all

# Check for remaining warnings
cargo build 2>&1 | grep warning | wc -l
# Should be < 10 (down from 260+)
```

### 9.2 Update Documentation
- Update README with new structure
- Document API changes
- Update examples

## Expected Outcomes

### Before:
- 260+ warnings
- 11+ modules
- 3 tool registries
- Multiple perception engines
- Duplicate API endpoints
- ~50% dead code

### After:
- < 10 warnings
- 5-6 focused modules
- 1 tool registry
- 1 perception engine
- Consolidated API
- ~90% active code

### Benefits:
- 50% faster compilation
- 30% smaller binary
- Easier to maintain
- Clear module boundaries
- Better performance
- Reduced confusion

## Rollback Plan

If issues arise:
```bash
git checkout backup-before-refactor
```

## Timeline

- **Day 1**: Backup, analyze, remove dead code
- **Day 2-3**: Consolidate modules
- **Day 3-4**: Restructure project
- **Day 5**: Update API and test
- **Day 6**: Final cleanup and documentation

Total estimated time: 5-6 days of focused work