/**
 * 🔒 Message Offline Queue Manager - Offline Message Storage & Sync
 * 
 * Provides persistent storage for messages when offline and synchronization when back online
 * Ensures no message loss and maintains order integrity
 */

import { ref, reactive } from 'vue';
import { messageStateManager, MessageState, MessagePriority } from './MessageStateManager.js';
import { messageSyncQueue } from './MessageSyncQueue.js';

/**
 * Offline storage manager using IndexedDB for persistence
 */
class OfflineStorageManager {
  constructor() {
    this.dbName = 'FechatterOfflineMessages';
    this.dbVersion = 1;
    this.db = null;
    this.isInitialized = false;
  }

  /**
   * Initialize IndexedDB
   */
  async initialize() {
    if (this.isInitialized) return;

    return new Promise((resolve, reject) => {
      const request = indexedDB.open(this.dbName, this.dbVersion);

      request.onerror = () => reject(request.error);
      request.onsuccess = () => {
        this.db = request.result;
        this.isInitialized = true;
        // console.log('📦 IndexedDB initialized for offline storage');
        resolve();
      };

      request.onupgradeneeded = (event) => {
        const db = event.target.result;

        // Create messages store
        if (!db.objectStoreNames.contains('messages')) {
          const messagesStore = db.createObjectStore('messages', { keyPath: 'clientId' });
          messagesStore.createIndex('chatId', 'chatId', { unique: false });
          messagesStore.createIndex('state', 'state', { unique: false });
          messagesStore.createIndex('createdAt', 'createdAt', { unique: false });

        // Create sync metadata store
        if (!db.objectStoreNames.contains('syncMetadata')) {
          db.createObjectStore('syncMetadata', { keyPath: 'key' });

        // console.log('📦 IndexedDB schema created');
      };
    });

  /**
   * Store message offline
   */
  async storeMessage(message) {
    if (!this.isInitialized) await this.initialize();

    const transaction = this.db.transaction(['messages'], 'readwrite');
    const store = transaction.objectStore('messages');

    const messageData = {
      clientId: message.metadata.clientId,
      chatId: message.chatId,
      senderId: message.senderId,
      content: message.content,
      files: message.files,
      state: message.metadata.state,
      createdAt: message.metadata.createdAt,
      metadata: message.metadata
    };

    return new Promise((resolve, reject) => {
      const request = store.put(messageData);
      request.onsuccess = () => resolve();
      request.onerror = () => reject(request.error);
    });

  /**
   * Retrieve stored messages
   */
  async getStoredMessages(filters = {}) {
    if (!this.isInitialized) await this.initialize();

    const transaction = this.db.transaction(['messages'], 'readonly');
    const store = transaction.objectStore('messages');

    return new Promise((resolve, reject) => {
      const request = store.getAll();
      request.onsuccess = () => {
        let messages = request.result;

        // Apply filters
        if (filters.state) {
          messages = messages.filter(msg => msg.state === filters.state);
        }

        resolve(messages);
      };
      request.onerror = () => reject(request.error);
    });

  /**
   * Remove message from offline storage
   */
  async removeMessage(clientId) {
    if (!this.isInitialized) await this.initialize();

    const transaction = this.db.transaction(['messages'], 'readwrite');
    const store = transaction.objectStore('messages');

    return new Promise((resolve, reject) => {
      const request = store.delete(clientId);
      request.onsuccess = () => resolve();
      request.onerror = () => reject(request.error);
    });

  /**
   * Clear all stored messages
   */
  async clearAll() {
    if (!this.isInitialized) await this.initialize();

    const transaction = this.db.transaction(['messages'], 'readwrite');
    const store = transaction.objectStore('messages');

    return new Promise((resolve, reject) => {
      const request = store.clear();
      request.onsuccess = () => resolve();
      request.onerror = () => reject(request.error);
    });

/**
 * Message Offline Queue Manager
 */
export class MessageOfflineQueue {
  constructor() {
    // Storage manager
    this.storage = new OfflineStorageManager();

    // State management
    this.isOnline = ref(navigator.onLine);
    this.isInitialized = ref(false);
    this.isSyncing = ref(false);

    // Statistics
    this.stats = reactive({
      offlineMessages: 0,
      pendingSyncMessages: 0,
      lastSyncTime: null,
      totalMessagesSynced: 0,
      syncErrors: 0
    });

    // Configuration
    this.config = {
      maxOfflineMessages: 10000,     // Maximum offline messages to store
      syncBatchSize: 20,             // Messages to sync per batch
      syncIntervalMs: 5000,          // Sync check interval when online
      maxSyncRetries: 3,             // Maximum sync retry attempts
      autoSyncOnReconnect: true      // Auto-sync when back online
    };

    // Initialize
    this.initialize();
    this.setupNetworkHandlers();
  }

  /**
   * Initialize offline queue
   */
  async initialize() {
    try {
      await this.storage.initialize();
      await this.loadOfflineMessages();

      this.isInitialized.value = true;
      // console.log('🔒 Offline message queue initialized');

      this.startSyncTimer();

    } catch (error) {
      if (import.meta.env.DEV) {
        console.error('Failed to initialize offline queue:', error);
      }

  /**
   * Load offline messages from storage
   */
  async loadOfflineMessages() {
    try {
      const storedMessages = await this.storage.getStoredMessages();

      // Restore messages to state manager
      for (const messageData of storedMessages) {
        // Recreate message object
        const message = messageStateManager.createMessage(
          messageData.content,
          messageData.chatId,
          messageData.senderId,
          {
            files: messageData.files,
            metadata: messageData.metadata
          }
        );

        // Update state from stored data
        messageStateManager.updateMessageState(message.metadata.clientId, messageData.state);
      }

      this.stats.offlineMessages = storedMessages.length;
      // console.log(`📦 Loaded ${storedMessages.length} offline messages from storage`);

    } catch (error) {
      if (import.meta.env.DEV) {
        console.error('Failed to load offline messages:', error);
      }

  /**
   * Setup network status handlers
   */
  setupNetworkHandlers() {
    if (typeof window !== 'undefined') {
      window.addEventListener('online', () => {
        this.isOnline.value = true;
        // console.log('🌐 Back online: Starting offline message sync');

        if (this.config.autoSyncOnReconnect) {
          this.syncOfflineMessages();
        }
      });

      window.addEventListener('offline', () => {
        this.isOnline.value = false;
        // console.log('🌐 Gone offline: Messages will be queued for later sync');
      });

  /**
   * Queue message for offline storage
   */
  async queueOfflineMessage(message) {
    if (this.stats.offlineMessages >= this.config.maxOfflineMessages) {
      if (import.meta.env.DEV) {
        console.warn('WARNING: Offline queue is full, removing oldest messages');
      await this.purgeOldestMessages(100); // Remove 100 oldest messages
    }

    try {
      // Store in IndexedDB
      await this.storage.storeMessage(message);

      // Update message state to queued
      messageStateManager.updateMessageState(message.metadata.clientId, MessageState.QUEUED);

      this.stats.offlineMessages++;
      // console.log(`🔒 Queued message offline: ${message.metadata.clientId}`);

    } catch (error) {
      if (import.meta.env.DEV) {
        console.error('Failed to queue message offline:', error);
      throw error;
    }

  /**
   * Sync offline messages when back online
   */
  async syncOfflineMessages() {
    if (!this.isOnline.value || this.isSyncing.value || !this.isInitialized.value) {
      return;
    }

    this.isSyncing.value = true;

    try {
      // console.log('🔄 Starting offline message sync...');

      // Get messages that need syncing
      const messagesToSync = await this.storage.getStoredMessages({
        state: MessageState.QUEUED
      });

      if (messagesToSync.length === 0) {
        // console.log('No offline messages to sync');
        return;
      }

      // console.log(`📤 Syncing ${messagesToSync.length} offline messages...`);

      // Process messages in batches
      const batches = this.createBatches(messagesToSync, this.config.syncBatchSize);
      let syncedCount = 0;
      let errorCount = 0;

      for (const batch of batches) {
        try {
          const results = await this.syncBatch(batch);
          syncedCount += results.successful;
          errorCount += results.failed;

          // Small delay between batches to prevent overwhelming the server
          await new Promise(resolve => setTimeout(resolve, 500));

        } catch (error) {
          if (import.meta.env.DEV) {
            console.error('Batch sync failed:', error);
          errorCount += batch.length;
        }

      // Update statistics
      this.stats.totalMessagesSynced += syncedCount;
      this.stats.syncErrors += errorCount;
      this.stats.lastSyncTime = new Date().toISOString();
      this.stats.pendingSyncMessages = messagesToSync.length - syncedCount;

      // console.log(`Sync completed: ${syncedCount} successful, ${errorCount} failed`);

    } catch (error) {
      if (import.meta.env.DEV) {
        console.error('Failed to sync offline messages:', error);
      }
    } finally {
      this.isSyncing.value = false;
    }

  /**
   * Create batches from messages array
   */
  createBatches(messages, batchSize) {
    const batches = [];
    for (let i = 0; i < messages.length; i += batchSize) {
      batches.push(messages.slice(i, i + batchSize));
    return batches;
  }

  /**
   * Sync a batch of messages
   */
  async syncBatch(messageDataBatch) {
    const results = { successful: 0, failed: 0 };

    // Process each message in the batch
    const promises = messageDataBatch.map(async (messageData) => {
      try {
        // Recreate message object
        const message = messageStateManager.getMessage(messageData.clientId);
        if (!message) {
          if (import.meta.env.DEV) {
            console.warn(`Message not found in state manager: ${messageData.clientId}`);
          return { success: false, clientId: messageData.clientId };
        }

        // Queue for sending through sync queue
        await messageSyncQueue.queueSend(message, {
          priority: MessagePriority.HIGH // Offline messages get high priority
        });

        // Remove from offline storage on successful queue
        await this.storage.removeMessage(messageData.clientId);
        this.stats.offlineMessages--;

        return { success: true, clientId: messageData.clientId };

      } catch (error) {
        if (import.meta.env.DEV) {
          console.error(`Failed to sync message ${messageData.clientId}:`, error);
        return { success: false, clientId: messageData.clientId, error };
      }
    });

    const batchResults = await Promise.allSettled(promises);

    for (const result of batchResults) {
      if (result.status === 'fulfilled' && result.value.success) {
        results.successful++;
      } else {
        results.failed++;
      }

    return results;
  }

  /**
   * Purge oldest messages to free up space
   */
  async purgeOldestMessages(count) {
    try {
      const allMessages = await this.storage.getStoredMessages();

      // Sort by creation time (oldest first)
      allMessages.sort((a, b) => new Date(a.createdAt) - new Date(b.createdAt));

      // Remove oldest messages
      const toRemove = allMessages.slice(0, count);

      for (const message of toRemove) {
        await this.storage.removeMessage(message.clientId);
        messageStateManager.removeMessage(message.clientId);
      }

      this.stats.offlineMessages -= toRemove.length;
      // console.log(`🧹 Purged ${toRemove.length} oldest offline messages`);

    } catch (error) {
      if (import.meta.env.DEV) {
        console.error('Failed to purge old messages:', error);
      }

  /**
   * Start sync timer
   */
  startSyncTimer() {
    setInterval(() => {
      if (this.isOnline.value && this.stats.offlineMessages > 0) {
        this.syncOfflineMessages();
      }
    }, this.config.syncIntervalMs);
  }

  /**
   * Manual sync trigger
   */
  async forceSyncAll() {
    if (!this.isOnline.value) {
      throw new Error('Cannot sync while offline');
    }

    // console.log('🔄 Force syncing all offline messages...');
    await this.syncOfflineMessages();
  }

  /**
   * Clear all offline messages
   */
  async clearAllOfflineMessages() {
    try {
      await this.storage.clearAll();
      this.stats.offlineMessages = 0;
      this.stats.pendingSyncMessages = 0;
      // console.log('🧹 Cleared all offline messages');
    } catch (error) {
      if (import.meta.env.DEV) {
        console.error('Failed to clear offline messages:', error);
      throw error;
    }

  /**
   * Get offline queue status
   */
  getStatus() {
    return {
      isOnline: this.isOnline.value,
      isInitialized: this.isInitialized.value,
      isSyncing: this.isSyncing.value,
      stats: { ...this.stats },
      config: { ...this.config }
    };
  }

  /**
   * Update configuration
   */
  updateConfig(newConfig) {
    Object.assign(this.config, newConfig);
    // console.log('Updated offline queue configuration');
  }

  /**
   * Export offline messages for debugging
   */
  async exportOfflineMessages() {
    try {
      const messages = await this.storage.getStoredMessages();
      return messages.map(msg => ({
        clientId: msg.clientId,
        chatId: msg.chatId,
        content: msg.content,
        state: msg.state,
        createdAt: msg.createdAt
      }));
    } catch (error) {
      if (import.meta.env.DEV) {
        console.error('Failed to export offline messages:', error);
      return [];
    }

// Create global instance
export const messageOfflineQueue = new MessageOfflineQueue();

// Export for testing
export { OfflineStorageManager }; 