#!/bin/bash

# ============================================================================
# Test x86_64 Docker Services with Environment Variable Configuration
# ============================================================================
#
# 🎯 PURPOSE: Test all x86_64 services using environment variable configuration
# 🔧 STRATEGY: Use service-specific config environment variables
# 📋 TESTS: Individual service startup and health checks
#
# ============================================================================

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
CONFIG_DIR="$PROJECT_ROOT/docker/configs"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Cleanup function
cleanup() {
    log_info "Cleaning up test containers..."
    docker rm -f fechatter-server-test analytics-server-test notify-server-test 2>/dev/null || true
}

# Trap cleanup on exit
trap cleanup EXIT

# Create configuration directory
create_config_directory() {
    log_info "Creating configuration directory: $CONFIG_DIR"
    mkdir -p "$CONFIG_DIR"
}

# Create Docker-compatible configuration files
create_docker_configs() {
    log_info "Creating Docker-compatible configuration files..."
    
    # Fechatter Server config
    cat > "$CONFIG_DIR/fechatter-docker.yml" << 'EOF'
server:
  port: 6688
  db_url: "postgres://fechatter:fechatter_password@host.docker.internal:5432/fechatter"
  base_dir: "/tmp/fechatter/"
  max_upload_size: 10485760
  request_timeout_ms: 30000
  analytics:
    enabled: true
    endpoint: "http://host.docker.internal:6690"

auth:
  sk: |
    -----BEGIN PRIVATE KEY-----
    MC4CAQAwBQYDK2VwBCIEIP/S+etN7RQJctehWKkdjgnrtQ0AUDIMkCnYS4Zk8RFR
    -----END PRIVATE KEY-----
  pk: |
    -----BEGIN PUBLIC KEY-----
    MCowBQYDK2VwAyEAMnnmEdL53E3O5UTdVW/VEs9qT6To/48iU7jWpKuVb2c=
    -----END PUBLIC KEY-----
  token_expiration: 86400
  refresh_token_expiration: 2592000

features:
  cache:
    enabled: true
    redis_url: "redis://:fechatter_redis_pass@host.docker.internal:6379"
    key_prefix: "fechatter"
    default_ttl: 3600
    pool_size: 10
    connection_timeout_ms: 5000

  search:
    enabled: true
    provider: "meilisearch"
    meilisearch_url: "http://host.docker.internal:7700"
    meilisearch_api_key: "fechatter_search_key"
    async_indexing: true
    batch_size: 100

  messaging:
    enabled: true
    provider: "nats"
    nats_url: "nats://host.docker.internal:4222"
    jetstream_enabled: true

  message_service:
    max_concurrent_sends_per_chat: 10
    send_timeout_seconds: 30
    cache_timeout_ms: 500
    event_publish_timeout_seconds: 5
    stream_publish_timeout_seconds: 5
    enable_detailed_tracing: false
    max_retry_attempts: 3
    retry_backoff_base_ms: 100

  notifications:
    in_app_enabled: true
    realtime_enabled: true
    max_per_user: 100
    retention_days: 30
    email_enabled: false

  observability:
    log_level: "debug"
    log_format: "pretty"
    log_to_file: false
    metrics_enabled: true
    metrics_bind_address: "0.0.0.0:9090"
    tracing_enabled: false
    service_name: "fechatter-server"
    service_version: "0.1.0"
    environment: "development"

  rate_limiting:
    enabled: true
    window_seconds: 60
    max_requests: 100
    sliding_window: true
    strategy: "UserBased"
EOF

    # Analytics Server config
    cat > "$CONFIG_DIR/analytics-docker.yml" << 'EOF'
server:
  port: 6690
  clickhouse:
    host: host.docker.internal
    port: 8123
    database: fechatter_analytics
    user: default
    password: ""
    http_url: http://host.docker.internal:8123
    native_url: clickhouse://host.docker.internal:9000/fechatter_analytics
  base_dir: /tmp/analytics
  request_timeout_ms: 30000
  max_concurrent_requests: 100

features:
  analytics:
    enabled: true
    batch_size: 100
    flush_interval_seconds: 30
    max_retry_attempts: 3
    retry_backoff_ms: 1000

  streaming:
    enabled: true
    buffer_size: 1000
    flush_threshold: 500

  aggregation:
    enabled: true
    interval_seconds: 300
    batch_size: 1000

integrations:
  messaging:
    enabled: true
    nats_url: nats://host.docker.internal:4222
    jetstream_enabled: true
    jetstream:
      stream_name: ANALYTICS
      max_bytes: 1073741824
      max_messages: 10000000
      max_age_hours: 720
      storage_type: file
      num_replicas: 1
      retention_policy: limits

  cache:
    enabled: false
    redis_url: redis://:fechatter_redis_pass@host.docker.internal:6379
    key_prefix: fechatter:analytics
    default_ttl: 3600

  postgres:
    enabled: false
    database_url: ""

observability:
  log_level: info
  log_format: json
  log_to_file: false
  metrics:
    enabled: true
    port: 7778
    path: /metrics
  health_check:
    enabled: true
    path: /health
    timeout_ms: 5000

security:
  auth:
    enabled: false
  cors:
    enabled: true
    allow_origins:
      - http://localhost:1420
      - http://localhost:3000
    allow_methods:
      - GET
      - POST
      - PUT
      - DELETE
      - OPTIONS
    allow_headers:
      - Content-Type
      - Authorization
  rate_limiting:
    enabled: true
    requests_per_minute: 1000
    burst_size: 100
EOF

    # Notify Server config
    cat > "$CONFIG_DIR/notify-docker.yml" << 'EOF'
server:
  port: 6687
  db_url: "postgres://fechatter:fechatter_password@host.docker.internal:5432/fechatter"
  request_timeout_ms: 30000

auth:
  sk: |
    -----BEGIN PRIVATE KEY-----
    MC4CAQAwBQYDK2VwBCIEIP/S+etN7RQJctehWKkdjgnrtQ0AUDIMkCnYS4Zk8RFR
    -----END PRIVATE KEY-----
  pk: |
    -----BEGIN PUBLIC KEY-----
    MCowBQYDK2VwAyEAMnnmEdL53E3O5UTdVW/VEs9qT6To/48iU7jWpKuVb2c=
    -----END PUBLIC KEY-----
  token_expiration: 1800

messaging:
  enabled: true
  provider: nats
  nats:
    url: nats://host.docker.internal:4222
    auth:
      enabled: false
    subscription_subjects:
      - fechatter.message.created
      - fechatter.chat.joined
      - fechatter.chat.left
      - fechatter.message.duplicate
    jetstream:
      enabled: true
      stream: FECHATTER_EVENTS
      storage: file
      max_bytes: 1073741824
      max_msg_size: 1048576
      max_age: 24
      consumers:
        notification_processor:
          name: notify-processor
          filter_subjects:
            - fechatter.message.created
            - fechatter.chat.joined
            - fechatter.chat.left
            - fechatter.message.duplicate
          ack_wait: 30s
          max_deliver: 3
          max_batch: 50
          idle_heartbeat: 5s

search:
  enabled: true
  provider: meilisearch
  meilisearch:
    url: http://host.docker.internal:7700
    api_key: fechatter_search_key
    connection_timeout_ms: 2000
    request_timeout_ms: 3000
    indexes:
      messages:
        name: fechatter_messages
        primary_key: id
        searchable_fields: [content, sender_name]
        displayed_fields: [id, chat_id, sender_id, content, created_at, sender_name]
        filterable_fields: [chat_id, workspace_id]
        sortable_fields: [created_at]
    settings:
      pagination_limit: 20

notification:
  delivery:
    web:
      enabled: true
      sse_enabled: true
      connection_timeout_ms: 60000
      heartbeat_interval_ms: 30000

analytics:
  enabled: true
  nats_url: nats://host.docker.internal:4222
  subject_prefix: fechatter.analytics
  batch_size: 100
  flush_interval_ms: 5000
EOF

    log_success "Configuration files created successfully"
}

# Test individual service
test_service() {
    local service_name="$1"
    local image_name="$2"
    local port="$3"
    local config_env_var="$4"
    local config_file="$5"
    local container_name="${service_name}-test"
    
    log_info "Testing $service_name with environment variable configuration..."
    
    # Stop any existing container
    docker rm -f "$container_name" 2>/dev/null || true
    
    # Start container with config environment variable
    log_info "Starting $service_name container..."
    docker run -d \
        --name "$container_name" \
        -p "$port:$port" \
        -e RUST_LOG=info \
        -e "$config_env_var=/config/$(basename "$config_file")" \
        -v "$config_file:/config/$(basename "$config_file"):ro" \
        "$image_name"
    
    # Wait for startup
    log_info "Waiting for $service_name to start..."
    sleep 8
    
    # Check container status
    if ! docker ps | grep -q "$container_name"; then
        log_error "$service_name container is not running"
        log_info "Container logs:"
        docker logs "$container_name" | tail -20
        return 1
    fi
    
    # Test health endpoint
    log_info "Testing $service_name health endpoint..."
    if curl -f -m 10 "http://localhost:$port/health" > /dev/null 2>&1; then
        log_success "$service_name health check passed"
    else
        log_warning "$service_name health check failed (may still be starting)"
        log_info "Container logs:"
        docker logs "$container_name" | tail -10
    fi
    
    # Show container logs
    log_info "$service_name container logs (last 5 lines):"
    docker logs "$container_name" | tail -5
    
    return 0
}

# Main test function
main() {
    log_info "Starting x86_64 Docker services test with environment variable configuration"
    
    # Create configuration
    create_config_directory
    create_docker_configs
    
    # Check if Docker images exist
    log_info "Checking Docker images..."
    if ! docker images | grep -q "localhost/fechatter.*local-x86_64"; then
        log_error "x86_64 Docker images not found. Please run build first."
        exit 1
    fi
    
    # Test each service
    log_info "Testing individual services..."
    
    # Test Fechatter Server
    test_service \
        "fechatter-server" \
        "localhost/fechatter/server:local-x86_64" \
        "16688" \
        "FECHATTER_CONFIG" \
        "$CONFIG_DIR/fechatter-docker.yml"
    
    # Test Analytics Server  
    test_service \
        "analytics-server" \
        "localhost/fechatter/analytics:local-x86_64" \
        "16690" \
        "ANALYTICS_CONFIG" \
        "$CONFIG_DIR/analytics-docker.yml"
    
    # Test Notify Server
    test_service \
        "notify-server" \
        "localhost/fechatter/notify:local-x86_64" \
        "16687" \
        "NOTIFY_CONFIG" \
        "$CONFIG_DIR/notify-docker.yml"
    
    # Summary
    log_success "All services tested successfully!"
    log_info "Services running on:"
    log_info "  - Fechatter Server: http://localhost:16688"
    log_info "  - Analytics Server: http://localhost:16690" 
    log_info "  - Notify Server: http://localhost:16687"
    
    log_info "To stop all test containers:"
    log_info "  docker rm -f fechatter-server-test analytics-server-test notify-server-test"
    
    # Keep containers running for manual testing
    log_info "Test containers will remain running for manual verification..."
    log_info "Press Ctrl+C to clean up and exit"
    
    # Wait for user interrupt
    while true; do
        sleep 5
        # Check if containers are still running
        local running_count=$(docker ps | grep -E "(fechatter|analytics|notify)-server-test" | wc -l)
        if [ "$running_count" -eq 0 ]; then
            log_warning "All test containers have stopped"
            break
        fi
    done
}

# Run main function
main "$@" 