# 彩虹城浏览器 V8.0 标准工具集文档

## 📋 概述

彩虹城浏览器 V8.0 提供 12 个标准化工具，为 AI 在数字世界中的所有核心操作提供统一、智能、可预测的接口。这些工具覆盖导航、交互、同步、记忆和元认知五大类别。

### 设计原则

1. **统一参数模式**：所有工具使用一致的参数结构
2. **智能错误处理**：自动重试和降级策略
3. **性能监控**：内置性能指标收集
4. **类型安全**：完整的 TypeScript 类型定义

## 🧭 导航类工具

### 1. navigate_to_url

**用途**：智能导航到指定 URL，支持重定向处理、页面加载等待策略。

#### 参数定义

```typescript
interface NavigateToUrlParams {
  url: string;                     // 目标 URL
  options?: {
    wait_until?: WaitUntil;        // 等待策略
    timeout?: number;              // 超时时间（毫秒）
    retry?: boolean;               // 失败时自动重试
    headers?: Record<string, string>; // 自定义请求头
    referrer?: string;             // 来源页面
  };
}

type WaitUntil = 
  | 'load'              // 页面 load 事件
  | 'domcontentloaded'  // DOM 内容加载完成
  | 'networkidle0'      // 网络空闲（0个连接）
  | 'networkidle2';     // 网络空闲（最多2个连接）
```

#### 返回值

```typescript
interface NavigateResult {
  success: boolean;
  final_url: string;               // 最终 URL（可能重定向）
  load_time: number;               // 加载时间（毫秒）
  status_code: number;             // HTTP 状态码
  redirects: RedirectInfo[];       // 重定向链
  
  performance: {
    dns_lookup: number;
    tcp_connect: number;
    request_sent: number;
    response_received: number;
    dom_loaded: number;
    page_loaded: number;
  };
  
  error?: {
    code: string;
    message: string;
    details: any;
  };
}
```

#### 使用示例

```python
# 基础导航
result = await browser.navigate_to_url("https://example.com")

# 高级导航（等待网络空闲）
result = await browser.navigate_to_url(
    "https://spa-app.com",
    options={
        "wait_until": "networkidle0",
        "timeout": 30000,
        "retry": True
    }
)

# 带自定义头的导航
result = await browser.navigate_to_url(
    "https://api.example.com",
    options={
        "headers": {
            "Authorization": "Bearer token123",
            "X-Custom-Header": "value"
        }
    }
)
```

#### 错误处理

```python
try:
    result = await browser.navigate_to_url("https://example.com")
    if not result.success:
        print(f"导航失败: {result.error.message}")
        # 处理特定错误
        if result.error.code == "TIMEOUT":
            # 增加超时时间重试
            result = await browser.navigate_to_url(
                url, 
                options={"timeout": 60000}
            )
except NavigationError as e:
    # 处理导航异常
    logger.error(f"导航异常: {e}")
```

### 2. scroll_page

**用途**：智能滚动页面，支持多种滚动模式和目标。

#### 参数定义

```typescript
interface ScrollPageParams {
  direction: ScrollDirection;
  amount?: number;                 // 滚动距离（像素）
  options?: {
    smooth?: boolean;              // 平滑滚动
    duration?: number;             // 滚动持续时间（毫秒）
    wait_after?: number;           // 滚动后等待时间
  };
}

type ScrollDirection = 
  | 'up' 
  | 'down' 
  | 'left' 
  | 'right'
  | 'top'                         // 滚动到顶部
  | 'bottom'                      // 滚动到底部
  | { to_element: string }        // 滚动到特定元素
  | { to_position: Position };    // 滚动到特定位置

interface Position {
  x: number;
  y: number;
}
```

#### 返回值

```typescript
interface ScrollResult {
  success: boolean;
  previous_position: Position;
  current_position: Position;
  viewport: {
    width: number;
    height: number;
  };
  document: {
    width: number;
    height: number;
  };
  reached_boundary: {
    top: boolean;
    bottom: boolean;
    left: boolean;
    right: boolean;
  };
}
```

#### 使用示例

```python
# 向下滚动 500 像素
await browser.scroll_page("down", amount=500)

# 平滑滚动到底部
await browser.scroll_page(
    "bottom",
    options={"smooth": True, "duration": 1000}
)

# 滚动到特定元素
await browser.scroll_page(
    {"to_element": "#comments-section"},
    options={"smooth": True, "wait_after": 500}
)

# 无限滚动加载
async def scroll_until_loaded():
    while True:
        result = await browser.scroll_page("down", amount=1000)
        if result.reached_boundary.bottom:
            break
        await asyncio.sleep(1)  # 等待新内容加载
```

## 🎯 交互类工具

### 3. click

**用途**：精准点击页面元素，支持多种点击模式和智能等待。

#### 参数定义

```typescript
interface ClickParams {
  selector: string;                // CSS 选择器或元素 ID
  options?: {
    button?: MouseButton;          // 鼠标按钮
    click_count?: number;          // 点击次数（双击等）
    delay?: number;                // 点击前延迟（毫秒）
    offset?: Offset;               // 相对元素的偏移
    force?: boolean;               // 强制点击（忽略遮挡）
    wait_for?: WaitForOptions;     // 点击前等待条件
    modifiers?: Modifier[];        // 键盘修饰键
  };
}

type MouseButton = 'left' | 'right' | 'middle';
type Modifier = 'Alt' | 'Control' | 'Meta' | 'Shift';

interface Offset {
  x: number;                       // 水平偏移
  y: number;                       // 垂直偏移
}
```

#### 返回值

```typescript
interface ClickResult {
  success: boolean;
  element: {
    tag_name: string;
    text: string;
    attributes: Record<string, string>;
    bounding_box: BoundingBox;
  };
  
  effects: {
    navigation_triggered: boolean;
    form_submitted: boolean;
    popup_opened: boolean;
    dom_changed: boolean;
  };
  
  timing: {
    element_found: number;         // 找到元素耗时
    click_executed: number;        // 执行点击耗时
    total: number;                 // 总耗时
  };
}
```

#### 使用示例

```python
# 基础点击
await browser.click("#submit-button")

# 右键点击
await browser.click(
    "#context-menu-target",
    options={"button": "right"}
)

# 双击
await browser.click(
    ".editable-cell",
    options={"click_count": 2}
)

# 带偏移的点击（点击元素右上角）
element_info = await browser.get_element_info("#large-button")
await browser.click(
    "#large-button",
    options={
        "offset": {
            "x": element_info.bounding_box.width - 10,
            "y": 10
        }
    }
)

# 带修饰键的点击（Ctrl+Click）
await browser.click(
    ".multi-select-item",
    options={"modifiers": ["Control"]}
)
```

#### 高级用法

```python
# 智能重试点击
async def smart_click(selector, max_retries=3):
    for i in range(max_retries):
        try:
            result = await browser.click(
                selector,
                options={
                    "wait_for": {"visible": True, "enabled": True},
                    "force": i == max_retries - 1  # 最后一次强制点击
                }
            )
            if result.success:
                return result
        except ElementNotFound:
            if i < max_retries - 1:
                await asyncio.sleep(1)
            else:
                raise
```

### 4. type_text

**用途**：智能输入文本，支持特殊键、清空输入框等功能。

#### 参数定义

```typescript
interface TypeTextParams {
  selector: string;                // 输入框选择器
  text: string;                    // 要输入的文本
  options?: {
    delay?: number;                // 按键间延迟（毫秒）
    clear_first?: boolean;         // 先清空输入框
    press_enter?: boolean;         // 输入后按回车
    select_all?: boolean;          // 先全选文本
    paste?: boolean;               // 使用粘贴而非逐字输入
    validate?: (value: string) => boolean; // 输入验证
  };
}
```

#### 返回值

```typescript
interface TypeTextResult {
  success: boolean;
  input_element: {
    type: string;                  // input类型
    name: string;
    id: string;
    initial_value: string;         // 输入前的值
    final_value: string;           // 输入后的值
  };
  
  validation: {
    passed: boolean;
    message?: string;
  };
  
  timing: {
    clear_time?: number;
    type_time: number;
    total: number;
  };
}
```

#### 使用示例

```python
# 基础输入
await browser.type_text("#username", "john_doe")

# 清空并输入
await browser.type_text(
    "#search-box",
    "AI Browser",
    options={"clear_first": True}
)

# 输入并提交
await browser.type_text(
    "#search-input",
    "Rainbow Browser features",
    options={
        "clear_first": True,
        "press_enter": True,
        "delay": 50  # 模拟人类输入速度
    }
)

# 使用粘贴（适合长文本）
long_text = "这是一段很长的文本..." * 100
await browser.type_text(
    "#content-editor",
    long_text,
    options={"paste": True}
)

# 带验证的输入
await browser.type_text(
    "#email-input",
    "user@example.com",
    options={
        "validate": lambda v: "@" in v and "." in v
    }
)
```

### 5. select_option

**用途**：选择下拉框选项，支持单选和多选。

#### 参数定义

```typescript
interface SelectOptionParams {
  selector: string;                // select 元素选择器
  value: string | string[];        // 选项值（单选或多选）
  options?: {
    by?: SelectBy;                 // 选择方式
    deselect_others?: boolean;     // 先取消其他选择
    wait_for_options?: boolean;    // 等待选项加载
  };
}

type SelectBy = 
  | 'value'                        // 通过 value 属性
  | 'text'                         // 通过显示文本
  | 'index';                       // 通过索引
```

#### 返回值

```typescript
interface SelectOptionResult {
  success: boolean;
  select_element: {
    name: string;
    id: string;
    multiple: boolean;             // 是否多选
    option_count: number;
  };
  
  selected: {
    values: string[];
    texts: string[];
    indices: number[];
  };
  
  deselected: {
    values: string[];
    texts: string[];
  };
}
```

#### 使用示例

```python
# 通过值选择
await browser.select_option("#country", "CN")

# 通过文本选择
await browser.select_option(
    "#language",
    "简体中文",
    options={"by": "text"}
)

# 多选
await browser.select_option(
    "#interests",
    ["sports", "music", "reading"]
)

# 动态加载的选项
await browser.select_option(
    "#city",
    "Shanghai",
    options={
        "wait_for_options": True,
        "by": "text"
    }
)
```

## ⏳ 同步类工具

### 6. wait_for_element

**用途**：等待元素出现、可见或满足特定条件。

#### 参数定义

```typescript
interface WaitForElementParams {
  selector: string;
  options?: {
    state?: ElementState;          // 等待的状态
    timeout?: number;              // 超时时间（毫秒）
    poll_interval?: number;        // 轮询间隔
    visible?: boolean;             // 等待可见
    enabled?: boolean;             // 等待可用
    has_text?: string | RegExp;    // 等待包含文本
  };
}

type ElementState = 
  | 'attached'     // 元素存在于 DOM
  | 'detached'     // 元素不存在
  | 'visible'      // 元素可见
  | 'hidden';      // 元素隐藏
```

#### 返回值

```typescript
interface WaitForElementResult {
  success: boolean;
  element?: ElementInfo;
  wait_time: number;               // 实际等待时间
  
  state_changes: {
    timestamp: string;
    from_state: ElementState;
    to_state: ElementState;
  }[];
}
```

#### 使用示例

```python
# 等待元素出现
element = await browser.wait_for_element("#dynamic-content")

# 等待元素可见且包含特定文本
element = await browser.wait_for_element(
    ".notification",
    options={
        "visible": True,
        "has_text": "操作成功",
        "timeout": 5000
    }
)

# 等待元素消失
await browser.wait_for_element(
    ".loading-spinner",
    options={
        "state": "detached",
        "timeout": 10000
    }
)

# 自定义轮询间隔
await browser.wait_for_element(
    "#slow-loading",
    options={
        "timeout": 30000,
        "poll_interval": 500  # 每500ms检查一次
    }
)
```

### 7. wait_for_condition

**用途**：等待自定义条件满足，支持 JavaScript 表达式。

#### 参数定义

```typescript
interface WaitForConditionParams {
  condition: WaitCondition | string;  // 条件或JS表达式
  options?: {
    timeout?: number;
    poll_interval?: number;
    args?: any[];                    // 传递给JS表达式的参数
  };
}

type WaitCondition = 
  | { url_contains: string }
  | { url_equals: string }
  | { url_matches: RegExp }
  | { title_contains: string }
  | { title_equals: string }
  | { element_count: { selector: string; count: number } }
  | { custom: string };              // 自定义JS表达式
```

#### 返回值

```typescript
interface WaitForConditionResult {
  success: boolean;
  condition_met: boolean;
  final_value: any;                  // 条件表达式的最终值
  wait_time: number;
  
  evaluation_history: {
    timestamp: string;
    value: any;
    met: boolean;
  }[];
}
```

#### 使用示例

```python
# 等待 URL 变化
await browser.wait_for_condition(
    {"url_contains": "/success"}
)

# 等待标题包含特定文本
await browser.wait_for_condition(
    {"title_contains": "支付成功"}
)

# 等待特定数量的元素
await browser.wait_for_condition({
    "element_count": {
        "selector": ".search-result",
        "count": 10
    }
})

# 自定义 JavaScript 条件
await browser.wait_for_condition({
    "custom": """
        return document.querySelectorAll('.item').length >= 5 &&
               window.dataLoaded === true
    """
})

# 带参数的条件
await browser.wait_for_condition({
    "custom": "return window.orderStatus === arguments[0]",
    "options": {
        "args": ["completed"],
        "timeout": 20000
    }
})
```

## 🧠 记忆类工具

### 8. get_element_info

**用途**：获取元素的详细信息，包括属性、样式、位置等。

#### 参数定义

```typescript
interface GetElementInfoParams {
  selector: string;
  options?: {
    include_computed_style?: boolean;  // 包含计算样式
    include_attributes?: boolean;      // 包含所有属性
    include_dataset?: boolean;         // 包含 data-* 属性
    include_screenshot?: boolean;      // 包含元素截图
    traverse_children?: boolean;       // 遍历子元素
    max_depth?: number;               // 遍历深度
  };
}
```

#### 返回值

```typescript
interface ElementInfo {
  // 基础信息
  tag_name: string;
  text_content: string;
  inner_text: string;
  inner_html: string;
  outer_html: string;
  
  // 标识信息
  id: string;
  class_list: string[];
  unique_id: string;                  // 系统生成的唯一ID
  
  // 位置信息
  bounding_box: {
    x: number;
    y: number;
    width: number;
    height: number;
    top: number;
    right: number;
    bottom: number;
    left: number;
  };
  
  // 状态信息
  is_visible: boolean;
  is_enabled: boolean;
  is_selected: boolean;
  is_focused: boolean;
  is_in_viewport: boolean;
  
  // 属性
  attributes?: Record<string, string>;
  dataset?: Record<string, string>;
  computed_style?: Record<string, string>;
  
  // 关系
  parent?: ElementInfo;
  children?: ElementInfo[];
  siblings?: ElementInfo[];
  
  // 元素特定信息
  input_type?: string;
  input_value?: string;
  href?: string;
  src?: string;
  
  // 截图
  screenshot?: {
    format: string;
    data: string;                    // Base64
  };
}
```

#### 使用示例

```python
# 获取基础信息
info = await browser.get_element_info("#user-profile")
print(f"用户名: {info.text_content}")

# 获取完整信息
full_info = await browser.get_element_info(
    ".product-card",
    options={
        "include_computed_style": True,
        "include_attributes": True,
        "include_screenshot": True
    }
)

# 分析元素可见性
def is_truly_visible(element_info):
    if not element_info.is_visible:
        return False
    
    style = element_info.computed_style
    if style.get("display") == "none":
        return False
    if float(style.get("opacity", "1")) == 0:
        return False
    
    bbox = element_info.bounding_box
    if bbox.width == 0 or bbox.height == 0:
        return False
        
    return element_info.is_in_viewport

# 遍历表格数据
table_info = await browser.get_element_info(
    "#data-table",
    options={
        "traverse_children": True,
        "max_depth": 3
    }
)

# 提取表格数据
rows = []
for tr in table_info.children:
    if tr.tag_name == "tr":
        row = []
        for td in tr.children:
            if td.tag_name in ["td", "th"]:
                row.append(td.text_content)
        rows.append(row)
```

### 9. take_screenshot

**用途**：智能截图，支持全页面、视口、元素级截图。

#### 参数定义

```typescript
interface TakeScreenshotParams {
  options?: {
    type?: ScreenshotType;
    format?: ImageFormat;
    quality?: number;              // 0-100，仅JPEG
    full_page?: boolean;           // 全页面截图
    clip?: Rectangle;              // 裁剪区域
    element?: string;              // 元素选择器
    omit_background?: boolean;     // 透明背景
    highlights?: Highlight[];      // 高亮区域
  };
}

type ScreenshotType = 
  | 'viewport'     // 仅可见区域
  | 'full_page'    // 整个页面
  | 'element';     // 特定元素

type ImageFormat = 'png' | 'jpeg' | 'webp';

interface Rectangle {
  x: number;
  y: number;
  width: number;
  height: number;
}

interface Highlight {
  selector: string;
  style?: {
    border?: string;
    background?: string;
    opacity?: number;
  };
}
```

#### 返回值

```typescript
interface Screenshot {
  format: ImageFormat;
  data: Uint8Array;                // 二进制数据
  data_url: string;                // data:image/png;base64,...
  
  dimensions: {
    width: number;
    height: number;
  };
  
  file_size: number;               // 字节
  
  metadata: {
    timestamp: string;
    url: string;
    device_pixel_ratio: number;
    color_space: string;
  };
}
```

#### 使用示例

```python
# 基础截图（视口）
screenshot = await browser.take_screenshot()
with open("viewport.png", "wb") as f:
    f.write(screenshot.data)

# 全页面截图
full_screenshot = await browser.take_screenshot(
    options={"full_page": True}
)

# 元素截图
element_screenshot = await browser.take_screenshot(
    options={
        "element": "#product-image",
        "format": "jpeg",
        "quality": 85
    }
)

# 带高亮的截图
highlighted_screenshot = await browser.take_screenshot(
    options={
        "full_page": True,
        "highlights": [
            {
                "selector": ".important-section",
                "style": {
                    "border": "3px solid red",
                    "background": "rgba(255,0,0,0.1)"
                }
            }
        ]
    }
)

# 批量截图比较
async def capture_responsive_views():
    viewports = [
        {"width": 375, "height": 667},   # iPhone
        {"width": 768, "height": 1024},  # iPad
        {"width": 1920, "height": 1080}  # Desktop
    ]
    
    screenshots = []
    for viewport in viewports:
        await browser.set_viewport(viewport)
        screenshot = await browser.take_screenshot()
        screenshots.append({
            "viewport": viewport,
            "screenshot": screenshot
        })
    
    return screenshots
```

### 10. retrieve_history

**用途**：检索浏览历史和操作记录。

#### 参数定义

```typescript
interface RetrieveHistoryParams {
  options?: {
    type?: HistoryType;
    limit?: number;                // 返回条数
    offset?: number;               // 偏移量
    
    time_range?: {
      start?: string;              // ISO 8601
      end?: string;
    };
    
    filters?: {
      url_pattern?: string | RegExp;
      action_types?: ActionType[];
      status?: 'success' | 'failure';
      min_duration?: number;       // 最小耗时
    };
    
    include?: {
      perceptions?: boolean;       // 包含感知数据
      screenshots?: boolean;       // 包含截图
      performance?: boolean;       // 包含性能数据
    };
    
    sort?: {
      field: 'timestamp' | 'duration' | 'url';
      order: 'asc' | 'desc';
    };
  };
}

type HistoryType = 
  | 'navigation'    // 导航历史
  | 'action'        // 操作历史
  | 'perception'    // 感知历史
  | 'all';          // 所有历史
```

#### 返回值

```typescript
interface HistoryData {
  entries: HistoryEntry[];
  
  summary: {
    total_count: number;
    filtered_count: number;
    time_span: {
      start: string;
      end: string;
    };
    
    statistics: {
      avg_duration: number;
      success_rate: number;
      most_visited_urls: Array<{
        url: string;
        count: number;
      }>;
      most_used_actions: Array<{
        action: string;
        count: number;
      }>;
    };
  };
  
  pagination: {
    limit: number;
    offset: number;
    has_more: boolean;
  };
}

interface HistoryEntry {
  id: string;
  type: HistoryType;
  timestamp: string;
  
  // 导航历史
  navigation?: {
    from_url: string;
    to_url: string;
    trigger: 'link' | 'form' | 'script' | 'manual';
    duration_ms: number;
  };
  
  // 操作历史
  action?: {
    type: ActionType;
    target: string;
    parameters: any;
    result: 'success' | 'failure';
    error?: string;
  };
  
  // 感知历史
  perception?: {
    mode: PerceptionMode;
    summary: PerceptionSummary;
  };
  
  // 可选数据
  screenshot?: Screenshot;
  performance?: PerformanceMetrics;
}
```

#### 使用示例

```python
# 获取最近的导航历史
nav_history = await browser.retrieve_history(
    options={
        "type": "navigation",
        "limit": 10,
        "sort": {"field": "timestamp", "order": "desc"}
    }
)

# 获取特定时间范围的操作历史
from datetime import datetime, timedelta

action_history = await browser.retrieve_history(
    options={
        "type": "action",
        "time_range": {
            "start": (datetime.now() - timedelta(hours=1)).isoformat(),
            "end": datetime.now().isoformat()
        },
        "filters": {
            "action_types": ["click", "type_text"],
            "status": "success"
        }
    }
)

# 分析访问模式
history = await browser.retrieve_history(
    options={
        "type": "all",
        "limit": 1000,
        "include": {
            "performance": True
        }
    }
)

# 找出访问最频繁的页面
url_visits = {}
for entry in history.entries:
    if entry.navigation:
        url = entry.navigation.to_url
        url_visits[url] = url_visits.get(url, 0) + 1

# 按访问次数排序
sorted_urls = sorted(
    url_visits.items(), 
    key=lambda x: x[1], 
    reverse=True
)
```

## 🎭 元认知类工具

### 11. report_insight

**用途**：报告 AI 发现的洞察、模式或异常。

#### 参数定义

```typescript
interface ReportInsightParams {
  insight: {
    type: InsightType;
    title: string;
    description: string;
    confidence: number;            // 0-1
    severity?: InsightSeverity;
    
    evidence: Evidence[];
    
    context: {
      url?: string;
      timestamp: string;
      session_id?: string;
      related_actions?: string[];  // Action IDs
    };
    
    recommendations?: Recommendation[];
    
    metadata?: Record<string, any>;
  };
}

type InsightType = 
  | 'pattern'          // 行为模式
  | 'anomaly'          // 异常情况
  | 'optimization'     // 优化建议
  | 'prediction'       // 预测
  | 'discovery'        // 新发现
  | 'warning';         // 警告

type InsightSeverity = 
  | 'info' 
  | 'low' 
  | 'medium' 
  | 'high' 
  | 'critical';

interface Evidence {
  type: 'screenshot' | 'data' | 'observation' | 'metric';
  description: string;
  value: any;
  source: string;
}

interface Recommendation {
  action: string;
  reason: string;
  priority: number;               // 1-5
  estimated_impact: string;
}
```

#### 返回值

```typescript
interface InsightResult {
  insight_id: string;
  accepted: boolean;
  
  processing: {
    stored: boolean;
    analyzed: boolean;
    actioned: boolean;
  };
  
  related_insights: string[];     // 相关洞察 IDs
  
  impact: {
    affected_sessions: number;
    affected_users: number;
    potential_savings: {
      time_ms?: number;
      actions?: number;
      resources?: number;
    };
  };
}
```

#### 使用示例

```python
# 报告发现的模式
await browser.report_insight({
    "type": "pattern",
    "title": "重复的登录失败模式",
    "description": "检测到用户在登录时经常输错密码，平均尝试3次",
    "confidence": 0.85,
    "severity": "medium",
    
    "evidence": [
        {
            "type": "data",
            "description": "登录尝试统计",
            "value": {"attempts": [3, 4, 3, 2, 3], "avg": 3},
            "source": "action_history"
        }
    ],
    
    "recommendations": [
        {
            "action": "添加密码可见性切换",
            "reason": "减少输入错误",
            "priority": 4,
            "estimated_impact": "减少50%的重试"
        }
    ]
})

# 报告性能优化建议
await browser.report_insight({
    "type": "optimization",
    "title": "页面加载性能可优化",
    "description": "检测到大量未使用的JavaScript代码",
    "confidence": 0.92,
    
    "evidence": [
        {
            "type": "metric",
            "description": "未使用代码比例",
            "value": {"unused_ratio": 0.67, "size_kb": 450},
            "source": "performance_analysis"
        }
    ],
    
    "recommendations": [
        {
            "action": "实施代码分割",
            "reason": "减少初始加载大小",
            "priority": 5,
            "estimated_impact": "加载时间减少40%"
        }
    ]
})

# 报告异常情况
await browser.report_insight({
    "type": "anomaly",
    "title": "异常的用户行为序列",
    "description": "用户快速点击了50个不同的链接",
    "confidence": 0.95,
    "severity": "high",
    
    "evidence": [
        {
            "type": "observation",
            "description": "点击频率",
            "value": {"clicks_per_second": 5.2},
            "source": "action_monitor"
        }
    ],
    
    "context": {
        "url": "https://example.com/products",
        "timestamp": datetime.now().isoformat()
    }
})
```

### 12. complete_task

**用途**：标记任务完成并报告执行结果。

#### 参数定义

```typescript
interface CompleteTaskParams {
  task_id: string;
  result: {
    status: TaskStatus;
    output?: any;                  // 任务输出
    error?: TaskError;
    
    metrics: {
      duration_ms: number;
      actions_count: number;
      perceptions_count: number;
      retries: number;
      
      resource_usage: {
        cpu_percent: number;
        memory_mb: number;
        network_kb: number;
      };
    };
    
    artifacts?: TaskArtifact[];    // 任务产物
    
    insights?: string[];           // 相关洞察IDs
    
    follow_up?: {
      required: boolean;
      tasks: FollowUpTask[];
      reason?: string;
    };
  };
}

type TaskStatus = 
  | 'success'       // 完全成功
  | 'partial'       // 部分成功
  | 'failure'       // 失败
  | 'cancelled'     // 取消
  | 'timeout';      // 超时

interface TaskError {
  code: string;
  message: string;
  recoverable: boolean;
  retry_after?: number;            // 秒
}

interface TaskArtifact {
  type: 'screenshot' | 'data' | 'report' | 'file';
  name: string;
  content: any;
  mime_type?: string;
}

interface FollowUpTask {
  type: string;
  description: string;
  priority: number;
  deadline?: string;
}
```

#### 返回值

```typescript
interface TaskCompletionResult {
  acknowledged: boolean;
  task_record_id: string;
  
  evaluation: {
    performance_score: number;     // 0-100
    efficiency_score: number;      // 0-100
    quality_score: number;         // 0-100
  };
  
  rewards: {
    experience_gained: number;
    capabilities_unlocked: string[];
  };
  
  next_actions: {
    suggested_tasks: string[];
    learning_opportunities: string[];
  };
}
```

#### 使用示例

```python
# 成功完成任务
result = await browser.complete_task(
    task_id="task_001",
    result={
        "status": "success",
        "output": {
            "extracted_data": product_data,
            "product_count": len(product_data),
            "categories": list(set(p["category"] for p in product_data))
        },
        
        "metrics": {
            "duration_ms": 4500,
            "actions_count": 15,
            "perceptions_count": 3,
            "retries": 1,
            "resource_usage": {
                "cpu_percent": 45,
                "memory_mb": 120,
                "network_kb": 580
            }
        },
        
        "artifacts": [
            {
                "type": "data",
                "name": "products.json",
                "content": json.dumps(product_data),
                "mime_type": "application/json"
            },
            {
                "type": "screenshot",
                "name": "final_state.png",
                "content": screenshot.data
            }
        ]
    }
)

# 部分成功的任务
await browser.complete_task(
    task_id="task_002",
    result={
        "status": "partial",
        "output": {
            "processed_items": 45,
            "failed_items": 5,
            "success_rate": 0.9
        },
        
        "error": {
            "code": "PARTIAL_FAILURE",
            "message": "5个项目处理失败",
            "recoverable": True
        },
        
        "follow_up": {
            "required": True,
            "tasks": [
                {
                    "type": "retry_failed",
                    "description": "重试失败的5个项目",
                    "priority": 4
                }
            ],
            "reason": "部分项目需要特殊处理"
        }
    }
)

# 失败的任务
await browser.complete_task(
    task_id="task_003",
    result={
        "status": "failure",
        "error": {
            "code": "AUTH_REQUIRED",
            "message": "需要登录才能访问",
            "recoverable": True,
            "retry_after": 300
        },
        
        "metrics": {
            "duration_ms": 1200,
            "actions_count": 3,
            "perceptions_count": 1,
            "retries": 0,
            "resource_usage": {
                "cpu_percent": 10,
                "memory_mb": 50,
                "network_kb": 20
            }
        }
    }
)
```

## 🔧 工具组合使用

### 示例1：完整的表单提交流程

```python
async def submit_form_with_validation():
    # 1. 导航到表单页面
    await browser.navigate_to_url("https://example.com/form")
    
    # 2. 等待表单加载
    await browser.wait_for_element("#registration-form")
    
    # 3. 填写表单
    await browser.type_text("#name", "张三", options={"clear_first": True})
    await browser.type_text("#email", "zhangsan@example.com")
    await browser.select_option("#country", "CN")
    
    # 4. 点击同意条款
    await browser.click("#agree-terms")
    
    # 5. 截图存档
    screenshot = await browser.take_screenshot(
        options={"element": "#registration-form"}
    )
    
    # 6. 提交表单
    await browser.click("#submit-button")
    
    # 7. 等待结果
    result = await browser.wait_for_condition({
        "url_contains": "/success"
    })
    
    # 8. 报告完成
    await browser.complete_task(
        task_id="form_submission",
        result={
            "status": "success",
            "artifacts": [{
                "type": "screenshot",
                "name": "form_filled.png",
                "content": screenshot.data
            }]
        }
    )
```

### 示例2：智能数据抓取

```python
async def smart_scraping():
    url = "https://example.com/products"
    products = []
    page = 1
    
    while True:
        # 导航到页面
        await browser.navigate_to_url(f"{url}?page={page}")
        
        # 等待产品加载
        await browser.wait_for_element(".product-card")
        
        # 获取产品信息
        product_cards = await browser.get_element_info(
            ".product-list",
            options={"traverse_children": True}
        )
        
        # 提取数据
        for card in product_cards.children:
            if "product-card" in card.class_list:
                product = {
                    "name": card.querySelector(".product-name").text_content,
                    "price": card.querySelector(".price").text_content,
                    "image": card.querySelector("img").src
                }
                products.append(product)
        
        # 检查是否有下一页
        next_button = await browser.get_element_info(".next-page")
        if not next_button.is_enabled:
            break
            
        # 点击下一页
        await browser.click(".next-page")
        page += 1
        
        # 报告进度
        await browser.report_insight({
            "type": "pattern",
            "title": f"已抓取第{page}页",
            "description": f"当前已收集{len(products)}个产品",
            "confidence": 1.0
        })
    
    return products
```

## 📊 性能基准

| 工具 | 平均响应时间 | P95 响应时间 | 成功率 |
|------|-------------|-------------|--------|
| navigate_to_url | 120ms* | 200ms* | 99.5% |
| scroll_page | 15ms | 30ms | 99.9% |
| click | 25ms | 50ms | 99.7% |
| type_text | 50ms | 100ms | 99.8% |
| select_option | 20ms | 40ms | 99.9% |
| wait_for_element | - | - | 98.5% |
| wait_for_condition | - | - | 99.0% |
| get_element_info | 10ms | 20ms | 99.9% |
| take_screenshot | 80ms | 150ms | 99.5% |
| retrieve_history | 15ms | 30ms | 99.9% |
| report_insight | 5ms | 10ms | 100% |
| complete_task | 5ms | 10ms | 100% |

*不包括网络延迟

## 🎯 最佳实践

1. **合理使用等待策略**
   - 优先使用 `wait_for_element` 而非固定延时
   - 设置合理的超时时间
   - 使用条件等待确保操作时机

2. **错误处理和重试**
   - 实现智能重试机制
   - 记录详细的错误信息
   - 提供降级方案

3. **性能优化**
   - 批量操作减少往返
   - 使用适当的感知模式
   - 缓存重复的查询结果

4. **数据质量**
   - 验证提取的数据
   - 保存操作证据（截图）
   - 报告异常情况

---

**掌握这 12 个工具，让 AI 在数字世界中自由翱翔！** 🚀