/**
 * Simple Message Store
 * Following Frontend Design Principles:
 * - YAGNI: Only what we need
 * - KISS: Keep it simple
 * - Single Responsibility: Only manage messages
 * - Occam's Razor: Simplest solution that works
 */

import { defineStore } from 'pinia';
import api from '@/services/api';

export const useSimpleMessageStore = defineStore('simpleMessages', {
  state: () => ({
    // Single source of truth for messages
    messagesByChat: new Map(), // chatId -> Message[]

    // Simple loading state
    loading: false,

    // Current chat context
    currentChatId: null
  }),

  getters: {
    // Get messages for current chat
    currentMessages: (state) => {
      if (!state.currentChatId) return [];
      return state.messagesByChat.get(state.currentChatId) || [];
    },

    // Check if has messages for a chat
    hasMessages: (state) => (chatId) => {
      return state.messagesByChat.has(chatId);
    }
  },

  actions: {
    /**
     * Load messages for a chat - Simple and direct
     */
    async loadMessages(chatId, forceRefresh = false) {
      // Set current chat
      this.currentChatId = chatId;

      // Return cached if available and not forcing refresh
      if (!forceRefresh && this.messagesByChat.has(chatId)) {
        return this.messagesByChat.get(chatId);
      }

      // Load from backend
      this.loading = true;
      try {
        const response = await api.get(`/chat/${chatId}/messages`);
        const messages = response.data?.data || [];

        // Simple normalization - only what UI needs
        const normalized = messages.map(msg => ({
          id: msg.id,
          content: msg.content,
          sender_id: msg.sender_id,
          sender_name: msg.sender?.fullname || msg.sender_name || 'Unknown',
          created_at: msg.created_at,
          status: 'sent',
          files: msg.files || []
        }));

        // Store messages
        this.messagesByChat.set(chatId, normalized);

        return normalized;
      } catch (error) {
        if (import.meta.env.DEV) {
          console.error('Failed to load messages:', error);
        throw error;
      } finally {
        this.loading = false;
      }
    },

    /**
     * Send a message - Simple optimistic update
     */
    async sendMessage(chatId, content) {
      // Create temp message for immediate display
      const tempMessage = {
        id: `temp_${Date.now()}`,
        content,
        sender_id: 'current_user', // Will be replaced
        sender_name: 'You',
        created_at: new Date().toISOString(),
        status: 'sending',
        files: []
      };

      // Add to messages immediately (optimistic update)
      const messages = this.messagesByChat.get(chatId) || [];
      messages.push(tempMessage);
      this.messagesByChat.set(chatId, messages);

      try {
        // Send to backend
        const response = await api.post(`/chat/${chatId}/messages`, { content });
        const sentMessage = response.data?.data;

        // Replace temp message with real one
        const messageIndex = messages.findIndex(m => m.id === tempMessage.id);
        if (messageIndex >= 0 && sentMessage) {
          messages[messageIndex] = {
            id: sentMessage.id,
            content: sentMessage.content,
            sender_id: sentMessage.sender_id,
            sender_name: sentMessage.sender?.fullname || 'You',
            created_at: sentMessage.created_at,
            status: 'sent',
            files: sentMessage.files || []
          };
        }

        return sentMessage;
      } catch (error) {
        // Mark as failed
        tempMessage.status = 'failed';
        if (import.meta.env.DEV) {
          console.error('Failed to send message:', error);
        throw error;
      }
    },

    /**
     * Handle incoming SSE message - Simple append
     */
    handleSSEMessage(message) {
      // Only handle messages for chats we have loaded
      if (!this.messagesByChat.has(message.chat_id)) {
        return;
      }

      // Don't add our own messages (they're already optimistically added)
      const messages = this.messagesByChat.get(message.chat_id);
      const exists = messages.some(m => m.id === message.id);

      if (!exists) {
        // Simple append
        messages.push({
          id: message.id,
          content: message.content,
          sender_id: message.sender_id,
          sender_name: message.sender?.fullname || message.sender_name || 'Unknown',
          created_at: message.created_at,
          status: 'sent',
          files: message.files || []
        });
    },

    /**
     * Clear messages for a chat
     */
    clearMessages(chatId) {
      this.messagesByChat.delete(chatId);
    },

    /**
     * Clear all messages
     */
    clearAll() {
      this.messagesByChat.clear();
      this.currentChatId = null;
    },

    /**
     * Retry failed message
     */
    async retryMessage(chatId, messageId) {
      const messages = this.messagesByChat.get(chatId);
      if (!messages) return;

      const message = messages.find(m => m.id === messageId);
      if (!message || message.status !== 'failed') return;

      // Extract content and remove failed message
      const { content } = message;
      const messageIndex = messages.indexOf(message);
      messages.splice(messageIndex, 1);

      // Resend as new message
      return this.sendMessage(chatId, content);
    }
});

/**
 * Usage Example:
 * 
 * const messageStore = useSimpleMessageStore();
 * 
 * // Load messages
 * await messageStore.loadMessages(chatId);
 * 
 * // Send message
 * await messageStore.sendMessage(chatId, 'Hello!');
 * 
 * // Handle SSE
 * sseService.on('new_message', (msg) => {
 *   messageStore.handleSSEMessage(msg);
 * });
 */ 