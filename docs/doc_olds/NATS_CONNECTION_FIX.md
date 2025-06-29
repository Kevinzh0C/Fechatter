# NATS Connection Fix

## Problem
The notify_server was experiencing two types of errors:
1. "missed idle heartbeat" - Connection timeout issues
2. "no responders" - Subject routing mismatches

## Root Causes

### 1. Heartbeat Configuration Mismatch
- NATS connection ping_interval: 30s
- Consumer idle_heartbeat: 15s/30s
- This causes the consumer to timeout before the connection sends a ping

### 2. Subject Pattern Mismatches
The "no responders" error occurs when:
- Publisher sends to `fechatter.realtime.chat.123`
- But consumer expects different pattern or isn't properly set up

## Solutions Applied

### 1. Fixed Heartbeat Configuration
```yaml
# notify.yml
consumers:
  notification_processor:
    idle_heartbeat: "45s"  # Increased from 30s
  realtime_processor:
    idle_heartbeat: "45s"  # Increased from 15s
```

The idle_heartbeat should be longer than the connection ping_interval (30s) to prevent timeouts.

### 2. Subject Routing Architecture

#### Domain Events Stream (JetStream - Persistent)
- `fechatter.messages.>` - Message lifecycle events
- `fechatter.chats.>` - Chat management events
- Uses JetStream for guaranteed delivery

#### Realtime Stream (Core NATS - Non-persistent)
- `fechatter.realtime.chat.{chat_id}` - Chat-specific realtime events
- `fechatter.realtime.chat.{chat_id}.read` - Read receipts
- `fechatter.realtime.chat.{chat_id}.typing` - Typing indicators
- `fechatter.realtime.user.{user_id}.presence` - User presence

### 3. Publisher-Subscriber Alignment

The fechatter_server publishes to:
```rust
// In DualStreamDispatcher
match &event {
    RealtimeEvent::MessageReceived { chat_id, .. } => 
        format!("fechatter.realtime.chat.{}", chat_id),
    RealtimeEvent::MessageRead { chat_id, .. } => 
        format!("fechatter.realtime.chat.{}.read", chat_id),
    // etc...
}
```

The notify_server subscribes to:
```yaml
filter_subjects:
  - "fechatter.realtime.>"  # Matches all realtime events
```

## Additional Recommendations

### 1. Connection Resilience
```rust
// Better connection options
let connect_options = async_nats::ConnectOptions::new()
    .connection_timeout(Duration::from_secs(10))
    .ping_interval(Duration::from_secs(30))
    .max_reconnects(Some(10))
    .reconnect_delay_callback(|attempts| {
        Duration::from_secs(min(2u64.pow(attempts as u32), 30))
    });
```

### 2. Consumer Configuration
```rust
// Ensure proper consumer config
let consumer_config = jetstream::consumer::pull::Config {
    name: Some(consumer_name.to_string()),
    filter_subjects,
    ack_policy: jetstream::consumer::AckPolicy::Explicit,
    max_deliver: 3,
    ack_wait: Duration::from_secs(45),
    idle_heartbeat: Duration::from_secs(45),  // Match config file
    ..Default::default()
};
```

### 3. Error Handling
- Log connection state changes
- Implement exponential backoff for reconnects
- Monitor consumer lag and processing times
- Set up alerts for consecutive errors

## Testing the Fix

1. Restart notify_server with new configuration
2. Monitor logs for connection stability
3. Check that realtime events are properly delivered
4. Verify no more "missed idle heartbeat" errors
5. Confirm "no responders" errors are resolved

## Future Improvements

1. **Dynamic Configuration**: Allow runtime adjustment of heartbeat intervals
2. **Health Checks**: Add NATS connection health endpoints
3. **Metrics**: Track message latency and delivery rates
4. **Circuit Breaker**: Implement circuit breaker for degraded NATS connections