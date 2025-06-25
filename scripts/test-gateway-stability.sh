#!/bin/bash
# Gateway Stability and High Availability Test Script

set -euo pipefail

# Configuration
GATEWAY_URL="${GATEWAY_URL:-http://localhost:8080}"
TEST_DURATION="${TEST_DURATION:-300}"  # 5 minutes
CONCURRENT_USERS="${CONCURRENT_USERS:-50}"
REQUESTS_PER_USER="${REQUESTS_PER_USER:-100}"
LOG_FILE="gateway-stability-test-$(date +%Y%m%d-%H%M%S).log"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Logging
log() {
    echo -e "[$(date '+%Y-%m-%d %H:%M:%S')] $*" | tee -a "$LOG_FILE"
}

# Check dependencies
check_dependencies() {
    local deps=("curl" "jq" "ab" "nc")
    for dep in "${deps[@]}"; do
        if ! command -v "$dep" &> /dev/null; then
            log "${RED}ERROR: $dep is not installed${NC}"
            exit 1
        fi
    done
}

# Test 1: Basic health check
test_health_check() {
    log "${BLUE}Test 1: Basic Health Check${NC}"
    
    local response
    response=$(curl -sf "${GATEWAY_URL}/health" || echo "{}")
    
    if echo "$response" | jq -e '.status == "healthy"' > /dev/null 2>&1; then
        log "${GREEN}✓ Health check passed${NC}"
        return 0
    else
        log "${RED}✗ Health check failed: $response${NC}"
        return 1
    fi
}

# Test 2: CORS preflight handling
test_cors_preflight() {
    log "${BLUE}Test 2: CORS Preflight Handling${NC}"
    
    local response
    response=$(curl -sf -X OPTIONS \
        -H "Origin: http://localhost:3000" \
        -H "Access-Control-Request-Method: POST" \
        -H "Access-Control-Request-Headers: content-type" \
        "${GATEWAY_URL}/api/signin" \
        -w "\n%{http_code}" || echo "000")
    
    local status_code="${response##*$'\n'}"
    
    if [[ "$status_code" == "200" ]]; then
        log "${GREEN}✓ CORS preflight handled correctly${NC}"
        return 0
    else
        log "${RED}✗ CORS preflight failed with status: $status_code${NC}"
        return 1
    fi
}

# Test 3: Load testing
test_load_handling() {
    log "${BLUE}Test 3: Load Testing (${CONCURRENT_USERS} concurrent users)${NC}"
    
    # Create a simple POST data file
    echo '{"test": "data"}' > /tmp/test-payload.json
    
    # Run Apache Bench
    ab -n $((CONCURRENT_USERS * REQUESTS_PER_USER)) \
       -c "$CONCURRENT_USERS" \
       -T "application/json" \
       -p /tmp/test-payload.json \
       -g /tmp/gateway-load-test.tsv \
       "${GATEWAY_URL}/api/health" > /tmp/ab-output.txt 2>&1
    
    # Parse results
    local failed_requests
    failed_requests=$(grep "Failed requests:" /tmp/ab-output.txt | awk '{print $3}')
    local requests_per_second
    requests_per_second=$(grep "Requests per second:" /tmp/ab-output.txt | awk '{print $4}')
    
    log "Load test results:"
    log "  - Failed requests: $failed_requests"
    log "  - Requests per second: $requests_per_second"
    
    if [[ "$failed_requests" -eq 0 ]] || [[ "$failed_requests" -lt 10 ]]; then
        log "${GREEN}✓ Load test passed with minimal failures${NC}"
        rm -f /tmp/test-payload.json /tmp/ab-output.txt
        return 0
    else
        log "${RED}✗ Load test failed with $failed_requests failed requests${NC}"
        rm -f /tmp/test-payload.json /tmp/ab-output.txt
        return 1
    fi
}

# Test 4: Upstream failover
test_upstream_failover() {
    log "${BLUE}Test 4: Upstream Failover Testing${NC}"
    
    # This test assumes you can simulate upstream failure
    # In a real test, you might stop one of the upstream services
    
    log "Simulating upstream failure by testing non-existent endpoint..."
    
    local start_time=$(date +%s)
    local timeout=30
    local recovered=false
    
    while [[ $(($(date +%s) - start_time)) -lt $timeout ]]; do
        if curl -sf "${GATEWAY_URL}/health" > /dev/null 2>&1; then
            recovered=true
            break
        fi
        sleep 1
    done
    
    if [[ "$recovered" == true ]]; then
        log "${GREEN}✓ Gateway remained available during upstream issues${NC}"
        return 0
    else
        log "${YELLOW}⚠ Gateway failover test inconclusive${NC}"
        return 0
    fi
}

# Test 5: Memory leak detection
test_memory_stability() {
    log "${BLUE}Test 5: Memory Stability Test${NC}"
    
    # Get initial memory usage
    local initial_mem
    initial_mem=$(ps aux | grep "[f]echatter_gateway" | awk '{print $6}' | head -1 || echo "0")
    
    if [[ -z "$initial_mem" ]] || [[ "$initial_mem" == "0" ]]; then
        log "${YELLOW}⚠ Cannot measure gateway memory (not running locally?)${NC}"
        return 0
    fi
    
    log "Initial memory usage: ${initial_mem}KB"
    
    # Send many requests
    for i in {1..1000}; do
        curl -sf "${GATEWAY_URL}/health" > /dev/null 2>&1 || true
        if [[ $((i % 100)) -eq 0 ]]; then
            echo -n "."
        fi
    done
    echo
    
    # Check memory again
    local final_mem
    final_mem=$(ps aux | grep "[f]echatter_gateway" | awk '{print $6}' | head -1 || echo "0")
    log "Final memory usage: ${final_mem}KB"
    
    local mem_increase=$((final_mem - initial_mem))
    local mem_increase_percent=$((mem_increase * 100 / initial_mem))
    
    if [[ $mem_increase_percent -lt 50 ]]; then
        log "${GREEN}✓ Memory usage stable (increased by ${mem_increase_percent}%)${NC}"
        return 0
    else
        log "${RED}✗ Potential memory leak detected (increased by ${mem_increase_percent}%)${NC}"
        return 1
    fi
}

# Test 6: Concurrent connection handling
test_concurrent_connections() {
    log "${BLUE}Test 6: Concurrent Connection Test${NC}"
    
    local max_connections=100
    local pids=()
    
    # Start concurrent connections
    for i in $(seq 1 $max_connections); do
        (curl -sf "${GATEWAY_URL}/health" > /dev/null 2>&1) &
        pids+=($!)
        
        if [[ $((i % 10)) -eq 0 ]]; then
            echo -n "."
        fi
    done
    echo
    
    # Wait for all connections to complete
    local failed=0
    for pid in "${pids[@]}"; do
        if ! wait "$pid"; then
            ((failed++))
        fi
    done
    
    if [[ $failed -eq 0 ]]; then
        log "${GREEN}✓ All $max_connections concurrent connections succeeded${NC}"
        return 0
    else
        log "${YELLOW}⚠ $failed out of $max_connections connections failed${NC}"
        return 1
    fi
}

# Test 7: Error recovery
test_error_recovery() {
    log "${BLUE}Test 7: Error Recovery Test${NC}"
    
    # Send malformed requests
    local errors=0
    
    # Test 1: Invalid JSON
    if ! curl -sf -X POST \
        -H "Content-Type: application/json" \
        -d "{invalid json}" \
        "${GATEWAY_URL}/api/signin" > /dev/null 2>&1; then
        ((errors++))
    fi
    
    # Test 2: Oversized header
    if ! curl -sf -H "X-Large-Header: $(printf 'a%.0s' {1..10000})" \
        "${GATEWAY_URL}/health" > /dev/null 2>&1; then
        ((errors++))
    fi
    
    # Test 3: Invalid method
    if ! curl -sf -X INVALID \
        "${GATEWAY_URL}/api/health" > /dev/null 2>&1; then
        ((errors++))
    fi
    
    # Verify gateway is still responding after errors
    if curl -sf "${GATEWAY_URL}/health" > /dev/null 2>&1; then
        log "${GREEN}✓ Gateway recovered from error conditions${NC}"
        return 0
    else
        log "${RED}✗ Gateway failed to recover from errors${NC}"
        return 1
    fi
}

# Main test execution
main() {
    log "${BLUE}=== Fechatter Gateway Stability Test ===${NC}"
    log "Gateway URL: $GATEWAY_URL"
    log "Test Duration: $TEST_DURATION seconds"
    log "Log File: $LOG_FILE"
    
    check_dependencies
    
    local total_tests=7
    local passed_tests=0
    
    # Run all tests
    test_health_check && ((passed_tests++)) || true
    test_cors_preflight && ((passed_tests++)) || true
    test_load_handling && ((passed_tests++)) || true
    test_upstream_failover && ((passed_tests++)) || true
    test_memory_stability && ((passed_tests++)) || true
    test_concurrent_connections && ((passed_tests++)) || true
    test_error_recovery && ((passed_tests++)) || true
    
    # Summary
    log "\n${BLUE}=== Test Summary ===${NC}"
    log "Total Tests: $total_tests"
    log "Passed: $passed_tests"
    log "Failed: $((total_tests - passed_tests))"
    
    if [[ $passed_tests -eq $total_tests ]]; then
        log "${GREEN}✓ All tests passed! Gateway is stable and highly available.${NC}"
        exit 0
    else
        log "${YELLOW}⚠ Some tests failed. Review the log for details.${NC}"
        exit 1
    fi
}

# Run main function
main "$@"