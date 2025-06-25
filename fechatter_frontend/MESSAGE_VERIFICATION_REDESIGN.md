# Message Verification Logic Redesign

## 🗄️ Database Schema Analysis

### Messages Table Structure
```sql
CREATE TABLE messages (
    id BIGSERIAL PRIMARY KEY,
    chat_id BIGINT NOT NULL,
    sender_id BIGINT NOT NULL,
    content TEXT,
    files TEXT[] DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    idempotency_key UUID UNIQUE,
    sequence_number BIGINT NOT NULL,
    -- Additional fields from migrations
    status VARCHAR,
    reply_to BIGINT,
    is_edited BOOLEAN DEFAULT FALSE
);
```

### Key Indexes
- `idx_message_idempotency` on `idempotency_key`
- `idx_messages_chat_sequence` on `(chat_id, sequence_number)`
- Unique constraint on `(chat_id, sequence_number)`

## 🔍 Current Problems

### 1. **Unreliable Message Matching**
- Current logic relies on content + time matching (unreliable)
- No use of `idempotency_key` for deduplication
- `sequence_number` not utilized for ordering

### 2. **Race Conditions**
- SSE broadcast may arrive before API response
- Multiple devices sending simultaneously
- No proper conflict resolution

### 3. **Missing Database Features**
- Not using `idempotency_key` for duplicate prevention
- Ignoring `sequence_number` for message ordering
- No tracking of message status transitions

## 🔧 Redesigned Verification Logic

### 1. **Use Idempotency Key**
```javascript
// Generate idempotency key on frontend
const idempotencyKey = crypto.randomUUID();

// Track with message
pendingMessages.set(idempotencyKey, {
  tempId,
  chatId,
  content,
  timestamp: Date.now()
});

// Send with request
const payload = {
  content,
  idempotency_key: idempotencyKey,
  // ... other fields
};
```

### 2. **Enhanced Message Verification**
```javascript
verifyMessageBroadcast(broadcastMessage) {
  const authStore = useAuthStore();
  
  // Only process own messages
  if (broadcastMessage.sender_id !== authStore.user?.id) {
    return false;
  }

  // Method 1: Match by idempotency_key (most reliable)
  if (broadcastMessage.idempotency_key) {
    for (const [key, pending] of this.pendingMessages.entries()) {
      if (key === broadcastMessage.idempotency_key) {
        // Found exact match
        this.confirmMessage(pending.tempId, broadcastMessage);
        this.pendingMessages.delete(key);
        return true;
      }
    }
  }

  // Method 2: Match by sequence_number for ordering
  if (broadcastMessage.sequence_number) {
    // Use sequence number to ensure proper ordering
    this.insertMessageBySequence(broadcastMessage);
    return true;
  }

  // Method 3: Fallback to current logic (last resort)
  return this.fallbackVerification(broadcastMessage);
}
```

### 3. **Message State Machine**
```javascript
// Message states
const MessageStatus = {
  PENDING: 'pending',      // Created locally, not sent
  SENDING: 'sending',      // API request in flight
  SENT: 'sent',           // API returned success
  CONFIRMED: 'confirmed',  // SSE broadcast received
  FAILED: 'failed',       // Send failed
  RETRY: 'retry'          // Retrying send
};

// State transitions
const messageStateMachine = {
  [MessageStatus.PENDING]: [MessageStatus.SENDING, MessageStatus.FAILED],
  [MessageStatus.SENDING]: [MessageStatus.SENT, MessageStatus.FAILED, MessageStatus.RETRY],
  [MessageStatus.SENT]: [MessageStatus.CONFIRMED],
  [MessageStatus.FAILED]: [MessageStatus.RETRY],
  [MessageStatus.RETRY]: [MessageStatus.SENDING, MessageStatus.FAILED],
  [MessageStatus.CONFIRMED]: [] // Final state
};
```

### 4. **Sequence-Based Ordering**
```javascript
insertMessageBySequence(message) {
  // Find insertion point based on sequence_number
  const insertIndex = this.messages.findIndex(m => 
    m.sequence_number > message.sequence_number
  );

  if (insertIndex === -1) {
    // Add to end
    this.messages.push(message);
  } else {
    // Insert at correct position
    this.messages.splice(insertIndex, 0, message);
  }

  // Check for sequence gaps
  this.checkSequenceGaps(message.chat_id);
}

checkSequenceGaps(chatId) {
  const chatMessages = this.messages.filter(m => m.chat_id === chatId);
  const sequences = chatMessages.map(m => m.sequence_number).sort((a, b) => a - b);
  
  for (let i = 1; i < sequences.length; i++) {
    if (sequences[i] - sequences[i-1] > 1) {
      // Gap detected, request missing messages
      this.requestMissingMessages(chatId, sequences[i-1] + 1, sequences[i] - 1);
    }
  }
}
```

### 5. **Improved Duplicate Detection**
```javascript
isDuplicateMessage(message) {
  // Check by ID (most reliable)
  if (this.messages.some(m => m.id === message.id)) {
    return true;
  }

  // Check by idempotency_key
  if (message.idempotency_key) {
    return this.messages.some(m => 
      m.idempotency_key === message.idempotency_key
    );
  }

  // Check by sequence_number (for same chat)
  if (message.sequence_number) {
    return this.messages.some(m => 
      m.chat_id === message.chat_id && 
      m.sequence_number === message.sequence_number
    );
  }

  return false;
}
```

## 📊 Implementation Plan

### Phase 1: Backend Updates
1. Ensure all messages have `idempotency_key`
2. Ensure all messages have `sequence_number`
3. Include these fields in API responses and SSE events

### Phase 2: Frontend Updates
1. Generate `idempotency_key` for all messages
2. Track messages by `idempotency_key` instead of `temp_id`
3. Implement sequence-based ordering
4. Add message state machine

### Phase 3: Testing & Monitoring
1. Add metrics for duplicate messages
2. Monitor sequence gaps
3. Track message state transitions
4. Performance testing with multiple devices

## 🎯 Benefits

1. **Reliability**: No more duplicate messages
2. **Ordering**: Messages always in correct order
3. **Performance**: Faster verification using indexes
4. **Debugging**: Clear state transitions
5. **Multi-device**: Proper handling of simultaneous sends

## 🚀 Migration Strategy

1. **Backward Compatibility**: Support both old and new logic
2. **Feature Flag**: Enable new logic gradually
3. **Monitoring**: Track success rates
4. **Rollback Plan**: Easy switch back to old logic 