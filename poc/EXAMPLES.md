# 🎯 RainbowBrowserAI 示例命令

## 基础命令

### 简单导航
```
navigate to github.com
go to google
open stackoverflow
visit youtube.com
```

### 带截图的导航
```
navigate to github.com and take a screenshot
go to google and capture the screen
open stackoverflow and snap a picture
```

## 中级命令

### 多网站测试
```
test google.com,github.com,stackoverflow.com
test these sites: google, github, stackoverflow
```

### 自定义视口截图
```
navigate to github.com with 1920x1080 viewport and take screenshot
go to google with 1280x720 resolution and capture
```

### 数据提取（通过API）
```bash
# POST to /extract endpoint
curl -X POST http://localhost:3000/extract \
  -H "Content-Type: application/json" \
  -d '{
    "url": "https://news.ycombinator.com",
    "format": "json",
    "selector": "a.storylink"
  }'
```

## 高级命令

### 批量网站分析
```
test google.com,github.com,stackoverflow.com,reddit.com with screenshots
```

**输出示例**：
```json
{
  "action": "test",
  "total_tests": 4,
  "successful_tests": 4,
  "success_rate": 1.0,
  "screenshots_enabled": true,
  "results": [
    {
      "url": "google.com",
      "index": 1,
      "success": true,
      "loading_time_ms": 1243,
      "title": "Google",
      "screenshot_path": "test_google_com_20250818_123456.png"
    },
    {
      "url": "github.com", 
      "index": 2,
      "success": true,
      "loading_time_ms": 856,
      "title": "GitHub",
      "screenshot_path": "test_github_com_20250818_123458.png"
    }
  ]
}
```

### 自定义参数导航
```
navigate to github.com with 1920x1080 viewport and take full page screenshot
```

### 报告生成
```
show cost report
display usage statistics
generate report
```

## API 示例

### 复杂工作流
```bash
# 1. 导航并截图
curl -X POST http://localhost:3000/command \
  -H "Content-Type: application/json" \
  -d '{"command":"navigate to github.com and take screenshot"}'

# 2. 提取数据
curl -X POST http://localhost:3000/extract \
  -H "Content-Type: application/json" \
  -d '{
    "url": "https://github.com/trending",
    "format": "json",
    "selector": "h1.h3"
  }'

# 3. 检查状态
curl http://localhost:3000/health

# 4. 查看成本
curl http://localhost:3000/cost
```

### 批量处理
```bash
# 测试多个网站
curl -X POST http://localhost:3000/command \
  -H "Content-Type: application/json" \
  -d '{"command":"test google.com,github.com,stackoverflow.com with screenshots"}'
```

## 命令行示例

### 直接命令
```bash
# 导航
cargo run -- navigate https://github.com --screenshot

# 数据提取
cargo run -- extract https://news.ycombinator.com --format json --selector "a.storylink"

# 自然语言（Mock模式）
cargo run -- ask "go to github and take a screenshot"
```

### 工作流文件
创建 `workflow.yaml`:
```yaml
name: "Website Analysis"
steps:
  - action: navigate
    url: "https://github.com/trending"
    screenshot: true
  - action: extract
    selector: "h1.h3"
    format: json
  - action: navigate  
    url: "https://stackoverflow.com/questions"
    screenshot: true
```

然后运行:
```bash
cargo run -- workflow workflow.yaml
```

## Mock模式支持的复杂命令

### 网站测试
```
test these websites: google, github, stackoverflow, reddit
test google.com,github.com,stackoverflow.com with screenshots
```

### 参数化导航
```
navigate to github.com with 1920x1080 and take screenshot
go to google with 1280x720 viewport and capture screen
```

### 组合命令
```
open stackoverflow with full page screenshot
visit youtube.com and take a 1600x900 screenshot
```

## 实际使用场景

### 网站监控
```bash
# 监控多个网站状态
curl -X POST http://localhost:3000/command \
  -H "Content-Type: application/json" \
  -d '{"command":"test mysite.com,api.mysite.com,cdn.mysite.com"}'
```

### 竞品分析
```bash
# 截图对比
curl -X POST http://localhost:3000/command \
  -H "Content-Type: application/json" \
  -d '{"command":"navigate to competitor.com with 1920x1080 and take screenshot"}'
```

### 数据采集
```bash
# 提取新闻标题
curl -X POST http://localhost:3000/extract \
  -H "Content-Type: application/json" \
  -d '{
    "url": "https://news.ycombinator.com",
    "format": "csv",
    "selector": "a.storylink"
  }'
```

## 提示

1. **Mock模式**支持大部分基础命令，无需API密钥
2. **API模式**支持更复杂的自然语言理解
3. **组合使用**Web界面和API获得最佳体验
4. **批量操作**可以显著提高效率

## 错误处理

如果命令无法识别，系统会：
1. 尝试最佳匹配
2. 提供建议
3. 记录到日志
4. 返回错误信息

---

🎉 **开始实验这些命令，探索更多可能性！**