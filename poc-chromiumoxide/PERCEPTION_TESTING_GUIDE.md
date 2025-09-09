# Testing the Enhanced Perception Features

## How to Start the Server

```bash
cd poc-chromiumoxide
cargo run --release --bin rainbow-poc-chromiumoxide -- serve --port 3002
```

Then open your browser to: http://localhost:3002

## New Features in the Perception Tab

### 1. Layered Perception (四层感知)

The new layered perception architecture provides 5 different modes:

- **Lightning** (<50ms): Ultra-fast basic page information
  - URL, title, ready state
  - Element counts (clickable, inputs, links, forms)
  
- **Quick** (<200ms): Fast interactive element discovery
  - All Lightning data
  - Interactive elements list
  - Visible text blocks
  - Form fields
  - Layout information

- **Standard** (<1000ms): Comprehensive semantic analysis
  - All Quick data
  - Semantic structure analysis
  - Accessibility information
  - Computed styles
  - Performance metrics

- **Deep** (<5000ms): Full AI-powered analysis
  - All Standard data
  - Complete DOM analysis
  - Visual content analysis
  - Behavioral pattern detection
  - AI-generated insights

- **Adaptive**: Automatically selects the best mode based on page complexity

### 2. Quick Page Scan

Provides a rapid overview of the current page:
- Interactive element count
- Text blocks
- Forms
- Images
- Key elements with selectors

### 3. Smart Element Search

AI-powered element search using natural language:
- Search by description: "red button", "login form", "navigation menu"
- Returns elements with confidence scores
- Shows selector, type, and text content
- Can highlight found elements

## Testing Steps

1. **Navigate to a test page**:
   - Go to the Browse tab
   - Enter a URL like https://example.com or https://github.com
   - Click Go

2. **Test Layered Perception**:
   - Switch to the Perception tab
   - Click each perception mode button (Lightning, Quick, Standard, Deep, Adaptive)
   - Observe the different levels of detail returned
   - Check response times displayed

3. **Test Quick Scan**:
   - Click "Quick Scan" button
   - Review the summary of page elements
   - Compare with what you see on the actual page

4. **Test Smart Element Search**:
   - Enter natural language queries:
     - "main heading"
     - "blue button"
     - "search input field"
     - "navigation links"
   - Review search results with confidence scores
   - Try the "Highlight" button to locate elements visually

## API Endpoints for Direct Testing

You can also test the API directly using curl or Postman:

```bash
# Layered Perception
curl -X POST http://localhost:3002/api/perceive-mode \
  -H "Content-Type: application/json" \
  -d '{"mode": "lightning"}'

# Quick Scan
curl -X POST http://localhost:3002/api/quick-scan \
  -H "Content-Type: application/json" \
  -d '{}'

# Smart Element Search
curl -X POST http://localhost:3002/api/smart-element-search \
  -H "Content-Type: application/json" \
  -d '{"query": "submit button", "max_results": 5}'
```

## Expected Results

### Lightning Mode
- Should return in <50ms
- Basic page metrics
- Element counts

### Quick Mode
- Should return in <200ms
- List of interactive elements
- Text content blocks

### Standard Mode
- Should return in <1000ms
- Semantic analysis
- Accessibility information

### Deep Mode
- May take up to 5 seconds
- Complete page analysis
- AI insights (when connected to LLM)

## Troubleshooting

1. **If perception times out**: The page may be too complex. Try Lightning or Quick mode first.

2. **If no elements found**: Ensure the page is fully loaded. Try navigating to the page first in the Browse tab.

3. **If server won't start**: Check that port 3002 is not in use. You can change the port in the command.

## Performance Notes

- The perception system uses caching to improve performance on repeated requests
- First perception on a page may be slower as it builds the cache
- Adaptive mode automatically selects the best perception level based on page complexity