# Tools Implementation Plan

## Overview
This document outlines the comprehensive plan for implementing the tools system in the chromiumoxide-based POC. Tools are the core building blocks for browser automation.

## Tool Categories

### 1. Navigation Tools
- **navigate_to_url**: Navigate to a URL with options
- **scroll_page**: Scroll to specific positions or elements
- **go_back/go_forward**: Browser history navigation
- **refresh_page**: Reload with cache options

### 2. Interaction Tools
- **click**: Click on elements (with smart detection)
- **type_text**: Type text into input fields
- **select_option**: Select from dropdowns
- **hover**: Hover over elements
- **drag_and_drop**: Drag elements

### 3. Data Extraction Tools
- **extract_text**: Extract text from elements
- **extract_links**: Get all links from page
- **extract_table**: Extract table data
- **extract_form**: Get form structure and values
- **extract_data**: Generic data extraction with selectors

### 4. Synchronization Tools
- **wait_for_element**: Wait for element to appear
- **wait_for_condition**: Wait for custom condition
- **wait_for_navigation**: Wait for page navigation
- **wait_for_network_idle**: Wait for network to be idle

### 5. Memory Tools
- **take_screenshot**: Capture screenshots
- **session_memory**: Store/retrieve session data
- **history_tracker**: Track navigation history
- **persistent_cache**: Long-term storage
- **get_element_info**: Get element properties

### 6. Intelligence Tools
- **perception_analyzer**: Analyze page structure
- **smart_actions**: Intelligent action selection
- **visual_validator**: Validate visual elements

### 7. Meta-cognitive Tools
- **complete_task**: Mark task as complete
- **report_insight**: Report findings

### 8. Advanced Automation Tools
- **workflow_orchestrator**: Execute complex workflows
- **performance_monitor**: Monitor performance metrics

## Architecture Design

```rust
// Core trait that all tools must implement
#[async_trait]
pub trait Tool: Send + Sync {
    type Input: Serialize + DeserializeOwned + Send + Sync;
    type Output: Serialize + Send + Sync;
    
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn category(&self) -> ToolCategory;
    
    async fn execute(&self, input: Self::Input) -> Result<Self::Output>;
    async fn validate_input(&self, input: &Self::Input) -> Result<()>;
}

// Tool categories for organization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ToolCategory {
    Navigation,
    Interaction,
    DataExtraction,
    Synchronization,
    Memory,
    Intelligence,
    MetaCognitive,
    AdvancedAutomation,
}

// Tool registry for managing all tools
pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn DynamicTool>>,
    browser: Arc<Browser>,
}

// Dynamic tool wrapper for runtime dispatch
pub trait DynamicTool: Send + Sync {
    fn name(&self) -> &str;
    fn execute_json(&self, input: Value) -> BoxFuture<'_, Result<Value>>;
}
```

## Implementation Phases

### Phase 1: Core Infrastructure (Day 1)
- [x] Define Tool trait
- [ ] Create DynamicTool wrapper
- [ ] Implement ToolRegistry
- [ ] Set up error handling
- [ ] Create tool configuration

### Phase 2: Navigation Tools (Day 1-2)
- [ ] navigate_to_url
- [ ] scroll_page
- [ ] go_back/go_forward
- [ ] refresh_page

### Phase 3: Interaction Tools (Day 2-3)
- [ ] click (with smart detection)
- [ ] type_text
- [ ] select_option
- [ ] hover
- [ ] drag_and_drop (optional)

### Phase 4: Data Extraction Tools (Day 3-4)
- [ ] extract_text
- [ ] extract_links
- [ ] extract_table
- [ ] extract_form
- [ ] extract_data

### Phase 5: Synchronization Tools (Day 4)
- [ ] wait_for_element
- [ ] wait_for_condition
- [ ] wait_for_navigation
- [ ] wait_for_network_idle

### Phase 6: Memory Tools (Day 5)
- [ ] take_screenshot
- [ ] session_memory
- [ ] history_tracker
- [ ] persistent_cache
- [ ] get_element_info

### Phase 7: Testing & Integration (Day 5-6)
- [ ] Unit tests for each tool
- [ ] Integration tests
- [ ] Performance benchmarks
- [ ] Documentation
- [ ] Examples

## Tool Implementation Template

```rust
use super::{Tool, ToolCategory};
use crate::browser::{Browser, BrowserOps};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use async_trait::async_trait;

#[derive(Debug, Serialize, Deserialize)]
pub struct NavigateInput {
    pub url: String,
    pub wait_until: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct NavigateOutput {
    pub success: bool,
    pub final_url: String,
    pub load_time_ms: u64,
}

pub struct NavigateTool {
    browser: Arc<Browser>,
}

#[async_trait]
impl Tool for NavigateTool {
    type Input = NavigateInput;
    type Output = NavigateOutput;
    
    fn name(&self) -> &str {
        "navigate_to_url"
    }
    
    fn description(&self) -> &str {
        "Navigate to a specified URL"
    }
    
    fn category(&self) -> ToolCategory {
        ToolCategory::Navigation
    }
    
    async fn execute(&self, input: Self::Input) -> Result<Self::Output> {
        let start = std::time::Instant::now();
        self.browser.navigate_to(&input.url).await?;
        
        Ok(NavigateOutput {
            success: true,
            final_url: self.browser.current_url().await?,
            load_time_ms: start.elapsed().as_millis() as u64,
        })
    }
    
    async fn validate_input(&self, input: &Self::Input) -> Result<()> {
        if input.url.is_empty() {
            return Err(anyhow::anyhow!("URL cannot be empty"));
        }
        Ok(())
    }
}
```

## Testing Strategy

### Unit Tests
Each tool should have:
- Input validation tests
- Success case tests
- Error handling tests
- Edge case tests

### Integration Tests
- Tool chaining tests
- Browser interaction tests
- Performance tests
- Concurrent execution tests

### Example Test
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_navigate_tool() {
        let browser = Browser::new_headless().await.unwrap();
        let tool = NavigateTool::new(Arc::new(browser));
        
        let input = NavigateInput {
            url: "https://example.com".to_string(),
            wait_until: None,
        };
        
        let output = tool.execute(input).await.unwrap();
        assert!(output.success);
        assert!(output.final_url.contains("example.com"));
    }
}
```

## Tool Chain Example

```rust
// Example of chaining multiple tools
let registry = ToolRegistry::new(browser);

// Navigate to a page
registry.execute("navigate_to_url", json!({
    "url": "https://example.com"
})).await?;

// Wait for an element
registry.execute("wait_for_element", json!({
    "selector": "#search-box",
    "timeout_ms": 5000
})).await?;

// Type text
registry.execute("type_text", json!({
    "selector": "#search-box",
    "text": "chromiumoxide"
})).await?;

// Click search button
registry.execute("click", json!({
    "selector": "#search-button"
})).await?;

// Extract results
let results = registry.execute("extract_data", json!({
    "selector": ".search-result",
    "attributes": ["title", "href", "description"]
})).await?;
```

## Performance Considerations

1. **Async Execution**: All tools must be async
2. **Resource Management**: Proper cleanup in Drop implementations
3. **Error Recovery**: Graceful error handling with retries
4. **Caching**: Cache element references when possible
5. **Batching**: Batch similar operations

## Security Considerations

1. **Input Validation**: Strict input validation
2. **XSS Prevention**: Sanitize JavaScript execution
3. **Rate Limiting**: Prevent abuse
4. **Timeout Enforcement**: Prevent infinite loops
5. **Resource Limits**: Memory and CPU limits

## Success Criteria

- [ ] All core tools implemented
- [ ] 100% test coverage for critical paths
- [ ] Performance benchmarks pass
- [ ] Documentation complete
- [ ] Examples working
- [ ] Integration with main API

## Next Steps

1. Start with core infrastructure
2. Implement navigation tools first (most basic)
3. Then interaction tools (build on navigation)
4. Data extraction (uses interaction)
5. Synchronization (supports all)
6. Memory and advanced features last