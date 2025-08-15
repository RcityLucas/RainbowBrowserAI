# 🌈 RainbowBrowserAI - Real Browser Setup Guide

This guide explains how to enable real browser control in RainbowBrowserAI.

## Current Status

By default, the project runs in **mock mode** to ensure it compiles and runs on all systems without external dependencies. This allows you to test the AI architecture and workflow without needing browser drivers.

## Enabling Real Browser Control

To use real browser automation with WebDriver:

### 1. Install Prerequisites

#### On Ubuntu/Debian:
```bash
# Install OpenSSL development libraries
sudo apt-get update
sudo apt-get install pkg-config libssl-dev

# Install Chrome and ChromeDriver
sudo apt-get install google-chrome-stable
sudo apt-get install chromium-chromedriver

# Or download ChromeDriver manually:
wget https://chromedriver.storage.googleapis.com/114.0.5735.90/chromedriver_linux64.zip
unzip chromedriver_linux64.zip
sudo mv chromedriver /usr/local/bin/
```

#### On macOS:
```bash
# Install with Homebrew
brew install chromedriver
brew install geckodriver  # For Firefox
```

#### On Windows:
1. Download ChromeDriver from https://chromedriver.chromium.org/
2. Add to PATH or place in project directory
3. Install OpenSSL: https://slproweb.com/products/Win32OpenSSL.html

### 2. Build with WebDriver Feature

```bash
# Build with real browser support
cargo build --features webdriver

# Or run directly
cargo run --features webdriver
```

### 3. Start WebDriver Server

Before running the application, start the WebDriver server:

```bash
# For Chrome (default port 9515)
chromedriver --port=9515

# For Firefox (default port 4444)
geckodriver --port=4444
```

### 4. Configure Browser Settings

Create a configuration file or set environment variables:

```rust
// In your code
use rainbow_browser_ai::base::browser::{WebDriverConfig, BrowserType};

let config = WebDriverConfig {
    webdriver_url: "http://localhost:9515".to_string(),
    browser: BrowserType::Chrome,
    headless: false,  // Set to true for headless mode
    window_size: (1920, 1080),
    user_agent: Some("RainbowBrowser/8.0".to_string()),
    timeout: Duration::from_secs(30),
};
```

## Using Different Browsers

### Chrome (Recommended)
- Driver: ChromeDriver
- Port: 9515 (default)
- Most stable and feature-rich

### Firefox
- Driver: GeckoDriver  
- Port: 4444 (default)
- Good alternative option

### Edge
- Driver: EdgeDriver
- Port: 9515
- Windows native option

### Safari
- Driver: SafariDriver
- macOS only
- Enable in Safari Developer menu

## Troubleshooting

### Common Issues

1. **"Could not connect to WebDriver"**
   - Ensure WebDriver server is running
   - Check the port is correct
   - Verify firewall settings

2. **"OpenSSL not found"**
   - Install OpenSSL development libraries
   - Set `OPENSSL_DIR` environment variable if needed

3. **"Browser binary not found"**
   - Install the browser (Chrome/Firefox/etc.)
   - Ensure browser is in PATH

4. **Performance Issues**
   - Use headless mode for faster execution
   - Disable image loading in browser options
   - Reduce window size for resource savings

## Mock Mode vs Real Mode

### Mock Mode (Default)
- ✅ No external dependencies
- ✅ Fast execution
- ✅ Always works
- ❌ No real browser interaction
- ❌ No actual web scraping

### Real Mode (WebDriver)
- ✅ Real browser automation
- ✅ Actual web interaction
- ✅ JavaScript execution
- ✅ Screenshots and visual testing
- ❌ Requires browser and driver setup
- ❌ Slower execution

## Example: Real Browser Usage

```rust
use rainbow_browser_ai::base::browser::{WebDriverController, WebDriverConfig};

#[tokio::main]
async fn main() -> Result<()> {
    // Create WebDriver controller
    let config = WebDriverConfig::default();
    let mut browser = WebDriverController::new(config).await?;
    
    // Start browser
    browser.start().await?;
    
    // Navigate and interact
    browser.navigate("https://www.google.com").await?;
    browser.input_text("input[name='q']", "RainbowBrowserAI").await?;
    browser.click("input[type='submit']").await?;
    
    // Get results
    let title = browser.page_title().await?;
    println!("Page title: {}", title);
    
    // Clean up
    browser.quit().await?;
    
    Ok(())
}
```

## CI/CD Integration

For continuous integration, use headless mode:

```yaml
# GitHub Actions example
- name: Setup Chrome
  uses: browser-actions/setup-chrome@latest
- name: Setup ChromeDriver
  uses: nanasess/setup-chromedriver@v2
- name: Run tests
  run: |
    chromedriver --port=9515 &
    cargo test --features webdriver
```

## Contributing

When contributing, please ensure your code works in both mock and real modes. Use feature flags to conditionally compile WebDriver-specific code:

```rust
#[cfg(feature = "webdriver")]
use crate::base::browser::WebDriverController;

#[cfg(not(feature = "webdriver"))]
use crate::base::browser::BrowserController;
```

## Support

For issues with browser setup:
- Check the [Issues](https://github.com/yourusername/RainbowBrowserAI/issues) page
- Consult WebDriver documentation
- Ask in discussions

---

**Note**: The AI and LLM features work independently of the browser mode. You can use the full AI architecture even in mock mode for development and testing.