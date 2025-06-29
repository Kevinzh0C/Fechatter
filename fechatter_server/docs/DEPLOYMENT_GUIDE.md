# Fechatter Server Deployment Guide

## 📋 Table of Contents

1. [Overview](#overview)
2. [Prerequisites](#prerequisites)
3. [Environment Configuration](#environment-configuration)
4. [Database Setup](#database-setup)
5. [Docker Deployment](#docker-deployment)
6. [Kubernetes Deployment](#kubernetes-deployment)
7. [Monitoring & Logging](#monitoring--logging)
8. [Security Considerations](#security-considerations)
9. [Performance Tuning](#performance-tuning)
10. [Maintenance & Operations](#maintenance--operations)

## 🎯 Overview

This guide covers deploying Fechatter Server in production environments. The server supports multiple deployment strategies:

- **Docker Compose**: Single server or small deployments
- **Kubernetes**: Large-scale, high-availability deployments
- **Cloud Platforms**: AWS, GCP, Azure
- **Bare Metal**: Direct installation on servers

## 📋 Prerequisites

### System Requirements

- **CPU**: 4+ cores (8+ recommended)
- **RAM**: 8GB minimum (16GB+ recommended)
- **Storage**: 50GB+ SSD
- **OS**: Ubuntu 20.04+ or compatible Linux distribution

### Software Dependencies

- **Docker**: 20.10+
- **PostgreSQL**: 14+
- **Redis**: 6.0+
- **Nginx**: 1.18+ (as reverse proxy)
- **Meilisearch**: 1.0+ (optional)

## ⚙️ Environment Configuration

### Production Environment Variables

```bash
# Application
RUST_ENV=production
HOST=0.0.0.0
PORT=8080

# Database
DATABASE_URL=postgresql://fechatter:password@db:5432/fechatter_prod
DATABASE_MAX_CONNECTIONS=100
DATABASE_MIN_CONNECTIONS=10

# Redis
REDIS_URL=redis://redis:6379
REDIS_MAX_CONNECTIONS=50

# Security
JWT_SECRET=your-strong-secret-key-here
ENCRYPTION_KEY=your-32-byte-encryption-key
TOKEN_EXPIRY_SECONDS=3600

# Search (Optional)
MEILISEARCH_URL=http://meilisearch:7700
MEILISEARCH_KEY=your-master-key

# Performance
ENABLE_CIRCUIT_BREAKER=true
CONNECTION_LIMIT=1000
CACHE_TTL_SECONDS=300
REQUEST_TIMEOUT_SECONDS=30

# Monitoring
ENABLE_METRICS=true
METRICS_PORT=9090
LOG_LEVEL=info
```

### Configuration File

Create `config/production.yml`:

```yaml
server:
  host: 0.0.0.0
  port: 8080
  workers: 8
  keep_alive: 75
  request_timeout: 30

database:
  url: ${DATABASE_URL}
  max_connections: 100
  min_connections: 10
  acquire_timeout: 3
  idle_timeout: 600
  max_lifetime: 1800

redis:
  url: ${REDIS_URL}
  pool_size: 50
  connection_timeout: 5

security:
  jwt_secret: ${JWT_SECRET}
  token_expiry_seconds: 3600
  bcrypt_cost: 12
  rate_limit:
    requests_per_minute: 60
    burst_size: 10

cache:
  default_ttl: 300
  max_size: 10000
  eviction_policy: lru

monitoring:
  enable_metrics: true
  enable_tracing: true
  jaeger_endpoint: http://jaeger:14268/api/traces
```

## 💾 Database Setup

### PostgreSQL Configuration

```sql
-- Create production database
CREATE DATABASE fechatter_prod;
CREATE USER fechatter WITH ENCRYPTED PASSWORD 'your-secure-password';
GRANT ALL PRIVILEGES ON DATABASE fechatter_prod TO fechatter;

-- Performance tuning
ALTER SYSTEM SET shared_buffers = '4GB';
ALTER SYSTEM SET effective_cache_size = '12GB';
ALTER SYSTEM SET maintenance_work_mem = '1GB';
ALTER SYSTEM SET checkpoint_completion_target = 0.9;
ALTER SYSTEM SET wal_buffers = '16MB';
ALTER SYSTEM SET default_statistics_target = 100;
ALTER SYSTEM SET random_page_cost = 1.1;
ALTER SYSTEM SET effective_io_concurrency = 200;
ALTER SYSTEM SET work_mem = '20MB';
ALTER SYSTEM SET min_wal_size = '1GB';
ALTER SYSTEM SET max_wal_size = '4GB';
```

### Run Migrations

```bash
# Using Docker
docker run --rm \
  -e DATABASE_URL="postgresql://fechatter:password@db:5432/fechatter_prod" \
  fechatter/server:latest \
  /app/migrate

# Or directly
cargo run --bin migrate --release
```

### Backup Strategy

```bash
#!/bin/bash
# backup.sh
DATE=$(date +%Y%m%d_%H%M%S)
BACKUP_DIR="/backups/postgres"

# Create backup
pg_dump $DATABASE_URL | gzip > "$BACKUP_DIR/fechatter_$DATE.sql.gz"

# Keep only last 7 days
find $BACKUP_DIR -name "*.sql.gz" -mtime +7 -delete
```

## 🐳 Docker Deployment

### Docker Compose Production

```yaml
version: '3.8'

services:
  fechatter:
    image: fechatter/server:latest
    ports:
      - "8080:8080"
    environment:
      - RUST_ENV=production
      - DATABASE_URL=postgresql://fechatter:password@db:5432/fechatter_prod
      - REDIS_URL=redis://redis:6379
      - JWT_SECRET=${JWT_SECRET}
    depends_on:
      - db
      - redis
    restart: unless-stopped
    deploy:
      resources:
        limits:
          cpus: '2'
          memory: 4G
        reservations:
          cpus: '1'
          memory: 2G

  db:
    image: postgres:14-alpine
    volumes:
      - postgres_data:/var/lib/postgresql/data
    environment:
      - POSTGRES_DB=fechatter_prod
      - POSTGRES_USER=fechatter
      - POSTGRES_PASSWORD=password
    restart: unless-stopped

  redis:
    image: redis:7-alpine
    command: redis-server --appendonly yes
    volumes:
      - redis_data:/data
    restart: unless-stopped

  nginx:
    image: nginx:alpine
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf:ro
      - ./certs:/etc/nginx/certs:ro
    depends_on:
      - fechatter
    restart: unless-stopped

volumes:
  postgres_data:
  redis_data:
```

### Nginx Configuration

```nginx
upstream fechatter {
    server fechatter:8080;
}

server {
    listen 80;
    server_name api.fechatter.com;
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl http2;
    server_name api.fechatter.com;

    ssl_certificate /etc/nginx/certs/cert.pem;
    ssl_certificate_key /etc/nginx/certs/key.pem;
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers HIGH:!aNULL:!MD5;

    client_max_body_size 10M;

    location / {
        proxy_pass http://fechatter;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_read_timeout 86400;
    }

    location /health {
        proxy_pass http://fechatter/health;
        access_log off;
    }
}
```

### Building Docker Image

```dockerfile
# Dockerfile
FROM rust:1.70 as builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations

RUN cargo build --release

FROM debian:bullseye-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl1.1 \
    libpq5 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/fechatter_server /app/fechatter_server
COPY --from=builder /app/target/release/migrate /app/migrate
COPY --from=builder /app/migrations /app/migrations

EXPOSE 8080

CMD ["/app/fechatter_server"]
```

## ☸️ Kubernetes Deployment

### Deployment Manifest

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: fechatter-server
  namespace: fechatter
spec:
  replicas: 3
  selector:
    matchLabels:
      app: fechatter-server
  template:
    metadata:
      labels:
        app: fechatter-server
    spec:
      containers:
      - name: fechatter
        image: fechatter/server:latest
        ports:
        - containerPort: 8080
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: fechatter-secrets
              key: database-url
        - name: REDIS_URL
          value: "redis://redis-service:6379"
        - name: JWT_SECRET
          valueFrom:
            secretKeyRef:
              name: fechatter-secrets
              key: jwt-secret
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
            path: /health
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5
---
apiVersion: v1
kind: Service
metadata:
  name: fechatter-service
  namespace: fechatter
spec:
  selector:
    app: fechatter-server
  ports:
  - port: 80
    targetPort: 8080
  type: LoadBalancer
```

### Horizontal Pod Autoscaler

```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: fechatter-hpa
  namespace: fechatter
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: fechatter-server
  minReplicas: 3
  maxReplicas: 10
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
```

## 📊 Monitoring & Logging

### Prometheus Configuration

```yaml
# prometheus.yml
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'fechatter'
    static_configs:
    - targets: ['fechatter:9090']
    metrics_path: '/metrics'
```

### Grafana Dashboard

Import the Fechatter dashboard from `grafana-dashboards/fechatter-overview.json`

Key metrics to monitor:
- Request rate and latency
- Active connections
- Database connection pool usage
- Cache hit rates
- Error rates
- CPU and memory usage

### Logging Setup

```yaml
# fluent-bit configuration
[SERVICE]
    Flush        1
    Daemon       Off
    Log_Level    info

[INPUT]
    Name              forward
    Listen            0.0.0.0
    Port              24224

[FILTER]
    Name              parser
    Match             fechatter.*
    Key_Name          log
    Parser            json

[OUTPUT]
    Name              es
    Match             *
    Host              elasticsearch
    Port              9200
    Index             fechatter
    Type              _doc
```

## 🔒 Security Considerations

### SSL/TLS Configuration

```bash
# Generate SSL certificate with Let's Encrypt
certbot certonly --standalone -d api.fechatter.com
```

### Firewall Rules

```bash
# Allow only necessary ports
ufw default deny incoming
ufw default allow outgoing
ufw allow 22/tcp    # SSH
ufw allow 80/tcp    # HTTP
ufw allow 443/tcp   # HTTPS
ufw allow 9090/tcp  # Metrics (internal only)
ufw enable
```

### Security Headers

Add to Nginx configuration:

```nginx
add_header X-Frame-Options "SAMEORIGIN" always;
add_header X-Content-Type-Options "nosniff" always;
add_header X-XSS-Protection "1; mode=block" always;
add_header Referrer-Policy "no-referrer-when-downgrade" always;
add_header Content-Security-Policy "default-src 'self' http: https: data: blob: 'unsafe-inline'" always;
add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;
```

## ⚡ Performance Tuning

### System Limits

```bash
# /etc/security/limits.conf
* soft nofile 65536
* hard nofile 65536
* soft nproc 32768
* hard nproc 32768
```

### Kernel Parameters

```bash
# /etc/sysctl.conf
net.core.somaxconn = 65535
net.ipv4.tcp_max_syn_backlog = 65535
net.ipv4.ip_local_port_range = 1024 65535
net.ipv4.tcp_tw_reuse = 1
net.ipv4.tcp_fin_timeout = 15
net.core.netdev_max_backlog = 65535
net.ipv4.tcp_keepalive_time = 300
net.ipv4.tcp_keepalive_probes = 5
net.ipv4.tcp_keepalive_intvl = 15
```

### Application Tuning

```yaml
# Production configuration
server:
  workers: ${CPU_COUNT}  # Set to number of CPU cores
  keep_alive: 75
  request_timeout: 30
  
database:
  max_connections: 100   # Adjust based on load
  statement_cache_size: 100
  
cache:
  max_size: 100000      # Increase for better performance
  ttl: 600              # Longer TTL for stable data
```

## 🔧 Maintenance & Operations

### Health Checks

```bash
# Check application health
curl https://api.fechatter.com/health

# Check detailed health
curl https://api.fechatter.com/admin/production/health
```

### Rolling Updates

```bash
# Kubernetes rolling update
kubectl set image deployment/fechatter-server \
  fechatter=fechatter/server:v2.0.0 \
  -n fechatter

# Monitor rollout
kubectl rollout status deployment/fechatter-server -n fechatter
```

### Backup and Restore

```bash
# Automated backup script
#!/bin/bash
# backup-all.sh

# Database backup
pg_dump $DATABASE_URL | gzip > /backup/db_$(date +%Y%m%d).sql.gz

# Redis backup
redis-cli --rdb /backup/redis_$(date +%Y%m%d).rdb

# Application data
tar -czf /backup/data_$(date +%Y%m%d).tar.gz /app/data

# Upload to S3
aws s3 sync /backup s3://fechatter-backups/$(date +%Y%m%d)/
```

### Monitoring Alerts

```yaml
# Alertmanager rules
groups:
- name: fechatter
  rules:
  - alert: HighErrorRate
    expr: rate(http_requests_total{status=~"5.."}[5m]) > 0.05
    for: 5m
    annotations:
      summary: "High error rate detected"
      
  - alert: DatabaseConnectionPoolExhausted
    expr: db_connections_active / db_connections_max > 0.9
    for: 5m
    annotations:
      summary: "Database connection pool nearly exhausted"
```

## 📈 Scaling Strategies

### Vertical Scaling
- Increase CPU/RAM for single instance
- Optimize database queries
- Increase cache size

### Horizontal Scaling
- Add more application instances
- Use load balancer
- Implement database read replicas
- Distribute cache across multiple Redis nodes

### Auto-scaling Rules
```yaml
# Scale up when:
- CPU usage > 70% for 5 minutes
- Memory usage > 80% for 5 minutes
- Request queue > 100 for 2 minutes

# Scale down when:
- CPU usage < 30% for 10 minutes
- Memory usage < 40% for 10 minutes
```

---

**Version**: 1.0.0  
**Last Updated**: December 2024  
**Status**: Production Ready ✅ 