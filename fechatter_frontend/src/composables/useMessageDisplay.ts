/**
 * useMessageDisplay Composable
 * Encapsulates message display logic with clean API
 */

import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import { useMessagesStore } from '@/stores/messages'
import { useViewportStore } from '@/stores/viewport'
import type { Message, MessageVisibility } from '@/types/message'

export function useMessageDisplay(chatId: number) {
  const messagesStore = useMessagesStore()
  const viewportStore = useViewportStore()

  // Local state
  const messageElements = ref(new Map<number, HTMLElement>())
  const observer = ref<IntersectionObserver | null>(null)
  const visibilityMap = ref(new Map<number, MessageVisibility>())

  // Computed
  const visibleMessageIds = computed(() => {
    return Array.from(visibilityMap.value.entries())
      .filter(([_, visibility]) => visibility.isVisible)
      .map(([id]) => id)
  })

  // Methods
  function registerMessageElement(messageId: number, element: HTMLElement | null) {
    if (!element) {
      messageElements.value.delete(messageId)
      return
    }

    messageElements.value.set(messageId, element)

    // Start observing for visibility
    if (observer.value) {
      observer.value.observe(element)
    }
  }

  function checkVisibility(entries: IntersectionObserverEntry[]) {
    const updates: MessageVisibility[] = []

    entries.forEach(entry => {
      const messageId = parseInt(entry.target.getAttribute('data-message-id') || '0')
      if (!messageId) return

      const visibility: MessageVisibility = {
        messageId,
        isVisible: entry.isIntersecting,
        visiblePercentage: entry.intersectionRatio * 100,
        element: entry.target as HTMLElement
      }

      visibilityMap.value.set(messageId, visibility)
      updates.push(visibility)

      // Update viewport store
      if (entry.isIntersecting) {
        viewportStore.addVisibleMessage(chatId, messageId)
      } else {
        viewportStore.removeVisibleMessage(chatId, messageId)
      }
    })

    return updates
  }

  function createObserver() {
    const options = {
      root: null, // viewport
      rootMargin: '0px',
      threshold: [0, 0.25, 0.5, 0.75, 1] // Multiple thresholds for accurate tracking
    }

    observer.value = new IntersectionObserver((entries) => {
      const updates = checkVisibility(entries)
      // Emit visibility changes if needed
    }, options)

    // Observe existing elements
    messageElements.value.forEach((element) => {
      observer.value?.observe(element)
    })
  }

  function destroyObserver() {
    if (observer.value) {
      observer.value.disconnect()
      observer.value = null
    }
  }

  function getVisibilityInfo(): MessageVisibility[] {
    return Array.from(visibilityMap.value.values())
  }

  // Lifecycle
  onMounted(() => {
    createObserver()
  })

  onUnmounted(() => {
    destroyObserver()
    messageElements.value.clear()
    visibilityMap.value.clear()
  })

  return {
    registerMessageElement,
    visibleMessageIds,
    getVisibilityInfo,
    messageElements: computed(() => messageElements.value)
  }
} 