# 彩虹城浏览器 V8.0 快速入门

> 🚀 **5分钟内开始使用 AI 驱动的浏览器自动化**

## 📋 前置要求

- **Rust** 1.75+ 或 **Python** 3.9+
- **Docker** 20.10+ (推荐)
- 8GB 内存，10GB 可用磁盘空间

## 🎯 快速安装

### 方式一：Docker 一键部署（推荐）

```bash
# 拉取镜像
docker pull rainbow/browser:v8.0

# 启动服务
docker run -d \
  --name rainbow-browser \
  -p 8080:8080 \
  -v rainbow-data:/data \
  rainbow/browser:v8.0

# 验证安装
curl http://localhost:8080/health
```

### 方式二：本地安装

#### Rust 开发者

```bash
# 添加依赖到 Cargo.toml
[dependencies]
rainbow-browser = "8.0"
tokio = { version = "1", features = ["full"] }

# 或使用 cargo add
cargo add rainbow-browser tokio
```

#### Python 开发者

```bash
# 使用 pip 安装
pip install rainbow-browser

# 或使用 poetry
poetry add rainbow-browser
```

## 🏃 第一个示例

### 示例 1：基础网页操作

#### Rust 版本

```rust
use rainbow_browser::{Browser, Result};

#[tokio::main]
async fn main() -> Result<()> {
    // 创建浏览器实例
    let browser = Browser::new_default().await?;
    let page = browser.new_page().await?;
    
    // 导航到网页
    page.navigate("https://www.example.com").await?;
    
    // 获取页面标题
    let title = page.get_title().await?;
    println!("页面标题: {}", title);
    
    // 截图
    let screenshot = page.take_screenshot().await?;
    std::fs::write("example.png", screenshot.data)?;
    
    // 清理资源
    browser.close().await?;
    Ok(())
}
```

#### Python 版本

```python
import asyncio
from rainbow_browser import Browser

async def main():
    # 使用上下文管理器自动管理资源
    async with Browser() as browser:
        page = await browser.new_page()
        
        # 导航到网页
        await page.navigate("https://www.example.com")
        
        # 获取页面标题
        title = await page.get_title()
        print(f"页面标题: {title}")
        
        # 截图
        screenshot = await page.take_screenshot()
        with open("example.png", "wb") as f:
            f.write(screenshot.data)

# 运行
asyncio.run(main())
```

### 示例 2：智能表单填写

```python
async def fill_form_example():
    async with Browser() as browser:
        page = await browser.new_page()
        await page.navigate("https://example.com/form")
        
        # 使用智能感知识别表单
        perception = await page.perceive(PerceptionMode.Standard)
        
        # 填写表单
        await page.type_text("#username", "test_user")
        await page.type_text("#email", "test@example.com")
        await page.select_option("#country", "CN")
        
        # 点击提交
        await page.click("#submit-button")
        
        # 等待结果
        await page.wait_for_element(".success-message", timeout=5000)
        
        print("表单提交成功！")
```

### 示例 3：AI 驱动的内容提取

```rust
use rainbow_browser::{Browser, PerceptionMode};

async fn extract_content() -> Result<()> {
    let browser = Browser::new_default().await?;
    let page = browser.new_page().await?;
    
    page.navigate("https://news.example.com").await?;
    
    // 使用深度感知模式
    let perception = page.perceive(PerceptionMode::Deep).await?;
    
    // AI 自动识别并提取主要内容
    if let Some(semantic) = perception.semantic {
        println!("文章标题: {}", semantic.main_title);
        println!("主要内容: {}", semantic.main_content);
        println!("作者: {:?}", semantic.author);
        println!("发布时间: {:?}", semantic.publish_date);
    }
    
    Ok(())
}
```

## 🎨 三种感知模式

彩虹城浏览器提供三种感知模式，适用于不同场景：

### Fast 模式 (<50ms)
```python
# 快速获取页面基本信息
perception = await page.perceive(PerceptionMode.Fast)
print(f"页面包含 {len(perception.structure.interactive_elements)} 个可交互元素")
```

### Standard 模式 (<200ms)
```python
# 标准分析，包含结构和视觉信息
perception = await page.perceive(PerceptionMode.Standard)
# 获取所有按钮
buttons = [e for e in perception.structure.elements 
           if e.tag_name == "button"]
```

### Deep 模式 (<500ms)
```python
# 深度语义分析
perception = await page.perceive(PerceptionMode.Deep)
# AI 理解页面意图
user_intent = perception.semantic.user_intent
next_action = perception.semantic.suggested_action
```

## 🛠️ 使用标准工具

### 导航和等待

```python
# 智能导航，等待页面完全加载
await page.navigate_to_url(
    "https://example.com",
    wait_until="networkidle",
    timeout=30000
)

# 等待特定元素出现
element = await page.wait_for_element(
    "#dynamic-content",
    visible=True,
    timeout=10000
)
```

### 交互操作

```python
# 精准点击
await page.click("#menu-button", delay=500)

# 智能输入（自动清空并输入）
await page.type_text(
    "#search-box",
    "AI Browser",
    clear_first=True,
    press_enter=True
)

# 滚动到元素
await page.scroll_page(
    ScrollDirection.ToElement("#footer")
)
```

## 📊 并发操作

```python
async def concurrent_browsing():
    async with Browser() as browser:
        # 创建多个页面
        pages = []
        for i in range(5):
            page = await browser.new_page()
            pages.append(page)
        
        # 并发执行任务
        tasks = []
        urls = ["https://example1.com", "https://example2.com", ...]
        
        for page, url in zip(pages, urls):
            task = page.navigate(url)
            tasks.append(task)
        
        # 等待所有任务完成
        await asyncio.gather(*tasks)
        
        # 并发获取所有标题
        titles = await asyncio.gather(*[
            page.get_title() for page in pages
        ])
        
        print("获取的标题:", titles)
```

## 🔍 错误处理

```python
from rainbow_browser import BrowserError, ElementNotFound

async def robust_interaction():
    try:
        await page.click("#may-not-exist")
    except ElementNotFound as e:
        print(f"元素未找到: {e}")
        # 使用备选方案
        await page.click(".alternative-selector")
    except BrowserError as e:
        print(f"浏览器错误: {e}")
        # 实施重试逻辑
```

## 🚄 性能优化技巧

### 1. 使用缓存

```python
from rainbow_browser import Browser, CacheStrategy

browser = Browser(cache_strategy=CacheStrategy.Aggressive)
# 重复访问相同页面时将使用缓存
```

### 2. 批量操作

```python
# 批量执行多个操作，减少往返通信
results = await page.execute_batch([
    {"action": "click", "selector": "#btn1"},
    {"action": "type_text", "selector": "#input1", "text": "Hello"},
    {"action": "click", "selector": "#submit"}
])
```

### 3. 资源管理

```python
# 及时释放不需要的页面
page1 = await browser.new_page()
# ... 使用 page1
await page1.close()  # 释放资源

# 或使用上下文管理器
async with browser.new_page() as page:
    # 页面会自动关闭
    pass
```

## 📱 常见使用场景

### 场景 1：电商价格监控
```python
async def monitor_price(product_url):
    async with Browser() as browser:
        page = await browser.new_page()
        await page.navigate(product_url)
        
        # AI 自动识别价格
        perception = await page.perceive(PerceptionMode.Standard)
        price_element = perception.semantic.price_info
        
        if price_element:
            print(f"当前价格: {price_element.value}")
            # 保存到数据库或发送通知
```

### 场景 2：自动化测试
```python
async def test_login_flow():
    async with Browser() as browser:
        page = await browser.new_page()
        await page.navigate("https://app.example.com/login")
        
        # 执行登录流程
        await page.type_text("#username", "testuser")
        await page.type_text("#password", "testpass")
        await page.click("#login-button")
        
        # 验证登录成功
        await page.wait_for_element(".dashboard", timeout=5000)
        assert await page.get_url() == "https://app.example.com/dashboard"
```

### 场景 3：内容聚合
```python
async def aggregate_news():
    sources = [
        "https://news1.com",
        "https://news2.com",
        "https://news3.com"
    ]
    
    async with Browser() as browser:
        articles = []
        
        for source in sources:
            page = await browser.new_page()
            await page.navigate(source)
            
            # Deep 模式提取文章
            perception = await page.perceive(PerceptionMode.Deep)
            if perception.semantic.articles:
                articles.extend(perception.semantic.articles)
            
            await page.close()
        
        return articles
```

## 🎓 下一步

- 📖 阅读[完整 API 文档](API_REFERENCE.md)了解所有功能
- 🏗️ 查看[架构设计](ARCHITECTURE.md)理解内部原理
- 💡 探索[开发者指南](DEVELOPER_GUIDE.md)获取最佳实践
- 🚀 查看[部署指南](DEPLOYMENT.md)了解生产环境配置

## 💬 获取帮助

- 📧 邮箱：support@rainbow-browser.ai
- 💬 Discord：[加入我们的社区](https://discord.gg/rainbow-browser)
- 📚 文档：[https://docs.rainbow-browser.ai](https://docs.rainbow-browser.ai)
- 🐛 问题反馈：[GitHub Issues](https://github.com/rainbow-browser/issues)

---

**Happy Browsing with AI! 🌈**