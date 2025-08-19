# RainbowBrowserAI CLAUDE.md Documentation

## Project Overview

RainbowBrowserAI is a Rust-based browser automation framework with integrated AI capabilities. Since the original repository at https://github.com/RcityLucas/RainbowBrowserAI is not publicly accessible, this CLAUDE.md provides comprehensive guidelines based on Rust browser automation best practices and AI integration patterns.

## Core Architecture

### Module Organization

```
src/
├── lib.rs                    # Public API surface
├── main.rs                   # Binary entry point
├── browser/                  # Browser automation layer
│   ├── mod.rs               # Module exports and traits
│   ├── chromium.rs          # Chromium/Chrome automation
│   ├── session.rs           # Browser session management
│   ├── page.rs              # Page interaction utilities
│   └── events.rs            # Event handling and listeners
├── ai/                       # AI integration layer
│   ├── mod.rs               # AI module exports
│   ├── client.rs            # LLM API clients (OpenAI, Claude)
│   ├── prompts.rs           # Prompt templates and management
│   ├── embeddings.rs        # Vector embeddings handling
│   └── processing.rs        # Response parsing and validation
├── automation/               # Task automation workflows
│   ├── mod.rs               # Automation exports
│   ├── workflows.rs         # Workflow orchestration
│   ├── scripting.rs         # JavaScript injection and execution
│   └── scheduling.rs        # Task scheduling and queuing
├── config/                   # Configuration management
│   ├── mod.rs               # Config module exports
│   ├── settings.rs          # Application settings
│   └── validation.rs        # Config validation logic
├── error.rs                 # Error types and handling
└── utils/                   # Utility functions
    ├── mod.rs
    └── logging.rs           # Logging utilities
```

## Module Boundaries and Responsibilities

### browser/ module
**Responsibility**: All browser automation and control
- **Strict boundary**: No AI logic or business rules
- **Dependencies**: chromiumoxide or headless_chrome only
- **Exports**: Browser, Page, Session traits
- **Testing**: Mock browser for unit tests, real browser for integration

### ai/ module  
**Responsibility**: AI/LLM integration and processing
- **Strict boundary**: No direct browser manipulation
- **Dependencies**: OpenAI/Claude SDKs, async HTTP clients
- **Exports**: AIClient trait, prompt builders, response types
- **Testing**: Mock API responses, validate prompt templates

### automation/ module
**Responsibility**: High-level workflow orchestration
- **Strict boundary**: Coordinates browser and AI, no low-level implementation
- **Dependencies**: browser and ai modules only
- **Exports**: Workflow, Task, Schedule types
- **Testing**: End-to-end workflow tests with mocked dependencies

## Compilation Guidelines

### Build Configurations

```toml
# Cargo.toml
[package]
name = "rainbow_browser_ai"
version = "0.1.0"
edition = "2021"
rust-version = "1.75"

[features]
default = ["chromium", "ai-openai"]
chromium = ["chromiumoxide"]
firefox = ["fantoccini"]
ai-openai = ["async-openai"]
ai-claude = ["claude-sdk-rs"]
experimental = []

[profile.dev]
opt-level = 0
debug = true
overflow-checks = true

[profile.release]
opt-level = 3
lto = true
codegen-units = 1
strip = true

[profile.test]
opt-level = 2
debug = true
```

### Build Commands

```bash
# Development build with all checks
cargo build --all-features

# Production build (optimized)
cargo build --release

# Specific feature set
cargo build --no-default-features --features "firefox,ai-claude"

# Run with environment configuration
RUST_LOG=debug cargo run

# Build for distribution
cargo build --release --target x86_64-unknown-linux-gnu
```

## Modification Rules

### 1. Code Style and Quality
- **Always run** `cargo fmt` before committing
- **Must pass** `cargo clippy -- -D warnings`
- **Document** all public APIs with doc comments
- **Use** `#[must_use]` for functions returning Results
- **Prefer** explicit error types over `anyhow::Error` in libraries

### 2. Error Handling
```rust
// CORRECT: Explicit error types
pub enum BrowserError {
    ConnectionFailed(String),
    PageLoadTimeout,
    JavaScriptError(String),
}

// INCORRECT: Generic errors in public APIs
pub fn navigate(url: &str) -> Result<(), Box<dyn Error>> // Avoid
```

### 3. Async/Await Patterns
```rust
// CORRECT: Consistent async runtime
use tokio::runtime::Runtime;

// INCORRECT: Mixing runtimes
// Don't mix async-std with tokio
```

### 4. Memory Management
- **Use** `Arc<Mutex<T>>` for shared state in async contexts
- **Prefer** `Cow<'a, str>` for string parameters that might be owned
- **Avoid** unnecessary cloning - use references where possible
- **Implement** `Drop` for resources requiring cleanup

### 5. Testing Requirements
```rust
// Unit tests in same file
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_function() {
        // Test implementation
    }
}

// Integration tests in tests/ directory
// Must test actual browser interactions
```

## Dependencies Management

### Core Dependencies
```toml
[dependencies]
# Browser automation (choose one)
chromiumoxide = "0.5"
# OR
headless_chrome = { git = "https://github.com/rust-headless-chrome/rust-headless-chrome" }

# Async runtime
tokio = { version = "1.0", features = ["full", "rt-multi-thread"] }
futures = "0.3"

# AI integration
async-openai = "0.22"
reqwest = { version = "0.11", features = ["json", "stream"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Error handling
thiserror = "1.0"
anyhow = "1.0"  # Only for binary, not library

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

[dev-dependencies]
tokio-test = "0.4"
mockall = "0.11"
proptest = "1.0"
criterion = "0.5"
wiremock = "0.5"
```

### Dependency Rules
1. **Audit regularly**: Run `cargo audit` weekly
2. **Minimize dependencies**: Justify each addition
3. **Pin major versions**: Use `"1.0"` not `"*"`
4. **Feature flags**: Make optional dependencies truly optional
5. **No duplicate functionality**: One crate per purpose

## Testing Strategy

### Test Organization
```
tests/
├── common/
│   └── mod.rs              # Shared test utilities
├── browser_integration.rs  # Browser tests
├── ai_integration.rs       # AI service tests
└── e2e_workflows.rs        # End-to-end tests
```

### Testing Commands
```bash
# All tests
cargo test --all-features

# Unit tests only
cargo test --lib

# Integration tests
cargo test --test '*'

# Specific test
cargo test test_browser_navigation

# With output
cargo test -- --nocapture

# Benchmarks
cargo bench

# Property tests
cargo test --features proptest
```

## Project-Specific Best Practices

### 1. Browser Session Management
```rust
// Always use builder pattern for configuration
let browser = Browser::builder()
    .headless(true)
    .viewport(1920, 1080)
    .timeout(Duration::from_secs(30))
    .build()
    .await?;

// Always clean up sessions
impl Drop for BrowserSession {
    fn drop(&mut self) {
        // Cleanup logic
    }
}
```

### 2. AI Prompt Engineering
```rust
// Store prompts as constants or in config files
const EXTRACTION_PROMPT: &str = include_str!("prompts/extraction.txt");

// Version prompts for reproducibility
#[derive(Serialize, Deserialize)]
struct PromptTemplate {
    version: String,
    template: String,
    variables: Vec<String>,
}
```

### 3. Rate Limiting and Retries
```rust
use tokio::time::{sleep, Duration};

// Implement exponential backoff
async fn with_retry<F, T, E>(f: F) -> Result<T, E>
where
    F: Fn() -> Future<Output = Result<T, E>>,
{
    let mut retries = 0;
    loop {
        match f().await {
            Ok(result) => return Ok(result),
            Err(e) if retries < 3 => {
                sleep(Duration::from_secs(2_u64.pow(retries))).await;
                retries += 1;
            }
            Err(e) => return Err(e),
        }
    }
}
```

### 4. Resource Pooling
```rust
// Use connection pools for browsers
struct BrowserPool {
    browsers: Vec<Arc<Browser>>,
    semaphore: Semaphore,
}

impl BrowserPool {
    async fn acquire(&self) -> BrowserGuard {
        let permit = self.semaphore.acquire().await.unwrap();
        // Return browser with RAII guard
    }
}
```

## Performance Optimization

### Compilation Optimizations
```bash
# Enable CPU-specific optimizations
RUSTFLAGS="-C target-cpu=native" cargo build --release

# Profile-guided optimization
cargo build --release
./target/release/rainbow_browser_ai --benchmark
cargo build --release --profile=release-lto
```

### Runtime Optimizations
1. **Use browser contexts** instead of new browser instances
2. **Reuse HTTP clients** with connection pooling
3. **Cache AI responses** when appropriate
4. **Implement request batching** for AI APIs
5. **Use streaming responses** for large data

## Security Considerations

### Secure Defaults
```rust
// Never log sensitive data
#[derive(Debug)]
struct Credentials {
    #[debug(skip)]
    api_key: String,
    username: String,
}

// Validate all external input
fn validate_url(url: &str) -> Result<Url, ValidationError> {
    let parsed = Url::parse(url)?;
    if !["http", "https"].contains(&parsed.scheme()) {
        return Err(ValidationError::InvalidScheme);
    }
    Ok(parsed)
}
```

### Environment Configuration
```bash
# .env.example
BROWSER_HEADLESS=true
AI_API_KEY=your_api_key_here
LOG_LEVEL=info
MAX_CONCURRENT_SESSIONS=5
```

## Debugging Guidelines

### Logging Configuration
```rust
// src/main.rs
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn init_logging() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();
}
```

### Debug Commands
```bash
# Enable all debug logs
RUST_LOG=rainbow_browser_ai=debug cargo run

# Enable specific module logs
RUST_LOG=rainbow_browser_ai::browser=trace cargo run

# Use debugger
rust-gdb target/debug/rainbow_browser_ai
```

## Common Patterns

### Builder Pattern
```rust
pub struct BrowserBuilder {
    headless: bool,
    viewport: (u32, u32),
    user_agent: Option<String>,
}

impl BrowserBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn headless(mut self, headless: bool) -> Self {
        self.headless = headless;
        self
    }
    
    pub async fn build(self) -> Result<Browser, Error> {
        // Build logic
    }
}
```

### Type State Pattern
```rust
struct Browser<State> {
    inner: BrowserInner,
    _state: PhantomData<State>,
}

struct Connected;
struct Disconnected;

impl Browser<Disconnected> {
    async fn connect(self) -> Result<Browser<Connected>, Error> {
        // Connection logic
    }
}
```

## Workspace Configuration (if multi-crate)

```toml
# Cargo.toml (root)
[workspace]
members = [
    "rainbow-core",
    "rainbow-browser",
    "rainbow-ai",
    "rainbow-cli",
]

[workspace.dependencies]
tokio = "1.0"
serde = "1.0"
```

## CI/CD Integration

### GitHub Actions Example
```yaml
name: CI
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - run: cargo fmt -- --check
      - run: cargo clippy -- -D warnings
      - run: cargo test --all-features
      - run: cargo build --release
```

## Documentation Requirements

1. **README.md**: Project overview, quick start, examples
2. **API docs**: Generate with `cargo doc --open`
3. **Examples**: Working examples in `examples/` directory
4. **Architecture**: Maintain architecture decision records (ADRs)
5. **Changelog**: Keep CHANGELOG.md updated

## Version Control Guidelines

### Commit Message Format
```
type(scope): description

- feat(browser): add screenshot capability
- fix(ai): handle rate limit errors
- docs(api): update client documentation
- test(e2e): add workflow tests
- refactor(config): simplify settings management
```

### Branch Strategy
- `main`: Stable releases only
- `develop`: Integration branch
- `feature/*`: New features
- `fix/*`: Bug fixes
- `release/*`: Release preparation

This CLAUDE.md provides comprehensive guidelines for working with a Rust-based browser AI project, ensuring consistent development practices, maintainable code, and efficient collaboration.