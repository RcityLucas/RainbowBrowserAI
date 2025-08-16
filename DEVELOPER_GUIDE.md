# RainbowBrowserAI Developer Onboarding Guide 🚀

Welcome to the RainbowBrowserAI project! This guide will help you get up to speed quickly and start contributing effectively.

## Table of Contents
1. [Project Overview](#project-overview)
2. [Development Environment Setup](#development-environment-setup)
3. [Project Structure](#project-structure)
4. [Development Workflow](#development-workflow)
5. [Coding Standards](#coding-standards)
6. [Testing Guidelines](#testing-guidelines)
7. [Common Tasks](#common-tasks)
8. [Troubleshooting](#troubleshooting)
9. [Contributing](#contributing)

---

## Project Overview

### What is RainbowBrowserAI?

RainbowBrowserAI is an AI-powered browser automation platform that enables users to control web browsers through natural language commands. It combines:

- **WebDriver Protocol**: For browser control
- **OpenAI GPT-4**: For natural language understanding
- **Rust**: For performance and safety
- **Async/Await**: For concurrent operations

### Key Features
- Natural language command parsing
- Workflow automation (YAML/JSON)
- Browser connection pooling
- Cost tracking and budget protection
- Security middleware
- Prometheus metrics

### Architecture Overview
```
User Input → CLI/API → LLM Service → Command Parser → Browser Engine → WebDriver
                ↓                           ↓              ↓
            Workflow Engine           Security Layer   Resource Pool
                ↓                           ↓              ↓
            Metrics Collector          Rate Limiter    Cache Layer
```

---

## Development Environment Setup

### Prerequisites

1. **Rust Installation** (1.75+)
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
rustup update
```

2. **ChromeDriver**
```bash
# macOS
brew install chromedriver

# Linux
sudo apt-get install chromium-driver

# Windows
# Download from https://chromedriver.chromium.org/
```

3. **Development Tools**
```bash
# Install useful Rust tools
cargo install cargo-watch cargo-edit cargo-audit cargo-tarpaulin

# Install pre-commit hooks (optional)
pip install pre-commit
pre-commit install
```

4. **IDE Setup**

**VS Code** (Recommended):
```json
// .vscode/settings.json
{
    "rust-analyzer.cargo.features": "all",
    "rust-analyzer.checkOnSave.command": "clippy",
    "editor.formatOnSave": true,
    "editor.rulers": [100]
}
```

Extensions:
- rust-analyzer
- CodeLLDB (for debugging)
- Even Better TOML
- YAML

**IntelliJ IDEA**:
- Install Rust plugin
- Enable cargo integration
- Configure code style

### Environment Variables

Create a `.env` file in the project root:
```bash
# Required for natural language features
OPENAI_API_KEY=sk-your-api-key-here

# Optional configuration
DAILY_BUDGET=5.00
CHROME_DRIVER_URL=http://localhost:9515
RUST_LOG=debug

# For testing
TEST_URL=https://example.com
CI=false
```

### First Build

```bash
# Clone the repository
git clone https://github.com/yourusername/RainbowBrowserAI.git
cd RainbowBrowserAI/poc

# Build the project
cargo build

# Run tests to verify setup
cargo test

# Run the application
cargo run -- --help
```

---

## Project Structure

### Directory Layout
```
poc/
├── src/
│   ├── main.rs              # CLI entry point
│   ├── lib.rs               # Library exports
│   ├── browser.rs           # Browser automation
│   ├── llm_service.rs       # LLM integration
│   ├── workflow.rs          # Workflow engine
│   ├── browser_pool.rs      # Connection pooling
│   ├── cache.rs             # Caching layer
│   ├── metrics.rs           # Metrics collection
│   ├── security.rs          # Security middleware
│   ├── config.rs            # Configuration
│   ├── context.rs           # Conversation memory
│   └── cost_tracker.rs      # Budget management
├── tests/
│   └── integration_tests.rs # Integration tests
├── benches/
│   └── performance.rs       # Benchmarks
├── workflows/
│   └── templates/           # Workflow templates
├── Cargo.toml               # Dependencies
├── Dockerfile               # Container definition
└── docker-compose.yml       # Stack definition
```

### Module Responsibilities

| Module | Purpose | Key Types |
|--------|---------|-----------|
| `browser` | WebDriver control | `SimpleBrowser`, `ScreenshotOptions` |
| `llm_service` | Natural language | `LLMService`, `ParsedCommand` |
| `workflow` | Automation engine | `WorkflowEngine`, `Workflow` |
| `browser_pool` | Resource pooling | `BrowserPool`, `PooledBrowserHandle` |
| `cache` | Caching layer | `Cache<K,V>`, `LlmCache` |
| `metrics` | Performance tracking | `MetricsCollector`, `Metrics` |
| `security` | Input validation | `SecurityMiddleware`, `RateLimiter` |
| `config` | Configuration | `Config`, `BrowserConfig` |

---

## Development Workflow

### 1. Feature Development

```bash
# Create a feature branch
git checkout -b feature/your-feature-name

# Watch for changes and auto-rebuild
cargo watch -x build

# Run tests continuously
cargo watch -x test

# Check code quality
cargo clippy
cargo fmt
```

### 2. Testing Workflow

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_browser_navigation

# Run tests with output
cargo test -- --nocapture

# Run integration tests only
cargo test --test integration_tests

# Generate coverage report
cargo tarpaulin --out Html
```

### 3. Debugging

**Using VS Code**:
1. Set breakpoints in code
2. Press F5 to start debugging
3. Use Debug Console for expressions

**Using print debugging**:
```rust
use tracing::{debug, info, warn, error};

debug!("Variable value: {:?}", variable);
info!("Operation completed");
warn!("Potential issue: {}", message);
error!("Error occurred: {}", error);
```

**Enable debug logging**:
```bash
RUST_LOG=debug cargo run
```

### 4. Performance Profiling

```bash
# Run benchmarks
cargo bench

# Profile with flamegraph
cargo install flamegraph
cargo flamegraph --bin rainbow-poc

# Check binary size
cargo bloat --release
```

---

## Coding Standards

### Rust Style Guide

1. **Naming Conventions**
```rust
// Modules: snake_case
mod browser_pool;

// Types: PascalCase
struct BrowserConfig;
enum ActionType;

// Functions: snake_case
fn parse_command() {}

// Constants: SCREAMING_SNAKE_CASE
const MAX_RETRIES: u32 = 3;

// Variables: snake_case
let user_input = "example";
```

2. **Error Handling**
```rust
// Use Result<T> for fallible operations
fn risky_operation() -> Result<String> {
    // Use ? for error propagation
    let data = fetch_data()?;
    
    // Add context to errors
    process_data(data)
        .context("Failed to process data")?;
    
    Ok("success".to_string())
}

// Never use unwrap() in production code
// Use expect() only with clear messages
let config = Config::load()
    .expect("Config file must exist");
```

3. **Async Best Practices**
```rust
// Always use async/await for I/O
async fn fetch_data() -> Result<Data> {
    // ...
}

// Use tokio::join! for parallel operations
let (result1, result2) = tokio::join!(
    operation1(),
    operation2()
);

// Use Arc for shared state
let shared = Arc::new(RwLock::new(State::new()));
```

4. **Documentation**
```rust
/// Brief description of the function.
///
/// # Arguments
///
/// * `param` - Description of parameter
///
/// # Returns
///
/// Description of return value
///
/// # Errors
///
/// Description of error conditions
///
/// # Examples
///
/// ```
/// let result = function(param)?;
/// ```
pub fn function(param: &str) -> Result<String> {
    // Implementation
}
```

### Code Quality Checklist

- [ ] No compiler warnings
- [ ] Passes `cargo clippy`
- [ ] Formatted with `cargo fmt`
- [ ] Documented public APIs
- [ ] Includes tests
- [ ] No `unwrap()` in production code
- [ ] Handles all error cases
- [ ] Uses appropriate data structures
- [ ] Avoids unnecessary allocations
- [ ] Thread-safe where needed

---

## Testing Guidelines

### Test Organization

```rust
// Unit tests in the same file
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_unit_functionality() {
        // Test implementation
    }
}

// Integration tests in tests/ directory
// tests/integration_tests.rs
#[tokio::test]
async fn test_full_workflow() {
    // Integration test
}
```

### Writing Good Tests

1. **Test Naming**
```rust
#[test]
fn test_parse_valid_url() {} // Good: descriptive
#[test]
fn test1() {} // Bad: not descriptive
```

2. **Test Structure (AAA)**
```rust
#[test]
fn test_cache_eviction() {
    // Arrange
    let cache = Cache::new(Duration::from_secs(60), 3);
    
    // Act
    cache.insert(1, "one");
    cache.insert(2, "two");
    cache.insert(3, "three");
    cache.insert(4, "four"); // Should evict oldest
    
    // Assert
    assert_eq!(cache.get(&1), None);
    assert_eq!(cache.get(&4), Some("four"));
}
```

3. **Async Tests**
```rust
#[tokio::test]
async fn test_browser_navigation() {
    let browser = SimpleBrowser::new().await.unwrap();
    browser.navigate_to("https://example.com").await.unwrap();
    let title = browser.get_title().await.unwrap();
    assert!(title.contains("Example"));
}
```

### Test Coverage Goals

- Unit tests: 80% coverage
- Integration tests: Critical paths
- Performance tests: Benchmarks for hot paths
- Security tests: Input validation

---

## Common Tasks

### Adding a New Feature

1. **Create the module**
```bash
touch src/new_feature.rs
```

2. **Add to lib.rs**
```rust
pub mod new_feature;
pub use new_feature::NewFeature;
```

3. **Implement the feature**
```rust
// src/new_feature.rs
use anyhow::Result;

pub struct NewFeature {
    // Fields
}

impl NewFeature {
    pub fn new() -> Self {
        Self {
            // Initialize
        }
    }
    
    pub async fn do_something(&self) -> Result<()> {
        // Implementation
        Ok(())
    }
}
```

4. **Add tests**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_new_feature() {
        let feature = NewFeature::new();
        assert!(feature.do_something().await.is_ok());
    }
}
```

### Adding a New Workflow Action

1. **Update ActionType enum**
```rust
// src/workflow.rs
pub enum ActionType {
    // Existing actions...
    NewAction,
}
```

2. **Implement action handler**
```rust
impl WorkflowEngine {
    async fn execute_new_action(&mut self, params: &Value) -> Result<Value> {
        // Implementation
    }
}
```

3. **Update execute_action**
```rust
match action_type {
    // Existing cases...
    ActionType::NewAction => self.execute_new_action(params).await,
}
```

### Adding a New CLI Command

1. **Update Commands enum**
```rust
// src/main.rs
#[derive(Subcommand)]
enum Commands {
    // Existing commands...
    NewCommand {
        #[arg(short, long)]
        param: String,
    },
}
```

2. **Handle the command**
```rust
match args.command {
    // Existing cases...
    Commands::NewCommand { param } => {
        execute_new_command(&param).await
    }
}
```

---

## Troubleshooting

### Common Issues and Solutions

#### ChromeDriver Connection Failed
```
Error: Failed to connect to ChromeDriver
```
**Solution**:
```bash
# Start ChromeDriver
chromedriver --port=9515

# Or use Docker
docker run -p 9515:9515 selenium/standalone-chrome
```

#### OpenAI API Key Missing
```
Error: OPENAI_API_KEY not set
```
**Solution**:
```bash
export OPENAI_API_KEY="sk-your-key"
# Or add to .env file
```

#### Compilation Errors

**Lifetime errors**:
```rust
// Problem
fn process(&self, data: &str) -> &str {
    &self.internal_process(data) // Returns reference to temporary
}

// Solution
fn process(&self, data: &str) -> String {
    self.internal_process(data) // Return owned value
}
```

**Async recursion**:
```rust
// Problem
async fn recursive(&self) {
    self.recursive().await; // Error: recursion in async fn
}

// Solution
fn recursive(&self) -> Pin<Box<dyn Future<Output = ()>>> {
    Box::pin(async move {
        self.recursive().await;
    })
}
```

#### Test Failures

**Async test timeout**:
```rust
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_with_timeout() {
    tokio::time::timeout(
        Duration::from_secs(5),
        async_operation()
    ).await.unwrap();
}
```

**Resource cleanup**:
```rust
#[tokio::test]
async fn test_with_cleanup() {
    let browser = SimpleBrowser::new().await.unwrap();
    
    // Test implementation
    
    // Always clean up
    let _ = browser.close().await;
}
```

### Debug Commands

```bash
# Check for outdated dependencies
cargo outdated

# Audit for security vulnerabilities
cargo audit

# Clean build artifacts
cargo clean

# Check for unused dependencies
cargo machete

# Format code
cargo fmt

# Lint code
cargo clippy -- -D warnings

# Generate documentation
cargo doc --open
```

---

## Contributing

### Contribution Process

1. **Fork the repository**
2. **Create a feature branch**
3. **Make your changes**
4. **Add tests**
5. **Update documentation**
6. **Submit a pull request**

### Pull Request Checklist

- [ ] Code compiles without warnings
- [ ] All tests pass
- [ ] Added/updated tests for new functionality
- [ ] Updated documentation
- [ ] Ran `cargo fmt` and `cargo clippy`
- [ ] Updated CHANGELOG.md
- [ ] Descriptive commit messages
- [ ] PR description explains changes

### Commit Message Format

```
type(scope): brief description

Longer explanation of the change if necessary. Wrap at 72 characters.

Fixes #123
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation
- `style`: Formatting
- `refactor`: Code restructuring
- `test`: Tests
- `chore`: Maintenance

### Code Review Guidelines

**For Reviewers**:
- Check for correctness
- Verify test coverage
- Ensure documentation
- Look for performance issues
- Check error handling
- Verify security considerations

**For Authors**:
- Respond to all comments
- Update PR based on feedback
- Keep changes focused
- Rebase on main if needed

---

## Resources

### Internal Documentation
- [API Documentation](poc/API_DOCUMENTATION.md)
- [User Guide](poc/USER_GUIDE.md)
- [Assessment Report](poc/ASSESSMENT_REPORT.md)
- [Development Progress](DEVELOPMENT_PROGRESS.md)

### External Resources
- [Rust Book](https://doc.rust-lang.org/book/)
- [Async Book](https://rust-lang.github.io/async-book/)
- [WebDriver Protocol](https://www.w3.org/TR/webdriver/)
- [OpenAI API](https://platform.openai.com/docs/)
- [Tokio Documentation](https://tokio.rs/)

### Community
- GitHub Issues: Bug reports and features
- Discussions: Questions and ideas
- Discord: Real-time chat (if available)

---

## Quick Reference

### Build Commands
```bash
cargo build              # Debug build
cargo build --release    # Release build
cargo test              # Run tests
cargo bench             # Run benchmarks
cargo doc --open        # Generate docs
cargo run -- <args>     # Run application
```

### Environment Variables
```bash
OPENAI_API_KEY          # OpenAI API key
DAILY_BUDGET            # Cost limit
CHROME_DRIVER_URL       # WebDriver URL
RUST_LOG               # Log level
```

### Key Files
```
Cargo.toml             # Dependencies
src/lib.rs             # Public API
src/main.rs            # CLI entry
config.yaml            # Configuration
.env                   # Environment vars
```

---

Welcome to the team! If you have questions, check the documentation or ask in the community channels. Happy coding! 🚀