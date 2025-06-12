# Message Flow Analysis

## 🔍 Complete Message Flow (DAG)

### Frontend → Backend Flow
```
User types message
    ↓
Chat.vue calls sendMessage()
    ↓
chatStore.sendMessage() [chat.js:1233]
    ├─→ Creates optimistic message (temp_id)
    ├─→ Adds to UI immediately
    ├─→ Tracks in pendingOptimisticMessages
    └─→ Sends API request
         ↓
    POST /api/chat/{id}/messages (JSON)
         ↓
    Backend processes message
    ├─→ Saves to PostgreSQL
    ├─→ Publishes to NATS
    └─→ Returns response
         ↓
    Frontend updates optimistic message
    └─→ Replaces temp_id with real id
```

### SSE Flow (Backend → Frontend)
```
Message saved in DB
    ↓
Backend publishes to NATS
    ↓
notify_server receives NATS message
    ↓
SSE broadcasts to all clients
    ↓
Frontend SSE handler receives
    ↓
chatStore.handleIncomingMessage()
    ├─→ Checks if own message (verifyMessageBroadcast)
    ├─→ Updates UI if new message
    └─→ Updates unread counts
```

## 🐛 Potential Issues

### 1. **Message Not Showing After Send**
- **Symptom**: Message sent but doesn't appear in UI
- **Possible Causes**:
  - API returned error (check Network tab)
  - Optimistic update failed
  - SSE not connected

### 2. **Duplicate Messages**
- **Symptom**: Same message appears twice
- **Possible Causes**:
  - SSE broadcasts own message back
  - Optimistic message not properly matched with server response
  - Multiple SSE connections

### 3. **Message Stuck in "Sending"**
- **Symptom**: Message shows sending status indefinitely
- **Possible Causes**:
  - API request failed silently
  - Network timeout
  - Backend error not properly handled

## 🔧 Debug Steps

### 1. Check Frontend State
```javascript
// In browser console:
window.debugMessage()
```

### 2. Check Network
- Open Network tab
- Look for POST to `/api/chat/{id}/messages`
- Check response status and body

### 3. Check SSE
- Look for EventSource connection to `/api/events`
- Check if receiving heartbeat events

### 4. Check Backend Logs
```bash
# Check server logs
tail -f server.log | grep -E "message|Message"

# Check NATS
docker logs fechatter_nats | tail -100

# Check database
docker exec -it fechatter_postgres psql -U postgres -d fechatter -c "SELECT * FROM messages ORDER BY created_at DESC LIMIT 5;"
```

## 📊 Message State Tracking

### Optimistic Message States
1. **sending** - Initial state when created
2. **sent** - API returned success
3. **confirmed** - SSE broadcast received
4. **failed** - API or network error

### Key Data Structures
```javascript
// pendingOptimisticMessages Map
{
  "temp_123456_0.789": {
    chatId: 1,
    content: "Hello world",
    timestamp: 1234567890,
    realId: 42 // Set after API response
  }
}

// Message object
{
  id: 42,              // Real ID from server
  temp_id: "temp_...", // Temporary ID (removed after confirmation)
  status: "sent",      // Message state
  content: "Hello",
  sender_id: 1,
  created_at: "2024-01-01T00:00:00Z"
}
```

## 🚨 Common Fixes

### 1. Message Not Sending
```javascript
// Check if using correct format
const payload = {
  content: "message text",
  files: [],  // Array of URLs, not File objects
  idempotency_key: null,
  reply_to: null,
  mentions: null
};
```

### 2. SSE Not Working
```javascript
// Check SSE connection
const sseService = await import('@/services/sse');
const state = sseService.default.getConnectionState();
console.log('SSE Connected:', state.isConnected);
```

### 3. Clear Stuck Messages
```javascript
// Clear pending messages
const chatStore = useChatStore();
chatStore.pendingOptimisticMessages.clear();
chatStore.cleanupExpiredOptimisticMessages();
``` 