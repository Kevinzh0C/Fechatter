/**
 * 🔄 REFACTOR: Message Service with Clean Architecture
 * 
 * Follows SOLID principles:
 * - Single Responsibility: Only orchestrates message operations
 * - Open/Closed: Extensible through dependency injection
 * - Liskov Substitution: All dependencies are replaceable
 * - Interface Segregation: Small, focused interfaces
 * - Dependency Inversion: Depends on abstractions, not concretions
 */

import { ref, reactive } from 'vue';

export class RefactoredMessageService {
  constructor(dependencies) {
    // 🔧 Dependency Injection - No global access
    this.apiClient = dependencies.apiClient;
    this.userDataResolver = dependencies.userDataResolver;
    this.messageDisplayGuarantee = dependencies.messageDisplayGuarantee;
    this.logger = dependencies.logger;
    this.cacheProvider = dependencies.cacheProvider;

    // 🔧 Performance optimization
    this.batchSize = dependencies.batchSize || 50;
    this.prefetchThreshold = dependencies.prefetchThreshold || 10;

    // 🔧 Metrics tracking
    this.metrics = {
      fetchCount: 0,
      cacheHits: 0,
      averageFetchTime: 0,
      userResolutionTime: 0
    };

    // 🚀 STATE MANAGEMENT: Reactive state without mixing concerns
    this.isInitialized = ref(false);
    this.isOnline = ref(navigator.onLine);
    this.messagesByChat = reactive(new Map());
    this.messageCache = reactive({});
    this.hasMoreByChat = new Map();

    // 📊 METRICS: Separate from business logic
    this.stats = reactive({
      totalMessages: 0,
      messagesSent: 0,
      messagesReceived: 0,
      messagesFailedToSend: 0,
      averageResponseTime: 0,
      lastActivity: null
    });

    this.setupOnlineListener();
  }

  /**
   * 🚀 INITIALIZATION: Clean startup
   */
  async initialize() {
    if (this.isInitialized.value) return;

    try {
      this.logger.log('info', 'initialization', 'service-start', {
        message: 'Initializing RefactoredMessageService'
      });

      this.isInitialized.value = true;

      this.logger.log('info', 'initialization', 'service-ready', {
        message: 'RefactoredMessageService initialized successfully'
      });
    } catch (error) {
      this.logger.logError('initialization', error);
      throw error;
    }
  }

  /**
   * 📥 FETCH MESSAGES: Clean orchestration of fetch pipeline
   */
  async fetchMessages(chatId, options = {}) {
    const startTime = performance.now();

    try {
      this.logger.logMessageFetch(chatId, 0, { starting: true });

      // 🔧 Step 1: Fetch raw message data
      const rawMessages = await this.fetchRawMessages(chatId, options);

      if (!rawMessages?.length) {
        this.logger.logMessageFetch(chatId, 0, { result: 'no_messages' });
        return [];
      }

      // 🔧 Step 2: Process messages in batch for performance
      const processedMessages = await this.processMessagesBatch(rawMessages);

      // 🔧 Step 3: Start tracking for display guarantee
      if (options.enableTracking !== false) {
        await this.startMessageTracking(chatId, processedMessages);
      }

      // 🔧 Step 4: Update metrics
      this.updateMetrics('fetch', performance.now() - startTime);

      this.logger.logMessageFetch(chatId, processedMessages.length, {
        duration: performance.now() - startTime,
        cached: false
      });

      return processedMessages;

    } catch (error) {
      this.logger.logTrackingFailure(chatId, {
        operation: 'fetchMessages',
        error: error.message,
        duration: performance.now() - startTime
      });
      throw error;
    }
  }

  /**
   * Fetch raw messages from API
   * Single responsibility: API communication
   */
  async fetchRawMessages(chatId, options) {
    const cacheKey = `messages_${chatId}_${JSON.stringify(options)}`;

    // Check cache first
    if (this.cacheProvider && !options.bypassCache) {
      const cached = await this.cacheProvider.get(cacheKey);
      if (cached) {
        this.metrics.cacheHits++;
        return cached;
      }
    }

    // Fetch from API
    const response = await this.apiClient.get(`/chat/${chatId}/messages`, {
      params: options
    });

    const messages = response.data?.messages || [];

    // Cache result
    if (this.cacheProvider && messages.length > 0) {
      await this.cacheProvider.set(cacheKey, messages, { ttl: 5 * 60 * 1000 }); // 5 minutes
    }

    return messages;
  }

  /**
   * Process messages in batches for performance
   * Single responsibility: Message data processing
   */
  async processMessagesBatch(rawMessages) {
    const batches = this.createBatches(rawMessages, this.batchSize);
    const processedBatches = await Promise.all(
      batches.map(batch => this.processBatch(batch))
    );

    return processedBatches.flat();
  }

  /**
   * Process a single batch of messages
   */
  async processBatch(batch) {
    const startTime = performance.now();

    try {
      // 🔧 Batch resolve user names for performance
      const userNamesMap = await this.userDataResolver.batchResolveUsers(batch);

      // 🔧 Create processed messages
      const processedMessages = batch.map(message => this.createProcessedMessage(message, userNamesMap));

      this.updateMetrics('userResolution', performance.now() - startTime);

      return processedMessages;

    } catch (error) {
      this.logger.logTrackingFailure(null, {
        operation: 'processBatch',
        batchSize: batch.length,
        error: error.message
      });

      // Fallback: process without batch user resolution
      return batch.map(message => this.createProcessedMessage(message, new Map()));
    }
  }

  /**
   * Create processed message object
   * Single responsibility: Message object creation
   */
  createProcessedMessage(rawMessage, userNamesMap) {
    return {
      ...rawMessage,
      id: parseInt(rawMessage.id),
      sender_id: parseInt(rawMessage.sender_id),

      // 🔧 Resolved user name from batch operation
      resolvedUserName: userNamesMap.get(rawMessage.id) || `User ${rawMessage.sender_id}`,

      // 🔧 Enhanced sender object
      sender: {
        id: parseInt(rawMessage.sender_id),
        fullname: userNamesMap.get(rawMessage.id) || rawMessage.sender?.fullname || `User ${rawMessage.sender_id}`,
        email: rawMessage.sender?.email || '',
        avatar_url: rawMessage.sender?.avatar_url || null
      },

      // 🔧 Standardized timestamps
      created_at: new Date(rawMessage.created_at),
      updated_at: rawMessage.updated_at ? new Date(rawMessage.updated_at) : null,

      // 🔧 Processing metadata
      processedAt: Date.now(),
      processingVersion: '2.0'
    };
  }

  /**
   * Start message tracking for display guarantee
   * Single responsibility: Tracking coordination
   */
  async startMessageTracking(chatId, messages) {
    if (!this.messageDisplayGuarantee) {
      this.logger.logMessageFetch(chatId, messages.length, { tracking: 'disabled' });
      return;
    }

    try {
      const messageIds = messages.map(msg => msg.id);
      const trackingId = await this.messageDisplayGuarantee.startMessageTracking(chatId, messageIds);

      this.logger.logTrackingStart(chatId, messageIds, trackingId);

      return trackingId;

    } catch (error) {
      this.logger.logTrackingFailure(chatId, {
        operation: 'startMessageTracking',
        messageCount: messages.length,
        error: error.message
      });
      // Don't throw - tracking failure shouldn't break message loading
    }
  }

  /**
   * 🧹 CLEANUP: Clear chat data
   */
  async clearMessagesForChat(chatId) {
    const normalizedChatId = parseInt(chatId);

    try {
      this.logger.logClearingStart(chatId, new Error().stack);

      // Clear from storage
      this.messagesByChat.delete(normalizedChatId);
      delete this.messageCache[chatId];
      this.hasMoreByChat.delete(normalizedChatId);

      // Clear tracking contexts
      const clearedContexts = this.messageDisplayGuarantee.clearTrackingForChat(normalizedChatId);

      this.logger.logClearingComplete(chatId, clearedContexts);

    } catch (error) {
      this.logger.logError('clearing', error, { chatId });
    }
  }

  /**
   * 💾 STORAGE: Update message storage and cache
   */
  updateMessageStorage(chatId, messages, hasMore) {
    const normalizedChatId = parseInt(chatId);

    this.messagesByChat.set(normalizedChatId, messages);
    this.hasMoreByChat.set(normalizedChatId, hasMore);

    this.messageCache[chatId] = {
      messages,
      timestamp: Date.now(),
      isFresh: true
    };
  }

  /**
   * 📊 METRICS: Update performance statistics
   */
  updateStats(messageCount, duration) {
    this.stats.totalMessages += messageCount;
    this.stats.messagesReceived += messageCount;
    this.stats.lastActivity = Date.now();

    // Update average response time (rolling average)
    const alpha = 0.1; // Smoothing factor
    this.stats.averageResponseTime =
      this.stats.averageResponseTime * (1 - alpha) + duration * alpha;
  }

  /**
   * 📶 ONLINE STATE: Monitor connectivity
   */
  setupOnlineListener() {
    const updateOnlineStatus = () => {
      this.isOnline.value = navigator.onLine;
    };

    window.addEventListener('online', updateOnlineStatus);
    window.addEventListener('offline', updateOnlineStatus);
  }

  /**
   * 🔍 GETTERS: Public interface for data access
   */
  getMessagesForChat(chatId, states = null) {
    const messages = this.messagesByChat.get(parseInt(chatId)) || [];

    // Filter by states if specified
    if (states && states.length > 0) {
      return messages.filter(msg => states.includes(msg.status));
    }

    return messages;
  }

  hasMoreMessages(chatId) {
    return this.hasMoreByChat.get(parseInt(chatId)) !== false;
  }

  getStatus() {
    return {
      isInitialized: this.isInitialized.value,
      isOnline: this.isOnline.value,
      stats: { ...this.stats }
    };
  }

  getMessageCache() {
    return this.messageCache;
  }

  /**
   * 📨 SENDING: Send new message (placeholder for now)
   */
  async sendMessage(content, chatId, options = {}) {
    try {
      // This would integrate with the sending pipeline
      const tempMessage = {
        clientId: Date.now().toString(),
        tempId: Date.now(),
        state: 'sending',
        message: {
          id: Date.now(),
          content,
          chat_id: parseInt(chatId),
          created_at: new Date().toISOString(),
          sender_id: options.senderId,
          status: 'sending'
        }
      };

      this.stats.messagesSent++;
      return tempMessage;

    } catch (error) {
      this.stats.messagesFailedToSend++;
      this.logger.logError('message-sending', error, { chatId, content: content.substring(0, 50) });
      throw error;
    }
  }

  /**
   * 🧹 CLEANUP: Clear all data
   */
  clearAllMessages() {
    this.messagesByChat.clear();
    Object.keys(this.messageCache).forEach(key => {
      delete this.messageCache[key];
    });
    this.hasMoreByChat.clear();

    // Reset stats
    Object.assign(this.stats, {
      totalMessages: 0,
      messagesSent: 0,
      messagesReceived: 0,
      messagesFailedToSend: 0,
      averageResponseTime: 0,
      lastActivity: null
    });
  }

  /**
   * 🧪 TESTING & DEBUG: Export internal state
   */
  async exportDebugInfo() {
    return {
      isInitialized: this.isInitialized.value,
      isOnline: this.isOnline.value,
      messageCount: Array.from(this.messagesByChat.values())
        .reduce((sum, messages) => sum + messages.length, 0),
      chatCount: this.messagesByChat.size,
      stats: { ...this.stats },
      dependencies: {
        hasApiClient: !!this.apiClient,
        hasUserDataResolver: !!this.userDataResolver,
        hasMessageDisplayGuarantee: !!this.messageDisplayGuarantee,
        hasLogger: !!this.logger,
        hasCacheProvider: !!this.cacheProvider
      },
      loggerStats: this.logger.getStats?.() || 'No stats available',
      metrics: this.getMetrics()
    };
  }

  /**
   * 🔄 BATCH OPERATIONS: Optimize user data loading
   */
  async preloadUserData(chatId) {
    try {
      const messages = this.getMessagesForChat(chatId);
      const senderIds = [...new Set(messages.map(m => m.sender_id))];

      if (senderIds.length > 0) {
        await this.userDataResolver.batchResolveUsers(senderIds);
        this.logger.log('info', 'optimization', 'batch-preload', {
          message: `Preloaded user data for ${senderIds.length} users in chat ${chatId}`,
          chatId: parseInt(chatId),
          userCount: senderIds.length
        });
      }
    } catch (error) {
      this.logger.logError('user-preload', error, { chatId });
    }
  }

  /**
   * Update performance metrics
   */
  updateMetrics(operation, duration) {
    switch (operation) {
      case 'fetch':
        this.metrics.fetchCount++;
        this.metrics.averageFetchTime =
          (this.metrics.averageFetchTime * (this.metrics.fetchCount - 1) + duration) / this.metrics.fetchCount;
        break;
      case 'userResolution':
        this.metrics.userResolutionTime =
          (this.metrics.userResolutionTime * 0.8) + (duration * 0.2);
        break;
    }
  }

  /**
   * Get service metrics
   */
  getMetrics() {
    return {
      ...this.metrics,
      cacheHitRate: this.metrics.fetchCount > 0
        ? ((this.metrics.cacheHits / this.metrics.fetchCount) * 100).toFixed(2) + '%'
        : '0%',
      userResolver: this.userDataResolver?.getMetrics?.() || null,
      displayGuarantee: this.messageDisplayGuarantee?.getMetrics?.() || null
    };
  }

  /**
   * Health check for all dependencies
   */
  async healthCheck() {
    const results = {
      service: 'RefactoredMessageService',
      status: 'healthy',
      dependencies: {},
      timestamp: Date.now()
    };

    // Check API client
    try {
      results.dependencies.apiClient = {
        status: this.apiClient ? 'available' : 'missing',
        type: typeof this.apiClient
      };
    } catch (error) {
      results.dependencies.apiClient = { status: 'error', error: error.message };
    }

    // Check user data resolver
    try {
      const resolverMetrics = this.userDataResolver?.getMetrics?.();
      results.dependencies.userDataResolver = {
        status: this.userDataResolver ? 'available' : 'missing',
        metrics: resolverMetrics
      };
    } catch (error) {
      results.dependencies.userDataResolver = { status: 'error', error: error.message };
    }

    // Check message display guarantee
    try {
      const guaranteeMetrics = this.messageDisplayGuarantee?.getMetrics?.();
      results.dependencies.messageDisplayGuarantee = {
        status: this.messageDisplayGuarantee ? 'available' : 'missing',
        metrics: guaranteeMetrics
      };
    } catch (error) {
      results.dependencies.messageDisplayGuarantee = { status: 'error', error: error.message };
    }

    // Overall health
    const hasErrors = Object.values(results.dependencies).some(dep => dep.status === 'error');
    if (hasErrors) {
      results.status = 'degraded';
    }

    return results;
  }

  /**
   * Cleanup resources
   */
  destroy() {
    if (this.userDataResolver?.destroy) {
      this.userDataResolver.destroy();
    }

    // Clear metrics
    this.metrics = {
      fetchCount: 0,
      cacheHits: 0,
      averageFetchTime: 0,
      userResolutionTime: 0
    };
  }

  /**
   * 📄 PAGINATION: Load more messages
   */
  async fetchMoreMessages(chatId, options = {}) {
    const { limit = 15 } = options;

    try {
      const existingMessages = this.messagesByChat.get(parseInt(chatId)) || [];
      const offset = existingMessages.length;

      // Use the same fetcher but with offset
      const rawMessages = await this.fetchRawMessages(chatId, {
        ...options,
        limit,
        offset
      });

      const normalizedMessages = this.processMessagesBatch(rawMessages);

      // Combine with existing messages
      const combinedMessages = [...existingMessages, ...normalizedMessages];

      // Update storage
      this.updateMessageStorage(chatId, combinedMessages, normalizedMessages.length >= limit);

      return normalizedMessages;

    } catch (error) {
      this.logger.logError('pagination', error, { chatId, options });
      return [];
    }
  }

  /**
   * Prefetch messages for performance
   * Single responsibility: Performance optimization
   */
  async prefetchMessages(chatId, options = {}) {
    if (!this.cacheProvider) return;

    try {
      const prefetchOptions = {
        ...options,
        limit: this.prefetchThreshold
      };

      // Prefetch in background without blocking
      setImmediate(async () => {
        try {
          await this.fetchRawMessages(chatId, prefetchOptions);
          this.logger.logPerformance('prefetch', 0, { chatId, success: true });
        } catch (error) {
          this.logger.logPerformance('prefetch', 0, { chatId, success: false, error: error.message });
        }
      });

    } catch (error) {
      // Prefetch failures are non-critical
      this.logger.logPerformance('prefetch', 0, { chatId, success: false, error: error.message });
    }
  }

  /**
   * Utility: Create batches from array
   */
  createBatches(array, batchSize) {
    const batches = [];
    for (let i = 0; i < array.length; i += batchSize) {
      batches.push(array.slice(i, i + batchSize));
    }
    return batches;
  }
} 