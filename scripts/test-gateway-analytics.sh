#!/bin/bash

# Gateway Analytics API Testing Script
# 验证 fechatter_gateway 是否正确路由了 analytics_server 的所有 API

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
GATEWAY_URL=${GATEWAY_URL:-"http://localhost:8080"}
ANALYTICS_DIRECT_URL=${ANALYTICS_DIRECT_URL:-"http://localhost:6690"}

echo -e "${BLUE}🚀 Gateway Analytics API Testing${NC}"
echo "====================================="
echo "Gateway URL: $GATEWAY_URL"
echo "Analytics Direct URL: $ANALYTICS_DIRECT_URL"
echo ""

# Function to test an endpoint
test_endpoint() {
    local method=$1
    local path=$2
    local url=$3
    local description=$4
    local expect_code=${5:-200}
    
    echo -n "Testing $description... "
    
    response=$(curl -s -w "%{http_code}" -X "$method" \
        -H "Content-Type: application/json" \
        "$url$path" -o /dev/null 2>/dev/null || echo "000")
    
    if [[ "$response" == "$expect_code" ]]; then
        echo -e "${GREEN}✅ Pass (HTTP $response)${NC}"
        return 0
    else
        echo -e "${RED}❌ Fail (HTTP $response, expected $expect_code)${NC}"
        return 1
    fi
}

# Function to test JSON endpoint with response
test_json_endpoint() {
    local method=$1
    local path=$2
    local url=$3
    local description=$4
    
    echo "Testing $description..."
    
    response=$(curl -s -X "$method" \
        -H "Content-Type: application/json" \
        -H "Accept: application/json" \
        "$url$path" 2>/dev/null || echo "{\"error\": \"connection_failed\"}")
    
    echo -e "${YELLOW}Response:${NC} $response"
    echo ""
}

# Function to run all tests
run_tests() {
    local base_url=$1
    local test_name=$2
    
    echo -e "${BLUE}📋 Testing $test_name${NC}"
    echo "-------------------"
    
    # Test Main Analytics API Endpoints
    echo -e "${YELLOW}📤 Analytics Event API${NC}"
    test_endpoint "POST" "/api/event" "$base_url" "Single Event Endpoint" "400"
    test_endpoint "POST" "/api/batch" "$base_url" "Batch Events Endpoint" "400"
    test_endpoint "OPTIONS" "/api/event" "$base_url" "Event CORS" "200"
    test_endpoint "OPTIONS" "/api/batch" "$base_url" "Batch CORS" "200"
    
    # Test Health Check Endpoints  
    echo -e "${YELLOW}🏥 Health Check API${NC}"
    test_endpoint "GET" "/analytics/health" "$base_url" "Health Check"
    test_endpoint "GET" "/analytics/ready" "$base_url" "Ready Check"
    test_endpoint "GET" "/analytics/live" "$base_url" "Live Check"
    test_endpoint "GET" "/analytics/metrics" "$base_url" "Metrics Endpoint"
    
    # Test OpenAPI Documentation
    echo -e "${YELLOW}📚 OpenAPI Documentation${NC}"
    test_endpoint "GET" "/analytics/openapi.json" "$base_url" "OpenAPI Spec"
    test_endpoint "GET" "/analytics/swagger-ui/" "$base_url" "Swagger UI" "200"
    
    echo ""
}

# Main test execution
echo -e "${YELLOW}🎯 Starting Gateway Analytics API Tests${NC}"
echo ""

# Test 1: Direct Analytics Server
if curl -s "$ANALYTICS_DIRECT_URL/health" > /dev/null 2>&1; then
    echo -e "${GREEN}✅ Analytics Server is running${NC}"
    run_tests "$ANALYTICS_DIRECT_URL" "Direct Analytics Server"
else
    echo -e "${RED}❌ Analytics Server is not accessible${NC}"
    echo "Please start analytics_server first:"
    echo "  cd analytics_server && cargo run"
    echo ""
fi

# Test 2: Gateway Routing
if curl -s "$GATEWAY_URL/health" > /dev/null 2>&1; then
    echo -e "${GREEN}✅ Gateway is running${NC}"
    run_tests "$GATEWAY_URL" "Gateway Routing"
else
    echo -e "${RED}❌ Gateway is not accessible${NC}"
    echo "Please start fechatter_gateway first:"
    echo "  cd fechatter_gateway && cargo run"
    echo ""
fi

# Test 3: Gateway vs Direct Comparison
echo -e "${BLUE}🔍 Gateway vs Direct Server Comparison${NC}"
echo "-------------------------------------"

if curl -s "$GATEWAY_URL/health" > /dev/null 2>&1 && curl -s "$ANALYTICS_DIRECT_URL/health" > /dev/null 2>&1; then
    echo "Comparing health check responses..."
    
    echo -e "${YELLOW}Direct Analytics Server:${NC}"
    test_json_endpoint "GET" "/health" "$ANALYTICS_DIRECT_URL" "Direct Health Check"
    
    echo -e "${YELLOW}Via Gateway:${NC}"
    test_json_endpoint "GET" "/analytics/health" "$GATEWAY_URL" "Gateway Health Check"
    
    echo -e "${YELLOW}OpenAPI Spec Comparison:${NC}"
    echo "Direct: $ANALYTICS_DIRECT_URL/openapi.json"
    echo "Gateway: $GATEWAY_URL/analytics/openapi.json"
else
    echo -e "${YELLOW}⚠️  Cannot compare - one or both services are not running${NC}"
fi

echo ""
echo -e "${BLUE}✅ Gateway Analytics API Test Complete${NC}"
echo ""
echo -e "${YELLOW}📋 Summary of Analytics API Endpoints:${NC}"
echo "  Main APIs:"
echo "    POST $GATEWAY_URL/api/event      (Single analytics event)"
echo "    POST $GATEWAY_URL/api/batch      (Batch analytics events)"
echo ""
echo "  Health & Monitoring:"
echo "    GET  $GATEWAY_URL/analytics/health   (Health check)"
echo "    GET  $GATEWAY_URL/analytics/ready    (Ready check)"  
echo "    GET  $GATEWAY_URL/analytics/live     (Live check)"
echo "    GET  $GATEWAY_URL/analytics/metrics  (Prometheus metrics)"
echo ""
echo "  Documentation:"
echo "    GET  $GATEWAY_URL/analytics/openapi.json   (OpenAPI spec)"
echo "    GET  $GATEWAY_URL/analytics/swagger-ui/    (Swagger UI)" 