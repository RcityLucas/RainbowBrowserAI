# RainbowBrowserAI Standalone Browser

本文档介绍如何构建和使用RainbowBrowserAI的独立可执行文件版本。

## 🌟 特性

- **独立运行**: 无需安装Rust环境或其他依赖
- **内置AI服务器**: 自动启动本地AI服务器
- **系统浏览器集成**: 自动打开默认浏览器
- **跨平台支持**: Windows、macOS、Linux
- **单文件分发**: 只需一个可执行文件

## 🛠️ 构建独立可执行文件

### 方法1: 使用构建脚本 (推荐)

#### Linux/macOS:
```bash
./build_standalone.sh
```

#### Windows:
```cmd
build_standalone.bat
```

### 方法2: 手动构建

```bash
cargo build --release --bin rainbow-browser-standalone --features standalone,web-server
```

构建完成后，可执行文件位于 `target/release/rainbow-browser-standalone`

### 构建选项

| 选项 | 描述 | 示例 |
|------|------|------|
| `--features` | 启用的功能特性 | `--features standalone,web-server` |
| `--target` | 目标平台 | `--target x86_64-pc-windows-gnu` |
| `--output` | 输出目录 | `--output dist` |

### 跨平台构建

```bash
# Windows (从Linux/macOS构建)
./build_standalone.sh --target x86_64-pc-windows-gnu

# macOS (从Linux构建)
./build_standalone.sh --target x86_64-apple-darwin

# Linux (从其他平台构建)
./build_standalone.sh --target x86_64-unknown-linux-gnu
```

## 🚀 使用方法

### 启动应用

直接运行可执行文件:

```bash
./rainbow-browser-standalone
```

### 启动流程

1. **初始化**: 应用启动并初始化AI引擎
2. **服务器启动**: 本地AI服务器在 `http://localhost:8888` 启动
3. **欢迎页面**: 欢迎服务器在 `http://localhost:8889` 启动
4. **浏览器打开**: 自动打开系统默认浏览器显示欢迎页面
5. **自动跳转**: 3秒后自动跳转到AI服务器界面

### 访问地址

- **欢迎页面**: http://localhost:8889
- **AI服务器**: http://localhost:8888
- **完整界面**: http://localhost:8888/app

## 🎯 功能说明

### AI服务器功能

- **自然语言控制**: 使用自然语言控制浏览器
- **智能任务执行**: 自动化网页操作
- **数据提取**: 智能提取网页数据
- **表单填写**: 自动填写表单
- **批量处理**: 批量执行任务

### 浏览器控制

- **导航**: 访问任意网页
- **点击**: 点击页面元素
- **输入**: 填写表单字段
- **滚动**: 页面滚动操作
- **截图**: 页面截图保存
- **脚本执行**: 执行JavaScript

## 🔧 配置选项

### 环境变量

| 变量名 | 描述 | 默认值 |
|--------|------|--------|
| `RAINBOW_PORT` | AI服务器端口 | 8888 |
| `RAINBOW_WELCOME_PORT` | 欢迎页面端口 | 8889 |
| `RAINBOW_LOG_LEVEL` | 日志级别 | info |

### 配置文件

创建 `config.json` 文件在可执行文件同目录:

```json
{
  "browser": {
    "title": "自定义标题",
    "width": 1400,
    "height": 900,
    "initial_url": "https://example.com"
  },
  "server": {
    "ai_port": 8888,
    "welcome_port": 8889
  }
}
```

## 📦 分发方式

### 单文件分发

独立可执行文件可以直接分发给最终用户，无需安装任何依赖。

### 系统要求

#### Windows
- Windows 10/11 (64位)
- Visual C++ Redistributable (通常已安装)

#### macOS
- macOS 10.15+ (Catalina)
- 首次运行需要在"安全性与隐私"中允许运行

#### Linux
- Ubuntu 18.04+ / CentOS 7+ / 其他现代发行版
- glibc 2.17+

### 防火墙设置

首次运行时，系统可能询问是否允许网络访问，请选择"允许"以启用AI功能。

## 🔍 故障排除

### 常见问题

#### 1. 端口被占用

**症状**: 启动失败，显示端口被占用错误

**解决方案**:
```bash
# 检查端口占用
netstat -an | grep 8888
netstat -an | grep 8889

# 终止占用进程
kill -9 <PID>

# 或使用不同端口
RAINBOW_PORT=8890 ./rainbow-browser-standalone
```

#### 2. 浏览器未自动打开

**症状**: 应用启动但浏览器没有打开

**解决方案**:
- 手动打开浏览器访问 http://localhost:8889
- 检查系统是否设置了默认浏览器
- 在某些Linux发行版中，需要安装 `xdg-utils`

#### 3. 权限问题

**症状**: Linux/macOS下提示权限不足

**解决方案**:
```bash
chmod +x rainbow-browser-standalone
```

#### 4. Windows安全警告

**症状**: Windows Defender拦截执行

**解决方案**:
- 在Windows Defender中添加排除项
- 或选择"仍要运行"

### 日志调试

启用详细日志:

```bash
# 设置日志级别为debug
RAINBOW_LOG_LEVEL=debug ./rainbow-browser-standalone

# 保存日志到文件
./rainbow-browser-standalone 2>&1 | tee rainbow.log
```

### 性能优化

#### 内存使用

- 典型内存占用: 50-100MB
- 大型网页操作时可能达到 200-500MB

#### CPU使用

- 空闲时: <5%
- AI处理时: 20-50%
- 大量操作时: 可能短暂达到 80-100%

## 🎨 自定义开发

### 扩展功能

如需添加自定义功能，可以修改 `src/standalone_browser.rs`:

```rust
// 添加新的浏览器命令
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BrowserCommand {
    // 现有命令...
    CustomAction(String),
}

// 处理自定义命令
match cmd {
    BrowserCommand::CustomAction(action) => {
        // 自定义逻辑
    }
    // 其他命令...
}
```

### API扩展

添加新的API端点:

```rust
// 在 handle_ai_request 函数中添加
if request.contains("POST /custom") {
    let custom_response = handle_custom_request().await?;
    // 返回自定义响应
}
```

## 📞 支持

### 获取帮助

- GitHub Issues: https://github.com/rainbow-city/browser-ai/issues
- 文档: https://docs.rainbowcity.ai
- 社区: https://community.rainbowcity.ai

### 贡献代码

欢迎提交Pull Request来改进独立浏览器功能：

1. Fork本仓库
2. 创建功能分支
3. 提交更改
4. 创建Pull Request

### 许可证

本项目采用 MIT 许可证，详见 `LICENSE` 文件。

---

🌈 **RainbowBrowserAI** - 让AI控制浏览器变得简单！