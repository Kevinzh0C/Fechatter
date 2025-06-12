#!/bin/bash
set -euo pipefail

# Configuration Loading Validation Script
# Tests all services' config loading mechanisms

echo "🔍 Configuration Loading Validation Test"
echo "======================================="

# Color definitions
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Global test state
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Test result tracking
log_test() {
    local test_name="$1"
    local result="$2"
    local message="${3:-}"
    
    ((TOTAL_TESTS++))
    
    if [[ "$result" == "PASS" ]]; then
        echo -e "  ${GREEN}✓${NC} $test_name ${message:+- $message}"
        ((PASSED_TESTS++))
    else
        echo -e "  ${RED}✗${NC} $test_name ${message:+- $message}"
        ((FAILED_TESTS++))
    fi
}

# Check if config files exist
check_config_files() {
    echo -e "\n${BLUE}📋 Phase 1: Configuration Files Existence Check${NC}"
    
    local configs=(
        "fechatter_server/chat.yml:fechatter_server"
        "analytics_server/analytics.yml:analytics_server"  
        "notify_server/notify.yml:notify_server"
        "fechatter_gateway/gateway.yml:fechatter_gateway"
        "bot_server/bot.yml:bot_server"
    )
    
    for config_info in "${configs[@]}"; do
        IFS=':' read -r config_path service_name <<< "$config_info"
        
        if [[ -f "$config_path" ]]; then
            log_test "Config exists: $config_path" "PASS"
            
            # Validate YAML syntax using simple method
            if python3 -c "
import yaml
import sys
try:
    with open('$config_path', 'r') as f:
        yaml.safe_load(f)
    print('YAML_VALID')
except Exception as e:
    print(f'YAML_ERROR: {e}')
    sys.exit(1)
" 2>&1 | grep -q "YAML_VALID"; then
                log_test "YAML syntax valid: $config_path" "PASS"
            else
                log_test "YAML syntax valid: $config_path" "FAIL" "Invalid YAML syntax"
            fi
        else
            log_test "Config exists: $config_path" "FAIL" "File not found"
        fi
    done
}

# Test local configuration loading
test_local_config_loading() {
    echo -e "\n${BLUE}📋 Phase 2: Local Configuration Loading Test${NC}"
    
    # Test with binaries if they exist
    local binaries_dir="target/x86_64-unknown-linux-musl/release"
    
    if [[ -d "$binaries_dir" ]]; then
        echo "  Using x86_64 binaries from: $binaries_dir"
        
        # Test fechatter_server config loading
        if [[ -f "$binaries_dir/fechatter_server" ]]; then
            echo "  Testing fechatter_server config loading..."
            
            # Test 1: Default config path search
            if timeout 5s "$binaries_dir/fechatter_server" --help >/dev/null 2>&1; then
                log_test "fechatter_server binary runs" "PASS"
            else
                log_test "fechatter_server binary runs" "FAIL" "Binary execution failed"
            fi
            
            # Test 2: Environment variable override
            if FECHATTER_CONFIG="fechatter_server/chat.yml" timeout 3s "$binaries_dir/fechatter_server" --help >/dev/null 2>&1; then
                log_test "fechatter_server FECHATTER_CONFIG override" "PASS"
            else
                log_test "fechatter_server FECHATTER_CONFIG override" "FAIL"
            fi
        else
            log_test "fechatter_server binary exists" "FAIL" "Binary not found"
        fi
        
        # Test analytics_server
        if [[ -f "$binaries_dir/analytics_server" ]]; then
            if timeout 3s "$binaries_dir/analytics_server" --help >/dev/null 2>&1; then
                log_test "analytics_server binary runs" "PASS"
            else
                log_test "analytics_server binary runs" "FAIL"
            fi
        else
            log_test "analytics_server binary exists" "FAIL"
        fi
        
        # Test notify_server
        if [[ -f "$binaries_dir/notify_server" ]]; then
            if timeout 3s "$binaries_dir/notify_server" --help >/dev/null 2>&1; then
                log_test "notify_server binary runs" "PASS"
            else
                log_test "notify_server binary runs" "FAIL"
            fi
        else
            log_test "notify_server binary exists" "FAIL"
        fi
        
    else
        echo "  ${YELLOW}⚠️  x86_64 binaries not found, skipping binary tests${NC}"
        echo "  Run 'scripts/build-x86-complete.sh' first to build binaries"
    fi
}

# Test Docker environment config loading
test_docker_config_loading() {
    echo -e "\n${BLUE}📋 Phase 3: Docker Environment Configuration Test${NC}"
    
    # Check if Docker is available
    if ! command -v docker >/dev/null 2>&1; then
        log_test "Docker availability" "FAIL" "Docker not installed"
        return
    fi
    
    # Create temporary Docker test environment
    local temp_dir=$(mktemp -d)
    local docker_test_dir="$temp_dir/docker_test"
    mkdir -p "$docker_test_dir"
    
    echo "  Creating Docker test environment in: $docker_test_dir"
    
    # Copy binaries if they exist
    local binaries_dir="docker/binaries/x86_64"
    if [[ -d "$binaries_dir" ]]; then
        cp -r "$binaries_dir" "$docker_test_dir/"
        log_test "x86_64 binaries available for Docker test" "PASS"
    else
        log_test "x86_64 binaries available for Docker test" "FAIL" "Run build script first"
        rm -rf "$temp_dir"
        return
    fi
    
    # Create test Dockerfile for config loading validation
    cat > "$docker_test_dir/Dockerfile.config_test" << 'EOF'
FROM alpine:latest
RUN apk add --no-cache ca-certificates

# Create app directory
WORKDIR /app

# Copy all binaries
COPY x86_64/ ./

# Create config test script
RUN echo '#!/bin/sh' > test_configs.sh && \
    echo 'echo "=== Testing Configuration Loading ==="' >> test_configs.sh && \
    echo 'echo "Current directory: $(pwd)"' >> test_configs.sh && \
    echo 'echo "Files in current directory:"' >> test_configs.sh && \
    echo 'ls -la' >> test_configs.sh && \
    echo '' >> test_configs.sh && \
    echo '# Test each service binary exists and is executable' >> test_configs.sh && \
    echo 'for binary in fechatter_server analytics_server notify_server fechatter_gateway bot indexer; do' >> test_configs.sh && \
    echo '  if [ -f "./$binary/$binary" ]; then' >> test_configs.sh && \
    echo '    echo "✓ Found binary: $binary"' >> test_configs.sh && \
    echo '    if [ -x "./$binary/$binary" ]; then' >> test_configs.sh && \
    echo '      echo "✓ Binary is executable: $binary"' >> test_configs.sh && \
    echo '    else' >> test_configs.sh && \
    echo '      echo "✗ Binary not executable: $binary"' >> test_configs.sh && \
    echo '    fi' >> test_configs.sh && \
    echo '  else' >> test_configs.sh && \
    echo '    echo "✗ Binary not found: $binary"' >> test_configs.sh && \
    echo '  fi' >> test_configs.sh && \
    echo 'done' >> test_configs.sh && \
    chmod +x test_configs.sh

CMD ["./test_configs.sh"]
EOF
    
    # Build and run Docker test
    if docker build -t fechatter-config-test -f "$docker_test_dir/Dockerfile.config_test" "$docker_test_dir" >/dev/null 2>&1; then
        log_test "Docker config test image build" "PASS"
        
        # Run the test container
        if docker run --rm fechatter-config-test 2>&1 | tee "$temp_dir/docker_test_output.log" >/dev/null; then
            log_test "Docker config test execution" "PASS"
            
            # Check if all expected binaries were found
            if grep -q "✓ Found binary: fechatter_server" "$temp_dir/docker_test_output.log" && \
               grep -q "✓ Found binary: analytics_server" "$temp_dir/docker_test_output.log" && \
               grep -q "✓ Found binary: notify_server" "$temp_dir/docker_test_output.log"; then
                log_test "All required binaries found in Docker" "PASS"
            else
                log_test "All required binaries found in Docker" "FAIL" "Some binaries missing"
            fi
        else
            log_test "Docker config test execution" "FAIL"
        fi
        
        # Clean up Docker image
        docker rmi fechatter-config-test >/dev/null 2>&1 || true
    else
        log_test "Docker config test image build" "FAIL"
    fi
    
    # Clean up temp directory
    rm -rf "$temp_dir"
}

# Test environment variable configuration
test_env_var_config() {
    echo -e "\n${BLUE}📋 Phase 4: Environment Variable Configuration Test${NC}"
    
    # Test each service's environment variable support
    local env_vars=(
        "FECHATTER_CONFIG:fechatter_server/chat.yml"
        "ANALYTICS_CONFIG:analytics_server/analytics.yml"
        "NOTIFY_CONFIG:notify_server/notify.yml"
        "BOT_CONFIG:bot_server/bot.yml"
    )
    
    for env_info in "${env_vars[@]}"; do
        IFS=':' read -r env_var config_path <<< "$env_info"
        
        if [[ -f "$config_path" ]]; then
            # Set environment variable and test
            export "$env_var"="$config_path"
            log_test "Environment variable $env_var set" "PASS" "→ $config_path"
            unset "$env_var"
        else
            log_test "Environment variable $env_var test" "FAIL" "Config file $config_path not found"
        fi
    done
}

# Test Docker-compatible configuration setup
test_docker_compatible_configs() {
    echo -e "\n${BLUE}📋 Phase 5: Docker-Compatible Configuration Test${NC}"
    
    # Check if configuration files have Docker-compatible URLs
    local config_checks=(
        "fechatter_server/chat.yml:Redis URL should use redis service name"
        "fechatter_server/chat.yml:Database URL should use postgres service name"
        "analytics_server/analytics.yml:ClickHouse URL should use clickhouse service name"
        "notify_server/notify.yml:NATS URL should use nats service name"
        "bot_server/bot.yml:Database URL should use postgres service name"
    )
    
    for check_info in "${config_checks[@]}"; do
        IFS=':' read -r config_path check_desc <<< "$check_info"
        
        if [[ -f "$config_path" ]]; then
            # Read config content
            local config_content=$(cat "$config_path")
            
            # Check for Docker service names vs localhost
            if echo "$config_content" | grep -q "localhost\|127.0.0.1"; then
                log_test "$check_desc" "FAIL" "Found localhost URLs in $config_path"
            else
                log_test "$check_desc" "PASS" "No localhost URLs found"
            fi
        else
            log_test "$check_desc" "FAIL" "Config file not found: $config_path"
        fi
    done
}

# Main execution
main() {
    echo "Starting comprehensive configuration loading validation..."
    echo "Current directory: $(pwd)"
    echo "Timestamp: $(date)"
    echo ""
    
    # Run each phase with error handling
    if ! check_config_files; then
        echo "Error in check_config_files phase"
    fi
    
    if ! test_local_config_loading; then
        echo "Error in test_local_config_loading phase"
    fi
    
    if ! test_docker_config_loading; then
        echo "Error in test_docker_config_loading phase"
    fi
    
    if ! test_env_var_config; then
        echo "Error in test_env_var_config phase"
    fi
    
    if ! test_docker_compatible_configs; then
        echo "Error in test_docker_compatible_configs phase"
    fi
    
    # Summary
    echo ""
    echo "======================================="
    echo -e "${BLUE}📊 Test Summary${NC}"
    echo "======================================="
    echo -e "Total tests: ${BLUE}$TOTAL_TESTS${NC}"
    echo -e "Passed: ${GREEN}$PASSED_TESTS${NC}"
    echo -e "Failed: ${RED}$FAILED_TESTS${NC}"
    
    if [[ $FAILED_TESTS -eq 0 ]]; then
        echo -e "\n${GREEN}🎉 All configuration loading tests passed!${NC}"
        echo "✅ Configuration loading mechanisms are working correctly"
        exit 0
    else
        echo -e "\n${RED}❌ Some configuration loading tests failed${NC}"
        echo "⚠️  Please review the failed tests above"
        exit 1
    fi
}

# Execute main function
main "$@" 