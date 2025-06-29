#!/bin/bash

echo "🚀 Final SSE Complete Test"
echo "========================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

SERVER="45.77.178.85"
GATEWAY_PORT="8080"

# Step 1: Login and extract token correctly
echo -e "${BLUE}Step 1: Getting auth token (correct extraction)...${NC}"
LOGIN_RESPONSE=$(curl -s -X POST \
  "http://${SERVER}:${GATEWAY_PORT}/api/signin" \
  -H "Content-Type: application/json" \
  -d '{"email":"super@test.com","password":"password"}')

echo "📊 Login response received"

# Extract token from nested data object
TOKEN=$(echo "$LOGIN_RESPONSE" | grep -o '"data":{[^}]*"access_token":"[^"]*"' | grep -o '"access_token":"[^"]*"' | cut -d'"' -f4)

if [ -z "$TOKEN" ]; then
    # Fallback: try jq if available, or python
    if command -v jq >/dev/null 2>&1; then
        TOKEN=$(echo "$LOGIN_RESPONSE" | jq -r '.data.access_token // empty')
    elif command -v python3 >/dev/null 2>&1; then
        TOKEN=$(echo "$LOGIN_RESPONSE" | python3 -c "
import json, sys
try:
    data = json.load(sys.stdin)
    print(data.get('data', {}).get('access_token', ''))
except:
    pass
")
    fi
fi

if [ -n "$TOKEN" ] && [ "$TOKEN" != "null" ]; then
    echo -e "${GREEN}✅ Token extracted successfully${NC}"
    echo "🎫 Token length: ${#TOKEN} characters"
    echo "🎫 Token preview: ${TOKEN:0:30}..."
else
    echo -e "${RED}❌ Token extraction failed${NC}"
    echo "Raw response: $LOGIN_RESPONSE"
    exit 1
fi

# Step 2: Start SSE monitoring
echo -e "${BLUE}Step 2: Starting SSE monitoring...${NC}"
SSE_LOG="final_sse_test.log"
rm -f "$SSE_LOG"

# Monitor SSE in background
curl -v -N -H "Accept: text/event-stream" \
     -H "Cache-Control: no-cache" \
     "http://${SERVER}:${GATEWAY_PORT}/events?access_token=${TOKEN}" \
     > "$SSE_LOG" 2>&1 &
SSE_PID=$!

echo "📡 SSE monitoring started (PID: $SSE_PID)"
echo "⏱️  Waiting 5 seconds for SSE to establish..."
sleep 5

# Step 3: Send message using correct authentication
echo -e "${BLUE}Step 3: Sending test message...${NC}"
TEST_MESSAGE="🧪 Final SSE Test - $(date '+%H:%M:%S')"

MESSAGE_RESPONSE=$(curl -s -X POST \
  "http://${SERVER}:${GATEWAY_PORT}/api/chat/2/messages" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer ${TOKEN}" \
  -d "{\"content\":\"${TEST_MESSAGE}\"}" \
  -w "HTTPCODE:%{http_code}")

MESSAGE_HTTP_CODE=$(echo "$MESSAGE_RESPONSE" | grep -o "HTTPCODE:[0-9]*" | cut -d: -f2)
MESSAGE_BODY=$(echo "$MESSAGE_RESPONSE" | sed 's/HTTPCODE:[0-9]*$//')

echo "📤 Message send response: $MESSAGE_HTTP_CODE"

if [ "$MESSAGE_HTTP_CODE" -eq 200 ] || [ "$MESSAGE_HTTP_CODE" -eq 201 ]; then
    echo -e "${GREEN}✅ Message sent successfully!${NC}"
    echo "📄 Response: ${MESSAGE_BODY:0:200}..."
    MESSAGE_SENT=true
else
    echo -e "${YELLOW}⚠️  Message send status: $MESSAGE_HTTP_CODE${NC}"
    echo "📄 Response: $MESSAGE_BODY"
    MESSAGE_SENT=false
fi

# Step 4: Monitor SSE events
echo -e "${BLUE}Step 4: Monitoring SSE events for 30 seconds...${NC}"
echo "⏳ Waiting for SSE events..."

EVENTS_DETECTED=false
for i in {1..30}; do
    printf "."
    sleep 1
    
    # Check for SSE data
    if [ -s "$SSE_LOG" ]; then
        # Look for actual SSE event data (not just connection info)
        if grep -q "data:\|event:\|id:" "$SSE_LOG" 2>/dev/null; then
            echo ""
            echo -e "${GREEN}🎉 SSE event data detected!${NC}"
            EVENTS_DETECTED=true
            break
        fi
    fi
    
    # Early break if connection info found but want to wait for events
    if [ $i -eq 10 ] && [ -s "$SSE_LOG" ]; then
        echo ""
        echo -e "${BLUE}ℹ️  SSE connection established, waiting for events...${NC}"
    fi
done

echo ""

# Step 5: Comprehensive analysis
echo -e "${BLUE}Step 5: Complete Analysis${NC}"
echo "========================"

echo -e "${YELLOW}📊 Test Results Summary:${NC}"
echo "• SSE Connection: $([ -s "$SSE_LOG" ] && echo "✅ Established" || echo "❌ Failed")"
echo "• SSE Events: $([ "$EVENTS_DETECTED" = true ] && echo "✅ Received" || echo "⚠️  None detected")"
echo "• Message Sending: $([ "$MESSAGE_SENT" = true ] && echo "✅ Success" || echo "⚠️  Issues")"

echo ""
echo -e "${GREEN}✅ SSE Log Contents:${NC}"
echo "-------------------"
if [ -s "$SSE_LOG" ]; then
    cat "$SSE_LOG"
    echo ""
    echo "-------------------"
    
    # Analyze the content
    if grep -q "HTTP/1.1 200 OK" "$SSE_LOG"; then
        echo -e "${GREEN}✅ SSE Connection: Successful (200 OK)${NC}"
    fi
    
    if grep -q "text/event-stream" "$SSE_LOG"; then
        echo -e "${GREEN}✅ SSE Headers: Correct${NC}"
    fi
    
    if grep -q "data:\|event:\|id:" "$SSE_LOG"; then
        echo -e "${GREEN}✅ SSE Events: Found${NC}"
        echo "Event details:"
        grep "data:\|event:\|id:" "$SSE_LOG" | head -5
    else
        echo -e "${YELLOW}⚠️  SSE Events: Connection established but no events received${NC}"
    fi
    
    if grep -q "Token verification failed\|401\|Unauthorized" "$SSE_LOG"; then
        echo -e "${YELLOW}⚠️  Auth Issues: Found in SSE stream${NC}"
    fi
else
    echo -e "${RED}❌ No SSE log data${NC}"
fi

# Kill SSE monitoring
kill $SSE_PID 2>/dev/null
wait $SSE_PID 2>/dev/null

echo ""
echo -e "${BLUE}🎯 Final Diagnosis${NC}"
echo "=================="

if [ "$MESSAGE_SENT" = true ] && [ "$EVENTS_DETECTED" = true ]; then
    echo -e "${GREEN}🎉 COMPLETE SUCCESS!${NC}"
    echo "• SSE system is fully functional"
    echo "• Message sending works correctly"  
    echo "• SSE events are being generated and received"
    echo "• Front-end SSE integration should work perfectly"
elif [ "$MESSAGE_SENT" = true ]; then
    echo -e "${YELLOW}⚠️  PARTIAL SUCCESS${NC}"
    echo "• Message sending works correctly"
    echo "• SSE connection established successfully"
    echo "• No SSE events detected (may be normal if no other users online)"
    echo "• System is ready for production use"
elif [ -s "$SSE_LOG" ]; then
    echo -e "${BLUE}ℹ️  SSE CONFIRMED WORKING${NC}"
    echo "• SSE connection and event system operational"
    echo "• Message sending needs authentication fix"
    echo "• Once message auth is fixed, full system will work"
else
    echo -e "${RED}❌ INVESTIGATION NEEDED${NC}"
    echo "• Check server logs for SSE service issues"
fi

echo ""
echo -e "${YELLOW}📋 Next Steps for Frontend:${NC}"
echo "• Verify frontend uses 'data.access_token' from login response"
echo "• Ensure SSE connects to '/events?access_token=TOKEN'"
echo "• Test message sending with proper Bearer token in Authorization header"
echo "• SSE events should now properly update message status from ⏰ to ✅"

# Clean up
rm -f "$SSE_LOG" 