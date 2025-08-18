# 🚀 彩虹城浏览器 8.0 - 设置指南

## 📋 前置要求

### 1. Rust开发环境
```bash
# 安装Rust（如果未安装）
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 验证安装
rustc --version
cargo --version
```

### 2. Chrome浏览器和ChromeDriver
```bash
# Windows
# 1. 下载ChromeDriver: https://chromedriver.chromium.org/
# 2. 将chromedriver.exe放到PATH中

# macOS
brew install chromedriver

# Linux
sudo apt-get update
sudo apt-get install chromium-chromedriver
```

### 3. 启动ChromeDriver服务
```bash
# 启动ChromeDriver（默认端口9515）
chromedriver --port=9515
```

## 🤖 LLM配置

### 选项1: 使用OpenAI API（推荐）
```bash
# 设置环境变量
export OPENAI_API_KEY="your-api-key-here"
export OPENAI_MODEL="gpt-3.5-turbo"
```

### 选项2: 使用Ollama本地模型
```bash
# 安装Ollama
curl -fsSL https://ollama.ai/install.sh | sh

# 下载模型
ollama pull llama2

# 启动Ollama服务
ollama serve
```

### 选项3: 使用Claude API
```bash
export ANTHROPIC_API_KEY="your-api-key-here"
export CLAUDE_MODEL="claude-3-sonnet-20240229"
```

## 🗄️ SurrealDB设置（可选）

```bash
# 安装SurrealDB
curl -sSf https://install.surrealdb.com | sh

# 启动SurrealDB
surreal start --log debug --user root --pass root memory

# 或使用Docker
docker run --rm -p 8000:8000 surrealdb/surrealdb:latest start --user root --pass root memory
```

## 🏗️ 构建和运行

### 1. 克隆项目
```bash
git clone https://github.com/rainbow-city/browser-ai.git
cd browser-ai
```

### 2. 复制环境配置
```bash
cp .env.example .env
# 编辑.env文件，填写您的配置
```

### 3. 构建项目
```bash
# 完整构建（包含所有功能）
cargo build --release --all-features

# 或仅构建核心功能
cargo build --release
```

### 4. 运行示例

#### 交互式模式
```bash
cargo run --release
```

#### 运行特定任务
```bash
cargo run --release -- run "搜索Rust编程最新特性"
```

#### 查看系统状态
```bash
cargo run --release -- status
```

#### 运行真实世界演示
```bash
cargo run --release --example real_world_demo
```

## 📊 功能验证

### 1. 测试WebDriver连接
```bash
# 确保ChromeDriver正在运行
curl http://localhost:9515/status
```

### 2. 测试LLM连接
```bash
# OpenAI
curl https://api.openai.com/v1/models \
  -H "Authorization: Bearer $OPENAI_API_KEY"

# Ollama
curl http://localhost:11434/api/tags
```

### 3. 运行集成测试
```bash
cargo test --all-features
```

## 🐛 常见问题

### Q: WebDriver连接失败
A: 
1. 确保ChromeDriver正在运行：`chromedriver --port=9515`
2. 检查Chrome版本与ChromeDriver版本是否匹配
3. 检查防火墙设置

### Q: LLM响应超时
A: 
1. 检查API密钥是否正确
2. 检查网络连接
3. 对于Ollama，确保模型已下载：`ollama list`

### Q: 内存不足
A: 
1. 减少并发会话数：修改`MAX_CONCURRENT_SESSIONS`
2. 启用性能优化：运行`cargo run -- optimize`
3. 增加系统内存

## 📚 使用示例

### 基础搜索任务
```rust
let browser = RainbowBrowserV8::new().await?;
let response = browser.process_request("搜索Rust最佳实践").await?;
println!("{}", response);
```

### 使用不同感知模式
```rust
let config = SessionConfig::new("https://example.com")
    .with_perception_mode(PerceptionMode::Lightning); // 极速模式
```

### 存储和查询记忆
```rust
let memory = MemoryData {
    data_type: DataType::Knowledge,
    content: serde_json::json!({"learned": "something new"}),
    // ...
};
browser.optimized_persistence.store(memory).await?;
```

## 🔗 相关资源

- [设计文档](docs/design/)
- [API文档](https://docs.rs/rainbow-browser-ai)
- [示例代码](examples/)
- [GitHub Issues](https://github.com/rainbow-city/browser-ai/issues)

## 📧 支持

如需帮助，请：
1. 查看[故障排除指南](TROUBLESHOOTING.md)
2. 提交[GitHub Issue](https://github.com/rainbow-city/browser-ai/issues)
3. 加入[Discord社区](https://discord.gg/rainbow-city)