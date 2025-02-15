#!/bin/bash

# Test script for Fechatter monitoring endpoints

echo "======================================"
echo "Fechatter Monitoring Test Script"
echo "======================================"
echo ""

# Color codes
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to test endpoint
test_endpoint() {
    local service=$1
    local endpoint=$2
    local expected_status=${3:-200}
    
    echo -n "Testing $service $endpoint... "
    
    response=$(curl -s -o /dev/null -w "%{http_code}" $endpoint 2>/dev/null)
    
    if [ "$response" = "$expected_status" ]; then
        echo -e "${GREEN}✓ OK${NC} (Status: $response)"
        return 0
    else
        echo -e "${RED}✗ FAILED${NC} (Expected: $expected_status, Got: $response)"
        return 1
    fi
}

# Function to check if service is running
check_service() {
    local service=$1
    local port=$2
    
    echo -e "\n${YELLOW}Checking $service...${NC}"
    
    if nc -z localhost $port 2>/dev/null; then
        echo -e "${GREEN}✓ $service is running on port $port${NC}"
        return 0
    else
        echo -e "${RED}✗ $service is not running on port $port${NC}"
        return 1
    fi
}

# Test fechatter_server
if check_service "fechatter_server" 6688; then
    test_endpoint "fechatter_server" "http://localhost:6688/health"
    test_endpoint "fechatter_server" "http://localhost:6688/api/health"
    test_endpoint "fechatter_server" "http://localhost:9090/metrics"
fi

# Test notify_server
if check_service "notify_server" 6689; then
    test_endpoint "notify_server" "http://localhost:6689/health"
    test_endpoint "notify_server" "http://localhost:6689/ready"
    test_endpoint "notify_server" "http://localhost:6689/live"
    test_endpoint "notify_server" "http://localhost:9091/metrics"
fi

# Test bot_server
if check_service "bot_server (health)" 6686; then
    test_endpoint "bot_server" "http://localhost:6686/health"
    test_endpoint "bot_server" "http://localhost:6686/ready"
    test_endpoint "bot_server" "http://localhost:6686/live"
    test_endpoint "bot_server" "http://localhost:9092/metrics"
fi

# Test analytics_server
if check_service "analytics_server" 7777; then
    test_endpoint "analytics_server" "http://localhost:7777/health"
    test_endpoint "analytics_server" "http://localhost:7777/ready"
    test_endpoint "analytics_server" "http://localhost:7777/live"
    test_endpoint "analytics_server" "http://localhost:7778/metrics"
fi

echo -e "\n======================================"
echo "Monitoring Test Complete"
echo "======================================"

# Prometheus configuration snippet
echo -e "\n${YELLOW}Prometheus Configuration:${NC}"
cat << EOF
# Add this to your prometheus.yml:

scrape_configs:
  - job_name: 'fechatter_server'
    static_configs:
      - targets: ['localhost:9090']
      
  - job_name: 'notify_server'
    static_configs:
      - targets: ['localhost:9091']
      
  - job_name: 'bot_server'
    static_configs:
      - targets: ['localhost:9092']
      
  - job_name: 'analytics_server'
    static_configs:
      - targets: ['localhost:7778']
EOF

echo -e "\n${YELLOW}Quick metrics check:${NC}"
echo "You can view metrics by running:"
echo "  curl http://localhost:9090/metrics | grep fechatter_"
echo "  curl http://localhost:9091/metrics | grep notify_"
echo "  curl http://localhost:9092/metrics | grep bot_"
echo "  curl http://localhost:7778/metrics | grep analytics_"