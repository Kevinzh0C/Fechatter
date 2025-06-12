#!/bin/bash

# ============================================================================
# x86_64 Cross-compilation One-Click Deployment Script
# ============================================================================
#
# 🎯 PURPOSE: Complete build and deployment pipeline for x86_64 architecture
# 🔧 STRATEGY: Cross-compile → Docker build → Service deployment
# 🚀 USAGE: ./deploy-x86.sh [options]
#
# ============================================================================

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Default configuration
PROFILE="release"
CLEAN_BUILD=false
DEPLOYMENT_MODE="core"
SKIP_BUILD=false
VERBOSE=false

# Logging functions
log() {
    echo -e "${BLUE}[$(date +'%Y-%m-%d %H:%M:%S')]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1" >&2
}

success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

info() {
    echo -e "${CYAN}[INFO]${NC} $1"
}

# Show usage information
show_help() {
    cat << EOF
x86_64 Cross-compilation Deployment Script

USAGE:
    $0 [OPTIONS]

OPTIONS:
    -p, --profile PROFILE      Build profile (release, debug) [default: release]
    -m, --mode MODE           Deployment mode (infrastructure, core, full, allinone) [default: core]
    -c, --clean               Clean build artifacts before compilation
    -s, --skip-build          Skip compilation step (use existing binaries)
    -v, --verbose             Enable verbose output
    -h, --help                Show this help message

DEPLOYMENT MODES:
    infrastructure            Start only infrastructure services (postgres, redis, etc.)
    core                      Start core application services
    full                      Start full stack including bot service
    allinone                  Start all services in a single container

EXAMPLES:
    $0                        # Build and deploy core services
    $0 -m full -c             # Clean build and deploy full stack
    $0 -m allinone -p debug   # Deploy all-in-one with debug profile
    $0 -s -m infrastructure   # Skip build, start only infrastructure

WORKFLOW:
    1. Cross-compile Rust binaries for x86_64
    2. Build Docker images with pre-compiled binaries  
    3. Deploy services using docker compose
    4. Verify service health and connectivity

EOF
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -p|--profile)
            PROFILE="$2"
            shift 2
            ;;
        -m|--mode)
            DEPLOYMENT_MODE="$2"
            shift 2
            ;;
        -c|--clean)
            CLEAN_BUILD=true
            shift
            ;;
        -s|--skip-build)
            SKIP_BUILD=true
            shift
            ;;
        -v|--verbose)
            VERBOSE=true
            shift
            ;;
        -h|--help)
            show_help
            exit 0
            ;;
        *)
            error "Unknown option: $1"
            show_help
            exit 1
            ;;
    esac
done

# Enable verbose output if requested
if [ "$VERBOSE" = true ]; then
    set -x
    export RUST_LOG=debug
fi

# Validate profile
case $PROFILE in
    release|debug) ;;
    *)
        error "Invalid profile: $PROFILE. Must be 'release' or 'debug'"
        exit 1
        ;;
esac

# Validate deployment mode  
case $DEPLOYMENT_MODE in
    infrastructure|core|full|allinone) ;;
    *)
        error "Invalid deployment mode: $DEPLOYMENT_MODE"
        show_help
        exit 1
        ;;
esac

# Script start
START_TIME=$(date +%s)
log "🚀 Starting x86_64 cross-compilation deployment"
echo "=================================================="
echo "Profile: $PROFILE"
echo "Mode: $DEPLOYMENT_MODE"
echo "Clean Build: $CLEAN_BUILD"
echo "Skip Build: $SKIP_BUILD"
echo "=================================================="

# Step 1: Environment setup
log "🔧 Setting up environment..."

# Check for .env file and create if needed
if [ ! -f ".env" ]; then
    if [ -f "env.x86.template" ]; then
        log "Creating .env file from template..."
        cp env.x86.template .env
        info "Please review and customize .env file before proceeding"
    else
        warn "No .env file or template found. Using default environment."
    fi
fi

# Load environment variables
if [ -f ".env" ]; then
    export $(grep -v '^#' .env | xargs)
fi

# Pre-flight checks
log "🔍 Running pre-flight checks..."

# Check required tools
REQUIRED_TOOLS=("cargo" "docker" "docker-compose")
for tool in "${REQUIRED_TOOLS[@]}"; do
    if ! command -v "$tool" &> /dev/null; then
        error "$tool is not installed or not in PATH"
        exit 1
    fi
done

# Check cross tool if not skipping build
if [ "$SKIP_BUILD" = false ] && ! command -v cross &> /dev/null; then
    log "Installing cross tool..."
    cargo install cross --git https://github.com/cross-rs/cross
fi

# Step 2: Cross-compilation (if not skipped)
if [ "$SKIP_BUILD" = false ]; then
    log "🔨 Starting cross-compilation for x86_64..."
    
    BUILD_ARGS="--profile $PROFILE"
    if [ "$CLEAN_BUILD" = true ]; then
        BUILD_ARGS="$BUILD_ARGS --clean"
    fi
    
    if ! ./build-cross.sh $BUILD_ARGS; then
        error "Cross-compilation failed"
        exit 1
    fi
    
    success "Cross-compilation completed successfully"
else
    log "⏭️  Skipping build step - using existing binaries"
    
    # Verify binaries exist
    if [ ! -d "target/main/$PROFILE" ] || [ -z "$(ls -A target/main/$PROFILE 2>/dev/null)" ]; then
        error "No pre-compiled binaries found in target/main/$PROFILE/"
        error "Run without --skip-build or compile binaries first"
        exit 1
    fi
fi

# Step 3: Docker image building
log "🐳 Building Docker images..."

COMPOSE_FILE="docker-compose.local.yml"
COMPOSE_ARGS=""

# Set appropriate environment for allinone mode
if [ "$DEPLOYMENT_MODE" = "allinone" ]; then
    export BOT_ENABLED=true
    export GATEWAY_ENABLED=true
    export RAG_ENABLED=true
fi

if ! docker compose -f "$COMPOSE_FILE" build; then
    error "Docker image building failed"
    exit 1
fi

success "Docker images built successfully"

# Step 4: Service deployment
log "🚀 Deploying services in $DEPLOYMENT_MODE mode..."

# Stop existing services first
log "Stopping existing services..."
docker compose -f "$COMPOSE_FILE" down --remove-orphans

# Start services based on deployment mode
case $DEPLOYMENT_MODE in
    infrastructure)
        COMPOSE_ARGS="--profile infrastructure"
        ;;
    core)
        COMPOSE_ARGS="--profile infrastructure --profile core"
        ;;
    full)
        COMPOSE_ARGS="--profile infrastructure --profile full"
        ;;
    allinone)
        COMPOSE_ARGS="--profile infrastructure --profile allinone"
        ;;
esac

if ! docker compose -f "$COMPOSE_FILE" $COMPOSE_ARGS up -d; then
    error "Service deployment failed"
    exit 1
fi

# Step 5: Health checks and verification
log "🏥 Performing health checks..."

# Wait for services to start
sleep 10

# Check service health based on deployment mode
HEALTH_CHECK_FAILED=false

case $DEPLOYMENT_MODE in
    infrastructure)
        info "Checking infrastructure services..."
        # Add specific infrastructure health checks if needed
        ;;
    core|full)
        info "Checking application services..."
        
        # Check main services
        SERVICES=("http://localhost:6688/health" "http://localhost:6690/health" "http://localhost:6687/health")
        
        if [ "$DEPLOYMENT_MODE" = "full" ]; then
            SERVICES+=("http://localhost:6686/health")
        fi
        
        for service in "${SERVICES[@]}"; do
            if ! curl -f -s "$service" >/dev/null 2>&1; then
                error "Health check failed for $service"
                HEALTH_CHECK_FAILED=true
            else
                success "✅ $service is healthy"
            fi
        done
        ;;
    allinone)
        info "Checking all-in-one service..."
        
        # Wait longer for all-in-one to fully start
        sleep 20
        
        ALL_IN_ONE_SERVICES=("http://localhost:6688/health" "http://localhost:6690/health" "http://localhost:6687/health")
        
        for service in "${ALL_IN_ONE_SERVICES[@]}"; do
            if ! curl -f -s "$service" >/dev/null 2>&1; then
                error "Health check failed for $service"
                HEALTH_CHECK_FAILED=true
            else
                success "✅ $service is healthy"
            fi
        done
        ;;
esac

# Final status
END_TIME=$(date +%s)
DEPLOY_DURATION=$((END_TIME - START_TIME))

echo ""
echo "=================================================="
echo "📊 DEPLOYMENT SUMMARY"
echo "=================================================="
echo "Profile: $PROFILE"
echo "Mode: $DEPLOYMENT_MODE"
echo "Duration: ${DEPLOY_DURATION}s"

if [ "$HEALTH_CHECK_FAILED" = true ]; then
    error "Some health checks failed"
    echo ""
    echo "🔧 Troubleshooting steps:"
    echo "1. Check service logs: docker compose -f $COMPOSE_FILE logs -f"
    echo "2. Check container status: docker compose -f $COMPOSE_FILE ps"
    echo "3. Restart failed services: docker compose -f $COMPOSE_FILE restart <service>"
    exit 1
else
    success "🎉 Deployment completed successfully!"
    echo ""
    echo "🌐 Service URLs:"
    
    case $DEPLOYMENT_MODE in
        infrastructure)
            echo "   PostgreSQL: postgresql://fechatter:fechatter_password@localhost:5432/fechatter"
            echo "   Redis: redis://:fechatter_redis_pass@localhost:6379"
            echo "   NATS: nats://localhost:4222"
            ;;
        core|full|allinone)
            echo "   Main Server: http://localhost:6688"
            echo "   Analytics: http://localhost:6690"
            echo "   Notifications: http://localhost:6687"
            
            if [ "$DEPLOYMENT_MODE" = "full" ]; then
                echo "   Bot Service: http://localhost:6686"
            fi
            
            if [ "$DEPLOYMENT_MODE" = "allinone" ] && [ "${GATEWAY_ENABLED:-false}" = "true" ]; then
                echo "   Gateway: http://localhost:8080"
            fi
            
            if [ "$DEPLOYMENT_MODE" = "allinone" ] && [ "${BOT_ENABLED:-false}" = "true" ]; then
                echo "   Bot Service: http://localhost:6686"
            fi
            ;;
    esac
    
    echo ""
    echo "📝 Management commands:"
    echo "   View logs: docker compose -f $COMPOSE_FILE logs -f"
    echo "   Stop services: docker compose -f $COMPOSE_FILE down"
    echo "   Restart: docker compose -f $COMPOSE_FILE restart"
fi 