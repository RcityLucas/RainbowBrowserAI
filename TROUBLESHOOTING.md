# RainbowBrowserAI Troubleshooting Guide 🔧

This guide covers common issues and their solutions for the RainbowBrowserAI project.

## Table of Contents
1. [Installation Issues](#installation-issues)
2. [Runtime Errors](#runtime-errors)
3. [Browser Automation Issues](#browser-automation-issues)
4. [Natural Language Processing Issues](#natural-language-processing-issues)
5. [Performance Issues](#performance-issues)
6. [Docker Issues](#docker-issues)
7. [Development Issues](#development-issues)
8. [Security Issues](#security-issues)
9. [Debugging Techniques](#debugging-techniques)
10. [Getting Help](#getting-help)

---

## Installation Issues

### Rust Installation Failed

**Error**: `curl: command not found` or installation script fails

**Solution**:
```bash
# Alternative installation method
# Download from https://rustup.rs/
# Or use package manager:

# macOS
brew install rust

# Ubuntu/Debian
sudo apt update
sudo apt install rustc cargo

# Windows
# Download installer from https://rustup.rs/
```

### ChromeDriver Not Found

**Error**: `Failed to connect to ChromeDriver at http://localhost:9515`

**Solutions**:

1. **Install ChromeDriver**:
```bash
# macOS
brew install chromedriver
brew services start chromedriver

# Linux
sudo apt-get install chromium-driver
chromedriver --port=9515 &

# Windows
# Download from https://chromedriver.chromium.org/
# Add to PATH
```

2. **Verify ChromeDriver is running**:
```bash
curl http://localhost:9515/status
```

3. **Use different port**:
```bash
chromedriver --port=4444
export CHROME_DRIVER_URL=http://localhost:4444
```

### Compilation Errors

**Error**: `error[E0433]: failed to resolve: use of undeclared crate`

**Solution**:
```bash
# Update dependencies
cargo update

# Clean and rebuild
cargo clean
cargo build

# If specific dependency issues:
rm Cargo.lock
cargo build
```

**Error**: `error: could not compile 'package-name'`

**Solution**:
```bash
# Check Rust version
rustc --version  # Should be 1.75+

# Update Rust
rustup update

# Try with specific toolchain
rustup default stable
cargo build
```

---

## Runtime Errors

### Browser Connection Failed

**Error**: `WebDriverError: Connection refused`

**Diagnosis**:
```bash
# Check if ChromeDriver is running
ps aux | grep chromedriver

# Test connection
curl http://localhost:9515/status

# Check ports
netstat -an | grep 9515
```

**Solutions**:

1. **Start ChromeDriver manually**:
```bash
chromedriver --port=9515 --verbose
```

2. **Use Docker**:
```bash
docker run -d -p 9515:9515 selenium/standalone-chrome
```

3. **Check firewall**:
```bash
# Linux
sudo ufw allow 9515

# macOS
sudo pfctl -d  # Disable firewall temporarily

# Windows
# Check Windows Defender Firewall settings
```

### OpenAI API Errors

**Error**: `OpenAI API key not configured`

**Solution**:
```bash
# Set environment variable
export OPENAI_API_KEY="sk-your-key-here"

# Or create .env file
echo "OPENAI_API_KEY=sk-your-key-here" > .env

# Verify
echo $OPENAI_API_KEY
```

**Error**: `Rate limit exceeded`

**Solution**:
```rust
// Implement retry logic in code
use tokio::time::sleep;
use std::time::Duration;

async fn call_with_retry() -> Result<Response> {
    for attempt in 0..3 {
        match call_api().await {
            Ok(response) => return Ok(response),
            Err(e) if e.to_string().contains("rate limit") => {
                sleep(Duration::from_secs(2_u64.pow(attempt))).await;
            }
            Err(e) => return Err(e),
        }
    }
    Err(anyhow::anyhow!("Max retries exceeded"))
}
```

### Budget Exceeded

**Error**: `Daily budget exceeded! Cannot proceed.`

**Solutions**:

1. **Increase budget**:
```bash
export DAILY_BUDGET=10.00
```

2. **Reset tracker**:
```bash
rm cost_tracker.json
```

3. **Check spending**:
```bash
cargo run -- report
```

---

## Browser Automation Issues

### Page Load Timeout

**Error**: `TimeoutError: Timed out waiting for page to load`

**Solutions**:

1. **Increase timeout**:
```rust
let browser = SimpleBrowser::new_with_config(
    3,  // retries
    Duration::from_secs(60)  // timeout
).await?;
```

2. **Wait for specific element**:
```rust
browser.wait_for_element("#content", Duration::from_secs(30)).await?;
```

3. **Use retry logic**:
```rust
browser.navigate_to_with_retry("https://slow-site.com", 5).await?;
```

### Screenshot Failed

**Error**: `Failed to capture screenshot`

**Solutions**:

1. **Check viewport size**:
```rust
let options = ScreenshotOptions {
    full_page: false,  // Try viewport only
    viewport_width: 1920,
    viewport_height: 1080,
    wait_after_load: Duration::from_secs(3),  // Wait longer
};
```

2. **Ensure page loaded**:
```rust
browser.navigate_to(url).await?;
tokio::time::sleep(Duration::from_secs(2)).await;
browser.take_screenshot("screenshot.png").await?;
```

3. **Check disk space**:
```bash
df -h  # Linux/macOS
# Ensure screenshots/ directory exists
mkdir -p screenshots
```

### Element Not Found

**Error**: `NoSuchElement: Unable to locate element`

**Solutions**:

1. **Verify selector**:
```javascript
// Test in browser console
document.querySelector("your-selector")
```

2. **Wait for element**:
```rust
browser.wait_for_element("#element", Duration::from_secs(10)).await?;
browser.click("#element").await?;
```

3. **Handle dynamic content**:
```rust
// Execute JavaScript to wait
browser.execute_script(r#"
    return new Promise(resolve => {
        const observer = new MutationObserver(() => {
            if (document.querySelector('#element')) {
                observer.disconnect();
                resolve(true);
            }
        });
        observer.observe(document.body, { childList: true, subtree: true });
    });
"#).await?;
```

---

## Natural Language Processing Issues

### Command Not Understood

**Error**: `Sorry, I couldn't understand that command`

**Solutions**:

1. **Use clearer commands**:
```bash
# Instead of: "do something with google"
cargo run -- ask "navigate to google.com and take a screenshot"

# Instead of: "test stuff"
cargo run -- ask "test google.com, github.com, and stackoverflow.com"
```

2. **Check confidence score**:
```rust
if parsed_command.confidence < 0.7 {
    println!("Low confidence. Try rephrasing.");
}
```

3. **Use structured commands**:
```bash
# Fallback to CLI commands
cargo run -- navigate google.com --screenshot
cargo run -- test --urls "site1.com,site2.com"
```

### High LLM Costs

**Problem**: Excessive API costs

**Solutions**:

1. **Enable caching**:
```yaml
# config.yaml
cache:
  enabled: true
  llm_ttl: 3600  # Cache for 1 hour
```

2. **Use cheaper model**:
```yaml
llm:
  model: "gpt-3.5-turbo"  # Instead of gpt-4
```

3. **Batch operations**:
```rust
// Process multiple commands at once
let commands = vec!["cmd1", "cmd2", "cmd3"];
let parsed = llm.parse_batch(commands).await?;
```

---

## Performance Issues

### Slow Operations

**Problem**: Operations taking too long

**Diagnosis**:
```bash
# Run benchmarks
cargo bench

# Profile with flamegraph
cargo install flamegraph
sudo cargo flamegraph --bin rainbow-poc
```

**Solutions**:

1. **Enable browser pooling**:
```yaml
pool:
  enabled: true
  max_size: 5
```

2. **Optimize cache settings**:
```yaml
cache:
  enabled: true
  llm_max_size: 10000
  workflow_max_size: 1000
```

3. **Use release build**:
```bash
cargo build --release
./target/release/rainbow-poc
```

### High Memory Usage

**Problem**: Excessive memory consumption

**Diagnosis**:
```bash
# Monitor memory
top -p $(pgrep rainbow-poc)

# Check for leaks
valgrind --leak-check=full ./target/debug/rainbow-poc
```

**Solutions**:

1. **Limit pool size**:
```rust
let pool = BrowserPool::with_config(
    2,  // Reduce max browsers
    Duration::from_secs(60),  // Shorter idle timeout
    Duration::from_secs(300),  // Shorter lifetime
    10  // Fewer uses per browser
);
```

2. **Clear caches periodically**:
```rust
// Add periodic cleanup
tokio::spawn(async move {
    let mut interval = tokio::time::interval(Duration::from_secs(3600));
    loop {
        interval.tick().await;
        cache.clear_expired().await;
    }
});
```

### Connection Pool Exhausted

**Error**: `No available browsers in pool`

**Solutions**:

1. **Increase pool size**:
```yaml
pool:
  max_size: 10
```

2. **Reduce idle timeout**:
```yaml
pool:
  idle_timeout: 60  # Seconds
```

3. **Handle pool exhaustion**:
```rust
match pool.acquire().await {
    Ok(handle) => {
        // Use browser
    }
    Err(_) => {
        // Fallback to creating new browser
        let browser = SimpleBrowser::new().await?;
    }
}
```

---

## Docker Issues

### Docker Build Failed

**Error**: `docker build` fails

**Solutions**:

1. **Clear Docker cache**:
```bash
docker system prune -a
docker build --no-cache -t rainbow-poc .
```

2. **Check Docker resources**:
```bash
docker system df
# Increase Docker memory/disk if needed
```

3. **Use multi-stage build**:
```dockerfile
# Optimize Dockerfile
FROM rust:1.75 as builder
# Build stage
FROM debian:bookworm-slim
# Runtime stage with minimal size
```

### Container Won't Start

**Error**: Container exits immediately

**Diagnosis**:
```bash
docker logs rainbow-poc
docker inspect rainbow-poc
```

**Solutions**:

1. **Check entrypoint**:
```bash
docker run -it --entrypoint /bin/bash rainbow-poc
```

2. **Environment variables**:
```bash
docker run -e OPENAI_API_KEY=sk-xxx rainbow-poc
```

3. **Volume permissions**:
```bash
docker run -v $(pwd)/data:/app/data:rw rainbow-poc
```

### Docker Compose Issues

**Error**: Services not connecting

**Solution**:
```yaml
# docker-compose.yml
services:
  app:
    depends_on:
      - chromedriver
    environment:
      CHROME_DRIVER_URL: http://chromedriver:9515
    networks:
      - rainbow-net

  chromedriver:
    networks:
      - rainbow-net

networks:
  rainbow-net:
    driver: bridge
```

---

## Development Issues

### Rust Compilation Errors

**Lifetime errors**:
```rust
// Problem
fn get_ref(&self) -> &String {
    &String::new()  // Returns reference to temporary
}

// Solution
fn get_ref(&self) -> String {
    String::new()  // Return owned value
}
```

**Async issues**:
```rust
// Problem
async fn recursive() {
    recursive().await;  // Error: recursion in async fn
}

// Solution
fn recursive() -> Pin<Box<dyn Future<Output = ()>>> {
    Box::pin(async move {
        recursive().await;
    })
}
```

### Test Failures

**Flaky tests**:
```rust
// Add retry logic for flaky tests
#[tokio::test]
async fn test_flaky_operation() {
    for _ in 0..3 {
        if try_operation().await.is_ok() {
            return;
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    panic!("Operation failed after retries");
}
```

**Resource cleanup**:
```rust
#[tokio::test]
async fn test_with_cleanup() {
    let browser = SimpleBrowser::new().await.unwrap();
    let result = std::panic::catch_unwind(|| {
        // Test code
    });
    
    // Always cleanup
    let _ = browser.close().await;
    
    // Re-panic if test failed
    if let Err(e) = result {
        std::panic::resume_unwind(e);
    }
}
```

---

## Security Issues

### Input Validation Errors

**Error**: `Invalid URL format` or `Input validation failed`

**Solutions**:

1. **Check URL format**:
```rust
// Ensure proper URL
let url = if !url.starts_with("http") {
    format!("https://{}", url)
} else {
    url.to_string()
};
```

2. **Sanitize input**:
```rust
let safe_input = security.validate_input(user_input, Some(1000))?;
```

3. **Handle special characters**:
```rust
let encoded = urlencoding::encode(&input);
```

### Rate Limiting

**Error**: `Rate limit exceeded`

**Solutions**:

1. **Adjust rate limits**:
```yaml
security:
  rate_limit: 120  # Increase to 120/min
```

2. **Implement backoff**:
```rust
use tokio::time::sleep;

for attempt in 0..3 {
    if security.check_request("user").await.is_ok() {
        break;
    }
    sleep(Duration::from_secs(2_u64.pow(attempt))).await;
}
```

---

## Debugging Techniques

### Enable Debug Logging

```bash
# Set log level
export RUST_LOG=debug
cargo run

# Or for specific modules
export RUST_LOG=rainbow_poc::browser=debug
```

### Use Tracing

```rust
use tracing::{instrument, debug, info, warn, error};

#[instrument]
async fn debug_function(param: &str) -> Result<()> {
    debug!("Starting with param: {}", param);
    
    let result = operation().await?;
    info!("Operation completed: {:?}", result);
    
    Ok(())
}
```

### Interactive Debugging

**VS Code**:
1. Install CodeLLDB extension
2. Add launch.json:
```json
{
    "version": "0.2.0",
    "configurations": [
        {
            "type": "lldb",
            "request": "launch",
            "name": "Debug rainbow-poc",
            "cargo": {
                "args": ["build", "--bin=rainbow-poc"],
                "filter": {
                    "name": "rainbow-poc",
                    "kind": "bin"
                }
            },
            "args": ["navigate", "google.com"],
            "cwd": "${workspaceFolder}/poc"
        }
    ]
}
```

### Memory Debugging

```bash
# Check for memory leaks
cargo install cargo-valgrind
cargo valgrind

# Profile heap usage
cargo install cargo-heaptrack
cargo heaptrack
```

### Performance Profiling

```bash
# CPU profiling
cargo install cargo-profiling
cargo profiling

# Generate flamegraph
cargo install flamegraph
cargo flamegraph --bin rainbow-poc -- navigate google.com
```

---

## Getting Help

### Resources

1. **Documentation**:
   - [User Guide](poc/USER_GUIDE.md)
   - [API Documentation](poc/API_DOCUMENTATION.md)
   - [Developer Guide](DEVELOPER_GUIDE.md)

2. **Code Examples**:
   - Check `examples/` directory
   - Integration tests in `tests/`

3. **Community**:
   - GitHub Issues for bugs
   - Discussions for questions
   - Stack Overflow with tag `rainbow-browser-ai`

### Reporting Issues

When reporting issues, include:

1. **Environment**:
```bash
rustc --version
cargo --version
chromedriver --version
echo $OPENAI_API_KEY | head -c 10  # First 10 chars only
```

2. **Error message**:
```bash
RUST_LOG=debug cargo run -- your-command 2>&1 | tee error.log
```

3. **Minimal reproduction**:
```rust
// Minimal code to reproduce issue
#[tokio::main]
async fn main() -> Result<()> {
    let browser = SimpleBrowser::new().await?;
    // Steps to reproduce...
    Ok(())
}
```

4. **What you've tried**:
   - List troubleshooting steps attempted
   - Include any relevant configuration

### Debug Checklist

- [ ] ChromeDriver is running
- [ ] Environment variables are set
- [ ] Using latest version
- [ ] Checked existing issues
- [ ] Enabled debug logging
- [ ] Tried with clean build
- [ ] Tested in Docker
- [ ] Checked system resources

---

*If you're still experiencing issues after trying these solutions, please open an issue on GitHub with detailed information about your problem.*