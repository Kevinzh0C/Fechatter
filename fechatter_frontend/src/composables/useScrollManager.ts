/**
 * useScrollManager Composable
 * Handles scroll behavior and position management
 */

import { ref, nextTick, onUnmounted, computed } from 'vue'
import { useViewportStore } from '@/stores/viewport'
import { debounce } from '@/utils/helpers'
import type { ScrollPosition } from '@/types/message'

export function useScrollManager(chatId: number) {
  const viewportStore = useViewportStore()

  // Local state
  const scrollContainer = ref<HTMLElement | null>(null)
  const isAutoScrolling = ref(false)
  const scrollThreshold = 100 // pixels from bottom to trigger auto-scroll
  const topThreshold = 50 // pixels from top to trigger load more

  // 🔥 NEW: Load more state management
  const isLoadingMore = ref(false)
  const autoLoadTriggerEnabled = ref(true)
  const loadMoreIndicatorVisible = ref(false)
  const lastTriggerTime = ref(0)
  const triggerCooldown = 1000 // 1 second cooldown between auto-triggers

  // Methods
  function bindScrollContainer(element: HTMLElement | null) {
    scrollContainer.value = element
    if (element) {
      element.addEventListener('scroll', handleScroll)
    }
  }

  // 🔥 NEW: Enhanced scroll handler with auto load more
  const handleScroll = debounce((_event?: Event) => {
    if (!scrollContainer.value || isAutoScrolling.value) return

    const position: ScrollPosition = {
      scrollTop: scrollContainer.value.scrollTop,
      scrollHeight: scrollContainer.value.scrollHeight,
      clientHeight: scrollContainer.value.clientHeight
    }

    viewportStore.updateScrollPosition(chatId, position)
    viewportStore.setScrolling(true)

    // 🔥 NEW: Check for auto load more trigger
    checkAutoLoadMore(position)

    setTimeout(() => {
      viewportStore.setScrolling(false)
    }, 150)
  }, 100)

  // 🔥 NEW: Auto load more trigger logic
  function checkAutoLoadMore(position: ScrollPosition) {
    if (!autoLoadTriggerEnabled.value || isLoadingMore.value) return

    const isNearTop = position.scrollTop <= topThreshold
    const hasScrollableContent = position.scrollHeight > position.clientHeight
    const cooldownPassed = Date.now() - lastTriggerTime.value > triggerCooldown

    if (isNearTop && hasScrollableContent && cooldownPassed) {
      triggerAutoLoadMore()
    }
  }

  // 🔥 NEW: Trigger auto load more with indicator
  function triggerAutoLoadMore() {
    lastTriggerTime.value = Date.now()

    // Show loading indicator
    showLoadMoreIndicator()

    // Emit load more event (will be handled by parent)
    const event = new CustomEvent('auto-load-more', {
      detail: { chatId, triggerType: 'scroll' }
    })
    document.dispatchEvent(event)
  }

  // 🔥 NEW: Show load more indicator
  function showLoadMoreIndicator() {
    loadMoreIndicatorVisible.value = true

    // Auto-hide after 2 seconds if loading completes
    setTimeout(() => {
      if (!isLoadingMore.value) {
        hideLoadMoreIndicator()
      }
    }, 2000)
  }

  // 🔥 NEW: Hide load more indicator
  function hideLoadMoreIndicator() {
    loadMoreIndicatorVisible.value = false
  }

  async function scrollToBottom(smooth = true) {
    if (!scrollContainer.value) return

    isAutoScrolling.value = true

    await nextTick()

    const targetScrollTop = scrollContainer.value.scrollHeight - scrollContainer.value.clientHeight

    if (smooth) {
      scrollContainer.value.scrollTo({
        top: targetScrollTop,
        behavior: 'smooth'
      })

      // Wait for smooth scroll to complete
      setTimeout(() => {
        isAutoScrolling.value = false
      }, 500)
    } else {
      scrollContainer.value.scrollTop = targetScrollTop
      isAutoScrolling.value = false
    }
  }

  async function scrollToMessage(messageId: number, options = { behavior: 'smooth' as ScrollBehavior, block: 'center' as ScrollLogicalPosition }) {
    await nextTick()

    const element = document.querySelector(`[data-message-id="${messageId}"]`)
    if (!element) return

    isAutoScrolling.value = true

    element.scrollIntoView(options)

    setTimeout(() => {
      isAutoScrolling.value = false
    }, 500)
  }

  // 🔥 ENHANCED: Preserve scroll with load more support
  function preserveScrollPosition(): ScrollPosition | null {
    if (!scrollContainer.value) return null

    const position = viewportStore.saveScrollPosition(chatId)

    // 🔥 NEW: Mark as loading more to prevent auto-triggers
    setLoadingMore(true)

    return position
  }

  // 🔥 ENHANCED: Restore scroll with stability guarantee
  function restoreScrollPosition(position?: ScrollPosition) {
    if (!scrollContainer.value) return

    // Disable auto-scrolling during restoration
    isAutoScrolling.value = true

    if (position) {
      // 🔥 NEW: Smart scroll restoration to maintain user reading position
      const container = scrollContainer.value
      const heightDiff = container.scrollHeight - (position.scrollHeight || 0)

      if (heightDiff > 0) {
        // New content loaded above, adjust scroll position
        const newScrollTop = position.scrollTop + heightDiff
        container.scrollTop = newScrollTop
      } else {
        container.scrollTop = position.scrollTop
      }
    } else {
      viewportStore.restoreScrollPosition(chatId)
    }

    // Re-enable auto-scrolling and auto-load triggers
    setTimeout(() => {
      isAutoScrolling.value = false
      setLoadingMore(false)
      hideLoadMoreIndicator()
    }, 100)
  }

  function isNearBottom(): boolean {
    if (!scrollContainer.value) return false

    const { scrollTop, scrollHeight, clientHeight } = scrollContainer.value
    return scrollTop + clientHeight >= scrollHeight - scrollThreshold
  }

  // 🔥 NEW: Check if near top for load more
  function isNearTop(): boolean {
    if (!scrollContainer.value) return false

    return scrollContainer.value.scrollTop <= topThreshold
  }

  function getScrollInfo(): ScrollPosition | null {
    if (!scrollContainer.value) return null

    return {
      scrollTop: scrollContainer.value.scrollTop,
      scrollHeight: scrollContainer.value.scrollHeight,
      clientHeight: scrollContainer.value.clientHeight
    }
  }

  // 🔥 NEW: Load more state management
  function setLoadingMore(value: boolean) {
    isLoadingMore.value = value

    if (value) {
      showLoadMoreIndicator()
    } else {
      // Delay hiding to provide visual feedback
      setTimeout(() => {
        hideLoadMoreIndicator()
      }, 500)
    }
  }

  function enableAutoLoadTrigger() {
    autoLoadTriggerEnabled.value = true
  }

  function disableAutoLoadTrigger() {
    autoLoadTriggerEnabled.value = false
  }

  // 🔥 NEW: Computed properties for external state tracking
  const canAutoLoadMore = computed(() =>
    autoLoadTriggerEnabled.value &&
    !isLoadingMore.value &&
    !isAutoScrolling.value
  )

  // Cleanup
  onUnmounted(() => {
    if (scrollContainer.value) {
      scrollContainer.value.removeEventListener('scroll', handleScroll)
    }
  })

  return {
    scrollContainer,
    bindScrollContainer,
    scrollToBottom,
    scrollToMessage,
    preserveScrollPosition,
    restoreScrollPosition,
    isNearBottom,
    isNearTop,
    getScrollInfo,
    isAutoScrolling,

    // 🔥 NEW: Load more functionality
    isLoadingMore,
    loadMoreIndicatorVisible,
    canAutoLoadMore,
    setLoadingMore,
    enableAutoLoadTrigger,
    disableAutoLoadTrigger,
    triggerAutoLoadMore,
    showLoadMoreIndicator,
    hideLoadMoreIndicator
  }
} 