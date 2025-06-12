# Fly.io Production Deployment Guide

This guide provides step-by-step instructions for deploying Fechatter to Fly.io in production.

## 🚀 Quick Start (Recommended - Fast Deployment)

Using pre-compiled binaries saves ~7 minutes per deployment:

```bash
# One-time setup
cp fly/env.production.template .env.production
# Edit .env.production with your configuration

# Deploy (builds binaries on first run, then fast deployments)
chmod +x deploy-to-fly-fast.sh
./deploy-to-fly-fast.sh
```

## Prerequisites

1. **Install Fly CLI**
   ```bash
   curl -L https://fly.io/install.sh | sh
   ```

2. **Login to Fly.io**
   ```bash
   flyctl auth login
   ```

3. **Install Rust with musl target**
   ```bash
   rustup target add x86_64-unknown-linux-musl
   ```

## Deployment Methods

### Method 1: Fast Deployment with Pre-compiled Binaries (Recommended)

This method compiles once locally and reuses binaries for subsequent deployments.

**First deployment:**
```bash
# Build musl static binaries (7 minutes, one-time)
./build-musl.sh

# Deploy using pre-built binaries (2 minutes)
./fly/deploy-production-fast.sh
```

**Subsequent deployments:**
```bash
# Just deploy (2 minutes) - no rebuild needed
./fly/deploy-production-fast.sh
```

**When to rebuild:**
- After code changes
- After dependency updates
- After Rust version updates

### Method 2: Traditional Deployment (Slower)

Builds during deployment (not recommended):
```bash
./fly/deploy-production.sh
```

## Build Process

### Building musl Static Binaries

The `build-musl.sh` script creates statically linked binaries suitable for Alpine Linux:

```bash
./build-musl.sh
```

Output location: `target/musl/release/`

Benefits:
- No runtime dependencies
- Smaller Docker images
- Better security
- Consistent behavior across environments

## Configuration

### Environment Variables

Create `.env.production` from template:
```bash
cp fly/env.production.template .env.production
```

Key settings:
- `FLY_APP_NAME`: Application name
- `FLY_REGION`: Deployment region
- Database and Redis configuration
- External service URLs

### Configuration Files

YML templates in `fly/config/`:
- `chat.yml` - Main server
- `analytics.yml` - Analytics server  
- `notify.yml` - Notification server
- `bot.yml` - Bot server
- `gateway.yml` - API Gateway

These use placeholders (`${VAR}`) replaced during deployment.

## Deployment Workflow

### Fast Deployment Flow

```
1. build-musl.sh
   ├── Compiles all services
   └── Creates static binaries in target/musl/release/

2. Docker build (Dockerfile.fly.prebuilt)
   ├── Uses Alpine Linux base
   ├── Copies pre-built binaries
   └── No compilation needed

3. fly deploy --local-only
   ├── Uses local Docker image
   └── Uploads to Fly.io
```

### Traditional Flow (Slower)

```
1. fly deploy
   ├── Sends code to Fly.io
   ├── Builds in Fly.io builders
   └── Takes ~10 minutes total
```

## Production Checklist

- [ ] Build musl binaries locally first
- [ ] Configure `.env.production` properly
- [ ] Set strong JWT_SECRET (32+ chars)
- [ ] Configure database with backups
- [ ] Set up monitoring
- [ ] Test disaster recovery

## Commands Reference

### Build Commands
```bash
# Build musl static binaries
./build-musl.sh

# Verify binaries
file target/musl/release/fechatter_server
```

### Deployment Commands
```bash
# Fast deployment (recommended)
./deploy-to-fly-fast.sh

# Traditional deployment
./deploy-to-fly.sh

# Deploy with Docker image
flyctl deploy --local-only --image fechatter-fly:latest
```

### Management Commands
```bash
# View logs
flyctl logs -a fechatter-prod

# SSH into container
flyctl ssh console -a fechatter-prod

# Scale application
flyctl scale count 2 -a fechatter-prod

# Update secrets
flyctl secrets set KEY=value -a fechatter-prod
```

## Troubleshooting

### Binary Issues

Check if binaries are statically linked:
```bash
file target/musl/release/fechatter_server
# Should show: "statically linked"
```

### Deployment Failures

1. Check Docker build:
   ```bash
   docker build -f Dockerfile.fly.prebuilt -t test .
   ```

2. Verify secrets:
   ```bash
   flyctl secrets list -a fechatter-prod
   ```

3. Check logs:
   ```bash
   flyctl logs -a fechatter-prod
   ```

## Architecture

```
┌─────────────────────────────┐
│   Local Development Machine  │
│  ┌────────────────────────┐ │
│  │ Pre-compiled Binaries  │ │
│  │ (musl static linked)   │ │
│  └───────────┬────────────┘ │
│              ↓              │
│  ┌────────────────────────┐ │
│  │   Docker Image Build   │ │
│  │ (Dockerfile.fly.prebuilt)│ │
│  └───────────┬────────────┘ │
└──────────────┼──────────────┘
               ↓
        flyctl deploy
        --local-only
               ↓
┌─────────────────────────────┐
│      Fly.io Platform        │
│  ┌────────────────────────┐ │
│  │   Alpine Container     │ │
│  │  with Static Binaries  │ │
│  └────────────────────────┘ │
└─────────────────────────────┘
```

## Performance Comparison

| Method | Build Time | Deploy Time | Total |
|--------|------------|-------------|-------|
| Traditional | 0 min | 10 min | 10 min |
| Pre-compiled (first) | 7 min | 2 min | 9 min |
| Pre-compiled (subsequent) | 0 min | 2 min | 2 min |

## Best Practices

1. **Build Once, Deploy Many**
   - Compile binaries locally
   - Reuse for multiple deployments
   - Only rebuild when code changes

2. **Version Control**
   - Tag releases before deployment
   - Keep binary versions in sync with code

3. **Security**
   - Use strong secrets
   - Enable HTTPS only
   - Regular security updates

4. **Monitoring**
   - Set up alerts
   - Monitor resource usage
   - Track deployment metrics

## Detailed Setup

### 1. Environment Configuration

Create `.env.production` from the template and configure:

- **FLY_APP_NAME**: Your application name
- **FLY_REGION**: Deployment region (nrt for Tokyo, iad for Virginia, etc.)
- **Database**: Choose Fly Postgres or external database
- **Redis**: Choose Upstash Redis or external Redis
- **External Services**: Configure NATS, Meilisearch, ClickHouse

### 2. Build Process

The deployment script automatically builds musl-static binaries. To build manually:

```bash
./build-cross.sh --profile release --target x86_64-unknown-linux-musl
```

### 3. Configuration Files

Configuration templates are in `fly/config/`:
- `chat.yml` - Main server configuration
- `analytics.yml` - Analytics server configuration  
- `notify.yml` - Notification server configuration
- `bot.yml` - Bot server configuration
- `gateway.yml` - API Gateway configuration

These use placeholders like `${DATABASE_URL}` that are replaced during deployment.

### 4. External Services Setup

#### PostgreSQL (Choose one)

**Option 1: Fly Postgres** (Recommended)
```bash
USE_FLY_POSTGRES=true  # in .env.production
```

**Option 2: External PostgreSQL**
```bash
DATABASE_URL=postgresql://user:password@host:5432/database
```

#### Redis (Choose one)

**Option 1: Upstash Redis**
```bash
USE_UPSTASH_REDIS=true  # in .env.production
```

**Option 2: External Redis**
```bash
REDIS_URL=redis://:password@host:6379
```

#### NATS

Deploy NATS on a cloud provider or use a managed service:
```bash
NATS_URL=nats://nats.example.com:4222
```

#### Meilisearch

Deploy Meilisearch or use Meilisearch Cloud:
```bash
MEILISEARCH_URL=https://ms.example.com
MEILISEARCH_KEY=your-master-key
```

#### ClickHouse

Use ClickHouse Cloud or self-hosted:
```bash
CLICKHOUSE_HOST=ch.example.com
CLICKHOUSE_PORT=8123
CLICKHOUSE_USER=default
CLICKHOUSE_PASSWORD=your-password
```

### 5. Deployment Commands

**Initial deployment:**
```bash
./fly/deploy-production.sh
```

**Update existing deployment:**
```bash
SKIP_BUILD=true ./fly/deploy-production.sh  # Skip build if binaries unchanged
```

**Scale application:**
```bash
flyctl scale count 2 --app fechatter-prod
flyctl scale vm shared-cpu-2x --app fechatter-prod
```

### 6. Post-Deployment

**View logs:**
```bash
flyctl logs --app fechatter-prod
```

**SSH into container:**
```bash
flyctl ssh console --app fechatter-prod
```

**View metrics:**
```bash
flyctl monitor --app fechatter-prod
```

**Update secrets:**
```bash
flyctl secrets set KEY=value --app fechatter-prod
```

## Production Checklist

- [ ] Configure production database with backups
- [ ] Set up Redis with persistence
- [ ] Configure NATS with clustering
- [ ] Enable Meilisearch authentication
- [ ] Set strong JWT_SECRET (32+ characters)
- [ ] Configure custom domain
- [ ] Set up monitoring and alerts
- [ ] Configure log aggregation
- [ ] Test disaster recovery

## Troubleshooting

### Services not starting

Check supervisor logs:
```bash
flyctl ssh console --app fechatter-prod
cat /var/log/fechatter/supervisord.log
```

### Configuration issues

Verify config files were created:
```bash
flyctl ssh console --app fechatter-prod
ls -la /app/config/
```

### Database connection issues

Check DATABASE_URL format and network connectivity:
```bash
flyctl ssh console --app fechatter-prod
psql $DATABASE_URL -c "SELECT 1"
```

## Architecture on Fly.io

```
┌─────────────────────────────────────────────┐
│             Fly.io Edge Network             │
└─────────────────────┬───────────────────────┘
                      │
         ┌────────────▼────────────┐
         │   Gateway (Port 8080)   │
         └────────────┬────────────┘
                      │
    ┌─────────────────┴─────────────────┐
    │                                   │
┌───▼────┐  ┌────▼────┐  ┌────▼────┐  ┌────▼────┐
│Fechatter│  │Analytics│  │ Notify  │  │  Bot    │
│ :6688   │  │  :6690  │  │  :6687  │  │  :6686  │
└─────────┘  └─────────┘  └─────────┘  └─────────┘
    │            │            │            │
    └────────────┴────────────┴────────────┘
                      │
         ┌────────────▼────────────┐
         │   Persistent Volume     │
         │     /data (10GB)        │
         └─────────────────────────┘
```

## Cost Optimization

- Use `auto_stop_machines = true` for dev/staging
- Start with shared-cpu-1x and scale as needed
- Use Fly Postgres dev plan for small deployments
- Monitor usage with `flyctl billing`

## Support

For issues specific to Fly.io deployment, check:
- [Fly.io Documentation](https://fly.io/docs)
- [Fly.io Community](https://community.fly.io) 