/**
 * 🎯 Refactored Chat Store - Using Unified Message Service
 * 
 * Simplified chat store that delegates message management to UnifiedMessageService
 * Provides clean interface for chat management with complete closed-loop message logic
 */

import { defineStore } from 'pinia';
import { computed, ref, reactive, nextTick } from 'vue';
import api from '@/services/api';
import { useAuthStore } from '@/stores/auth';
import { useUserStore } from '@/stores/user';
import { errorHandler } from '@/utils/errorHandler';
import { unifiedMessageService, MessageState } from '@/services/messageSystem/UnifiedMessageService.js';
import messageConfirmationService from '@/services/messageConfirmationService';
// 🚀 CRITICAL FIX: Import SSE service for real-time message handling
import minimalSSE from '@/services/sse-minimal';

export const useChatStore = defineStore('chat', {
  state: () => ({
    // Chat management
    chats: [],
    currentChatId: null,
    loading: false,
    error: null,

    // UI state
    isInitialized: false,
    _forceUpdate: 0, // 🚀 Force Vue reactivity trigger for message updates

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

    // Get current user ID - REMOVED: Moved to actions to avoid conflict
    // getCurrentUserId moved to actions section for better error handling

    // Get messages for current chat using unified service
    messages: (state) => {
      if (!state.currentChatId) return [];
      // 🚀 Include _forceUpdate in dependency to trigger reactivity
      state._forceUpdate; // Touch the reactive property
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

        // 🚀 CRITICAL FIX: Setup SSE message listeners for real-time updates
        this.setupSSEMessageListeners();

        // Fetch initial chat list
        await this.fetchChats();

        this.isInitialized = true;
        if (import.meta.env.DEV) {
          console.log('✅ Chat Store (Refactored) initialized with SSE listeners');
        }

      } catch (error) {
        if (import.meta.env.DEV) {
          console.error('Failed to initialize chat store:', error);
        }
        this.error = error.message;
      }
    },

    /**
     * 🚀 NEW: Safe content extraction to prevent [object Object] display issues
     */
    extractSafeContent(rawContent) {
      if (!rawContent) return '';

      // If it's already a string, check for object serialization issues
      if (typeof rawContent === 'string') {
        if (rawContent.includes('[object Object]')) {
          console.warn('[ChatStore] Detected [object Object] string in SSE data');
          return 'Message content error - please refresh';
        }
        return rawContent;
      }

      // If it's an object, extract content safely
      if (typeof rawContent === 'object' && rawContent !== null) {
        if (import.meta.env.DEV) {
          console.warn('[ChatStore] SSE content is object, extracting safely:', rawContent);
        }

        // Try multiple extraction strategies
        const extracted = rawContent.text ||
          rawContent.content ||
          rawContent.message ||
          rawContent.body ||
          rawContent.data ||
          // Handle array content
          (Array.isArray(rawContent) ? rawContent.join(' ') : null);

        if (extracted && typeof extracted === 'string') {
          return extracted;
        }

        // Last resort: safe JSON stringify
        try {
          return JSON.stringify(rawContent, null, 2);
        } catch (e) {
          console.error('[ChatStore] Failed to stringify SSE content:', e);
          return 'Complex object content - display error';
        }
      }

      // Convert any other type to string
      return String(rawContent);
    },

    /**
     * 🚀 CRITICAL FIX: Enhanced SSE message listeners with auto-reregistration
     */
    setupSSEMessageListeners() {
      if (import.meta.env.DEV) {
        console.log('🔗 Setting up SSE message listeners...');
      }

      // 🚀 CRITICAL FIX: Create registrator function for auto-reregistration
      const registerListeners = () => {
        if (import.meta.env.DEV) {
          console.log('📡 [SSE] Registering message listeners...');
        }

        // Listen for new messages via SSE
        minimalSSE.on('message', (data) => {
          try {
            if (import.meta.env.DEV) {
              console.log('📨 [Real SSE] Raw event received:', data);
            }

            // Check if this is a new message event
            if (data.type === 'new_message' || data.type === 'NewMessage') {
              if (import.meta.env.DEV) {
                console.log('📨 [Real SSE] New message event:', data);
              }

              // Convert SSE message to our message format with enhanced content safety
              const formattedMessage = {
                id: parseInt(data.id),                              // ✅ 后端发送 id
                chat_id: parseInt(data.chat_id),                   // ✅ 后端发送 chat_id  
                sender_id: data.sender_id,                         // ✅ 后端发送 sender_id
                content: this.extractSafeContent(data.content || ''), // ✅ 后端发送 content
                files: data.files || [],                           // ✅ 后端发送 files (可能为空)
                created_at: data.created_at || new Date().toISOString(), // ✅ 后端发送 created_at
                sender_name: 'User',                               // 🔧 暂时使用默认值，后端Message没有sender_name
                sender: {                                          // 🔧 构造sender对象，后端Message没有sender
                  id: data.sender_id,
                  fullname: 'User',                                // 暂时使用默认值
                  email: '',                                       // 暂时为空
                  avatar_url: null                                 // 暂时为空
                },
                status: 'delivered',                               // ✅ SSE事件代表已送达
                mentions: [],                                      // 🔧 后端Message没有mentions
                reply_to: null,                                    // 🔧 后端Message没有reply_to
                isOptimistic: false,                               // ✅ 真实SSE消息
                confirmed_via_sse: true,                           // ✅ 通过SSE确认
                idempotency_key: data.idempotency_key              // ✅ 后端发送 idempotency_key
              };

              // 🚀 CRITICAL FIX: Check if this is our own message for delivery confirmation
              const authStore = useAuthStore();
              const isOwnMessage = formattedMessage.sender_id === authStore.user?.id;

              if (import.meta.env.DEV) {
                console.log(`🔍 [Real SSE] Message analysis - ID: ${formattedMessage.id}, Sender: ${formattedMessage.sender_id}, Current User: ${authStore.user?.id}, IsOwnMessage: ${isOwnMessage}`);
              }

              if (isOwnMessage) {
                // This is our own message - update existing message status to delivered
                const updated = this.updateRealtimeMessage(formattedMessage.id, {
                  status: 'delivered',
                  delivered_at: formattedMessage.created_at,
                  server_id: formattedMessage.id,
                  confirmed_via_sse: true
                });

                if (import.meta.env.DEV) {
                  console.log(`✅ [Real SSE] Own message ${formattedMessage.id} marked as delivered via REAL SSE, updateResult: ${updated}`);
                }

                // If ID matching failed, try content matching as fallback
                if (!updated) {
                  if (import.meta.env.DEV) {
                    console.log(`🔄 [Real SSE] ID matching failed, trying content matching for message: "${formattedMessage.content}"`);
                  }
                  const contentMatched = this.updateRealtimeMessageByContent(formattedMessage);
                  if (import.meta.env.DEV) {
                    console.log(`🔄 [Real SSE] Content matching result: ${contentMatched}`);
                  }
                }
              } else {
                // Someone else's message - add to real-time messages
                this.addRealtimeMessage(formattedMessage);
                if (import.meta.env.DEV) {
                  console.log(`📨 [Real SSE] Added someone else's message: ${formattedMessage.id}`);
                }
              }
            }
            // Handle typing indicators
            else if (data.type === 'typing_status' || data.type === 'TypingStatus') {
              if (import.meta.env.DEV) {
                console.log('⌨️ [Real SSE] Typing status:', data);
              }
              // Could emit this to components that need typing indicators
            }
            // Handle presence updates
            else if (data.type === 'presence' || data.type === 'Presence') {
              if (import.meta.env.DEV) {
                console.log('👋 [Real SSE] Presence update:', data);
              }
              // Could update user online status
            }
            else {
              if (import.meta.env.DEV) {
                console.log(`❓ [Real SSE] Unknown event type: ${data.type}`, data);
                console.log(`💡 [Real SSE] Available event types: NewMessage, TypingStatus, UserPresence`);
                console.log(`💡 [Real SSE] Raw event data:`, JSON.stringify(data, null, 2));
              }
            }
          } catch (error) {
            if (import.meta.env.DEV) {
              console.error('❌ [Real SSE] Error processing message:', error, data);
            }
          }
        });

        if (import.meta.env.DEV) {
          const totalListeners = Array.from(minimalSSE.listeners.values()).reduce((sum, arr) => sum + arr.length, 0);
          console.log(`✅ [Real SSE] Message listeners registered. Total SSE listeners: ${totalListeners}`);
        }
      };

      // 🚀 CRITICAL FIX: Register this setup function for auto-reregistration
      minimalSSE.addListenerRegistrator(registerListeners);

      // Initial registration
      registerListeners();

      if (import.meta.env.DEV) {
        console.log('✅ SSE message listeners configured with auto-reregistration');
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

        // 🔧 CRITICAL FIX: Generate proper UUID for idempotency
        const generateUUID = () => {
          if (typeof crypto !== 'undefined' && crypto.randomUUID) {
            return crypto.randomUUID();
          }
          // Fallback UUID v4 generation
          return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, function (c) {
            const r = Math.random() * 16 | 0;
            const v = c == 'x' ? r : (r & 0x3 | 0x8);
            return v.toString(16);
          });
        };

        const tempId = `temp-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
        const idempotencyKey = options.idempotency_key || generateUUID();

        // 🔧 CRITICAL FIX: Get real user info from authStore instead of hardcoded 'You'
        const authStore = useAuthStore();
        const currentUser = authStore.user;
        const currentUserInfo = {
          id: currentUser?.id || this.getCurrentUserId(),
          fullname: currentUser?.fullname || 'User',
          email: currentUser?.email || '',
          avatar_url: currentUser?.avatar_url || null
        };

        // Create optimistic message for immediate UI update
        const optimisticMessage = {
          id: tempId,
          temp_id: tempId,
          content: content.trim(),
          sender_id: currentUserInfo.id,
          sender_name: currentUserInfo.fullname, // ✅ Use real fullname
          sender: {
            id: currentUserInfo.id,
            fullname: currentUserInfo.fullname, // ✅ Use real fullname
            email: currentUserInfo.email, // ✅ Use real email
            avatar_url: currentUserInfo.avatar_url // ✅ Use real avatar
          },
          created_at: new Date().toISOString(),
          chat_id: this.currentChatId,
          status: 'sending',
          files: options.files || [],
          mentions: options.mentions || [],
          reply_to: options.replyTo || null,
          isOptimistic: true,
          // 🚀 NEW: Add timeout tracking for SSE confirmation
          sentAt: Date.now(),
          sseTimeout: null,
          retryAttempts: 0,
          maxRetryAttempts: 3
        };

        // 🔧 IMMEDIATE UI UPDATE: Add to UnifiedMessageService to show instantly
        let currentMessages = unifiedMessageService.getMessagesForChat(this.currentChatId) || [];
        currentMessages.push(optimisticMessage);
        unifiedMessageService.messagesByChat.set(this.currentChatId, currentMessages);

        if (import.meta.env.DEV) {
          console.log('✅ Optimistic message added to UI:', optimisticMessage);
        }

        // Send to backend with proper UUID
        const payload = {
          content: content.trim(),
          files: options.files || [],
          mentions: options.mentions || [],
          reply_to: options.replyTo || null,
          idempotency_key: idempotencyKey
        };

        const response = await api.post(`/chat/${this.currentChatId}/messages`, payload);
        const sentMessage = response.data?.data || response.data;

        if (sentMessage) {
          // 🔧 REPLACE optimistic with real message
          currentMessages = unifiedMessageService.getMessagesForChat(this.currentChatId) || [];
          const optimisticIndex = currentMessages.findIndex(m => m.temp_id === tempId);

          if (optimisticIndex !== -1) {
            // Replace optimistic message with real one
            const realMessage = {
              id: sentMessage.id,
              content: this.extractSafeContent(sentMessage.content),
              sender_id: sentMessage.sender_id,
              sender_name: sentMessage.sender?.fullname || currentUserInfo.fullname, // ✅ Use real user info as fallback
              sender: sentMessage.sender || {
                id: sentMessage.sender_id || currentUserInfo.id,
                fullname: currentUserInfo.fullname, // ✅ Use real fullname as fallback
                email: currentUserInfo.email, // ✅ Use real email as fallback
                avatar_url: currentUserInfo.avatar_url // ✅ Use real avatar as fallback
              },
              created_at: sentMessage.created_at,
              chat_id: sentMessage.chat_id,
              // 🚀 CRITICAL FIX: Set to 'delivered' immediately since notify-server doesn't send SSE to sender
              status: 'delivered',
              delivered_at: new Date().toISOString(),
              confirmed_via_api: true, // Mark as API confirmed instead of SSE confirmed
              files: sentMessage.files || [],
              mentions: sentMessage.mentions || [],
              reply_to: sentMessage.reply_to || null,
              isOptimistic: false,
              // 🚀 REMOVED: SSE timeout tracking since we don't need it anymore
              sentAt: Date.now(),
              retryAttempts: 0,
              maxRetryAttempts: 3
            };

            currentMessages[optimisticIndex] = realMessage;
            unifiedMessageService.messagesByChat.set(this.currentChatId, currentMessages);

            // 🚀 REMOVED: SSE confirmation timeout since we immediately mark as delivered
            // this.startSSEConfirmationTimeout(realMessage.id, this.currentChatId);

            // Update chat's last message
            this.updateChatLastMessage(realMessage);

            if (import.meta.env.DEV) {
              console.log('✅ Message immediately marked as delivered via API success (notify-server doesn\'t send SSE to sender):', realMessage);
              console.log('🎯 API confirmed delivery, no SSE needed for own messages');
            }

            return { message: realMessage };
          }
        }

        return { message: sentMessage };

      } catch (error) {
        if (import.meta.env.DEV) {
          console.error('Failed to send message:', error);
        }

        // 🔧 Mark optimistic message as failed
        const currentMessages = unifiedMessageService.getMessagesForChat(this.currentChatId) || [];
        const failedMessage = currentMessages.find(m => m.temp_id && m.status === 'sending');
        if (failedMessage) {
          failedMessage.status = 'failed';
          failedMessage.error = error.message;
          unifiedMessageService.messagesByChat.set(this.currentChatId, currentMessages);
        }

        errorHandler.handle(error, {
          context: 'Send message',
          silent: false
        });
        throw error;
      }
    },

    /**
     * 🚀 NEW: Start SSE confirmation timeout for message delivery
     */
    startSSEConfirmationTimeout(messageId, chatId) {
      if (!messageId || !chatId) return;

      const timeoutId = setTimeout(() => {
        if (import.meta.env.DEV) {
          console.warn(`⏰ SSE confirmation timeout for message ${messageId}, triggering retry...`);
        }

        // Find the message and mark it for retry
        const currentMessages = unifiedMessageService.getMessagesForChat(chatId) || [];
        const messageIndex = currentMessages.findIndex(m => m.id === messageId);

        if (messageIndex !== -1) {
          const message = currentMessages[messageIndex];

          // Only retry if still in 'sent' status (not already delivered)
          if (message.status === 'sent') {
            message.retryAttempts = (message.retryAttempts || 0) + 1;

            if (message.retryAttempts < message.maxRetryAttempts) {
              // 🔄 Trigger automatic retry
              this.retryMessageDelivery(message, chatId);
            } else {
              // 🚨 Mark as timeout after max retries
              message.status = 'timeout';
              message.error = 'Message delivery timeout after 3 attempts';
              unifiedMessageService.messagesByChat.set(chatId, currentMessages);

              if (import.meta.env.DEV) {
                console.error(`❌ Message ${messageId} failed after ${message.retryAttempts} retry attempts`);
              }
            }
          }
        }
      }, 15000); // 15 second timeout

      // Store timeout ID to cancel if SSE confirmation arrives
      const currentMessages = unifiedMessageService.getMessagesForChat(chatId) || [];
      const message = currentMessages.find(m => m.id === messageId);
      if (message) {
        message.sseTimeout = timeoutId;
      }
    },

    /**
     * 🚀 NEW: Retry message delivery (used by timeout mechanism)
     */
    async retryMessageDelivery(message, chatId) {
      if (!message || !chatId) return;

      try {
        if (import.meta.env.DEV) {
          console.log(`🔄 Retrying message delivery: ${message.id} (attempt ${message.retryAttempts})`);
        }

        // Update status to show retry in progress
        message.status = 'sending';
        const currentMessages = unifiedMessageService.getMessagesForChat(chatId) || [];
        const messageIndex = currentMessages.findIndex(m => m.id === message.id);
        if (messageIndex !== -1) {
          currentMessages[messageIndex] = message;
          unifiedMessageService.messagesByChat.set(chatId, currentMessages);
        }

        // 🚀 CRITICAL FIX: Generate proper UUID for idempotency_key
        const generateRetryUUID = () => {
          if (typeof crypto !== 'undefined' && crypto.randomUUID) {
            return crypto.randomUUID();
          }
          // Fallback UUID v4 generation
          return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, function (c) {
            const r = Math.random() * 16 | 0;
            const v = c == 'x' ? r : (r & 0x3 | 0x8);
            return v.toString(16);
          });
        };

        // Re-send the message
        const payload = {
          content: message.content,
          files: message.files || [],
          mentions: message.mentions || [],
          reply_to: message.reply_to || null,
          idempotency_key: generateRetryUUID() // ✅ 使用标准UUID格式
        };

        const response = await api.post(`/chat/${chatId}/messages`, payload);
        const retryResponse = response.data?.data || response.data;

        if (retryResponse) {
          // Update message with new server response
          message.status = 'sent';
          message.server_id = retryResponse.id; // Store new server ID

          // Start new SSE timeout for this retry
          this.startSSEConfirmationTimeout(message.id, chatId);

          if (import.meta.env.DEV) {
            console.log(`✅ Message retry successful: ${message.id}`);
          }

          return true;
        }

      } catch (error) {
        if (import.meta.env.DEV) {
          console.error(`❌ Message retry failed: ${message.id}`, error);
        }

        // Mark as failed if retry fails
        message.status = 'failed';
        message.error = `Retry failed: ${error.message}`;

        const currentMessages = unifiedMessageService.getMessagesForChat(chatId) || [];
        const messageIndex = currentMessages.findIndex(m => m.id === message.id);
        if (messageIndex !== -1) {
          currentMessages[messageIndex] = message;
          unifiedMessageService.messagesByChat.set(chatId, currentMessages);
        }

        return false;
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
     * 🔧 CRITICAL FIX: Add realtime message to UI (called by SSE service)
     */
    addRealtimeMessage(message) {
      if (import.meta.env.DEV) {
        console.log(`📨 Adding realtime message for chat ${message.chat_id}:`, message);
      }

      // 🔧 Add message to UnifiedMessageService for UI display
      const chatId = parseInt(message.chat_id);
      const currentMessages = unifiedMessageService.getMessagesForChat(chatId) || [];

      // Check if message already exists (prevent duplicates)
      const existingMessage = currentMessages.find(m => m.id === message.id);
      if (existingMessage) {
        if (import.meta.env.DEV) {
          console.log('📨 Message already exists, skipping duplicate:', message.id);
        }
        return;
      }

      // Add the new message
      currentMessages.push(message);
      unifiedMessageService.messagesByChat.set(chatId, currentMessages);

      if (import.meta.env.DEV) {
        console.log(`✅ Realtime message added to chat ${chatId}, total messages: ${currentMessages.length}`);
      }

      // Update chat metadata
      this.handleIncomingMessage(message);
    },

    /**
     * 🚀 ENHANCED: Update message status via SSE (for delivery confirmation)
     */
    updateRealtimeMessage(messageId, updates) {
      if (import.meta.env.DEV) {
        console.log(`📨 [Real SSE] Updating message ${messageId} via REAL SSE:`, updates);
      }

      // 🔧 Find and update message in all chats where it might exist
      let updated = false;

      for (const [chatId, messages] of unifiedMessageService.messagesByChat.entries()) {
        const messageIndex = messages.findIndex(m =>
          m.id === messageId ||
          m.temp_id === messageId ||
          m.server_id === messageId
        );

        if (messageIndex !== -1) {
          const message = messages[messageIndex];

          // 🚀 CRITICAL: Clear SSE timeout when confirmation arrives
          if (message.sseTimeout) {
            clearTimeout(message.sseTimeout);
            message.sseTimeout = null;
            if (import.meta.env.DEV) {
              console.log(`⏰ [Real SSE] SSE timeout cleared for message ${messageId} - REAL SSE confirmation received!`);
            }
          }

          // Update message properties
          Object.assign(message, updates);

          // 🚀 ENSURE PERMANENT DELIVERY STATUS: Once delivered, always delivered
          if (updates.status === 'delivered' || updates.delivered_at || updates.status === 'read') {
            message.status = 'delivered';
            message.delivered_at = updates.delivered_at || new Date().toISOString();
            message.confirmed_via_sse = true; // Mark as permanently confirmed

            if (import.meta.env.DEV) {
              console.log(`✅ [Real SSE] Message ${messageId} permanently marked as delivered via REAL SSE`);
            }
          }

          // 🚀 CRITICAL FIX: Use simplified reactive update with new architecture
          unifiedMessageService.messagesByChat.set(chatId, messages);

          updated = true;

          if (import.meta.env.DEV) {
            console.log(`✅ [Real SSE] Message ${messageId} updated in chat ${chatId} via REAL SSE:`, message);
          }
          break;
        }
      }

      // 🔧 Also try to match by content for recently sent messages (fallback)
      if (!updated) {
        const now = Date.now();

        for (const [chatId, messages] of unifiedMessageService.messagesByChat.entries()) {
          const recentMessage = messages.find(m => {
            if (m.status !== 'sent') return false;

            const messageTime = new Date(m.created_at || m.sentAt).getTime();
            if (now - messageTime > 60000) return false; // Within 60 seconds

            // Match by content and sender (for messages that haven't been assigned server IDs yet)
            return m.content === updates.content &&
              m.sender_id === updates.sender_id &&
              m.chat_id === parseInt(chatId);
          });

          if (recentMessage) {
            // 🚀 CRITICAL: Clear SSE timeout when confirmation arrives
            if (recentMessage.sseTimeout) {
              clearTimeout(recentMessage.sseTimeout);
              recentMessage.sseTimeout = null;
            }

            // Update the message
            Object.assign(recentMessage, updates);
            if (updates.status === 'delivered' || updates.delivered_at || updates.status === 'read') {
              recentMessage.status = 'delivered';
              recentMessage.delivered_at = updates.delivered_at || new Date().toISOString();
              recentMessage.confirmed_via_sse = true; // Mark as permanently confirmed
            }

            // Update server ID if provided
            if (updates.id && !recentMessage.id) {
              recentMessage.id = updates.id;
            }

            // 🚀 CRITICAL FIX: Use simplified reactive update
            unifiedMessageService.messagesByChat.set(parseInt(chatId), messages);

            updated = true;

            if (import.meta.env.DEV) {
              console.log(`✅ [Real SSE] Message matched by content and updated to delivered via REAL SSE:`, recentMessage);
            }
            break;
          }
        }
      }

      if (!updated && import.meta.env.DEV) {
        console.warn(`⚠️ Could not find message ${messageId} to update with:`, updates);
      }

      return updated;
    },

    /**
     * 🚀 NEW: Update message status by content matching (fallback for SSE delivery confirmation)
     */
    updateRealtimeMessageByContent(sseMessage) {
      if (import.meta.env.DEV) {
        console.log(`🔍 [SSE] Attempting content matching for message:`, sseMessage);
      }

      const now = Date.now();
      let matched = false;

      for (const [chatId, messages] of unifiedMessageService.messagesByChat.entries()) {
        const recentMessage = messages.find(m => {
          // Only match 'sent' status messages within last 60 seconds
          if (m.status !== 'sent') return false;

          const messageTime = new Date(m.created_at || m.sentAt).getTime();
          if (now - messageTime > 60000) return false; // Within 60 seconds

          // Match by content, sender, and chat with safe content comparison
          const safeSSEContent = this.extractSafeContent(sseMessage.content);
          return m.content === safeSSEContent &&
            m.sender_id === sseMessage.sender_id &&
            m.chat_id === sseMessage.chat_id;
        });

        if (recentMessage) {
          // Clear SSE timeout if exists
          if (recentMessage.sseTimeout) {
            clearTimeout(recentMessage.sseTimeout);
            recentMessage.sseTimeout = null;
          }

          // Update message status
          recentMessage.status = 'delivered';
          recentMessage.delivered_at = sseMessage.created_at || new Date().toISOString();
          recentMessage.confirmed_via_sse = true;
          recentMessage.id = sseMessage.id; // Update with server ID

          // Update in the messages array with simplified reactivity
          unifiedMessageService.messagesByChat.set(chatId, messages);

          matched = true;

          if (import.meta.env.DEV) {
            console.log(`✅ [Real SSE] Message matched by content and updated to delivered via REAL SSE:`, recentMessage);
          }
          break;
        }
      }

      if (!matched && import.meta.env.DEV) {
        console.warn(`⚠️ [SSE] Could not match message by content:`, sseMessage);
      }

      return matched;
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
     * Find or create DM with user
     */
    async findOrCreateDM(userId) {
      try {
        const authStore = useAuthStore();
        const userStore = useUserStore();
        const currentUserId = authStore.user?.id;

        if (!currentUserId || currentUserId === userId) {
          throw new Error('Invalid user ID for DM creation');
        }

        if (import.meta.env.DEV) {
          console.log(`[ChatStore] Finding or creating DM with user ${userId}`);
        }

        // First try to find existing DM
        let existingDM = await this.findExistingDM(userId);

        if (existingDM) {
          if (import.meta.env.DEV) {
            console.log(`[ChatStore] Found existing DM:`, existingDM);
          }
          return existingDM;
        }

        // If no existing DM found, create a new one
        const targetUser = userStore.getUserById(userId);
        const dmName = targetUser ? targetUser.fullname : `User ${userId}`;

        if (import.meta.env.DEV) {
          console.log(`[ChatStore] Creating new DM with name: ${dmName}`);
        }

        const newDM = await this.createChat(
          dmName,
          [userId], // Members array with the target user
          '', // No description for DMs
          'Single' // Chat type for direct messages
        );

        if (import.meta.env.DEV) {
          console.log(`[ChatStore] Created new DM:`, newDM);
        }

        return newDM;

      } catch (error) {
        if (import.meta.env.DEV) {
          console.error(`[ChatStore] Failed to find or create DM:`, error);
        }

        errorHandler.handle(error, {
          context: 'Find or create DM',
          silent: false
        });
        throw error;
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
    },

    /**
     * 🚀 NEW: Get current user ID for message status determination
     * Helper method used by message status logic
     */
    getCurrentUserId() {
      try {
        const authStore = useAuthStore();
        if (authStore?.user?.id) {
          return authStore.user.id;
        }

        // Fallback: try localStorage
        const authUser = localStorage.getItem('auth_user');
        if (authUser) {
          const userData = JSON.parse(authUser);
          if (userData?.id) {
            return userData.id;
          }
        }

        if (import.meta.env.DEV) {
          console.warn('⚠️ [Chat Store] Could not determine current user ID');
        }
        return null;
      } catch (error) {
        if (import.meta.env.DEV) {
          console.error('❌ [Chat Store] Error getting current user ID:', error);
        }
        return null;
      }
    },

    /**
     * 🚀 NEW: Enhanced file message with optimistic updates and progress tracking
     * Supports both raw File objects and already-uploaded file URLs/objects
     */
    async sendMessageWithFiles(content, files, options = {}) {
      if (!files || files.length === 0) {
        // Fallback to regular message
        return this.sendMessage(content, options);
      }

      const authStore = useAuthStore();
      const currentUserInfo = authStore.user || {};
      const tempId = `temp_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;

      // 🚀 CRITICAL FIX: Generate proper UUID for idempotency_key
      const generateFileMessageUUID = () => {
        if (typeof crypto !== 'undefined' && crypto.randomUUID) {
          return crypto.randomUUID();
        }
        // Fallback UUID v4 generation
        return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, function (c) {
          const r = Math.random() * 16 | 0;
          const v = c == 'x' ? r : (r & 0x3 | 0x8);
          return v.toString(16);
        });
      };

      const idempotencyKey = options.idempotencyKey || generateFileMessageUUID(); // ✅ 使用标准UUID格式

      try {
        // 🔧 CRITICAL FIX: Detect if files are already uploaded URLs or raw File objects
        const areFilesAlreadyUploaded = files.every(file =>
          typeof file === 'string' ||
          (file && typeof file === 'object' && (file.url || file.file_url) && !file.constructor.name.includes('File'))
        );

        if (areFilesAlreadyUploaded) {
          // 🚀 Fast path: Files already uploaded, send message directly
          if (import.meta.env.DEV) {
            console.log('🚀 [sendMessageWithFiles] Files already uploaded, using fast path');
          }

          const fileUrls = files.map(file => {
            if (typeof file === 'string') return file;
            return file.url || file.file_url || file;
          });

          return this.sendMessage(content, { ...options, files: fileUrls });
        }

        // 🚀 Full path: Raw File objects, need upload + progress tracking
        if (import.meta.env.DEV) {
          console.log('🚀 [sendMessageWithFiles] Raw File objects detected, using full upload path');
        }

        // Step 1: Create optimistic message with file previews
        const optimisticMessage = {
          id: tempId,
          temp_id: tempId,
          content: content.trim(),
          sender_id: currentUserInfo.id,
          sender_name: currentUserInfo.fullname || currentUserInfo.name || 'Unknown',
          sender: {
            id: currentUserInfo.id,
            fullname: currentUserInfo.fullname || currentUserInfo.name || 'Unknown',
            email: currentUserInfo.email,
            avatar_url: currentUserInfo.avatar_url
          },
          created_at: new Date().toISOString(),
          chat_id: this.currentChatId,
          status: 'file_uploading', // 🚀 NEW: File uploading status
          files: files.map(file => ({
            id: `temp_file_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
            filename: file.name,
            size: file.size,
            mime_type: file.type,
            upload_status: 'uploading', // 🚀 NEW: Individual file status
            upload_progress: 0,         // 🚀 NEW: Progress tracking
            local_url: URL.createObjectURL(file), // 🚀 Safe: Only called for File objects
            server_url: null,
            upload_error: null
          })),
          mentions: options.mentions || [],
          reply_to: options.replyTo || null,
          isOptimistic: true,
          sentAt: Date.now(),
          sseTimeout: null,
          retryAttempts: 0,
          maxRetryAttempts: 3
        };

        // Add to UI immediately
        let currentMessages = unifiedMessageService.getMessagesForChat(this.currentChatId) || [];
        currentMessages.push(optimisticMessage);
        unifiedMessageService.messagesByChat.set(this.currentChatId, currentMessages);

        if (import.meta.env.DEV) {
          console.log('✅ File message added to UI with uploading status:', optimisticMessage);
        }

        // Step 2: Upload files with progress tracking
        const uploadedFileUrls = [];
        for (let i = 0; i < files.length; i++) {
          const file = files[i];

          try {
            // Update individual file status
            this.updateFileUploadStatus(tempId, i, 'uploading', 0);

            // Upload with progress
            const uploadResult = await this.uploadFileWithProgress(
              file,
              (progress) => this.updateFileUploadProgress(tempId, i, progress)
            );

            // Mark file as uploaded
            this.updateFileUploadStatus(tempId, i, 'uploaded', 100, uploadResult.file_url || uploadResult.url);
            uploadedFileUrls.push(uploadResult.file_url || uploadResult.url);

          } catch (uploadError) {
            // Mark individual file as failed
            this.updateFileUploadStatus(tempId, i, 'failed', 0, null, uploadError.message);
            throw new Error(`File ${file.name} upload failed: ${uploadError.message}`);
          }
        }

        // Step 3: Update status to sending message
        this.updateOptimisticMessageStatus(tempId, 'sending');

        // Step 4: Send message with uploaded file URLs
        const payload = {
          content: content.trim(),
          files: uploadedFileUrls,
          mentions: options.mentions || [],
          reply_to: options.replyTo || null,
          idempotency_key: idempotencyKey
        };

        const response = await api.post(`/chat/${this.currentChatId}/messages`, payload);
        const sentMessage = response.data?.data || response.data;

        if (sentMessage) {
          // Replace optimistic with real message
          currentMessages = unifiedMessageService.getMessagesForChat(this.currentChatId) || [];
          const optimisticIndex = currentMessages.findIndex(m => m.temp_id === tempId);

          if (optimisticIndex !== -1) {
            const realMessage = {
              id: sentMessage.id,
              content: sentMessage.content,
              sender_id: sentMessage.sender_id,
              sender_name: sentMessage.sender?.fullname || currentUserInfo.fullname,
              sender: sentMessage.sender || {
                id: sentMessage.sender_id || currentUserInfo.id,
                fullname: currentUserInfo.fullname,
                email: currentUserInfo.email,
                avatar_url: currentUserInfo.avatar_url
              },
              created_at: sentMessage.created_at,
              chat_id: sentMessage.chat_id,
              status: 'delivered',
              delivered_at: new Date().toISOString(),
              confirmed_via_api: true,
              files: sentMessage.files || uploadedFileUrls.map(url => ({ url })),
              mentions: sentMessage.mentions || [],
              reply_to: sentMessage.reply_to || null,
              isOptimistic: false,
              sentAt: Date.now(),
              retryAttempts: 0,
              maxRetryAttempts: 3
            };

            currentMessages[optimisticIndex] = realMessage;
            unifiedMessageService.messagesByChat.set(this.currentChatId, currentMessages);

            this.updateChatLastMessage(realMessage);

            if (import.meta.env.DEV) {
              console.log('✅ Message immediately marked as delivered via API success (notify-server doesn\'t send SSE to sender):', realMessage);
              console.log('🎯 API confirmed delivery, no SSE needed for own messages');
            }

            return { message: realMessage };
          }
        }

        return { message: sentMessage };

      } catch (error) {
        if (import.meta.env.DEV) {
          console.error('Failed to send file message:', error);
        }

        // Mark optimistic message as failed
        const currentMessages = unifiedMessageService.getMessagesForChat(this.currentChatId) || [];
        const failedMessage = currentMessages.find(m => m.temp_id === tempId);
        if (failedMessage) {
          failedMessage.status = 'failed';
          failedMessage.error = error.message;
          unifiedMessageService.messagesByChat.set(this.currentChatId, currentMessages);
        }

        errorHandler.handle(error, {
          context: 'Send file message',
          silent: false
        });
        throw error;
      }
    },

    /**
     * 🚀 NEW: Upload file with progress tracking using XMLHttpRequest
     */
    async uploadFileWithProgress(file, onProgress) {
      const authStore = useAuthStore();

      return new Promise((resolve, reject) => {
        const formData = new FormData();
        formData.append('file', file);

        const xhr = new XMLHttpRequest();

        // Track upload progress
        xhr.upload.addEventListener('progress', (e) => {
          if (e.lengthComputable) {
            const progress = Math.round((e.loaded / e.total) * 100);
            onProgress(progress);
          }
        });

        xhr.addEventListener('load', () => {
          if (xhr.status === 200) {
            try {
              const result = JSON.parse(xhr.responseText);
              resolve(result.data || result);
            } catch (parseError) {
              reject(new Error(`Upload response parse error: ${parseError.message}`));
            }
          } else {
            reject(new Error(`Upload failed with status: ${xhr.status}`));
          }
        });

        xhr.addEventListener('error', () => {
          reject(new Error('Upload network error'));
        });

        xhr.addEventListener('timeout', () => {
          reject(new Error('Upload timeout'));
        });

        // Set timeout
        xhr.timeout = 30000; // 30 seconds

        // Set request headers
        xhr.open('POST', '/api/files/single');

        // Add authentication header if available
        const token = authStore.token;
        if (token) {
          xhr.setRequestHeader('Authorization', `Bearer ${token}`);
        }

        xhr.send(formData);
      });
    },

    /**
     * 🚀 NEW: Update file upload status for a specific file in message
     */
    updateFileUploadStatus(messageId, fileIndex, status, progress = 0, serverUrl = null, error = null) {
      const currentMessages = unifiedMessageService.getMessagesForChat(this.currentChatId) || [];
      const messageIndex = currentMessages.findIndex(m => m.temp_id === messageId || m.id === messageId);

      if (messageIndex !== -1) {
        const message = currentMessages[messageIndex];
        if (message.files && message.files[fileIndex]) {
          message.files[fileIndex].upload_status = status;
          message.files[fileIndex].upload_progress = progress;
          if (serverUrl) message.files[fileIndex].server_url = serverUrl;
          if (error) message.files[fileIndex].upload_error = error;

          unifiedMessageService.messagesByChat.set(this.currentChatId, currentMessages);

          if (import.meta.env.DEV) {
            console.log(`📤 File ${fileIndex} status updated to ${status} (${progress}%)`);
          }
        }
      }
    },

    /**
     * 🚀 NEW: Update file upload progress for a specific file
     */
    updateFileUploadProgress(messageId, fileIndex, progress) {
      this.updateFileUploadStatus(messageId, fileIndex, 'uploading', progress);
    },

    /**
     * 🚀 NEW: Update optimistic message status
     */
    updateOptimisticMessageStatus(messageId, status, error = null) {
      const currentMessages = unifiedMessageService.getMessagesForChat(this.currentChatId) || [];
      const messageIndex = currentMessages.findIndex(m => m.temp_id === messageId || m.id === messageId);

      if (messageIndex !== -1) {
        const message = currentMessages[messageIndex];
        message.status = status;
        if (error) message.error = error;

        unifiedMessageService.messagesByChat.set(this.currentChatId, currentMessages);

        if (import.meta.env.DEV) {
          console.log(`📨 Message ${messageId} status updated to ${status}`);
        }
      }
    },
  }
});
