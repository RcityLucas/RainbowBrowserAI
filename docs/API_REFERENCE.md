# 彩虹城浏览器 V8.0 API 参考

## 📚 概述

彩虹城浏览器 V8.0 提供简洁而强大的 API，核心 API 数量控制在 20 个以内。所有 API 遵循统一的设计原则：

- **类型安全**：Rust 强类型，Python 类型注解
- **错误处理**：Result<T, E> 模式，100% 错误捕获
- **性能优先**：标准模式 <100ms 响应
- **透明决策**：所有操作可追踪

## 🚀 快速开始

### Rust SDK

```rust
use rainbow_browser::{Browser, BrowserConfig, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let browser = Browser::new(BrowserConfig::default()).await?;
    let page = browser.new_page().await?;
    
    page.navigate("https://example.com").await?;
    let title = page.get_title().await?;
    
    println!("Page title: {}", title);
    Ok(())
}
```

### Python SDK

```python
import asyncio
from rainbow_browser import Browser, PerceptionMode

async def main():
    async with Browser() as browser:
        page = await browser.new_page()
        await page.navigate("https://example.com")
        title = await page.get_title()
        print(f"Page title: {title}")

asyncio.run(main())
```

## 🎯 核心 API

### Browser 类

#### `Browser::new(config: BrowserConfig) -> Result<Browser>`

创建新的浏览器实例。

**参数：**
```rust
pub struct BrowserConfig {
    pub data_dir: Option<PathBuf>,      // 数据目录
    pub headless: bool,                 // 无头模式，默认 true
    pub timeout: Duration,              // 全局超时，默认 30s
    pub perception_mode: PerceptionMode,// 感知模式，默认 Standard
    pub cache_strategy: CacheStrategy,  // 缓存策略，默认 Balanced
}
```

**返回：** `Result<Browser>` - 浏览器实例或错误

**示例：**
```rust
let config = BrowserConfig {
    headless: false,
    perception_mode: PerceptionMode::Deep,
    ..Default::default()
};
let browser = Browser::new(config).await?;
```

#### `browser.new_page() -> Result<Page>`

创建新的页面（标签页）。

**返回：** `Result<Page>` - 页面实例

**性能：** <10ms

### Page 类

#### `page.navigate(url: &str) -> Result<()>`

导航到指定 URL。

**参数：**
- `url` - 目标 URL

**返回：** `Result<()>` - 成功或错误

**性能：** 取决于网络，本地处理 <50ms

#### `page.perceive(mode: PerceptionMode) -> Result<PerceptionResult>`

执行页面感知，获取结构化数据。

**参数：**
```rust
pub enum PerceptionMode {
    Fast,     // <50ms，基础感知
    Standard, // <200ms，标准感知
    Deep,     // <500ms，深度感知
}
```

**返回：**
```rust
pub struct PerceptionResult {
    pub structure: StructureData,    // DOM 结构
    pub visual: Option<VisualData>,  // 视觉信息
    pub semantic: Option<SemanticData>, // 语义理解
    pub timestamp: SystemTime,
    pub mode: PerceptionMode,
}
```

#### `page.execute_action(action: Action) -> Result<ActionResult>`

执行智能行动。

**参数：**
```rust
pub enum Action {
    Click { selector: String },
    TypeText { selector: String, text: String },
    Scroll { direction: ScrollDirection, amount: i32 },
    Wait { condition: WaitCondition },
    // ... 更多动作
}
```

**返回：** `Result<ActionResult>` - 执行结果

## 🛠️ 12个标准工具 API

### 导航类工具

#### 1. `navigate_to_url(url: String, options?: NavigateOptions) -> Result<NavigateResult>`

智能导航到指定 URL。

**参数：**
```typescript
interface NavigateOptions {
  wait_until?: 'load' | 'domcontentloaded' | 'networkidle';
  timeout?: number;  // 毫秒
  retry?: boolean;   // 失败时自动重试
}
```

**返回：**
```typescript
interface NavigateResult {
  success: boolean;
  final_url: string;     // 最终 URL（可能重定向）
  load_time: number;     // 加载时间(ms)
  status_code: number;   // HTTP 状态码
  error?: string;
}
```

**示例：**
```python
result = await page.navigate_to_url(
    "https://example.com",
    wait_until="networkidle",
    timeout=30000
)
```

#### 2. `scroll_page(direction: ScrollDirection, amount?: number) -> Result<ScrollResult>`

智能滚动页面。

**参数：**
```rust
pub enum ScrollDirection {
    Up,
    Down,
    Left,
    Right,
    ToTop,
    ToBottom,
    ToElement(String), // CSS 选择器
}
```

**返回：**
```typescript
interface ScrollResult {
  success: boolean;
  new_position: { x: number; y: number };
  viewport_height: number;
  document_height: number;
}
```

### 交互类工具

#### 3. `click(selector: String, options?: ClickOptions) -> Result<ClickResult>`

精准点击元素。

**参数：**
```typescript
interface ClickOptions {
  button?: 'left' | 'right' | 'middle';
  click_count?: number;    // 单击/双击
  delay?: number;          // 点击前延迟(ms)
  offset?: { x: number; y: number };  // 相对元素的偏移
  force?: boolean;         // 强制点击（忽略可见性）
}
```

**返回：**
```typescript
interface ClickResult {
  success: boolean;
  element_found: boolean;
  clicked_element: ElementInfo;
  triggered_navigation: boolean;
  error?: string;
}
```

#### 4. `type_text(selector: String, text: String, options?: TypeOptions) -> Result<TypeResult>`

智能输入文本。

**参数：**
```typescript
interface TypeOptions {
  delay?: number;        // 按键间延迟(ms)
  clear_first?: boolean; // 先清空
  press_enter?: boolean; // 输入后按回车
}
```

**返回：**
```typescript
interface TypeResult {
  success: boolean;
  element_found: boolean;
  text_entered: string;
  input_element: ElementInfo;
}
```

#### 5. `select_option(selector: String, value: String | String[]) -> Result<SelectResult>`

选择下拉选项。

**参数：**
- `selector` - 选择器
- `value` - 选项值或值数组（多选）

**返回：**
```typescript
interface SelectResult {
  success: boolean;
  selected_values: string[];
  selected_texts: string[];
}
```

### 同步类工具

#### 6. `wait_for_element(selector: String, options?: WaitOptions) -> Result<Element>`

等待元素出现。

**参数：**
```typescript
interface WaitOptions {
  timeout?: number;      // 超时(ms)，默认 30000
  visible?: boolean;     // 等待可见，默认 true
  hidden?: boolean;      // 等待隐藏
}
```

**返回：** `Result<Element>` - 找到的元素

#### 7. `wait_for_condition(condition: WaitCondition, timeout?: number) -> Result<bool>`

等待条件满足。

**参数：**
```rust
pub enum WaitCondition {
    UrlContains(String),
    UrlEquals(String),
    TitleContains(String),
    ElementVisible(String),
    ElementHidden(String),
    Custom(String), // JavaScript 表达式
}
```

### 记忆类工具

#### 8. `get_element_info(selector: String) -> Result<ElementInfo>`

获取元素详细信息。

**返回：**
```typescript
interface ElementInfo {
  tag_name: string;
  text_content: string;
  attributes: Record<string, string>;
  bounding_box: BoundingBox;
  is_visible: boolean;
  is_enabled: boolean;
  computed_style: Record<string, string>;
  unique_id: string;  // 系统注入的唯一 ID
}
```

#### 9. `take_screenshot(options?: ScreenshotOptions) -> Result<Screenshot>`

智能截图。

**参数：**
```typescript
interface ScreenshotOptions {
  full_page?: boolean;     // 全页截图
  element?: string;        // 元素选择器
  quality?: number;        // 图片质量 (0-100)
  format?: 'png' | 'jpeg';
}
```

**返回：**
```typescript
interface Screenshot {
  data: Uint8Array;      // 图片数据
  width: number;
  height: number;
  format: string;
  size: number;          // 字节数
}
```

#### 10. `retrieve_history(options?: HistoryOptions) -> Result<HistoryData>`

检索历史记录。

**参数：**
```typescript
interface HistoryOptions {
  limit?: number;        // 返回条数
  time_range?: {
    start: Date;
    end: Date;
  };
  filter?: {
    url_pattern?: string;
    action_type?: ActionType[];
  };
}
```

### 元认知类工具

#### 11. `report_insight(insight: InsightData) -> Result<()>`

报告 AI 洞察。

**参数：**
```typescript
interface InsightData {
  type: 'pattern' | 'anomaly' | 'suggestion' | 'prediction';
  confidence: number;    // 0-1
  description: string;
  evidence: any[];
  timestamp: Date;
}
```

#### 12. `complete_task(result: TaskResult) -> Result<()>`

标记任务完成。

**参数：**
```typescript
interface TaskResult {
  task_id: string;
  status: 'success' | 'failure' | 'partial';
  output: any;
  error?: string;
  metrics: {
    duration: number;
    actions_count: number;
    perception_calls: number;
  };
}
```

## 🔒 错误处理

### 错误类型

```rust
#[derive(Debug, Error)]
pub enum BrowserError {
    #[error("Navigation failed: {0}")]
    NavigationError(String),
    
    #[error("Element not found: {0}")]
    ElementNotFound(String),
    
    #[error("Timeout: {0}")]
    Timeout(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Invalid selector: {0}")]
    InvalidSelector(String),
    
    // ... 更多错误类型
}
```

### 错误处理示例

```rust
match page.click("#submit-button").await {
    Ok(result) => println!("Click successful"),
    Err(BrowserError::ElementNotFound(selector)) => {
        eprintln!("Element not found: {}", selector);
        // 实施降级策略
    }
    Err(BrowserError::Timeout(_)) => {
        // 重试逻辑
    }
    Err(e) => return Err(e),
}
```

## 📊 性能指标

### API 响应时间承诺

| API 类别 | 目标响应时间 | P99 延迟 |
|---------|------------|---------|
| 导航类 | <100ms* | <200ms |
| 交互类 | <50ms | <100ms |
| 同步类 | 可配置 | - |
| 记忆类 | <20ms | <50ms |
| 元认知类 | <10ms | <20ms |

*不包括网络延迟

### 并发性能

- 最大并发页面数：100
- 单页面最大并发操作：10
- 任务队列吞吐量：>100 tasks/s

## 🔄 WebSocket API

用于实时事件流和双向通信。

```typescript
const ws = browser.connect_websocket();

ws.on('perception_update', (data: PerceptionResult) => {
  console.log('New perception data:', data);
});

ws.on('action_executed', (result: ActionResult) => {
  console.log('Action completed:', result);
});

// 发送命令
ws.send('execute_action', {
  action: 'click',
  selector: '#button'
});
```

## 🎯 最佳实践

1. **使用适当的感知模式**
   ```python
   # 快速检查使用 Fast 模式
   perception = await page.perceive(PerceptionMode.Fast)
   
   # 复杂分析使用 Deep 模式
   deep_analysis = await page.perceive(PerceptionMode.Deep)
   ```

2. **错误处理与重试**
   ```rust
   let result = retry_with_backoff(|| async {
       page.click("#dynamic-button").await
   }, 3, Duration::from_secs(1)).await?;
   ```

3. **资源管理**
   ```python
   async with Browser() as browser:
       # 自动清理资源
       page = await browser.new_page()
       # ... 使用页面
   # 页面和浏览器自动关闭
   ```

4. **性能优化**
   ```rust
   // 批量操作
   let actions = vec![
       Action::Click { selector: "#btn1".into() },
       Action::TypeText { selector: "#input1".into(), text: "Hello".into() },
   ];
   let results = page.execute_batch(actions).await?;
   ```

## 📝 更新日志

### V8.0.0 (2024-03)
- 初始版本发布
- 12个标准工具
- 三层感知系统
- PyO3 Python 绑定

---

完整的 API 文档和示例请访问：[https://docs.rainbow-browser.ai](https://docs.rainbow-browser.ai)