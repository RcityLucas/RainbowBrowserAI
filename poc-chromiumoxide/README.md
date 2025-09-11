# RainbowBrowserAI - Chromiumoxide Edition

An advanced AI-powered browser automation framework built with Rust and Chrome DevTools Protocol. Combines intelligent perception capabilities with a comprehensive suite of browser automation tools through a sophisticated web interface.

## ✨ Key Features

- **🧠 AI-Powered Perception**: Intelligent element detection and semantic understanding
- **🚀 28 Browser Automation Tools**: Complete suite organized by category with dependency management
- **🔧 No ChromeDriver Required**: Direct Chrome communication via DevTools Protocol  
- **⚡ High Performance**: Native async/await with browser pooling
- **🛡️ Connection Stability**: Robust error handling and retry logic
- **🌐 Smart Web Interface**: AI-enhanced testing dashboard with perception insights
- **📊 Layered Perception**: Multiple intelligence layers for complex scenarios
- **🎯 Adaptive Learning**: Pattern recognition and intelligent decision making

## Quick Start

```bash
# Build and run the server
cargo run --release -- serve --port 3002 --headless

# Access web interface
# Open http://localhost:3002 in your browser

# Test all tools systematically
# Use the "Test All Tools" button in the interface
```

### Startup & Health

- The server now starts without launching a browser up front (lazy initialization). This makes health checks and the UI available even if Chromium isn’t installed yet.
- Health endpoint: `GET http://127.0.0.1:<port>/api/health` returns status immediately after boot.
- Tools and perception features will automatically launch a browser on first use.
- The HTTP server binds to `127.0.0.1` by default for safer local development.

### Start Script

You can also use the provided script for a clean workflow:

```bash
./start.sh               # headed by default (opens dashboard)
./start.sh --headless    # run headless (no browser window)
./start.sh --no-browser  # do not auto-open dashboard
```

Notes:
- The script shows only true compiler errors (filters out attribute lines like `#[error("...")]`).
- Build failures correctly stop the script.
- The script chooses the first available port starting at 3001.

## 🛠️ Tool Categories

### Navigation Tools (5)
- `navigate` - Navigate to any URL with advanced options
- `scroll` - Scroll to specific coordinates or elements
- `refresh` - Refresh current page
- `go_back` / `go_forward` - Browser history navigation

### Interaction Tools (5)
- `click` - Click elements by CSS selector
- `type_text` - Type into input fields with validation
- `hover` / `focus` - Element interaction and focus management
- `select_option` - Dropdown and select element handling

### Data Extraction Tools (5)
- `extract_text` - Text content extraction with context
- `extract_links` - Link harvesting and analysis
- `extract_data` - Structured data with custom attributes
- `extract_table` / `extract_form` - Specialized table and form extraction

### Synchronization Tools (4)
- `wait_for_element` - Wait for element appearance with timeout
- `wait_for_condition` - Wait for custom JavaScript conditions  
- `wait_for_navigation` - Wait for page navigation completion
- `wait_for_network_idle` - CDP-backed network idle detection

### Memory Tools (5)
- `screenshot` - Capture full-page, viewport, or element screenshots
- `session_memory` - Manage browser session data
- `get_element_info` - Extract detailed element information
- `history_tracker` - Track and analyze browsing history
- `persistent_cache` - Manage persistent data storage

### CDP Monitoring Tools (3)
- `network_monitor` - Monitor network requests and performance
- `performance_metrics` - Collect detailed performance data
- `cdp_network_idle` - CDP-backed network idle detection with real-time tracking

### Advanced Automation (1)
- `create_test_fixture` - Generate synthetic HTML test pages for testing

## 🏗️ Project Structure

```
src/
├── main.rs              # Application entry point
├── api/                 # REST API handlers & perception endpoints
│   ├── mod.rs          # Core API handlers
│   └── perception_handlers.rs # AI perception endpoints
├── browser/             # Browser automation core
│   ├── core.rs         # Main browser implementation
│   ├── pool.rs         # Browser pool management
│   ├── navigation.rs   # Navigation utilities
│   └── session.rs      # Session management
├── tools/               # 28 tool implementations
│   ├── navigation.rs   # Navigation tools
│   ├── interaction.rs  # Interaction tools
│   ├── extraction.rs   # Data extraction tools
│   ├── synchronization.rs # Wait/sync tools
│   ├── memory.rs       # Memory & caching tools
│   └── traits.rs       # Tool trait definitions
├── perception/          # AI perception engine
│   ├── mod.rs          # Perception module exports
│   ├── context_aware.rs # Context-aware perception
│   ├── semantic.rs     # Semantic analysis
│   ├── smart_forms.rs  # Intelligent form handling
│   ├── layered_perception.rs # Multi-layer intelligence
│   └── integration.rs  # Perception integration
├── llm/                 # Large Language Model integration
│   ├── client.rs       # LLM client implementation
│   ├── providers.rs    # Multiple LLM providers
│   ├── prompt_engine.rs # Prompt management
│   ├── task_planner.rs # AI task planning
│   └── cost_tracker.rs # Usage tracking
├── intelligence/        # AI intelligence components
│   ├── adaptation_manager.rs # Adaptive behavior
│   ├── decision_maker.rs     # Decision logic
│   ├── learning_engine.rs    # Learning capabilities
│   ├── pattern_recognition.rs # Pattern analysis
│   └── organic_perception.rs # Natural perception
├── action/              # Enhanced action system
│   ├── element_locator.rs    # Smart element location
│   ├── verification_engine.rs # Action verification
│   └── retry_mechanism.rs    # Intelligent retries
└── lib.rs              # Library exports

static/                  # Web interface
├── index.html          # AI-enhanced UI
├── app.js              # Tool execution & AI features
└── styles.css          # Modern styling

docs/                    # Complete documentation
├── TOOL_PARAMETERS_REFERENCE.md
├── API_DOCUMENTATION.md
└── PERCEPTION_MODULE_PLAN.md
```

## 🔌 API Reference

### Core Browser Endpoints
- `GET /api/tools` - List all 28 available tools
- `POST /api/tools/execute` - Execute any tool with parameters
- `POST /api/navigate` - Navigate to URL
- `POST /api/screenshot` - Capture screenshots
- `POST /api/click` - Click elements
- `POST /api/type` - Type text into fields

### AI Perception Endpoints
- `POST /api/perception/analyze` - AI-powered page analysis
- `POST /api/perception/find` - Intelligent element search
- `POST /api/perception/command` - Execute AI commands
- `POST /api/perception/forms/analyze` - Smart form analysis
- `POST /api/perception/forms/fill` - Automated form filling
- `POST /api/perceive-mode` - Layered perception modes
- `POST /api/quick-scan` - Fast page scanning
- `POST /api/smart-element-search` - AI element location

### Session Management
- `POST /api/session/create` - Create new session
- `GET /api/session/:id` - Get session details
- `DELETE /api/session/:id` - Delete session
- `GET /api/sessions` - List all sessions

### Tool Execution Format
```json
POST /api/tools/execute
{
  "tool_name": "navigate_to_url", 
  "parameters": {"url": "https://example.com"}
}
```

### AI Perception Format
```json
POST /api/perception/analyze
{
  "url": "https://example.com",
  "analysis_type": "comprehensive",
  "include_suggestions": true
}
```

### Response Format  
```json
{"success": true, "data": {"status": "navigated", "url": "..."}}
{"success": false, "data": null, "error": "Missing parameter"}
```

## Migration from thirtyfour

See [MIGRATION_GUIDE.md](MIGRATION_GUIDE.md) for detailed migration instructions.

## Examples

### CLI Usage
```bash
# Start server with AI perception enabled
cargo run --release -- serve --port 3002 --headless

# Test browser functionality with AI insights
./target/release/rainbow-poc-chromiumoxide test --ai-enabled

# Navigate with AI analysis
./target/release/rainbow-poc-chromiumoxide navigate https://example.com --screenshot out.png --analyze
```

### Basic Browser API Usage
```bash
# Navigate to URL
curl -X POST http://localhost:3002/api/navigate \
  -H "Content-Type: application/json" \
  -d '{"url": "https://example.com"}'

# Take screenshot
curl -X POST http://localhost:3002/api/screenshot \
  -H "Content-Type: application/json" \
  -d '{"url": "https://example.com", "full_page": true}'

# Click element
curl -X POST http://localhost:3002/api/click \
  -H "Content-Type: application/json" \
  -d '{"selector": "button.submit"}'
```

### AI Perception API Usage
```bash
# Analyze page with AI
curl -X POST http://localhost:3002/api/perception/analyze \
  -H "Content-Type: application/json" \
  -d '{"url": "https://example.com", "analysis_type": "comprehensive"}'

# Intelligent form analysis
curl -X POST http://localhost:3002/api/perception/forms/analyze \
  -H "Content-Type: application/json" \
  -d '{"selector": "form", "include_validation": true}'

# Smart element search
curl -X POST http://localhost:3002/api/smart-element-search \
  -H "Content-Type: application/json" \
  -d '{"description": "login button", "context": "authentication form"}'

# Execute intelligent commands
curl -X POST http://localhost:3002/api/perception/command \
  -H "Content-Type: application/json" \
  -d '{"command": "fill out contact form", "auto_submit": false}'
```

### Layered Perception Usage
```bash
# Quick scan for rapid analysis
curl -X POST http://localhost:3002/api/quick-scan \
  -H "Content-Type: application/json" \
  -d '{"focus_areas": ["forms", "navigation", "content"]}'

# Multi-mode perception analysis
curl -X POST http://localhost:3002/api/perceive-mode \
  -H "Content-Type: application/json" \
  -d '{"mode": "comprehensive", "depth": "deep", "include_suggestions": true}'
```

## 🧠 AI Intelligence Features

### Perception Engine
- **Context-Aware Analysis**: Understands page structure and user intent
- **Semantic Element Detection**: Identifies elements by meaning, not just selectors
- **Smart Form Handling**: Intelligent form analysis and auto-filling
- **Adaptive Learning**: Improves performance based on usage patterns

### LLM Integration
- **Multiple Provider Support**: OpenAI, Claude, and other LLM providers
- **Intelligent Prompt Engineering**: Optimized prompts for browser tasks
- **Cost Tracking**: Monitor and optimize API usage
- **Task Planning**: AI-driven workflow orchestration

### Advanced Capabilities
- **Layered Perception**: Multiple intelligence layers (quick, standard, deep)
- **Organic Perception**: Natural, human-like page understanding
- **Pattern Recognition**: Learn from successful interaction patterns
- **Decision Making**: Intelligent choice between multiple strategies
- **Verification Engine**: Smart action verification and retry logic

### Intelligence Modules
- **Adaptation Manager**: Adjusts behavior based on context
- **Learning Engine**: Continuous improvement from interactions
- **Element Locator**: Advanced element finding with fallback strategies
- **Retry Mechanism**: Intelligent retry with escalating strategies

## Requirements

- **Rust**: 1.70+ with 2021 edition features
- **Browser**: Chrome or Chromium browser installed
- **AI Integration**: Optional LLM API keys for advanced features
- **Memory**: 2GB+ RAM for perception processing
- **Network**: Internet connection for AI features
- **No ChromeDriver needed!** Direct DevTools Protocol communication

## Configuration

### Environment Variables
```bash
# Core settings
BROWSER_HEADLESS=true
SERVER_PORT=3002
LOG_LEVEL=info

# AI features (optional)
OPENAI_API_KEY=your_openai_key_here
CLAUDE_API_KEY=your_claude_key_here
AI_PROVIDER=openai  # or claude, local, etc.

# Perception settings
PERCEPTION_MODE=comprehensive  # quick, standard, comprehensive
LEARNING_ENABLED=true
PATTERN_RECOGNITION=true
```

### Advanced Configuration
Create `config.toml` for fine-tuned settings:
```toml
[browser]
headless = true
viewport_width = 1920
viewport_height = 1080
timeout_seconds = 30

[perception]
default_mode = "comprehensive"
enable_learning = true
cache_patterns = true
max_retries = 3

[ai]
default_provider = "openai"
enable_cost_tracking = true
max_tokens_per_request = 4000
temperature = 0.1
```

## Architecture Overview

RainbowBrowserAI combines traditional browser automation with cutting-edge AI capabilities:

1. **Browser Layer**: High-performance Rust automation using Chromiumoxide
2. **Perception Layer**: AI-powered page understanding and element detection
3. **Intelligence Layer**: Learning, adaptation, and decision-making components
4. **LLM Layer**: Large Language Model integration for natural language processing
5. **API Layer**: RESTful endpoints for both traditional and AI-enhanced features

## Development Status

- ✅ **Core Browser Automation**: Complete (22+ tools implemented)
- ✅ **Web Interface**: Complete with AI enhancement indicators
- ✅ **Session Management**: Complete with persistent state
- ✅ **Perception Engine**: Complete with layered intelligence
- 🔄 **LLM Integration**: Active development (provider abstraction complete)
- 🔄 **Learning Engine**: Active development (pattern recognition implemented)
- 📋 **Advanced Workflows**: Planned (AI-driven task sequences)

### Recent Changes

- Lazy Tool Registry: Tools initialize on first use; server no longer fails to start if a browser is unavailable.
- Localhost Binding: Server binds to `127.0.0.1:<port>` for local dev stability.
- Diagnostics Endpoint: `GET /api/diagnostics?probe=true` optionally probes whether a headless browser can launch and returns a structured report.

### Troubleshooting

- If tool/perception endpoints return 503 with “Tool registry not initialized (browser unavailable)”, ensure Chrome/Chromium is installed and can run headless.
- Linux headless requirements: fonts + sandbox libs may be needed depending on distro. If you see messages like `crashpad ... setsockopt: Operation not permitted`, run on a host OS shell (not a restricted sandbox) and verify Chrome can start with `--headless=new`.
- Health and UI still work without a browser; use `/api/diagnostics?probe=true` to check readiness.

## Contributing

This project follows Rust best practices and modern AI integration patterns. See [CONTRIBUTING.md](../CONTRIBUTING.md) for guidelines.

## License

See parent project for licensing information.
