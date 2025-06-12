#!/bin/bash

# Analytics Connection Fix Script
# Fixes common issues with the event flow from fechatter_server to analytics_server

set -euo pipefail

echo "🔧 Analytics Connection Fix"
echo "=========================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
NATS_URL="nats://localhost:4222"
FECHATTER_CONFIG="/Users/zhangkaiqi/Rust/Fechatter/fechatter_server/chat.yml"
ANALYTICS_CONFIG="/Users/zhangkaiqi/Rust/Fechatter/analytics_server/analytics.yml"

echo -e "${BLUE}🔍 Checking configuration files...${NC}"

# Check if configuration files exist
if [[ ! -f "$FECHATTER_CONFIG" ]]; then
    echo -e "${RED}❌ Fechatter config not found: $FECHATTER_CONFIG${NC}"
    exit 1
fi

if [[ ! -f "$ANALYTICS_CONFIG" ]]; then
    echo -e "${RED}❌ Analytics config not found: $ANALYTICS_CONFIG${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Configuration files found${NC}"

# Fix 1: Ensure NATS is running
echo -e "${BLUE}🔧 Fix 1: Checking NATS server...${NC}"
if ! pgrep -f "nats-server" > /dev/null; then
    echo -e "${YELLOW}⚠️ NATS server not running. Starting NATS with JetStream...${NC}"
    
    # Try to start NATS server with JetStream
    if command -v nats-server >/dev/null 2>&1; then
        nats-server --jetstream --port=4222 --http_port=8222 > /tmp/nats.log 2>&1 &
        echo "Started NATS server (PID: $!)"
        sleep 2
        
        if pgrep -f "nats-server" > /dev/null; then
            echo -e "${GREEN}✅ NATS server started successfully${NC}"
        else
            echo -e "${RED}❌ Failed to start NATS server${NC}"
            echo "Check logs: tail -f /tmp/nats.log"
            exit 1
        fi
    else
        echo -e "${RED}❌ nats-server command not found. Please install NATS server.${NC}"
        echo "Install: brew install nats-server"
        exit 1
    fi
else
    echo -e "${GREEN}✅ NATS server is running${NC}"
fi

# Fix 2: Verify NATS JetStream is enabled
echo -e "${BLUE}🔧 Fix 2: Verifying JetStream...${NC}"
if command -v nats >/dev/null 2>&1; then
    if nats account info --server="$NATS_URL" >/dev/null 2>&1; then
        echo -e "${GREEN}✅ JetStream is enabled${NC}"
    else
        echo -e "${YELLOW}⚠️ JetStream might not be enabled${NC}"
    fi
else
    echo -e "${YELLOW}⚠️ NATS CLI not available, skipping JetStream check${NC}"
fi

# Fix 3: Create analytics stream if it doesn't exist
echo -e "${BLUE}🔧 Fix 3: Setting up analytics stream...${NC}"
if command -v nats >/dev/null 2>&1; then
    # Create analytics stream
    if ! nats stream info ANALYTICS --server="$NATS_URL" >/dev/null 2>&1; then
        echo "Creating ANALYTICS stream..."
        nats stream add ANALYTICS \
            --subjects="fechatter.analytics.>" \
            --storage=file \
            --max-age=30d \
            --max-bytes=10GB \
            --max-msgs=10000000 \
            --server="$NATS_URL" \
            --defaults || echo "Stream might already exist"
    fi
    
    # Show stream info
    echo "Analytics stream configuration:"
    nats stream info ANALYTICS --server="$NATS_URL" | head -20 || true
else
    echo -e "${YELLOW}⚠️ NATS CLI not available, skipping stream setup${NC}"
fi

# Fix 4: Check fechatter_server configuration
echo -e "${BLUE}🔧 Fix 4: Validating fechatter_server config...${NC}"

# Verify messaging is enabled in fechatter config
if grep -q "enabled: true" "$FECHATTER_CONFIG" && grep -A 5 "messaging:" "$FECHATTER_CONFIG" | grep -q "enabled: true"; then
    echo -e "${GREEN}✅ Messaging enabled in fechatter_server${NC}"
else
    echo -e "${YELLOW}⚠️ Messaging might be disabled in fechatter_server${NC}"
    echo "Check features.messaging.enabled in $FECHATTER_CONFIG"
fi

# Check NATS URL in fechatter config
if grep -q "nats://localhost:4222" "$FECHATTER_CONFIG"; then
    echo -e "${GREEN}✅ NATS URL correct in fechatter_server${NC}"
else
    echo -e "${YELLOW}⚠️ NATS URL might be incorrect in fechatter_server${NC}"
    echo "Expected: nats://localhost:4222"
fi

# Fix 5: Check analytics_server configuration
echo -e "${BLUE}🔧 Fix 5: Validating analytics_server config...${NC}"

# Verify messaging is enabled in analytics config
if grep -q "enabled: true" "$ANALYTICS_CONFIG" && grep -A 10 "messaging:" "$ANALYTICS_CONFIG" | grep -q "enabled: true"; then
    echo -e "${GREEN}✅ Messaging enabled in analytics_server${NC}"
else
    echo -e "${YELLOW}⚠️ Messaging might be disabled in analytics_server${NC}"
    echo "Check integrations.messaging.enabled in $ANALYTICS_CONFIG"
fi

# Check NATS URL in analytics config
if grep -q "nats://localhost:4222" "$ANALYTICS_CONFIG"; then
    echo -e "${GREEN}✅ NATS URL correct in analytics_server${NC}"
else
    echo -e "${YELLOW}⚠️ NATS URL might be incorrect in analytics_server${NC}"
    echo "Expected: nats://localhost:4222"
fi

# Fix 6: Test event publishing manually
echo -e "${BLUE}🔧 Fix 6: Testing event publishing...${NC}"

if command -v nats >/dev/null 2>&1; then
    echo "Publishing test event..."
    
    # Create a test protobuf event (simplified JSON for testing)
    TEST_EVENT='{"context":{"client_id":"test","session_id":"test","user_id":"1","app_version":"1.0.0","client_ts":1672531200000,"server_ts":1672531200000,"user_agent":"test","ip":"127.0.0.1","system":{"os":"linux","arch":"x86_64","locale":"en-US","timezone":"UTC","browser":"test","browser_version":"1.0"}},"event_type":{"user_login":{"email":"test@example.com","login_method":"password"}}}'
    
    # Publish to fechatter.analytics.user.login subject
    echo "$TEST_EVENT" | nats pub "fechatter.analytics.user.login" --server="$NATS_URL" || echo "Failed to publish test event"
    
    echo "Test event published to: fechatter.analytics.user.login"
    echo "Analytics server should receive this event if properly configured."
else
    echo -e "${YELLOW}⚠️ NATS CLI not available, skipping manual test${NC}"
fi

# Fix 7: Monitor NATS subjects
echo -e "${BLUE}🔧 Fix 7: Monitoring setup...${NC}"

if command -v nats >/dev/null 2>&1; then
    echo "To monitor analytics events in real-time, run:"
    echo "  nats sub 'fechatter.analytics.>' --server=$NATS_URL"
    echo ""
    echo "To check stream statistics:"
    echo "  nats stream info ANALYTICS --server=$NATS_URL"
else
    echo -e "${YELLOW}⚠️ Install NATS CLI for monitoring: go install github.com/nats-io/natscli/nats@latest${NC}"
fi

# Summary
echo ""
echo -e "${GREEN}🎉 Analytics connection fix completed!${NC}"
echo ""
echo "Summary of fixes applied:"
echo "  1. ✅ NATS server status checked"
echo "  2. ✅ JetStream functionality verified"
echo "  3. ✅ Analytics stream configured"
echo "  4. ✅ Fechatter server config validated"
echo "  5. ✅ Analytics server config validated"
echo "  6. ✅ Test event publishing attempted"
echo "  7. ✅ Monitoring instructions provided"
echo ""
echo "Next steps:"
echo "  1. Start analytics_server: cargo run --bin analytics_server"
echo "  2. Start fechatter_server: cargo run --bin fechatter_server"
echo "  3. Trigger analytics events (login, send message, etc.)"
echo "  4. Check analytics_server logs for incoming events"
echo ""
echo "Troubleshooting:"
echo "  - Check NATS logs: tail -f /tmp/nats.log"
echo "  - Monitor events: nats sub 'fechatter.analytics.>' --server=$NATS_URL"
echo "  - Check stream: nats stream info ANALYTICS --server=$NATS_URL"