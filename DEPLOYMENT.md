# RainbowBrowserAI 部署和分发指南

本文档详细介绍如何部署和分发RainbowBrowserAI项目的各种形式。

## 📦 部署方式概览

| 部署方式 | 适用场景 | 技术要求 | 维护复杂度 |
|----------|----------|----------|------------|
| 独立可执行文件 | 终端用户 | 无 | 低 |
| 源码编译 | 开发者 | Rust环境 | 中 |
| 浏览器扩展 | 日常用户 | 浏览器 | 低 |
| Web服务 | 企业部署 | 服务器 | 高 |

## 🚀 独立可执行文件部署

### 构建发布版本

```bash
# 使用构建脚本 (推荐)
./build_standalone.sh

# 手动构建
cargo build --release --bin rainbow-browser-standalone --features standalone,web-server

# 跨平台构建
./build_standalone.sh --target x86_64-pc-windows-gnu    # Windows
./build_standalone.sh --target x86_64-apple-darwin      # macOS  
./build_standalone.sh --target x86_64-unknown-linux-gnu # Linux
```

### 分发包结构

```
rainbow-browser-ai/
├── rainbow-browser-standalone(.exe)     # 主执行文件
├── README.md                            # 使用说明
├── STANDALONE.md                        # 详细文档
├── config.json.example                  # 配置示例
└── browser_extension/                   # 可选浏览器扩展
    ├── manifest.json
    ├── popup.html
    └── content.js
```

### 系统要求

#### Windows
- **系统**: Windows 10/11 (64位)
- **依赖**: Visual C++ Redistributable (通常已安装)
- **权限**: 标准用户权限即可
- **防火墙**: 允许网络访问 (首次运行时询问)

#### macOS
- **系统**: macOS 10.15+ (Catalina或更新版本)
- **安全**: 首次运行需要在"系统偏好设置 > 安全性与隐私"中允许
- **权限**: 标准用户权限即可

#### Linux
- **系统**: Ubuntu 18.04+ / CentOS 7+ / 现代发行版
- **依赖**: glibc 2.17+, xdg-utils (用于打开浏览器)
- **权限**: 标准用户权限即可

### 分发策略

#### 1. GitHub Releases
```bash
# 创建发布标签
git tag -a v8.0.0 -m "RainbowBrowserAI v8.0.0"
git push origin v8.0.0

# 上传构建的二进制文件到GitHub Releases
# - rainbow-browser-standalone-windows-x64.exe
# - rainbow-browser-standalone-macos-x64
# - rainbow-browser-standalone-linux-x64
```

#### 2. 直接下载链接
```bash
# 生成下载脚本
echo '#!/bin/bash
LATEST_RELEASE=$(curl -s https://api.github.com/repos/RcityLucas/RainbowBrowserAI/releases/latest | grep "tag_name" | cut -d '"' -f 4)
curl -L -o rainbow-browser-standalone https://github.com/RcityLucas/RainbowBrowserAI/releases/download/$LATEST_RELEASE/rainbow-browser-standalone-$(uname -s)-$(uname -m)
chmod +x rainbow-browser-standalone
' > download.sh
```

#### 3. 安装脚本
```bash
# install.sh
#!/bin/bash
set -e

echo "🌈 安装RainbowBrowserAI..."

# 检测系统
OS=$(uname -s)
ARCH=$(uname -m)

case $OS in
    Linux)  PLATFORM="linux" ;;
    Darwin) PLATFORM="macos" ;;
    *)      echo "❌ 不支持的操作系统: $OS"; exit 1 ;;
esac

case $ARCH in
    x86_64) ARCH="x64" ;;
    *)      echo "❌ 不支持的架构: $ARCH"; exit 1 ;;
esac

# 下载最新版本
DOWNLOAD_URL="https://github.com/RcityLucas/RainbowBrowserAI/releases/latest/download/rainbow-browser-standalone-$PLATFORM-$ARCH"

echo "📥 下载中..."
curl -L -o /usr/local/bin/rainbow-browser-standalone "$DOWNLOAD_URL"
chmod +x /usr/local/bin/rainbow-browser-standalone

echo "✅ 安装完成！"
echo "🚀 运行: rainbow-browser-standalone"
```

## 🔧 浏览器扩展部署

### 开发版本安装

#### Chrome/Edge
1. 打开扩展管理页面: `chrome://extensions/`
2. 开启"开发者模式"
3. 点击"加载已解压的扩展程序"
4. 选择 `src/browser_extension` 文件夹

#### Firefox
1. 打开调试页面: `about:debugging`
2. 点击"此Firefox"
3. 点击"临时载入附加组件"
4. 选择 `src/browser_extension/manifest.json`

### 生产版本发布

#### Chrome Web Store
```bash
# 打包扩展
cd src/browser_extension
zip -r rainbow-browser-extension.zip * -x "*.git*" "node_modules/*" "*.DS_Store"

# 上传到Chrome Web Store开发者控制台
# https://chrome.google.com/webstore/devconsole/
```

#### Firefox Add-ons
```bash
# 生成Firefox兼容的manifest
# 修改manifest.json中的API调用为Firefox兼容版本
# 提交到 https://addons.mozilla.org/developers/
```

#### Edge Add-ons
```bash
# Edge使用Chrome兼容格式
# 提交到 https://partner.microsoft.com/dashboard/microsoftedge/
```

## 🌐 Web服务部署

### Docker部署

#### Dockerfile
```dockerfile
FROM rust:1.75 as builder

WORKDIR /app
COPY . .
RUN cargo build --release --features web-server

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/rainbow-browser /usr/local/bin/
EXPOSE 8888
CMD ["rainbow-browser", "server"]
```

#### Docker Compose
```yaml
version: '3.8'
services:
  rainbow-browser-ai:
    build: .
    ports:
      - "8888:8888"
    environment:
      - RUST_LOG=info
      - RAINBOW_PORT=8888
    volumes:
      - ./data:/app/data
    restart: unless-stopped
```

### 传统服务器部署

#### 系统服务配置 (systemd)
```ini
[Unit]
Description=RainbowBrowserAI Service
After=network.target

[Service]
Type=simple
User=rainbow
WorkingDirectory=/opt/rainbow-browser-ai
ExecStart=/opt/rainbow-browser-ai/rainbow-browser server
Restart=always
RestartSec=10

Environment=RUST_LOG=info
Environment=RAINBOW_PORT=8888

[Install]
WantedBy=multi-user.target
```

#### Nginx反向代理
```nginx
server {
    listen 80;
    server_name rainbow-browser.example.com;
    
    location / {
        proxy_pass http://localhost:8888;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_cache_bypass $http_upgrade;
    }
}
```

## 📊 监控和维护

### 健康检查

```bash
# 添加健康检查端点
curl -f http://localhost:8888/health || exit 1
```

### 日志配置

```bash
# 环境变量配置
export RUST_LOG=rainbow_browser_ai=info,tower_http=debug
export RUST_BACKTRACE=1

# 日志轮转 (logrotate)
/var/log/rainbow-browser-ai/*.log {
    daily
    rotate 30
    compress
    delaycompress
    missingok
    create 644 rainbow rainbow
    postrotate
        systemctl reload rainbow-browser-ai
    endscript
}
```

### 性能监控

```bash
# Prometheus metrics endpoint
curl http://localhost:8888/metrics

# 自定义监控脚本
#!/bin/bash
while true; do
    CPU=$(ps -p $(pgrep rainbow-browser) -o %cpu= | tail -1)
    MEM=$(ps -p $(pgrep rainbow-browser) -o %mem= | tail -1) 
    echo "$(date): CPU: ${CPU}%, MEM: ${MEM}%"
    sleep 60
done
```

## 🔒 安全配置

### 生产环境安全

```bash
# 限制文件权限
chmod 755 /opt/rainbow-browser-ai/rainbow-browser
chown rainbow:rainbow /opt/rainbow-browser-ai/

# 防火墙配置
ufw allow 8888/tcp
ufw enable

# SSL/TLS配置 (使用Nginx + Let's Encrypt)
certbot --nginx -d rainbow-browser.example.com
```

### 配置安全

```json
{
  "security": {
    "cors_origins": ["https://trusted-domain.com"],
    "rate_limit": {
      "requests_per_minute": 60,
      "burst_size": 10
    },
    "auth": {
      "enabled": true,
      "secret_key": "your-secret-key-here"
    }
  }
}
```

## 📈 扩展部署

### 负载均衡部署

```yaml
# docker-compose.yml (多实例)
version: '3.8'
services:
  rainbow-browser-ai-1:
    build: .
    ports:
      - "8888:8888"
  
  rainbow-browser-ai-2:
    build: .
    ports:
      - "8889:8888"
      
  nginx:
    image: nginx:alpine
    ports:
      - "80:80"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf
```

### 高可用配置

```bash
# 使用HAProxy进行负载均衡
global
    daemon

defaults
    mode http
    timeout connect 5000ms
    timeout client 50000ms
    timeout server 50000ms

frontend rainbow_frontend
    bind *:80
    default_backend rainbow_backend

backend rainbow_backend
    balance roundrobin
    server web1 127.0.0.1:8888 check
    server web2 127.0.0.1:8889 check
```

## 🚀 CI/CD 自动化

### GitHub Actions工作流

```yaml
name: Build and Deploy
on:
  push:
    tags: ['v*']

jobs:
  build:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
    
    steps:
    - uses: actions/checkout@v3
    - uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
    
    - name: Build standalone
      run: |
        if [ "$RUNNER_OS" = "Windows" ]; then
          ./build_standalone.bat
        else
          ./build_standalone.sh
        fi
    
    - name: Upload artifacts
      uses: actions/upload-artifact@v3
      with:
        name: rainbow-browser-${{ matrix.os }}
        path: target/standalone/rainbow-browser-standalone*
```

### 自动更新机制

```rust
// 添加到standalone_browser.rs
pub async fn check_for_updates() -> Result<Option<String>> {
    let client = reqwest::Client::new();
    let response = client
        .get("https://api.github.com/repos/RcityLucas/RainbowBrowserAI/releases/latest")
        .send()
        .await?;
    
    let release: serde_json::Value = response.json().await?;
    let latest_version = release["tag_name"].as_str().unwrap_or("");
    
    if latest_version != env!("CARGO_PKG_VERSION") {
        Ok(Some(latest_version.to_string()))
    } else {
        Ok(None)
    }
}
```

## 📋 部署检查清单

### 发布前检查
- [ ] 所有测试通过
- [ ] 文档更新完成
- [ ] 版本号已更新
- [ ] 构建脚本测试通过
- [ ] 跨平台兼容性验证
- [ ] 安全扫描完成

### 部署后验证
- [ ] 应用正常启动
- [ ] 健康检查端点响应
- [ ] 日志正常记录
- [ ] 监控数据正常
- [ ] 用户功能测试通过
- [ ] 性能基准测试通过

---

🌈 **RainbowBrowserAI部署指南** - 让AI浏览器控制无处不在！