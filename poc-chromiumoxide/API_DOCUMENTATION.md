# RainbowBrowserAI Chromiumoxide API Documentation

## Overview

RainbowBrowserAI Chromiumoxide Edition provides a comprehensive set of 22 browser automation tools accessible through a RESTful API. This document describes all available endpoints and tools.

## Table of Contents

1. [Getting Started](#getting-started)
2. [API Endpoints](#api-endpoints)
3. [Tool Categories](#tool-categories)
4. [Tool Reference](#tool-reference)
5. [Error Handling](#error-handling)
6. [Examples](#examples)
7. [Performance Considerations](#performance-considerations)

## Getting Started

### Starting the Server

```bash
# Release mode (optimized)
cargo run --release -- serve --port 3002

# With headless browser
cargo run --release -- serve --port 3002 --headless

# Debug mode
cargo run -- serve --port 3002
```

### Base URL

```
http://localhost:3002
```

### Health Check

```bash
curl http://localhost:3002/api/health
```

Response:
```json
{
  "service": "rainbow-poc-chromiumoxide",
  "status": "healthy",
  "timestamp": "2025-09-04T07:00:00.000Z"
}
```

## API Endpoints

### Core Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/` | GET | Dashboard UI |
| `/api/health` | GET | Service health check |
| `/api/tools` | GET | List all available tools |
| `/api/tools/execute` | POST | Execute a specific tool |
| `/api/session/create` | POST | Create a new browser session |
| `/api/sessions` | GET | List all active sessions |
| `/api/session/{id}` | GET | Get session details |
| `/api/session/{id}` | DELETE | Delete a session |

### Tool Execution Endpoint

**URL:** `/api/tools/execute`  
**Method:** POST  
**Content-Type:** `application/json`

**Request Body:**
```json
{
  "tool_name": "string",
  "parameters": {}
}
```

**Response:**
```json
{
  "success": true,
  "data": {},
  "error": null
}
```

## Tool Categories

### 1. Navigation Tools (5 tools)
Tools for navigating and controlling browser history.

### 2. Interaction Tools (5 tools)
Tools for interacting with page elements.

### 3. Data Extraction Tools (5 tools)
Tools for extracting information from pages.

### 4. Synchronization Tools (2 tools)
Tools for waiting and synchronization.

### 5. Memory Tools (5 tools)
Tools for capturing and storing information.

## Tool Reference

### Navigation Tools

#### 1. navigate_to_url
Navigate to a specific URL.

**Parameters:**
- `url` (string, required): The URL to navigate to

**Example:**
```bash
curl -X POST http://localhost:3002/api/tools/execute \
  -H "Content-Type: application/json" \
  -d '{"tool_name":"navigate_to_url","parameters":{"url":"https://example.com"}}'
```

#### 2. scroll
Scroll the page to specific coordinates.

**Parameters:**
- `x` (number, required): Horizontal scroll position
- `y` (number, required): Vertical scroll position

**Example:**
```bash
curl -X POST http://localhost:3002/api/tools/execute \
  -H "Content-Type: application/json" \
  -d '{"tool_name":"scroll","parameters":{"x":0,"y":500}}'
```

#### 3. refresh
Refresh the current page.

**Parameters:** None

**Example:**
```bash
curl -X POST http://localhost:3002/api/tools/execute \
  -H "Content-Type: application/json" \
  -d '{"tool_name":"refresh","parameters":{}}'
```

#### 4. go_back
Navigate back in browser history.

**Parameters:** None

**Example:**
```bash
curl -X POST http://localhost:3002/api/tools/execute \
  -H "Content-Type: application/json" \
  -d '{"tool_name":"go_back","parameters":{}}'
```

#### 5. go_forward
Navigate forward in browser history.

**Parameters:** None

**Example:**
```bash
curl -X POST http://localhost:3002/api/tools/execute \
  -H "Content-Type: application/json" \
  -d '{"tool_name":"go_forward","parameters":{}}'
```

### Interaction Tools

#### 6. click
Click on an element.

**Parameters:**
- `selector` (string, required): CSS selector for the element

**Example:**
```bash
curl -X POST http://localhost:3002/api/tools/execute \
  -H "Content-Type: application/json" \
  -d '{"tool_name":"click","parameters":{"selector":"button#submit"}}'
```

#### 7. type_text
Type text into an input field.

**Parameters:**
- `selector` (string, required): CSS selector for the input
- `text` (string, required): Text to type

**Example:**
```bash
curl -X POST http://localhost:3002/api/tools/execute \
  -H "Content-Type: application/json" \
  -d '{"tool_name":"type_text","parameters":{"selector":"input#search","text":"Hello World"}}'
```

#### 8. hover
Hover over an element.

**Parameters:**
- `selector` (string, required): CSS selector for the element

**Example:**
```bash
curl -X POST http://localhost:3002/api/tools/execute \
  -H "Content-Type: application/json" \
  -d '{"tool_name":"hover","parameters":{"selector":"div.menu"}}'
```

#### 9. focus
Focus on an element.

**Parameters:**
- `selector` (string, required): CSS selector for the element

**Example:**
```bash
curl -X POST http://localhost:3002/api/tools/execute \
  -H "Content-Type: application/json" \
  -d '{"tool_name":"focus","parameters":{"selector":"input#username"}}'
```

#### 10. select_option
Select an option from a dropdown.

**Parameters:**
- `selector` (string, required): CSS selector for the select element
- `value` (string, required): Value of the option to select

**Example:**
```bash
curl -X POST http://localhost:3002/api/tools/execute \
  -H "Content-Type: application/json" \
  -d '{"tool_name":"select_option","parameters":{"selector":"select#country","value":"US"}}'
```

### Data Extraction Tools

#### 11. extract_text
Extract text from elements.

**Parameters:**
- `selector` (string, required): CSS selector for the elements

**Example:**
```bash
curl -X POST http://localhost:3002/api/tools/execute \
  -H "Content-Type: application/json" \
  -d '{"tool_name":"extract_text","parameters":{"selector":"h1"}}'
```

#### 12. extract_links
Extract all links from the page.

**Parameters:**
- `selector` (string, optional): CSS selector for links (default: "a")

**Example:**
```bash
curl -X POST http://localhost:3002/api/tools/execute \
  -H "Content-Type: application/json" \
  -d '{"tool_name":"extract_links","parameters":{"selector":"a.external"}}'
```

#### 13. extract_data
Extract specific attributes from elements.

**Parameters:**
- `selector` (string, required): CSS selector for the elements
- `attributes` (array, optional): List of attributes to extract

**Example:**
```bash
curl -X POST http://localhost:3002/api/tools/execute \
  -H "Content-Type: application/json" \
  -d '{"tool_name":"extract_data","parameters":{"selector":"img","attributes":["src","alt"]}}'
```

#### 14. extract_table
Extract table data.

**Parameters:**
- `selector` (string, optional): CSS selector for the table (default: "table")

**Example:**
```bash
curl -X POST http://localhost:3002/api/tools/execute \
  -H "Content-Type: application/json" \
  -d '{"tool_name":"extract_table","parameters":{"selector":"table#results"}}'
```

#### 15. extract_form
Extract form data.

**Parameters:**
- `selector` (string, optional): CSS selector for the form (default: "form")

**Example:**
```bash
curl -X POST http://localhost:3002/api/tools/execute \
  -H "Content-Type: application/json" \
  -d '{"tool_name":"extract_form","parameters":{"selector":"form#login"}}'
```

### Synchronization Tools

#### 16. wait_for_element
Wait for an element to appear.

**Parameters:**
- `selector` (string, required): CSS selector for the element
- `timeout` (number, optional): Timeout in milliseconds (default: 5000)

**Example:**
```bash
curl -X POST http://localhost:3002/api/tools/execute \
  -H "Content-Type: application/json" \
  -d '{"tool_name":"wait_for_element","parameters":{"selector":"div.loaded","timeout":10000}}'
```

#### 17. wait_for_condition
Wait for a JavaScript condition to be true.

**Parameters:**
- `condition` (string, required): JavaScript expression that returns boolean
- `timeout` (number, optional): Timeout in milliseconds (default: 5000)

**Example:**
```bash
curl -X POST http://localhost:3002/api/tools/execute \
  -H "Content-Type: application/json" \
  -d '{"tool_name":"wait_for_condition","parameters":{"condition":"document.readyState === \"complete\"","timeout":10000}}'
```

### Memory Tools

#### 18. screenshot
Take a screenshot of the page.

**Parameters:**
- `full_page` (boolean, optional): Capture full page (default: false)

**Example:**
```bash
curl -X POST http://localhost:3002/api/tools/execute \
  -H "Content-Type: application/json" \
  -d '{"tool_name":"screenshot","parameters":{"full_page":true}}'
```

**Response includes base64-encoded image data.**

#### 19. session_memory
Access session memory storage.

**Parameters:**
- `action` (string, required): "get", "set", or "clear"
- `key` (string, optional): Key for get/set operations
- `value` (any, optional): Value for set operation

**Example:**
```bash
# Get all session data
curl -X POST http://localhost:3002/api/tools/execute \
  -H "Content-Type: application/json" \
  -d '{"tool_name":"session_memory","parameters":{"action":"get"}}'

# Set a value
curl -X POST http://localhost:3002/api/tools/execute \
  -H "Content-Type: application/json" \
  -d '{"tool_name":"session_memory","parameters":{"action":"set","key":"user","value":"john"}}'
```

#### 20. get_element_info
Get detailed information about an element.

**Parameters:**
- `selector` (string, required): CSS selector for the element

**Example:**
```bash
curl -X POST http://localhost:3002/api/tools/execute \
  -H "Content-Type: application/json" \
  -d '{"tool_name":"get_element_info","parameters":{"selector":"button#submit"}}'
```

#### 21. history_tracker
Track and manage page history.

**Parameters:**
- `action` (string, required): "get" or "clear"

**Example:**
```bash
curl -X POST http://localhost:3002/api/tools/execute \
  -H "Content-Type: application/json" \
  -d '{"tool_name":"history_tracker","parameters":{"action":"get"}}'
```

#### 22. persistent_cache
Access persistent cache storage.

**Parameters:**
- `action` (string, required): "get", "set", or "clear"
- `key` (string, optional): Key for get/set operations
- `value` (any, optional): Value for set operation

**Example:**
```bash
curl -X POST http://localhost:3002/api/tools/execute \
  -H "Content-Type: application/json" \
  -d '{"tool_name":"persistent_cache","parameters":{"action":"set","key":"config","value":{"theme":"dark"}}}'
```

## Error Handling

### Error Response Format

```json
{
  "success": false,
  "data": null,
  "error": "Error message describing what went wrong"
}
```

### Common Error Codes

| Error | Description | Solution |
|-------|-------------|----------|
| `Unknown tool` | Tool name not recognized | Check tool name spelling |
| `Missing required parameter` | Required parameter not provided | Include all required parameters |
| `Element not found` | CSS selector didn't match any elements | Verify selector is correct |
| `Timeout` | Operation timed out | Increase timeout or check condition |
| `Failed to launch Chrome browser` | Browser couldn't start | Check Chrome installation, restart server |
| `Failed to acquire browser` | Browser pool exhausted | Wait and retry, or restart server |

## Examples

### Complete Workflow Example

```bash
# 1. Navigate to a website
curl -X POST http://localhost:3002/api/tools/execute \
  -H "Content-Type: application/json" \
  -d '{"tool_name":"navigate_to_url","parameters":{"url":"https://example.com"}}'

# 2. Wait for page to load
curl -X POST http://localhost:3002/api/tools/execute \
  -H "Content-Type: application/json" \
  -d '{"tool_name":"wait_for_condition","parameters":{"condition":"document.readyState === \"complete\""}}'

# 3. Extract the page title
curl -X POST http://localhost:3002/api/tools/execute \
  -H "Content-Type: application/json" \
  -d '{"tool_name":"extract_text","parameters":{"selector":"h1"}}'

# 4. Take a screenshot
curl -X POST http://localhost:3002/api/tools/execute \
  -H "Content-Type: application/json" \
  -d '{"tool_name":"screenshot","parameters":{"full_page":false}}'
```

### Form Automation Example

```bash
# 1. Navigate to login page
curl -X POST http://localhost:3002/api/tools/execute \
  -H "Content-Type: application/json" \
  -d '{"tool_name":"navigate_to_url","parameters":{"url":"https://example.com/login"}}'

# 2. Type username
curl -X POST http://localhost:3002/api/tools/execute \
  -H "Content-Type: application/json" \
  -d '{"tool_name":"type_text","parameters":{"selector":"input#username","text":"user@example.com"}}'

# 3. Type password
curl -X POST http://localhost:3002/api/tools/execute \
  -H "Content-Type: application/json" \
  -d '{"tool_name":"type_text","parameters":{"selector":"input#password","text":"password123"}}'

# 4. Click submit
curl -X POST http://localhost:3002/api/tools/execute \
  -H "Content-Type: application/json" \
  -d '{"tool_name":"click","parameters":{"selector":"button[type=\"submit\"]"}}'

# 5. Wait for redirect
curl -X POST http://localhost:3002/api/tools/execute \
  -H "Content-Type: application/json" \
  -d '{"tool_name":"wait_for_element","parameters":{"selector":"div.dashboard","timeout":10000}}'
```

## Performance Considerations

### Browser Pool Management

- The server maintains a pool of browser instances for better performance
- Browsers are reused between requests when possible
- Pool size is limited to prevent resource exhaustion

### Best Practices

1. **Use appropriate timeouts:** Don't set timeouts too high for wait operations
2. **Reuse sessions:** Use the session API for related operations
3. **Clean up:** Clear session memory and cache when no longer needed
4. **Batch operations:** Group related operations together
5. **Error handling:** Always handle potential errors in your client code

### Performance Tips

- **Headless mode** is faster than headed mode
- **Screenshots** are resource-intensive, use sparingly
- **Full-page screenshots** take longer than viewport screenshots
- **Complex selectors** take longer to evaluate
- **JavaScript conditions** should be simple and efficient

## Troubleshooting

### Common Issues

#### Browser Won't Launch
- Ensure Chrome/Chromium is installed
- Check system resources (RAM, CPU)
- Try restarting the server
- Use headless mode if display issues occur

#### Elements Not Found
- Verify the page has loaded completely
- Check if element is in an iframe
- Use wait_for_element before interacting
- Verify CSS selector syntax

#### Timeouts
- Increase timeout values
- Check network connectivity
- Verify the condition/element exists
- Use simpler conditions

### Debug Mode

Enable debug logging:
```bash
RUST_LOG=debug cargo run -- serve --port 3002
```

### Support

For issues or questions:
- Check the [API Documentation](#api-documentation)
- Review the [Examples](#examples)
- Check server logs for error details
- Restart the server if experiencing persistent issues

## Version History

### v0.1.0 (Current)
- Initial release with 22 browser automation tools
- Chrome DevTools Protocol integration via chromiumoxide
- RESTful API with JSON responses
- Browser pool management
- Session management
- Comprehensive error handling

## License

This project is part of the RainbowBrowserAI suite. See the main repository for license information.