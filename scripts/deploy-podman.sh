#!/bin/bash

# ============================================================================
# Fechatter Podman Deployment Script
# ============================================================================
#
# This script provides a complete deployment workflow using Podman
# Updated for latest architecture changes:
# - Unified analytics (NATS + Protobuf)
# - JWT authentication in backend services  
# - Latest project structure
# - Fixed Rust Edition 2024 support
# - Binary selection via docker command override
#
# Usage:
#   ./scripts/deploy-podman.sh build     # Build images only
#   ./scripts/deploy-podman.sh start     # Start services
#   ./scripts/deploy-podman.sh stop      # Stop services
#   ./scripts/deploy-podman.sh restart   # Restart all
#   ./scripts/deploy-podman.sh logs      # Show logs
#   ./scripts/deploy-podman.sh health    # Health check
# ============================================================================

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
IMAGE_NAME="localhost/fechatter:latest"
COMPOSE_FILE="docker-compose.yml"
PROJECT_NAME="fechatter"

# Print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if podman is installed
check_podman() {
    if ! command -v podman &> /dev/null; then
        print_error "Podman is not installed. Please install podman first."
        exit 1
    fi
    
    print_status "Podman version: $(podman --version)"
}

# Check if docker-compose is available (for podman-compose)
check_compose() {
    if command -v podman-compose &> /dev/null; then
        COMPOSE_CMD="podman-compose"
        print_status "Using podman-compose"
    elif command -v docker-compose &> /dev/null; then
        COMPOSE_CMD="docker-compose"
        print_status "Using docker-compose with podman backend"
        # Set podman as docker backend
        export DOCKER_HOST="unix:///run/user/$(id -u)/podman/podman.sock"
    else
        print_error "Neither podman-compose nor docker-compose found"
        exit 1
    fi
}

# Build Fechatter image
build_image() {
    print_status "Building Fechatter image..."
    
    # Check if Dockerfile exists
    if [[ ! -f "Dockerfile" ]]; then
        print_error "Dockerfile not found in current directory"
        exit 1
    fi
    
    # Build with buildah/podman
    podman build \
        --tag "${IMAGE_NAME}" \
        --progress=plain \
        .
    
    if [[ $? -eq 0 ]]; then
        print_success "Image built successfully: ${IMAGE_NAME}"
        podman images | grep fechatter
    else
        print_error "Failed to build image"
        exit 1
    fi
}

# Start infrastructure services
start_infrastructure() {
    print_status "Starting infrastructure services..."
    
    ${COMPOSE_CMD} up -d \
        postgres \
        clickhouse \
        redis \
        meilisearch \
        nats
    
    # Wait for services to be healthy
    print_status "Waiting for infrastructure services to be ready..."
    sleep 10
    
    # Check infrastructure health
    check_infrastructure_health
}

# Start application services
start_applications() {
    print_status "Starting application services..."
    
    ${COMPOSE_CMD} --profile app up -d
    
    # Wait for applications to start
    sleep 5
    
    print_status "Starting gateway..."
    ${COMPOSE_CMD} --profile gateway up -d
    
    print_success "All services started"
}

# Check infrastructure health
check_infrastructure_health() {
    print_status "Checking infrastructure health..."
    
    # PostgreSQL
    if ${COMPOSE_CMD} exec postgres pg_isready -U postgres > /dev/null 2>&1; then
        print_success "PostgreSQL is ready"
    else
        print_warning "PostgreSQL is not ready"
    fi
    
    # Redis
    if ${COMPOSE_CMD} exec redis redis-cli -a fechatter_redis_pass ping > /dev/null 2>&1; then
        print_success "Redis is ready"
    else
        print_warning "Redis is not ready"
    fi
    
    # ClickHouse
    if curl -s http://localhost:8123/ping > /dev/null 2>&1; then
        print_success "ClickHouse is ready"
    else
        print_warning "ClickHouse is not ready"
    fi
    
    # NATS
    if curl -s http://localhost:8222/healthz > /dev/null 2>&1; then
        print_success "NATS is ready"
    else
        print_warning "NATS is not ready"
    fi
    
    # Meilisearch  
    if curl -s http://localhost:7700/health > /dev/null 2>&1; then
        print_success "Meilisearch is ready"
    else
        print_warning "Meilisearch is not ready"
    fi
}

# Check application health
check_application_health() {
    print_status "Checking application health..."
    
    # Gateway
    if curl -s http://localhost:8080/health > /dev/null 2>&1; then
        print_success "Gateway is ready"
    else
        print_warning "Gateway is not ready"
    fi
    
    # Fechatter Server
    if curl -s http://localhost:6688/health > /dev/null 2>&1; then
        print_success "Fechatter Server is ready"
    else
        print_warning "Fechatter Server is not ready"
    fi
    
    # Notify Server
    if curl -s http://localhost:6687/sse/health > /dev/null 2>&1; then
        print_success "Notify Server is ready"
    else
        print_warning "Notify Server is not ready"
    fi
    
    # Analytics Server
    if curl -s http://localhost:6690/health > /dev/null 2>&1; then
        print_success "Analytics Server is ready"
    else
        print_warning "Analytics Server is not ready"
    fi
    
    # Bot Server
    if curl -s http://localhost:6686/health > /dev/null 2>&1; then
        print_success "Bot Server is ready"
    else
        print_warning "Bot Server is not ready"
    fi
}

# Show service logs
show_logs() {
    print_status "Showing service logs..."
    
    if [[ $# -eq 2 ]]; then
        # Show logs for specific service
        ${COMPOSE_CMD} logs -f "$2"
    else
        # Show logs for all services
        ${COMPOSE_CMD} logs -f
    fi
}

# Stop all services
stop_services() {
    print_status "Stopping all services..."
    
    ${COMPOSE_CMD} --profile gateway --profile app down
    
    print_success "All services stopped"
}

# Clean up (remove containers and volumes)
cleanup() {
    print_status "Cleaning up containers and volumes..."
    
    ${COMPOSE_CMD} --profile gateway --profile app down -v
    podman system prune -f
    
    print_success "Cleanup completed"
}

# Main script logic
case "${1:-help}" in
    "build")
        check_podman
        build_image
        ;;
    "start")
        check_podman
        check_compose
        
        # Check if image exists
        if ! podman images | grep -q fechatter; then
            print_warning "Fechatter image not found, building it first..."
            build_image
        fi
        
        start_infrastructure
        start_applications
        ;;
    "stop")
        check_compose
        stop_services
        ;;
    "restart")
        check_compose
        stop_services
        sleep 2
        start_infrastructure
        start_applications
        ;;
    "logs")
        check_compose
        show_logs "$@"
        ;;
    "health")
        check_infrastructure_health
        check_application_health
        ;;
    "cleanup")
        check_compose
        cleanup
        ;;
    "help"|*)
        echo "Fechatter Podman Deployment Script"
        echo ""
        echo "Usage: $0 COMMAND"
        echo ""
        echo "Commands:"
        echo "  build     Build Fechatter image"
        echo "  start     Start all services"
        echo "  stop      Stop all services"
        echo "  restart   Restart all services"
        echo "  logs      Show logs (optional: logs SERVICE_NAME)"
        echo "  health    Check service health"
        echo "  cleanup   Stop services and remove volumes"
        echo "  help      Show this help message"
        echo ""
        echo "Examples:"
        echo "  $0 build                    # Build image"
        echo "  $0 start                    # Start all services"
        echo "  $0 logs fechatter-server    # Show specific service logs"
        echo "  $0 health                   # Check all services"
        ;;
esac 