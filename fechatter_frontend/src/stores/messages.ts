/**
 * Messages Store - Normalized State Management
 * Following Pinia best practices with TypeScript
 */

import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { Message, MessageStoreState } from '@/types/message'
import api from '@/services/api'

export const useMessagesStore = defineStore('messages', () => {
  // State
  const messagesByChat = ref(new Map<number, Message[]>())
  const loadingByChat = ref(new Map<number, boolean>())
  const hasMoreByChat = ref(new Map<number, boolean>())
  const error = ref<string | null>(null)
  const currentChatId = ref<number | null>(null)

  // Getters
  const currentMessages = computed(() => {
    if (!currentChatId.value) return []
    return messagesByChat.value.get(currentChatId.value) || []
  })

  const isLoading = computed(() => {
    if (!currentChatId.value) return false
    return loadingByChat.value.get(currentChatId.value) || false
  })

  const hasMore = computed(() => {
    if (!currentChatId.value) return false
    return hasMoreByChat.value.get(currentChatId.value) ?? true
  })

  // Actions
  async function fetchMessages(chatId: number, options = { limit: 15, offset: 0 }) {
    try {
      loadingByChat.value.set(chatId, true)
      error.value = null

      const response = await api.get(`/chat/${chatId}/messages`, {
        params: options
      })

      const messages: Message[] = response.data?.data || response.data || []

      if (options.offset === 0) {
        // Initial load
        messagesByChat.value.set(chatId, messages)
      } else {
        // Load more - prepend to existing
        const existing = messagesByChat.value.get(chatId) || []
        messagesByChat.value.set(chatId, [...messages, ...existing])
      }

      // Update hasMore based on returned count
      hasMoreByChat.value.set(chatId, messages.length >= options.limit)

      return messages
    } catch (err: any) {
      error.value = err.message || 'Failed to fetch messages'
      throw err
    } finally {
      loadingByChat.value.set(chatId, false)
    }
  }

  function addMessage(message: Message) {
    const messages = messagesByChat.value.get(message.chatId) || []
    messages.push(message)
    messagesByChat.value.set(message.chatId, messages)
  }

  function updateMessage(chatId: number, messageId: number, updates: Partial<Message>) {
    const messages = messagesByChat.value.get(chatId)
    if (!messages) return

    const index = messages.findIndex(m => m.id === messageId)
    if (index !== -1) {
      messages[index] = { ...messages[index], ...updates }
      messagesByChat.value.set(chatId, [...messages])
    }
  }

  function setCurrentChat(chatId: number | null) {
    currentChatId.value = chatId
  }

  function clearChat(chatId: number) {
    messagesByChat.value.delete(chatId)
    loadingByChat.value.delete(chatId)
    hasMoreByChat.value.delete(chatId)
  }

  function clearAll() {
    messagesByChat.value.clear()
    loadingByChat.value.clear()
    hasMoreByChat.value.clear()
    error.value = null
    currentChatId.value = null
  }

  return {
    // State
    messagesByChat,
    error,
    currentChatId,

    // Getters
    currentMessages,
    isLoading,
    hasMore,

    // Actions
    fetchMessages,
    addMessage,
    updateMessage,
    setCurrentChat,
    clearChat,
    clearAll
  }
}) 