#!/bin/bash
# Deploy existing image to Fly.io

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

# Configuration
APP_NAME="${FLY_APP_NAME:-fechatter-prod}"
IMAGE_TAG="${IMAGE_TAG:-x86-latest}"
CONFIG_FILE="${CONFIG_FILE:-fly-deploy-multiservice.toml}"

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

# Check if config file exists
if [ ! -f "$CONFIG_FILE" ]; then
    print_error "Config file $CONFIG_FILE not found"
    exit 1
fi

# Display deployment info
print_step "Deployment Information:"
echo "  App: $APP_NAME"
echo "  Image: registry.fly.io/$APP_NAME:$IMAGE_TAG"
echo "  Config: $CONFIG_FILE"
echo ""

# Check if image exists in registry
print_step "Verifying image in registry..."
if docker manifest inspect "registry.fly.io/$APP_NAME:$IMAGE_TAG" &> /dev/null; then
    print_info "✅ Image found in registry"
else
    print_warn "Image might not exist in registry, deployment may fail"
    print_info "To push image: docker push registry.fly.io/$APP_NAME:$IMAGE_TAG"
fi

# Set secrets if needed
print_step "Checking secrets..."

# Load environment if exists
if [ -f ".env.production" ]; then
    print_info "Loading .env.production"
    source .env.production
    
    # Set essential secrets
    flyctl secrets set \
        JWT_SECRET="${JWT_SECRET:-$(openssl rand -base64 32)}" \
        --app "$APP_NAME" --stage || true
fi

# Deploy using the existing image
print_step "Deploying with existing image..."

flyctl deploy \
    --app "$APP_NAME" \
    --config "$CONFIG_FILE" \
    --image "registry.fly.io/$APP_NAME:$IMAGE_TAG" \
    --strategy rolling

# Check deployment status
print_step "Checking deployment status..."

flyctl status --app "$APP_NAME"

# Get app info
APP_URL=$(flyctl info --app "$APP_NAME" | grep "Hostname" | awk '{print $2}')

if [ -n "$APP_URL" ]; then
    echo ""
    print_info "🎉 Deployment successful!"
    echo ""
    echo "📋 Deployment Information:"
    echo "  App: $APP_NAME"
    echo "  URL: https://$APP_URL"
    echo "  Image: registry.fly.io/$APP_NAME:$IMAGE_TAG"
    echo ""
    echo "🛠️ Commands:"
    echo "  Logs: flyctl logs -a $APP_NAME"
    echo "  SSH: flyctl ssh console -a $APP_NAME"
    echo "  Scale: flyctl scale count 2 -a $APP_NAME"
    echo ""
    echo "🔍 Troubleshooting:"
    echo "  Check logs: flyctl logs -a $APP_NAME --tail 100"
    echo "  Check services: flyctl ssh console -a $APP_NAME -C 'supervisorctl status'"
else
    print_error "Deployment might have failed. Check logs with: flyctl logs -a $APP_NAME"
fi 