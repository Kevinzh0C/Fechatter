#!/bin/bash

# Bot Server Health Check Script
# This script tests if the bot_server is working correctly

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default configuration
BOT_HEALTH_PORT=${BOT_HEALTH_PORT:-6686}
POSTGRES_URL=${DATABASE_URL:-"postgres://postgres:postgres@localhost:5432/fechatter"}
NATS_URL=${NATS_URL:-"nats://localhost:4222"}
ANALYTICS_URL=${ANALYTICS_URL:-"http://localhost:6690"}

echo -e "${BLUE}🤖 Bot Server Health Check${NC}"
echo "=================================="

# Function to print status
print_status() {
    if [ $1 -eq 0 ]; then
        echo -e "${GREEN}✅ $2${NC}"
    else
        echo -e "${RED}❌ $2${NC}"
    fi
}

# Function to check if a service is running on a port
check_port() {
    local port=$1
    local service_name=$2
    if nc -z localhost $port 2>/dev/null; then
        print_status 0 "$service_name is running on port $port"
        return 0
    else
        print_status 1 "$service_name is not running on port $port"
        return 1
    fi
}

# Function to check HTTP endpoint
check_http() {
    local url=$1
    local service_name=$2
    local timeout=${3:-5}
    
    if curl -s --max-time $timeout "$url" > /dev/null 2>&1; then
        print_status 0 "$service_name HTTP endpoint is responding"
        return 0
    else
        print_status 1 "$service_name HTTP endpoint is not responding"
        return 1
    fi
}

# Function to check HTTP endpoint and show response
check_http_verbose() {
    local url=$1
    local service_name=$2
    local timeout=${3:-5}
    
    echo -e "${YELLOW}📡 Testing $service_name: $url${NC}"
    
    response=$(curl -s --max-time $timeout "$url" 2>/dev/null || echo "")
    if [ $? -eq 0 ] && [ -n "$response" ]; then
        print_status 0 "$service_name is healthy"
        echo "Response: $response" | jq . 2>/dev/null || echo "Response: $response"
        return 0
    else
        print_status 1 "$service_name health check failed"
        return 1
    fi
}

echo -e "\n${BLUE}1. Checking prerequisite services...${NC}"

# Check PostgreSQL
echo -e "\n${YELLOW}🐘 PostgreSQL Database${NC}"
if command -v psql >/dev/null 2>&1; then
    if psql "$POSTGRES_URL" -c "SELECT 1;" > /dev/null 2>&1; then
        print_status 0 "PostgreSQL connection successful"
        
        # Check if bot users exist
        bot_count=$(psql "$POSTGRES_URL" -t -c "SELECT COUNT(*) FROM users WHERE is_bot = TRUE;" 2>/dev/null | xargs)
        echo "Bot users in database: $bot_count"
    else
        print_status 1 "PostgreSQL connection failed"
        echo -e "${YELLOW}💡 Start PostgreSQL or check DATABASE_URL${NC}"
    fi
else
    print_status 1 "psql command not found"
    echo -e "${YELLOW}💡 Install PostgreSQL client tools${NC}"
fi

# Check NATS
echo -e "\n${YELLOW}📡 NATS Server${NC}"
if check_port 4222 "NATS"; then
    # Additional NATS test if nats CLI is available
    if command -v nats >/dev/null 2>&1; then
        if echo "test" | nats pub test.health.check --stdin 2>/dev/null; then
            print_status 0 "NATS publish test successful"
        else
            print_status 1 "NATS publish test failed"
        fi
    fi
else
    echo -e "${YELLOW}💡 Start NATS server: nats-server${NC}"
fi

# Check Analytics Server
echo -e "\n${YELLOW}📊 Analytics Server${NC}"
if check_port 6690 "Analytics Server"; then
    check_http_verbose "$ANALYTICS_URL/health" "Analytics Server" 3
else
    echo -e "${YELLOW}💡 Start analytics server: cd analytics_server && cargo run${NC}"
fi

echo -e "\n${BLUE}2. Checking bot_server health endpoint...${NC}"

# Check Bot Server Health
if check_port $BOT_HEALTH_PORT "Bot Server"; then
    echo -e "\n${YELLOW}🏥 Bot Server Health Checks${NC}"
    
    # Full health check
    check_http_verbose "http://localhost:$BOT_HEALTH_PORT/health" "Bot Server Health" 10
    
    # Readiness check
    echo -e "\n${YELLOW}📋 Readiness Check${NC}"
    check_http_verbose "http://localhost:$BOT_HEALTH_PORT/ready" "Bot Server Readiness" 5
    
    # Liveness check
    echo -e "\n${YELLOW}💓 Liveness Check${NC}"
    check_http_verbose "http://localhost:$BOT_HEALTH_PORT/live" "Bot Server Liveness" 3
else
    echo -e "${RED}❌ Bot server is not running${NC}"
    echo -e "${YELLOW}💡 Start bot server: cd bot_server && cargo run --bin bot${NC}"
    exit 1
fi

echo -e "\n${BLUE}3. Checking AI capabilities...${NC}"

# Test AI SDK
echo -e "\n${YELLOW}🧠 AI SDK Test${NC}"
if [ -f "ai_sdk/Cargo.toml" ]; then
    cd ai_sdk
    if cargo run --example test_bot; then
        print_status 0 "AI SDK test completed"
    else
        print_status 1 "AI SDK test failed"
        echo -e "${YELLOW}💡 Check OPENAI_API_KEY or Ollama installation${NC}"
    fi
    cd ..
else
    print_status 1 "AI SDK not found"
fi

echo -e "\n${BLUE}4. Configuration validation...${NC}"

# Check bot configuration
echo -e "\n${YELLOW}⚙️  Bot Configuration${NC}"
if [ -f "bot_server/bot.yml" ]; then
    print_status 0 "bot.yml configuration file found"
    
    # Validate YAML syntax
    if command -v yq >/dev/null 2>&1; then
        if yq eval . bot_server/bot.yml > /dev/null 2>&1; then
            print_status 0 "bot.yml syntax is valid"
        else
            print_status 1 "bot.yml has syntax errors"
        fi
    fi
else
    print_status 1 "bot.yml configuration file not found"
    echo -e "${YELLOW}💡 Copy bot.yml.example to bot.yml and configure${NC}"
fi

# Check environment variables
echo -e "\n${YELLOW}🌍 Environment Variables${NC}"
if [ -n "$OPENAI_API_KEY" ]; then
    print_status 0 "OPENAI_API_KEY is set"
else
    print_status 1 "OPENAI_API_KEY is not set"
    echo -e "${YELLOW}💡 Set OPENAI_API_KEY for AI functionality${NC}"
fi

echo -e "\n${BLUE}5. End-to-end test...${NC}"

# Test message processing (if we can)
echo -e "\n${YELLOW}📨 NATS Message Test${NC}"
if command -v nats >/dev/null 2>&1; then
    # Create a test message
    test_message='{
        "msg": {
            "id": 999999,
            "chat_id": 1,
            "sender_id": 1,
            "content": "Hello bot!",
            "created_at": "'$(date -u +%Y-%m-%dT%H:%M:%S.%3NZ)'"
        },
        "members": [1, 2]
    }'
    
    echo "Publishing test message to fechatter.messages.created..."
    if echo "$test_message" | nats pub fechatter.messages.created --stdin; then
        print_status 0 "Test message published successfully"
        echo -e "${YELLOW}💡 Check bot_server logs to see if message was processed${NC}"
    else
        print_status 1 "Failed to publish test message"
    fi
else
    print_status 1 "nats CLI not available for end-to-end test"
    echo -e "${YELLOW}💡 Install nats CLI: go install github.com/nats-io/natscli/nats@latest${NC}"
fi

echo -e "\n${BLUE}📋 Summary & Recommendations${NC}"
echo "=================================="

# Generate recommendations based on checks
echo -e "\n${YELLOW}💡 Next Steps:${NC}"
echo "1. ✅ Ensure all prerequisite services are running"
echo "2. 🔧 Check bot_server health endpoint: http://localhost:$BOT_HEALTH_PORT/health"
echo "3. 📝 Review bot_server logs for any errors"
echo "4. 🧪 Send a test message to a bot user via the main app"
echo "5. 📊 Monitor analytics for bot response events"

echo -e "\n${YELLOW}🚀 Quick Start Commands:${NC}"
echo "# Start all services:"
echo "docker-compose up -d postgres redis nats clickhouse"
echo ""
echo "# Start analytics server:"
echo "cd analytics_server && cargo run"
echo ""
echo "# Start bot server:"
echo "cd bot_server && OPENAI_API_KEY=your_key cargo run --bin bot"
echo ""
echo "# Test health:"
echo "curl http://localhost:$BOT_HEALTH_PORT/health | jq"

echo -e "\n${GREEN}🎉 Bot health check complete!${NC}" 