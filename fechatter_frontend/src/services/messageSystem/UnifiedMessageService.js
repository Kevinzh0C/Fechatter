/**
 * 🎯 Unified Message Service - Complete Message System Integration
 * 
 * Provides a unified interface for all message operations with complete closed-loop logic
 * Integrates: State Management + Sync Queue + Offline Queue + SSE Integration
 */

import { ref, reactive, computed } from 'vue';
import { useAuthStore } from '@/stores/auth';
import { messageStateManager, MessageState, MessagePriority } from './MessageStateManager.js';

/**
 * Unified Message Service - Main service class
 */
export class UnifiedMessageService {
  constructor() {
    // Core dependencies (lazy loaded to avoid circular imports)
    this.syncQueue = null;
    this.offlineQueue = null;
    this.sseService = null;

    // Service state
    this.isInitialized = ref(false);
    this.isOnline = ref(navigator.onLine);

    // Statistics
    this.stats = reactive({
      totalMessages: 0,
      messagesSent: 0,
      messagesReceived: 0,
      messagesFailedToSend: 0,
      averageResponseTime: 0,
      lastActivity: null
    });

    // Configuration
    this.config = {
      enableOptimisticUpdates: true,
      enableOfflineQueue: true,
      maxRetries: 3,
      timeoutMs: 30000,
      batchSize: 20
    };

    // Initialize
    this.initialize();
  }

  /**
   * Initialize the unified message service
   */
  async initialize() {
    try {
      console.log('🎯 Initializing Unified Message Service...');

      // Lazy load dependencies to avoid circular imports
      const { messageSyncQueue } = await import('./MessageSyncQueue.js');
      this.syncQueue = messageSyncQueue;

      // Try to load offline queue (optional)
      try {
        const { messageOfflineQueue } = await import('./MessageOfflineQueue.js');
        this.offlineQueue = messageOfflineQueue;
      } catch (error) {
        console.warn('Offline queue not available:', error);
      }

      // Setup SSE integration
      this.setupSSEIntegration();

      // Setup network status monitoring
      this.setupNetworkMonitoring();

      this.isInitialized.value = true;
      console.log('✅ Unified Message Service initialized');

    } catch (error) {
      console.error('Failed to initialize Unified Message Service:', error);
      throw error;
    }
  }

  /**
   * Setup SSE integration for real-time message handling
   */
  async setupSSEIntegration() {
    try {
      // Dynamically import SSE service to avoid circular dependency
      const sseModule = await import('@/services/sse');
      this.sseService = sseModule.default;

      // Listen for incoming messages
      this.sseService.on('new_message', (sseMessage) => {
        this.handleIncomingSSEMessage(sseMessage);
      });

      console.log('🔗 SSE integration setup complete');

    } catch (error) {
      console.warn('SSE service not available:', error);
    }
  }

  /**
   * Setup network status monitoring
   */
  setupNetworkMonitoring() {
    if (typeof window !== 'undefined') {
      window.addEventListener('online', () => {
        this.isOnline.value = true;
        console.log('🌐 Network online: Message service ready');
      });

      window.addEventListener('offline', () => {
        this.isOnline.value = false;
        console.log('🌐 Network offline: Messages will be queued');
      });
    }
  }

  /**
   * Send a message with complete lifecycle management
   */
  async sendMessage(content, chatId, options = {}) {
    const authStore = useAuthStore();
    if (!authStore.user) {
      throw new Error('User not authenticated');
    }

    const startTime = Date.now();

    try {
      // Create message with complete metadata
      const message = messageStateManager.createMessage(
        content,
        parseInt(chatId),
        authStore.user.id,
        {
          files: options.files || [],
          priority: options.priority || MessagePriority.NORMAL,
          sender: {
            id: authStore.user.id,
            fullname: authStore.user.fullname || authStore.user.username,
            email: authStore.user.email
          },
          mentions: options.mentions || [],
          replyTo: options.replyTo || null
        }
      );

      console.log(`📝 Created message for sending: ${message.metadata.clientId}`);

      // Handle online vs offline scenarios
      if (this.isOnline.value) {
        // Online: Queue for immediate sending
        await this.syncQueue.queueSend(message, {
          priority: options.priority || MessagePriority.NORMAL
        });
      } else {
        // Offline: Queue for later sync
        if (this.offlineQueue) {
          await this.offlineQueue.queueOfflineMessage(message);
        } else {
          throw new Error('Network unavailable and offline queue not initialized');
        }
      }

      // Update statistics
      this.stats.messagesSent++;
      this.stats.totalMessages++;
      this.stats.lastActivity = new Date().toISOString();

      // Calculate response time (for successful queueing)
      const responseTime = Date.now() - startTime;
      this.updateAverageResponseTime(responseTime);

      return {
        clientId: message.metadata.clientId,
        tempId: message.metadata.tempId,
        state: message.metadata.state,
        message: message.toStoreFormat()
      };

    } catch (error) {
      console.error('Failed to send message:', error);
      this.stats.messagesFailedToSend++;

      // If message was created, update its state to failed
      if (error.clientId) {
        messageStateManager.updateMessageState(error.clientId, MessageState.FAILED, {
          errorInfo: { message: error.message, timestamp: new Date().toISOString() }
        });
      }

      throw error;
    }
  }

  /**
   * Handle incoming SSE message with proper matching and deduplication
   */
  handleIncomingSSEMessage(sseMessage) {
    console.log('📨 Received SSE message:', sseMessage.id);

    try {
      // Try to match with pending message (resolve optimistic update)
      const matchedMessage = messageStateManager.matchSSEMessage(sseMessage);

      if (matchedMessage) {
        console.log(`🔗 Matched SSE message with pending: ${matchedMessage.metadata.clientId}`);
        this.stats.messagesReceived++;
        return; // Message was matched and updated, no need to add as new
      }

      // Not a match for our pending messages - this is a new message from another user
      const authStore = useAuthStore();
      if (sseMessage.sender_id !== authStore.user?.id) {
        // Create new message for incoming message from others
        const incomingMessage = messageStateManager.createMessage(
          sseMessage.content,
          sseMessage.chat_id,
          sseMessage.sender_id,
          {
            files: sseMessage.files || [],
            metadata: {
              serverId: sseMessage.id,
              state: MessageState.DELIVERED,
              createdAt: sseMessage.created_at
            },
            sender: sseMessage.sender || {
              id: sseMessage.sender_id,
              fullname: sseMessage.sender_name || sseMessage.sender_fullname || 'Unknown User'
            }
          }
        );

        // Mark as delivered immediately since it came via SSE
        messageStateManager.updateMessageState(
          incomingMessage.metadata.clientId,
          MessageState.DELIVERED,
          { deliveredAt: sseMessage.created_at }
        );

        this.stats.messagesReceived++;
        this.stats.totalMessages++;

        console.log(`📨 Added new incoming message: ${incomingMessage.metadata.clientId}`);
      }

    } catch (error) {
      console.error('Error handling incoming SSE message:', error);
    }
  }

  /**
   * Retry failed message
   */
  async retryMessage(clientId) {
    const message = messageStateManager.getMessage(clientId);
    if (!message) {
      throw new Error(`Message not found: ${clientId}`);
    }

    if (!message.metadata.canRetry()) {
      throw new Error(`Message cannot be retried: ${clientId}`);
    }

    console.log(`🔄 Retrying message: ${clientId}`);

    try {
      if (this.isOnline.value) {
        await this.syncQueue.queueRetry(message);
      } else if (this.offlineQueue) {
        await this.offlineQueue.queueOfflineMessage(message);
      } else {
        throw new Error('Cannot retry: offline and no offline queue');
      }

      return true;
    } catch (error) {
      console.error(`Failed to retry message ${clientId}:`, error);
      throw error;
    }
  }

  /**
   * Get messages for a chat with proper state formatting
   */
  getMessagesForChat(chatId, states = null) {
    const messages = messageStateManager.getMessagesForChat(parseInt(chatId), states);

    // Convert to store format for display
    return messages.map(message => message.toStoreFormat());
  }

  /**
   * Get all messages in various states
   */
  getMessagesByState(state) {
    const messages = messageStateManager.getMessagesByState(state);
    return messages.map(message => message.toStoreFormat());
  }

  /**
   * Get pending messages (for UI indicators)
   */
  getPendingMessages() {
    const pendingStates = [MessageState.QUEUED, MessageState.SENDING, MessageState.SENT];
    const allPending = [];

    for (const state of pendingStates) {
      const messages = this.getMessagesByState(state);
      allPending.push(...messages);
    }

    return allPending;
  }

  /**
   * Get failed messages (for retry UI)
   */
  getFailedMessages() {
    return this.getMessagesByState(MessageState.FAILED);
  }

  /**
   * Update average response time
   */
  updateAverageResponseTime(newTime) {
    const alpha = 0.1; // Exponential moving average
    this.stats.averageResponseTime =
      this.stats.averageResponseTime * (1 - alpha) + newTime * alpha;
  }

  /**
   * Force sync all offline messages
   */
  async forceSyncOfflineMessages() {
    if (!this.offlineQueue) {
      throw new Error('Offline queue not available');
    }

    return await this.offlineQueue.forceSyncAll();
  }

  /**
   * Clear all messages (for testing/reset)
   */
  async clearAllMessages() {
    // Clear state manager
    const allMessages = messageStateManager.getMessagesByState();
    for (const message of allMessages) {
      messageStateManager.removeMessage(message.metadata.clientId);
    }

    // Clear offline queue if available
    if (this.offlineQueue) {
      await this.offlineQueue.clearAllOfflineMessages();
    }

    // Clear sync queue
    if (this.syncQueue) {
      this.syncQueue.clearQueue();
    }

    // Reset statistics
    Object.assign(this.stats, {
      totalMessages: 0,
      messagesSent: 0,
      messagesReceived: 0,
      messagesFailedToSend: 0,
      averageResponseTime: 0,
      lastActivity: null
    });

    console.log('🧹 All messages cleared');
  }

  /**
   * Get comprehensive service status
   */
  getStatus() {
    const stateManagerStats = messageStateManager.getStats();
    const syncQueueStatus = this.syncQueue?.getStatus() || {};
    const offlineQueueStatus = this.offlineQueue?.getStatus() || {};

    return {
      isInitialized: this.isInitialized.value,
      isOnline: this.isOnline.value,
      stats: { ...this.stats },
      config: { ...this.config },
      stateManager: stateManagerStats,
      syncQueue: syncQueueStatus,
      offlineQueue: offlineQueueStatus,
      services: {
        syncQueueAvailable: !!this.syncQueue,
        offlineQueueAvailable: !!this.offlineQueue,
        sseServiceAvailable: !!this.sseService
      }
    };
  }

  /**
   * Update service configuration
   */
  updateConfig(newConfig) {
    Object.assign(this.config, newConfig);

    // Propagate relevant config to sub-services
    if (this.syncQueue && newConfig.maxRetries) {
      this.syncQueue.updateConfig({ maxRetries: newConfig.maxRetries });
    }

    if (this.offlineQueue && newConfig.batchSize) {
      this.offlineQueue.updateConfig({ syncBatchSize: newConfig.batchSize });
    }

    console.log('📝 Updated unified message service configuration');
  }

  /**
   * Export debug information
   */
  async exportDebugInfo() {
    const status = this.getStatus();
    const pendingMessages = this.getPendingMessages();
    const failedMessages = this.getFailedMessages();

    let offlineMessages = [];
    if (this.offlineQueue) {
      offlineMessages = await this.offlineQueue.exportOfflineMessages();
    }

    return {
      timestamp: new Date().toISOString(),
      status,
      pendingMessages: pendingMessages.length,
      failedMessages: failedMessages.length,
      offlineMessages: offlineMessages.length,
      details: {
        pending: pendingMessages,
        failed: failedMessages,
        offline: offlineMessages
      }
    };
  }
}

// Create and export global instance
export const unifiedMessageService = new UnifiedMessageService();

// Export message states for external use
export { MessageState, MessagePriority }; 