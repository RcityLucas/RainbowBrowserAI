# RainbowBrowserAI User Guide 📚

Complete guide for using the RainbowBrowserAI browser automation platform.

## Table of Contents
1. [Quick Start](#quick-start)
2. [Installation](#installation)
3. [Basic Usage](#basic-usage)
4. [Natural Language Commands](#natural-language-commands)
5. [Workflow Automation](#workflow-automation)
6. [Advanced Features](#advanced-features)
7. [Configuration](#configuration)
8. [Troubleshooting](#troubleshooting)

## Quick Start

Get up and running in 5 minutes:

```bash
# Clone the repository
git clone <repository>
cd poc

# Set your OpenAI API key (optional, for natural language)
export OPENAI_API_KEY="your-api-key"

# Build and run
cargo build --release
cargo run -- --help
```

### Docker Quick Start

```bash
# Using Docker Compose
docker-compose up -d

# Run a command
docker-compose exec rainbow-poc rainbow-poc navigate google.com --screenshot
```

## Installation

### Prerequisites

- **Rust 1.70+**: Install from [rustup.rs](https://rustup.rs/)
- **ChromeDriver**: Required for browser automation
- **OpenAI API Key**: Optional, for natural language features

### Local Installation

1. **Install ChromeDriver**:
```bash
# macOS
brew install chromedriver

# Linux
sudo apt-get install chromium-driver

# Windows
# Download from https://chromedriver.chromium.org/
```

2. **Build the Project**:
```bash
cargo build --release
```

3. **Run ChromeDriver**:
```bash
chromedriver --port=9515
```

### Docker Installation

```bash
# Build the image
docker build -t rainbow-poc .

# Run with docker-compose
docker-compose up -d
```

## Basic Usage

### CLI Commands

RainbowBrowserAI provides three ways to interact:
1. **Structured Commands**: Direct CLI commands
2. **Natural Language**: Human-friendly commands
3. **Workflows**: YAML/JSON automation scripts

### Navigate Command

Basic navigation with optional screenshots:

```bash
# Simple navigation
rainbow-poc navigate google.com

# With screenshot
rainbow-poc navigate github.com --screenshot

# Custom viewport
rainbow-poc navigate example.com --screenshot --width 1280 --height 720

# Viewport-only screenshot (not full page)
rainbow-poc navigate site.com --screenshot --viewport-only
```

### Test Command

Test multiple websites:

```bash
# Test multiple sites
rainbow-poc test --urls "google.com,github.com,stackoverflow.com"

# With screenshots
rainbow-poc test --urls "site1.com,site2.com" --screenshots

# Custom retry and timeout
rainbow-poc test --urls "slow-site.com" --retries 5 --timeout 60
```

### Report Command

View cost and usage reports:

```bash
rainbow-poc report
```

## Natural Language Commands

### Prerequisites

Set your OpenAI API key:
```bash
export OPENAI_API_KEY="sk-..."
```

### Basic Natural Language

```bash
# Navigation
rainbow-poc ask "go to google"
rainbow-poc ask "navigate to github and take a screenshot"

# Testing
rainbow-poc ask "test google, github, and stackoverflow"
rainbow-poc ask "check if amazon and ebay are working"

# Screenshots
rainbow-poc ask "take a screenshot of rust-lang.org"
rainbow-poc ask "capture the google homepage at 1280x720"

# Reports
rainbow-poc ask "show me the cost report"
rainbow-poc ask "how much have I spent today?"
```

### Advanced Natural Language

```bash
# Complex parameters
rainbow-poc ask "navigate to example.com with a mobile viewport of 375x667"
rainbow-poc ask "test these sites with 5 retries: google.com, github.com"

# Multi-step requests
rainbow-poc ask "go to google, take a screenshot, then test github and stackoverflow"
```

## Workflow Automation

### Workflow Structure

Workflows are defined in YAML or JSON files:

```yaml
name: "My Workflow"
description: "Example workflow"
inputs:
  - name: url
    input_type: string
    required: true

steps:
  - name: "Navigate"
    action:
      type: navigate
      url: "{{url}}"
      screenshot: true
```

### Running Workflows

```bash
# Execute a workflow
rainbow-poc workflow my_workflow.yaml --inputs url=google.com

# Dry run (validate without executing)
rainbow-poc workflow my_workflow.yaml --dry-run

# Multiple inputs
rainbow-poc workflow complex.yaml \
  --inputs url=example.com,username=user,password=pass
```

### Available Actions

#### Navigate
```yaml
action:
  type: navigate
  url: "https://example.com"
  screenshot: true
```

#### Click
```yaml
action:
  type: click
  selector: "button.submit"
  wait_after: 2  # seconds
```

#### Fill
```yaml
action:
  type: fill
  selector: "input#username"
  value: "{{username}}"
```

#### Extract
```yaml
action:
  type: extract
  selector: "h1.title"
  attribute: "text"  # or any HTML attribute
store_as: "page_title"
```

#### Wait
```yaml
# Wait for element
action:
  type: wait
  wait_for: element
  selector: ".results"

# Wait for text
action:
  type: wait
  wait_for: text
  text: "Loading complete"

# Wait for time
action:
  type: wait
  wait_for: time
  seconds: 5
```

#### Assert
```yaml
action:
  type: assert
  assert: element_exists
  selector: "#success-message"
```

#### Loop
```yaml
action:
  type: loop
  over: "items"
  do:
    - name: "Process item"
      action:
        type: click
        selector: ".item-{{_loop_index}}"
```

#### Conditional
```yaml
action:
  type: conditional
  if:
    check: element_exists
    selector: ".login-required"
  then:
    - name: "Login"
      action:
        type: navigate
        url: "/login"
  else:
    - name: "Continue"
      action:
        type: navigate
        url: "/dashboard"
```

### Workflow Templates

Use pre-built templates from `workflows/templates/`:

```bash
# Google search
rainbow-poc workflow workflows/templates/google_search.yaml \
  --inputs query="Rust programming"

# Multi-site test
rainbow-poc workflow workflows/templates/multi_site_test.yaml

# Login flow
rainbow-poc workflow workflows/templates/login_flow.yaml \
  --inputs site_url=example.com,username=user,password=pass
```

## Advanced Features

### Browser Pooling

The system automatically pools browser connections for better performance:
- Reuses browser instances across operations
- Configurable pool size and timeouts
- Automatic cleanup of expired connections

### Caching

Intelligent caching reduces costs and improves speed:
- LLM response caching
- Workflow template caching
- Configurable TTL and size limits

### Metrics and Monitoring

Track performance and usage:

```bash
# View metrics summary
rainbow-poc metrics

# Export Prometheus metrics
rainbow-poc metrics --format prometheus

# Real-time monitoring with Grafana
# Access at http://localhost:3000 when using docker-compose
```

### Cost Management

Built-in budget protection:
- Daily budget limits
- Per-operation cost tracking
- Automatic blocking when approaching limits

```bash
# Set daily budget
export DAILY_BUDGET=10.00

# Check current spending
rainbow-poc report
```

### Conversation Memory

The system learns from your usage:
- Remembers preferences (screenshot defaults, viewport sizes)
- Tracks frequently visited sites
- Suggests similar commands from history

## Configuration

### Environment Variables

```bash
# Required for natural language
OPENAI_API_KEY=sk-...

# Optional configuration
DAILY_BUDGET=5.00              # Daily spending limit
CHROME_DRIVER_URL=http://localhost:9515
RUST_LOG=info                  # Logging level (debug, info, warn, error)
```

### Configuration File

Create `.env` file in the project root:

```env
OPENAI_API_KEY=sk-...
DAILY_BUDGET=10.00
CHROME_DRIVER_URL=http://localhost:9515
RUST_LOG=debug
```

### User Preferences

Preferences are automatically learned and saved in `conversation_context.json`:
- Default screenshot settings
- Preferred viewport sizes
- Favorite websites
- Command patterns

## Troubleshooting

### Common Issues

#### ChromeDriver Not Found
```
Error: Failed to connect to ChromeDriver
```
**Solution**: Ensure ChromeDriver is running on port 9515:
```bash
chromedriver --port=9515
```

#### OpenAI API Key Missing
```
Error: OpenAI API key not configured
```
**Solution**: Set the environment variable:
```bash
export OPENAI_API_KEY="your-key"
```

#### Budget Exceeded
```
Error: Daily budget exceeded
```
**Solution**: Check spending and increase budget if needed:
```bash
rainbow-poc report
export DAILY_BUDGET=10.00
```

#### Workflow Parse Error
```
Error: Failed to parse workflow
```
**Solution**: Validate YAML syntax and check for:
- Proper indentation (2 spaces)
- Valid action types
- Required fields present

### Debug Mode

Enable detailed logging for troubleshooting:

```bash
# Set debug logging
export RUST_LOG=debug

# Run with verbose output
rainbow-poc navigate google.com --screenshot
```

### Performance Issues

If experiencing slow performance:

1. **Check browser pool**:
   - Increase pool size for parallel operations
   - Adjust timeouts for slow sites

2. **Monitor metrics**:
```bash
rainbow-poc metrics
```

3. **Clear cache if needed**:
```bash
rainbow-poc cache clear
```

### Docker Issues

For Docker-specific problems:

```bash
# View logs
docker-compose logs -f rainbow-poc

# Restart services
docker-compose restart

# Clean rebuild
docker-compose down
docker-compose build --no-cache
docker-compose up
```

## Best Practices

### Workflow Design

1. **Use templates**: Start with existing templates and modify
2. **Add error handling**: Use `on_error` strategies
3. **Test with dry-run**: Always validate before execution
4. **Use variables**: Make workflows reusable with inputs

### Performance Optimization

1. **Batch operations**: Use multi-site testing for efficiency
2. **Enable caching**: Reduces API calls and costs
3. **Pool browsers**: Reuse connections when possible
4. **Monitor metrics**: Track performance and optimize

### Cost Management

1. **Set budgets**: Use DAILY_BUDGET to prevent overruns
2. **Use caching**: Reduce repeated LLM calls
3. **Batch similar operations**: More efficient than individual runs
4. **Monitor usage**: Regular `rainbow-poc report` checks

### Security

1. **Protect API keys**: Never commit keys to version control
2. **Use environment variables**: Store sensitive data securely
3. **Run as non-root**: Use Docker's user isolation
4. **Validate inputs**: Be cautious with user-provided URLs

## Support

For issues, questions, or contributions:
- GitHub Issues: [Report bugs or request features]
- Documentation: Check this guide and API docs
- Examples: See `workflows/templates/` for patterns

---

**Happy Automating!** 🌈🤖