/**
 * FIXED: UnifiedMessageService - Eliminates Recursive Update Issues
 */

import { ref } from 'vue';
import api from '@/services/api';

export class UnifiedMessageServiceFixed {
  constructor() {
    this.isInitialized = ref(false);
    this.messagesByChat = new Map();
    this.messageCache = {};
    this.hasMoreByChat = new Map();

    // FIX: Add anti-duplicate call protection
    this.activeRequests = new Map();
    this.lastRequestTime = new Map();
    this.requestDebounceTime = 500; // 500ms debounce

    this.cacheConfig = {
      maxCacheSize: 10,
      maxMessagesPerChat: 500,
      cacheExpiration: 5 * 60 * 1000
    };

    this.stats = {
      totalFetches: 0,
      cachehits: 0,
      errors: 0
    };

    this.isInitialized.value = true;
  }

  /**
   * FIXED: Debounced message fetching
   */
  async fetchMessages(chatId, options = {}) {
    const { limit = 50, forceRefresh = false } = options;
    const normalizedChatId = parseInt(chatId);

    // Anti-duplicate protection
    if (this._shouldSkipRequest(normalizedChatId, 'fetchMessages')) {
      return this.getMessagesForChat(normalizedChatId) || [];
    }

    // Check cache
    if (!forceRefresh) {
      const cached = this._getCachedMessages(normalizedChatId);
      if (cached) {
        this.stats.cachehits++;
        return cached;
      }
    }

    try {
      this._markRequestStart(normalizedChatId, 'fetchMessages');
      this.stats.totalFetches++;

      const response = await api.get(`/api/chats/${normalizedChatId}/messages`, {
        params: { limit, offset: 0 }
      });

      const rawMessages = response.data?.messages || [];
      const messages = rawMessages.map(msg => this._normalizeMessage(msg, normalizedChatId));

      messages.sort((a, b) => new Date(a.created_at) - new Date(b.created_at));

      this._updateMessageCache(normalizedChatId, messages);
      this.hasMoreByChat.set(normalizedChatId, messages.length >= limit);

      return messages;

    } catch (error) {
      this.stats.errors++;
      return this.getMessagesForChat(normalizedChatId) || [];
    } finally {
      this._markRequestEnd(normalizedChatId, 'fetchMessages');
    }
  }

  /**
   * FIXED: Removed event dispatching that caused loops
   */
  async fetchMoreMessages(chatId, options = {}) {
    const { limit = 20 } = options;
    const normalizedChatId = parseInt(chatId);

    if (this._shouldSkipRequest(normalizedChatId, 'fetchMoreMessages')) {
      return [];
    }

    if (!this.hasMoreMessages(normalizedChatId)) {
      return [];
    }

    const existing = this.getMessagesForChat(normalizedChatId) || [];

    try {
      this._markRequestStart(normalizedChatId, 'fetchMoreMessages');

      const oldestMessage = existing[0];
      const beforeId = oldestMessage ? oldestMessage.id : null;

      const response = await api.get(`/api/chats/${normalizedChatId}/messages`, {
        params: { limit, before: beforeId }
      });

      const rawMessages = response.data?.messages || [];
      let moreMessages = rawMessages.map(msg => this._normalizeMessage(msg, normalizedChatId));

      const existingIds = new Set(existing.map(msg => msg.id));
      const uniqueMoreMessages = moreMessages.filter(msg => !existingIds.has(msg.id));

      this.hasMoreByChat.set(normalizedChatId, rawMessages.length >= limit);

      if (uniqueMoreMessages.length > 0) {
        const combined = [...uniqueMoreMessages, ...existing];
        this._updateMessageCache(normalizedChatId, combined);
      }

      // REMOVED: No more event dispatching - components use Promise completion
      return uniqueMoreMessages;

    } catch (error) {
      this.stats.errors++;
      return [];
    } finally {
      this._markRequestEnd(normalizedChatId, 'fetchMoreMessages');
    }
  }

  _shouldSkipRequest(chatId, requestType) {
    const requestKey = `${chatId}-${requestType}`;
    const now = Date.now();

    if (this.activeRequests.has(requestKey)) return true;

    const lastRequestTime = this.lastRequestTime.get(requestKey);
    if (lastRequestTime && (now - lastRequestTime) < this.requestDebounceTime) {
      return true;
    }

    return false;
  }

  _markRequestStart(chatId, requestType) {
    const requestKey = `${chatId}-${requestType}`;
    this.activeRequests.set(requestKey, true);
    this.lastRequestTime.set(requestKey, Date.now());
  }

  _markRequestEnd(chatId, requestType) {
    const requestKey = `${chatId}-${requestType}`;
    this.activeRequests.delete(requestKey);
  }

  _getCachedMessages(chatId) {
    const cache = this.messageCache[chatId];
    if (!cache) return null;

    const now = Date.now();
    if (now - cache.timestamp > this.cacheConfig.cacheExpiration) {
      delete this.messageCache[chatId];
      this.messagesByChat.delete(chatId);
      return null;
    }

    return cache.messages;
  }

  _updateMessageCache(chatId, messages) {
    this.messagesByChat.set(chatId, messages);
    this.messageCache[chatId] = {
      messages,
      timestamp: Date.now()
    };
  }

  _normalizeMessage(msg, chatId) {
    return {
      id: msg.id,
      content: msg.content || '',
      sender_id: msg.sender_id,
      sender_name: this._resolveUserName(msg),
      sender: this._createSenderObject(msg),
      created_at: msg.created_at,
      chat_id: parseInt(chatId),
      status: 'sent',
      files: msg.files || [],
      mentions: msg.mentions || [],
      reply_to: msg.reply_to || null
    };
  }

  _resolveUserName(msg) {
    if (msg.sender?.fullname) return msg.sender.fullname;
    if (msg.sender_name) return msg.sender_name;
    return msg.sender_id ? `User ${msg.sender_id}` : 'Unknown User';
  }

  _createSenderObject(msg) {
    return {
      id: parseInt(msg.sender_id) || 0,
      fullname: this._resolveUserName(msg),
      email: msg.sender?.email || '',
      avatar_url: msg.sender?.avatar_url || null,
      username: msg.sender?.username || null
    };
  }

  getMessagesForChat(chatId) {
    return this.messagesByChat.get(parseInt(chatId)) || [];
  }

  hasMoreMessages(chatId) {
    return this.hasMoreByChat.get(parseInt(chatId)) !== false;
  }

  async clearMessagesForChat(chatId) {
    const normalizedChatId = parseInt(chatId);
    this.messagesByChat.delete(normalizedChatId);
    delete this.messageCache[normalizedChatId];
    this.hasMoreByChat.delete(normalizedChatId);

    // Clear active requests
    const keysToRemove = [];
    this.activeRequests.forEach((value, key) => {
      if (key.startsWith(`${normalizedChatId}-`)) {
        keysToRemove.push(key);
      }
    });

    keysToRemove.forEach(key => {
      this.activeRequests.delete(key);
      this.lastRequestTime.delete(key);
    });
  }

  getStats() {
    return {
      ...this.stats,
      cacheSize: Object.keys(this.messageCache).length,
      totalChats: this.messagesByChat.size,
      activeRequests: this.activeRequests.size
    };
  }
}

export const unifiedMessageServiceFixed = new UnifiedMessageServiceFixed();
export default unifiedMessageServiceFixed; 