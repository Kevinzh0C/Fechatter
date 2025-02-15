#!/bin/bash

# Analytics Connection Test Script
# Tests the event flow from fechatter_server to analytics_server via NATS

set -euo pipefail

echo "🧪 Analytics Connection Test"
echo "============================="

# Configuration
NATS_URL="nats://localhost:4222"
FECHATTER_URL="http://localhost:6688"
ANALYTICS_URL="http://localhost:6690"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to check service health
check_service() {
    local service_name="$1"
    local url="$2"
    local timeout=5

    echo -n "Checking $service_name at $url... "
    
    if curl -s --max-time $timeout "$url/health" > /dev/null; then
        echo -e "${GREEN}✅ OK${NC}"
        return 0
    else
        echo -e "${RED}❌ FAILED${NC}"
        return 1
    fi
}

# Function to check NATS connection
check_nats() {
    echo -n "Checking NATS at $NATS_URL... "
    
    # Check if NATS is running (try to connect)
    if command -v nats >/dev/null 2>&1; then
        if nats server check --server="$NATS_URL" >/dev/null 2>&1; then
            echo -e "${GREEN}✅ OK${NC}"
            return 0
        fi
    fi
    
    # Alternative check with nc if nats CLI is not available
    if command -v nc >/dev/null 2>&1; then
        if echo "" | nc -w 1 localhost 4222 >/dev/null 2>&1; then
            echo -e "${GREEN}✅ OK${NC}"
            return 0
        fi
    fi
    
    echo -e "${RED}❌ FAILED${NC}"
    return 1
}

# Function to test event publishing
test_analytics_event() {
    echo -e "${BLUE}📊 Testing analytics event publishing...${NC}"
    
    # Test user login (this should trigger analytics event)
    echo "Attempting to trigger analytics event via user login..."
    
    # Login request to fechatter_server
    local login_response=$(curl -s -X POST "$FECHATTER_URL/api/signin" \
        -H "Content-Type: application/json" \
        -d '{
            "email": "super@none.org",
            "password": "password123"
        }' || echo "failed")
    
    if [[ "$login_response" == "failed" ]]; then
        echo -e "${RED}❌ Failed to make login request${NC}"
        return 1
    fi
    
    # Check if login was successful
    if echo "$login_response" | grep -q "access_token"; then
        echo -e "${GREEN}✅ Login successful - analytics event should be triggered${NC}"
        
        # Wait a moment for event processing
        echo "Waiting 3 seconds for event processing..."
        sleep 3
        
        # Check analytics_server logs for the event
        echo "Event should appear in analytics_server logs with subject: fechatter.analytics.user.login"
        return 0
    else
        echo -e "${YELLOW}⚠️ Login failed, but this might be expected${NC}"
        echo "Response: $login_response"
        return 1
    fi
}

# Function to check NATS stream configuration
check_nats_streams() {
    echo -e "${BLUE}📦 Checking NATS JetStream configuration...${NC}"
    
    if command -v nats >/dev/null 2>&1; then
        echo "Checking ANALYTICS stream..."
        if nats stream info ANALYTICS --server="$NATS_URL" >/dev/null 2>&1; then
            echo -e "${GREEN}✅ ANALYTICS stream exists${NC}"
            nats stream info ANALYTICS --server="$NATS_URL" | grep -E "(Name|Subjects|Storage)" || true
        else
            echo -e "${YELLOW}⚠️ ANALYTICS stream not found${NC}"
        fi
    else
        echo -e "${YELLOW}⚠️ NATS CLI not available, skipping stream check${NC}"
    fi
}

# Function to show analytics_server configuration
show_config() {
    echo -e "${BLUE}⚙️ Configuration Summary:${NC}"
    echo "NATS URL: $NATS_URL"
    echo "Fechatter Server: $FECHATTER_URL"
    echo "Analytics Server: $ANALYTICS_URL"
    echo ""
    echo "Expected NATS subjects:"
    echo "  - fechatter.analytics.user.login"
    echo "  - fechatter.analytics.message.sent"
    echo "  - fechatter.analytics.chat.created"
    echo ""
}

# Main test execution
main() {
    show_config
    
    echo -e "${BLUE}🔍 Service Health Checks${NC}"
    echo "-------------------------"
    
    # Check if all services are running
    local services_ok=true
    
    if ! check_nats; then
        services_ok=false
    fi
    
    if ! check_service "Fechatter Server" "$FECHATTER_URL"; then
        services_ok=false
    fi
    
    if ! check_service "Analytics Server" "$ANALYTICS_URL"; then
        services_ok=false
    fi
    
    if [[ "$services_ok" == "false" ]]; then
        echo -e "${RED}❌ Some services are not running. Please start all services first.${NC}"
        echo ""
        echo "To start services:"
        echo "  1. Start NATS: nats-server --jetstream"
        echo "  2. Start Analytics Server: cargo run --bin analytics_server"
        echo "  3. Start Fechatter Server: cargo run --bin fechatter_server"
        exit 1
    fi
    
    echo ""
    
    # Check NATS streams
    check_nats_streams
    echo ""
    
    # Test analytics event flow
    if test_analytics_event; then
        echo ""
        echo -e "${GREEN}🎉 Analytics connection test completed successfully!${NC}"
        echo ""
        echo "Next steps:"
        echo "  1. Check analytics_server logs for incoming events"
        echo "  2. Query ClickHouse to verify data storage"
        echo "  3. Monitor NATS subjects with: nats sub 'fechatter.analytics.>' --server=$NATS_URL"
    else
        echo ""
        echo -e "${YELLOW}⚠️ Analytics test completed with warnings${NC}"
        echo "Check the logs for more details."
    fi
}

# Help function
show_help() {
    echo "Analytics Connection Test Script"
    echo ""
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  -h, --help     Show this help message"
    echo "  --nats URL     NATS server URL (default: nats://localhost:4222)"
    echo "  --fechatter URL Fechatter server URL (default: http://localhost:6688)"
    echo "  --analytics URL Analytics server URL (default: http://localhost:6690)"
    echo ""
    echo "This script tests the analytics event connection between fechatter_server"
    echo "and analytics_server via NATS messaging."
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            show_help
            exit 0
            ;;
        --nats)
            NATS_URL="$2"
            shift 2
            ;;
        --fechatter)
            FECHATTER_URL="$2"
            shift 2
            ;;
        --analytics)
            ANALYTICS_URL="$2"
            shift 2
            ;;
        *)
            echo "Unknown option: $1"
            show_help
            exit 1
            ;;
    esac
done

# Run main function
main