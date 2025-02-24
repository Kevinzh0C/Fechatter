/**
 * 🎯 Refactored Chat Store - Using Unified Message Service
 * 
 * Simplified chat store that delegates message management to UnifiedMessageService
 * Provides clean interface for chat management with complete closed-loop message logic
 */

import { defineStore } from 'pinia';
import { computed, ref, reactive } from 'vue';
import api from '@/services/api';
import { useAuthStore } from '@/stores/auth';
import { useUserStore } from '@/stores/user';
import { errorHandler } from '@/utils/errorHandler';
import { unifiedMessageService, MessageState } from '@/services/messageSystem/UnifiedMessageService.js';

export const useChatStore = defineStore('chat', {
  state: () => ({
    // Chat management
    chats: [],
    currentChatId: null,
    loading: false,
    error: null,

    // UI state
    isInitialized: false,

    // Cache for chat metadata
    chatMembers: {},

    // Upload state
    uploadProgress: 0,
  }),

  getters: {
    // Current chat getter
    getCurrentChat: (state) => {
      return state.chats.find(chat => chat.id === state.currentChatId);
    },

    // Get chat members
    getChatMembers: (state) => (chatId) => {
      return state.chatMembers[chatId] || [];
    },

    // Get chat by ID
    getChatById: (state) => (chatId) => {
      return state.chats.find(chat => chat.id === chatId);
    },

    // Get current user ID
    getCurrentUserId: () => {
      const authStore = useAuthStore();
      return authStore.user?.id;
    },

    // Get messages for current chat using unified service
    messages: (state) => {
      if (!state.currentChatId) return [];
      return unifiedMessageService.getMessagesForChat(state.currentChatId);
    },

    // Get pending messages for UI indicators
    pendingMessages: () => {
      return unifiedMessageService.getPendingMessages();
    },

    // Get failed messages for retry UI
    failedMessages: () => {
      return unifiedMessageService.getFailedMessages();
    },

    // Get visible chats (can add filtering logic here)
    visibleChats: (state) => {
      return state.chats.filter(chat => chat.is_active !== false);
    },

    // Get message service status
    messageServiceStatus: () => {
      return unifiedMessageService.getStatus();
    },

    // Check if current chat has more messages to load
    hasMoreMessages: (state) => {
      if (!state.currentChatId) return false;

      // 🔧 CRITICAL FIX: Use UnifiedMessageService's hasMoreMessages method
      try {
        return unifiedMessageService.hasMoreMessages(state.currentChatId);
      } catch (error) {
        if (import.meta.env.DEV) {
          console.warn(`[ChatStore] Failed to check hasMoreMessages for chat ${state.currentChatId}:`, error);
        }
        return false;
      }
    },

    // Get message cache for compatibility
    messageCache: () => {
      return unifiedMessageService.getMessageCache();
    }
  },

  actions: {
    /**
     * Initialize the chat store
     */
    async initialize() {
      if (this.isInitialized) return;

      try {
        if (import.meta.env.DEV) {
          console.log('🎯 Initializing Chat Store (Refactored)...');
        }

        // Wait for unified message service to initialize
        if (!unifiedMessageService.isInitialized.value) {
          if (import.meta.env.DEV) {
            console.log('⏳ Waiting for message service to initialize...');
          }
          await new Promise(resolve => {
            const unwatch = unifiedMessageService.isInitialized.value ? resolve() :
              unifiedMessageService.$watch('isInitialized', (newVal) => {
                if (newVal) {
                  unwatch?.();
                  resolve();
                }
              });
          });
        }

        // Fetch initial chat list
        await this.fetchChats();

        this.isInitialized = true;
        if (import.meta.env.DEV) {
          console.log('✅ Chat Store (Refactored) initialized');
        }

      } catch (error) {
        if (import.meta.env.DEV) {
          console.error('Failed to initialize chat store:', error);
        }
        this.error = error.message;
      }
    },

    /**
     * Fetch chats from server
     */
    async fetchChats() {
      this.loading = true;
      this.error = null;

      try {
        const response = await api.get('/workspace/chats');

        // Handle API response formats
        let chatsData = [];
        const responseData = response.data;

        if (responseData) {
          if (responseData.data && Array.isArray(responseData.data)) {
            chatsData = responseData.data;
          } else if (Array.isArray(responseData)) {
            chatsData = responseData;
          } else if (responseData.chats && Array.isArray(responseData.chats)) {
            chatsData = responseData.chats;
          }
        }

        // Normalize chat data
        this.chats = chatsData.map(chat => this.normalizeChat(chat));

        // Cache to localStorage
        this.cacheChats();

        return this.chats;

      } catch (error) {
        errorHandler.handle(error, {
          context: 'Fetch chats',
          silent: false
        });
        this.error = error.response?.data?.message || 'Failed to fetch chats';
        throw error;
      } finally {
        this.loading = false;
      }
    },

    /**
     * Set current chat and load messages
     */
    async setCurrentChat(chatId) {
      if (this.currentChatId === chatId) {
        if (import.meta.env.DEV) {
          console.log(`🎯 Already in chat ${chatId}, but resetting hasMoreMessages state for consistency`);
        }

        // 🔧 CRITICAL FIX: Even for same chat, reset hasMoreMessages for load more functionality
        try {
          unifiedMessageService.resetHasMoreMessages(parseInt(chatId));
          if (import.meta.env.DEV) {
            console.log(`✅ Reset hasMoreMessages state for chat ${chatId}`);
          }
        } catch (error) {
          if (import.meta.env.DEV) {
            console.warn('Failed to reset hasMoreMessages:', error);
          }
        }
        return;
      }

      if (import.meta.env.DEV) {
        console.log(`🎯 Switching to chat: ${chatId}`);
      }

      // Set current chat
      this.currentChatId = parseInt(chatId);

      // 🔧 CRITICAL FIX: Reset hasMoreMessages state when switching chats
      // This ensures "load more messages" works consistently every time we enter a channel
      try {
        unifiedMessageService.resetHasMoreMessages(parseInt(chatId));
        if (import.meta.env.DEV) {
          console.log(`🔄 [ChatStore] Reset hasMoreMessages state for chat ${chatId} (Slack-like behavior)`);
        }
      } catch (error) {
        if (import.meta.env.DEV) {
          console.warn('[ChatStore] Failed to reset hasMoreMessages:', error);
        }
      }

      // Fetch chat members if not cached
      if (!this.chatMembers[chatId]) {
        try {
          await this.fetchChatMembers(chatId);
        } catch (error) {
          if (import.meta.env.DEV) {
            console.warn('Failed to fetch chat members:', error);
          }
        }
      }

      // Reset chat unread count
      this.resetChatUnreadCount(chatId);
    },

    /**
     * Send message using unified service
     */
    async sendMessage(content, options = {}) {
      if (!this.currentChatId) {
        throw new Error('No chat selected');
      }

      try {
        if (import.meta.env.DEV) {
          console.log(`📤 Sending message to chat ${this.currentChatId}:`, content);
        }

        const result = await unifiedMessageService.sendMessage(
          content,
          this.currentChatId,
          {
            files: options.files || [],
            mentions: options.mentions || [],
            replyTo: options.replyTo || null,
            priority: options.priority
          }
        );

        // Update chat's last message
        this.updateChatLastMessage(result.message);

        return result;

      } catch (error) {
        if (import.meta.env.DEV) {
          console.error('Failed to send message:', error);
        }
        errorHandler.handle(error, {
          context: 'Send message',
          silent: false
        });
        throw error;
      }
    },

    /**
     * Retry failed message
     */
    async retryMessage(messageId) {
      try {
        if (import.meta.env.DEV) {
          console.log(`🔄 Retrying message: ${messageId}`);
        }

        const result = await unifiedMessageService.retryMessage(messageId);

        if (result) {
          if (import.meta.env.DEV) {
            console.log(`✅ Message retry queued: ${messageId}`);
          }
        }

        return result;

      } catch (error) {
        if (import.meta.env.DEV) {
          console.error('Failed to retry message:', error);
        }
        errorHandler.handle(error, {
          context: 'Retry message',
          silent: false
        });
        throw error;
      }
    },

    /**
     * Handle incoming real-time message (called by SSE service)
     */
    handleIncomingMessage(message) {
      if (import.meta.env.DEV) {
        console.log(`📨 Handling incoming message for chat ${message.chat_id}`);
      }

      // The unified message service handles all the logic
      // We just need to update chat metadata here

      const chat = this.getChatById(message.chat_id);
      if (chat) {
        // Update chat's last message
        this.updateChatLastMessage(message);

        // Update unread count if not current chat
        if (message.chat_id !== this.currentChatId) {
          this.incrementChatUnreadCount(message.chat_id);

          // Move chat to top
          const chatIndex = this.chats.findIndex(c => c.id === message.chat_id);
          if (chatIndex > 0) {
            const [movedChat] = this.chats.splice(chatIndex, 1);
            this.chats.unshift(movedChat);
          }
        }
      }
    },

    /**
     * Update chat's last message
     */
    updateChatLastMessage(message) {
      const chat = this.getChatById(message.chat_id);
      if (chat) {
        chat.last_message = {
          id: message.id,
          content: message.content,
          created_at: message.created_at,
          sender_id: message.sender_id
        };
        chat.updated_at = message.created_at;
      }
    },

    /**
     * Create new chat
     */
    async createChat(name, members = [], description = '', chatType = 'PrivateChannel') {
      this.loading = true;
      this.error = null;

      try {
        const payload = {
          name,
          chat_type: chatType,
          members,
          description
        };

        const response = await api.post('/workspace/chats', payload);
        const newChat = this.normalizeChat(response.data?.data || response.data);

        // Add to local chats
        this.chats.unshift(newChat);

        // Cache updated chats
        this.cacheChats();

        return newChat;

      } catch (error) {
        errorHandler.handle(error, {
          context: 'Create chat',
          silent: false
        });
        this.error = error.response?.data?.message || 'Failed to create chat';
        throw error;
      } finally {
        this.loading = false;
      }
    },

    /**
     * Find existing DM with user
     */
    async findExistingDM(userId) {
      try {
        const authStore = useAuthStore();
        const currentUserId = authStore.user?.id;

        if (!currentUserId || currentUserId === userId) {
          return null;
        }

        // Search in local chats first
        const existingDM = this.chats.find(chat => {
          return chat.chat_type === 'Single' &&
            chat.chat_members &&
            chat.chat_members.includes(userId) &&
            chat.chat_members.includes(currentUserId);
        });

        if (existingDM) {
          return existingDM;
        }

        // If not found locally, refresh and try again
        await this.fetchChats();

        return this.chats.find(chat => {
          return chat.chat_type === 'Single' &&
            chat.chat_members &&
            chat.chat_members.includes(userId) &&
            chat.chat_members.includes(currentUserId);
        });

      } catch (error) {
        errorHandler.handle(error, {
          context: 'Find existing DM',
          silent: true
        });
        return null;
      }
    },

    /**
     * Fetch chat members
     */
    async fetchChatMembers(chatId) {
      try {
        const response = await api.get(`/chat/${chatId}/members`);
        this.chatMembers[chatId] = response.data;
        return response.data;
      } catch (error) {
        errorHandler.handle(error, {
          context: 'Fetch chat members',
          silent: true
        });
        return [];
      }
    },

    /**
     * File upload handling
     */
    async uploadFiles(files) {
      if (!files || files.length === 0) return [];

      const formData = new FormData();
      files.forEach(file => {
        formData.append('file', file);
      });

      try {
        this.uploadProgress = 0;

        const response = await api.post('/files/single', formData, {
          onUploadProgress: (progressEvent) => {
            this.uploadProgress = Math.round(
              (progressEvent.loaded * 100) / progressEvent.total
            );
          }
        });

        return response.data || [];

      } catch (error) {
        errorHandler.handle(error, {
          context: 'File upload',
          silent: false
        });
        throw error;
      } finally {
        this.uploadProgress = 0;
      }
    },

    /**
     * Unread count management
     */
    updateChatUnreadCount(chatId, count) {
      const chat = this.getChatById(chatId);
      if (chat) {
        chat.unread_count = count;
      }
    },

    incrementChatUnreadCount(chatId) {
      const chat = this.getChatById(chatId);
      if (chat) {
        chat.unread_count = (chat.unread_count || 0) + 1;
      }
    },

    resetChatUnreadCount(chatId) {
      this.updateChatUnreadCount(chatId, 0);
    },

    /**
     * Normalize chat data
     */
    normalizeChat(chat) {
      const userStore = useUserStore();
      const authStore = useAuthStore();

      const normalizedChat = {
        id: chat.id,
        name: chat.name || 'Unnamed Chat',
        chat_type: this.normalizeChatType(chat.chat_type),
        description: chat.description || '',
        created_at: chat.created_at,
        updated_at: chat.updated_at,
        chat_members: chat.chat_members || [],
        member_count: chat.member_count || chat.chat_members?.length || 0,
        creator_id: chat.creator_id,
        is_active: chat.is_active !== false,
        unread_count: chat.unread_count || 0,
        last_message: chat.last_message || null,
        display_name: chat.name,
        display_avatar: null
      };

      // Special handling for DM chats
      if (normalizedChat.chat_type === 'Single') {
        normalizedChat.display_name = chat.name;

        // Find the other user for avatar and name
        const otherMemberId = chat.chat_members?.find(id => id !== authStore.user?.id);
        if (otherMemberId) {
          const user = userStore.getUserById(otherMemberId);
          if (user) {
            normalizedChat.display_name = user.fullname;
            normalizedChat.display_avatar = user.avatar_url;
          }
        }

        if (normalizedChat.display_name === 'Direct Message' || !normalizedChat.display_name) {
          normalizedChat.display_name = 'Unknown User';
        }
      }

      return normalizedChat;
    },

    /**
     * Normalize chat type
     */
    normalizeChatType(chatType) {
      if (!chatType) return 'PublicChannel';

      const lowerType = chatType.toLowerCase();
      const typeMap = {
        'publicchannel': 'PublicChannel',
        'public_channel': 'PublicChannel',
        'public': 'PublicChannel',
        'privatechannel': 'PrivateChannel',
        'private_channel': 'PrivateChannel',
        'private': 'PrivateChannel',
        'single': 'Single',
        'dm': 'Single',
        'direct': 'Single',
        'directmessage': 'Single',
        'direct_message': 'Single'
      };

      return typeMap[lowerType] || chatType;
    },

    /**
     * Cache chats to localStorage
     */
    cacheChats() {
      try {
        localStorage.setItem('chatsCache', JSON.stringify({
          timestamp: Date.now(),
          chats: this.chats
        }));
      } catch (error) {
        if (import.meta.env.DEV) {
          console.warn('Failed to cache chats:', error);
        }
      }
    },

    /**
     * Load chats from cache
     */
    loadChatsFromCache() {
      try {
        const cached = localStorage.getItem('chatsCache');
        if (cached) {
          const { timestamp, chats } = JSON.parse(cached);
          // Cache valid for 10 minutes
          if (Date.now() - timestamp < 10 * 60 * 1000) {
            this.chats = Array.isArray(chats) ? chats : [];
            return true;
          }
        }
      } catch (error) {
        if (import.meta.env.DEV) {
          console.warn('Failed to load chats from cache:', error);
        }
      }
      return false;
    },

    /**
     * Clear all data
     */
    clearAll() {
      this.chats = [];
      this.currentChatId = null;
      this.chatMembers = {};
      this.error = null;
      this.loading = false;

      // Clear unified message service
      unifiedMessageService.clearAllMessages();

      // Clear cache
      localStorage.removeItem('chatsCache');

      if (import.meta.env.DEV) {
        console.log('🧹 Chat store cleared');
      }
    },

    /**
     * Get debug information
     */
    async getDebugInfo() {
      const messageServiceDebug = await unifiedMessageService.exportDebugInfo();

      return {
        timestamp: new Date().toISOString(),
        chatStore: {
          chatsCount: this.chats.length,
          currentChatId: this.currentChatId,
          isInitialized: this.isInitialized,
          loading: this.loading,
          error: this.error
        },
        messageService: messageServiceDebug
      };
    },

    /**
     * Fetch messages with signal for abort control
     */
    async fetchMessagesWithSignal(chatId, abortSignal = null, limit = 15, isPreload = false) {
      try {
        if (import.meta.env.DEV && !isPreload) {
          console.log(`📥 Fetching messages for chat ${chatId} with limit ${limit}`);
        }

        // Use unified message service to fetch messages
        const result = await unifiedMessageService.fetchMessages(
          chatId,
          {
            limit,
            abortSignal,
            isPreload
          }
        );

        return result || [];

      } catch (error) {
        if (error.name === 'AbortError') {
          if (import.meta.env.DEV && !isPreload) {
            console.log(`🚫 Fetch aborted for chat ${chatId}`);
          }
          return [];
        }

        if (import.meta.env.DEV) {
          console.error(`❌ Failed to fetch messages for chat ${chatId}:`, error);
        }

        errorHandler.handle(error, {
          context: `Fetch messages for chat ${chatId}`,
          silent: isPreload
        });

        throw error;
      }
    },

    /**
     * Navigate to chat with optimized loading
     */
    async navigateToChat(chatId) {
      try {
        if (import.meta.env.DEV) {
          console.log(`🚀 Navigating to chat: ${chatId}`);
        }

        // Set current chat first
        await this.setCurrentChat(chatId);

        // Load messages using unified service
        await unifiedMessageService.loadMessagesForChat(chatId);

        if (import.meta.env.DEV) {
          console.log(`✅ Navigation to chat ${chatId} completed`);
        }

        return true;

      } catch (error) {
        if (import.meta.env.DEV) {
          console.error(`❌ Failed to navigate to chat ${chatId}:`, error);
        }

        errorHandler.handle(error, {
          context: `Navigate to chat ${chatId}`,
          silent: false
        });

        throw error;
      }
    },

    /**
     * Fetch more messages for pagination
     */
    async fetchMoreMessages(chatId, limit = 15) {
      try {
        if (import.meta.env.DEV) {
          console.log(`📥 Fetching more messages for chat ${chatId}`);
        }

        // Use unified message service for pagination
        const result = await unifiedMessageService.fetchMoreMessages(chatId, { limit });

        return result || [];

      } catch (error) {
        if (import.meta.env.DEV) {
          console.error(`❌ Failed to fetch more messages for chat ${chatId}:`, error);
        }

        errorHandler.handle(error, {
          context: `Fetch more messages for chat ${chatId}`,
          silent: false
        });

        throw error;
      }
    },

    /**
     * Fetch messages (legacy method for compatibility)
     */
    async fetchMessages(chatId, limit = 15) {
      return this.fetchMessagesWithSignal(chatId, null, limit, false);
    },

    /**
     * Update chat details
     */
    async updateChat(chatId, name, description = '') {
      this.loading = true;
      this.error = null;

      try {
        const payload = {
          name: name.trim(),
          description: description.trim()
        };

        const response = await api.put(`/chat/${chatId}`, payload);
        const updatedChat = this.normalizeChat(response.data?.data || response.data);

        // Update local chat
        const chatIndex = this.chats.findIndex(c => c.id === chatId);
        if (chatIndex !== -1) {
          this.chats[chatIndex] = updatedChat;
        }

        // Cache updated chats
        this.cacheChats();

        return updatedChat;

      } catch (error) {
        errorHandler.handle(error, {
          context: 'Update chat',
          silent: false
        });
        this.error = error.response?.data?.message || 'Failed to update chat';
        throw error;
      } finally {
        this.loading = false;
      }
    },

    /**
     * Delete chat
     */
    async deleteChat(chatId) {
      this.loading = true;
      this.error = null;

      try {
        await api.delete(`/chat/${chatId}`);

        // Remove from local chats
        this.chats = this.chats.filter(c => c.id !== chatId);

        // Clear current chat if it was deleted
        if (this.currentChatId === chatId) {
          this.currentChatId = null;
        }

        // Clear messages for this chat
        await unifiedMessageService.clearMessagesForChat(chatId);

        // Cache updated chats
        this.cacheChats();

        return true;

      } catch (error) {
        errorHandler.handle(error, {
          context: 'Delete chat',
          silent: false
        });
        this.error = error.response?.data?.message || 'Failed to delete chat';
        throw error;
      } finally {
        this.loading = false;
      }
    },

    /**
     * Leave chat
     */
    async leaveChat(chatId) {
      this.loading = true;
      this.error = null;

      try {
        await api.post(`/chat/${chatId}/leave`);

        // Remove from local chats
        this.chats = this.chats.filter(c => c.id !== chatId);

        // Clear current chat if it was left
        if (this.currentChatId === chatId) {
          this.currentChatId = null;
        }

        // Clear messages for this chat
        await unifiedMessageService.clearMessagesForChat(chatId);

        // Cache updated chats
        this.cacheChats();

        return true;

      } catch (error) {
        errorHandler.handle(error, {
          context: 'Leave chat',
          silent: false
        });
        this.error = error.response?.data?.message || 'Failed to leave chat';
        throw error;
      } finally {
        this.loading = false;
      }
    },

    /**
     * Fetch full chat details for settings
     */
    async fetchFullChatDetails(chatId) {
      try {
        const response = await api.get(`/chat/${chatId}/details`);
        return response.data?.data || response.data;

      } catch (error) {
        errorHandler.handle(error, {
          context: 'Fetch chat details',
          silent: false
        });
        throw error;
      }
    },

    /**
     * 🔧 新增：根据ID获取单个chat（Perfect Navigation兼容）
     */
    async fetchChatById(chatId) {
      try {
        const response = await api.get(`/chat/${chatId}`);
        const chatData = response.data?.data || response.data;

        if (chatData) {
          const normalizedChat = this.normalizeChat(chatData);

          // 添加到本地chats数组（如果不存在）
          const existingIndex = this.chats.findIndex(c => c.id === parseInt(chatId));
          if (existingIndex === -1) {
            this.chats.push(normalizedChat);
          } else {
            this.chats[existingIndex] = normalizedChat;
          }

          // 更新缓存
          this.cacheChats();

          return normalizedChat;
        }

        return null;
      } catch (error) {
        // 404错误表示chat不存在，不是系统错误
        if (error.response?.status === 404) {
          console.warn(`Chat ${chatId} does not exist or user has no access`);
          return null;
        }

        errorHandler.handle(error, {
          context: `Fetch chat ${chatId}`,
          silent: true
        });
        throw error;
      }
    },

    /**
     * 🔧 新增：确保chat存在（兼容方法）
     */
    async ensureChat(chatId) {
      // 先检查本地是否存在
      let chat = this.getChatById(parseInt(chatId));
      if (chat) {
        return chat;
      }

      // 不存在则尝试从API获取
      chat = await this.fetchChatById(chatId);
      return chat;
    },

    /**
     * 🔧 新增：加载chat（兼容方法）
     */
    async loadChat(chatId) {
      return this.ensureChat(chatId);
    },

    /**
     * 🔧 增强：智能chat检查
     */
    async smartChatCheck(chatId) {
      const checkResult = {
        exists: false,
        hasAccess: false,
        chat: null,
        source: null
      };

      // 1. 检查本地缓存
      let chat = this.getChatById(parseInt(chatId));
      if (chat) {
        checkResult.exists = true;
        checkResult.hasAccess = true;
        checkResult.chat = chat;
        checkResult.source = 'local_cache';
        return checkResult;
      }

      // 2. 尝试从API获取
      try {
        chat = await this.fetchChatById(chatId);
        if (chat) {
          checkResult.exists = true;
          checkResult.hasAccess = true;
          checkResult.chat = chat;
          checkResult.source = 'api_fetch';
        }
      } catch (error) {
        if (error.response?.status === 404) {
          checkResult.exists = false;
          checkResult.hasAccess = false;
          checkResult.source = 'api_not_found';
        } else if (error.response?.status === 403) {
          checkResult.exists = true;
          checkResult.hasAccess = false;
          checkResult.source = 'api_no_access';
        } else {
          // 网络错误等
          checkResult.source = 'api_error';
        }
      }

      return checkResult;
    }
  }
});
