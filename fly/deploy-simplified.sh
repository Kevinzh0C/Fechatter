#!/bin/bash
# deploy-simplified.sh - Simplified deployment for core services only

set -e

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m'

print_info() { echo -e "${GREEN}[INFO]${NC} $1"; }
print_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
print_error() { echo -e "${RED}[ERROR]${NC} $1"; }
print_step() { echo -e "${BLUE}[STEP]${NC} $1"; }

# Default values
APP_NAME="${FLY_APP_NAME:-fechatter-demo}"
REGION="${FLY_REGION:-nrt}"
ORG="${FLY_ORG:-personal}"

print_step "🚀 Fechatter Simplified Deployment to Fly.io"
echo ""
echo "This deployment includes:"
echo "  ✅ PostgreSQL (Fly Postgres)"
echo "  ✅ Redis (Fly Redis)"
echo "  ✅ Core chat functionality"
echo "  ❌ ClickHouse (disabled)"
echo "  ❌ NATS (disabled)"  
echo "  ❌ Meilisearch (disabled)"
echo ""

# Check prerequisites
if ! command -v flyctl &> /dev/null; then
    print_error "flyctl not installed"
    echo "Install with: curl -L https://fly.io/install.sh | sh"
    exit 1
fi

if ! flyctl auth whoami &> /dev/null; then
    print_error "Not logged in to Fly.io"
    echo "Please run: flyctl auth login"
    exit 1
fi

# Check binaries
print_step "Checking binaries..."
if [ ! -f "target/main/release/fechatter_server" ]; then
    print_error "fechatter_server binary not found"
    print_info "Run: ./build-cross.sh --profile release"
    exit 1
fi

# Create simplified configs
print_step "Creating simplified configuration..."

mkdir -p fly/config-simple

# Simplified chat config (core features only)
cat > fly/config-simple/chat.yml << 'EOF'
server:
  port: 6688
  db_url: ${DATABASE_URL}
  base_dir: /data/uploads
  max_upload_size: 10485760
  request_timeout_ms: 30000
  analytics:
    enabled: false  # Disabled for simplified deployment

auth:
  pk: ${JWT_SECRET}
  sk: ${JWT_SECRET}
  token_expiration: 86400
  refresh_token_expiration: 604800

features:
  cache:
    enabled: true
    redis_url: ${REDIS_URL}
    ttl: 3600
    
  search:
    enabled: false  # Disabled (no Meilisearch)
    
  messaging:
    enabled: false  # Disabled (no NATS)
    
  message_service:
    enabled: true
    max_message_length: 5000
    
  notifications:
    enabled: false  # Disabled (no NATS)
    
  observability:
    log_level: info
    metrics:
      enabled: true
      port: 9091
      
  rate_limiting:
    enabled: true
    requests_per_minute: 60
EOF

# Single-service Dockerfile
cat > Dockerfile.fly.simple << 'EOF'
FROM alpine:3.19

WORKDIR /app

# Install runtime dependencies
RUN apk add --no-cache \
    ca-certificates \
    curl \
    tini \
    bash && \
    rm -rf /var/cache/apk/*

# Copy single service binary
COPY target/main/release/fechatter_server /usr/local/bin/fechatter_server
COPY migrations /app/migrations

# Copy simplified config
COPY fly/config-simple/chat.yml /app/fechatter_server/chat.yml

# Set permissions
RUN chmod +x /usr/local/bin/fechatter_server

# Create directories and user
RUN adduser -D -u 1001 appuser && \
    mkdir -p /data/uploads /app/logs && \
    chown -R appuser:appuser /app /data

USER appuser
EXPOSE 6688

HEALTHCHECK --interval=30s --timeout=10s --start-period=60s --retries=3 \
    CMD curl -f http://localhost:6688/health || exit 1

ENTRYPOINT ["/sbin/tini", "--"]
CMD ["/usr/local/bin/fechatter_server"]
EOF

# Build simplified image
print_step "Building simplified Docker image..."
docker build -f Dockerfile.fly.simple -t fechatter-simple:latest .

# Create fly.toml for single service
cat > fly.toml << EOF
app = "$APP_NAME"
primary_region = "$REGION"
kill_signal = "SIGTERM"
kill_timeout = 30

[env]
ENVIRONMENT = "production"
RUST_LOG = "info,sqlx=warn"

[http_service]
internal_port = 6688
force_https = true
auto_stop_machines = true
auto_start_machines = true
min_machines_running = 0

[http_service.concurrency]
type = "connections"
hard_limit = 1000
soft_limit = 800

[[http_service.checks]]
grace_period = "30s"
interval = "30s"
method = "GET"
path = "/health"
protocol = "http"
timeout = "10s"

[processes]
app = "/usr/local/bin/fechatter_server"

[[mounts]]
source = "fechatter_data"
destination = "/data"

[[vm]]
cpu_kind = "shared"
cpus = 1
memory_mb = 512

[metrics]
port = 9091
path = "/metrics"
EOF

# Setup Fly app
print_step "Setting up Fly.io application..."

if ! flyctl apps list | grep -q "$APP_NAME"; then
    print_info "Creating app $APP_NAME"
    flyctl apps create "$APP_NAME" --org "$ORG"
fi

# Create volume
if ! flyctl volumes list -a "$APP_NAME" | grep -q "fechatter_data"; then
    print_info "Creating data volume..."
    flyctl volumes create fechatter_data \
        --region "$REGION" \
        --size 1 \
        --app "$APP_NAME"
fi

# Setup Fly Postgres
print_step "Setting up Fly PostgreSQL..."
if ! flyctl postgres list | grep -q "$APP_NAME-db"; then
    flyctl postgres create \
        --name "$APP_NAME-db" \
        --region "$REGION" \
        --initial-cluster-size 1 \
        --vm-size shared-cpu-1x \
        --volume-size 1
    
    flyctl postgres attach "$APP_NAME-db" --app "$APP_NAME"
fi

# Setup Fly Redis  
print_step "Setting up Fly Redis..."
if ! flyctl redis list | grep -q "$APP_NAME-redis"; then
    flyctl redis create \
        --name "$APP_NAME-redis" \
        --region "$REGION" \
        --plan free
        
    flyctl redis attach "$APP_NAME-redis" --app "$APP_NAME"
fi

# Set secrets
print_step "Setting application secrets..."
flyctl secrets set \
    JWT_SECRET="demo-jwt-secret-$(date +%s)" \
    RUST_LOG="info,sqlx=warn" \
    --app "$APP_NAME"

# Deploy
print_step "Deploying simplified version..."
flyctl deploy --app "$APP_NAME" --local-only --image fechatter-simple:latest

# Check status
flyctl status --app "$APP_NAME"

# Get app info
APP_URL=$(flyctl info --app "$APP_NAME" | grep "Hostname" | awk '{print $2}')

if [ -n "$APP_URL" ]; then
    echo ""
    print_info "🎉 Simplified deployment successful!"
    echo ""
    echo "📋 Deployment Information:"
    echo "  App: $APP_NAME"
    echo "  URL: https://$APP_URL"
    echo "  Services: PostgreSQL + Redis + Core Chat"
    echo ""
    echo "🔗 Available endpoints:"
    echo "  Health: https://$APP_URL/health"
    echo "  API: https://$APP_URL/api/"
    echo ""
    echo "💰 Cost: ~$0/month (auto-sleep enabled)"
fi 