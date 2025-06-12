/**
 * 🎯 Message State Manager - Complete Message Lifecycle Management
 * 
 * Provides unified message state management with complete closed-loop logic
 * Handles message lifecycle: DRAFT → QUEUED → SENDING → SENT → DELIVERED → READ
 */

import { ref, reactive, computed } from 'vue';
import { v4 as uuidv4 } from 'uuid';

/**
 * Message state enumeration
 */
export const MessageState = {
  DRAFT: 'draft',           // Message being composed
  QUEUED: 'queued',         // Queued for sending (offline or rate limited)
  SENDING: 'sending',       // Currently being sent to server
  SENT: 'sent',             // Sent to server, awaiting delivery confirmation
  DELIVERED: 'delivered',   // Delivered to recipients via SSE
  READ: 'read',             // Read by recipients
  FAILED: 'failed',         // Failed to send (retryable)
  TIMEOUT: 'timeout',       // Send timeout (retryable)
  REJECTED: 'rejected'      // Permanently rejected (not retryable)
};

/**
 * Message priority levels
 */
export const MessagePriority = {
  LOW: 1,
  NORMAL: 2,
  HIGH: 3,
  CRITICAL: 4
};

/**
 * Message metadata structure
 */
class MessageMetadata {
  constructor(options = {}) {
    this.clientId = options.clientId || uuidv4();           // Unique client-side ID
    this.tempId = `temp_${this.clientId}`;                  // Temporary ID for optimistic updates
    this.serverId = options.serverId || null;              // Server-assigned ID
    this.idempotencyKey = options.idempotencyKey || uuidv4(); // Idempotency key for server
    this.version = options.version || 1;                   // Message version for conflict resolution
    this.state = options.state || MessageState.DRAFT;      // Current state
    this.priority = options.priority || MessagePriority.NORMAL; // Message priority
    this.retryCount = options.retryCount || 0;             // Number of retry attempts
    this.maxRetries = options.maxRetries || 3;             // Maximum retry attempts
    this.createdAt = options.createdAt || new Date().toISOString();
    this.updatedAt = options.updatedAt || new Date().toISOString();
    this.sentAt = options.sentAt || null;                  // When sent to server
    this.deliveredAt = options.deliveredAt || null;        // When delivered via SSE
    this.readAt = options.readAt || null;                  // When read by recipient
    this.errorInfo = options.errorInfo || null;            // Error information if failed
    this.timeout = options.timeout || null;                // Timeout handle
  }

  /**
   * Update metadata state
   */
  updateState(newState, additionalData = {}) {
    this.state = newState;
    this.updatedAt = new Date().toISOString();

    // Update specific timestamps based on state
    switch (newState) {
      case MessageState.SENT:
        this.sentAt = additionalData.sentAt || new Date().toISOString();
        break;
      case MessageState.DELIVERED:
        this.deliveredAt = additionalData.deliveredAt || new Date().toISOString();
        break;
      case MessageState.READ:
        this.readAt = additionalData.readAt || new Date().toISOString();
        break;
      case MessageState.FAILED:
      case MessageState.TIMEOUT:
        this.errorInfo = additionalData.errorInfo || null;
        this.retryCount++;
        break;
    }
  }

  /**
   * Check if message can be retried
   */
  canRetry() {
    return (this.state === MessageState.FAILED || this.state === MessageState.TIMEOUT) &&
      this.retryCount < this.maxRetries;
  }

  /**
   * Check if message is in terminal state
   */
  isTerminal() {
    return [MessageState.DELIVERED, MessageState.READ, MessageState.REJECTED].includes(this.state);
  }

  /**
   * Check if message is pending
   */
  isPending() {
    return [MessageState.QUEUED, MessageState.SENDING, MessageState.SENT].includes(this.state);
  }
}

/**
 * Complete Message structure with metadata
 */
class CompleteMessage {
  constructor(content, chatId, senderId, options = {}) {
    this.content = content;
    this.chatId = chatId;
    this.senderId = senderId;
    this.files = options.files || [];
    this.metadata = new MessageMetadata(options.metadata || {});

    // Additional message properties
    this.sender = options.sender || null;
    this.mentions = options.mentions || [];
    this.replyTo = options.replyTo || null;
    this.edited = options.edited || false;
    this.editedAt = options.editedAt || null;
  }

  /**
   * Get unique identifier (prefer server ID, fallback to client ID)
   */
  get id() {
    return this.metadata.serverId || this.metadata.tempId;
  }

  /**
   * Get display timestamp
   */
  get createdAt() {
    return this.metadata.createdAt;
  }

  /**
   * Update message state
   */
  updateState(newState, additionalData = {}) {
    this.metadata.updateState(newState, additionalData);
  }

  /**
   * Convert to API-compatible format
   */
  toApiFormat() {
    return {
      content: this.content,
      files: this.files,
      idempotency_key: this.metadata.idempotencyKey,
      client_id: this.metadata.clientId,
      reply_to: this.replyTo
    };
  }

  /**
   * Convert to store format for display
   */
  toStoreFormat() {
    return {
      id: this.id,
      temp_id: this.metadata.tempId,
      chat_id: this.chatId,
      sender_id: this.senderId,
      sender: this.sender,
      content: this.content,
      files: this.files,
      created_at: this.metadata.createdAt,
      state: this.metadata.state,
      sending: this.metadata.state === MessageState.SENDING,
      error: this.metadata.state === MessageState.FAILED,
      canRetry: this.metadata.canRetry(),
      mentions: this.mentions,
      reply_to: this.replyTo,
      edited: this.edited,
      edited_at: this.editedAt
    };
  }
}

/**
 * Message State Manager - Core state management class
 */
export class MessageStateManager {
  constructor() {
    // Message storage by client ID
    this.messages = reactive(new Map());

    // Index mappings for fast lookup
    this.serverIdIndex = reactive(new Map());     // serverId -> clientId
    this.chatIndex = reactive(new Map());         // chatId -> Set of clientIds
    this.stateIndex = reactive(new Map());        // state -> Set of clientIds

    // Pending operations
    this.pendingOperations = reactive(new Map()); // clientId -> operation info

    // Statistics
    this.stats = reactive({
      total: 0,
      byState: new Map(),
      byChat: new Map()
    });

    // Configuration
    this.config = {
      maxRetries: 3,
      timeoutMs: 30000,           // 30 seconds timeout
      maxPendingMessages: 100,    // Maximum pending messages per chat
      cleanupIntervalMs: 300000   // 5 minutes cleanup interval
    };

    // Start periodic cleanup
    this.startCleanupTimer();
  }

  /**
   * Create a new message
   */
  createMessage(content, chatId, senderId, options = {}) {
    const message = new CompleteMessage(content, chatId, senderId, options);
    const clientId = message.metadata.clientId;

    // Store message
    this.messages.set(clientId, message);

    // Update indexes
    this.updateIndexes(message);

    // Update statistics
    this.updateStats(message, 'add');

    console.log(`📝 Created message: ${clientId} (chat: ${chatId})`);
    return message;
  }

  /**
   * Update message state with complete validation
   */
  updateMessageState(clientId, newState, additionalData = {}) {
    const message = this.messages.get(clientId);
    if (!message) {
      console.warn(`❌ Message not found: ${clientId}`);
      return false;
    }

    const oldState = message.metadata.state;

    // Validate state transition
    if (!this.isValidStateTransition(oldState, newState)) {
      console.warn(`❌ Invalid state transition: ${oldState} → ${newState} for message ${clientId}`);
      return false;
    }

    // Update state
    message.updateState(newState, additionalData);

    // Update indexes
    this.updateIndexes(message, oldState);

    // Update statistics
    this.updateStats(message, 'update', oldState);

    // Handle state-specific logic
    this.handleStateChange(message, oldState, newState);

    console.log(`🔄 Updated message state: ${clientId} (${oldState} → ${newState})`);
    return true;
  }

  /**
   * Validate state transition
   */
  isValidStateTransition(fromState, toState) {
    const validTransitions = {
      [MessageState.DRAFT]: [MessageState.QUEUED, MessageState.SENDING],
      [MessageState.QUEUED]: [MessageState.SENDING, MessageState.FAILED],
      [MessageState.SENDING]: [MessageState.SENT, MessageState.FAILED, MessageState.TIMEOUT],
      [MessageState.SENT]: [MessageState.DELIVERED, MessageState.FAILED, MessageState.TIMEOUT],
      [MessageState.DELIVERED]: [MessageState.READ],
      [MessageState.FAILED]: [MessageState.QUEUED, MessageState.SENDING, MessageState.REJECTED],
      [MessageState.TIMEOUT]: [MessageState.QUEUED, MessageState.SENDING, MessageState.REJECTED],
      [MessageState.READ]: [], // Terminal state
      [MessageState.REJECTED]: [] // Terminal state
    };

    return validTransitions[fromState]?.includes(toState) || false;
  }

  /**
   * Handle state change logic
   */
  handleStateChange(message, oldState, newState) {
    const clientId = message.metadata.clientId;

    // Clear timeout when message reaches terminal state
    if (message.metadata.isTerminal() && message.metadata.timeout) {
      clearTimeout(message.metadata.timeout);
      message.metadata.timeout = null;
    }

    // Set timeout for sending state
    if (newState === MessageState.SENDING) {
      message.metadata.timeout = setTimeout(() => {
        this.updateMessageState(clientId, MessageState.TIMEOUT, {
          errorInfo: { reason: 'Send timeout', timestamp: new Date().toISOString() }
        });
      }, this.config.timeoutMs);
    }

    // Remove from pending operations when completed or failed
    if (message.metadata.isTerminal() || newState === MessageState.FAILED) {
      this.pendingOperations.delete(clientId);
    }
  }

  /**
   * Get message by client ID
   */
  getMessage(clientId) {
    return this.messages.get(clientId);
  }

  /**
   * Get message by server ID
   */
  getMessageByServerId(serverId) {
    const clientId = this.serverIdIndex.get(serverId);
    return clientId ? this.messages.get(clientId) : null;
  }

  /**
   * Get messages for a chat
   */
  getMessagesForChat(chatId, states = null) {
    const clientIds = this.chatIndex.get(chatId) || new Set();
    const messages = [];

    for (const clientId of clientIds) {
      const message = this.messages.get(clientId);
      if (message && (!states || states.includes(message.metadata.state))) {
        messages.push(message);
      }
    }

    // Sort by creation time
    return messages.sort((a, b) =>
      new Date(a.metadata.createdAt) - new Date(b.metadata.createdAt)
    );
  }

  /**
   * Get messages by state
   */
  getMessagesByState(state) {
    const clientIds = this.stateIndex.get(state) || new Set();
    const messages = [];

    for (const clientId of clientIds) {
      const message = this.messages.get(clientId);
      if (message) {
        messages.push(message);
      }
    }

    return messages;
  }

  /**
   * Update message with server response
   */
  updateFromServerResponse(clientId, serverResponse) {
    const message = this.messages.get(clientId);
    if (!message) {
      console.warn(`❌ Message not found for server response: ${clientId}`);
      return false;
    }

    // Update server ID
    if (serverResponse.id && !message.metadata.serverId) {
      message.metadata.serverId = serverResponse.id;
      this.serverIdIndex.set(serverResponse.id, clientId);
    }

    // Update other server-provided data
    if (serverResponse.created_at) {
      message.metadata.createdAt = serverResponse.created_at;
    }

    // Update state to sent
    this.updateMessageState(clientId, MessageState.SENT, {
      sentAt: serverResponse.created_at || new Date().toISOString()
    });

    console.log(`📨 Updated message from server: ${clientId} → ${serverResponse.id}`);
    return true;
  }

  /**
   * Match incoming SSE message with pending message
   */
  matchSSEMessage(sseMessage) {
    // Try to match by server ID first
    if (sseMessage.id) {
      const message = this.getMessageByServerId(sseMessage.id);
      if (message) {
        this.updateMessageState(message.metadata.clientId, MessageState.DELIVERED, {
          deliveredAt: sseMessage.created_at || new Date().toISOString()
        });
        return message;
      }
    }

    // Try to match by content and sender for recent messages
    const recentMessages = this.getMessagesByState(MessageState.SENT);
    const now = Date.now();

    for (const message of recentMessages) {
      // Match within last 60 seconds
      const messageTime = new Date(message.metadata.sentAt || message.metadata.createdAt).getTime();
      if (now - messageTime > 60000) continue;

      // Match by content and sender
      if (message.content === sseMessage.content &&
        message.senderId === sseMessage.sender_id &&
        message.chatId === sseMessage.chat_id) {

        // Update with server ID
        if (sseMessage.id) {
          message.metadata.serverId = sseMessage.id;
          this.serverIdIndex.set(sseMessage.id, message.metadata.clientId);
        }

        this.updateMessageState(message.metadata.clientId, MessageState.DELIVERED, {
          deliveredAt: sseMessage.created_at || new Date().toISOString()
        });

        console.log(`🔗 Matched SSE message: ${message.metadata.clientId} ↔ ${sseMessage.id}`);
        return message;
      }
    }

    return null;
  }

  /**
   * Remove message
   */
  removeMessage(clientId) {
    const message = this.messages.get(clientId);
    if (!message) return false;

    // Clear timeout
    if (message.metadata.timeout) {
      clearTimeout(message.metadata.timeout);
    }

    // Remove from indexes
    this.removeFromIndexes(message);

    // Update statistics
    this.updateStats(message, 'remove');

    // Remove from storage
    this.messages.delete(clientId);

    console.log(`🗑️ Removed message: ${clientId}`);
    return true;
  }

  /**
   * Update indexes
   */
  updateIndexes(message, oldState = null) {
    const clientId = message.metadata.clientId;
    const { chatId, state, serverId } = {
      chatId: message.chatId,
      state: message.metadata.state,
      serverId: message.metadata.serverId
    };

    // Update chat index
    if (!this.chatIndex.has(chatId)) {
      this.chatIndex.set(chatId, new Set());
    }
    this.chatIndex.get(chatId).add(clientId);

    // Update state index
    if (oldState && this.stateIndex.has(oldState)) {
      this.stateIndex.get(oldState).delete(clientId);
    }
    if (!this.stateIndex.has(state)) {
      this.stateIndex.set(state, new Set());
    }
    this.stateIndex.get(state).add(clientId);

    // Update server ID index
    if (serverId) {
      this.serverIdIndex.set(serverId, clientId);
    }
  }

  /**
   * Remove from indexes
   */
  removeFromIndexes(message) {
    const clientId = message.metadata.clientId;
    const { chatId, state, serverId } = {
      chatId: message.chatId,
      state: message.metadata.state,
      serverId: message.metadata.serverId
    };

    // Remove from chat index
    if (this.chatIndex.has(chatId)) {
      this.chatIndex.get(chatId).delete(clientId);
    }

    // Remove from state index
    if (this.stateIndex.has(state)) {
      this.stateIndex.get(state).delete(clientId);
    }

    // Remove from server ID index
    if (serverId) {
      this.serverIdIndex.delete(serverId);
    }
  }

  /**
   * Update statistics
   */
  updateStats(message, operation, oldState = null) {
    if (operation === 'add') {
      this.stats.total++;
      this.updateStatCount(this.stats.byState, message.metadata.state, 1);
      this.updateStatCount(this.stats.byChat, message.chatId, 1);
    } else if (operation === 'remove') {
      this.stats.total--;
      this.updateStatCount(this.stats.byState, message.metadata.state, -1);
      this.updateStatCount(this.stats.byChat, message.chatId, -1);
    } else if (operation === 'update' && oldState) {
      this.updateStatCount(this.stats.byState, oldState, -1);
      this.updateStatCount(this.stats.byState, message.metadata.state, 1);
    }
  }

  /**
   * Update stat count helper
   */
  updateStatCount(statMap, key, delta) {
    const current = statMap.get(key) || 0;
    const newValue = current + delta;
    if (newValue <= 0) {
      statMap.delete(key);
    } else {
      statMap.set(key, newValue);
    }
  }

  /**
   * Get statistics
   */
  getStats() {
    return {
      total: this.stats.total,
      byState: Object.fromEntries(this.stats.byState),
      byChat: Object.fromEntries(this.stats.byChat),
      pendingOperations: this.pendingOperations.size
    };
  }

  /**
   * Cleanup old completed messages
   */
  cleanup() {
    const now = Date.now();
    const maxAge = 24 * 60 * 60 * 1000; // 24 hours
    const toRemove = [];

    for (const [clientId, message] of this.messages) {
      const messageAge = now - new Date(message.metadata.updatedAt).getTime();

      // Remove old completed messages
      if (message.metadata.isTerminal() && messageAge > maxAge) {
        toRemove.push(clientId);
      }
    }

    toRemove.forEach(clientId => this.removeMessage(clientId));

    if (toRemove.length > 0) {
      console.log(`🧹 Cleaned up ${toRemove.length} old messages`);
    }
  }

  /**
   * Start cleanup timer
   */
  startCleanupTimer() {
    setInterval(() => {
      this.cleanup();
    }, this.config.cleanupIntervalMs);
  }

  /**
   * Get configuration
   */
  getConfig() {
    return { ...this.config };
  }

  /**
   * Update configuration
   */
  updateConfig(newConfig) {
    Object.assign(this.config, newConfig);
  }
}

// Create global instance
export const messageStateManager = new MessageStateManager();

// Export for testing
export { CompleteMessage, MessageMetadata }; 