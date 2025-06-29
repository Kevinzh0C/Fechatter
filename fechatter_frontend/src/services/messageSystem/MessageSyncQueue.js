/**
 * Message Sync Queue Manager - Production-grade Message Sending Queue
 * 
 * Provides reliable message sending with queue management, retry logic, and conflict resolution
 * Handles online/offline synchronization and ensures message delivery
 */

import { ref, reactive } from 'vue';
import { messageStateManager, MessageState, MessagePriority } from './MessageStateManager.js';
import api from '@/services/api';

/**
 * Queue operation types
 */
export const QueueOperation = {
  SEND: 'send',
  RETRY: 'retry',
  EDIT: 'edit',
  DELETE: 'delete'
};

/**
 * Queue item structure
 */
class QueueItem {
  constructor(operation, message, options = {}) {
    this.id = `queue_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
    this.operation = operation;
    this.message = message;
    this.priority = options.priority || MessagePriority.NORMAL;
    this.retryCount = options.retryCount || 0;
    this.maxRetries = options.maxRetries || 3;
    this.createdAt = new Date().toISOString();
    this.lastAttemptAt = options.lastAttemptAt || null;
    this.nextAttemptAt = options.nextAttemptAt || new Date().toISOString();
    this.backoffMultiplier = options.backoffMultiplier || 2;
    this.baseDelayMs = options.baseDelayMs || 1000;
    this.errorInfo = options.errorInfo || null;
    this.aborted = false;
  }

  /**
   * Calculate next retry delay
   */
  calculateNextDelay() {
    const exponentialDelay = this.baseDelayMs * Math.pow(this.backoffMultiplier, this.retryCount);
    const jitter = Math.random() * 1000; // Add jitter to prevent thundering herd
    return Math.min(exponentialDelay + jitter, 30000); // Max 30 seconds
  }

  /**
   * Update for next retry
   */
  prepareRetry(errorInfo = null) {
    this.retryCount++;
    this.lastAttemptAt = new Date().toISOString();
    this.errorInfo = errorInfo;

    const delay = this.calculateNextDelay();
    this.nextAttemptAt = new Date(Date.now() + delay).toISOString();
  }

  /**
   * Check if item can be retried
   */
  canRetry() {
    return !this.aborted && this.retryCount < this.maxRetries;
  }

/**
 * Message Sync Queue Manager
 */
export class MessageSyncQueue {
  constructor() {
    // Queue storage
    this.queue = reactive(new Map()); // queueId -> QueueItem
    this.processingQueue = reactive(new Set()); // Currently processing items

    // Priority queues for different operations
    this.priorityQueues = reactive({
      [MessagePriority.CRITICAL]: [],
      [MessagePriority.HIGH]: [],
      [MessagePriority.NORMAL]: [],
      [MessagePriority.LOW]: []
    });

    // Processing state
    this.isProcessing = ref(false);
    this.isPaused = ref(false);
    this.isOnline = ref(navigator.onLine);

    // Configuration
    this.config = {
      maxConcurrentOperations: 3,
      processingIntervalMs: 100,
      batchSize: 5,
      maxQueueSize: 1000,
      cleanupIntervalMs: 300000 // 5 minutes
    };

    // Statistics
    this.stats = reactive({
      totalQueued: 0,
      totalProcessed: 0,
      totalFailed: 0,
      totalRetried: 0,
      currentQueueSize: 0,
      averageProcessingTime: 0
    });

    // Bind network status handlers
    this.setupNetworkHandlers();

    // Start processing
    this.startProcessing();

    // Start cleanup timer
    this.startCleanupTimer();
  }

  /**
   * Setup network status handlers
   */
  setupNetworkHandlers() {
    if (typeof window !== 'undefined') {
      window.addEventListener('online', () => {
        this.isOnline.value = true;
        this.resume();
        // console.log('🌐 Network online: Resuming message queue processing');
      });

      window.addEventListener('offline', () => {
        this.isOnline.value = false;
        this.pause();
        // console.log('🌐 Network offline: Pausing message queue processing');
      });

  /**
   * Add message to send queue
   */
  async queueSend(message, options = {}) {
    if (this.queue.size >= this.config.maxQueueSize) {
      throw new Error('Message queue is full');
    }

    // Update message state to queued
    messageStateManager.updateMessageState(message.metadata.clientId, MessageState.QUEUED);

    // Create queue item
    const queueItem = new QueueItem(QueueOperation.SEND, message, {
      priority: options.priority || MessagePriority.NORMAL,
      maxRetries: options.maxRetries || 3
    });

    // Add to queue and priority queue
    this.queue.set(queueItem.id, queueItem);
    this.addToPriorityQueue(queueItem);

    this.stats.totalQueued++;
    this.stats.currentQueueSize = this.queue.size;

    // console.log(`📤 Queued message for sending: ${message.metadata.clientId}`);

    // Trigger immediate processing if online
    if (this.isOnline.value && !this.isPaused.value) {
      this.processNextBatch();
    }

    return queueItem.id;
  }

  /**
   * Add retry item to queue
   */
  async queueRetry(message, options = {}) {
    const queueItem = new QueueItem(QueueOperation.RETRY, message, {
      priority: MessagePriority.HIGH, // Retries get higher priority
      retryCount: options.retryCount || 0,
      maxRetries: options.maxRetries || 3,
      errorInfo: options.errorInfo
    });

    this.queue.set(queueItem.id, queueItem);
    this.addToPriorityQueue(queueItem);

    this.stats.totalRetried++;
    this.stats.currentQueueSize = this.queue.size;

    // console.log(`🔄 Queued message for retry: ${message.metadata.clientId} (attempt ${queueItem.retryCount + 1})`);

    return queueItem.id;
  }

  /**
   * Add to priority queue
   */
  addToPriorityQueue(queueItem) {
    this.priorityQueues[queueItem.priority].push(queueItem.id);
  }

  /**
   * Remove from priority queue
   */
  removeFromPriorityQueue(queueItem) {
    const queue = this.priorityQueues[queueItem.priority];
    const index = queue.indexOf(queueItem.id);
    if (index > -1) {
      queue.splice(index, 1);
    }

  /**
   * Get next items to process
   */
  getNextBatch() {
    const batch = [];
    const maxBatch = Math.min(
      this.config.batchSize,
      this.config.maxConcurrentOperations - this.processingQueue.size
    );

    if (maxBatch <= 0) return batch;

    // Process by priority: CRITICAL → HIGH → NORMAL → LOW
    const priorities = [MessagePriority.CRITICAL, MessagePriority.HIGH, MessagePriority.NORMAL, MessagePriority.LOW];

    for (const priority of priorities) {
      const queue = this.priorityQueues[priority];

      while (queue.length > 0 && batch.length < maxBatch) {
        const queueId = queue.shift();
        const queueItem = this.queue.get(queueId);

        if (!queueItem || queueItem.aborted) continue;

        // Check if it's time to process this item
        const now = new Date();
        const nextAttemptTime = new Date(queueItem.nextAttemptAt);

        if (now >= nextAttemptTime) {
          batch.push(queueItem);
          this.processingQueue.add(queueId);
        } else {
          // Put back in queue for later
          queue.push(queueId);
          break; // Stop processing this priority level
        }

    return batch;
  }

  /**
   * Process next batch of messages
   */
  async processNextBatch() {
    if (this.isPaused.value || !this.isOnline.value) {
      return;
    }

    const batch = this.getNextBatch();
    if (batch.length === 0) {
      return;
    }

    // console.log(`Processing batch of ${batch.length} messages`);

    // Process all items in batch concurrently
    const promises = batch.map(queueItem => this.processQueueItem(queueItem));
    await Promise.allSettled(promises);
  }

  /**
   * Process individual queue item
   */
  async processQueueItem(queueItem) {
    const startTime = Date.now();

    try {
      // Update message state to sending
      if (queueItem.operation === QueueOperation.SEND || queueItem.operation === QueueOperation.RETRY) {
        messageStateManager.updateMessageState(queueItem.message.metadata.clientId, MessageState.SENDING);
      }

      let result;
      switch (queueItem.operation) {
        case QueueOperation.SEND:
        case QueueOperation.RETRY:
          result = await this.sendMessage(queueItem.message);
          break;
        case QueueOperation.EDIT:
          result = await this.editMessage(queueItem.message);
          break;
        case QueueOperation.DELETE:
          result = await this.deleteMessage(queueItem.message);
          break;
        default:
          throw new Error(`Unknown queue operation: ${queueItem.operation}`);
      }

      // Handle successful result
      await this.handleProcessingSuccess(queueItem, result);

      // Update statistics
      this.stats.totalProcessed++;
      const processingTime = Date.now() - startTime;
      this.updateAverageProcessingTime(processingTime);

    } catch (error) {
      // Handle processing error
      await this.handleProcessingError(queueItem, error);
      this.stats.totalFailed++;
    } finally {
      // Clean up
      this.processingQueue.delete(queueItem.id);
      this.queue.delete(queueItem.id);
      this.stats.currentQueueSize = this.queue.size;
    }

  /**
   * Send message to server
   */
  async sendMessage(message) {
    const response = await api.post(`/chat/${message.chatId}/messages`, message.toApiFormat());

    // Update message with server response
    messageStateManager.updateFromServerResponse(message.metadata.clientId, response.data?.data || response.data);

    return response.data;
  }

  /**
   * Edit message on server
   */
  async editMessage(message) {
    const response = await api.patch(
      `/chat/${message.chatId}/messages/${message.metadata.serverId}`,
      { content: message.content }
    );

    return response.data;
  }

  /**
   * Delete message on server
   */
  async deleteMessage(message) {
    const response = await api.delete(`/chat/${message.chatId}/messages/${message.metadata.serverId}`);
    return response.data;
  }

  /**
   * Handle processing success
   */
  async handleProcessingSuccess(queueItem, result) {
    // console.log(`Successfully processed: ${queueItem.message.metadata.clientId}`);

    // Remove from queue - item is already removed in processQueueItem finally block
    // No additional state updates needed here as they're handled in sendMessage, etc.
  }

  /**
   * Handle processing error
   */
  async handleProcessingError(queueItem, error) {
    // console.error(`ERROR: Error processing queue item:`, error);

    const errorInfo = {
      message: error.message,
      status: error.response?.status,
      timestamp: new Date().toISOString()
    };

    // Check if error is retryable
    const isRetryable = this.isRetryableError(error);

    if (isRetryable && queueItem.canRetry()) {
      // Prepare for retry
      queueItem.prepareRetry(errorInfo);

      // Update message state to failed (retryable)
      messageStateManager.updateMessageState(
        queueItem.message.metadata.clientId,
        MessageState.FAILED,
        { errorInfo }
      );

      // Re-queue for retry
      this.addToPriorityQueue(queueItem);

      // console.log(`🔄 Will retry: ${queueItem.message.metadata.clientId} (attempt ${queueItem.retryCount})`);
    } else {
      // Permanent failure
      messageStateManager.updateMessageState(
        queueItem.message.metadata.clientId,
        MessageState.REJECTED,
        { errorInfo }
      );

      // console.log(`💀 Permanently failed: ${queueItem.message.metadata.clientId}`);
    }

  /**
   * Check if error is retryable
   */
  isRetryableError(error) {
    const status = error.response?.status;

    // Network errors are retryable
    if (!status) return true;

    // Server errors (5xx) are retryable
    if (status >= 500) return true;

    // Rate limiting is retryable
    if (status === 429) return true;

    // Timeout is retryable
    if (status === 408) return true;

    // Client errors (4xx) are generally not retryable
    // except for specific cases
    if (status >= 400 && status < 500) {
      // Conflict might be retryable with updated data
      if (status === 409) return true;

      // Other 4xx are not retryable
      return false;
    }

    return false;
  }

  /**
   * Update average processing time
   */
  updateAverageProcessingTime(newTime) {
    const alpha = 0.1; // Exponential moving average factor
    this.stats.averageProcessingTime =
      this.stats.averageProcessingTime * (1 - alpha) + newTime * alpha;
  }

  /**
   * Start processing loop
   */
  startProcessing() {
    this.isProcessing.value = true;

    const processLoop = async () => {
      if (!this.isProcessing.value) return;

      try {
        await this.processNextBatch();
      } catch (error) {
        // console.error('Error in processing loop:', error);
      }

      // Schedule next iteration
      setTimeout(processLoop, this.config.processingIntervalMs);
    };

    processLoop();
    // console.log('Message queue processing started');
  }

  /**
   * Stop processing
   */
  stopProcessing() {
    this.isProcessing.value = false;
    // console.log('⏹️ Message queue processing stopped');
  }

  /**
   * Pause processing
   */
  pause() {
    this.isPaused.value = true;
    // console.log('⏸️ Message queue processing paused');
  }

  /**
   * Resume processing
   */
  resume() {
    this.isPaused.value = false;
    // console.log('▶️ Message queue processing resumed');
  }

  /**
   * Clear queue
   */
  clearQueue() {
    // Abort all pending items
    for (const queueItem of this.queue.values()) {
      queueItem.aborted = true;
    }

    this.queue.clear();
    this.processingQueue.clear();

    // Clear priority queues
    Object.values(this.priorityQueues).forEach(queue => queue.length = 0);

    this.stats.currentQueueSize = 0;
    // console.log('🧹 Message queue cleared');
  }

  /**
   * Get queue status
   */
  getStatus() {
    return {
      isProcessing: this.isProcessing.value,
      isPaused: this.isPaused.value,
      isOnline: this.isOnline.value,
      queueSize: this.queue.size,
      processingCount: this.processingQueue.size,
      stats: { ...this.stats },
      config: { ...this.config }
    };
  }

  /**
   * Cleanup completed items
   */
  cleanup() {
    const now = Date.now();
    const maxAge = 24 * 60 * 60 * 1000; // 24 hours
    let cleaned = 0;

    for (const [queueId, queueItem] of this.queue.entries()) {
      const itemAge = now - new Date(queueItem.createdAt).getTime();

      // Remove old aborted or permanently failed items
      if ((queueItem.aborted || !queueItem.canRetry()) && itemAge > maxAge) {
        this.queue.delete(queueId);
        this.removeFromPriorityQueue(queueItem);
        cleaned++;
      }

    this.stats.currentQueueSize = this.queue.size;

    if (cleaned > 0) {
      // console.log(`🧹 Cleaned up ${cleaned} old queue items`);
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

// Create global instance
export const messageSyncQueue = new MessageSyncQueue();

// Export for testing
export { QueueItem }; 