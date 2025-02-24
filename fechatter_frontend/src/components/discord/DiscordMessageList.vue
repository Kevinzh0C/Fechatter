<template>
  <div class="discord-message-list" ref="scrollContainer" @scroll="handleScroll">
    <!-- Load More Indicator -->
    <div v-if="loadingMore" class="load-more-indicator">
      <div class="loading-spinner"></div>
      <span>Loading earlier messages...</span>
    </div>

    <!-- Messages Container -->
    <div class="messages-container" ref="messagesContainer">
      <!-- Message Items with Date Separators -->
      <template v-for="(message, index) in messages" :key="message.id || message.temp_id">
        <!-- Date Separator -->
        <div v-if="shouldShowDateSeparator(index)" class="date-separator">
          <span class="date-text">{{ formatMessageDate(message.created_at) }}</span>
        </div>

        <!-- Time Minute Separator -->
        <div v-if="shouldShowTimeSeparator(index)" class="time-separator">
          <span class="time-text">{{ formatMessageTime(message.created_at) }}</span>
        </div>

        <!-- Message Item -->
        <DiscordMessageItem :message="message" :current-user-id="currentUserId" :chat-id="chatId"
          :is-grouped="shouldGroupMessage(message, index)" @user-profile-opened="$emit('user-profile-opened', $event)"
          @dm-created="$emit('dm-created', $event)" @reply-to="handleReplyTo" @edit-message="handleEditMessage"
          @delete-message="handleDeleteMessage" @scroll-to-message="handleScrollToMessage" />
      </template>

      <!-- Typing Indicators -->
      <div v-if="typingUsers.length > 0" class="typing-indicator">
        <div class="typing-dots">
          <span></span>
          <span></span>
          <span></span>
        </div>
        <span class="typing-text">
          {{ formatTypingText(typingUsers) }}
        </span>
      </div>
    </div>

    <!-- Scroll to Bottom Button -->
    <Transition name="fade">
      <button v-if="showScrollToBottom" class="scroll-to-bottom-btn" @click="scrollToBottom(true)"
        title="Jump to latest">
        <Icon name="chevron-down" />
        <span v-if="unreadCount > 0" class="unread-badge">{{ unreadCount }}</span>
      </button>
    </Transition>

    <!-- Perfect Search Highlight -->
    <div v-if="searchHighlight" class="search-highlight-overlay"></div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted, nextTick, watch } from 'vue'
import { useAuthStore } from '@/stores/auth'
import { useChatStore } from '@/stores/chat'
import DiscordMessageItem from './DiscordMessageItem.vue'
import Icon from '@/components/ui/Icon.vue'
import { debounce, throttle } from '@/utils/performance'

// Props
const props = defineProps({
  chatId: {
    type: [Number, String],
    required: true
  },
  currentUserId: {
    type: Number,
    default: 0
  },
  messages: {
    type: Array,
    default: () => []
  },
  loading: {
    type: Boolean,
    default: false
  },
  hasMoreMessages: {
    type: Boolean,
    default: true
  },
  typingUsers: {
    type: Array,
    default: () => []
  }
})

// Emits
const emit = defineEmits([
  'load-more-messages',
  'user-profile-opened',
  'dm-created',
  'reply-to',
  'edit-message',
  'delete-message',
  'scroll-position-changed',
  'reading-position-updated'
])

// Stores
const authStore = useAuthStore()
const chatStore = useChatStore()

// Refs
const scrollContainer = ref(null)
const messagesContainer = ref(null)

// Reactive state
const loadingMore = ref(false)
const showScrollToBottom = ref(false)
const searchHighlight = ref(null)
const lastScrollTop = ref(0)
const autoScrollEnabled = ref(true)
const readingPosition = ref(null)
const lastLoadTime = ref(0)
const isLoading = ref(false)

// Computed
const unreadCount = computed(() => {
  // TODO: Calculate actual unread count
  return 0
})

const isAtBottom = computed(() => {
  if (!scrollContainer.value) return true
  const { scrollTop, scrollHeight, clientHeight } = scrollContainer.value
  return scrollTop + clientHeight >= scrollHeight - 100
})

const isNearTop = computed(() => {
  if (!scrollContainer.value) return false
  return scrollContainer.value.scrollTop < 200
})

// Methods
const formatMessageDate = (timestamp) => {
  const date = new Date(timestamp)
  const today = new Date()
  const yesterday = new Date(today)
  yesterday.setDate(yesterday.getDate() - 1)

  if (date.toDateString() === today.toDateString()) {
    return 'Today'
  } else if (date.toDateString() === yesterday.toDateString()) {
    return 'Yesterday'
  } else {
    return date.toLocaleDateString('en-US', {
      weekday: 'long',
      year: 'numeric',
      month: 'long',
      day: 'numeric'
    })
  }
}

const shouldShowDateSeparator = (index) => {
  if (index === 0) return true

  const currentMessage = props.messages[index]
  const prevMessage = props.messages[index - 1]

  if (!currentMessage || !prevMessage) return false

  const currentDate = new Date(currentMessage.created_at).toDateString()
  const prevDate = new Date(prevMessage.created_at).toDateString()

  return currentDate !== prevDate
}

const shouldShowTimeSeparator = (index) => {
  if (index === 0) return false

  const currentMessage = props.messages[index]
  const prevMessage = props.messages[index - 1]

  if (!currentMessage || !prevMessage) return false

  // Don't show time separator if date separator is already shown
  const currentDate = new Date(currentMessage.created_at).toDateString()
  const prevDate = new Date(prevMessage.created_at).toDateString()
  if (currentDate !== prevDate) return false

  const currentTime = new Date(currentMessage.created_at)
  const prevTime = new Date(prevMessage.created_at)

  const diffInMinutes = Math.floor((currentTime - prevTime) / (1000 * 60))

  // Smart separation rules for all message types including historical messages:
  // 1. Show for 15+ minutes gap
  // 2. Show for cross-hour boundary with 5+ minutes gap
  // 3. Show for 1+ hour gap (always)
  if (diffInMinutes >= 60) {
    return true // Over 1 hour gap
  }

  if (diffInMinutes >= 15) {
    return true // 15+ minutes gap
  }

  // Cross-hour boundary with 5+ minutes gap
  if (diffInMinutes >= 5 && currentTime.getHours() !== prevTime.getHours()) {
    return true
  }

  return false
}

const shouldGroupMessage = (message, index) => {
  if (index === 0) return false

  const prevMessage = props.messages[index - 1]
  if (!prevMessage) return false

  // Group if same sender and within 5 minutes
  const isSameSender = message.sender_id === prevMessage.sender_id
  const timeDiff = new Date(message.created_at) - new Date(prevMessage.created_at)
  const withinTimeLimit = timeDiff < 5 * 60 * 1000 // 5 minutes

  return isSameSender && withinTimeLimit
}

const formatTypingText = (users) => {
  if (users.length === 1) {
    return `${users[0].name} is typing...`
  } else if (users.length === 2) {
    return `${users[0].name} and ${users[1].name} are typing...`
  } else {
    return 'Several people are typing...'
  }
}

const formatMessageTime = (timestamp) => {
  if (!timestamp) return ''

  const date = new Date(timestamp)
  const now = new Date()
  const diffInMinutes = Math.floor((now - date) / (1000 * 60))

  // Format as HH:MM in 24-hour format
  const timeString = date.toLocaleTimeString('en-US', {
    hour: '2-digit',
    minute: '2-digit',
    hour12: false
  })

  // Smart relative time display in English
  let relativeInfo = ''
  if (diffInMinutes < 1) {
    relativeInfo = 'Just now'
  } else if (diffInMinutes < 60) {
    relativeInfo = `${diffInMinutes} min${diffInMinutes > 1 ? 's' : ''} ago`
  } else if (diffInMinutes < 1440) {
    const hours = Math.floor(diffInMinutes / 60)
    relativeInfo = `${hours} hour${hours > 1 ? 's' : ''} ago`
  } else if (diffInMinutes < 10080) { // Within 7 days
    const days = Math.floor(diffInMinutes / 1440)
    relativeInfo = `${days} day${days > 1 ? 's' : ''} ago`
  } else {
    // Over 7 days, show date
    relativeInfo = date.toLocaleDateString('en-US', {
      month: 'short',
      day: 'numeric'
    })
  }

  return `${timeString} • ${relativeInfo}`
}

// 🎯 Optimized historical message loading with scroll position preservation
const loadMoreMessages = async () => {
  if (
    loadingMore.value ||
    !props.hasMoreMessages ||
    isLoading.value ||
    Date.now() - lastLoadTime.value < 800
  ) {
    return
  }

  loadingMore.value = true
  isLoading.value = true
  lastLoadTime.value = Date.now()

  try {
    // 🔧 Step 1: Save current scroll state
    const container = scrollContainer.value
    if (!container) return

    const beforeScrollTop = container.scrollTop
    const beforeScrollHeight = container.scrollHeight

    console.log('📊 Load more - Before:', { scrollTop: beforeScrollTop, scrollHeight: beforeScrollHeight })

    // 🔧 Step 2: Trigger historical message loading
    await emit('load-more-messages')

    // 🔧 Step 3: Wait for DOM update completion
    await nextTick()
    // Wait one more frame to ensure rendering completion
    await new Promise(resolve => requestAnimationFrame(resolve))

    // 🔧 Step 4: Calculate height difference and adjust scroll position
    const afterScrollHeight = container.scrollHeight
    const heightDifference = afterScrollHeight - beforeScrollHeight

    if (heightDifference > 0) {
      // 🎯 Key: Keep user's viewing position unchanged
      const newScrollTop = beforeScrollTop + heightDifference
      container.scrollTop = newScrollTop

      console.log('✅ Load more - After:', {
        scrollTop: newScrollTop,
        scrollHeight: afterScrollHeight,
        heightAdded: heightDifference
      })

      console.log('🎯 Historical messages loaded, scroll position preserved with time separators applied')
    }

  } catch (error) {
    console.error('❌ Load more messages failed:', error)
  } finally {
    // Cleanup state
    setTimeout(() => {
      loadingMore.value = false
      isLoading.value = false
    }, 100)
  }
}

// 🔧 Optimized scroll handling logic
const handleScroll = throttle((event) => {
  const { scrollTop, scrollHeight, clientHeight } = event.target

  // Update scroll to bottom button visibility
  showScrollToBottom.value = !isAtBottom.value

  // 🎯 Trigger historical message loading conditions
  const shouldLoadMore = (
    scrollTop < 100 &&                // Near top trigger
    props.hasMoreMessages &&
    !loadingMore.value &&
    !isLoading.value &&
    props.messages.length > 0 &&
    Date.now() - lastLoadTime.value > 800 // Debounce
  )

  if (shouldLoadMore) {
    loadMoreMessages()
  }

  // Save reading position
  saveReadingPosition()

  // Send scroll position change event
  emit('scroll-position-changed', {
    scrollTop,
    scrollHeight,
    clientHeight,
    isAtBottom: isAtBottom.value,
    isNearTop: isNearTop.value
  })

  lastScrollTop.value = scrollTop
}, 50)

const scrollToBottom = (smooth = false) => {
  if (!scrollContainer.value) return

  const scrollOptions = {
    top: scrollContainer.value.scrollHeight,
    behavior: smooth ? 'smooth' : 'instant'
  }

  scrollContainer.value.scrollTo(scrollOptions)
  showScrollToBottom.value = false
  autoScrollEnabled.value = true
}

const scrollToMessage = async (messageId, options = {}) => {
  await nextTick()

  const messageElement = document.querySelector(`[data-message-id="${messageId}"]`)
  if (!messageElement) {
    console.warn(`Message ${messageId} not found in DOM`)
    return false
  }

  const {
    behavior = 'smooth',
    block = 'center',
    highlight = true,
    duration = 3000
  } = options

  // Scroll to message
  messageElement.scrollIntoView({ behavior, block })

  // Highlight message if requested
  if (highlight) {
    messageElement.classList.add('search-highlight')
    setTimeout(() => {
      messageElement.classList.remove('search-highlight')
    }, duration)
  }

  return true
}

const saveReadingPosition = debounce(() => {
  if (!scrollContainer.value) return

  // Find the message in the center of viewport
  const viewport = scrollContainer.value
  const viewportCenter = viewport.scrollTop + viewport.clientHeight / 2

  const messageElements = viewport.querySelectorAll('[data-message-id]')
  let centerMessage = null
  let minDistance = Infinity

  messageElements.forEach(element => {
    const rect = element.getBoundingClientRect()
    const elementCenter = rect.top + rect.height / 2
    const distance = Math.abs(elementCenter - viewportCenter)

    if (distance < minDistance) {
      minDistance = distance
      centerMessage = element
    }
  })

  if (centerMessage) {
    const messageId = centerMessage.getAttribute('data-message-id')
    readingPosition.value = {
      messageId: parseInt(messageId),
      scrollOffset: viewport.scrollTop,
      timestamp: Date.now()
    }

    emit('reading-position-updated', readingPosition.value)
  }
}, 1000)

const restoreReadingPosition = async (position) => {
  if (!position || !scrollContainer.value) return

  await nextTick()

  // Try to find the message and scroll to it
  const success = await scrollToMessage(position.messageId, {
    behavior: 'instant',
    highlight: false
  })

  if (!success && position.scrollOffset) {
    // Fallback to scroll offset
    scrollContainer.value.scrollTop = position.scrollOffset
  }
}

// Event handlers
const handleReplyTo = (message) => {
  emit('reply-to', message)
}

const handleEditMessage = (message) => {
  emit('edit-message', message)
}

const handleDeleteMessage = (message) => {
  emit('delete-message', message)
}

const handleScrollToMessage = (messageId) => {
  scrollToMessage(messageId)
}

// Perfect Search integration
const highlightSearchResult = (messageId, query, options = {}) => {
  scrollToMessage(messageId, {
    ...options,
    highlight: true,
    duration: 5000
  })

  // Additional highlighting for search terms
  if (query) {
    setTimeout(() => {
      const messageElement = document.querySelector(`[data-message-id="${messageId}"]`)
      if (messageElement) {
        highlightSearchTerms(messageElement, query)
      }
    }, 100)
  }
}

const highlightSearchTerms = (element, query) => {
  const textElements = element.querySelectorAll('.message-text')
  textElements.forEach(textEl => {
    const originalText = textEl.textContent
    const highlightedText = originalText.replace(
      new RegExp(`(${query})`, 'gi'),
      '<mark class="search-term-highlight">$1</mark>'
    )
    textEl.innerHTML = highlightedText

    // Remove highlight after 5 seconds
    setTimeout(() => {
      textEl.textContent = originalText
    }, 5000)
  })
}

// Watchers
watch(() => props.messages.length, async (newLength, oldLength) => {
  if (newLength > oldLength) {
    // New messages received
    await nextTick()

    if (autoScrollEnabled.value && isAtBottom.value) {
      scrollToBottom(true)
    }
  }
})

watch(() => props.chatId, () => {
  // Reset state when changing chats
  showScrollToBottom.value = false
  autoScrollEnabled.value = true
  readingPosition.value = null

  // Scroll to bottom after a short delay
  nextTick(() => {
    setTimeout(() => {
      scrollToBottom(false)
    }, 100)
  })
})

// Lifecycle
onMounted(() => {
  // Initial scroll to bottom
  nextTick(() => {
    scrollToBottom(false)
  })

  // Setup Perfect Search listener
  window.addEventListener('fechatter:navigate-to-message', (event) => {
    const { messageId, query, options } = event.detail
    if (parseInt(props.chatId) === parseInt(event.detail.chatId)) {
      highlightSearchResult(messageId, query, options)
    }
  })
})

onUnmounted(() => {
  window.removeEventListener('fechatter:navigate-to-message', () => { })
})

// Expose methods for parent components
defineExpose({
  scrollToBottom,
  scrollToMessage,
  highlightSearchResult,
  saveReadingPosition,
  restoreReadingPosition,
  getScrollPosition: () => ({
    scrollTop: scrollContainer.value?.scrollTop || 0,
    scrollHeight: scrollContainer.value?.scrollHeight || 0,
    clientHeight: scrollContainer.value?.clientHeight || 0
  })
})

// Console log for verification
console.log(`✅ [DiscordMessageList] Mounted for chat ${props.chatId} with ${props.messages.length} messages`)
</script>

<style scoped>
.discord-message-list {
  height: 100%;
  overflow-y: auto;
  overflow-x: hidden;
  background-color: var(--bg-primary, #36393f);
  position: relative;
  scroll-behavior: smooth;
  display: flex;
  flex-direction: column;
  align-items: center;
}

.discord-message-list::-webkit-scrollbar {
  width: 8px;
}

.discord-message-list::-webkit-scrollbar-track {
  background: var(--scrollbar-track, #2f3136);
}

.discord-message-list::-webkit-scrollbar-thumb {
  background: var(--scrollbar-thumb, #202225);
  border-radius: 4px;
}

.discord-message-list::-webkit-scrollbar-thumb:hover {
  background: var(--scrollbar-thumb-hover, #1a1d21);
}

.messages-container {
  max-width: 960px;
  width: 100%;
  padding: 0 16px;
  display: flex;
  flex-direction: column;
}

.date-separator {
  display: flex;
  align-items: center;
  justify-content: center;
  margin: 38.2px 0 23.6px 0;
  /* φ * 24px = 38.2px, φ⁻¹ * 38.2px = 23.6px (golden ratio) */
  position: relative;
  opacity: 0.8;
  transition: opacity 0.3s ease;
}

.date-separator:hover {
  opacity: 1;
}

.date-separator::before {
  content: '';
  flex: 1;
  height: 1px;
  background: linear-gradient(90deg,
      transparent 0%,
      var(--border-color, rgba(64, 68, 75, 0.2)) 38.2%,
      var(--border-color, rgba(64, 68, 75, 0.4)) 50%,
      var(--border-color, rgba(64, 68, 75, 0.2)) 61.8%,
      transparent 100%);
  margin-right: 19.4px;
  /* φ * 12px = 19.4px */
}

.date-separator::after {
  content: '';
  flex: 1;
  height: 1px;
  background: linear-gradient(90deg,
      transparent 0%,
      var(--border-color, rgba(64, 68, 75, 0.2)) 38.2%,
      var(--border-color, rgba(64, 68, 75, 0.4)) 50%,
      var(--border-color, rgba(64, 68, 75, 0.2)) 61.8%,
      transparent 100%);
  margin-left: 19.4px;
  /* φ * 12px = 19.4px */
}

.date-text {
  font-size: 12px;
  font-weight: 500;
  color: var(--text-muted, rgba(114, 118, 125, 0.9));
  background: var(--bg-primary, #36393f);
  padding: 6.18px 12.36px;
  /* φ * 4px = 6.18px, φ * 8px = 12.36px */
  border-radius: 8px;
  text-transform: uppercase;
  letter-spacing: 0.02em;
  border: 1px solid var(--border-color, rgba(64, 68, 75, 0.15));
  backdrop-filter: blur(2px);
  transition: all 0.2s ease;
}

.date-text:hover {
  transform: translateY(-0.618px);
  /* φ⁻¹ px */
  box-shadow: 0 3.09px 12.36px rgba(0, 0, 0, 0.1);
  /* φ/2 px and φ * 8px */
}

.time-separator {
  display: flex;
  align-items: center;
  justify-content: center;
  margin: 19.4px 0 9.7px 0;
  /* φ * 12px = 19.4px, φ * 6px = 9.7px */
  position: relative;
  opacity: 0.75;
  animation: timeSlideIn 0.3s ease-out;
  transition: opacity 0.3s ease;
}

.time-separator:hover {
  opacity: 1;
}

.time-separator::before {
  content: '';
  flex: 1;
  height: 0.618px;
  /* φ⁻¹ px */
  background: linear-gradient(90deg,
      transparent 0%,
      var(--border-color, rgba(64, 68, 75, 0.15)) 38.2%,
      var(--border-color, rgba(64, 68, 75, 0.25)) 50%,
      var(--border-color, rgba(64, 68, 75, 0.15)) 61.8%,
      transparent 100%);
  margin-right: 19.4px;
  /* φ * 12px = 19.4px */
}

.time-separator::after {
  content: '';
  flex: 1;
  height: 0.618px;
  /* φ⁻¹ px */
  background: linear-gradient(90deg,
      transparent 0%,
      var(--border-color, rgba(64, 68, 75, 0.15)) 38.2%,
      var(--border-color, rgba(64, 68, 75, 0.25)) 50%,
      var(--border-color, rgba(64, 68, 75, 0.15)) 61.8%,
      transparent 100%);
  margin-left: 19.4px;
  /* φ * 12px = 19.4px */
}

.time-text {
  font-size: 10px;
  font-weight: 400;
  color: var(--text-muted, rgba(114, 118, 125, 0.7));
  background: var(--bg-primary, rgba(54, 57, 63, 0.9));
  padding: 4.85px 11.8px;
  /* φ * 3px = 4.85px, φ * 7.3px = 11.8px */
  border-radius: 16.18px;
  /* φ * 10px = 16.18px */
  border: 1px solid var(--border-color, rgba(64, 68, 75, 0.1));
  letter-spacing: 0.3px;
  backdrop-filter: blur(6px);
  position: relative;
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.05);
}

.time-text::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: linear-gradient(135deg,
      rgba(255, 255, 255, 0.05) 0%,
      rgba(255, 255, 255, 0.02) 61.8%,
      rgba(255, 255, 255, 0.01) 100%);
  border-radius: 16.18px;
  /* φ * 10px = 16.18px */
  pointer-events: none;
}

.time-text:hover {
  transform: translateY(-1.618px) scale(1.02);
  /* φ px */
  box-shadow: 0 4.85px 19.4px rgba(0, 0, 0, 0.08);
  /* φ * 3px and φ * 12px */
  color: var(--text-muted, rgba(114, 118, 125, 0.9));
  background: var(--bg-primary, rgba(54, 57, 63, 0.95));
}

.load-more-indicator {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 16px;
  gap: 8px;
  color: var(--text-muted, #72767d);
  font-size: 14px;
}

.loading-spinner {
  width: 20px;
  height: 20px;
  border: 2px solid var(--border-primary, #40444b);
  border-top: 2px solid var(--accent-primary, #5865f2);
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

.typing-indicator {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 16px;
  color: var(--text-muted, #72767d);
  font-size: 14px;
}

.typing-dots {
  display: flex;
  gap: 2px;
}

.typing-dots span {
  width: 4px;
  height: 4px;
  background: var(--text-muted, #72767d);
  border-radius: 50%;
  animation: typing 1.4s ease-in-out infinite;
}

.typing-dots span:nth-child(2) {
  animation-delay: 0.2s;
}

.typing-dots span:nth-child(3) {
  animation-delay: 0.4s;
}

.scroll-to-bottom-btn {
  position: absolute;
  bottom: 20px;
  right: 20px;
  width: 40px;
  height: 40px;
  background: var(--bg-secondary, #4f545c);
  border: none;
  border-radius: 50%;
  color: var(--text-primary, #dcddde);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.2);
  transition: all 0.2s ease;
  z-index: 10;
}

.scroll-to-bottom-btn:hover {
  background: var(--bg-secondary-hover, #5d6269);
}

.unread-badge {
  position: absolute;
  top: -2px;
  right: -2px;
  background: var(--danger-color, #ed4245);
  color: white;
  border-radius: 10px;
  padding: 2px 6px;
  font-size: 10px;
  font-weight: 600;
  min-width: 16px;
  text-align: center;
}

.search-highlight-overlay {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  pointer-events: none;
  z-index: 5;
}

/* Message highlighting */
:deep(.search-highlight) {
  background-color: var(--warning-color, rgba(255, 212, 59, 0.3));
  border-left: 3px solid var(--warning-color, #ffd43b);
  animation: highlight-pulse 0.6s ease-in-out;
}

:deep(.search-term-highlight) {
  background-color: var(--warning-color, #ffd43b);
  color: var(--text-dark, #000);
  padding: 1px 2px;
  border-radius: 2px;
  font-weight: 600;
}

/* Transitions */
.fade-enter-active,
.fade-leave-active {
  transition: all 0.3s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
  transform: translateY(10px) scale(0.9);
}

/* Animations */
@keyframes spin {
  0% {
    transform: rotate(0deg);
  }

  100% {
    transform: rotate(360deg);
  }
}

@keyframes typing {

  0%,
  60%,
  100% {
    transform: translateY(0);
    opacity: 0.4;
  }

  30% {
    transform: translateY(-6px);
    opacity: 1;
  }
}

@keyframes highlight-pulse {
  0% {
    background-color: var(--warning-color, rgba(255, 212, 59, 0.6));
  }

  100% {
    background-color: var(--warning-color, rgba(255, 212, 59, 0.3));
  }
}

@keyframes timeSlideIn {
  from {
    opacity: 0;
    transform: translateY(-10px) scale(0.95);
  }

  to {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
}

/* Mobile responsive */
@media (max-width: 768px) {
  .discord-message-list {
    padding-bottom: 60px;
    /* Space for mobile input */
  }

  .scroll-to-bottom-btn {
    bottom: 80px;
    /* Above mobile input */
    right: 16px;
  }

  .date-text {
    font-size: 11px;
    padding: 0 12px;
  }
}

/* High contrast mode */
@media (prefers-contrast: high) {

  .date-separator::before,
  .date-separator::after {
    background: var(--text-primary, #dcddde);
  }

  .search-highlight {
    background-color: yellow !important;
    color: black !important;
  }
}

/* Reduced motion */
@media (prefers-reduced-motion: reduce) {
  .discord-message-list {
    scroll-behavior: auto;
  }

  .typing-dots span {
    animation: none;
  }

  .scroll-to-bottom-btn {
    transition: none;
  }

  .fade-enter-active,
  .fade-leave-active {
    transition: none;
  }
}

/* Dark mode time separators with golden ratio optimization */
@media (prefers-color-scheme: dark) {
  .date-separator::before {
    background: linear-gradient(90deg,
        transparent 0%,
        rgba(255, 255, 255, 0.08) 38.2%,
        rgba(255, 255, 255, 0.12) 50%,
        rgba(255, 255, 255, 0.08) 61.8%,
        transparent 100%);
  }

  .date-separator::after {
    background: linear-gradient(90deg,
        transparent 0%,
        rgba(255, 255, 255, 0.08) 38.2%,
        rgba(255, 255, 255, 0.12) 50%,
        rgba(255, 255, 255, 0.08) 61.8%,
        transparent 100%);
  }

  .date-text {
    color: rgba(220, 221, 222, 0.85);
    background: var(--bg-primary, #2f3136);
    border: 1px solid rgba(255, 255, 255, 0.08);
    box-shadow: 0 1.618px 6.18px rgba(0, 0, 0, 0.15);
    /* Golden ratio shadows */
  }

  .date-text:hover {
    color: rgba(220, 221, 222, 0.95);
    background: var(--bg-primary, rgba(47, 49, 54, 0.95));
    box-shadow: 0 4.85px 19.4px rgba(0, 0, 0, 0.2);
    /* φ * 3px and φ * 12px */
  }

  .time-separator::before {
    background: linear-gradient(90deg,
        transparent 0%,
        rgba(255, 255, 255, 0.06) 38.2%,
        rgba(255, 255, 255, 0.1) 50%,
        rgba(255, 255, 255, 0.06) 61.8%,
        transparent 100%);
  }

  .time-separator::after {
    background: linear-gradient(90deg,
        transparent 0%,
        rgba(255, 255, 255, 0.06) 38.2%,
        rgba(255, 255, 255, 0.1) 50%,
        rgba(255, 255, 255, 0.06) 61.8%,
        transparent 100%);
  }

  .time-text {
    color: rgba(220, 221, 222, 0.6);
    background: var(--bg-primary, rgba(47, 49, 54, 0.85));
    border: 1px solid rgba(255, 255, 255, 0.06);
    box-shadow: 0 0.618px 2.36px rgba(0, 0, 0, 0.1);
    /* φ⁻¹ px and φ * 1.45px */
  }

  .time-text::before {
    background: linear-gradient(135deg,
        rgba(255, 255, 255, 0.03) 0%,
        rgba(255, 255, 255, 0.015) 61.8%,
        rgba(255, 255, 255, 0.008) 100%);
  }

  .time-text:hover {
    color: rgba(220, 221, 222, 0.8);
    background: var(--bg-primary, rgba(47, 49, 54, 0.95));
    box-shadow: 0 6.18px 24.72px rgba(0, 0, 0, 0.15);
    /* φ * 3.8px and φ * 15.3px */
  }
}
</style>