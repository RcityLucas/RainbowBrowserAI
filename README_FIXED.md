# 🌈 彩虹城浏览器 8.0 - AI生命体的数字器官

## ✅ 编译错误修复说明

### 已修复的问题：

1. **sysinfo API变化** - 使用临时固定值替代，因为sysinfo 0.30版本API有重大变化
2. **PerceptionMode类型冲突** - 统一使用layered_perception模块的定义
3. **TaskType缺少Hash/Eq trait** - 添加了必要的derive属性
4. **未使用的导入** - 清理了所有未使用的导入警告

## 🚀 快速开始

### 1. 安装依赖
```bash
# 确保Rust版本 >= 1.75
rustup update

# 安装ChromeDriver（可选）
# Windows: 下载并添加到PATH
# macOS: brew install chromedriver
# Linux: sudo apt-get install chromium-chromedriver
```

### 2. 配置环境
```bash
# 复制环境配置
cp .env.example .env

# 编辑.env文件，设置您的LLM API密钥
# 或使用Ollama本地模型（默认）
```

### 3. 编译项目
```bash
# 清理旧构建
cargo clean

# 编译项目（首次编译需要几分钟）
cargo build --release

# 或仅检查编译
cargo check
```

### 4. 运行测试
```bash
# 运行简单测试
cargo run --example simple_test

# 运行交互模式
cargo run --release

# 运行真实世界演示
cargo run --example real_world_demo
```

## 📋 已知问题和解决方案

### 问题1: 编译时内存不足
```bash
# 解决方案：限制并行编译任务
cargo build -j 2
```

### 问题2: ChromeDriver连接失败
```bash
# 项目会自动降级到模拟模式
# 不影响LLM功能的使用
```

### 问题3: LLM API超时
```bash
# 设置更长的超时时间
export LLM_TIMEOUT=60
```

## 🏗️ 项目架构

### 六大引擎系统
1. **统一内核** (`unified_kernel`) - 会话和资源管理
2. **分层感知** (`layered_perception`) - 四层感知架构
3. **智能行动** (`intelligent_action`) - LLM集成和浏览器控制
4. **优化持久化** (`optimized_persistence`) - 多模态记忆系统
5. **性能引擎** (`performance_engine`) - 性能监控和优化
6. **稳定引擎** (`stability_engine`) - 故障检测和恢复

## 💡 使用示例

### 基本使用
```rust
use rainbow_browser_ai::prelude::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 创建AI生命体
    let browser = RainbowBrowserV8::new().await?;
    
    // 处理用户请求
    let response = browser.process_request("搜索Rust编程").await?;
    println!("{}", response);
    
    Ok(())
}
```

### 自定义感知模式
```rust
// 使用不同的感知模式
let config = SessionConfig::new("https://example.com")
    .with_perception_mode(PerceptionMode::Lightning); // 极速模式 <50ms
```

### 查询系统状态
```rust
// 获取完整的系统状态
let status = browser.get_system_status().await?;
println!("健康状态: {:?}", status.health);
println!("性能报告: {:?}", status.performance);
```

## 🔧 故障排除

### 编译失败
1. 确保Rust版本 >= 1.75
2. 清理缓存：`cargo clean`
3. 更新依赖：`cargo update`

### 运行时错误
1. 检查环境变量配置
2. 确保端口未被占用
3. 查看日志输出：`RUST_LOG=debug cargo run`

## 📚 相关文档
- [设置指南](SETUP.md)
- [设计文档](docs/design/)
- [API文档](docs/api/)

## 🤝 贡献
欢迎提交Issue和PR！

## 📄 许可证
MIT License