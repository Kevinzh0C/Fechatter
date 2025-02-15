#!/bin/bash
# wait-for-services.sh - Universal service readiness checker
# Works in local, Docker, and Kubernetes environments

set -e

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

print_info() { echo -e "${GREEN}[INFO]${NC} $1"; }
print_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
print_error() { echo -e "${RED}[ERROR]${NC} $1"; }

TIMEOUT=${WAIT_TIMEOUT:-300}
ENVIRONMENT_TYPE=""

# Detect environment
detect_environment() {
    if [ -f /var/run/secrets/kubernetes.io/serviceaccount/token ]; then
        ENVIRONMENT_TYPE="kubernetes"
        NAMESPACE=${POD_NAMESPACE:-default}
    elif [ -n "$DOCKER_HOST" ] || [ -S /var/run/docker.sock ]; then
        ENVIRONMENT_TYPE="docker"
    else
        ENVIRONMENT_TYPE="local"
    fi
    
    print_info "Detected environment: $ENVIRONMENT_TYPE"
}

# Wait for service based on environment
wait_for_service() {
    local service_name=$1
    local port=$2
    local path=${3:-"/"}
    local host=${4:-"localhost"}
    
    case $ENVIRONMENT_TYPE in
        kubernetes)
            wait_for_k8s_service "$service_name" "$port" "$path"
            ;;
        docker)
            wait_for_docker_service "$service_name" "$port" "$path"
            ;;
        local)
            wait_for_local_service "$service_name" "$host" "$port" "$path"
            ;;
    esac
}

# Kubernetes service wait
wait_for_k8s_service() {
    local service_name=$1
    local port=$2
    local path=$3
    local max_attempts=$((TIMEOUT / 5))
    local attempt=0
    
    print_info "Waiting for K8s service: $service_name"
    
    while [ $attempt -lt $max_attempts ]; do
        if kubectl get service "$service_name" -n "$NAMESPACE" >/dev/null 2>&1; then
            if kubectl get endpoints "$service_name" -n "$NAMESPACE" -o jsonpath='{.subsets[0].addresses[0].ip}' 2>/dev/null | grep -q .; then
                if curl -f -s "http://$service_name.$NAMESPACE.svc.cluster.local:$port$path" >/dev/null 2>&1; then
                    print_info "✅ K8s service $service_name ready"
                    return 0
                fi
            fi
        fi
        
        echo -n "."
        sleep 5
        attempt=$((attempt + 1))
    done
    
    print_error "❌ K8s service $service_name timeout"
    return 1
}

# Docker service wait
wait_for_docker_service() {
    local service_name=$1
    local port=$2
    local path=$3
    local max_attempts=$((TIMEOUT / 2))
    local attempt=0
    
    print_info "Waiting for Docker service: $service_name"
    
    while [ $attempt -lt $max_attempts ]; do
        # Try container name first, then localhost
        if curl -f -s "http://$service_name:$port$path" >/dev/null 2>&1 || \
           curl -f -s "http://localhost:$port$path" >/dev/null 2>&1; then
            print_info "✅ Docker service $service_name ready"
            return 0
        fi
        
        echo -n "."
        sleep 2
        attempt=$((attempt + 1))
    done
    
    print_error "❌ Docker service $service_name timeout"
    return 1
}

# Local service wait
wait_for_local_service() {
    local service_name=$1
    local host=$2
    local port=$3
    local path=$4
    local max_attempts=$((TIMEOUT / 2))
    local attempt=0
    
    print_info "Waiting for local service: $service_name ($host:$port)"
    
    while [ $attempt -lt $max_attempts ]; do
        if nc -z "$host" "$port" 2>/dev/null; then
            if [[ "$path" != "/" ]] && command -v curl >/dev/null; then
                if curl -f -s "http://$host:$port$path" >/dev/null 2>&1; then
                    print_info "✅ Local service $service_name ready"
                    return 0
                fi
            else
                print_info "✅ Local service $service_name ready"
                return 0
            fi
        fi
        
        echo -n "."
        sleep 2
        attempt=$((attempt + 1))
    done
    
    print_error "❌ Local service $service_name timeout"
    return 1
}

# Check PostgreSQL readiness
wait_for_postgres() {
    local host="${DB_HOST:-postgres}"
    
    case $ENVIRONMENT_TYPE in
        kubernetes)
            host="postgres.$NAMESPACE.svc.cluster.local"
            ;;
        local)
            host="${DB_HOST:-localhost}"
            ;;
    esac
    
    print_info "Waiting for PostgreSQL..."
    local max_attempts=$((TIMEOUT / 2))
    local attempt=0
    
    while [ $attempt -lt $max_attempts ]; do
        if pg_isready -h "$host" -p 5432 -U postgres >/dev/null 2>&1; then
            print_info "✅ PostgreSQL ready"
            return 0
        fi
        
        echo -n "."
        sleep 2
        attempt=$((attempt + 1))
    done
    
    print_error "❌ PostgreSQL timeout"
    return 1
}

# Check Redis readiness
wait_for_redis() {
    local host="${REDIS_HOST:-redis}"
    
    case $ENVIRONMENT_TYPE in
        kubernetes)
            host="redis.$NAMESPACE.svc.cluster.local"
            ;;
        local)
            host="${REDIS_HOST:-localhost}"
            ;;
    esac
    
    print_info "Waiting for Redis..."
    local max_attempts=$((TIMEOUT / 2))
    local attempt=0
    
    while [ $attempt -lt $max_attempts ]; do
        if redis-cli -h "$host" -p 6379 ping >/dev/null 2>&1; then
            print_info "✅ Redis ready"
            return 0
        fi
        
        echo -n "."
        sleep 2
        attempt=$((attempt + 1))
    done
    
    print_error "❌ Redis timeout"
    return 1
}

# Main execution
main() {
    print_info "🕐 Waiting for Fechatter services to be ready..."
    
    # Detect environment
    detect_environment
    
    # Wait for infrastructure services
    if [ "${WAIT_FOR_POSTGRES:-true}" = "true" ]; then
        wait_for_postgres
    fi
    
    if [ "${WAIT_FOR_REDIS:-true}" = "true" ]; then
        wait_for_redis
    fi
    
    if [ "${WAIT_FOR_NATS:-true}" = "true" ]; then
        wait_for_service "nats" 4222 "/"
        wait_for_service "nats" 8222 "/healthz"
        
        # Check JetStream
        print_info "Waiting for NATS JetStream..."
        local max_attempts=$((TIMEOUT / 2))
        local attempt=0
        local nats_host="${NATS_HOST:-nats}"
        
        case $ENVIRONMENT_TYPE in
            kubernetes)
                nats_host="nats.$NAMESPACE.svc.cluster.local"
                ;;
            local)
                nats_host="${NATS_HOST:-localhost}"
                ;;
        esac
        
        while [ $attempt -lt $max_attempts ]; do
            if curl -s "http://$nats_host:8222/varz" | grep -q '"jetstream".*true' >/dev/null 2>&1; then
                print_info "✅ NATS JetStream ready"
                break
            fi
            
            echo -n "."
            sleep 2
            attempt=$((attempt + 1))
        done
        
        if [ $attempt -ge $max_attempts ]; then
            print_error "❌ NATS JetStream timeout"
            exit 1
        fi
    fi
    
    if [ "${WAIT_FOR_MEILISEARCH:-true}" = "true" ]; then
        wait_for_service "meilisearch" 7700 "/health"
    fi
    
    if [ "${WAIT_FOR_CLICKHOUSE:-false}" = "true" ]; then
        wait_for_service "clickhouse" 8123 "/ping"
    fi
    
    print_info "🎉 All services ready!"
    
    # Execute passed command
    if [ $# -gt 0 ]; then
        exec "$@"
    fi
}

# Run main function
main "$@"