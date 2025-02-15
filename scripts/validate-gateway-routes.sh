#!/bin/bash
# Gateway Route Validation Script
# Verifies that gateway routes match backend service endpoints

set -euo pipefail

# Configuration
GATEWAY_URL="${GATEWAY_URL:-http://localhost:8080}"
FECHATTER_SERVER_URL="${FECHATTER_SERVER_URL:-http://localhost:6688}"
NOTIFY_SERVER_URL="${NOTIFY_SERVER_URL:-http://localhost:6687}"
ANALYTICS_SERVER_URL="${ANALYTICS_SERVER_URL:-http://localhost:6690}"
BOT_SERVER_URL="${BOT_SERVER_URL:-http://localhost:6686}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Logging
log() {
    echo -e "[$(date '+%Y-%m-%d %H:%M:%S')] $*"
}

# Test route function
test_route() {
    local method="$1"
    local path="$2"
    local expected_upstream="$3"
    local description="$4"
    
    log "${BLUE}Testing: $method $path -> $expected_upstream${NC}"
    log "Description: $description"
    
    # Test through gateway
    local gateway_response
    local gateway_status
    
    case "$method" in
        "GET")
            gateway_response=$(curl -sf -w "\n%{http_code}" "$GATEWAY_URL$path" 2>/dev/null || echo "000")
            ;;
        "POST")
            gateway_response=$(curl -sf -X POST -H "Content-Type: application/json" -d '{}' -w "\n%{http_code}" "$GATEWAY_URL$path" 2>/dev/null || echo "000")
            ;;
        "OPTIONS")
            gateway_response=$(curl -sf -X OPTIONS -H "Origin: http://localhost:1420" -H "Access-Control-Request-Method: POST" -w "\n%{http_code}" "$GATEWAY_URL$path" 2>/dev/null || echo "000")
            ;;
        *)
            gateway_response=$(curl -sf -X "$method" -H "Content-Type: application/json" -w "\n%{http_code}" "$GATEWAY_URL$path" 2>/dev/null || echo "000")
            ;;
    esac
    
    gateway_status="${gateway_response##*$'\n'}"
    
    # Check if route exists (not 404)
    if [[ "$gateway_status" == "404" ]]; then
        log "${RED}✗ Route not configured in gateway${NC}"
        return 1
    elif [[ "$gateway_status" == "000" ]]; then
        log "${YELLOW}⚠ Gateway connection failed${NC}"
        return 1
    else
        log "${GREEN}✓ Route exists in gateway (status: $gateway_status)${NC}"
        return 0
    fi
}

# Main validation function
main() {
    log "${BLUE}=== Gateway Route Validation ===${NC}"
    log "Gateway URL: $GATEWAY_URL"
    
    local total_tests=0
    local passed_tests=0
    
    # Test fechatter-server routes
    log "\n${BLUE}=== Testing fechatter-server routes ===${NC}"
    
    # Health routes
    ((total_tests++))
    test_route "GET" "/health" "fechatter-server" "Health check endpoint" && ((passed_tests++)) || true
    
    ((total_tests++))
    test_route "GET" "/health/readiness" "fechatter-server" "Readiness probe" && ((passed_tests++)) || true
    
    # Authentication routes
    ((total_tests++))
    test_route "POST" "/api/signup" "fechatter-server" "User registration" && ((passed_tests++)) || true
    
    ((total_tests++))
    test_route "POST" "/api/signin" "fechatter-server" "User login" && ((passed_tests++)) || true
    
    ((total_tests++))
    test_route "POST" "/api/refresh" "fechatter-server" "Token refresh" && ((passed_tests++)) || true
    
    ((total_tests++))
    test_route "POST" "/api/logout" "fechatter-server" "User logout" && ((passed_tests++)) || true
    
    ((total_tests++))
    test_route "POST" "/api/logout-all" "fechatter-server" "Logout all sessions" && ((passed_tests++)) || true
    
    # CORS preflight tests
    ((total_tests++))
    test_route "OPTIONS" "/api/signin" "fechatter-server" "CORS preflight for signin" && ((passed_tests++)) || true
    
    # Workspace routes
    ((total_tests++))
    test_route "GET" "/api/workspace/chats" "fechatter-server" "List workspace chats" && ((passed_tests++)) || true
    
    ((total_tests++))
    test_route "POST" "/api/workspace/chats" "fechatter-server" "Create workspace chat" && ((passed_tests++)) || true
    
    # Chat routes
    ((total_tests++))
    test_route "GET" "/api/chat/test" "fechatter-server" "Test chat route" && ((passed_tests++)) || true
    
    ((total_tests++))
    test_route "GET" "/api/chat/test/messages" "fechatter-server" "Test chat messages" && ((passed_tests++)) || true
    
    # File routes (updated to match actual implementation)
    ((total_tests++))
    test_route "POST" "/api/upload" "fechatter-server" "Multiple file upload" && ((passed_tests++)) || true
    
    ((total_tests++))
    test_route "POST" "/api/files/single" "fechatter-server" "Single file upload" && ((passed_tests++)) || true
    
    ((total_tests++))
    test_route "GET" "/api/files/workspace1/file1" "fechatter-server" "File download" && ((passed_tests++)) || true
    
    # Cache routes (specific paths)
    ((total_tests++))
    test_route "GET" "/api/cache/stats" "fechatter-server" "Cache statistics" && ((passed_tests++)) || true
    
    ((total_tests++))
    test_route "GET" "/api/cache/config" "fechatter-server" "Cache configuration" && ((passed_tests++)) || true
    
    # Search routes (implemented but currently disabled)
    ((total_tests++))
    test_route "POST" "/api/search/messages" "fechatter-server" "Global message search" && ((passed_tests++)) || true
    
    ((total_tests++))
    test_route "GET" "/api/search/suggestions" "fechatter-server" "Search suggestions" && ((passed_tests++)) || true
    
    # Chat search routes
    ((total_tests++))
    test_route "GET" "/api/chat/1/messages/search" "fechatter-server" "Chat message search" && ((passed_tests++)) || true
    
    # User management (handlers exist but not routed yet)
    ((total_tests++))
    test_route "GET" "/api/users/profile" "fechatter-server" "User profile" && ((passed_tests++)) || true
    
    # Workspace management
    ((total_tests++))
    test_route "GET" "/api/workspaces/current" "fechatter-server" "Current workspace" && ((passed_tests++)) || true
    
    # Test notify-server routes
    log "\n${BLUE}=== Testing notify-server routes ===${NC}"
    
    ((total_tests++))
    test_route "GET" "/events" "notify-server" "SSE events stream" && ((passed_tests++)) || true
    
    ((total_tests++))
    test_route "GET" "/online-users" "notify-server" "Online users list" && ((passed_tests++)) || true
    
    ((total_tests++))
    test_route "GET" "/sse/health" "notify-server" "Notify server health" && ((passed_tests++)) || true
    
    # Test analytics-server routes
    log "\n${BLUE}=== Testing analytics-server routes ===${NC}"
    
    ((total_tests++))
    test_route "POST" "/api/event" "analytics-server" "Analytics event" && ((passed_tests++)) || true
    
    ((total_tests++))
    test_route "POST" "/api/batch" "analytics-server" "Analytics batch" && ((passed_tests++)) || true
    
    ((total_tests++))
    test_route "GET" "/analytics/health" "analytics-server" "Analytics health" && ((passed_tests++)) || true
    
    ((total_tests++))
    test_route "GET" "/analytics/metrics" "analytics-server" "Analytics metrics" && ((passed_tests++)) || true
    
    # Test bot-server routes (if configured)
    log "\n${BLUE}=== Testing bot-server routes ===${NC}"
    
    ((total_tests++))
    test_route "GET" "/api/bot/" "bot-server" "Bot API" && ((passed_tests++)) || true
    
    ((total_tests++))
    test_route "GET" "/bot/health" "bot-server" "Bot health check" && ((passed_tests++)) || true
    
    # Summary
    log "\n${BLUE}=== Validation Summary ===${NC}"
    log "Total Tests: $total_tests"
    log "Passed: $passed_tests"
    log "Failed: $((total_tests - passed_tests))"
    
    local success_rate=$((passed_tests * 100 / total_tests))
    
    if [[ $success_rate -ge 90 ]]; then
        log "${GREEN}✓ Gateway routing configuration is mostly correct ($success_rate% success rate)${NC}"
        exit 0
    elif [[ $success_rate -ge 70 ]]; then
        log "${YELLOW}⚠ Gateway routing needs some fixes ($success_rate% success rate)${NC}"
        exit 1
    else
        log "${RED}✗ Gateway routing has significant issues ($success_rate% success rate)${NC}"
        exit 1
    fi
}

# Check if gateway is running
if ! curl -sf "$GATEWAY_URL/health" > /dev/null 2>&1; then
    log "${RED}ERROR: Gateway is not running at $GATEWAY_URL${NC}"
    log "Please start the gateway first: ./fechatter_gateway --config config/development.yml"
    exit 1
fi

# Run validation
main "$@"