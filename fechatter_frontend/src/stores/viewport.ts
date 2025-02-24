/**
 * Viewport Store - Scroll and Visibility State Management
 * Separating UI state from business logic
 */

import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { ScrollPosition, MessageVisibility } from '@/types/message'

export const useViewportStore = defineStore('viewport', () => {
  // State
  const scrollPositions = ref(new Map<number, ScrollPosition>())
  const visibleMessages = ref(new Map<number, Set<number>>())
  const activeViewport = ref<number | null>(null)
  const isScrolling = ref(false)

  // Actions
  function updateScrollPosition(chatId: number, position: ScrollPosition) {
    scrollPositions.value.set(chatId, position)
  }

  function saveScrollPosition(chatId: number) {
    const container = document.querySelector('.simple-message-list')
    if (!container) return null

    const position: ScrollPosition = {
      scrollTop: container.scrollTop,
      scrollHeight: container.scrollHeight,
      clientHeight: container.clientHeight
    }

    scrollPositions.value.set(chatId, position)
    return position
  }

  function restoreScrollPosition(chatId: number) {
    const position = scrollPositions.value.get(chatId)
    if (!position) return

    const container = document.querySelector('.simple-message-list')
    if (!container) return

    container.scrollTop = position.scrollTop
  }

  function addVisibleMessage(chatId: number, messageId: number) {
    if (!visibleMessages.value.has(chatId)) {
      visibleMessages.value.set(chatId, new Set())
    }
    visibleMessages.value.get(chatId)!.add(messageId)
  }

  function removeVisibleMessage(chatId: number, messageId: number) {
    visibleMessages.value.get(chatId)?.delete(messageId)
  }

  function updateVisibleMessages(chatId: number, visibilities: MessageVisibility[]) {
    const visible = new Set(
      visibilities
        .filter(v => v.isVisible)
        .map(v => v.messageId)
    )
    visibleMessages.value.set(chatId, visible)
  }

  function setActiveViewport(chatId: number | null) {
    activeViewport.value = chatId
  }

  function setScrolling(value: boolean) {
    isScrolling.value = value
  }

  function clearViewport(chatId: number) {
    scrollPositions.value.delete(chatId)
    visibleMessages.value.delete(chatId)
  }

  function clearAll() {
    scrollPositions.value.clear()
    visibleMessages.value.clear()
    activeViewport.value = null
    isScrolling.value = false
  }

  return {
    // State
    scrollPositions,
    visibleMessages,
    activeViewport,
    isScrolling,

    // Actions
    updateScrollPosition,
    saveScrollPosition,
    restoreScrollPosition,
    addVisibleMessage,
    removeVisibleMessage,
    updateVisibleMessages,
    setActiveViewport,
    setScrolling,
    clearViewport,
    clearAll
  }
}) 