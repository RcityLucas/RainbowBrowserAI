# Tool Parameters Reference

This document provides a comprehensive reference for all 22 browser automation tools and their required/optional parameters.

## Navigation Tools (5)

### 1. navigate_to_url
- **Required**: `url` (string) - The URL to navigate to
- **Example**: `{"url": "https://example.com"}`

### 2. scroll
- **Required**: `x` (number) - Horizontal scroll position
- **Required**: `y` (number) - Vertical scroll position  
- **Example**: `{"x": 0, "y": 300}`

### 3. refresh
- **Parameters**: None
- **Example**: `{}`

### 4. go_back
- **Parameters**: None
- **Example**: `{}`

### 5. go_forward
- **Parameters**: None
- **Example**: `{}`

## Interaction Tools (5)

### 6. click
- **Required**: `selector` (string) - CSS selector for the element to click
- **Example**: `{"selector": "button.submit"}`

### 7. type_text
- **Required**: `selector` (string) - CSS selector for the input field
- **Required**: `text` (string) - Text to type
- **Example**: `{"selector": "input[name='username']", "text": "myusername"}`

### 8. hover
- **Required**: `selector` (string) - CSS selector for the element to hover
- **Example**: `{"selector": ".dropdown-trigger"}`

### 9. focus
- **Required**: `selector` (string) - CSS selector for the element to focus
- **Example**: `{"selector": "input[type='text']"}`

### 10. select_option
- **Required**: `selector` (string) - CSS selector for the select element
- **Required**: `value` (string) - Value of the option to select
- **Example**: `{"selector": "select#country", "value": "US"}`

## Data Extraction Tools (5)

### 11. extract_text
- **Required**: `selector` (string) - CSS selector for elements to extract text from
- **Example**: `{"selector": "h1"}`

### 12. extract_links
- **Optional**: `selector` (string) - CSS selector for link elements (default: "a")
- **Example**: `{"selector": "a.external"}` or `{}`

### 13. extract_data
- **Required**: `selector` (string) - CSS selector for elements
- **Optional**: `attributes` (array of strings) - HTML attributes to extract
- **Example**: `{"selector": "img", "attributes": ["src", "alt"]}`

### 14. extract_table
- **Optional**: `selector` (string) - CSS selector for table element (default: "table")
- **Example**: `{"selector": "table.data"}` or `{}`

### 15. extract_form
- **Optional**: `selector` (string) - CSS selector for form element (default: "form")
- **Example**: `{"selector": "form#login"}` or `{}`

## Synchronization Tools (2)

### 16. wait_for_element
- **Required**: `selector` (string) - CSS selector to wait for
- **Optional**: `timeout` (number) - Timeout in milliseconds (default: 5000)
- **Example**: `{"selector": ".loading-spinner", "timeout": 10000}`

### 17. wait_for_condition
- **Required**: `condition` (string) - JavaScript condition to evaluate
- **Optional**: `timeout` (number) - Timeout in milliseconds (default: 5000)
- **Example**: `{"condition": "document.readyState === 'complete'", "timeout": 5000}`

## Memory & Sync Tools (5)

### 18. screenshot
- **Parameters**: None (uses default options)
- **Example**: `{}`
- **Note**: Returns base64 encoded image data

### 19. session_memory
- **Required**: `action` (string) - Action to perform: "get", "set", "clear"
- **Conditional**: `key` (string) - Required for "get" and "set" actions
- **Conditional**: `value` (any) - Required for "set" action
- **Examples**: 
  - Get all: `{"action": "get"}`
  - Get key: `{"action": "get", "key": "username"}`
  - Set value: `{"action": "set", "key": "username", "value": "john"}`
  - Clear: `{"action": "clear"}`

### 20. get_element_info
- **Required**: `selector` (string) - CSS selector for the element
- **Example**: `{"selector": "button#submit"}`
- **Returns**: Element tag, text, attributes, and position information

### 21. history_tracker
- **Optional**: `action` (string) - Action to perform: "get", "clear" (default: "get")
- **Examples**:
  - Get history: `{"action": "get"}` or `{}`
  - Clear history: `{"action": "clear"}`

### 22. persistent_cache
- **Required**: `action` (string) - Action to perform: "get", "set", "clear"
- **Conditional**: `key` (string) - Required for "get" and "set" actions
- **Conditional**: `value` (any) - Required for "set" action
- **Examples**:
  - Get all: `{"action": "get"}`
  - Get key: `{"action": "get", "key": "theme"}`
  - Set value: `{"action": "set", "key": "theme", "value": "dark"}`
  - Clear: `{"action": "clear"}`

## Parameter Validation Rules

1. **String parameters** cannot be empty unless explicitly optional
2. **Numeric parameters** are validated for reasonable ranges
3. **Array parameters** can be empty but must be arrays if provided
4. **CSS selectors** should be valid CSS selector syntax
5. **URLs** are automatically prefixed with https:// if no protocol is provided
6. **Timeouts** have reasonable defaults and maximum limits

## Error Handling

All tools return standardized responses:
- **Success**: `{"success": true, "data": {...}}`
- **Error**: `{"success": false, "data": null, "error": "error message"}`

## Usage Notes

1. **CSS Selectors**: Use specific selectors to avoid ambiguity
2. **Timeouts**: Adjust based on expected page load times
3. **Arrays**: For attributes, provide comma-separated values in the UI
4. **Memory Tools**: Session memory is temporary, persistent cache survives restarts
5. **Screenshots**: Returned as base64 data URLs for display in web interface