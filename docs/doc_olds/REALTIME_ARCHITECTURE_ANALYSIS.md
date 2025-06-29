# Realtime Architecture Analysis

## Current Problems

1. **Unclear State Ownership**: Who maintains typing state? Chat member lists? Read receipts?
2. **Redundant Network Hops**: fechatter-server → NATS → notify-server → WebSocket
3. **Complex Event Flow**: Too many event types without clear purpose

## Key Questions

### 1. What is notify-server's actual responsibility?

**Option A: Pure Connection Manager (Recommended)**
- Manages WebSocket/SSE connections only
- No business logic or state
- Just broadcasts what it receives
- fechatter-server owns ALL state

**Option B: Stateful Notification Service**
- Maintains ephemeral state (typing, presence)
- Handles notification routing logic
- More complex but potentially more efficient

### 2. What realtime events actually need the dual-server architecture?

Let's categorize:

#### Direct WebSocket (fechatter-server → client)
These could go directly if we embed WebSocket in fechatter-server:
- Typing indicators
- Read receipts
- User presence
- Message delivery confirmations

#### Via notify-server (Justified)
These benefit from dedicated notification service:
- Cross-device notifications
- Offline message queue
- Email/Push notification triggers
- Webhook deliveries
- Multi-channel orchestration

## Recommended Architecture

### 1. Simplify notify-server to Pure SSE/WebSocket Broker

```
fechatter-server (Business Logic + State)
    ↓
NATS (Event Bus)
    ↓
notify-server (Connection Management Only)
    ↓
WebSocket/SSE Clients
```

### 2. State Management Rules

**fechatter-server maintains ALL state:**
- Who's in each chat
- Who's typing (with TTL)
- Read receipts
- User presence

**notify-server is stateless:**
- Receives events from NATS
- Looks up user_id → WebSocket connections
- Broadcasts to connected clients
- No business logic

### 3. Simplified Event Flow

```rust
// In fechatter-server
pub struct RealtimeEvent {
    target_users: Vec<i64>,  // Who should receive this
    event_type: String,      // "typing", "message", "presence"
    payload: Value,          // The actual data
}

// In notify-server
// Just receive and broadcast, no logic
async fn handle_realtime_event(event: RealtimeEvent) {
    for user_id in event.target_users {
        if let Some(connection) = connections.get(user_id) {
            connection.send(event.payload).await;
        }
    }
}
```

## Implementation Changes Needed

### 1. Typing Indicator Flow

**Current (Complex):**
```
Client → REST API → fechatter-server → NATS → notify-server → decide who gets it → broadcast
```

**Proposed (Simple):**
```
Client → REST API → fechatter-server (maintains typing state) → NATS with target_users → notify-server (just broadcast)
```

### 2. Handler Alignment

Current handlers need updates:

#### In fechatter-server:
```rust
// POST /chats/{id}/typing
async fn start_typing(chat_id, user_id) {
    // 1. Update typing state with TTL
    typing_state.set(chat_id, user_id, TTL=5s);
    
    // 2. Get chat members
    let members = get_chat_members(chat_id);
    
    // 3. Send to notify-server with explicit targets
    publish_event(RealtimeEvent {
        target_users: members.exclude(user_id),
        event_type: "typing_started",
        payload: json!({
            "chat_id": chat_id,
            "user_id": user_id,
            "user_name": get_user_name(user_id)
        })
    });
}
```

#### In notify-server:
```rust
// Just broadcast, no logic
async fn process_realtime_event(event: RealtimeEvent) {
    for user_id in event.target_users {
        if let Some(tx) = websocket_connections.get(&user_id) {
            let _ = tx.send(Event::from(event.payload)).await;
        }
    }
}
```

## Benefits of This Approach

1. **Clear Separation**: Business logic in one place
2. **Simpler Testing**: Stateless notify-server is easier to test
3. **Better Scaling**: Can scale connection layer independently
4. **Reduced Complexity**: No state synchronization issues

## Alternative: Embed WebSocket in fechatter-server

If we don't need independent scaling, we could:
1. Add WebSocket endpoint directly to fechatter-server
2. Remove notify-server entirely
3. Use in-memory pubsub for realtime events

This is simpler but less scalable.

## Decision Criteria

Keep notify-server if:
- Need to scale WebSocket connections independently
- Want fault isolation
- Plan to add more notification channels (SMS, Push, etc.)
- Need geographic distribution of connection servers

Embed WebSocket in fechatter-server if:
- Simplicity is priority
- Single server is sufficient
- Don't need notification orchestration

## Recommended Next Steps

1. **Decide on architecture**: Separate or embedded
2. **If keeping notify-server**: Simplify to stateless broker
3. **Update event structures**: Include target_users explicitly
4. **Move all state to fechatter-server**: Including typing, presence
5. **Simplify notify-server**: Remove business logic
6. **Add missing endpoints**: 
   - POST /chats/{id}/typing
   - DELETE /chats/{id}/typing
   - POST /messages/{id}/read
   - PUT /users/{id}/presence