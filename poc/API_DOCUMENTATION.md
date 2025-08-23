# RainbowBrowserAI API Documentation 📚

## Table of Contents
1. [Core Modules](#core-modules)
2. [Browser Automation API](#browser-automation-api)
3. [LLM Service API](#llm-service-api)
4. [Workflow Engine API](#workflow-engine-api)
5. [Resource Management APIs](#resource-management-apis)
6. [Security API](#security-api)
7. [Configuration API](#configuration-api)
8. [Error Types](#error-types)
9. [Examples](#examples)

---

## Core Modules

### SimpleBrowser

The main browser automation interface.

```rust
pub struct SimpleBrowser {
    driver: WebDriver,
    config: BrowserConfig,
}
```

#### Methods

##### `new() -> Result<Self>`
Creates a new browser instance with default configuration.

```rust
let browser = SimpleBrowser::new().await?;
```

##### `new_with_config(retries: u32, timeout: Duration) -> Result<Self>`
Creates a browser with custom retry and timeout settings.

```rust
let browser = SimpleBrowser::new_with_config(3, Duration::from_secs(30)).await?;
```

##### `navigate_to(&self, url: &str) -> Result<()>`
Navigates to the specified URL.

```rust
browser.navigate_to("https://example.com").await?;
```

##### `navigate_to_with_retry(&self, url: &str, max_retries: u32) -> Result<()>`
Navigates with automatic retry on failure.

```rust
browser.navigate_to_with_retry("https://example.com", 3).await?;
```

##### `take_screenshot(&self, filename: &str) -> Result<()>`
Takes a full-page screenshot.

```rust
browser.take_screenshot("screenshot.png").await?;
```

##### `take_screenshot_with_options(&self, filename: &str, options: &ScreenshotOptions) -> Result<()>`
Takes a screenshot with custom options.

```rust
let options = ScreenshotOptions {
    full_page: false,
    viewport_width: 1920,
    viewport_height: 1080,
    wait_after_load: Duration::from_secs(2),
};
browser.take_screenshot_with_options("screenshot.png", &options).await?;
```

##### `get_title(&self) -> Result<String>`
Gets the current page title.

```rust
let title = browser.get_title().await?;
```

##### `get_url(&self) -> Result<String>`
Gets the current URL.

```rust
let url = browser.get_url().await?;
```

##### `click(&self, selector: &str) -> Result<()>`
Clicks an element by CSS selector.

```rust
browser.click("button#submit").await?;
```

##### `fill_field(&self, selector: &str, value: &str) -> Result<()>`
Fills a form field with the specified value.

```rust
browser.fill_field("input#username", "user@example.com").await?;
```

##### `get_text(&self, selector: &str) -> Result<String>`
Gets the text content of an element.

```rust
let text = browser.get_text("h1.title").await?;
```

##### `wait_for_element(&self, selector: &str, timeout: Duration) -> Result<()>`
Waits for an element to appear.

```rust
browser.wait_for_element("#results", Duration::from_secs(10)).await?;
```

##### `execute_script(&self, script: &str) -> Result<serde_json::Value>`
Executes JavaScript in the browser.

```rust
let result = browser.execute_script("return document.title").await?;
```

##### `close(self) -> Result<()>`
Closes the browser and cleans up resources.

```rust
browser.close().await?;
```

---

## LLM Service API

### LLMService

Interface for natural language processing with OpenAI GPT models.

```rust
pub struct LLMService {
    client: Client,
    api_key: String,
}
```

#### Methods

##### `new(api_key: String) -> Self`
Creates a new LLM service instance.

```rust
let llm = LLMService::new("sk-...".to_string());
```

##### `parse_natural_command(&self, command: &str, cost_tracker: &mut CostTracker) -> Result<ParsedCommand>`
Parses a natural language command into structured format.

```rust
let parsed = llm.parse_natural_command("go to google", &mut tracker).await?;
```

##### `explain_command(&self, command: &ParsedCommand) -> String`
Generates a human-readable explanation of a parsed command.

```rust
let explanation = llm.explain_command(&parsed).await;
```

### ParsedCommand

Structured representation of a parsed natural language command.

```rust
pub struct ParsedCommand {
    pub action: String,           // Primary action (navigate, test, etc.)
    pub urls: Vec<String>,        // Target URLs
    pub parameters: CommandParams, // Additional parameters
    pub confidence: f32,          // Confidence score (0.0-1.0)
}
```

### CommandParams

Additional parameters extracted from natural language.

```rust
pub struct CommandParams {
    pub take_screenshot: bool,
    pub screenshot_filename: Option<String>,
    pub viewport_width: Option<u32>,
    pub viewport_height: Option<u32>,
    pub retries: Option<u32>,
    pub timeout_seconds: Option<u64>,
    pub show_report: bool,
}
```

---

## Workflow Engine API

### WorkflowEngine

Executes multi-step automation workflows.

```rust
pub struct WorkflowEngine {
    browser: Option<SimpleBrowser>,
    variables: Arc<RwLock<HashMap<String, serde_json::Value>>>,
    state: WorkflowState,
}
```

#### Methods

##### `new() -> Self`
Creates a new workflow engine.

```rust
let engine = WorkflowEngine::new();
```

##### `execute(&mut self, workflow: &Workflow) -> Result<WorkflowResult>`
Executes a complete workflow.

```rust
let result = engine.execute(&workflow).await?;
```

##### `set_variable(&self, name: &str, value: serde_json::Value)`
Sets a workflow variable.

```rust
engine.set_variable("username", json!("user@example.com")).await;
```

##### `get_variable(&self, name: &str) -> Option<serde_json::Value>`
Gets a workflow variable value.

```rust
let value = engine.get_variable("result").await;
```

##### `render_template(&self, template: &str) -> Result<String>`
Renders a template string with variables.

```rust
let rendered = engine.render_template("Hello {{name}}").await?;
```

### Workflow

Workflow definition structure.

```rust
pub struct Workflow {
    pub name: String,
    pub description: Option<String>,
    pub inputs: Vec<WorkflowInput>,
    pub steps: Vec<WorkflowStep>,
    pub on_error: Option<ErrorStrategy>,
}
```

### WorkflowStep

Individual workflow step.

```rust
pub struct WorkflowStep {
    pub name: String,
    pub action: serde_json::Value,
    pub store_as: Option<String>,
    pub condition: Option<Condition>,
    pub on_error: Option<ErrorStrategy>,
}
```

### ActionType

Available workflow actions.

```rust
pub enum ActionType {
    Navigate,      // Navigate to URL
    Click,         // Click element
    Fill,          // Fill form field
    Extract,       // Extract data
    Wait,          // Wait for condition
    Assert,        // Assert condition
    Loop,          // Loop over items
    Conditional,   // Conditional execution
    Script,        // Execute JavaScript
    Parallel,      // Parallel execution
}
```

---

## Resource Management APIs

### BrowserPool

Manages a pool of browser connections for efficiency.

```rust
pub struct BrowserPool {
    max_size: usize,
    idle_timeout: Duration,
    max_lifetime: Duration,
    max_usage: usize,
}
```

#### Methods

##### `new() -> Self`
Creates a pool with default settings.

```rust
let pool = BrowserPool::new();
```

##### `with_config(max_size: usize, idle_timeout: Duration, max_lifetime: Duration, max_usage: usize) -> Self`
Creates a pool with custom configuration.

```rust
let pool = BrowserPool::with_config(5, Duration::from_secs(300), Duration::from_secs(3600), 100);
```

##### `acquire(&self) -> Result<PooledBrowserHandle>`
Acquires a browser from the pool.

```rust
let handle = pool.acquire().await?;
let browser = handle.browser().unwrap();
```

##### `stats(&self) -> PoolStats`
Gets pool statistics.

```rust
let stats = pool.stats().await;
println!("Active browsers: {}", stats.current_size);
```

##### `clear(&self) -> Result<()>`
Clears all browsers from the pool.

```rust
pool.clear().await?;
```

### Cache

Generic caching implementation with TTL and LRU eviction.

```rust
pub struct Cache<K, V> {
    store: Arc<RwLock<HashMap<K, CachedValue<V>>>>,
    default_ttl: Duration,
    max_size: usize,
}
```

#### Methods

##### `new(ttl: Duration, max_size: usize) -> Self`
Creates a new cache.

```rust
let cache: Cache<String, String> = Cache::new(Duration::from_secs(3600), 1000);
```

##### `insert(&self, key: K, value: V)`
Inserts a value into the cache.

```rust
cache.insert("key".to_string(), "value".to_string()).await;
```

##### `get(&self, key: &K) -> Option<V>`
Retrieves a value from the cache.

```rust
let value = cache.get(&"key".to_string()).await;
```

##### `remove(&self, key: &K) -> Option<V>`
Removes and returns a value from the cache.

```rust
let value = cache.remove(&"key".to_string()).await;
```

##### `clear(&self)`
Clears all entries from the cache.

```rust
cache.clear().await;
```

### MetricsCollector

Collects and reports performance metrics.

```rust
pub struct MetricsCollector {
    metrics: Arc<RwLock<Metrics>>,
    start_time: Instant,
}
```

#### Methods

##### `new() -> Self`
Creates a new metrics collector.

```rust
let collector = MetricsCollector::new();
```

##### `record_operation(&self, duration: Duration, success: bool, cost: f64)`
Records an operation metric.

```rust
collector.record_operation(Duration::from_millis(100), true, 0.01).await;
```

##### `record_llm_operation(&self, duration: Duration, cost: f64)`
Records an LLM operation.

```rust
collector.record_llm_operation(Duration::from_millis(500), 0.03).await;
```

##### `get_metrics(&self) -> Metrics`
Gets current metrics snapshot.

```rust
let metrics = collector.get_metrics().await;
```

##### `get_summary(&self) -> MetricsSummary`
Gets a summary of metrics.

```rust
let summary = collector.get_summary().await;
println!("Success rate: {:.1}%", summary.success_rate);
```

##### `export_prometheus(&self) -> String`
Exports metrics in Prometheus format.

```rust
let prometheus_data = collector.export_prometheus().await;
```

---

## Security API

### SecurityMiddleware

Provides security features like rate limiting and input validation.

```rust
pub struct SecurityMiddleware {
    rate_limiter: RateLimiter,
    validator: InputValidator,
}
```

#### Methods

##### `new(config: SecurityConfig) -> Self`
Creates security middleware with configuration.

```rust
let security = SecurityMiddleware::new(SecurityConfig::default());
```

##### `check_request(&self, identifier: &str) -> Result<()>`
Checks if a request is allowed (rate limiting).

```rust
security.check_request("user123").await?;
```

##### `validate_url(&self, url: &str) -> Result<Url>`
Validates and sanitizes a URL.

```rust
let safe_url = security.validate_url("https://example.com")?;
```

##### `validate_input(&self, input: &str) -> Result<String>`
Validates and sanitizes text input.

```rust
let safe_input = security.validate_input(user_input)?;
```

### RateLimiter

Rate limiting implementation.

```rust
pub struct RateLimiter {
    limit: u32,
    window: Duration,
}
```

#### Methods

##### `new(requests_per_minute: u32) -> Self`
Creates a rate limiter.

```rust
let limiter = RateLimiter::new(60);
```

##### `check(&self, identifier: &str) -> Result<bool>`
Checks if a request is allowed.

```rust
let allowed = limiter.check("user123").await?;
```

### InputValidator

Input validation and sanitization.

```rust
pub struct InputValidator {
    config: SecurityConfig,
}
```

#### Methods

##### `validate_url(&self, url: &str) -> Result<Url>`
Validates a URL for safety.

```rust
let url = validator.validate_url("https://example.com")?;
```

##### `validate_input(&self, input: &str, max_length: Option<usize>) -> Result<String>`
Validates general text input.

```rust
let safe_text = validator.validate_input(user_input, Some(1000))?;
```

##### `validate_workflow(&self, content: &str) -> Result<()>`
Validates workflow YAML/JSON.

```rust
validator.validate_workflow(workflow_yaml)?;
```

---

## Configuration API

### Config

Main configuration structure.

```rust
pub struct Config {
    pub browser: BrowserConfig,
    pub llm: LlmConfig,
    pub workflow: WorkflowConfig,
    pub pool: PoolConfig,
    pub cache: CacheConfig,
    pub budget: BudgetConfig,
    pub logging: LoggingConfig,
    pub metrics: MetricsConfig,
}
```

#### Methods

##### `default() -> Self`
Creates default configuration.

```rust
let config = Config::default();
```

##### `from_file<P: AsRef<Path>>(path: P) -> Result<Self>`
Loads configuration from file.

```rust
let config = Config::from_file("config.yaml")?;
```

##### `from_env() -> Result<Self>`
Loads configuration from environment variables.

```rust
let config = Config::from_env()?;
```

##### `load<P: AsRef<Path>>(path: Option<P>) -> Result<Self>`
Loads configuration with fallbacks.

```rust
let config = Config::load(Some("config.yaml"))?;
```

##### `save<P: AsRef<Path>>(&self, path: P) -> Result<()>`
Saves configuration to file.

```rust
config.save("config.yaml")?;
```

##### `validate(&self) -> Result<()>`
Validates configuration values.

```rust
config.validate()?;
```

---

## Error Types

### Common Error Types

```rust
use anyhow::{Result, Error, Context};

// Most functions return Result<T> which is anyhow::Result<T>
// This allows for rich error context and easy error propagation

// Adding context to errors
operation().context("Failed to perform operation")?;

// Creating custom errors
Err(anyhow::anyhow!("Custom error message"))

// Checking error types
if let Err(e) = operation() {
    if e.is::<std::io::Error>() {
        // Handle IO error
    }
}
```

---

## Examples

### Basic Browser Automation

```rust
use rainbow_poc::{SimpleBrowser, ScreenshotOptions};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    // Create browser
    let browser = SimpleBrowser::new().await?;
    
    // Navigate
    browser.navigate_to("https://example.com").await?;
    
    // Take screenshot
    let options = ScreenshotOptions {
        full_page: true,
        viewport_width: 1920,
        viewport_height: 1080,
        wait_after_load: Duration::from_secs(2),
    };
    browser.take_screenshot_with_options("example.png", &options).await?;
    
    // Get page info
    let title = browser.get_title().await?;
    println!("Page title: {}", title);
    
    // Clean up
    browser.close().await?;
    
    Ok(())
}
```

### Natural Language Processing

```rust
use rainbow_poc::{LLMService, CostTracker};

#[tokio::main]
async fn main() -> Result<()> {
    let mut tracker = CostTracker::new(5.0);
    let llm = LLMService::new("sk-...".to_string());
    
    // Parse natural language
    let command = "navigate to github and take a screenshot";
    let parsed = llm.parse_natural_command(command, &mut tracker).await?;
    
    println!("Action: {}", parsed.action);
    println!("URLs: {:?}", parsed.urls);
    println!("Screenshot: {}", parsed.parameters.take_screenshot);
    println!("Confidence: {:.1}%", parsed.confidence * 100.0);
    
    Ok(())
}
```

### Workflow Execution

```rust
use rainbow_poc::{WorkflowEngine, Workflow};

#[tokio::main]
async fn main() -> Result<()> {
    let mut engine = WorkflowEngine::new();
    
    // Load workflow
    let yaml = std::fs::read_to_string("workflow.yaml")?;
    let workflow: Workflow = serde_yaml::from_str(&yaml)?;
    
    // Set input variables
    engine.set_variable("username", serde_json::json!("user@example.com")).await;
    engine.set_variable("password", serde_json::json!("secret")).await;
    
    // Execute
    let result = engine.execute(&workflow).await?;
    
    match result {
        WorkflowResult::Success(data) => {
            println!("Workflow succeeded: {:?}", data);
        }
        WorkflowResult::Failed(error) => {
            println!("Workflow failed: {}", error);
        }
    }
    
    Ok(())
}
```

### Using Browser Pool

```rust
use rainbow_poc::BrowserPool;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    let pool = BrowserPool::with_config(
        3,  // max_size
        Duration::from_secs(300),  // idle_timeout
        Duration::from_secs(3600), // max_lifetime
        100 // max_usage
    );
    
    // Acquire browser from pool
    let handle = pool.acquire().await?;
    if let Some(browser) = handle.browser() {
        browser.navigate_to("https://example.com").await?;
        // Browser automatically returns to pool when handle is dropped
    }
    
    // Check pool stats
    let stats = pool.stats().await;
    println!("Browsers in pool: {}", stats.current_size);
    println!("Total checkouts: {}", stats.total_checkouts);
    
    Ok(())
}
```

### Security Validation

```rust
use rainbow_poc::{SecurityMiddleware, SecurityConfig};

#[tokio::main]
async fn main() -> Result<()> {
    let security = SecurityMiddleware::new(SecurityConfig::default());
    
    // Rate limiting
    for i in 0..10 {
        match security.check_request("user123").await {
            Ok(_) => println!("Request {} allowed", i),
            Err(_) => println!("Request {} rate limited", i),
        }
    }
    
    // URL validation
    let urls = vec![
        "https://example.com",      // Valid
        "javascript:alert(1)",       // Invalid - XSS
        "http://localhost/admin",    // Invalid - localhost
    ];
    
    for url in urls {
        match security.validate_url(url) {
            Ok(safe_url) => println!("✅ Valid: {}", safe_url),
            Err(e) => println!("❌ Invalid: {} - {}", url, e),
        }
    }
    
    Ok(())
}
```

### Metrics Collection

```rust
use rainbow_poc::MetricsCollector;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    let collector = MetricsCollector::new();
    
    // Record operations
    for i in 0..100 {
        let success = i % 10 != 0; // 90% success rate
        let duration = Duration::from_millis(100 + i * 10);
        collector.record_operation(duration, success, 0.001).await;
    }
    
    // Get summary
    let summary = collector.get_summary().await;
    println!("Operations: {}", summary.operations_total);
    println!("Success rate: {:.1}%", summary.success_rate);
    println!("Avg response: {:.1}ms", summary.avg_response_time_ms);
    println!("P95 response: {:.1}ms", summary.p95_response_time_ms);
    
    // Export for Prometheus
    let prometheus = collector.export_prometheus().await;
    println!("{}", prometheus);
    
    Ok(())
}
```

---

## Best Practices

### Error Handling
- Always use `?` for error propagation
- Add context with `.context("description")`
- Handle specific error types when needed

### Resource Management
- Use browser pools for multiple operations
- Enable caching for repeated operations
- Monitor metrics for performance optimization

### Security
- Always validate user input
- Use rate limiting for public APIs
- Sanitize URLs before navigation

### Performance
- Use async/await for all I/O operations
- Batch operations when possible
- Monitor and optimize based on metrics

---

*This documentation covers the public API of RainbowBrowserAI PoC v0.7.0*