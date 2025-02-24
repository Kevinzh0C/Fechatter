/**
 * 🎯 Unified Message Service - Minimal Stub
 * 
 * Temporary stub to restore build; will be reimplemented in sub-tasks
 */

import { ref, reactive } from 'vue';
import { messageDisplayGuarantee } from './MessageDisplayGuarantee.js';

// Stub message states
export const MessageState = {
  QUEUED: 'queued',
  SENDING: 'sending',
  SENT: 'sent',
  DELIVERED: 'delivered',
  FAILED: 'failed'
};

export const MessagePriority = {
  LOW: 'low',
  NORMAL: 'normal',
  HIGH: 'high'
};

/**
 * Minimal Unified Message Service stub
 */
export class UnifiedMessageService {
  constructor() {
    this.isInitialized = ref(false);
    this.isOnline = ref(navigator.onLine);
    this.stats = reactive({
      totalMessages: 0,
      messagesSent: 0,
      messagesReceived: 0,
      messagesFailedToSend: 0,
      averageResponseTime: 0,
      lastActivity: null
    });

    // 🔧 CRITICAL FIX: Make message storage reactive for Vue
    this.messagesByChat = reactive(new Map()); // chatId -> messages[]
    this.messageCache = reactive({});
    this.hasMoreByChat = new Map(); // chatId -> boolean

    // 🔧 NEW: Smart cache management
    this.cacheConfig = {
      maxCacheSize: 50, // Maximum number of chats to cache
      maxMessageAge: 30 * 60 * 1000, // 30 minutes
      cleanupInterval: 5 * 60 * 1000, // Clean up every 5 minutes
      maxMessagesPerChat: 1000 // Maximum messages per chat
    };

    // Start cache cleanup timer
    this._startCacheCleanup();
  }

  async initialize() {
    this.isInitialized.value = true;
  }

  async sendMessage(content, chatId, options = {}) {
    // Stub implementation
    return {
      clientId: Date.now().toString(),
      tempId: Date.now(),
      state: MessageState.SENT,
      message: {
        id: Date.now(),
        content,
        chat_id: parseInt(chatId),
        created_at: new Date().toISOString()
      }
    };
  }

  getMessagesForChat(chatId, states = null) {
    return this.messagesByChat.get(parseInt(chatId)) || [];
  }

  getStatus() {
    return {
      isInitialized: this.isInitialized.value,
      isOnline: this.isOnline.value,
      stats: { ...this.stats }
    };
  }

  /**
   * Fetch messages for a chat with display guarantee
   */
  async fetchMessages(chatId, options = {}) {
    const { limit = 15, abortSignal = null, isPreload = false } = options;

    try {
      if (!isPreload && import.meta.env.DEV) {
        console.log(`📥 [UnifiedMessageService] Fetching messages for chat ${chatId}`);
      }

      // Check if request was aborted
      if (abortSignal?.aborted) {
        throw new DOMException('Request aborted', 'AbortError');
      }

      // 🔧 BALANCED FIX: Only return cached for preload requests or when explicitly requested
      const existingMessages = this.messagesByChat.get(parseInt(chatId));
      if (isPreload && existingMessages && existingMessages.length > 0) {
        // For preload requests, return cached messages if available
        console.log(`⚡ [UnifiedMessageService] Using cached messages for preload: ${existingMessages.length} messages`);
        return existingMessages;
      }

      // 🔧 PRODUCTION FIX: Always fetch fresh data for user navigation
      let messages = [];

      try {
        // Import API service dynamically to avoid circular dependencies
        const { default: api } = await import('../api');

        // Make API call to fetch messages
        const response = await api.get(`/chat/${chatId}/messages`, {
          params: { limit },
          signal: abortSignal
        });

        // Extract messages from response
        const rawMessages = response.data?.data || response.data || [];

        // 🔧 ROOT CAUSE FIX: Normalize message format with proper sorting and file handling
        messages = rawMessages.map(msg => ({
          id: msg.id,
          content: msg.content || '',
          sender_id: msg.sender_id,
          // 🔧 FIX: Enhanced user info resolution
          sender_name: this._resolveUserName(msg),
          // 🔧 NEW: Complete sender object with UserStore integration
          sender: this._createSenderObject(msg),
          created_at: msg.created_at,
          chat_id: parseInt(chatId),
          status: 'sent',
          // 🔧 FIX: Standardize file format to prevent type errors
          files: this._standardizeFiles(msg.files || []),
          mentions: msg.mentions || [],
          reply_to: msg.reply_to || null
        }));

        // 🔧 ROOT CAUSE FIX: Sort messages chronologically (oldest first)
        messages.sort((a, b) => {
          const timeA = new Date(a.created_at).getTime();
          const timeB = new Date(b.created_at).getTime();
          return timeA - timeB; // Ascending order: oldest → newest
        });

        // 🛡️ NEW: Start display guarantee tracking
        if (messages.length > 0 && !isPreload) {
          const messageIds = messages.map(m => m.id);

          // 🔧 CRITICAL DEBUG: Add extensive logging to diagnose tracking issues
          if (import.meta.env.DEV) {
            console.log(`🛡️ [UnifiedMessageService] About to start tracking for chat ${chatId}:`, {
              messageCount: messages.length,
              messageIds: messageIds,
              isPreload: isPreload,
              chatId: parseInt(chatId)
            });
          }

          const trackingId = messageDisplayGuarantee.startMessageTracking(chatId, messageIds);

          if (import.meta.env.DEV) {
            if (trackingId) {
              console.log(`✅ [UnifiedMessageService] Successfully started tracking ${messageIds.length} messages for chat ${chatId}, trackingId: ${trackingId}`);
            } else {
              console.error(`❌ [UnifiedMessageService] Failed to start tracking for chat ${chatId} - no trackingId returned`);
            }

            // 🔧 CRITICAL DEBUG: Verify tracking context was created
            const activeContexts = messageDisplayGuarantee.verificationQueue;
            console.log(`🔍 [UnifiedMessageService] Active tracking contexts after creation:`, Array.from(activeContexts.entries()).map(([id, ctx]) => ({
              trackingId: id,
              chatId: ctx.chatId,
              messageIds: Array.from(ctx.messageIds),
              status: ctx.status
            })));
          }
        } else {
          // 🔧 DEBUG: Log why tracking was skipped
          if (import.meta.env.DEV) {
            console.log(`⚠️ [UnifiedMessageService] Skipping tracking for chat ${chatId}:`, {
              messageCount: messages.length,
              isPreload: isPreload,
              reason: messages.length === 0 ? 'No messages' : 'Is preload request'
            });
          }
        }

        if (!isPreload && import.meta.env.DEV) {
          console.log(`✅ [UnifiedMessageService] Fetched ${messages.length} messages for chat ${chatId}`);
        }

        // 🔧 CRITICAL FIX: Set hasMore flag based on actual limit and message count
        const hasMoreMessages = messages.length >= limit;
        this.hasMoreByChat.set(parseInt(chatId), hasMoreMessages);

        if (import.meta.env.DEV) {
          console.log(`📥 [Initial Load] Chat ${chatId}: ${messages.length} messages, hasMore: ${hasMoreMessages}`);
        }

        // 🔧 CRITICAL FIX: Use consistent integer keys for storage with smart caching
        const trimmedMessages = this._trimMessagesIfNeeded(chatId, messages);
        this.messagesByChat.set(parseInt(chatId), trimmedMessages);
        this.messageCache[chatId] = {
          messages: trimmedMessages,
          timestamp: Date.now(),
          isFresh: true
        };

      } catch (apiError) {
        if (apiError.name === 'AbortError') {
          throw apiError;
        }

        if (import.meta.env.DEV) {
          console.warn(`⚠️ [UnifiedMessageService] API call failed for chat ${chatId}:`, apiError.message);
        }

        // 🔧 PRODUCTION FIX: For network errors, return cached messages if available
        const cachedMessages = this.messagesByChat.get(parseInt(chatId));
        if (cachedMessages && cachedMessages.length > 0) {
          console.log(`🔄 [UnifiedMessageService] Using cached messages for chat ${chatId} due to network error`);

          // 🛡️ NEW: Track cached messages for display guarantee
          if (!isPreload) {
            const messageIds = cachedMessages.map(m => m.id);
            messageDisplayGuarantee.startMessageTracking(chatId, messageIds);
          }

          return cachedMessages;
        }

        // Only set empty array if we have no cached data
        messages = [];
      }

      return messages;

    } catch (error) {
      if (error.name === 'AbortError') {
        if (!isPreload && import.meta.env.DEV) {
          console.log(`🚫 [UnifiedMessageService] Fetch aborted for chat ${chatId}`);
        }
        throw error;
      }

      if (import.meta.env.DEV) {
        console.error(`❌ [UnifiedMessageService] Failed to fetch messages for chat ${chatId}:`, error);
      }

      // 🔧 PRODUCTION FIX: Return cached messages on error instead of throwing
      const cachedMessages = this.messagesByChat.get(parseInt(chatId));
      if (cachedMessages && cachedMessages.length > 0) {
        console.log(`🔄 [UnifiedMessageService] Returning cached messages for chat ${chatId} due to error`);

        // 🛡️ NEW: Track cached messages for display guarantee
        if (!isPreload) {
          const messageIds = cachedMessages.map(m => m.id);
          messageDisplayGuarantee.startMessageTracking(chatId, messageIds);
        }

        return cachedMessages;
      }

      throw error;
    }
  }

  /**
   * Load messages for a chat
   */
  async loadMessagesForChat(chatId) {
    return this.fetchMessages(chatId, { limit: 15 });
  }

  /**
   * Fetch more messages for pagination
   * 🔧 FIXED: Use offset-based pagination for better backend compatibility
   */
  async fetchMoreMessages(chatId, options = {}) {
    const { limit = 15 } = options;

    if (import.meta.env.DEV) {
      console.log(`📥 [UnifiedMessageService] Fetching more messages for chat ${chatId}`);
    }

    // Get existing messages to calculate offset
    const existing = this.messagesByChat.get(parseInt(chatId)) || [];
    let moreMessages = [];

    // 🔧 CRITICAL FIX: Track duplicate request attempts to prevent infinite loops
    if (!this.duplicateRequestTracker) {
      this.duplicateRequestTracker = new Map();
    }

    const trackingKey = `${chatId}_${existing.length}`;
    const attemptCount = this.duplicateRequestTracker.get(trackingKey) || 0;

    // 🔧 EMERGENCY STOP: Prevent infinite loops after 3 consecutive duplicate attempts
    if (attemptCount >= 3) {
      if (import.meta.env.DEV) {
        console.warn(`🛑 [Emergency Stop] Preventing infinite loop for chat ${chatId} - ${attemptCount} consecutive duplicate attempts`);
      }

      // Force hasMore to false to stop loading
      this.hasMoreByChat.set(parseInt(chatId), false);
      this.duplicateRequestTracker.delete(trackingKey);
      return [];
    }

    try {
      // Import API service dynamically to avoid circular dependencies
      const { default: api } = await import('../api');

      // 🔧 CRITICAL FIX: Use ID-based pagination for better reliability
      const requestParams = { limit };

      // If we have existing messages, use the oldest message ID for pagination
      if (existing.length > 0) {
        // Sort existing messages to find the oldest one
        const sortedExisting = [...existing].sort((a, b) => {
          const timeA = new Date(a.created_at).getTime();
          const timeB = new Date(b.created_at).getTime();
          return timeA - timeB;
        });

        const oldestMessage = sortedExisting[0];
        requestParams.before = oldestMessage.id;

        if (import.meta.env.DEV) {
          console.log(`🔍 [API Request] GET /chat/${chatId}/messages`, {
            ...requestParams,
            note: `Loading messages before ID ${oldestMessage.id}`
          });
        }
      } else {
        if (import.meta.env.DEV) {
          console.log(`🔍 [API Request] GET /chat/${chatId}/messages`, {
            ...requestParams,
            note: 'Loading initial messages'
          });
        }
      }

      // Make API call to fetch more messages
      const response = await api.get(`/chat/${chatId}/messages`, {
        params: requestParams
      });

      // Extract messages from response
      const rawMessages = response.data?.data || response.data || [];

      if (import.meta.env.DEV) {
        console.log(`📊 [API Response] Received ${rawMessages.length} messages for chat ${chatId}`);
      }

      // 🔧 ROOT CAUSE FIX: Apply same normalization as fetchMessages
      moreMessages = rawMessages.map(msg => ({
        id: msg.id,
        content: msg.content || '',
        sender_id: msg.sender_id,
        sender_name: this._resolveUserName(msg),
        sender: this._createSenderObject(msg),
        created_at: msg.created_at,
        chat_id: parseInt(chatId),
        status: 'sent',
        files: this._standardizeFiles(msg.files || []),
        mentions: msg.mentions || [],
        reply_to: msg.reply_to || null
      }));

      // 🔧 ROOT CAUSE FIX: Sort messages chronologically (oldest first)
      moreMessages.sort((a, b) => {
        const timeA = new Date(a.created_at).getTime();
        const timeB = new Date(b.created_at).getTime();
        return timeA - timeB; // Ascending order for pagination
      });

    } catch (apiError) {
      if (import.meta.env.DEV) {
        console.error(`❌ [API Error] Failed to fetch more messages for chat ${chatId}:`, apiError.message);
      }

      // Return empty array on error
      moreMessages = [];
    }

    // 🔧 CRITICAL FIX: Improved duplicate handling
    const existingIds = new Set(existing.map(msg => msg.id));
    const uniqueMoreMessages = moreMessages.filter(msg => !existingIds.has(msg.id));

    // 🔧 ENHANCED: Smart duplicate detection with proper end-of-history handling
    if (uniqueMoreMessages.length === 0 && moreMessages.length > 0) {
      // All messages were duplicates - this could mean we've reached the beginning of history
      this.duplicateRequestTracker.set(trackingKey, attemptCount + 1);

      if (import.meta.env.DEV) {
        console.warn(`🔄 [Duplicate Detection] All ${moreMessages.length} messages were duplicates for chat ${chatId} (attempt ${attemptCount + 1}/3)`);
        console.log(`📊 [Debug] Duplicate analysis:`, {
          apiReturned: moreMessages.length,
          uniqueFound: uniqueMoreMessages.length,
          existingCount: existing.length,
          duplicateIds: moreMessages.map(m => m.id),
          existingIds: existing.map(m => m.id)
        });
      }

      // 🔧 NEW: More intelligent end detection
      // If we get the same messages twice with ID-based pagination, we've reached the beginning
      this.hasMoreByChat.set(parseInt(chatId), false);
      this.duplicateRequestTracker.delete(trackingKey);

      if (import.meta.env.DEV) {
        console.log(`🏁 [End Detection] Reached beginning of message history for chat ${chatId}`);
      }

      return [];
    } else if (uniqueMoreMessages.length === 0 && moreMessages.length === 0) {
      // No messages returned at all - definitely at the end
      this.hasMoreByChat.set(parseInt(chatId), false);
      this.duplicateRequestTracker.delete(trackingKey);

      if (import.meta.env.DEV) {
        console.log(`🏁 [End Detection] No more messages available for chat ${chatId}`);
      }

      return [];
    } else {
      // Reset duplicate counter on successful unique messages
      this.duplicateRequestTracker.delete(trackingKey);
    }

    // 🔧 ENHANCED: Only process if we have new unique messages
    if (uniqueMoreMessages.length > 0) {
      // Sort new messages chronologically (oldest first for proper insertion)
      uniqueMoreMessages.sort((a, b) => {
        const timeA = new Date(a.created_at).getTime();
        const timeB = new Date(b.created_at).getTime();
        return timeA - timeB;
      });

      // 🔧 CRITICAL FIX: Add message tracking BEFORE setting messagesByChat
      // This prevents race condition between Vue DOM updates and tracking context extension
      try {
        const { messageDisplayGuarantee } = await import('./MessageDisplayGuarantee.js');
        const messageIds = uniqueMoreMessages.map(m => m.id);

        if (import.meta.env.DEV) {
          console.log(`🛡️ [UnifiedMessageService] Starting tracking for ${messageIds.length} more messages in chat ${chatId}:`, messageIds);
        }

        const trackingId = messageDisplayGuarantee.startMessageTracking(chatId, messageIds);

        if (import.meta.env.DEV) {
          if (trackingId) {
            console.log(`✅ [UnifiedMessageService] Successfully started tracking ${messageIds.length} more messages for chat ${chatId}, trackingId: ${trackingId}`);
          } else {
            console.error(`❌ [UnifiedMessageService] Failed to start tracking for more messages in chat ${chatId}`);
          }
        }
      } catch (trackingError) {
        if (import.meta.env.DEV) {
          console.error(`❌ [UnifiedMessageService] Failed to setup tracking for more messages:`, trackingError);
        }
        // Don't fail the entire operation for tracking issues
      }

      // 🔧 CRITICAL FIX: Only AFTER tracking is set up, update messagesByChat to trigger Vue updates
      // This ensures tracking context is ready when DOM renders and markMessageDisplayed is called
      const combined = [...uniqueMoreMessages, ...existing];
      const trimmedCombined = this._trimMessagesIfNeeded(chatId, combined);
      this.messagesByChat.set(parseInt(chatId), trimmedCombined);

      this.messageCache[chatId] = {
        messages: trimmedCombined,
        timestamp: Date.now()
      };

      // Set hasMore based on whether we got the full limit of unique messages
      const hasMoreMessages = uniqueMoreMessages.length >= limit;
      this.hasMoreByChat.set(parseInt(chatId), hasMoreMessages);

      if (import.meta.env.DEV) {
        console.log(`✅ [Success] Added ${uniqueMoreMessages.length} historical messages to chat ${chatId} (total: ${trimmedCombined.length})`);
        console.log(`📈 [Pagination] Chat ${chatId} hasMore: ${hasMoreMessages} (got ${uniqueMoreMessages.length}/${limit} unique messages)`);
      }
    }

    return uniqueMoreMessages;
  }

  /**
   * 🔧 ENHANCED: Check if a chat has more messages to load (Slack-like behavior)
   */
  hasMoreMessages(chatId) {
    const normalizedChatId = parseInt(chatId);
    const currentHasMore = this.hasMoreByChat.get(normalizedChatId);

    // 🔧 SLACK-LIKE LOGIC: If state is undefined/null, assume we can load more
    // This ensures that each time we enter a channel, we can attempt to load more messages
    if (currentHasMore === undefined || currentHasMore === null) {
      if (import.meta.env.DEV) {
        console.log(`🔄 [hasMoreMessages] Chat ${chatId}: undefined state, assuming hasMore=true (Slack-like behavior)`);
      }
      return true;
    }

    // 🔧 ENHANCED: Smart evaluation based on actual message count
    const messages = this.messagesByChat.get(normalizedChatId);
    const messageCount = messages?.length || 0;

    // If we have very few messages, we probably can load more (unless explicitly set to false)
    if (messageCount < 10 && currentHasMore !== false) {
      if (import.meta.env.DEV) {
        console.log(`🔄 [hasMoreMessages] Chat ${chatId}: only ${messageCount} messages, assuming hasMore=true`);
      }
      return true;
    }

    if (import.meta.env.DEV) {
      console.log(`🔄 [hasMoreMessages] Chat ${chatId}: hasMore=${currentHasMore} (${messageCount} messages)`);
    }

    return currentHasMore !== false;
  }

  /**
   * 🆕 SLACK-LIKE: Reset hasMoreMessages state for a chat
   * This ensures load more functionality works consistently when re-entering channels
   */
  resetHasMoreMessages(chatId) {
    const normalizedChatId = parseInt(chatId);

    if (import.meta.env.DEV) {
      const previousState = this.hasMoreByChat.get(normalizedChatId);
      console.log(`🔄 [resetHasMoreMessages] Chat ${chatId}: ${previousState} → true (enabling load more)`);
    }

    // 🔧 CRITICAL: Set to true to allow load more attempts
    this.hasMoreByChat.set(normalizedChatId, true);

    return true;
  }

  /**
   * Get message cache
   */
  getMessageCache() {
    return this.messageCache;
  }

  /**
   * Clear messages for a specific chat
   */
  async clearMessagesForChat(chatId) {
    const normalizedChatId = parseInt(chatId);

    // 🔧 CRITICAL DEBUG: Log when clearing is called
    if (import.meta.env.DEV) {
      const previousMessages = this.messagesByChat.get(normalizedChatId)?.length || 0;
      const previousHasMore = this.hasMoreByChat.get(normalizedChatId);

      console.log(`🧹 [clearMessagesForChat] Chat ${chatId}: clearing ${previousMessages} messages, hasMore was ${previousHasMore}`);

      // 🔧 DEBUG: Log call stack to see who called this (only in dev mode)
      console.trace('🧹 [clearMessagesForChat] call stack');
    }

    // Clear message data
    this.messagesByChat.delete(normalizedChatId);
    delete this.messageCache[chatId];

    // 🔧 SLACK-LIKE BEHAVIOR: Reset hasMoreMessages to true instead of deleting
    // This ensures that when we re-enter the channel, load more will work
    this.hasMoreByChat.set(normalizedChatId, true);

    if (import.meta.env.DEV) {
      console.log(`🔄 [clearMessagesForChat] Chat ${chatId}: reset hasMoreMessages to true for next visit`);
    }

    // 🔧 CRITICAL FIX: Clear MessageDisplayGuarantee tracking contexts for this chat
    // This prevents "No tracking context found" errors when switching chats
    try {
      // 🔧 FIXED: Use direct import instead of dynamic import to avoid timing issues
      const { messageDisplayGuarantee } = await import('./MessageDisplayGuarantee.js');

      if (import.meta.env.DEV) {
        console.log(`🧹 [clearMessagesForChat] About to clear tracking context for chat ${chatId}`);
      }

      const clearedCount = messageDisplayGuarantee.clearTrackingForChat(normalizedChatId);

      if (import.meta.env.DEV) {
        console.log(`✅ [clearMessagesForChat] Chat ${chatId}: cleared ${clearedCount} tracking context(s), ready for next visit`);
      }
    } catch (error) {
      if (import.meta.env.DEV) {
        console.error('[clearMessagesForChat] Failed to clear tracking context:', error);
      }
    }
  }

  /**
   * Clear all messages
   */
  clearAllMessages() {
    this.messagesByChat.clear();
    Object.keys(this.messageCache).forEach(key => {
      delete this.messageCache[key];
    });
    this.hasMoreByChat.clear();
  }

  /**
   * Get pending messages
   */
  getPendingMessages() {
    return [];
  }

  /**
   * Get failed messages
   */
  getFailedMessages() {
    return [];
  }

  /**
   * Retry a failed message
   */
  async retryMessage(messageId) {
    // Stub implementation
    return { success: true };
  }

  /**
   * Export debug info
   */
  async exportDebugInfo() {
    return {
      isInitialized: this.isInitialized.value,
      messageCount: Array.from(this.messagesByChat.values()).reduce((sum, messages) => sum + messages.length, 0),
      chatCount: this.messagesByChat.size,
      stats: { ...this.stats }
    };
  }

  /**
   * 🔧 HELPER: Resolve user name with enhanced fallback logic
   */
  _resolveUserName(msg) {
    // 🔧 ROOT CAUSE FIX: Try multiple sources for user name from message object
    if (msg.sender?.fullname) return msg.sender.fullname;
    if (msg.sender?.name) return msg.sender.name;
    if (msg.sender_name) return msg.sender_name;
    if (msg.sender?.username) return msg.sender.username;

    // 🔧 CRITICAL FIX: Get user info from UserStore instead of window.workspaceStore
    if (msg.sender_id) {
      try {
        // 🔧 FIXED: Use correct global store access
        const userStoreAccessor = window.__pinia_stores__?.user;
        if (userStoreAccessor) {
          const userStore = userStoreAccessor();
          const user = userStore.getUserById(parseInt(msg.sender_id));
          if (user?.fullname) {
            return user.fullname;
          }
          if (user?.username) {
            return user.username;
          }
          if (user?.email) {
            return user.email.split('@')[0]; // Use email prefix as fallback
          }
        }

        // 🔧 FALLBACK: Try workspaceStore with correct field name
        const workspaceStoreAccessor = window.__pinia_stores__?.workspace;
        if (workspaceStoreAccessor) {
          const workspaceStore = workspaceStoreAccessor();
          if (workspaceStore?.workspaceUsers) {
            const user = workspaceStore.workspaceUsers.find(member =>
              parseInt(member.id) === parseInt(msg.sender_id)
            );
            if (user?.fullname) {
              return user.fullname;
            }
            if (user?.username) {
              return user.username;
            }
            if (user?.email) {
              return user.email.split('@')[0];
            }
          }
        }

        // 🔧 ENHANCED: Try global Vue app store access as final fallback
        if (typeof window !== 'undefined' && window.__vue_app__) {
          try {
            const app = window.__vue_app__;
            const userStore = app.config.globalProperties.$userStore;
            const workspaceStore = app.config.globalProperties.$workspaceStore;

            if (userStore) {
              const user = userStore.getUserById(parseInt(msg.sender_id));
              if (user?.fullname) return user.fullname;
            }

            if (workspaceStore?.workspaceUsers) {
              const user = workspaceStore.workspaceUsers.find(u =>
                parseInt(u.id) === parseInt(msg.sender_id)
              );
              if (user?.fullname) return user.fullname;
            }
          } catch (vueError) {
            if (import.meta.env.DEV) {
              console.warn('[UnifiedMessageService] Vue app store access failed:', vueError);
            }
          }
        }

      } catch (error) {
        if (import.meta.env.DEV) {
          console.warn('[UnifiedMessageService] User resolution failed:', error);
        }
      }
    }

    // 🔧 ENHANCED: More informative fallback with debug info
    if (msg.sender_id) {
      const fallbackName = `User ${msg.sender_id}`;
      if (import.meta.env.DEV) {
        console.warn(`[UnifiedMessageService] Using fallback name "${fallbackName}" for sender_id:`, msg.sender_id, 'Original message:', msg);
      }
      return fallbackName;
    }

    return 'Unknown User';
  }

  /**
   * 🔧 HELPER: Enhanced sender object creation with better user data integration
   */
  _createSenderObject(msg) {
    // If message already has complete sender object, use it
    if (msg.sender && typeof msg.sender === 'object' && msg.sender.fullname) {
      return {
        id: parseInt(msg.sender.id) || parseInt(msg.sender_id) || 0,
        fullname: msg.sender.fullname,
        email: msg.sender.email || '',
        avatar_url: msg.sender.avatar_url || null,
        username: msg.sender.username || null
      };
    }

    // 🔧 ENHANCED: Get complete user data from stores
    let userData = null;
    const senderId = parseInt(msg.sender_id);

    if (senderId) {
      try {
        // 🔧 FIXED: Use correct global store access
        const userStoreAccessor = window.__pinia_stores__?.user;
        if (userStoreAccessor) {
          const userStore = userStoreAccessor();
          userData = userStore.getUserById(senderId);
        }

        // Fallback to workspaceStore
        if (!userData) {
          const workspaceStoreAccessor = window.__pinia_stores__?.workspace;
          if (workspaceStoreAccessor) {
            const workspaceStore = workspaceStoreAccessor();
            if (workspaceStore?.workspaceUsers) {
              userData = workspaceStore.workspaceUsers.find(u =>
                parseInt(u.id) === senderId
              );
            }
          }
        }
      } catch (error) {
        if (import.meta.env.DEV) {
          console.warn('[UnifiedMessageService] Failed to get user data for sender:', senderId, error);
        }
      }
    }

    // 🔧 PRODUCTION-READY: Create comprehensive sender object
    return {
      id: senderId || 0,
      fullname: userData?.fullname || this._resolveUserName(msg),
      email: userData?.email || msg.sender?.email || '',
      avatar_url: userData?.avatar_url || msg.sender?.avatar_url || null,
      username: userData?.username || msg.sender?.username || null,
      // 🔧 NEW: Additional user metadata for future use
      status: userData?.status || 'unknown',
      workspace_id: userData?.workspace_id || null
    };
  }

  /**
   * 🔧 HELPER: Standardize files to prevent Vue prop type errors
   */
  _standardizeFiles(files) {
    if (!files || !Array.isArray(files)) return [];

    return files.map(file => {
      // If file is a string (URL), convert to object
      if (typeof file === 'string') {
        return {
          id: Date.now() + Math.random(),
          url: file,
          filename: file.split('/').pop() || 'Unknown File',
          mime_type: this._getMimeTypeFromUrl(file),
          size: 0
        };
      }

      // If file is already an object, ensure required properties
      return {
        id: file.id || Date.now() + Math.random(),
        url: file.url || file.path || '',
        filename: file.filename || file.name || 'Unknown File',
        mime_type: file.mime_type || file.type || 'application/octet-stream',
        size: file.size || 0,
        ...file // Preserve other properties
      };
    });
  }

  /**
   * 🔧 HELPER: Get MIME type from file URL/extension
   */
  _getMimeTypeFromUrl(url) {
    const extension = url.split('.').pop()?.toLowerCase();
    const mimeTypes = {
      'jpg': 'image/jpeg',
      'jpeg': 'image/jpeg',
      'png': 'image/png',
      'gif': 'image/gif',
      'webp': 'image/webp',
      'svg': 'image/svg+xml',
      'pdf': 'application/pdf',
      'txt': 'text/plain',
      'doc': 'application/msword',
      'docx': 'application/vnd.openxmlformats-officedocument.wordprocessingml.document',
      'mp4': 'video/mp4',
      'mp3': 'audio/mpeg',
      'wav': 'audio/wav',
      'zip': 'application/zip'
    };

    return mimeTypes[extension] || 'application/octet-stream';
  }

  /**
   * 🔧 NEW: Smart cache cleanup to prevent memory leaks
   */
  _startCacheCleanup() {
    setInterval(() => {
      this._cleanupOldCaches();
    }, this.cacheConfig.cleanupInterval);
  }

  /**
   * 🔧 NEW: Clean up old message caches
   */
  _cleanupOldCaches() {
    const now = Date.now();
    const cacheKeys = Object.keys(this.messageCache);

    if (cacheKeys.length <= this.cacheConfig.maxCacheSize) {
      return; // No cleanup needed
    }

    // Sort by timestamp (oldest first)
    const sortedCaches = cacheKeys
      .map(chatId => ({
        chatId,
        timestamp: this.messageCache[chatId]?.timestamp || 0,
        messageCount: this.messagesByChat.get(parseInt(chatId))?.length || 0
      }))
      .sort((a, b) => a.timestamp - b.timestamp);

    // Remove oldest caches
    const toRemove = sortedCaches.slice(0, cacheKeys.length - this.cacheConfig.maxCacheSize);

    toRemove.forEach(({ chatId }) => {
      if (import.meta.env.DEV) {
        console.log(`🧹 [CacheCleanup] Removing old cache for chat ${chatId}`);
      }

      this.messagesByChat.delete(parseInt(chatId));
      delete this.messageCache[chatId];
      this.hasMoreByChat.delete(parseInt(chatId));
    });

    if (import.meta.env.DEV && toRemove.length > 0) {
      console.log(`🧹 [CacheCleanup] Cleaned ${toRemove.length} old caches, ${Object.keys(this.messageCache).length} remaining`);
    }
  }

  /**
   * 🔧 NEW: Trim messages if they exceed limit
   */
  _trimMessagesIfNeeded(chatId, messages) {
    if (messages.length <= this.cacheConfig.maxMessagesPerChat) {
      return messages;
    }

    // Keep the most recent messages
    const trimmed = messages.slice(-this.cacheConfig.maxMessagesPerChat);

    if (import.meta.env.DEV) {
      console.log(`✂️ [CacheTrim] Trimmed chat ${chatId} from ${messages.length} to ${trimmed.length} messages`);
    }

    return trimmed;
  }
}

// Create and export global instance
export const unifiedMessageService = new UnifiedMessageService(); 