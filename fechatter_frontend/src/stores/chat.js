import { defineStore } from 'pinia';
import { watch } from 'vue';
import api, { SearchService } from '@/services/api';
import { useAuthStore } from '@/stores/auth';
import { useNotifications } from '@/composables/useNotifications';
import { useUserStore } from '@/stores/user';
import { useMessageBatcher } from '@/composables/useMessageBatcher';
import { analytics } from '../lib/analytics-protobuf';
import { errorHandler } from '@/utils/errorHandler';
import MentionsService from '@/services/MentionsService';
import ReadReceiptsService from '@/services/ReadReceiptsService';
import requestIsolation from '@/utils/requestIsolation';
import strictChannelMessageValidator from '@/utils/strictChannelMessageValidator';

// Create global message batcher instance
const messageBatcher = useMessageBatcher();

export const useChatStore = defineStore('chat', {
  state: () => {
    // Production-grade cache initialization with proper fallback
    const getInitialChatsCache = () => {
      try {
        const cached = localStorage.getItem('chatsCache');
        if (cached) {
          const { timestamp, chats } = JSON.parse(cached);
          // Cache validity 10 minutes
          if (Date.now() - timestamp < 10 * 60 * 1000) {
            return Array.isArray(chats) ? chats : [];
          }
        }
      } catch (e) {
        console.error('Failed to parse chats cache from localStorage', e);
      }
      return [];
    };

    // Initialize message cache from sessionStorage with safe fallback
    const getInitialCache = () => {
      try {
        const cached = sessionStorage.getItem('messageCache');
        if (cached) {
          // Cleanup expired cache entries
          const parsedCache = JSON.parse(cached);
          const now = Date.now();
          const fiveMinutes = 5 * 60 * 1000;
          Object.keys(parsedCache).forEach(chatId => {
            if (now - parsedCache[chatId].timestamp > fiveMinutes) {
              delete parsedCache[chatId];
            }
          });
          return parsedCache;
        }
      } catch (e) {
        console.error('Failed to parse message cache from sessionStorage', e);
      }
      return {};
    };

    return {
      chats: getInitialChatsCache(),
      messages: [],
      currentChatId: null,
      loading: false,
      error: null,
      hasMoreMessages: true,
      lastMessageId: null,
      uploadProgress: 0,
      // Pagination state
      messagesPerPage: 20,
      // Member management
      chatMembers: {},
      // File management
      uploadedFiles: [],
      // 🚀 Performance optimization: add message cache
      messageCache: getInitialCache(),
      cacheTimeout: 5 * 60 * 1000, // 5 minutes cache
      // Frequency detection and optimization strategy
      messageSendingFrequency: {}, // { chatId: { lastSentTime: Date, recentCount: number } }
      highFrequencyThreshold: 3, // 3 messages in 30 seconds is considered high frequency
      highFrequencyWindow: 30 * 1000, // 30 seconds window
      pendingOptimisticMessages: new Map(), // Optimistic update message queue
      // New: auto cleanup config
      optimisticMessageTimeout: 30000, // Auto cleanup unconfirmed optimistic messages after 30s
      loadingChatId: null, // New: track loading chat
      // New: sending state management
      sendingStates: new Map(), // Track sending messages to avoid duplicates

      // Mentions and receipts state
      unreadMentions: [],
      messageReceipts: {}, // messageId -> receipts array
      mentionSuggestions: [],
      lastSendAttempts: new Map(), // Track last send attempt time to prevent frequent retries
      // New: hidden DM list
      hiddenDMs: [], // Store hidden DM chat IDs

      // New: chat switch debounce mechanism
      chatSwitchDebouncer: {
        timer: null,
        pendingChatId: null,
        isProcessing: false,
        lastSwitchTime: 0,
        minSwitchInterval: 300 // Minimum switch interval 300ms
      },
    };
  },

  getters: {
    getCurrentChat: (state) => {
      return state.chats.find(chat => chat.id === state.currentChatId);
    },

    getChatMembers: (state) => (chatId) => {
      return state.chatMembers[chatId] || [];
    },

    getChatById: (state) => (chatId) => {
      return state.chats.find(chat => chat.id === chatId);
    },

    getMessageById: (state) => (messageId) => {
      return state.messages.find(msg => msg.id === messageId);
    },

    // Get current user ID (from auth store)
    getCurrentUserId: () => {
      const authStore = useAuthStore();
      return authStore.user?.id;
    },

    // Check if in high frequency sending state
    isHighFrequencySending: (state) => (chatId) => {
      const frequency = state.messageSendingFrequency[chatId];
      if (!frequency) return false;

      const now = Date.now();
      const recentTimestamps = frequency.recentTimestamps?.filter(
        timestamp => now - timestamp <= state.highFrequencyWindow
      ) || [];

      return recentTimestamps.length >= state.highFrequencyThreshold;
    },

    // Get visible chats (exclude hidden DMs)
    visibleChats: (state) => {
      return state.chats.filter(chat => {
        // If DM and in hidden list, do not show
        if (chat.chat_type === 'Single' && state.hiddenDMs && state.hiddenDMs.includes(chat.id)) {
          return false;
        }
        return true;
      });
    },

    // Get hidden DM chats
    hiddenDMChats: (state) => {
      return state.chats.filter(chat =>
        chat.chat_type === 'Single' && state.hiddenDMs && state.hiddenDMs.includes(chat.id)
      );
    },
  },

  actions: {
    /**
     * Normalize message data - with caching
     * Following Occam's Razor: Simple normalization, no over-engineering
     */
    normalizeMessage(message) {
      if (!message) return null;

      // Check if already normalized (has _normalized flag)
      if (message._normalized) {
        return message;
      }

      // Simple normalization
      const normalized = {
        ...message,
        _normalized: true, // Mark as normalized
        id: message.id || message._id || message.temp_id,
        temp_id: message.temp_id || null,
        content: message.content || '',
        sender_id: message.sender_id || message.sender?.id,
        sender: message.sender || null,
        created_at: message.created_at || new Date().toISOString(),
        updated_at: message.updated_at || message.created_at,
        files: message.files || [],
        reply_to: message.reply_to || null,
        mentions: message.mentions || [],
        status: message.status || 'sent',
        isOptimistic: message.isOptimistic || false,
        // Add timestamp for sorting (parse once)
        _timestamp: new Date(message.created_at || Date.now()).getTime()
      };

      // Handle sender normalization only if needed
      if (normalized.sender && typeof normalized.sender === 'object') {
        normalized.sender = {
          id: normalized.sender.id,
          fullname: normalized.sender.fullname || normalized.sender.name || 'Unknown',
          email: normalized.sender.email || '',
          avatar_url: normalized.sender.avatar_url || null
        };
      }

      return normalized;
    },

    // 🔥 Get unread counts for all chats
    async getAllUnreadCounts() {
      try {
        // Call the new /unread-counts endpoint
        const response = await api.get('/unread-counts');

        // Handle ApiResponse wrapped data structure: response.data.data
        const unreadData = response.data?.data || response.data;
        if (unreadData && Array.isArray(unreadData)) {
          // Return mapping of chatId -> unreadCount
          return unreadData.reduce((acc, item) => {
            acc[item.chat_id] = item.unread_count;
            return acc;
          }, {});
        }

        return {};
      } catch (error) {
        // Gracefully handle the case when the API endpoint does not exist
        if (error.response?.status === 404) {
          console.warn('⚠️ [CHAT_STORE] All unread counts API not available (404)');
          return {}; // Return empty object, does not affect functionality
        }

        errorHandler.handle(error, {
          context: 'Get all unread counts',
          silent: true,
          log: true
        });
        return {};
      }
    },

    // ===== MENTIONS MANAGEMENT =====

    async fetchUnreadMentions() {
      try {
        const mentions = await MentionsService.getUnreadMentions();
        this.unreadMentions = mentions;
        return mentions;
      } catch (error) {
        errorHandler.handle(error, {
          context: 'Fetch unread mentions',
          silent: true
        });
        return [];
      }
    },

    async getMessageMentions(messageId) {
      try {
        const mentions = await MentionsService.getMessageMentions(messageId);
        return mentions;
      } catch (error) {
        errorHandler.handle(error, {
          context: 'Get message mentions',
          silent: true
        });
        return [];
      }
    },

    // Check if current user is mentioned in a message
    isCurrentUserMentioned(message) {
      const authStore = useAuthStore();
      const currentUser = authStore.user;

      if (!currentUser || !message.content) return false;

      return MentionsService.isMentioned(message.content, currentUser.username);
    },

    // Get mention suggestions for autocomplete
    getMentionSuggestions(query) {
      const userStore = useUserStore();
      const availableUsers = userStore.workspaceUsers;

      return MentionsService.getMentionSuggestions(query, availableUsers);
    },

    // ===== READ RECEIPTS MANAGEMENT =====

    async getMessageReceipts(messageId, detailed = false) {
      try {
        const receipts = detailed
          ? await ReadReceiptsService.getDetailedReceipts(messageId)
          : await ReadReceiptsService.getBasicReceipts(messageId);

        // Cache receipts
        this.messageReceipts[messageId] = receipts;

        return receipts;
      } catch (error) {
        errorHandler.handle(error, {
          context: 'Get message receipts',
          silent: true
        });
        return [];
      }
    },

    async markMessageReadEnhanced(chatId, messageId) {
      try {
        const success = await ReadReceiptsService.markMessageReadEnhanced(chatId, messageId);

        if (success) {
          // Update local unread count
          this.updateChatUnreadCount(chatId, Math.max(0, this.getChatById(chatId)?.unread_count - 1 || 0));

          // Update mentions if this message mentioned the user
          this.unreadMentions = this.unreadMentions.filter(
            mention => mention.message_id !== messageId
          );
        }

        return success;
      } catch (error) {
        errorHandler.handle(error, {
          context: 'Mark message read enhanced',
          silent: true
        });
        return false;
      }
    },

    // Format receipt status for display
    formatReceiptStatus(messageId, totalMembers) {
      const receipts = this.messageReceipts[messageId] || [];
      return ReadReceiptsService.formatReceiptStatus(receipts, totalMembers);
    },

    // ===== SSE INTEGRATION =====

    // Handle real-time mention updates from SSE
    handleMentionUpdate(mentionData) {
      // Add new mention to unread list
      if (mentionData.type === 'mention_created') {
        this.unreadMentions.unshift(mentionData.mention);
      }
      // Remove mention from unread list when read
      else if (mentionData.type === 'mention_read') {
        this.unreadMentions = this.unreadMentions.filter(
          mention => mention.message_id !== mentionData.message_id
        );
      }
    },

    // Handle real-time receipt updates from SSE
    handleReceiptUpdate(receiptData) {
      const messageId = receiptData.message_id;

      if (!this.messageReceipts[messageId]) {
        this.messageReceipts[messageId] = [];
      }

      // Update or add receipt
      const existingIndex = this.messageReceipts[messageId].findIndex(
        r => r.user_id === receiptData.user_id
      );

      if (existingIndex >= 0) {
        this.messageReceipts[messageId][existingIndex] = receiptData;
      } else {
        this.messageReceipts[messageId].push(receiptData);
      }
    },

    // Real-time message handling (for SSE integration) - M1: Batched
    addRealtimeMessage(message) {
      // Add to batcher instead of processing immediately
      messageBatcher.addMessage(message);

      // Watch for batch flush
      if (!this._batchWatcher) {
        this._batchWatcher = true;
        const stopWatcher = watch(
          () => messageBatcher.batchedMessages.value,
          (batchedMessages) => {
            if (batchedMessages.length > 0) {
              this.processBatchedMessages(batchedMessages);
              messageBatcher.reset(); // Clear after processing
            }
          },
          { immediate: true }
        );

        // Store watcher cleanup
        this._batchWatcherCleanup = stopWatcher;
      }
    },

    // Process batched messages efficiently
    processBatchedMessages(messages) {
      const startTime = performance.now();

      // Group messages by chat
      const messagesByChat = new Map();
      messages.forEach(msg => {
        if (!messagesByChat.has(msg.chat_id)) {
          messagesByChat.set(msg.chat_id, []);
        }
        messagesByChat.get(msg.chat_id).push(msg);
      });

      // Process each chat's messages
      messagesByChat.forEach((chatMessages, chatId) => {
        let chat = this.chats.find(c => c.id === chatId);

        if (!chat) {
          // Batch fetch missing chats
          this._fetchMissingChat(chatId).then(normalizedChat => {
            if (normalizedChat) {
              // Check if chat already exists before adding (prevent duplicates)
              const existingChat = this.chats.find(c => c.id === normalizedChat.id);
              if (!existingChat) {
                this.chats.unshift(normalizedChat);
              }
              chatMessages.forEach(msg => this.handleIncomingMessage(existingChat || normalizedChat, msg));
            }
          });
        } else {
          // Process all messages for this chat
          chatMessages.forEach(msg => this.handleIncomingMessage(chat, msg));
        }
      });

      const elapsed = performance.now() - startTime;
      if (process.env.NODE_ENV === 'development') {
        console.log(`[ChatStore] Processed ${messages.length} messages in ${elapsed.toFixed(2)}ms`);
      }
    },

    async _fetchMissingChat(chatId) {
      try {
        const response = await api.get(`/chat/${chatId}`);
        const chatData = response.data?.data || response.data;
        return chatData ? this._normalizeChat(chatData) : null;
      } catch (err) {
        console.error(`Failed to fetch chat ${chatId} for real-time message`, err);
        return null;
      }
    },

    handleIncomingMessage(chat, message) {
      // Normalize the incoming message
      const normalizedMessage = this.normalizeMessage(message);
      const authStore = useAuthStore();

      // CRITICAL: Validate message belongs to the correct chat
      try {
        strictChannelMessageValidator.validateMessage(normalizedMessage, chat.id);
      } catch (error) {
        console.error('❌ [SSE] Incoming message validation failed:', error.message);
        return; // DO NOT process messages for wrong channel
      }

      // Optimization: Check if the message is sent by the current user
      if (normalizedMessage.sender_id === authStore.user?.id) {
        // First, try to verify optimistic update for the message
        const verified = this.verifyMessageBroadcast(normalizedMessage);
        if (verified) {
          console.log('[SSE] Own message verified and updated, skipping add');
          // Always update chat's last message, even for verified messages
          this.updateChatLastMessage(chat, normalizedMessage);
          return;
        }

        // If verification fails, check if the message is a duplicate
        const isDuplicate = this.isDuplicateMessage(normalizedMessage);
        if (isDuplicate) {
          console.log('📨 [SSE] Duplicate own message detected, skipping add');
          return;
        }

        console.log('📨 [SSE] Own message from another device/session, adding');
      }

      // Check for duplicates for all messages (not just own)
      if (this.isDuplicateMessage(normalizedMessage)) {
        console.log('📨 [SSE] Duplicate message detected, skipping');
        return;
      }

      // Update chat's last message
      this.updateChatLastMessage(chat, normalizedMessage);

      // Update unread count if not current chat
      if (message.chat_id !== this.currentChatId) {
        this.incrementChatUnreadCount(chat.id);

        // Move chat to top for all chat types
        const chatIndex = this.chats.findIndex(c => c.id === chat.id);
        if (chatIndex > 0) {
          this.chats.splice(chatIndex, 1);
          this.chats.unshift(chat);
        }
      }

      // Add to messages if it's the current chat
      if (message.chat_id === this.currentChatId) {
        // CRITICAL: Final validation before adding to current messages
        try {
          strictChannelMessageValidator.validateMessage(normalizedMessage, this.currentChatId);
        } catch (error) {
          console.error('❌ [SSE] Message does not belong to current chat:', error.message);
          return;
        }

        // Add new message to the end (newest messages at bottom)
        this.messages.push(normalizedMessage);

        // Update cache
        if (this.messageCache[this.currentChatId]) {
          this.messageCache[this.currentChatId].messages.push(normalizedMessage);
          this.messageCache[this.currentChatId].timestamp = Date.now();
          sessionStorage.setItem('messageCache', JSON.stringify(this.messageCache));
        }

        // Sort messages by created_at in ascending order (oldest first, newest last)
        // If sequence_number is available, use it for sorting
        if (normalizedMessage.sequence_number) {
          this.messages.sort((a, b) => {
            // First sort by sequence_number if both have it
            if (a.sequence_number && b.sequence_number) {
              return a.sequence_number - b.sequence_number;
            }
            // Fallback to created_at
            return new Date(a.created_at) - new Date(b.created_at);
          });
        } else {
          this.messages.sort((a, b) => new Date(a.created_at) - new Date(b.created_at));
        }

        console.log('📨 [SSE] Added new message to current chat');
      }
    },

    // 🔧 新增：更新聊天最后消息的辅助方法
    updateChatLastMessage(chat, message) {
      chat.last_message = {
        id: message.id,
        content: message.content,
        created_at: message.created_at,
        sender_id: message.sender_id
      };
      chat.updated_at = message.created_at;
    },

    // 🔧 新增：检查重复消息的方法
    checkForDuplicateMessage(message) {
      const authStore = useAuthStore();

      // 检查最近的消息中是否有相同的内容和发送者
      const recentTimeThreshold = 30000; // 30秒内
      const currentTime = new Date().getTime();
      const messageTime = new Date(message.created_at).getTime();

      return this.messages.some(existingMsg => {
        // 检查是否是相同用户发送的相同内容
        if (existingMsg.sender_id === message.sender_id &&
          existingMsg.content === message.content) {

          const existingTime = new Date(existingMsg.created_at).getTime();
          const timeDiff = Math.abs(messageTime - existingTime);

          // 如果在30秒内发送的相同内容，认为是重复
          if (timeDiff < recentTimeThreshold) {
            return true;
          }
        }

        // 检查是否已经存在相同ID的消息
        if (existingMsg.id === message.id) {
          return true;
        }

        return false;
      });
    },

    // 🔧 新增：改进的重复消息检测方法
    isDuplicateMessage(message) {
      // Check by ID (most reliable)
      if (message.id && this.messages.some(m => m.id === message.id)) {
        console.log(`📨 [DUPLICATE] Message with ID ${message.id} already exists`);
        return true;
      }

      // Check by idempotency_key
      if (message.idempotency_key) {
        const duplicate = this.messages.some(m =>
          m.idempotency_key === message.idempotency_key
        );
        if (duplicate) {
          console.log(`📨 [DUPLICATE] Message with idempotency_key ${message.idempotency_key} already exists`);
          return true;
        }
      }

      // Check by sequence_number (for same chat)
      if (message.sequence_number && message.chat_id) {
        const duplicate = this.messages.some(m =>
          m.chat_id === message.chat_id &&
          m.sequence_number === message.sequence_number
        );
        if (duplicate) {
          console.log(`📨 [DUPLICATE] Message with sequence_number ${message.sequence_number} in chat ${message.chat_id} already exists`);
          return true;
        }
      }

      // Fallback: Check by content and time (less reliable)
      const recentTimeThreshold = 5000; // 5 seconds
      const messageTime = new Date(message.created_at).getTime();

      const contentDuplicate = this.messages.some(m => {
        if (m.sender_id === message.sender_id &&
          m.content === message.content &&
          m.chat_id === message.chat_id) {
          const timeDiff = Math.abs(new Date(m.created_at).getTime() - messageTime);
          return timeDiff < recentTimeThreshold;
        }
        return false;
      });

      if (contentDuplicate) {
        console.log(`📨 [DUPLICATE] Similar message found within ${recentTimeThreshold}ms`);
        return true;
      }

      return false;
    },

    updateRealtimeMessage(messageId, updates) {
      const messageIndex = this.messages.findIndex(m => m.id === messageId);
      if (messageIndex !== -1) {
        this.messages[messageIndex] = { ...this.messages[messageIndex], ...updates };
      }
    },

    removeRealtimeMessage(messageId) {
      this.messages = this.messages.filter(m => m.id !== messageId);
    },

    // File storage fix (admin operation)
    async fixFileStorage(workspaceId) {
      try {
        this.loading = true;
        this.error = null;

        const response = await api.post(`/workspaces/${workspaceId}/files/fix`);
        return response.data;
      } catch (error) {
        errorHandler.handle(error, {
          context: 'Fix file storage',
          silent: false
        });
        this.error = error.response?.data?.message || 'Failed to fix file storage';
        throw error;
      } finally {
        this.loading = false;
      }
    },

    normalizeChatType(chatType) {
      if (!chatType) return 'PublicChannel';

      // Convert to lowercase for comparison
      const lowerType = chatType.toLowerCase();

      // Map to correct format (PascalCase)
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

      return typeMap[lowerType] || chatType; // Return original if not matched
    },

    // 新增：智能合并聊天列表，保持引用以减少Vue的重新渲染
    mergeChats(oldChats, newChats) {
      const chatMap = new Map(oldChats.map(c => [c.id, c]));
      newChats.forEach(newChat => {
        const existingChat = chatMap.get(newChat.id);
        if (existingChat) {
          // 更新现有对象，而不是替换，以优化性能
          Object.assign(existingChat, newChat, {
            // 保留旧的未读计数，除非新的有明确值
            unread_count: newChat.unread_count ?? existingChat.unread_count
          });
        } else {
          chatMap.set(newChat.id, newChat);
        }
      });
      return Array.from(chatMap.values());
    },

    createTemporaryDM(user) {
      const authStore = useAuthStore();
      const tempId = `preview-dm-${user.id}`;

      // Check if a temporary chat for this user already exists
      const existingTempChat = this.chats.find(c => c.id === tempId);
      if (existingTempChat) {
        return existingTempChat;
      }

      const tempChat = {
        id: tempId,
        is_preview: true,
        chat_type: 'Single',
        name: user.fullname,
        display_name: user.fullname,
        chat_members: [
          { id: user.id, fullname: user.fullname, email: user.email },
          { id: authStore.user.id, fullname: authStore.user.fullname, email: authStore.user.email }
        ],
        messages: [],
        created_at: new Date().toISOString(),
      };

      this.chats.unshift(tempChat);
      return tempChat;
    },

    removeTemporaryDM(tempId) {
      this.chats = this.chats.filter(c => c.id !== tempId);
    },

    closeDM(chatId) {
      const chatIndex = this.chats.findIndex(c => c.id === chatId);
      if (chatIndex > -1) {
        const chat = this.chats[chatIndex];
        if (chat.chat_type === 'Single') {
          this.chats.splice(chatIndex, 1);
          if (this.currentChatId === chatId) {
            this.currentChatId = null;
            // Use reactive method instead of direct assignment
            this.messages.length = 0;
            if (typeof window !== 'undefined' && window.$router) {
              window.$router.push('/home');
            }
          }
        } else {
          console.warn('Cannot close a non-DM chat from the list.');
        }
      }
    },

    _normalizeChat(chat) {
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
        unread_count: 0,
        last_message: chat.last_message || null,
        display_name: chat.name,
        display_avatar: null
      };

      if (normalizedChat.chat_type === 'Single') {
        normalizedChat.display_name = chat.name;
        const otherMemberId = chat.chat_members?.find(id => id !== authStore.user?.id);
        if (otherMemberId) {
          const user = userStore.getUserById(otherMemberId);
          if (user) {
            normalizedChat.display_name = user.fullname;
            normalizedChat.display_avatar = user.avatar_url;
          }
        } else if (normalizedChat.display_name === 'Direct Message' || !normalizedChat.display_name) {
          normalizedChat.display_name = 'Unknown User';
        }
      }
      return normalizedChat;
    },

    // Debug helper
    debugMessageState() {
      console.log('- Current Chat ID:', this.currentChatId);
      console.log('- Messages Count:', this.messages.length);
      console.log('- Loading:', this.loading);
      console.log('- Has More Messages:', this.hasMoreMessages);

      if (this.messages.length > 0) {
        console.log('- First Message:', this.messages[0]);
        console.log('- Last Message:', this.messages[this.messages.length - 1]);
      }

      console.log('- Message Cache Keys:', Object.keys(this.messageCache));
      console.log('- Pending Optimistic Messages:', this.pendingOptimisticMessages.size);

      // Check Vue reactivity
      console.log('- Is messages array reactive?', this.$state.messages === this.messages);

      return {
        currentChatId: this.currentChatId,
        messagesCount: this.messages.length,
        messages: this.messages
      };
    },

    /**
     * 🔧 优化后的通过SSE广播验证消息 - 避免本人消息重复显示
     */
    verifyMessageBroadcast(broadcastMessage) {
      const normalizedMessage = this.normalizeMessage(broadcastMessage);
      const authStore = useAuthStore();

      // 🔧 检查是否是本人发送的消息
      if (normalizedMessage.sender_id !== authStore.user?.id) {
        return false; // 不是本人消息，需要正常添加到聊天中
      }

      // 🔧 方法1: 通过idempotency_key匹配（最可靠）
      if (normalizedMessage.idempotency_key) {
        for (const [tempId, pending] of this.pendingOptimisticMessages.entries()) {
          if (pending.idempotencyKey === normalizedMessage.idempotency_key) {
            console.log(`📨 [VERIFY] Found matching message by idempotency_key: ${tempId} -> ${normalizedMessage.id}`);

            // Clear timeout if exists
            const msgIndex = this.messages.findIndex(m => m.temp_id === tempId);
            if (msgIndex !== -1 && this.messages[msgIndex].timeout) {
              clearTimeout(this.messages[msgIndex].timeout);
            }

            // Replace temporary message with confirmed status
            const replaceWithConfirmed = (messageList) => {
              const index = messageList.findIndex(m => m.temp_id === tempId);
              if (index !== -1) {
                console.log(`📨 [VERIFY] Updating message at index ${index} from temp to confirmed`);
                messageList[index] = {
                  ...messageList[index], // Keep original message content and format
                  ...normalizedMessage,  // Update with server data
                  status: 'confirmed',   // Update status to confirmed
                  temp_id: undefined,    // Remove temporary ID
                  timeout: null,         // Clear timeout
                  isOptimistic: false,   // No longer optimistic
                  confirmed_at: new Date().toISOString()
                };
              }
            };

            replaceWithConfirmed(this.messages);
            if (this.messageCache[pending.chatId]) {
              replaceWithConfirmed(this.messageCache[pending.chatId].messages);
              sessionStorage.setItem('messageCache', JSON.stringify(this.messageCache));
            }

            // Clean up
            this.pendingOptimisticMessages.delete(tempId);

            return true; // Message verified and updated
          }
        }
      }

      // 🔧 方法2: 通过realId匹配（API响应已返回）
      for (const [tempId, pending] of this.pendingOptimisticMessages.entries()) {
        if (pending.realId === normalizedMessage.id &&
          pending.chatId === normalizedMessage.chat_id) {

          console.log(`📨 [VERIFY] Found matching pending message by realId: ${tempId} -> ${normalizedMessage.id}`);

          // Clear timeout
          const msgIndex = this.messages.findIndex(m => m.temp_id === tempId);
          if (msgIndex !== -1 && this.messages[msgIndex].timeout) {
            clearTimeout(this.messages[msgIndex].timeout);
          }

          // Replace temporary message with confirmed status
          const replaceWithConfirmed = (messageList) => {
            const index = messageList.findIndex(m => m.temp_id === tempId);
            if (index !== -1) {
              messageList[index] = {
                ...messageList[index],
                ...normalizedMessage,
                status: 'confirmed',
                temp_id: undefined,
                timeout: null,
                isOptimistic: false,
                confirmed_at: new Date().toISOString()
              };
            }
          };

          replaceWithConfirmed(this.messages);
          if (this.messageCache[pending.chatId]) {
            replaceWithConfirmed(this.messageCache[pending.chatId].messages);
            sessionStorage.setItem('messageCache', JSON.stringify(this.messageCache));
          }

          // Clean up
          this.pendingOptimisticMessages.delete(tempId);

          return true;
        }
      }

      // 🔧 方法3: 检查是否已存在相同ID的消息（防止重复）
      if (this.messages.some(m => m.id === normalizedMessage.id)) {
        console.log(`📨 [VERIFY] Message with ID ${normalizedMessage.id} already exists, skipping`);
        return true; // Message already exists, don't add duplicate
      }

      // 🔧 方法4: 通过内容和时间匹配（最后的备用方案）
      const recentTempMessages = this.messages.filter(m =>
        m.temp_id &&
        m.sender_id === authStore.user?.id &&
        m.content === normalizedMessage.content &&
        m.status === 'sent'
      );

      for (const tempMsg of recentTempMessages) {
        const timeDiff = Math.abs(
          new Date(tempMsg.created_at).getTime() - new Date(normalizedMessage.created_at).getTime()
        );

        // If time difference is within reasonable range (60 seconds)
        if (timeDiff < 60000) {
          console.log(`📨 [VERIFY] Found matching temp message by content: ${tempMsg.temp_id} -> ${normalizedMessage.id}`);

          // Update temporary message to confirmed status
          const updateToConfirmed = (messageList) => {
            const index = messageList.findIndex(m => m.temp_id === tempMsg.temp_id);
            if (index !== -1) {
              messageList[index] = {
                ...messageList[index],
                ...normalizedMessage,
                status: 'confirmed',
                temp_id: undefined,
                timeout: null,
                isOptimistic: false,
                confirmed_at: new Date().toISOString()
              };
            }
          };

          updateToConfirmed(this.messages);
          if (this.messageCache[normalizedMessage.chat_id]) {
            updateToConfirmed(this.messageCache[normalizedMessage.chat_id].messages);
            sessionStorage.setItem('messageCache', JSON.stringify(this.messageCache));
          }

          return true;
        }
      }

      console.log('📨 [VERIFY] No matching temp message found for own message - might be from another device');
      return false; // Own message but no match found, might be from another device
    },

    // 🔧 新增：防抖聊天切换方法
    async setCurrentChatDebounced(chatId) {
      const now = Date.now();

      // 如果是同一个聊天，直接返回
      if (this.currentChatId === chatId) {
        return;
      }

      // 如果正在处理切换，更新待处理的聊天ID
      if (this.chatSwitchDebouncer.isProcessing) {
        this.chatSwitchDebouncer.pendingChatId = chatId;
        return;
      }

      // 检查最小切换间隔
      const timeSinceLastSwitch = now - this.chatSwitchDebouncer.lastSwitchTime;
      if (timeSinceLastSwitch < this.chatSwitchDebouncer.minSwitchInterval) {
        const delay = this.chatSwitchDebouncer.minSwitchInterval - timeSinceLastSwitch;
        // 清除之前的定时器
        if (this.chatSwitchDebouncer.timer) {
          clearTimeout(this.chatSwitchDebouncer.timer);
        }

        // 设置新的定时器
        this.chatSwitchDebouncer.timer = setTimeout(() => {
          this.chatSwitchDebouncer.timer = null;
          this.setCurrentChatDebounced(chatId);
        }, delay);

        return;
      }

      // 开始处理切换
      this.chatSwitchDebouncer.isProcessing = true;
      this.chatSwitchDebouncer.lastSwitchTime = now;
      this.chatSwitchDebouncer.pendingChatId = null;

      try {
        await this.setCurrentChat(chatId);

        // 检查是否有待处理的切换
        if (this.chatSwitchDebouncer.pendingChatId &&
          this.chatSwitchDebouncer.pendingChatId !== chatId) {
          const nextChatId = this.chatSwitchDebouncer.pendingChatId;
          this.chatSwitchDebouncer.pendingChatId = null;

          // 延迟处理下一个切换
          setTimeout(() => {
            this.setCurrentChatDebounced(nextChatId);
          }, this.chatSwitchDebouncer.minSwitchInterval);
        }

      } catch (error) {
        console.error('🎯 [CHAT] Switch failed:', error);
      } finally {
        this.chatSwitchDebouncer.isProcessing = false;
      }
    },

    /**
     * 🔧 初始化定期清理任务
     */
    initializeCleanupTasks() {
      // 每分钟清理一次过期的乐观消息
      setInterval(() => {
        const cleaned = this.cleanupExpiredOptimisticMessages();
        if (cleaned > 0) {
          console.log(`🧹 Cleaned up ${cleaned} expired optimistic messages`);
        }
      }, 60000); // 每60秒执行一次

      // 清理过期的发送状态
      setInterval(() => {
        const now = Date.now();
        const expiredKeys = [];

        // 清理超过5分钟的发送状态记录
        for (const [key, time] of this.lastSendAttempts.entries()) {
          if (now - time > 300000) { // 5分钟
            expiredKeys.push(key);
          }
        }

        expiredKeys.forEach(key => {
          this.lastSendAttempts.delete(key);
          this.sendingStates.delete(key);
        });

        if (expiredKeys.length > 0) {
          console.log(`🧹 Cleaned up ${expiredKeys.length} expired sending states`);
        }
      }, 300000); // 每5分钟执行一次
    },

    // ===== MISSING METHODS FOR COMPATIBILITY =====

    /**
     * Fetch chats list - compatibility method
     * Uses workspace API to get chats
     */
    async fetchChats() {
      try {
        const response = await api.get('/workspace/chats');
        const chatsData = response.data?.data || response.data || [];

        // Normalize and store chats
        this.chats = chatsData.map(chat => this._normalizeChat(chat));

        // Get unread counts
        const unreadCounts = await this.getAllUnreadCounts();

        // Apply unread counts
        this.chats.forEach(chat => {
          if (unreadCounts[chat.id] !== undefined) {
            chat.unread_count = unreadCounts[chat.id];
          }
        });

        return this.chats;
      } catch (error) {
        errorHandler.handle(error, {
          context: 'Fetch chats',
          silent: false
        });
        throw error;
      }
    },

    /**
     * Fetch messages for a chat - optimized version
     */
    async fetchMessages(chatId, limit = 50, retryCount = 0) {
      try {
        // CRITICAL: Validate chatId
        const validChatId = parseInt(chatId, 10);
        if (!validChatId || isNaN(validChatId)) {
          throw new Error(`Invalid chat ID: ${chatId}`);
        }

        console.log(`📥 [FETCH_MESSAGES] Starting fetch for chat ${validChatId}`);

        // Ensure workspace users are loaded for sender info
        const { useUserStore } = await import('@/stores/user');
        const { useWorkspaceStore } = await import('@/stores/workspace');
        const userStore = useUserStore();
        const workspaceStore = useWorkspaceStore();

        if (userStore.workspaceUsers.length === 0) {
          console.log('📥 [ChatStore] Loading workspace users for message senders...');
          await workspaceStore.fetchWorkspaceUsers();
        }

        // Check cache first
        if (this.messageCache[validChatId] &&
          Date.now() - this.messageCache[validChatId].timestamp < this.cacheTimeout) {
          console.log(`📦 [FETCH_MESSAGES] Using cache for chat ${validChatId}`);

          // CRITICAL: Validate cached messages before using them
          const cachedMessages = this.messageCache[validChatId].messages;
          const validatedMessages = strictChannelMessageValidator.validateMessageArray(cachedMessages, validChatId);

          // Clear and reload with validated messages only
          this.messages.length = 0;
          validatedMessages.forEach(msg => {
            this.messages.push(msg);
          });

          console.log(`✅ [FETCH_MESSAGES] Loaded ${validatedMessages.length} validated messages from cache`);
          return this.messages;
        }

        // Use request isolation to prevent extension interference
        const response = await requestIsolation.queueRequest(
          `fetch-messages-${validChatId}`,
          async ({ signal }) => {
            return await api.get(`/chat/${validChatId}/messages`, {
              params: { limit },
              signal // Pass abort signal to axios
            });
          }
        );

        const messagesData = response.data?.data || response.data || [];
        console.log(`📨 [FETCH_MESSAGES] Received ${messagesData.length} messages from API`);

        // CRITICAL: Validate ALL messages before processing
        const validMessages = [];
        const invalidMessages = [];

        messagesData.forEach(msg => {
          try {
            // Validate BEFORE normalization
            strictChannelMessageValidator.validateMessage(msg, validChatId);
            validMessages.push(msg);
          } catch (error) {
            console.error(`❌ [FETCH_MESSAGES] Rejected invalid message:`, error.message);
            invalidMessages.push(msg);
          }
        });

        if (invalidMessages.length > 0) {
          console.error(`❌ [FETCH_MESSAGES] Found ${invalidMessages.length} messages belonging to wrong channel!`);
          // Log details for debugging
          console.table(invalidMessages.map(msg => ({
            id: msg.id,
            chat_id: msg.chat_id,
            expected: validChatId,
            content: msg.content?.substring(0, 30) + '...'
          })));
        }

        // Normalize only valid messages
        const normalizedMessages = validMessages.map(msg => this.normalizeMessage(msg));

        // Enhance messages with user data from store
        const userMap = new Map();
        userStore.workspaceUsers.forEach(user => {
          userMap.set(user.id, user);
        });

        normalizedMessages.forEach(msg => {
          if (msg.sender_id && (!msg.sender || !msg.sender.fullname)) {
            const user = userMap.get(msg.sender_id);
            if (user) {
              msg.sender = {
                id: user.id,
                fullname: user.fullname || user.name || user.email?.split('@')[0],
                email: user.email,
                avatar_url: user.avatar_url || null
              };
            }
          }
        });

        // Sort by pre-computed timestamp
        normalizedMessages.sort((a, b) => {
          return (a._timestamp || 0) - (b._timestamp || 0);
        });

        // CRITICAL: Clear ALL existing messages
        this.messages.length = 0;

        // CRITICAL: One final validation before adding to store
        normalizedMessages.forEach(msg => {
          try {
            strictChannelMessageValidator.validateMessage(msg, validChatId);
            this.messages.push(msg);
          } catch (error) {
            console.error('❌ [FETCH_MESSAGES] Final validation failed:', error.message);
          }
        });

        console.log(`✅ [FETCH_MESSAGES] Successfully loaded ${this.messages.length} messages for chat ${validChatId}`);

        // Update cache with validated messages only
        this.messageCache[validChatId] = {
          messages: [...this.messages], // Create a copy
          timestamp: Date.now(),
          sorted: true,
          validated: true // Mark as validated
        };
        sessionStorage.setItem('messageCache', JSON.stringify(this.messageCache));

        return this.messages;
      } catch (error) {
        // Enhanced error handling with fallback to cache
        const isExtensionError = requestIsolation.isExtensionInterference(error);

        if (isExtensionError) {
          console.error('🔧 Extension conflict detected while fetching messages');
          // Try to show cached messages if available
          if (this.messageCache[chatId]) {
            console.log('📦 Using cached messages due to extension conflict');

            // Validate cached messages
            const cachedMessages = this.messageCache[chatId].messages;
            const validatedMessages = strictChannelMessageValidator.validateMessageArray(cachedMessages, chatId);

            this.messages.length = 0;
            validatedMessages.forEach(msg => {
              this.messages.push(msg);
            });

            // Show notification to user
            if (window.errorHandler?.showNotification) {
              window.errorHandler.showNotification('info',
                'Using cached messages. Disable browser extensions if issues persist.');
            }

            return this.messages;
          }
        }

        errorHandler.handle(error, {
          context: `Fetch messages for chat ${chatId}`,
          silent: false
        });
        throw error;
      }
    },

    /**
     * Sync unread counts - compatibility method
     */
    async syncUnreadCounts() {
      try {
        const counts = await this.getAllUnreadCounts();

        // Apply to all chats
        this.chats.forEach(chat => {
          if (counts[chat.id] !== undefined) {
            chat.unread_count = counts[chat.id];
          }
        });

        return counts;
      } catch (error) {
        errorHandler.handle(error, {
          context: 'Sync unread counts',
          silent: true
        });
        return {};
      }
    },

    // ===== MORE MISSING METHODS FOR CHAT.VUE COMPATIBILITY =====

    /**
     * Clear messages - used when switching chats
     */
    clearMessages() {
      // Use reactive method instead of direct assignment
      this.messages.length = 0;
      this.hasMoreMessages = true;
      this.lastMessageId = null;
    },

    /**
     * Set current chat
     */
    async setCurrentChat(chatId) {
      console.log('📍 [ChatStore] setCurrentChat called with:', chatId, 'current:', this.currentChatId);

      // FIXED: Don't skip if same chat ID - we need to refresh messages
      // This was causing all channels to show the same message list
      // Previously, if currentChatId === chatId, it would return early
      // Now we always fetch fresh messages to ensure correct display

      this.currentChatId = chatId;
      this.loading = true;

      try {
        // Find the chat in our list
        const chat = this.chats.find(c => c.id === chatId);
        console.log('📍 [ChatStore] Found chat in list:', !!chat);

        if (!chat) {
          console.log('📍 [ChatStore] Chat not found, fetching from API');
          // If not found, fetch it with isolation
          const response = await requestIsolation.executeIsolatedRequest(
            async ({ signal }) => {
              return await api.get(`/chat/${chatId}`, { signal });
            },
            {
              fallbackFn: async () => {
                // Fallback: return minimal chat object
                console.warn('[ChatStore] Using fallback chat data');
                return {
                  data: {
                    id: chatId,
                    name: 'Chat',
                    unread_count: 0,
                    chat_type: 'PublicChannel'
                  }
                };
              }
            }
          );

          const chatData = response.data?.data || response.data;
          if (chatData) {
            const normalizedChat = this._normalizeChat(chatData);

            // Check again if chat exists (might have been added by another process)
            const existingChat = this.chats.find(c => c.id === normalizedChat.id);
            if (!existingChat) {
              this.chats.push(normalizedChat);
            }
          }
        }

        // ✅ CRITICAL FIX: Always fetch messages for the chat
        console.log('📍 [ChatStore] About to fetch messages for chat:', chatId);
        await this.fetchMessages(chatId);
        console.log('📍 [ChatStore] Messages fetched successfully');

        // Mark messages as read
        if (chat && chat.unread_count > 0) {
          await this.markChatAsRead(chatId);
        }
      } catch (error) {
        console.error('📍 [ChatStore] Error in setCurrentChat:', error);
        errorHandler.handle(error, {
          context: `Set current chat ${chatId}`,
          silent: true
        });
      } finally {
        this.loading = false;
      }
    },

    /**
     * Fetch chat members
     */
    async fetchChatMembers(chatId) {
      try {
        // Use request isolation to prevent extension interference
        const response = await requestIsolation.executeIsolatedRequest(
          async ({ signal }) => {
            return await api.get(`/chat/${chatId}/members`, { signal });
          },
          {
            fallbackFn: async () => {
              // Fallback: return cached members if available
              const cachedMembers = this.chatMembers[chatId];
              if (cachedMembers && cachedMembers.length > 0) {
                console.log('[ChatStore] Using cached members due to extension conflict');
                return { data: { data: cachedMembers } };
              }
              // Return empty array as last resort
              return { data: { data: [] } };
            }
          }
        );

        const members = response.data?.data || response.data || [];

        // Store in chatMembers map
        this.chatMembers[chatId] = members;

        return members;
      } catch (error) {
        errorHandler.handle(error, {
          context: `Fetch members for chat ${chatId}`,
          silent: true
        });
        return [];
      }
    },

    /**
     * Create a new chat
     */
    async createChat(name, memberIds, description = '', chatType = 'PublicChannel') {
      try {
        this.loading = true;

        const payload = {
          name,
          description,
          chat_type: chatType,
          member_ids: memberIds
        };

        const response = await api.post('/workspace/chats', payload);
        const newChat = response.data?.data || response.data;

        if (newChat) {
          const normalizedChat = this._normalizeChat(newChat);

          // Check if chat already exists before adding
          const existingChatIndex = this.chats.findIndex(c => c.id === normalizedChat.id);
          if (existingChatIndex !== -1) {
            // Update existing chat instead of adding duplicate
            this.chats[existingChatIndex] = normalizedChat;
          } else {
            // Add new chat to the beginning
            this.chats.unshift(normalizedChat);
          }

          return normalizedChat;
        }

        throw new Error('Failed to create chat');
      } catch (error) {
        errorHandler.handle(error, {
          context: 'Create chat',
          silent: false
        });
        throw error;
      } finally {
        this.loading = false;
      }
    },

    /**
     * Send a message
     */
    async sendMessage(chatId, { content, files = [] }) {
      try {
        // Generate temporary ID and idempotency key for optimistic update
        const tempId = `temp_${Date.now()}_${Math.random()}`;
        const idempotencyKey = crypto.randomUUID ? crypto.randomUUID() :
          `${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
        const authStore = useAuthStore();

        // Create optimistic message
        const optimisticMessage = {
          id: tempId,
          temp_id: tempId,
          chat_id: chatId,
          content,
          files: files || [],
          sender_id: authStore.user?.id,
          sender: authStore.user,
          created_at: new Date().toISOString(),
          status: 'sending',
          isOptimistic: true,
          idempotency_key: idempotencyKey
        };

        // Add to messages if current chat
        if (chatId === this.currentChatId) {
          this.messages.push(this.normalizeMessage(optimisticMessage));
        }

        // Track in pending messages with idempotency key
        this.pendingOptimisticMessages.set(tempId, {
          chatId,
          content,
          timestamp: Date.now(),
          idempotencyKey // Store idempotency key for verification
        });

        // TODO: Handle file uploads separately
        // For now, send message without files
        const payload = {
          content,
          files: [], // File URLs should be added here after upload
          idempotency_key: idempotencyKey, // Include idempotency key
          reply_to: null,
          mentions: null
        };

        // Send with application/json
        const response = await api.post(`/chat/${chatId}/messages`, payload);

        const sentMessage = response.data?.data || response.data;

        // Update optimistic message with real data
        if (chatId === this.currentChatId) {
          const msgIndex = this.messages.findIndex(m => m.temp_id === tempId);
          if (msgIndex !== -1) {
            this.messages[msgIndex] = this.normalizeMessage({
              ...sentMessage,
              status: 'sent'
            });
          }
        }

        // Update pending messages with real ID
        const pending = this.pendingOptimisticMessages.get(tempId);
        if (pending) {
          pending.realId = sentMessage.id;
        }

        return sentMessage;
      } catch (error) {
        // Mark message as failed
        if (chatId === this.currentChatId) {
          const msgIndex = this.messages.findIndex(m => m.temp_id === tempId);
          if (msgIndex !== -1) {
            this.messages[msgIndex].status = 'failed';
          }
        }

        errorHandler.handle(error, {
          context: 'Send message',
          silent: false
        });
        throw error;
      }
    },

    /**
     * Update chat details
     */
    async updateChat(chatId, name, description) {
      try {
        this.loading = true;

        const payload = {
          name,
          description
        };

        const response = await api.patch(`/workspace/chats/${chatId}`, payload);
        const updatedChat = response.data?.data || response.data;

        // Update local chat
        const chatIndex = this.chats.findIndex(c => c.id === chatId);
        if (chatIndex !== -1) {
          this.chats[chatIndex] = this._normalizeChat(updatedChat);
        }

        return updatedChat;
      } catch (error) {
        errorHandler.handle(error, {
          context: 'Update chat',
          silent: false
        });
        throw error;
      } finally {
        this.loading = false;
      }
    },

    /**
     * Delete a chat
     */
    async deleteChat(chatId) {
      try {
        this.loading = true;

        await api.delete(`/workspace/chats/${chatId}`);

        // Remove from local state
        this.chats = this.chats.filter(c => c.id !== chatId);

        // Clear messages if it was current chat
        if (this.currentChatId === chatId) {
          this.currentChatId = null;
          // Use reactive method instead of direct assignment
          this.messages.length = 0;
        }

        return true;
      } catch (error) {
        errorHandler.handle(error, {
          context: 'Delete chat',
          silent: false
        });
        throw error;
      } finally {
        this.loading = false;
      }
    },

    /**
     * Leave a chat
     */
    async leaveChat(chatId) {
      try {
        this.loading = true;

        await api.post(`/workspace/chats/${chatId}/leave`);

        // Remove from local state
        this.chats = this.chats.filter(c => c.id !== chatId);

        // Clear messages if it was current chat
        if (this.currentChatId === chatId) {
          this.currentChatId = null;
          // Use reactive method instead of direct assignment
          this.messages.length = 0;
        }

        return true;
      } catch (error) {
        errorHandler.handle(error, {
          context: 'Leave chat',
          silent: false
        });
        throw error;
      } finally {
        this.loading = false;
      }
    },

    /**
     * Fetch more messages (pagination)
     */
    async fetchMoreMessages(chatId) {
      if (!this.hasMoreMessages || this.loading) {
        return [];
      }

      try {
        this.loading = true;

        // Get oldest message ID for pagination
        const oldestMessage = this.messages[0];
        const beforeId = oldestMessage?.id;

        const params = {
          limit: 20,
          before_id: beforeId
        };

        const response = await api.get(`/chat/${chatId}/messages`, { params });
        const olderMessages = response.data?.data || response.data || [];

        if (olderMessages.length === 0) {
          this.hasMoreMessages = false;
          return [];
        }

        // Normalize and prepend messages
        const normalizedMessages = olderMessages.map(msg => this.normalizeMessage(msg));

        // Prepend to messages array (older messages go to the beginning)
        this.messages.unshift(...normalizedMessages);

        // Sort by created_at ascending
        this.messages.sort((a, b) => new Date(a.created_at) - new Date(b.created_at));

        // Update cache
        if (this.messageCache[chatId]) {
          this.messageCache[chatId].messages = this.messages;
          this.messageCache[chatId].timestamp = Date.now();
          sessionStorage.setItem('messageCache', JSON.stringify(this.messageCache));
        }

        // Update hasMoreMessages based on response
        this.hasMoreMessages = olderMessages.length >= 20;

        return normalizedMessages;
      } catch (error) {
        errorHandler.handle(error, {
          context: 'Fetch more messages',
          silent: true
        });
        return [];
      } finally {
        this.loading = false;
      }
    },

    /**
     * Mark chat as read
     */
    async markChatAsRead(chatId) {
      try {
        await api.post(`/chat/${chatId}/read`);

        // Update local unread count
        const chat = this.chats.find(c => c.id === chatId);
        if (chat) {
          chat.unread_count = 0;
        }
      } catch (error) {
        errorHandler.handle(error, {
          context: 'Mark chat as read',
          silent: true
        });
      }
    },

    /**
     * Update chat unread count
     */
    updateChatUnreadCount(chatId, count) {
      const chat = this.chats.find(c => c.id === chatId);
      if (chat) {
        chat.unread_count = count;
      }
    },

    /**
     * Increment chat unread count
     */
    incrementChatUnreadCount(chatId) {
      const chat = this.chats.find(c => c.id === chatId);
      if (chat) {
        chat.unread_count = (chat.unread_count || 0) + 1;
      }
    },

    /**
     * Reset chat unread count
     */
    resetChatUnreadCount(chatId) {
      this.updateChatUnreadCount(chatId, 0);
    },

    /**
     * Clean up expired optimistic messages
     */
    cleanupExpiredOptimisticMessages() {
      const now = Date.now();
      let cleaned = 0;

      // Check each pending message
      for (const [tempId, pending] of this.pendingOptimisticMessages.entries()) {
        if (now - pending.timestamp > this.optimisticMessageTimeout) {
          // Remove from messages if still there
          const msgIndex = this.messages.findIndex(m => m.temp_id === tempId);
          if (msgIndex !== -1) {
            this.messages.splice(msgIndex, 1);
            cleaned++;
          }

          // Remove from pending
          this.pendingOptimisticMessages.delete(tempId);
        }
      }

      return cleaned;
    }
  }
});
