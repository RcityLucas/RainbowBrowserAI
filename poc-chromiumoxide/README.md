# RainbowBrowserAI - Chromiumoxide Edition

A high-performance browser automation framework built with Rust and the Chrome DevTools Protocol via chromiumoxide. Features 22 comprehensive browser automation tools through a clean web interface.

## ✨ Key Features

- **🚀 22 Browser Automation Tools**: Complete suite organized by category
- **🔧 No ChromeDriver Required**: Direct Chrome communication via DevTools Protocol  
- **⚡ High Performance**: Native async/await with browser pooling
- **🛡️ Connection Stability**: Robust error handling and retry logic
- **🌐 Web Interface**: Clean, organized testing dashboard
- **📊 Comprehensive Testing**: Systematic validation of all tools

## Quick Start

```bash
# Build and run the server
cargo run --release -- serve --port 3002 --headless

# Access web interface
# Open http://localhost:3002 in your browser

# Test all tools systematically
# Use the "Test All Tools" button in the interface
```

## 🛠️ Tool Categories

### Navigation Tools (5)
- `navigate_to_url` - Navigate to any URL
- `scroll` - Scroll to specific coordinates  
- `refresh` - Refresh current page
- `go_back` / `go_forward` - History navigation

### Interaction Tools (5)
- `click` - Click elements by CSS selector
- `type_text` - Type into input fields
- `hover` / `focus` - Element interaction
- `select_option` - Dropdown selection

### Data Extraction Tools (5)
- `extract_text` - Text content extraction
- `extract_links` - Link harvesting
- `extract_data` - Structured data with attributes
- `extract_table` / `extract_form` - Specialized extraction

### Synchronization Tools (2)
- `wait_for_element` - Wait for element appearance
- `wait_for_condition` - Wait for JavaScript conditions

### Memory & Sync Tools (5)
- `screenshot` - Page capture (base64)
- `session_memory` / `persistent_cache` - Data storage
- `get_element_info` - Element inspection
- `history_tracker` - Navigation tracking

## 🏗️ Project Structure

```
src/
├── main.rs              # Application entry point
├── api/                 # REST API handlers  
├── browser/             # Browser automation core
│   ├── core.rs         # Main browser implementation
│   ├── pool.rs         # Browser pool management
│   └── session.rs      # Session management
├── tools/               # 22 tool implementations
└── lib.rs              # Library exports

static/                  # Web interface
├── index.html          # Clean, organized UI
├── app.js              # Tool execution & testing
└── styles.css          # Modern styling

docs/                    # Complete documentation
├── TOOL_PARAMETERS_REFERENCE.md
└── API_DOCUMENTATION.md
```

## 🔌 API Reference

### Core Endpoints
- `GET /api/tools` - List all 22 available tools
- `POST /api/tools/execute` - Execute any tool with parameters

### Tool Execution Format
```json
POST /api/tools/execute
{
  "tool_name": "navigate_to_url", 
  "parameters": {"url": "https://example.com"}
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
# Test browser functionality
./target/release/rainbow-poc-chromiumoxide test

# Start server in headless mode
./target/release/rainbow-poc-chromiumoxide serve --headless

# Navigate and capture screenshot
./target/release/rainbow-poc-chromiumoxide navigate https://example.com --screenshot out.png
```

### API Usage
```bash
# Navigate to URL
curl -X POST http://localhost:3001/api/navigate \
  -H "Content-Type: application/json" \
  -d '{"url": "https://example.com"}'

# Take screenshot
curl -X POST http://localhost:3001/api/screenshot \
  -H "Content-Type: application/json" \
  -d '{"url": "https://example.com", "full_page": true}'

# Click element
curl -X POST http://localhost:3001/api/click \
  -H "Content-Type: application/json" \
  -d '{"selector": "button.submit"}'
```

## Requirements

- Rust 1.70+
- Chrome or Chromium browser installed
- No ChromeDriver needed!

## License

See parent project for licensing information.