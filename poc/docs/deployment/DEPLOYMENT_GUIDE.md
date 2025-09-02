# 🚀 RainbowBrowserAI Production Deployment Guide

## Table of Contents
1. [Prerequisites](#prerequisites)
2. [Build Optimization](#build-optimization)
3. [Docker Deployment](#docker-deployment)
4. [Systemd Service](#systemd-service)
5. [Reverse Proxy Setup](#reverse-proxy-setup)
6. [Environment Configuration](#environment-configuration)
7. [Monitoring & Logging](#monitoring--logging)
8. [Security Considerations](#security-considerations)
9. [Performance Tuning](#performance-tuning)
10. [Troubleshooting](#troubleshooting)

---

## Prerequisites

### System Requirements
- **OS**: Ubuntu 20.04+ / Debian 11+ / RHEL 8+
- **RAM**: Minimum 4GB, Recommended 8GB
- **CPU**: 2+ cores recommended
- **Disk**: 10GB free space
- **Network**: Ports 3001 (API), 9515 (ChromeDriver)

### Software Dependencies
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install Chrome/Chromium
sudo apt update
sudo apt install -y chromium-browser chromium-chromedriver

# Install build tools
sudo apt install -y build-essential pkg-config libssl-dev
```

---

## Build Optimization

### 1. Production Build
```bash
# Clean previous builds
cargo clean

# Build with maximum optimization
RUSTFLAGS="-C target-cpu=native -C opt-level=3" \
cargo build --release

# Strip debug symbols (reduces size by ~50%)
strip target/release/rainbow-poc

# Check binary size
du -h target/release/rainbow-poc
```

### 2. Cross-compilation (Optional)
```bash
# For Linux musl (static binary)
rustup target add x86_64-unknown-linux-musl
cargo build --release --target x86_64-unknown-linux-musl

# For ARM64
rustup target add aarch64-unknown-linux-gnu
cargo build --release --target aarch64-unknown-linux-gnu
```

---

## Docker Deployment

### Dockerfile
```dockerfile
# Build stage
FROM rust:1.75 as builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Build with optimizations
RUN RUSTFLAGS="-C target-cpu=native" cargo build --release
RUN strip /app/target/release/rainbow-poc

# Runtime stage
FROM ubuntu:22.04

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    chromium-browser \
    chromium-chromedriver \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy binary
COPY --from=builder /app/target/release/rainbow-poc /usr/local/bin/rainbow-poc

# Create non-root user
RUN useradd -m -s /bin/bash rainbow
USER rainbow

# Expose ports
EXPOSE 3001

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3001/health || exit 1

# Start service
CMD ["rainbow-poc", "serve", "--port", "3001"]
```

### Docker Compose
```yaml
version: '3.8'

services:
  rainbow-poc:
    build: .
    container_name: rainbow-browser-ai
    ports:
      - "3001:3001"
    environment:
      - RAINBOW_MOCK_MODE=false
      - CHROMEDRIVER_PORT=9515
      - RUST_LOG=info
    volumes:
      - ./config:/app/config
      - ./logs:/app/logs
    restart: unless-stopped
    networks:
      - rainbow-network

  # Optional: Redis for caching
  redis:
    image: redis:alpine
    container_name: rainbow-redis
    ports:
      - "6379:6379"
    networks:
      - rainbow-network

networks:
  rainbow-network:
    driver: bridge
```

---

## Systemd Service

### 1. Create Service File
```bash
sudo nano /etc/systemd/system/rainbow-poc.service
```

```ini
[Unit]
Description=RainbowBrowserAI Service
After=network.target
StartLimitIntervalSec=0

[Service]
Type=simple
Restart=always
RestartSec=10
User=rainbow
Group=rainbow
WorkingDirectory=/opt/rainbow-poc
Environment="RAINBOW_MOCK_MODE=false"
Environment="CHROMEDRIVER_PORT=9515"
Environment="RUST_LOG=info"
ExecStartPre=/usr/bin/chromedriver --port=9515 &
ExecStart=/opt/rainbow-poc/rainbow-poc serve --port 3001
StandardOutput=append:/var/log/rainbow-poc/stdout.log
StandardError=append:/var/log/rainbow-poc/stderr.log

[Install]
WantedBy=multi-user.target
```

### 2. Enable and Start Service
```bash
# Create user and directories
sudo useradd -m -s /bin/bash rainbow
sudo mkdir -p /opt/rainbow-poc /var/log/rainbow-poc
sudo chown -R rainbow:rainbow /opt/rainbow-poc /var/log/rainbow-poc

# Copy binary
sudo cp target/release/rainbow-poc /opt/rainbow-poc/
sudo chmod +x /opt/rainbow-poc/rainbow-poc

# Enable service
sudo systemctl daemon-reload
sudo systemctl enable rainbow-poc
sudo systemctl start rainbow-poc

# Check status
sudo systemctl status rainbow-poc
sudo journalctl -u rainbow-poc -f
```

---

## Reverse Proxy Setup

### Nginx Configuration
```nginx
upstream rainbow_backend {
    server 127.0.0.1:3001;
    keepalive 64;
}

server {
    listen 80;
    server_name api.rainbow-ai.example.com;
    
    # Redirect to HTTPS
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl http2;
    server_name api.rainbow-ai.example.com;
    
    # SSL Configuration
    ssl_certificate /etc/ssl/certs/rainbow-ai.crt;
    ssl_certificate_key /etc/ssl/private/rainbow-ai.key;
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers HIGH:!aNULL:!MD5;
    
    # Security headers
    add_header X-Frame-Options "SAMEORIGIN" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-XSS-Protection "1; mode=block" always;
    
    # Rate limiting
    limit_req_zone $binary_remote_addr zone=api:10m rate=10r/s;
    limit_req zone=api burst=20 nodelay;
    
    # API endpoints
    location /api/ {
        proxy_pass http://rainbow_backend;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_cache_bypass $http_upgrade;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        
        # Timeouts
        proxy_connect_timeout 60s;
        proxy_send_timeout 60s;
        proxy_read_timeout 60s;
    }
    
    # Health check endpoint
    location /health {
        proxy_pass http://rainbow_backend/health;
        access_log off;
    }
    
    # Static files
    location /static/ {
        alias /opt/rainbow-poc/static/;
        expires 30d;
        add_header Cache-Control "public, immutable";
    }
}
```

---

## Environment Configuration

### Production .env File
```bash
# API Configuration
SERVER_PORT=3001
SERVER_HOST=0.0.0.0
API_KEY=your-secure-api-key-here

# Browser Configuration
CHROMEDRIVER_PORT=9515
BROWSER_HEADLESS=true
BROWSER_TIMEOUT=30000
MAX_BROWSER_SESSIONS=10

# Performance
CACHE_TTL_SECONDS=300
MAX_CACHE_SIZE_MB=100
WORKER_THREADS=4

# Logging
RUST_LOG=info,rainbow_poc=debug
LOG_FORMAT=json
LOG_FILE=/var/log/rainbow-poc/app.log

# Security
ENABLE_CORS=true
CORS_ORIGINS=https://app.example.com
RATE_LIMIT_PER_MINUTE=60
JWT_SECRET=your-jwt-secret-here

# Database (optional)
DATABASE_URL=postgresql://user:pass@localhost/rainbow_poc
REDIS_URL=redis://localhost:6379

# Monitoring
METRICS_ENABLED=true
METRICS_PORT=9090
TRACING_ENABLED=true
JAEGER_ENDPOINT=http://localhost:14268/api/traces
```

---

## Monitoring & Logging

### 1. Prometheus Metrics
```yaml
# prometheus.yml
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'rainbow-poc'
    static_configs:
      - targets: ['localhost:9090']
```

### 2. Log Rotation
```bash
# /etc/logrotate.d/rainbow-poc
/var/log/rainbow-poc/*.log {
    daily
    rotate 14
    compress
    delaycompress
    missingok
    notifempty
    create 0644 rainbow rainbow
    sharedscripts
    postrotate
        systemctl reload rainbow-poc
    endscript
}
```

### 3. Health Monitoring Script
```bash
#!/bin/bash
# health_check.sh

API_URL="http://localhost:3001/health"
SLACK_WEBHOOK="https://hooks.slack.com/services/YOUR/WEBHOOK/URL"

response=$(curl -s -o /dev/null -w "%{http_code}" $API_URL)

if [ $response -ne 200 ]; then
    curl -X POST $SLACK_WEBHOOK \
        -H 'Content-Type: application/json' \
        -d '{"text":"⚠️ RainbowBrowserAI health check failed!"}'
    
    # Restart service
    sudo systemctl restart rainbow-poc
fi
```

---

## Security Considerations

### 1. Firewall Rules
```bash
# Allow only necessary ports
sudo ufw allow 22/tcp    # SSH
sudo ufw allow 443/tcp   # HTTPS
sudo ufw allow 3001/tcp  # API (only from reverse proxy)
sudo ufw enable
```

### 2. SELinux (RHEL/CentOS)
```bash
# Create policy for rainbow-poc
sudo semanage port -a -t http_port_t -p tcp 3001
sudo setsebool -P httpd_can_network_connect 1
```

### 3. AppArmor (Ubuntu/Debian)
```bash
# Create profile
sudo nano /etc/apparmor.d/rainbow-poc
```

---

## Performance Tuning

### 1. System Limits
```bash
# /etc/security/limits.conf
rainbow soft nofile 65536
rainbow hard nofile 65536
rainbow soft nproc 32768
rainbow hard nproc 32768
```

### 2. Kernel Parameters
```bash
# /etc/sysctl.conf
net.core.somaxconn = 65535
net.ipv4.tcp_fin_timeout = 30
net.ipv4.tcp_keepalive_time = 300
net.ipv4.tcp_max_syn_backlog = 8192
```

### 3. Application Tuning
```toml
# config/production.toml
[performance]
max_connections = 1000
connection_timeout = 30
keep_alive = true
compression = true

[cache]
enabled = true
max_size_mb = 500
ttl_seconds = 3600

[browser]
pool_size = 10
reuse_sessions = true
cleanup_interval = 300
```

---

## Troubleshooting

### Common Issues

#### 1. ChromeDriver Not Starting
```bash
# Check ChromeDriver
chromedriver --version
ps aux | grep chromedriver

# Test ChromeDriver
curl http://localhost:9515/status
```

#### 2. High Memory Usage
```bash
# Check memory
free -h
ps aux --sort=-%mem | head

# Restart service
sudo systemctl restart rainbow-poc
```

#### 3. Connection Refused
```bash
# Check if service is running
sudo systemctl status rainbow-poc
sudo netstat -tulpn | grep 3001

# Check logs
sudo journalctl -u rainbow-poc -n 100
```

### Debug Mode
```bash
# Run with debug logging
RUST_LOG=debug rainbow-poc serve --port 3001

# Enable backtrace
RUST_BACKTRACE=full rainbow-poc serve
```

---

## Backup & Recovery

### Backup Script
```bash
#!/bin/bash
# backup.sh

BACKUP_DIR="/backup/rainbow-poc"
DATE=$(date +%Y%m%d_%H%M%S)

# Create backup
tar -czf $BACKUP_DIR/rainbow-poc-$DATE.tar.gz \
    /opt/rainbow-poc \
    /etc/systemd/system/rainbow-poc.service \
    /etc/nginx/sites-available/rainbow-poc

# Keep only last 7 days
find $BACKUP_DIR -name "*.tar.gz" -mtime +7 -delete
```

---

## Scaling Considerations

### Horizontal Scaling
1. Use load balancer (HAProxy/Nginx)
2. Deploy multiple instances
3. Share session state (Redis)
4. Centralized logging (ELK stack)

### Vertical Scaling
1. Increase CPU/RAM
2. Tune connection pools
3. Optimize cache size
4. Adjust worker threads

---

*Last Updated: September 1, 2025*
*Version: 1.0.0*