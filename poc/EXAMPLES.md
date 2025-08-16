# RainbowBrowserAI PoC Examples 🌈

Natural language browser automation examples for the RainbowBrowserAI Proof of Concept.

## 🚀 Quick Start

The PoC supports both structured CLI commands and natural language instructions. Choose the style that works best for you!

## 📝 Natural Language Commands

### Basic Navigation
```bash
# Simple navigation
cargo run -- ask "go to google"
cargo run -- ask "navigate to github"
cargo run -- ask "visit example.com"

# With screenshots
cargo run -- ask "go to google and take a screenshot"
cargo run -- ask "navigate to github and capture the page"
cargo run -- ask "visit stackoverflow and save a picture"
```

### Multi-Website Testing
```bash
# Test multiple sites
cargo run -- ask "test google, github, and stackoverflow"
cargo run -- ask "check these websites: reddit, twitter, facebook"

# With screenshots
cargo run -- ask "test google and github with screenshots"
cargo run -- ask "check reddit, twitter, and facebook and take pictures"

# With custom settings
cargo run -- ask "test google and github with 5 retries each"
cargo run -- ask "check stackoverflow and rust-lang with 60 second timeout"
```

### Advanced Parameters
```bash
# Custom viewport sizes
cargo run -- ask "navigate to example.com with a 1280x720 screenshot"
cargo run -- ask "go to mobile.twitter.com with 375x667 viewport"

# Specific filenames
cargo run -- ask "screenshot google and save it as google_homepage.png"
cargo run -- ask "capture github.com and name it github_main.png"

# Complex combinations
cargo run -- ask "test google, github, stackoverflow with screenshots at 1440x900 resolution"
```

### Reporting & Monitoring
```bash
# Cost reports
cargo run -- ask "show me the cost report"
cargo run -- ask "how much have I spent today?"
cargo run -- ask "what's my budget status?"

# System status
cargo run -- ask "show my recent commands"
cargo run -- ask "what are my current preferences?"
```

## 🔧 Structured CLI Commands

### Navigation Commands
```bash
# Basic navigation
cargo run -- navigate google.com
cargo run -- navigate https://github.com
cargo run -- navigate example.com --screenshot

# Custom screenshots
cargo run -- navigate google.com --screenshot --filename google_home.png
cargo run -- navigate example.com --screenshot --viewport-only
cargo run -- navigate github.com --screenshot --width 1280 --height 720
```

### Multi-Website Testing
```bash
# Test multiple URLs
cargo run -- test --urls "google.com,github.com,stackoverflow.com"
cargo run -- test --urls "reddit.com,twitter.com" --screenshots

# Custom settings
cargo run -- test --urls "example.com,test.com" --retries 5 --timeout 60
cargo run -- test --urls "slow-site.com" --retries 10 --timeout 120 --screenshots
```

### Reporting
```bash
# View cost report
cargo run -- report
```

## 🧠 Intelligent Features

### Preference Learning
The system learns from your usage patterns:

```bash
# After using screenshots frequently
cargo run -- ask "navigate to google"
# System: "📸 Screenshots now default to ON"

# After visiting the same sites repeatedly  
cargo run -- ask "go to my usual sites"
# System: "⭐ Favorite sites: google.com, github.com, stackoverflow.com"
```

### Context Awareness
The system remembers your recent commands:

```bash
# First command
cargo run -- ask "navigate to google with screenshot"

# Later command
cargo run -- ask "test github and stackoverflow"
# System: "💭 I found similar commands in your history:"
# System: "   - 'navigate to google with screenshot': Successfully navigated to google.com and took screenshot"
```

### Smart URL Processing
The system intelligently handles various URL formats:

```bash
# These all work the same way:
cargo run -- ask "go to google"           # → google.com
cargo run -- ask "visit www.google.com"  # → google.com  
cargo run -- ask "navigate to https://google.com/"  # → google.com

# Domain completion
cargo run -- ask "test reddit, twitter, github"  # → reddit.com, twitter.com, github.com
```

## 🛡️ Error Handling Examples

### Missing API Key
```bash
cargo run -- ask "navigate to google"
# Output:
# ❌ OpenAI API key required for natural language commands.
# 💡 Set OPENAI_API_KEY environment variable or use structured commands:
#    cargo run -- navigate google.com --screenshot
```

### Low Confidence Parsing
```bash
cargo run -- ask "do something with websites maybe"
# Output:
# ⚠️ I'm not very confident about this interpretation (45.2% sure)
# 💡 You can continue or use structured commands instead.
```

### Network Issues
```bash
cargo run -- navigate unreachable-site.com
# Output:
# ❌ Navigation failed: Connection timeout after 30 seconds
# 💡 Try increasing timeout with --timeout 60 or check the URL
```

## 📊 Budget & Cost Examples

### Checking Budget Status
```bash
cargo run -- report
# Output:
# === Daily Cost Report ===
# Date: 2025-08-16
# Budget: $0.5000
# Spent: $0.0127 (2.5%)
# Remaining: $0.4873
# Operations: 5
# Success Rate: 100.0%
# ========================
```

### Cost Protection
```bash
# When approaching budget limit
cargo run -- ask "test 100 websites"
# Output:
# ❌ Cannot afford all 100 operations ($0.8500)
# 💡 Remaining budget: $0.4873. Try fewer sites or increase budget.
```

## 🎯 Real-World Scenarios

### Website Health Check
```bash
# Check if your key services are up
cargo run -- ask "test our main sites: mycompany.com, app.mycompany.com, api.mycompany.com with screenshots"
```

### Competitive Analysis
```bash
# Capture competitor homepages
cargo run -- ask "screenshot competitor1.com, competitor2.com, competitor3.com and save them"
```

### Mobile Testing
```bash
# Test mobile responsiveness
cargo run -- navigate m.facebook.com --screenshot --width 375 --height 667
cargo run -- ask "navigate to twitter.com with a mobile viewport 375x812"
```

### Performance Monitoring
```bash
# Test with different timeout settings
cargo run -- test --urls "slow-api.com,fast-api.com" --timeout 120 --retries 3
```

### Batch Documentation
```bash
# Document multiple pages
cargo run -- test --urls "docs.rust-lang.org,doc.rust-lang.org/book,doc.rust-lang.org/std" --screenshots
```

## 🔍 Troubleshooting Examples

### ChromeDriver Not Running
```bash
cargo run -- navigate google.com
# Output:
# ❌ Failed to connect to ChromeDriver after retries
# 💡 Make sure ChromeDriver is running on port 9515
# 💡 Download from: https://chromedriver.chromium.org/
```

### Invalid URLs
```bash
cargo run -- ask "navigate to not-a-real-website.invalid"
# System handles gracefully with retry logic and clear error messages
```

### Configuration Issues
```bash
# Missing environment variables
cargo run -- ask "navigate to google"
# Output includes helpful setup instructions
```

## 🎨 Advanced Examples

### Custom Workflows
```bash
# Test and compare loading times
cargo run -- test --urls "site1.com,site2.com,site3.com" --screenshots --timeout 30

# Mobile vs Desktop comparison  
cargo run -- navigate example.com --screenshot --width 1920 --height 1080 --filename desktop.png
cargo run -- navigate example.com --screenshot --width 375 --height 667 --filename mobile.png
```

### Automation Scripting
```bash
#!/bin/bash
# Daily health check script
echo "Starting daily website health check..."

cargo run -- ask "test our production sites with screenshots"
cargo run -- report

echo "Health check complete!"
```

### Quality Assurance
```bash
# Test different environments
cargo run -- test --urls "staging.myapp.com,production.myapp.com" --screenshots --retries 3

# Visual regression testing
cargo run -- ask "screenshot myapp.com/homepage, myapp.com/login, myapp.com/dashboard"
```

## 💡 Tips & Best Practices

### Natural Language Tips
- Use simple, direct commands: "navigate to X", "test X and Y", "screenshot Z"
- Include parameters naturally: "with screenshot", "5 retries", "mobile size"
- The system learns your preferences over time - let it adapt!

### Performance Tips  
- Use structured commands for scripts and automation
- Batch multiple URLs in one test command for efficiency
- Set reasonable timeouts based on site speed

### Cost Management
- Check budget regularly with `cargo run -- report`
- Use structured commands when you don't need AI parsing
- Be mindful of screenshot storage space

### Debugging Tips
- Check logs for detailed error information
- Use `--timeout` and `--retries` for unreliable sites
- Test with simple sites first (google.com, example.com)

---

Ready to automate your browser testing? Start with simple commands and let the AI learn your preferences! 🚀