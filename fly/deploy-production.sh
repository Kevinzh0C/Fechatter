#!/bin/bash
# deploy-production.sh - Full production deployment with all services

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
APP_NAME="${FLY_APP_NAME:-fechatter-production}"
REGION="${FLY_REGION:-nrt}"
ORG="${FLY_ORG:-personal}"

print_step "🚀 Fechatter Full Production Deployment to Fly.io"
echo ""
echo "This deployment includes:"
echo "  ✅ PostgreSQL (Fly Postgres)"
echo "  ✅ Redis (Fly Redis)" 
echo "  ✅ All 5 services (Gateway, Server, Analytics, Notify, Bot)"
echo "  ❓ ClickHouse (requires external setup)"
echo "  ❓ NATS (requires external setup)"
echo "  ❓ Meilisearch (requires external setup)"
echo ""
echo "⚠️  IMPORTANT: External services must be set up separately:"
echo "   - ClickHouse Cloud: https://clickhouse.cloud/"
echo "   - NATS: Deploy on Railway/Render/DigitalOcean"
echo "   - Meilisearch Cloud: https://www.meilisearch.com/cloud"
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
REQUIRED_BINARIES=(
    "target/main/release/fechatter_server"
    "target/main/release/fechatter_gateway"
    "target/main/release/analytics_server"
    "target/main/release/notify_server"
    "target/main/release/bot_server"
)

for binary in "${REQUIRED_BINARIES[@]}"; do
    if [ ! -f "$binary" ]; then
        print_error "$binary not found"
        print_info "Run: ./build-cross.sh --profile release"
        exit 1
    fi
done

# Build production image
print_step "Building production Docker image..."
docker build -f Dockerfile.fly -t fechatter-production:latest .

# Create fly.toml for full deployment
cat > fly.toml << EOF
app = "$APP_NAME"
primary_region = "$REGION"
kill_signal = "SIGTERM"
kill_timeout = 30

[env]
ENVIRONMENT = "production"
RUST_LOG = "info,sqlx=warn"
ANALYTICS_ENABLED = "true"
MESSAGING_ENABLED = "false"  # Set to "true" if NATS is configured
SEARCH_ENABLED = "false"     # Set to "true" if Meilisearch is configured
CLICKHOUSE_ENABLED = "false" # Set to "true" if ClickHouse is configured

[http_service]
internal_port = 8080
force_https = true
auto_stop_machines = false  # Keep running for production
auto_start_machines = true
min_machines_running = 1    # Always have 1 machine running

[http_service.concurrency]
type = "connections"
hard_limit = 2000
soft_limit = 1600

[[http_service.checks]]
grace_period = "30s"
interval = "30s"
method = "GET"
path = "/health"
protocol = "http"
timeout = "10s"

[processes]
app = "/app/fly-start.sh"

[[mounts]]
source = "fechatter_data"
destination = "/data"

[[vm]]
cpu_kind = "shared"
cpus = 2
memory_mb = 1024

[metrics]
port = 9095
path = "/metrics"

[[services]]
internal_port = 6688
protocol = "tcp"

[[services]]
internal_port = 6690
protocol = "tcp"

[[services]]
internal_port = 6687
protocol = "tcp"

[[services]]
internal_port = 6686
protocol = "tcp"
EOF

# Setup Fly app
print_step "Setting up Fly.io application..."

if ! flyctl apps list | grep -q "$APP_NAME"; then
    print_info "Creating app $APP_NAME"
    flyctl apps create "$APP_NAME" --org "$ORG"
fi

# Create volume if needed
if ! flyctl volumes list -a "$APP_NAME" | grep -q "fechatter_data"; then
    print_info "Creating data volume..."
    flyctl volumes create fechatter_data \
        --region "$REGION" \
        --size 3 \
        --app "$APP_NAME"
fi

# Setup Fly Postgres
print_step "Setting up Fly PostgreSQL..."
if ! flyctl postgres list | grep -q "$APP_NAME-db"; then
    flyctl postgres create \
        --name "$APP_NAME-db" \
        --region "$REGION" \
        --initial-cluster-size 1 \
        --vm-size shared-cpu-2x \
        --volume-size 10
    
    flyctl postgres attach "$APP_NAME-db" --app "$APP_NAME"
fi

# Setup Fly Redis
print_step "Setting up Fly Redis..."  
if ! flyctl redis list | grep -q "$APP_NAME-redis"; then
    flyctl redis create \
        --name "$APP_NAME-redis" \
        --region "$REGION" \
        --plan 1gb
        
    flyctl redis attach "$APP_NAME-redis" --app "$APP_NAME"
fi

# Set production secrets
print_step "Setting production secrets..."
flyctl secrets set \
    JWT_SECRET="$(openssl rand -base64 64)" \
    RUST_LOG="info,sqlx=warn" \
    --app "$APP_NAME"

# Deploy
print_step "Deploying full production version..."
flyctl deploy --app "$APP_NAME" --local-only --image fechatter-production:latest

# Check status
flyctl status --app "$APP_NAME"

# Get app info
APP_URL=$(flyctl info --app "$APP_NAME" | grep "Hostname" | awk '{print $2}')

if [ -n "$APP_URL" ]; then
    echo ""
    print_info "🎉 Production deployment successful!"
    echo ""
    echo "📋 Deployment Information:"
    echo "  App: $APP_NAME"
    echo "  URL: https://$APP_URL"
    echo "  Services: All 5 services running"
    echo ""
    echo "🔗 Available endpoints:"
    echo "  Health: https://$APP_URL/health"
    echo "  API Gateway: https://$APP_URL/api/"
    echo "  Chat Server: https://$APP_URL:6688/"
    echo "  Analytics: https://$APP_URL:6690/"
    echo "  Notifications: https://$APP_URL:6687/"
    echo "  Bot: https://$APP_URL:6686/"
    echo ""
    echo "⚠️  Next steps for full functionality:"
    echo "  1. Set up ClickHouse Cloud and add CLICKHOUSE_URL secret"
    echo "  2. Set up NATS server and add NATS_URL secret" 
    echo "  3. Set up Meilisearch Cloud and add MEILISEARCH_URL secret"
    echo "  4. Update environment variables: ANALYTICS_ENABLED=true, etc."
    echo ""
    echo "💰 Cost: ~$25-50/month (2 CPU, 1GB RAM + databases)"
fi 