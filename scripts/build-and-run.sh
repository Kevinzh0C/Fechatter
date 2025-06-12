#!/bin/bash
# Comprehensive build and run script for Fechatter

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_step() {
    echo -e "${BLUE}[STEP]${NC} $1"
}

# Check disk space
check_disk_space() {
    print_step "Checking disk space..."
    local available=$(df -h . | tail -1 | awk '{print $4}' | sed 's/Gi//')
    
    if (( $(echo "$available < 10" | bc -l) )); then
        print_warning "Low disk space: ${available}GB available"
        print_info "Cleaning up to free space..."
        
        # Clean Podman
        podman system prune -a -f --volumes 2>/dev/null || true
        
        # Clean Cargo cache
        print_info "Cleaning Cargo cache..."
        cargo clean 2>/dev/null || true
        rm -rf ~/.cargo/registry/cache 2>/dev/null || true
        
        # Re-check
        available=$(df -h . | tail -1 | awk '{print $4}' | sed 's/Gi//')
        print_info "Available space after cleanup: ${available}GB"
    else
        print_info "Disk space OK: ${available}GB available"
    fi
}

# Start infrastructure services
start_infrastructure() {
    print_step "Starting infrastructure services..."
    
    # Check if podman machine is running
    if ! podman machine list | grep -q "Currently running"; then
        print_info "Starting Podman machine..."
        podman machine start
        sleep 5
    fi
    
    # Start only infrastructure services
    print_info "Starting PostgreSQL, Redis, NATS, Meilisearch, ClickHouse..."
    podman-compose up -d postgres redis nats meilisearch clickhouse
    
    # Wait for services to be healthy
    print_info "Waiting for services to be healthy..."
    sleep 10
    
    # Check service health
    local services_healthy=true
    
    if ! pg_isready -h localhost -p 5432 > /dev/null 2>&1; then
        print_error "PostgreSQL is not ready"
        services_healthy=false
    fi
    
    if ! redis-cli -h localhost -p 6379 -a fechatter_redis_pass ping > /dev/null 2>&1; then
        print_error "Redis is not ready"
        services_healthy=false
    fi
    
    if ! curl -s http://localhost:8222/healthz > /dev/null 2>&1; then
        print_error "NATS is not ready"
        services_healthy=false
    fi
    
    if ! curl -s http://localhost:7700/health > /dev/null 2>&1; then
        print_error "Meilisearch is not ready"
        services_healthy=false
    fi
    
    if $services_healthy; then
        print_info "All infrastructure services are healthy"
    else
        print_error "Some services failed to start. Check logs with: podman-compose logs"
        exit 1
    fi
}

# Build with Docker/Podman
build_with_docker() {
    print_step "Building with Docker/Podman..."
    
    # Use the nightly Dockerfile
    if [ -f "Dockerfile.nightly" ]; then
        print_info "Using Dockerfile.nightly for edition 2024 support..."
        podman build -f Dockerfile.nightly -t fechatter/all-services:latest .
    else
        print_error "Dockerfile.nightly not found"
        exit 1
    fi
}

# Build locally
build_locally() {
    print_step "Building locally with Cargo..."
    
    # Install Rust nightly if not available
    if ! rustup toolchain list | grep -q "nightly"; then
        print_info "Installing Rust nightly toolchain..."
        rustup toolchain install nightly
    fi
    
    # Set nightly as default for this project
    rustup override set nightly
    
    # Build all services
    print_info "Building fechatter_server..."
    cargo +nightly build --release --bin fechatter_server
    
    print_info "Building notify_server..."
    cargo +nightly build --release --bin notify_server
    
    print_info "Building analytics_server..."
    cargo +nightly build --release --bin analytics_server
    
    print_info "Building bot_server..."
    cargo +nightly build --release --bin bot_server
    
    print_info "Build completed successfully!"
}

# Main menu
show_menu() {
    echo "╔════════════════════════════════════════╗"
    echo "║      Fechatter Build & Run Script      ║"
    echo "╠════════════════════════════════════════╣"
    echo "║ 1. Check and clean disk space          ║"
    echo "║ 2. Start infrastructure services       ║"
    echo "║ 3. Build with Docker/Podman            ║"
    echo "║ 4. Build locally (recommended)         ║"
    echo "║ 5. Run services locally                ║"
    echo "║ 6. Full setup (2+4+5)                  ║"
    echo "║ 7. Exit                                ║"
    echo "╚════════════════════════════════════════╝"
}

# Run services locally
run_locally() {
    print_step "Running services locally..."
    
    # Check if binaries exist
    if [ ! -f "target/release/fechatter_server" ]; then
        print_error "Binaries not found. Please build first (option 4)"
        exit 1
    fi
    
    # Export environment variables
    export DATABASE_URL="postgres://postgres:postgres@localhost:5432/fechatter"
    export REDIS_URL="redis://:fechatter_redis_pass@localhost:6379"
    export NATS_URL="nats://localhost:4222"
    export MEILISEARCH_URL="http://localhost:7700"
    export MEILISEARCH_KEY="fechatter_meili_master_key"
    export CLICKHOUSE_URL="clickhouse://localhost:9000/fechatter_analytics"
    export JWT_SECRET="your-secret-key-here"
    export RUST_LOG="info"
    export SQLX_OFFLINE="true"
    
    print_info "Choose which service to run:"
    echo "1. fechatter_server (port 6688)"
    echo "2. notify_server (port 6687)"
    echo "3. analytics_server (port 6690)"
    echo "4. bot_server (port 6686)"
    echo "5. All services (in separate terminals)"
    
    read -p "Enter choice: " service_choice
    
    case $service_choice in
        1)
            ./target/release/fechatter_server --config ./fixtures/fechatter.yml
            ;;
        2)
            ./target/release/notify_server --config ./fixtures/notify.yml
            ;;
        3)
            ./target/release/analytics_server --config ./fixtures/analytics.yml
            ;;
        4)
            ./target/release/bot_server --config ./fixtures/bot.yml
            ;;
        5)
            print_info "Starting all services in separate terminals..."
            osascript -e 'tell app "Terminal" to do script "cd '$PWD' && ./scripts/run-local-dev.sh fechatter_server"'
            osascript -e 'tell app "Terminal" to do script "cd '$PWD' && ./scripts/run-local-dev.sh notify_server"'
            osascript -e 'tell app "Terminal" to do script "cd '$PWD' && ./scripts/run-local-dev.sh analytics_server"'
            osascript -e 'tell app "Terminal" to do script "cd '$PWD' && ./scripts/run-local-dev.sh bot_server"'
            ;;
        *)
            print_error "Invalid choice"
            ;;
    esac
}

# Main execution
main() {
    while true; do
        show_menu
        read -p "Enter your choice: " choice
        
        case $choice in
            1)
                check_disk_space
                ;;
            2)
                start_infrastructure
                ;;
            3)
                check_disk_space
                build_with_docker
                ;;
            4)
                build_locally
                ;;
            5)
                run_locally
                ;;
            6)
                check_disk_space
                start_infrastructure
                build_locally
                run_locally
                ;;
            7)
                print_info "Exiting..."
                exit 0
                ;;
            *)
                print_error "Invalid choice"
                ;;
        esac
        
        echo
        read -p "Press Enter to continue..."
        clear
    done
}

# Check if we're in the project root
if [ ! -f "Cargo.toml" ]; then
    print_error "Please run this script from the project root directory"
    exit 1
fi

# Run main function
main "$@" 