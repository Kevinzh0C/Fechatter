#!/bin/bash
# deploy-production-fast.sh - Fast deployment using pre-compiled binaries

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
APP_NAME="${FLY_APP_NAME:-fechatter-prod}"
REGION="${FLY_REGION:-nrt}"
ORG="${FLY_ORG:-personal}"

# Check prerequisites
print_step "Checking prerequisites..."

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

# Check for pre-compiled binaries
print_step "Checking for pre-compiled x86_64 binaries..."

BINARY_DIR="target/main/release"
REQUIRED_BINARIES=(
    "fechatter_server"
    "fechatter_gateway"
    "notify_server"
    "analytics_server"
    "bot_server"
)

missing_binaries=()
for binary in "${REQUIRED_BINARIES[@]}"; do
    if [ ! -f "$BINARY_DIR/$binary" ]; then
        missing_binaries+=("$binary")
    fi
done

if [ ${#missing_binaries[@]} -gt 0 ]; then
    print_error "Missing pre-compiled binaries: ${missing_binaries[*]}"
    print_info "Please run: ./build-cross.sh --profile release"
    exit 1
fi

print_info "✅ All binaries found"

# Check for configuration files
print_step "Checking configuration files..."

CONFIG_FILES=(
    "fechatter_server/chat.yml"
    "analytics_server/analytics.yml" 
    "notify_server/notify.yml"
    "bot_server/bot.yml"
    "fechatter_gateway/gateway.yml"
)

for config_file in "${CONFIG_FILES[@]}"; do
    if [ ! -f "$config_file" ]; then
        print_error "Missing config file: $config_file"
        exit 1
    fi
done

print_info "✅ All configuration files found"

# Build Docker image locally
print_step "Building Docker image with pre-compiled binaries..."

docker build -f Dockerfile.fly -t fechatter-fly:latest .

# Create fly.toml
print_step "Creating fly.toml configuration..."

cat > fly.toml << EOF
# Fly.io Production Configuration - Pre-built Binaries
app = "$APP_NAME"
primary_region = "$REGION"
kill_signal = "SIGTERM"
kill_timeout = 30

# No build section - using pre-built image
[env]
ENVIRONMENT = "production"
RUST_LOG = "info,sqlx=warn"
BOT_ENABLED = "true"

[http_service]
internal_port = 8080
force_https = true
auto_stop_machines = false
auto_start_machines = true
min_machines_running = 1

[http_service.concurrency]
type = "connections"
hard_limit = 2000
soft_limit = 1500

[[http_service.checks]]
grace_period = "30s"
interval = "30s"
method = "GET"
path = "/health"
protocol = "http"
timeout = "10s"

# Additional service ports (same as local)
[[services]]
protocol = "tcp"
internal_port = 6688
processes = ["app"]

[[services.ports]]
port = 6688
handlers = ["tls", "http"]

[[services]]
protocol = "tcp"
internal_port = 6687
processes = ["app"]

[[services.ports]]
port = 6687
handlers = ["tls", "http"]

[[services]]
protocol = "tcp"
internal_port = 6686
processes = ["app"]

[[services.ports]]
port = 6686
handlers = ["tls", "http"]

[[services]]
protocol = "tcp"
internal_port = 6690
processes = ["app"]

[[services.ports]]
port = 6690
handlers = ["tls", "http"]

[processes]
app = "/usr/bin/supervisord -c /etc/supervisor/conf.d/fechatter.conf"

[[mounts]]
source = "fechatter_data"
destination = "/data"

[[vm]]
cpu_kind = "shared"
cpus = 2
memory_mb = 2048

[metrics]
port = 9091
path = "/metrics"

[deploy]
release_command = "/app/wait-for-services.sh"

EOF

# Setup Fly app
print_step "Setting up Fly.io application..."

if flyctl apps list | grep -q "$APP_NAME"; then
    print_info "App $APP_NAME exists"
else
    print_info "Creating app $APP_NAME"
    flyctl apps create "$APP_NAME" --org "$ORG"
fi

# Create volume if needed
if ! flyctl volumes list -a "$APP_NAME" | grep -q "fechatter_data"; then
    print_info "Creating data volume..."
    flyctl volumes create fechatter_data \
        --region "$REGION" \
        --size 10 \
        --app "$APP_NAME"
fi

# Setup services
print_step "Setting up external services..."

# PostgreSQL
if [ "$USE_FLY_POSTGRES" = "true" ]; then
    print_info "Setting up Fly PostgreSQL..."
    if ! flyctl postgres list | grep -q "$APP_NAME-db"; then
        flyctl postgres create \
            --name "$APP_NAME-db" \
            --region "$REGION" \
            --initial-cluster-size 1 \
            --vm-size shared-cpu-1x \
            --volume-size 10
        
        flyctl postgres attach "$APP_NAME-db" --app "$APP_NAME"
    fi
else
    print_warn "Using external PostgreSQL"
fi

# Redis (using Upstash)
if [ "$USE_UPSTASH_REDIS" = "true" ]; then
    print_info "Setting up Upstash Redis..."
    flyctl redis create \
        --name "$APP_NAME-redis" \
        --region "$REGION" \
        --plan free
fi

# Load environment if exists
if [ -f ".env.production" ]; then
    print_info "Loading .env.production"
    source .env.production
fi

# Set minimal secrets (only what's needed, not config files)
print_step "Setting application secrets..."

# JWT secret
flyctl secrets set \
    JWT_SECRET="${JWT_SECRET:-$(openssl rand -base64 32)}" \
    RUST_LOG="info,sqlx=warn" \
    --app "$APP_NAME"

# Database URL (if using external)
if [ -n "$DATABASE_URL" ] && [ "$USE_FLY_POSTGRES" != "true" ]; then
    flyctl secrets set DATABASE_URL="$DATABASE_URL" --app "$APP_NAME"
fi

# Redis URL (if using external)
if [ -n "$REDIS_URL" ] && [ "$USE_UPSTASH_REDIS" != "true" ]; then
    flyctl secrets set REDIS_URL="$REDIS_URL" --app "$APP_NAME"
fi

# Other service secrets
[ -n "$NATS_URL" ] && flyctl secrets set NATS_URL="$NATS_URL" --app "$APP_NAME"
[ -n "$OPENAI_API_KEY" ] && flyctl secrets set OPENAI_API_KEY="$OPENAI_API_KEY" --app "$APP_NAME"

# ClickHouse (if using)
if [ -n "$CLICKHOUSE_HOST" ]; then
    flyctl secrets set \
        CLICKHOUSE_HOST="$CLICKHOUSE_HOST" \
        CLICKHOUSE_PORT="${CLICKHOUSE_PORT:-8123}" \
        CLICKHOUSE_USER="${CLICKHOUSE_USER:-default}" \
        CLICKHOUSE_PASSWORD="$CLICKHOUSE_PASSWORD" \
        --app "$APP_NAME"
fi

# Meilisearch (if using)
if [ -n "$MEILISEARCH_URL" ]; then
    flyctl secrets set \
        MEILISEARCH_URL="$MEILISEARCH_URL" \
        MEILISEARCH_KEY="$MEILISEARCH_KEY" \
        --app "$APP_NAME"
fi

# Deploy using local Docker image
print_step "Deploying with pre-built image..."

flyctl deploy --app "$APP_NAME" --local-only --image fechatter-fly:latest

# Check status
print_step "Checking deployment..."

flyctl status --app "$APP_NAME"

# Get app info
APP_URL=$(flyctl info --app "$APP_NAME" | grep "Hostname" | awk '{print $2}')

if [ -n "$APP_URL" ]; then
    echo ""
    print_info "🎉 Fast deployment successful!"
    echo ""
    echo "📋 Deployment Information:"
    echo "  App: $APP_NAME"
    echo "  URL: https://$APP_URL"
    echo "  Region: $REGION"
    echo ""
    echo "⚡ Performance:"
    echo "  - Used pre-compiled x86_64 binaries from target/main/release"
    echo "  - Configuration from yml files (same as local)"
    echo "  - Docker image built locally"
    echo ""
    echo "🛠️ Commands:"
    echo "  Logs: flyctl logs -a $APP_NAME"
    echo "  SSH: flyctl ssh console -a $APP_NAME" 
    echo "  Scale: flyctl scale count 2 -a $APP_NAME"
fi 