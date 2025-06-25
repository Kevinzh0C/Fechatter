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

export const useChatStoreRefactored = defineStore('chat-refactored', {
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
    }
  },

  actions: {
    /**
     * Initialize the chat store
     */
    async initialize() {
      if (this.isInitialized) return;

      try {
        console.log('🎯 Initializing Chat Store (Refactored)...');

        // Wait for unified message service to initialize
        if (!unifiedMessageService.isInitialized.value) {
          console.log('⏳ Waiting for message service to initialize...');
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
        console.log('✅ Chat Store (Refactored) initialized');

      } catch (error) {
        console.error('Failed to initialize chat store:', error);
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
      if (this.currentChatId === chatId) return;

      console.log(`🎯 Switching to chat: ${chatId}`);

      // Set current chat
      this.currentChatId = parseInt(chatId);

      // Fetch chat members if not cached
      if (!this.chatMembers[chatId]) {
        try {
          await this.fetchChatMembers(chatId);
        } catch (error) {
          console.warn('Failed to fetch chat members:', error);
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
        console.log(`📤 Sending message to chat ${this.currentChatId}:`, content);

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
        console.error('Failed to send message:', error);
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
        console.log(`🔄 Retrying message: ${messageId}`);

        const result = await unifiedMessageService.retryMessage(messageId);

        if (result) {
          console.log(`✅ Message retry queued: ${messageId}`);
        }

        return result;

      } catch (error) {
        console.error('Failed to retry message:', error);
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
      console.log(`📨 Handling incoming message for chat ${message.chat_id}`);

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
        console.warn('Failed to cache chats:', error);
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
        console.warn('Failed to load chats from cache:', error);
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

      console.log('🧹 Chat store cleared');
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
    }
  }
});

// Export both stores for migration
export const useChatStore = useChatStoreRefactored; 