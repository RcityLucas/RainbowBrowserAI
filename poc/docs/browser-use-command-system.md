# Browser-Use LLM 命令执行系统详解

## 1. 命令执行流程

### 1.1 整体执行流程

```
用户指令
   ↓
LLM 理解与规划
   ↓
生成动作序列 (ActionOutput)
   ↓
Controller 验证与分发
   ↓
具体动作执行 (Action Functions)
   ↓
返回执行结果 (ActionResult)
```

### 1.2 LLM 生成命令的格式

LLM 会根据任务生成一个包含动作列表的 JSON 格式输出：

```json
{
  "action": [
    {
      "action_name": "go_to_url",
      "parameters": {
        "url": "https://example.com"
      }
    },
    {
      "action_name": "click_element",
      "parameters": {
        "selector": "button.submit"
      }
    }
  ]
}
```

## 2. 内置命令列表

Browser-Use 提供了一套丰富的内置命令，涵盖了常见的浏览器操作：

### 2.1 导航类命令

```python
# 搜索 Google
@registry.action('Search Google', param_model=SearchGoogleAction)
async def search_google(params: SearchGoogleAction, browser: BrowserContext)
# 参数：query (搜索关键词)

# 访问 URL
@registry.action('Navigate to URL in the current tab', param_model=GoToUrlAction)
async def go_to_url(params: GoToUrlAction, browser: BrowserContext)
# 参数：url (目标网址)

# 后退
@registry.action('Go back', param_model=NoParamsAction)
async def go_back(_: NoParamsAction, browser: BrowserContext)

# 前进
@registry.action('Go forward', param_model=NoParamsAction)
async def go_forward(_: NoParamsAction, browser: BrowserContext)

# 刷新页面
@registry.action('Refresh the page', param_model=NoParamsAction)
async def refresh_page(_: NoParamsAction, browser: BrowserContext)
```

### 2.2 交互类命令

```python
# 点击元素
@registry.action('Click element', param_model=ClickElementAction)
async def click_element(params: ClickElementAction, browser: BrowserContext)
# 参数：selector (元素选择器)

# 输入文本
@registry.action('Input text', param_model=InputTextAction)
async def input_text(params: InputTextAction, browser: BrowserContext)
# 参数：selector, text

# 发送按键
@registry.action('Send keyboard keys (keys like Escape, Enter ...)', param_model=SendKeysAction)
async def send_keys(params: SendKeysAction, browser: BrowserContext)
# 参数：keys (按键组合)

# 拖拽操作
@registry.action('Drag and drop element', param_model=DragDropAction)
async def drag_and_drop(params: DragDropAction, browser: BrowserContext)
# 参数：element_source, element_target 或 source_x, source_y, target_x, target_y
```

### 2.3 页面滚动命令

```python
# 向下滚动
@registry.action('Scroll down the page by pixel amount - if no amount is specified, scroll down one page', 
                param_model=ScrollAction)
async def scroll_down(params: ScrollAction, browser: BrowserContext)
# 参数：amount (可选，滚动像素数)

# 向上滚动
@registry.action('Scroll up the page by pixel amount - if no amount is specified, scroll up one page',
                param_model=ScrollAction)
async def scroll_up(params: ScrollAction, browser: BrowserContext)

# 滚动到元素
@registry.action('Scroll to element', param_model=ClickElementAction)
async def scroll_to_element(params: ClickElementAction, browser: BrowserContext)
```

### 2.4 标签页管理

```python
# 打开新标签页
@registry.action('Open a new tab', param_model=OpenTabAction)
async def open_tab(params: OpenTabAction, browser: BrowserContext)
# 参数：url (可选)

# 切换标签页
@registry.action('Switch to tab', param_model=SwitchTabAction)
async def switch_tab(params: SwitchTabAction, browser: BrowserContext)
# 参数：tab_index 或 tab_id

# 关闭标签页
@registry.action('Close the current tab', param_model=CloseTabAction)
async def close_tab(params: CloseTabAction, browser: BrowserContext)
```

### 2.5 内容提取命令

```python
# 提取整页内容
@registry.action('Extract information from the page (all visible text)')
async def extract_content()

# 获取页面信息
@registry.action('Get page information')
async def get_page_info()
# 返回：title, url, content

# 截图
@registry.action('Take screenshot of the current page or a specific element')
async def take_screenshot(params: ScreenshotAction)
# 参数：selector (可选，指定元素)
```

### 2.6 特殊领域命令

```python
# Google Sheets 相关
@registry.action('Google Sheets: Input text into the currently selected cell', domains=['sheets.google.com'])
async def input_selected_cell_text(browser: BrowserContext, text: str)

@registry.action('Google Sheets: Select a specific cell or range of cells', domains=['sheets.google.com'])
async def select_cell_or_range(browser: BrowserContext, cell_or_range: str)

# 更多 Google Sheets 操作...
```

### 2.7 任务完成命令

```python
# 标记任务完成
@registry.action('Complete task - when task is done, use this action to signal completion')
async def done(params: DoneAction)
# 参数：success (布尔值), data (可选，返回数据)
```

## 3. 命令限制机制

### 3.1 参数验证

每个命令都有严格的参数定义，使用 Pydantic 模型进行验证：

```python
class ClickElementAction(ActionModel):
    """点击操作的参数模型"""
    selector: str = Field(..., description="CSS选择器或元素索引")
    
class InputTextAction(ActionModel):
    """输入文本的参数模型"""
    selector: str
    text: str
    
class ScrollAction(ActionModel):
    """滚动操作的参数模型"""
    amount: Optional[int] = Field(None, description="滚动像素数")
```

### 3.2 域名限制

某些命令只在特定域名下可用：

```python
@registry.action('Google Sheets 操作', domains=['sheets.google.com'])
async def google_sheets_action(...)
    # 只在 Google Sheets 页面可用
```

### 3.3 页面过滤器

可以使用自定义函数进一步限制命令的可用性：

```python
async def is_ai_allowed(page: Page):
    """检查当前页面是否允许 AI 操作"""
    if check_some_condition(page.url):
        return True
    return False

@registry.action('敏感操作', 
                allowed_domains=['https://*.example.com'],
                page_filter=is_ai_allowed)
```

## 4. 自定义命令扩展

### 4.1 简单自定义命令

```python
from browser_use import Controller, ActionResult

controller = Controller()

@controller.action('保存数据到文件')
def save_to_file(content: str, filename: str) -> ActionResult:
    """将内容保存到指定文件"""
    with open(filename, 'w') as f:
        f.write(content)
    return ActionResult(
        extracted_content=f"已保存到 {filename}",
        include_in_memory=True
    )
```

### 4.2 带浏览器交互的命令

```python
@controller.action('检查元素是否存在')
async def check_element_exists(selector: str, page: Page) -> ActionResult:
    """检查页面上是否存在指定元素"""
    try:
        element = await page.query_selector(selector)
        exists = element is not None
        return ActionResult(
            extracted_content=f"元素 {selector} {'存在' if exists else '不存在'}",
            success=exists
        )
    except Exception as e:
        return ActionResult(error=str(e))
```

### 4.3 使用 Pydantic 模型的复杂命令

```python
from pydantic import BaseModel, Field

class UploadFileParams(BaseModel):
    file_path: str = Field(..., description="要上传的文件路径")
    selector: str = Field(..., description="文件输入框的选择器")
    wait_time: int = Field(3, description="上传后等待时间(秒)")

@controller.action('上传文件', param_model=UploadFileParams)
async def upload_file(params: UploadFileParams, page: Page) -> ActionResult:
    """处理文件上传"""
    try:
        # 找到文件输入元素
        file_input = await page.query_selector(params.selector)
        if not file_input:
            return ActionResult(error="找不到文件输入框")
        
        # 设置文件
        await file_input.set_input_files(params.file_path)
        
        # 等待上传
        await page.wait_for_timeout(params.wait_time * 1000)
        
        return ActionResult(
            extracted_content=f"成功上传文件: {params.file_path}",
            include_in_memory=True
        )
    except Exception as e:
        return ActionResult(error=f"上传失败: {str(e)}")
```

## 5. 命令执行的安全限制

### 5.1 排除危险命令

```python
# 初始化时排除某些命令
controller = Controller(exclude_actions=['search_google', 'open_tab'])
```

### 5.2 执行权限控制

```python
# 特殊参数注入
@controller.action('需要权限的操作')
async def sensitive_action(
    params: ActionParams,
    has_sensitive_data: bool,  # 框架注入的参数
    context: AgentContext      # 用户上下文
) -> ActionResult:
    if has_sensitive_data:
        return ActionResult(error="检测到敏感数据，操作已取消")
    
    # 检查用户权限
    if not context.user.has_permission('sensitive_ops'):
        return ActionResult(error="权限不足")
    
    # 执行操作...
```

### 5.3 执行超时控制

```python
# Agent 级别的步骤限制
agent = Agent(
    task="执行任务",
    llm=llm,
    max_steps=20  # 最多执行 20 个动作
)
```

## 6. 命令选择机制

### 6.1 LLM 如何选择命令

LLM 基于以下信息选择合适的命令：

1. **命令名称和描述**
   ```python
   @controller.action('点击提交按钮 - 用于表单提交')
   ```

2. **参数类型和描述**
   ```python
   class MyParams(BaseModel):
       selector: str = Field(..., description="按钮的CSS选择器")
   ```

3. **当前页面状态**
   - URL
   - 页面标题
   - 可交互元素列表

4. **执行历史**
   - 之前执行的动作
   - 执行结果

### 6.2 提高命令选择准确性的技巧

1. **清晰的命令命名**
   ```python
   # 好的命名
   @controller.action('填写登录表单的用户名')
   
   # 不好的命名
   @controller.action('操作1')
   ```

2. **详细的参数描述**
   ```python
   class LoginParams(BaseModel):
       username: str = Field(..., description="用户名或邮箱地址")
       password: str = Field(..., description="登录密码")
       remember_me: bool = Field(False, description="是否记住登录状态")
   ```

3. **合理的域名限制**
   ```python
   @controller.action('GitHub 专用操作', domains=['github.com', '*.github.com'])
   ```

## 7. 错误处理和重试

### 7.1 命令级错误处理

```python
@controller.action('可重试的操作')
async def retryable_action(params: ActionParams, browser: BrowserContext) -> ActionResult:
    max_retries = 3
    for attempt in range(max_retries):
        try:
            # 执行操作
            result = await perform_operation()
            return ActionResult(success=True, extracted_content=result)
        except Exception as e:
            if attempt == max_retries - 1:
                return ActionResult(error=f"重试 {max_retries} 次后仍失败: {str(e)}")
            await asyncio.sleep(1)  # 重试前等待
```

### 7.2 Agent 级错误恢复

Agent 会自动处理某些错误情况：
- 元素未找到：等待元素出现
- 页面加载超时：重新加载
- 动作执行失败：尝试替代方案

## 总结

Browser-Use 的命令系统设计得非常灵活和强大：

1. **丰富的内置命令**：覆盖了绝大部分浏览器操作需求
2. **严格的参数验证**：确保命令执行的安全性和可靠性
3. **灵活的扩展机制**：可以轻松添加自定义命令
4. **智能的限制机制**：通过域名、页面过滤器等多种方式控制命令可用性
5. **完善的错误处理**：支持重试和错误恢复

这种设计使得 LLM 能够安全、准确地控制浏览器，同时给开发者留出了充分的自定义空间。