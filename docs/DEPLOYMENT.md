# RainbowBrowserAI Deployment Guide

## 📋 部署概述

彩虹城浏览器 V8.0 遵循"**Docker 优先，Kubernetes 可选**"的部署策略，提供从开发到生产的完整部署方案。

### 部署架构选择

| 场景 | 推荐方案 | 资源需求 | 复杂度 |
|------|---------|---------|--------|
| 开发测试 | Docker Compose | 4GB RAM | ⭐ |
| 小型生产 | Docker Swarm | 8GB RAM | ⭐⭐ |
| 中型生产 | Kubernetes | 16GB RAM | ⭐⭐⭐ |
| 大型生产 | K8s + Operator | 32GB+ RAM | ⭐⭐⭐⭐ |

## 🚀 快速部署

### Docker 一键部署

```bash
# 方式一：使用预构建镜像
docker run -d \
  --name rainbow-browser \
  -p 8080:8080 \
  -v rainbow-data:/data \
  -e SURREALDB_URL=embedded \
  -e REDIS_URL=embedded \
  --restart unless-stopped \
  rainbow/browser:v8.0

# 方式二：使用 docker-compose
curl -O https://raw.githubusercontent.com/rainbow-browser/deploy/docker-compose.yml
docker-compose up -d
```

## 🐳 Docker 部署详解

### 1. Dockerfile 配置

```dockerfile
# 多阶段构建优化镜像大小
FROM rust:1.75 as builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# 构建优化
RUN cargo build --release --features production

# 运行时镜像
FROM debian:bookworm-slim

# 安装运行时依赖
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# 创建非root用户
RUN useradd -m -u 1000 rainbow

WORKDIR /app

# 复制构建产物
COPY --from=builder /app/target/release/rainbow-browser /app/
COPY --from=builder /app/static /app/static

# 设置权限
RUN chown -R rainbow:rainbow /app

USER rainbow

# 健康检查
HEALTHCHECK --interval=30s --timeout=3s --retries=3 \
  CMD curl -f http://localhost:8080/health || exit 1

EXPOSE 8080

# 启动命令
CMD ["./rainbow-browser", "serve"]
```

### 2. Docker Compose 完整配置

```yaml
version: '3.8'

services:
  rainbow-browser:
    image: rainbow/browser:v8.0
    container_name: rainbow-browser
    restart: unless-stopped
    ports:
      - "8080:8080"
      - "9090:9090"  # Metrics端口
    environment:
      # 基础配置
      - RUST_LOG=info
      - RAINBOW_ENV=production
      
      # 数据库配置
      - SURREALDB_URL=surreal://surrealdb:8000
      - SURREALDB_USER=root
      - SURREALDB_PASS=${SURREALDB_PASS}
      
      # 缓存配置
      - REDIS_URL=redis://redis:6379
      - CACHE_TTL=3600
      
      # 性能配置
      - MAX_CONNECTIONS=1000
      - WORKER_THREADS=4
      
    volumes:
      - rainbow-data:/app/data
      - rainbow-logs:/app/logs
    depends_on:
      surrealdb:
        condition: service_healthy
      redis:
        condition: service_healthy
    networks:
      - rainbow-net
    deploy:
      resources:
        limits:
          cpus: '2'
          memory: 2G
        reservations:
          cpus: '1'
          memory: 1G

  surrealdb:
    image: surrealdb/surrealdb:latest
    container_name: rainbow-surrealdb
    restart: unless-stopped
    command: start --user root --pass ${SURREALDB_PASS} file:/data/db
    volumes:
      - surrealdb-data:/data/db
    ports:
      - "8000:8000"
    environment:
      - SURREAL_LOG=info
    networks:
      - rainbow-net
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8000/health"]
      interval: 30s
      timeout: 5s
      retries: 3

  redis:
    image: redis:7-alpine
    container_name: rainbow-redis
    restart: unless-stopped
    command: redis-server --appendonly yes --maxmemory 512mb --maxmemory-policy allkeys-lru
    volumes:
      - redis-data:/data
    ports:
      - "6379:6379"
    networks:
      - rainbow-net
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 30s
      timeout: 5s
      retries: 3

  # 可选：监控
  prometheus:
    image: prom/prometheus:latest
    container_name: rainbow-prometheus
    restart: unless-stopped
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml
      - prometheus-data:/prometheus
    ports:
      - "9091:9090"
    networks:
      - rainbow-net
    profiles:
      - monitoring

  grafana:
    image: grafana/grafana:latest
    container_name: rainbow-grafana
    restart: unless-stopped
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=${GRAFANA_PASSWORD}
    volumes:
      - grafana-data:/var/lib/grafana
      - ./grafana/dashboards:/etc/grafana/provisioning/dashboards
    ports:
      - "3000:3000"
    networks:
      - rainbow-net
    profiles:
      - monitoring

volumes:
  rainbow-data:
  rainbow-logs:
  surrealdb-data:
  redis-data:
  prometheus-data:
  grafana-data:

networks:
  rainbow-net:
    driver: bridge
```

### 3. 环境变量配置

创建 `.env` 文件：

```bash
# 数据库密码
SURREALDB_PASS=your_secure_password

# Grafana 密码
GRAFANA_PASSWORD=admin_password

# 环境标识
RAINBOW_ENV=production

# 日志级别
RUST_LOG=rainbow_browser=info,tower_http=debug

# 性能调优
TOKIO_WORKER_THREADS=4
RAYON_NUM_THREADS=4
```

## ☸️ Kubernetes 部署

### 1. 基础部署配置

```yaml
# namespace.yaml
apiVersion: v1
kind: Namespace
metadata:
  name: rainbow-browser
---
# configmap.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: rainbow-config
  namespace: rainbow-browser
data:
  RUST_LOG: "info"
  RAINBOW_ENV: "production"
  MAX_CONNECTIONS: "1000"
  WORKER_THREADS: "4"
  CACHE_TTL: "3600"
---
# secret.yaml
apiVersion: v1
kind: Secret
metadata:
  name: rainbow-secrets
  namespace: rainbow-browser
type: Opaque
stringData:
  surrealdb-password: "your_secure_password"
  redis-password: "redis_password"
---
# deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: rainbow-browser
  namespace: rainbow-browser
  labels:
    app: rainbow-browser
spec:
  replicas: 3
  selector:
    matchLabels:
      app: rainbow-browser
  template:
    metadata:
      labels:
        app: rainbow-browser
    spec:
      containers:
      - name: rainbow-browser
        image: rainbow/browser:v8.0
        ports:
        - containerPort: 8080
          name: http
        - containerPort: 9090
          name: metrics
        env:
        - name: SURREALDB_URL
          value: "surreal://surrealdb-service:8000"
        - name: SURREALDB_PASS
          valueFrom:
            secretKeyRef:
              name: rainbow-secrets
              key: surrealdb-password
        - name: REDIS_URL
          value: "redis://redis-service:6379"
        envFrom:
        - configMapRef:
            name: rainbow-config
        resources:
          requests:
            memory: "1Gi"
            cpu: "500m"
          limits:
            memory: "2Gi"
            cpu: "1000m"
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5
        volumeMounts:
        - name: data
          mountPath: /app/data
        - name: logs
          mountPath: /app/logs
      volumes:
      - name: data
        persistentVolumeClaim:
          claimName: rainbow-data-pvc
      - name: logs
        emptyDir: {}
---
# service.yaml
apiVersion: v1
kind: Service
metadata:
  name: rainbow-browser-service
  namespace: rainbow-browser
spec:
  selector:
    app: rainbow-browser
  ports:
  - name: http
    port: 80
    targetPort: 8080
  - name: metrics
    port: 9090
    targetPort: 9090
  type: LoadBalancer
---
# pvc.yaml
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: rainbow-data-pvc
  namespace: rainbow-browser
spec:
  accessModes:
  - ReadWriteMany
  resources:
    requests:
      storage: 10Gi
  storageClassName: fast-ssd
```

### 2. Helm Chart 部署

```bash
# 添加 Helm 仓库
helm repo add rainbow https://charts.rainbow-browser.ai
helm repo update

# 使用自定义值文件部署
cat > values.yaml <<EOF
replicaCount: 3

image:
  repository: rainbow/browser
  tag: v8.0
  pullPolicy: IfNotPresent

service:
  type: LoadBalancer
  port: 80

ingress:
  enabled: true
  className: nginx
  hosts:
    - host: browser.example.com
      paths:
        - path: /
          pathType: Prefix

resources:
  limits:
    cpu: 1000m
    memory: 2Gi
  requests:
    cpu: 500m
    memory: 1Gi

autoscaling:
  enabled: true
  minReplicas: 3
  maxReplicas: 10
  targetCPUUtilizationPercentage: 70

persistence:
  enabled: true
  storageClass: fast-ssd
  size: 10Gi

surrealdb:
  enabled: true
  auth:
    rootPassword: secretpassword

redis:
  enabled: true
  auth:
    enabled: true
    password: redispassword

monitoring:
  enabled: true
  prometheus:
    enabled: true
  grafana:
    enabled: true
EOF

# 部署
helm install rainbow-browser rainbow/browser -f values.yaml -n rainbow-browser --create-namespace
```

### 3. 高可用配置

```yaml
# hpa.yaml - 水平自动扩缩
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: rainbow-browser-hpa
  namespace: rainbow-browser
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: rainbow-browser
  minReplicas: 3
  maxReplicas: 20
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
  - type: Pods
    pods:
      metric:
        name: http_requests_per_second
      target:
        type: AverageValue
        averageValue: "1000"
---
# pdb.yaml - Pod 中断预算
apiVersion: policy/v1
kind: PodDisruptionBudget
metadata:
  name: rainbow-browser-pdb
  namespace: rainbow-browser
spec:
  minAvailable: 2
  selector:
    matchLabels:
      app: rainbow-browser
```

## 📊 监控配置

### 1. Prometheus 配置

```yaml
# prometheus.yml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

scrape_configs:
  - job_name: 'rainbow-browser'
    static_configs:
      - targets: ['rainbow-browser:9090']
    metrics_path: '/metrics'
    
  - job_name: 'surrealdb'
    static_configs:
      - targets: ['surrealdb:8000']
    metrics_path: '/metrics'
    
  - job_name: 'redis'
    static_configs:
      - targets: ['redis-exporter:9121']

# 告警规则
rule_files:
  - 'alerts.yml'

alerting:
  alertmanagers:
    - static_configs:
        - targets: ['alertmanager:9093']
```

### 2. 告警规则

```yaml
# alerts.yml
groups:
  - name: rainbow_browser_alerts
    interval: 30s
    rules:
      - alert: HighMemoryUsage
        expr: |
          (sum(container_memory_usage_bytes{pod=~"rainbow-browser.*"}) / 
           sum(container_spec_memory_limit_bytes{pod=~"rainbow-browser.*"})) > 0.9
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High memory usage detected"
          description: "Memory usage is above 90% for 5 minutes"
          
      - alert: HighErrorRate
        expr: |
          sum(rate(http_requests_total{status=~"5.."}[5m])) /
          sum(rate(http_requests_total[5m])) > 0.05
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "High error rate detected"
          description: "Error rate is above 5% for 5 minutes"
          
      - alert: SlowResponseTime
        expr: |
          histogram_quantile(0.95, 
            sum(rate(http_request_duration_seconds_bucket[5m])) by (le)
          ) > 1
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Slow response time detected"
          description: "95th percentile response time is above 1s"
```

### 3. Grafana Dashboard

```json
{
  "dashboard": {
    "title": "Rainbow Browser V8.0 Dashboard",
    "panels": [
      {
        "title": "Request Rate",
        "targets": [
          {
            "expr": "sum(rate(http_requests_total[5m])) by (status)"
          }
        ]
      },
      {
        "title": "Response Time",
        "targets": [
          {
            "expr": "histogram_quantile(0.95, sum(rate(http_request_duration_seconds_bucket[5m])) by (le))"
          }
        ]
      },
      {
        "title": "Perception Performance",
        "targets": [
          {
            "expr": "histogram_quantile(0.95, sum(rate(perception_duration_bucket[5m])) by (le, mode))"
          }
        ]
      },
      {
        "title": "Active Sessions",
        "targets": [
          {
            "expr": "rainbow_browser_active_sessions"
          }
        ]
      }
    ]
  }
}
```

## 🔒 安全配置

### 1. TLS/SSL 配置

```yaml
# ingress-tls.yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: rainbow-browser-ingress
  namespace: rainbow-browser
  annotations:
    cert-manager.io/cluster-issuer: letsencrypt-prod
    nginx.ingress.kubernetes.io/ssl-redirect: "true"
spec:
  tls:
  - hosts:
    - browser.example.com
    secretName: rainbow-browser-tls
  rules:
  - host: browser.example.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: rainbow-browser-service
            port:
              number: 80
```

### 2. 网络策略

```yaml
# network-policy.yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: rainbow-browser-netpol
  namespace: rainbow-browser
spec:
  podSelector:
    matchLabels:
      app: rainbow-browser
  policyTypes:
  - Ingress
  - Egress
  ingress:
  - from:
    - namespaceSelector:
        matchLabels:
          name: ingress-nginx
    ports:
    - protocol: TCP
      port: 8080
  egress:
  - to:
    - podSelector:
        matchLabels:
          app: surrealdb
    ports:
    - protocol: TCP
      port: 8000
  - to:
    - podSelector:
        matchLabels:
          app: redis
    ports:
    - protocol: TCP
      port: 6379
```

## 🔧 运维操作

### 1. 备份策略

```bash
#!/bin/bash
# backup.sh

# 备份 SurrealDB
docker exec rainbow-surrealdb surreal export \
  --conn http://localhost:8000 \
  --user root \
  --pass ${SURREALDB_PASS} \
  --ns rainbow \
  --db browser \
  /backup/surrealdb-$(date +%Y%m%d-%H%M%S).sql

# 备份 Redis
docker exec rainbow-redis redis-cli BGSAVE
docker cp rainbow-redis:/data/dump.rdb ./backup/redis-$(date +%Y%m%d-%H%M%S).rdb

# 上传到对象存储
aws s3 sync ./backup s3://rainbow-browser-backup/$(date +%Y%m%d)/
```

### 2. 滚动更新

```bash
# Docker Swarm
docker service update \
  --image rainbow/browser:v8.1 \
  --update-parallelism 1 \
  --update-delay 30s \
  rainbow-browser

# Kubernetes
kubectl set image deployment/rainbow-browser \
  rainbow-browser=rainbow/browser:v8.1 \
  -n rainbow-browser

# 监控更新状态
kubectl rollout status deployment/rainbow-browser -n rainbow-browser
```

### 3. 故障恢复

```bash
# 检查服务状态
docker-compose ps
kubectl get pods -n rainbow-browser

# 查看日志
docker-compose logs -f rainbow-browser
kubectl logs -f deployment/rainbow-browser -n rainbow-browser

# 重启服务
docker-compose restart rainbow-browser
kubectl rollout restart deployment/rainbow-browser -n rainbow-browser

# 从备份恢复
docker exec -i rainbow-surrealdb surreal import \
  --conn http://localhost:8000 \
  --user root \
  --pass ${SURREALDB_PASS} \
  --ns rainbow \
  --db browser \
  < backup/surrealdb-latest.sql
```

## 📈 性能调优

### 1. 系统参数优化

```bash
# /etc/sysctl.conf
net.core.somaxconn = 65535
net.ipv4.tcp_max_syn_backlog = 65535
net.ipv4.ip_local_port_range = 1024 65535
net.ipv4.tcp_tw_reuse = 1
net.ipv4.tcp_fin_timeout = 30
fs.file-max = 1000000
```

### 2. 容器资源限制

```yaml
resources:
  requests:
    memory: "1Gi"
    cpu: "500m"
    ephemeral-storage: "1Gi"
  limits:
    memory: "2Gi"
    cpu: "1000m"
    ephemeral-storage: "2Gi"
```

### 3. 应用配置优化

```env
# 性能相关环境变量
TOKIO_WORKER_THREADS=8
RAYON_NUM_THREADS=8
RUST_LOG=warn
CACHE_CAPACITY=10000
CONNECTION_POOL_SIZE=100
```

## 🎯 生产检查清单

- [ ] **基础设施**
  - [ ] 足够的计算资源（CPU、内存）
  - [ ] 持久化存储配置
  - [ ] 网络带宽充足
  
- [ ] **安全**
  - [ ] TLS/SSL 证书配置
  - [ ] 防火墙规则设置
  - [ ] 密钥和密码安全存储
  
- [ ] **高可用**
  - [ ] 多副本部署
  - [ ] 负载均衡配置
  - [ ] 自动故障转移
  
- [ ] **监控**
  - [ ] 指标收集配置
  - [ ] 日志聚合设置
  - [ ] 告警规则定义
  
- [ ] **备份**
  - [ ] 自动备份脚本
  - [ ] 备份验证流程
  - [ ] 恢复测试完成

---

**部署愉快！** 🚀