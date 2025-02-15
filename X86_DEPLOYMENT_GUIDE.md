# x86_64 Cross-compilation Deployment Guide

## Overview

This guide provides a complete **production-grade** workflow for cross-compiling Fechatter Rust crates to x86_64 architecture and deploying them via Docker containers.

## 🎯 Architecture

```
Local Development (macOS/ARM64) 
    ↓ Cross-compilation
x86_64 Linux Binaries 
    ↓ Docker Packaging  
Production-Ready Containers
    ↓ Container Orchestration
Deployed Services
```

## 📋 Prerequisites

### Required Tools
- **Rust** (latest stable)
- **Docker** with BuildKit support
- **Docker Compose** v2.x
- **Cross** tool (auto-installed if missing)

### System Requirements
- **8GB RAM** minimum for cross-compilation
- **10GB** free disk space
- **Internet connection** for dependency downloads

## 🚀 Quick Start

### 1. Environment Setup
```bash
# Copy environment template
cp env.x86.template .env

# Customize environment variables (optional)
nano .env
```

### 2. One-Click Deployment
```bash
# Deploy core services (recommended for development)
./deploy-x86.sh

# Deploy full stack with all services
./deploy-x86.sh -m full

# Deploy all-in-one container (fastest startup)
./deploy-x86.sh -m allinone
```

### 3. Verify Deployment
```bash
# Check service health
curl http://localhost:6688/health
curl http://localhost:6690/health  
curl http://localhost:6687/health

# View service logs
docker compose -f docker-compose.local.yml logs -f
```

## 🔧 Manual Workflow

### Step 1: Cross-compile Binaries
```bash
# Production release build
./build-cross.sh --profile release

# Debug build with clean
./build-cross.sh --profile debug --clean
```

### Step 2: Build Docker Images
```bash
# Build all service images
docker compose -f docker-compose.local.yml build

# Build specific service
docker compose -f docker-compose.local.yml build fechatter-server
```

### Step 3: Deploy Services
```bash
# Infrastructure only
docker compose -f docker-compose.local.yml --profile infrastructure up -d

# Core application services  
docker compose -f docker-compose.local.yml --profile core up -d

# Full stack with bot service
docker compose -f docker-compose.local.yml --profile full up -d

# All-in-one deployment
BOT_ENABLED=true GATEWAY_ENABLED=true \
docker compose -f docker-compose.local.yml --profile allinone up -d
```

## 📊 Deployment Modes

### Infrastructure Mode
**Services**: PostgreSQL, Redis, NATS, ClickHouse, MeiliSearch
**Usage**: Database and messaging infrastructure only
**Command**: `./deploy-x86.sh -m infrastructure`

### Core Mode (Default)
**Services**: Infrastructure + Fechatter Server + Analytics + Notifications  
**Usage**: Essential application services
**Command**: `./deploy-x86.sh -m core`

### Full Mode
**Services**: Core + Bot Service
**Usage**: Complete application with AI features
**Command**: `./deploy-x86.sh -m full`

### All-in-One Mode  
**Services**: All services in single container with supervisor
**Usage**: Fastest startup for development
**Command**: `./deploy-x86.sh -m allinone`

## 🛠️ Configuration

### Environment Variables
Key variables in `.env` file:

```bash
# Build configuration
RUST_LOG=debug,sqlx=warn
ARCHITECTURE=x86_64

# Feature toggles for all-in-one mode
BOT_ENABLED=true
GATEWAY_ENABLED=true  
RAG_ENABLED=true

# External API keys
OPENAI_API_KEY=your-key-here
JWT_SECRET=production-secret
```

### Service Configuration
- **Fechatter Server**: `fechatter_server/chat.yml`
- **Analytics Server**: `analytics_server/analytics.yml`  
- **Notify Server**: `notify_server/notify.yml`
- **Bot Server**: `bot_server/bot.yml`
- **Gateway**: `fechatter_gateway/config/development.yml`

## 🔍 Monitoring & Debugging

### Health Checks
```bash
# Individual service health
curl http://localhost:6688/health  # Fechatter Server
curl http://localhost:6690/health  # Analytics Server  
curl http://localhost:6687/health  # Notify Server
curl http://localhost:6686/health  # Bot Server
curl http://localhost:8080/health  # Gateway

# Database connectivity
psql postgresql://fechatter:fechatter_password@localhost:5432/fechatter

# Redis connectivity  
redis-cli -h localhost -p 6379 -a fechatter_redis_pass ping
```

### Log Management
```bash
# View all service logs
docker compose -f docker-compose.local.yml logs -f

# Service-specific logs
docker compose -f docker-compose.local.yml logs -f fechatter-server

# Supervisor logs (all-in-one mode)
docker exec fechatter-all-in-one-local supervisorctl status
```

### Container Management
```bash
# Check container status
docker compose -f docker-compose.local.yml ps

# Restart specific service
docker compose -f docker-compose.local.yml restart fechatter-server

# Scale services
docker compose -f docker-compose.local.yml up -d --scale fechatter-server=2
```

## 🚨 Troubleshooting

### Build Issues
```bash
# Clean rebuild
./deploy-x86.sh -c -m core

# Manual cross-compilation debug
cross build --release --target x86_64-unknown-linux-gnu -p fechatter_server --verbose

# Check target architecture
file target/main/release/fechatter_server
```

### Runtime Issues
```bash
# Check binary compatibility
docker run --rm -v $(pwd)/target/main/release:/binaries alpine:3.20 file /binaries/*

# Debug container startup  
docker compose -f docker-compose.local.yml up fechatter-server

# Inspect container
docker exec -it fechatter-server-local sh
```

### Network Issues
```bash
# Check port availability
netstat -tulpn | grep -E ':(6688|6690|6687|6686|8080)'

# Test internal connectivity
docker compose -f docker-compose.local.yml exec fechatter-server curl http://postgres:5432
```

## 🔄 Development Workflow

### Code → Deploy Cycle
```bash
# 1. Make code changes
# 2. Cross-compile and deploy
./deploy-x86.sh -m core

# 3. Test changes
curl http://localhost:6688/api/test

# 4. View logs
docker compose -f docker-compose.local.yml logs -f fechatter-server
```

### Quick Service Rebuild
```bash
# Rebuild single service after changes
./build-cross.sh --profile release
docker compose -f docker-compose.local.yml build fechatter-server  
docker compose -f docker-compose.local.yml up -d fechatter-server
```

## 📈 Performance Tuning

### Build Optimization
```bash
# Use release profile for production
./deploy-x86.sh -p release -m full

# Parallel compilation
export CARGO_BUILD_JOBS=8
./build-cross.sh --profile release
```

### Runtime Optimization
```bash
# Increase container resources
docker compose -f docker-compose.local.yml up -d --scale fechatter-server=2

# Enable performance monitoring
ENABLE_METRICS=true ./deploy-x86.sh -m full
```

## 🔐 Security Considerations

### Container Security
- Non-root user execution
- Read-only file systems where possible
- Minimal attack surface with Alpine base
- Version-pinned dependencies

### Network Security  
```bash
# Internal network isolation
docker network ls | grep fechatter

# Port exposure review
docker compose -f docker-compose.local.yml config | grep ports -A2
```

## 🗂️ File Structure

```
.
├── build-cross.sh              # Cross-compilation script
├── deploy-x86.sh              # One-click deployment script  
├── Dockerfile.local           # Multi-stage Docker build
├── docker-compose.local.yml   # Service orchestration
├── env.x86.template           # Environment template
├── docker/
│   └── supervisor.conf        # All-in-one service management
└── target/main/release/       # Compiled x86_64 binaries
```

## 🎉 Success Criteria

After successful deployment, you should have:

✅ **All services running** and responding to health checks  
✅ **Database connectivity** established  
✅ **Inter-service communication** working  
✅ **Logging and monitoring** operational  
✅ **API endpoints** accessible via HTTP  

## 📚 Additional Resources

- [Rust Cross-compilation Guide](https://github.com/cross-rs/cross)
- [Docker Buildx Documentation](https://docs.docker.com/buildx/)
- [Alpine Linux Security](https://alpinelinux.org/about/security/)
- [Supervisor Process Management](http://supervisord.org/)

---

**Questions or Issues?** Check the troubleshooting section or review service logs for specific error messages. 