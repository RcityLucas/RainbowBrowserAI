# Migration Guide: thirtyfour to chromiumoxide

## Overview

This guide documents the migration from `thirtyfour` (Selenium WebDriver) to `chromiumoxide` (Chrome DevTools Protocol) for the RainbowBrowserAI project.

## Key Differences

### 1. Driver Management

#### Before (thirtyfour)
- Required ChromeDriver binary running separately
- ChromeDriver manages Chrome browser instances
- Communication via WebDriver protocol

```rust
// Old approach - requires ChromeDriver on port 9515
let caps = ChromeCapabilities::new();
let driver = WebDriver::new("http://localhost:9515", caps).await?;
```

#### After (chromiumoxide)
- Direct communication with Chrome via DevTools Protocol
- No separate driver binary needed
- Built-in browser process management

```rust
// New approach - launches Chrome directly
let (browser, handler) = Browser::launch(BrowserConfig::default()).await?;
```

### 2. API Differences

#### Navigation
```rust
// thirtyfour
driver.goto("https://example.com").await?;

// chromiumoxide  
page.goto("https://example.com").await?.wait_for_navigation().await?;
```

#### Finding Elements
```rust
// thirtyfour
let element = driver.find(By::Css("button")).await?;

// chromiumoxide
let element = page.find_element("button").await?;
```

#### Clicking
```rust
// thirtyfour
element.click().await?;

// chromiumoxide
element.click().await?;
```

#### Text Input
```rust
// thirtyfour
element.send_keys("text").await?;

// chromiumoxide
element.type_str("text").await?;
```

#### Screenshots
```rust
// thirtyfour
let screenshot = driver.screenshot_as_png().await?;

// chromiumoxide
let screenshot = page.screenshot(ScreenshotParams::default()).await?;
```

#### JavaScript Execution
```rust
// thirtyfour
let result = driver.execute_script("return document.title").await?;

// chromiumoxide
let result: String = page.evaluate("document.title").await?;
```

## Migration Steps

### Step 1: Update Dependencies

Replace in `Cargo.toml`:
```toml
# Remove
thirtyfour = "0.32"

# Add
chromiumoxide = { version = "0.5", features = ["tokio-runtime"] }
chromiumoxide_cdp = "0.5"
```

### Step 2: Update Browser Module

1. Replace WebDriver imports with chromiumoxide
2. Update browser initialization
3. Modify element interaction methods
4. Update screenshot functionality

### Step 3: Remove ChromeDriver Dependency

The start script no longer needs to:
- Check for ChromeDriver binary
- Start ChromeDriver process
- Manage ChromeDriver ports

### Step 4: Update API Endpoints

API endpoints remain largely the same, but the underlying implementation changes:
- Browser pool now manages chromiumoxide instances
- No need for WebDriver session management

## Benefits of Migration

### Performance
- **Faster**: Direct CDP communication vs HTTP/JSON protocol
- **Lower latency**: No intermediary driver process
- **Better parallelization**: Native async support

### Reliability
- **Fewer dependencies**: No external driver binary
- **Better error handling**: Direct Chrome communication
- **Improved stability**: No driver version mismatches

### Features
- **More capabilities**: Full access to Chrome DevTools Protocol
- **Better debugging**: Direct access to browser internals
- **Network interception**: Built-in request/response handling
- **Performance metrics**: Access to Chrome performance data

## Running the New Implementation

### Basic Usage
```bash
cd poc-chromiumoxide

# Build the project
cargo build --release

# Test browser connection
cargo run -- test

# Run in headless mode
cargo run -- test --headless

# Start API server
cargo run -- serve --port 3001

# Navigate and screenshot
cargo run -- navigate https://example.com --screenshot example.png
```

### API Compatibility

The API endpoints remain compatible with the original implementation:

```bash
# Health check
curl http://localhost:3001/api/health

# Navigate
curl -X POST http://localhost:3001/api/navigate \
  -H "Content-Type: application/json" \
  -d '{"url": "https://example.com"}'

# Screenshot
curl -X POST http://localhost:3001/api/screenshot \
  -H "Content-Type: application/json" \
  -d '{"url": "https://example.com", "full_page": true}'
```

## Common Issues and Solutions

### Issue 1: Chrome Not Found
**Solution**: chromiumoxide will download Chrome automatically if not found.

### Issue 2: Permission Errors
**Solution**: Run without sandbox in Docker/CI:
```rust
BrowserConfig::builder().no_sandbox().build()
```

### Issue 3: Memory Leaks
**Solution**: Properly close pages and browsers:
```rust
page.close().await?;
// Browser closes automatically when dropped
```

## Testing Migration

Run tests to verify functionality:
```bash
# Unit tests
cargo test --lib

# Integration tests  
cargo test --test '*'

# Specific test
cargo test test_browser_navigation
```

## Rollback Plan

If issues arise, the original thirtyfour implementation remains in the `poc/` directory and can be used by running the original start script.

## Next Steps

1. ✅ Core browser functionality implemented
2. ✅ API endpoints created
3. ⏳ Migrate advanced features (workflows, perception)
4. ⏳ Update tests
5. ⏳ Performance benchmarking
6. ⏳ Production deployment

## Support

For issues or questions about the migration:
- Check the [chromiumoxide documentation](https://docs.rs/chromiumoxide)
- Review the example code in `poc-chromiumoxide/examples/`
- Test using the provided CLI commands