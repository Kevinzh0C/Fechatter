#!/bin/bash
# start-dev.sh - Unified Fechatter Development Environment Starter
# Supports Docker/Podman containers and native local development

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_info() { echo -e "${GREEN}[INFO]${NC} $1"; }
print_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
print_error() { echo -e "${RED}[ERROR]${NC} $1"; }
print_step() { echo -e "${BLUE}[STEP]${NC} $1"; }

# Configuration
GATEWAY_ENV=development
NODE_ENV=development
MODE="local"  # local, podman, docker

# Show help
show_help() {
    cat << EOF
🚀 Fechatter Development Environment Starter

Usage: $0 [mode] [options]

Modes:
  local       Run services natively (default)
  podman      Use Podman containers
  docker      Use Docker containers

Options:
  --infrastructure-only    Start only infrastructure services
  --no-frontend           Skip frontend startup
  --service <name>        Start only specific service
  --clean                 Clean logs and restart fresh
  --help                  Show this help

Services:
  fechatter_server        Main API server (port 6688)
  notify_server           SSE notification server (port 6687)
  analytics_server        Analytics and metrics (port 6690)
  bot_server              AI bot (NATS subscriber)
  gateway                 API gateway (port 8080)
  frontend                React frontend (port 1420)

Examples:
  $0                      # Start all services locally
  $0 podman               # Use Podman containers
  $0 --service gateway    # Start only gateway
  $0 --infrastructure-only # Start only infrastructure
EOF
}

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        local|podman|docker)
            MODE="$1"
            shift
            ;;
        --infrastructure-only)
            INFRASTRUCTURE_ONLY=true
            shift
            ;;
        --no-frontend)
            NO_FRONTEND=true
            shift
            ;;
        --service)
            SINGLE_SERVICE="$2"
            shift 2
            ;;
        --clean)
            CLEAN_START=true
            shift
            ;;
        --help)
            show_help
            exit 0
            ;;
        *)
            print_error "Unknown option: $1"
            show_help
            exit 1
            ;;
    esac
done

# Check if port is available
check_port() {
    local port=$1
    local service=$2
    
    if lsof -Pi :$port -sTCP:LISTEN -t >/dev/null 2>&1; then
        print_error "❌ Port $port is already in use (needed for $service)"
        print_info "   Use './scripts/port-manager.sh clean' to free ports"
        return 1
    else
        print_info "✅ Port $port is available for $service"
        return 0
    fi
}

# Check required ports
check_ports() {
    if [[ "$INFRASTRUCTURE_ONLY" != "true" ]]; then
        check_port 6688 "fechatter_server" || exit 1
        check_port 6687 "notify_server" || exit 1
        check_port 6690 "analytics_server" || exit 1
        check_port 8080 "gateway" || exit 1
        
        if [[ "$NO_FRONTEND" != "true" ]]; then
            check_port 1420 "frontend" || exit 1
        fi
    fi
}

# Start infrastructure with containers
start_infrastructure() {
    print_step "Starting infrastructure services with $MODE..."
    
    case $MODE in
        podman)
            # Check Podman status
            if ! podman info > /dev/null 2>&1; then
                print_error "Podman not running. Start with: podman machine start"
                exit 1
            fi
            
            print_info "Using Podman containers..."
            podman-compose up -d postgres redis nats meilisearch clickhouse
            ;;
        docker)
            print_info "Using Docker containers..."
            docker-compose up -d postgres redis nats meilisearch clickhouse
            ;;
        local)
            print_warn "Local mode: Infrastructure services should be running externally"
            print_info "If using containers, run: podman-compose up -d postgres redis nats meilisearch clickhouse"
            ;;
    esac
    
    # Wait for services
    print_info "Waiting for infrastructure to be ready..."
    ./scripts/wait-for-services.sh echo "Infrastructure ready"
}

# Start single service locally
start_service() {
    local service_name=$1
    local service_dir=$2
    local start_command=$3
    local log_file="logs/${service_name}.log"
    
    print_step "Starting $service_name..."
    
    # Create logs directory
    mkdir -p logs
    
    # Clean logs if requested
    if [[ "$CLEAN_START" == "true" ]]; then
        rm -f "$log_file"
        rm -f "logs/${service_name}.pid"
    fi
    
    # Set environment variables
    export DATABASE_URL="postgres://postgres:postgres@localhost:5432/fechatter"
    export REDIS_URL="redis://:fechatter_redis_pass@localhost:6379"
    export NATS_URL="nats://localhost:4222"
    export MEILISEARCH_URL="http://localhost:7700"
    export MEILISEARCH_KEY="fechatter_meili_master_key"
    export CLICKHOUSE_URL="clickhouse://localhost:9000/fechatter_analytics"
    export JWT_SECRET="your-secret-key-here"
    export RUST_LOG="info"
    export SQLX_OFFLINE="true"
    
    # Start service
    if [[ -n "$service_dir" ]]; then
        cd "$service_dir"
        eval "$start_command" > "../$log_file" 2>&1 &
        cd - > /dev/null
    else
        eval "$start_command" > "$log_file" 2>&1 &
    fi
    
    local pid=$!
    echo $pid > "logs/${service_name}.pid"
    
    print_info "✅ $service_name started (PID: $pid)"
    print_info "   📄 Logs: $log_file"
    
    sleep 2
}

# Wait for service to be ready
wait_for_service() {
    local service_name=$1
    local url=$2
    local max_attempts=30
    local attempt=1
    
    print_info "⏳ Waiting for $service_name to be ready..."
    
    while [ $attempt -le $max_attempts ]; do
        if curl -s "$url" > /dev/null 2>&1; then
            print_info "✅ $service_name is ready!"
            return 0
        fi
        
        echo -n "."
        sleep 1
        attempt=$((attempt + 1))
    done
    
    print_error "❌ $service_name failed to start within $max_attempts seconds"
    print_info "Check logs: tail -f logs/${service_name}.log"
    return 1
}

# Start all application services
start_all_services() {
    print_step "Starting application services..."
    
    # 1. Start fechatter_server
    start_service "fechatter_server" "" "cargo run --bin fechatter_server"
    wait_for_service "fechatter_server" "http://127.0.0.1:6688/health"
    
    # 2. Start notify_server
    start_service "notify_server" "" "cargo run --bin notify_server"
    wait_for_service "notify_server" "http://127.0.0.1:6687/sse/health"
    
    # 3. Start analytics_server
    start_service "analytics_server" "" "cargo run --bin analytics_server"
    wait_for_service "analytics_server" "http://127.0.0.1:6690/health"
    
    # 4. Start bot_server (NATS subscriber)
    print_step "Starting bot_server (NATS Event Subscriber)..."
    start_service "bot_server" "" "cargo run --bin bot_server"
    print_info "✅ bot_server started as background NATS subscriber"
    
    # 5. Start gateway
    print_info "⏳ Waiting for backend services to stabilize..."
    sleep 3
    
    start_service "gateway" "fechatter_gateway" "cargo run"
    wait_for_service "gateway" "http://127.0.0.1:8080/health"
    
    # 6. Start frontend (if not disabled)
    if [[ "$NO_FRONTEND" != "true" ]]; then
        print_step "Starting frontend..."
        
        # Check if yarn is available
        if ! command -v yarn &> /dev/null; then
            print_warn "yarn not found, using npm instead"
            start_service "frontend" "fechatter_frontend" "npm run dev"
        else
            start_service "frontend" "fechatter_frontend" "yarn dev"
        fi
        
        sleep 5
        if curl -s "http://127.0.0.1:1420" > /dev/null 2>&1; then
            print_info "✅ Frontend is ready!"
        else
            print_warn "⚠️ Frontend may still be starting up..."
        fi
    fi
}

# Start specific service
start_single_service() {
    local service="$SINGLE_SERVICE"
    
    case $service in
        fechatter_server)
            start_service "fechatter_server" "" "cargo run --bin fechatter_server"
            wait_for_service "fechatter_server" "http://127.0.0.1:6688/health"
            ;;
        notify_server)
            start_service "notify_server" "" "cargo run --bin notify_server"
            wait_for_service "notify_server" "http://127.0.0.1:6687/sse/health"
            ;;
        analytics_server)
            start_service "analytics_server" "" "cargo run --bin analytics_server"
            wait_for_service "analytics_server" "http://127.0.0.1:6690/health"
            ;;
        bot_server)
            start_service "bot_server" "" "cargo run --bin bot_server"
            print_info "✅ bot_server started as NATS subscriber"
            ;;
        gateway)
            start_service "gateway" "fechatter_gateway" "cargo run"
            wait_for_service "gateway" "http://127.0.0.1:8080/health"
            ;;
        frontend)
            if command -v yarn &> /dev/null; then
                start_service "frontend" "fechatter_frontend" "yarn dev"
            else
                start_service "frontend" "fechatter_frontend" "npm run dev"
            fi
            ;;
        *)
            print_error "Unknown service: $service"
            print_info "Available services: fechatter_server, notify_server, analytics_server, bot_server, gateway, frontend"
            exit 1
            ;;
    esac
}

# Main execution
main() {
    print_info "🚀 Fechatter Development Environment Starter"
    print_info "Mode: $MODE"
    
    # Check if in project root
    if [[ ! -f "Cargo.toml" ]]; then
        print_error "Please run from project root directory"
        exit 1
    fi
    
    # Clean logs if requested
    if [[ "$CLEAN_START" == "true" ]]; then
        print_info "🧹 Cleaning logs and PIDs..."
        rm -rf logs/
    fi
    
    # Start infrastructure
    if [[ "$MODE" != "local" ]] || [[ "$INFRASTRUCTURE_ONLY" == "true" ]]; then
        start_infrastructure
    fi
    
    # Exit if infrastructure only
    if [[ "$INFRASTRUCTURE_ONLY" == "true" ]]; then
        print_info "🎉 Infrastructure services started!"
        exit 0
    fi
    
    # Check ports
    check_ports
    
    # Start services
    if [[ -n "$SINGLE_SERVICE" ]]; then
        start_single_service
    else
        start_all_services
    fi
    
    # Show summary
    print_info ""
    print_info "🎉 Fechatter Development Environment is ready!"
    print_info "=============================================="
    print_info ""
    print_info "📋 Service URLs:"
    if [[ "$NO_FRONTEND" != "true" ]]; then
        print_info "  🌐 Frontend:         http://localhost:1420"
    fi
    print_info "  🚪 Gateway:          http://localhost:8080"
    print_info "  📡 Fechatter Server: http://localhost:6688"
    print_info "  🔔 Notify Server:    http://localhost:6687"
    print_info "  📊 Analytics Server: http://localhost:6690"
    print_info "  🔍 Search Service:   http://localhost:7700"
    print_info "  🤖 Bot Server:       NATS subscriber"
    print_info ""
    print_info "📄 Log Files:         logs/*.log"
    print_info "🛑 Stop Services:     ./scripts/stop-dev.sh"
    print_info "🔧 Port Management:   ./scripts/port-manager.sh"
    print_info ""
    print_info "💡 Architecture:"
    print_info "  Frontend(1420) → Gateway(8080) → Backend Services"
    print_info "                                  ├─ fechatter_server(6688)"
    print_info "                                  ├─ notify_server(6687)"
    print_info "                                  ├─ analytics_server(6690)"
    print_info "                                  ├─ search_service(7700)"
    print_info "                                  └─ bot_server(NATS subscriber)"
    print_info ""
    print_info "🚀 Happy coding!"
}

# Run main function
main "$@" 