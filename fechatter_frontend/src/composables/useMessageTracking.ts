/**
 * useMessageTracking Composable
 * Handles message tracking logic with clean separation
 */

import { ref, computed, inject } from 'vue'
import { useMessagesStore } from '@/stores/messages'
import { useViewportStore } from '@/stores/viewport'
import type { IMessageTrackingService, TrackingStatus } from '@/types/message'

export function useMessageTracking(chatId: number) {
  // Inject tracking service if available, otherwise use null object pattern
  const trackingService = inject<IMessageTrackingService | null>('messageTrackingService', null)

  const messagesStore = useMessagesStore()
  const viewportStore = useViewportStore()

  // Local state
  const activeTrackingId = ref<string | null>(null)
  const trackingStatus = ref<TrackingStatus | null>(null)

  // Computed
  const isTracking = computed(() => activeTrackingId.value !== null)

  const trackingProgress = computed(() => {
    if (!trackingStatus.value) return 0
    return (trackingStatus.value.displayedMessages / trackingStatus.value.totalMessages) * 100
  })

  // Methods
  async function startTracking(messageIds: number[]) {
    if (!trackingService) {
      console.warn('Message tracking service not available')
      return null
    }

    // Stop any existing tracking
    if (activeTrackingId.value) {
      await stopTracking()
    }

    try {
      const trackingId = trackingService.startTracking(chatId, messageIds)
      activeTrackingId.value = trackingId

      // Start polling for status updates
      pollTrackingStatus()

      return trackingId
    } catch (error) {
      console.error('Failed to start message tracking:', error)
      return null
    }
  }

  async function stopTracking() {
    if (!trackingService || !activeTrackingId.value) return

    try {
      trackingService.stopTracking(activeTrackingId.value)
      activeTrackingId.value = null
      trackingStatus.value = null
    } catch (error) {
      console.error('Failed to stop message tracking:', error)
    }
  }

  function pollTrackingStatus() {
    if (!trackingService || !activeTrackingId.value) return

    const poll = () => {
      if (!activeTrackingId.value) return

      try {
        const status = trackingService.getTrackingStatus(activeTrackingId.value)
        trackingStatus.value = status

        if (!status.isComplete) {
          // Continue polling
          setTimeout(poll, 500)
        } else {
          // Tracking complete
          activeTrackingId.value = null
        }
      } catch (error) {
        console.error('Failed to get tracking status:', error)
        activeTrackingId.value = null
      }
    }

    poll()
  }

  function markMessageDisplayed(messageId: number) {
    // This is now handled by the viewport store
    // Just emit an event for any listeners
    return viewportStore.addVisibleMessage(chatId, messageId)
  }

  function getDisplayedMessages(): number[] {
    const visible = viewportStore.visibleMessages.get(chatId)
    return visible ? Array.from(visible) : []
  }

  // Auto-track when messages change
  function autoTrack() {
    const messages = messagesStore.currentMessages
    if (messages.length > 0) {
      const messageIds = messages.map(m => m.id)
      startTracking(messageIds)
    }
  }

  return {
    isTracking,
    trackingProgress,
    trackingStatus,
    startTracking,
    stopTracking,
    markMessageDisplayed,
    getDisplayedMessages,
    autoTrack
  }
} 