<template>
  <div class="discord-message-wrapper">
    <div class="discord-message-list" ref="scrollContainer" @scroll="handleScroll">
      <!-- 🎯 Scroll Position Indicator -->
      <div class="scroll-position-indicator" :class="{ active: loadingMore }"></div>

      <!-- Enhanced Load More Indicator -->
      <div v-if="loadingMore" class="load-more-indicator loading">
        <div class="loading-spinner"></div>
        <span>Loading earlier messages</span>
      </div>

      <!-- Messages Container with MessageSessionGrouper Integration -->
      <div class="messages-container" ref="messagesContainer">
        <!-- 🔧 NEW: Enhanced Message Items with Session Grouper and Date Separators -->
        <template v-for="(item, index) in groupedMessages"
          :key="item.id || item.temp_id || `divider_${item.type}_${index}`">

          <!-- 🎯 TimeSessionDivider for all divider types (date-divider, sub-date-divider, session-divider) -->
          <TimeSessionDivider
            v-if="item.type === 'date-divider' || item.type === 'sub-date-divider' || item.type === 'session-divider'"
            :divider="item" :compact="item.subType === 'short-break'" />

          <!-- 🔧 ENHANCED: Message Item with Loading Context -->
          <div v-else class="message-loading-context" :data-loading-batch="loadingMore ? 'current' : 'loaded'">
            <DiscordMessageItem :message="item" :current-user-id="currentUserId" :chat-id="chatId"
              :is-grouped="shouldGroupMessage(item, index)" @user-profile-opened="$emit('user-profile-opened', $event)"
              @dm-created="$emit('dm-created', $event)" @reply-to="handleReplyTo" @edit-message="handleEditMessage"
              @delete-message="handleDeleteMessage" @scroll-to-message="handleScrollToMessage" />
          </div>
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

      <!-- Perfect Search Highlight -->
      <div v-if="searchHighlight" class="search-highlight-overlay"></div>
    </div>

    <!-- 🎯 FIXED: Enhanced Scroll to Bottom Button - 固定在视口位置 -->
    <Transition name="fade">
      <button v-if="showScrollToBottom" class="scroll-to-bottom-btn-fixed" @click="scrollToBottom(true)"
        :title="`Jump to latest${unreadCount > 0 ? ` (${unreadCount} unread)` : ''}`">
        <Icon name="chevron-down" />
        <span v-if="unreadCount > 0" class="unread-badge">{{ unreadCount }}</span>
      </button>
    </Transition>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted, nextTick, watch } from 'vue'
import { useAuthStore } from '@/stores/auth'
import { useChatStore } from '@/stores/chat'
import DiscordMessageItem from './DiscordMessageItem.vue'
import TimeSessionDivider from '@/components/chat/TimeSessionDivider.vue'
import Icon from '@/components/ui/Icon.vue'
import { debounce, throttle } from '@/utils/performance'
import { messageSessionGrouper } from '@/services/MessageSessionGrouper.js'

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

// 🔧 NEW: Process messages with MessageSessionGrouper to include sub-date dividers
const groupedMessages = computed(() => {
  if (!props.messages || props.messages.length === 0) {
    return []
  }

  try {
    // Use MessageSessionGrouper to analyze and group messages with date/session dividers
    const result = messageSessionGrouper.analyzeAndGroupMessages(props.messages, props.chatId)

    if (import.meta.env.DEV) {
      console.log(`📅 [DiscordMessageList] Grouped messages for chat ${props.chatId}:`, {
        originalCount: props.messages.length,
        groupedCount: result.groupedMessages.length,
        dividersCount: result.dividers.length,
        mainDividers: result.dividers.filter(d => d.type === 'date-divider').length,
        subDividers: result.dividers.filter(d => d.type === 'sub-date-divider').length
      })
    }

    return result.groupedMessages
  } catch (error) {
    if (import.meta.env.DEV) {
      console.error('❌ [DiscordMessageList] MessageSessionGrouper failed:', error)
    }
    // Fallback to original messages if grouper fails
    return props.messages
  }
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
const shouldGroupMessage = (item, index) => {
  // Skip grouping for dividers
  if (item.type === 'date-divider' || item.type === 'sub-date-divider' || item.type === 'session-divider') {
    return false
  }

  if (index === 0) return false

  // Find the previous message (skip any dividers)
  let prevMessage = null
  for (let i = index - 1; i >= 0; i--) {
    const prevItem = groupedMessages.value[i]
    if (!prevItem.type || (!prevItem.type.includes('divider') && !prevItem.type.includes('date'))) {
      prevMessage = prevItem
      break
    }
  }

  if (!prevMessage) return false

  // Group if same sender and within 5 minutes
  const isSameSender = item.sender_id === prevMessage.sender_id
  const timeDiff = new Date(item.created_at) - new Date(prevMessage.created_at)
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

// 🎯 OPTIMIZED: 完美历史消息加载体验 - 简单可靠的固定阅读位置
const loadMoreMessages = async () => {
  if (
    loadingMore.value ||
    !props.hasMoreMessages ||
    isLoading.value ||
    Date.now() - lastLoadTime.value < 500
  ) {
    return
  }

  loadingMore.value = true
  isLoading.value = true
  lastLoadTime.value = Date.now()

  // 🚀 KEY FIX: 暂时禁用自动滚动，防止watch干扰
  const wasAutoScrollEnabled = autoScrollEnabled.value
  autoScrollEnabled.value = false

  try {
    const container = scrollContainer.value
    if (!container) return

    // 🔧 STEP 1: 保存当前精确状态 - 只记录关键数据
    const beforeScrollTop = container.scrollTop
    const beforeScrollHeight = container.scrollHeight

    if (import.meta.env.DEV) {
      console.log('📊 [Load More] 保存状态:', {
        beforeScrollTop,
        beforeScrollHeight,
        autoScrollDisabled: true
      })
    }

    // 🔧 STEP 2: 触发历史消息加载
    await emit('load-more-messages')

    // 🔧 STEP 3: 等待DOM完全更新
    await nextTick()
    await new Promise(resolve => requestAnimationFrame(resolve))

    // 🔧 STEP 4: 简单可靠的位置恢复 - 高度差补偿法
    const afterScrollHeight = container.scrollHeight
    const heightDifference = afterScrollHeight - beforeScrollHeight

    if (heightDifference > 0) {
      // 🚀 立即调整滚动位置保持阅读位置不变
      const newScrollTop = beforeScrollTop + heightDifference
      container.scrollTop = newScrollTop

      if (import.meta.env.DEV) {
        console.log('✅ [Load More] 位置固定成功:', {
          heightAdded: heightDifference,
          beforeScrollTop,
          newScrollTop,
          验证: container.scrollTop === newScrollTop
        })
      }
    }

  } catch (error) {
    console.error('❌ [Load More] 加载失败:', error)
  } finally {
    // 🔧 STEP 5: 状态清理 - 恢复自动滚动设置
    setTimeout(() => {
      loadingMore.value = false
      isLoading.value = false
      // 🔧 恢复原始自动滚动状态
      autoScrollEnabled.value = wasAutoScrollEnabled

      if (import.meta.env.DEV) {
        console.log('🔄 [Load More] 状态清理完成，自动滚动状态恢复:', wasAutoScrollEnabled)
      }
    }, 150) // 增加一点延迟确保DOM稳定
  }
}

// 🔧 ENHANCED: 更流畅智能的滚动处理
const handleScroll = throttle((event) => {
  const { scrollTop, scrollHeight, clientHeight } = event.target

  // 🚀 IMPROVED: 更准确的滚动到底部按钮显示逻辑
  const isNearBottom = scrollTop + clientHeight >= scrollHeight - 50
  const hasScrollableContent = scrollHeight > clientHeight + 20
  const shouldShowButton = hasScrollableContent && !isNearBottom && props.messages.length > 3

  showScrollToBottom.value = shouldShowButton

  if (import.meta.env.DEV && shouldShowButton !== showScrollToBottom.value) {
    console.log('🔽 [Scroll Button] 显示状态更新:', {
      isNearBottom,
      hasScrollableContent,
      shouldShowButton,
      scrollTop: Math.round(scrollTop),
      scrollHeight,
      clientHeight,
      messagesCount: props.messages.length
    })
  }

  // 🎯 优化的历史消息加载触发条件 - 更流畅的体验
  const scrollDirection = scrollTop < lastScrollTop.value ? 'up' : 'down'

  const shouldLoadMore = (
    scrollTop < 100 &&                // 降低触发距离提供更及时的响应
    scrollDirection === 'up' &&       // 只在向上滚动时触发
    props.hasMoreMessages &&
    !loadingMore.value &&
    !isLoading.value &&
    props.messages.length > 0 &&
    Date.now() - lastLoadTime.value > 400 // 降低延迟提高响应性
  )

  if (shouldLoadMore) {
    // 🚀 立即触发加载，无需延迟
    loadMoreMessages()
  }

  // Save reading position with improved debouncing
  saveReadingPosition()

  // Send scroll position change event
  emit('scroll-position-changed', {
    scrollTop,
    scrollHeight,
    clientHeight,
    isAtBottom: isAtBottom.value,
    isNearTop: isNearTop.value,
    scrollDirection
  })

  lastScrollTop.value = scrollTop
}, 16) // 提升到60fps (16ms) 获得更流畅的体验

// 🆕 ENHANCED: 更智能的阅读位置保存机制
const saveReadingPosition = debounce(() => {
  if (!scrollContainer.value) return

  // Find the message closest to the reading line (1/3 from top)
  const viewport = scrollContainer.value
  const readingLine = viewport.scrollTop + viewport.clientHeight * 0.33 // 1/3 from top is optimal reading position

  const messageElements = viewport.querySelectorAll('[data-message-id]')
  let closestMessage = null
  let minDistance = Infinity

  messageElements.forEach(element => {
    const rect = element.getBoundingClientRect()
    const containerRect = viewport.getBoundingClientRect()
    const elementTop = rect.top - containerRect.top + viewport.scrollTop
    const distance = Math.abs(elementTop - readingLine)

    if (distance < minDistance) {
      minDistance = distance
      closestMessage = element
    }
  })

  if (closestMessage) {
    const messageId = closestMessage.getAttribute('data-message-id')
    const rect = closestMessage.getBoundingClientRect()
    const containerRect = viewport.getBoundingClientRect()

    readingPosition.value = {
      messageId: parseInt(messageId),
      offset: rect.top - containerRect.top, // Precise offset from container top
      scrollTop: viewport.scrollTop,
      timestamp: Date.now(),
      // 🆕 Add viewport context for better restoration
      viewportHeight: viewport.clientHeight,
      messageRect: {
        top: rect.top - containerRect.top,
        height: rect.height
      }
    }

    emit('reading-position-updated', readingPosition.value)

    if (import.meta.env.DEV) {
      console.log('📖 [Reading Position] Saved:', {
        messageId,
        offset: readingPosition.value.offset,
        scrollTop: readingPosition.value.scrollTop
      })
    }
  }
}, 800) // Reduced debounce for better responsiveness

// 🆕 ENHANCED: 精确的阅读位置恢复
const restoreReadingPosition = async (position) => {
  if (!position || !scrollContainer.value) return

  await nextTick()
  await new Promise(resolve => requestAnimationFrame(resolve))

  const targetElement = document.querySelector(`[data-message-id="${position.messageId}"]`)

  if (targetElement) {
    const container = scrollContainer.value
    const containerRect = container.getBoundingClientRect()
    const targetRect = targetElement.getBoundingClientRect()

    // 计算目标滚动位置以恢复相同的视觉位置
    const currentOffset = targetRect.top - containerRect.top
    const scrollAdjustment = currentOffset - position.offset
    const targetScrollTop = container.scrollTop + scrollAdjustment

    // 平滑滚动到目标位置
    container.scrollTo({
      top: targetScrollTop,
      behavior: 'smooth'
    })

    if (import.meta.env.DEV) {
      console.log('📖 [Reading Position] Restored:', {
        messageId: position.messageId,
        targetScrollTop,
        adjustment: scrollAdjustment
      })
    }

    // 短暂高亮目标消息以提供视觉反馈
    targetElement.classList.add('message-loading-context')
    setTimeout(() => {
      targetElement.classList.remove('message-loading-context')
    }, 2000)

    return true
  } else {
    // 如果找不到目标消息，使用备用滚动位置
    if (position.scrollTop) {
      scrollContainer.value.scrollTop = position.scrollTop
      if (import.meta.env.DEV) {
        console.log('📖 [Reading Position] Fallback to scrollTop:', position.scrollTop)
      }
    }
    return false
  }
}

// 🆕 智能的平滑滚动到底部
const scrollToBottom = (smooth = false) => {
  if (!scrollContainer.value) return

  const container = scrollContainer.value
  const targetScrollTop = container.scrollHeight - container.clientHeight

  if (smooth) {
    // 使用自定义平滑滚动，更好的控制
    const startScrollTop = container.scrollTop
    const distance = targetScrollTop - startScrollTop
    const duration = Math.min(500, Math.abs(distance) * 0.5) // 根据距离调整动画时长

    if (distance === 0) return

    const startTime = performance.now()

    const animateScroll = (currentTime) => {
      const elapsed = currentTime - startTime
      const progress = Math.min(elapsed / duration, 1)

      // 使用easeOutQuart缓动函数，提供更自然的减速
      const easeOutQuart = 1 - Math.pow(1 - progress, 4)
      const currentScrollTop = startScrollTop + distance * easeOutQuart

      container.scrollTop = currentScrollTop

      if (progress < 1) {
        requestAnimationFrame(animateScroll)
      } else {
        // 确保到达精确位置
        container.scrollTop = targetScrollTop
        showScrollToBottom.value = false
        autoScrollEnabled.value = true
      }
    }

    requestAnimationFrame(animateScroll)
  } else {
    // 立即滚动
    container.scrollTop = targetScrollTop
    showScrollToBottom.value = false
    autoScrollEnabled.value = true
  }
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

    // 🛡️ ENHANCED FALLBACK REGISTRATION: 立即+延迟双重保障
    const performFallbackRegistration = async (attempt = 1) => {
      if (!window.messageDisplayGuarantee || !props.chatId) return

      await nextTick()
      await new Promise(resolve => requestAnimationFrame(resolve))

      const messageElements = document.querySelectorAll(`[data-message-id]`)
      let registered = 0
      let total = 0

      messageElements.forEach(el => {
        const messageId = el.getAttribute('data-message-id')
        if (messageId && el.offsetParent !== null) {
          total++
          try {
            // 重新注册确保tracking（重复注册会被优雅处理）
            window.messageDisplayGuarantee.markMessageDisplayed(
              parseInt(messageId),
              el,
              props.chatId
            )
            registered++
          } catch (error) {
            if (import.meta.env.DEV) {
              console.warn(`⚠️ [DiscordMessageList] Fallback attempt ${attempt} failed for message ${messageId}:`, error.message)
            }
          }
        }
      })

      if (import.meta.env.DEV) {
        console.log(`✅ [DiscordMessageList] Fallback attempt ${attempt}: ${registered}/${total} messages registered`)
      }

      // 如果注册率低于90%，进行额外尝试
      if (total > 0 && (registered / total) < 0.9 && attempt < 3) {
        setTimeout(() => performFallbackRegistration(attempt + 1), 150 * attempt)
      }
    }

    // 立即尝试后备注册
    performFallbackRegistration(1)

    // 延迟后备注册
    setTimeout(() => performFallbackRegistration(2), 200)

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

    // 🔽 ENHANCED: 初始化滚动按钮状态检查
    setTimeout(() => {
      if (scrollContainer.value) {
        const container = scrollContainer.value
        const { scrollTop, scrollHeight, clientHeight } = container
        const isNearBottom = scrollTop + clientHeight >= scrollHeight - 50
        const hasScrollableContent = scrollHeight > clientHeight + 20
        const shouldShowButton = hasScrollableContent && !isNearBottom && props.messages.length > 3

        showScrollToBottom.value = shouldShowButton

        if (import.meta.env.DEV) {
          console.log('🔽 [Scroll Button] 初始状态设置:', {
            isNearBottom,
            hasScrollableContent,
            shouldShowButton,
            scrollTop: Math.round(scrollTop),
            scrollHeight,
            clientHeight,
            messagesCount: props.messages.length
          })
        }
      }

      // 🧪 测试：强制显示按钮3秒用于验证样式
      if (import.meta.env.DEV) {
        console.log('🧪 [Test] 强制显示滚动按钮3秒用于测试...')
        showScrollToBottom.value = true
        setTimeout(() => {
          // 3秒后恢复正常逻辑
          if (scrollContainer.value) {
            const container = scrollContainer.value
            const { scrollTop, scrollHeight, clientHeight } = container
            const isNearBottom = scrollTop + clientHeight >= scrollHeight - 50
            const hasScrollableContent = scrollHeight > clientHeight + 20
            showScrollToBottom.value = hasScrollableContent && !isNearBottom && props.messages.length > 3
          }
          console.log('🧪 [Test] 测试结束，恢复正常按钮显示逻辑')
        }, 3000)
      }
    }, 200)

    // 🛡️ ENHANCED INITIAL REGISTRATION: 多阶段注册策略
    const performInitialRegistration = async (phase = 1) => {
      if (!window.messageDisplayGuarantee || !props.chatId || props.messages.length === 0) {
        return
      }

      await nextTick()
      await new Promise(resolve => requestAnimationFrame(resolve))

      const messageElements = document.querySelectorAll(`[data-message-id]`)
      let registered = 0
      let failed = 0

      messageElements.forEach(el => {
        const messageId = el.getAttribute('data-message-id')
        if (messageId && el.offsetParent !== null) {
          try {
            window.messageDisplayGuarantee.markMessageDisplayed(
              parseInt(messageId),
              el,
              props.chatId
            )
            registered++
          } catch (error) {
            failed++
            if (import.meta.env.DEV) {
              console.warn(`⚠️ [DiscordMessageList] Phase ${phase} registration failed for message ${messageId}:`, error.message)
            }
          }
        }
      })

      if (import.meta.env.DEV) {
        console.log(`✅ [DiscordMessageList] Phase ${phase} registration: ${registered}/${props.messages.length} messages (${failed} failed)`)
      }

      // 如果还有失败的消息，启动下一阶段
      if (failed > 0 && phase < 3) {
        setTimeout(() => performInitialRegistration(phase + 1), 200 * phase) // 递增延迟
      }
    }

    // 启动多阶段注册
    setTimeout(() => performInitialRegistration(1), 100)   // 100ms后第一阶段
    setTimeout(() => performInitialRegistration(2), 500)   // 500ms后第二阶段  
    setTimeout(() => performInitialRegistration(3), 1200)  // 1.2s后第三阶段
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
.discord-message-wrapper {
  position: relative;
  height: 100%;
  width: 100%;
  /* 🎯 FIXED: wrapper不处理滚动，只作为定位容器 */
}

.discord-message-list {
  height: 100%;
  width: 100%;
  overflow-y: auto;
  overflow-x: hidden;
  background-color: var(--bg-primary, #36393f);
  position: relative;
  scroll-behavior: auto;
  /* 🎯 确保瞬时滚动，避免平滑滚动干扰 */
  display: flex;
  flex-direction: column;
  align-items: center;
  /* 🎯 Enhanced scrolling performance - 优化滚动性能 */
  will-change: scroll-position;
  contain: layout style paint;
  /* 🎯 NEW: 减少重排和重绘 */
  transform: translateZ(0);
  /* 启用硬件加速 */
  -webkit-overflow-scrolling: touch;
  /* iOS平滑滚动 */
  /* 🎯 NEW: 优化滚动时的渲染性能 */
  overscroll-behavior: contain;
  /* 防止过度滚动 */
}

.discord-message-list::-webkit-scrollbar {
  width: 8px;
  background: transparent;
}

.discord-message-list::-webkit-scrollbar-track {
  background: var(--scrollbar-track, rgba(47, 49, 54, 0.6));
  border-radius: 4px;
  margin: 4px 0;
}

.discord-message-list::-webkit-scrollbar-thumb {
  background: var(--scrollbar-thumb, rgba(32, 34, 37, 0.8));
  border-radius: 4px;
  transition: background 0.2s ease;
}

.discord-message-list::-webkit-scrollbar-thumb:hover {
  background: var(--scrollbar-thumb-hover, rgba(26, 29, 33, 0.9));
}

.discord-message-list::-webkit-scrollbar-thumb:active {
  background: var(--accent-primary, #5865f2);
}

.messages-container {
  max-width: 960px;
  width: 100%;
  padding: 0 16px 48px 16px;
  /* 🎯 OPTIMIZED: 添加底部padding - 黄金分割比例优化 */
  display: flex;
  flex-direction: column;
  /* 🎯 NEW: 优化加载时的布局稳定性 */
  min-height: 0;
  /* 防止flex容器过度增长 */
  flex-shrink: 0;
  /* 防止收缩导致的布局变化 */
  /* 🎯 NEW: 提供更好的渲染性能 */
  contain: layout style;
  /* 限制重排影响范围 */
}

/* 🔧 REMOVED: Old date-separator and time-separator styles - now handled by TimeSessionDivider component */

.load-more-indicator {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 12px 16px;
  gap: 10px;
  color: var(--text-muted, #72767d);
  font-size: 13px;
  font-weight: 500;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  transform: translateY(-20px);
  opacity: 0;
  position: relative;
  background: linear-gradient(180deg,
      rgba(88, 101, 242, 0.08) 0%,
      rgba(88, 101, 242, 0.03) 50%,
      transparent 100%);
  backdrop-filter: blur(6px);
  border-radius: 6px;
  margin: 6px 12px;
  border: 1px solid rgba(88, 101, 242, 0.12);
  z-index: 10;
  will-change: transform, opacity;
  contain: layout style;
}

.load-more-indicator.loading {
  transform: translateY(0);
  opacity: 1;
}

.loading-spinner {
  width: 18px;
  height: 18px;
  border: 2px solid rgba(88, 101, 242, 0.2);
  border-top: 2px solid var(--accent-primary, #5865f2);
  border-radius: 50%;
  animation: spin 1s linear infinite, pulse 2s ease-in-out infinite;
  box-shadow: 0 0 0 0 rgba(88, 101, 242, 0.4);
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

.scroll-to-bottom-btn-fixed {
  position: fixed;
  bottom: 130px;
  right: 24px;
  width: 44px;
  height: 44px;
  background: rgba(255, 255, 255, 0.95);
  border: 1px solid rgba(0, 0, 0, 0.1);
  border-radius: 50%;
  color: var(--text-secondary, #666);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  box-shadow:
    0 4px 12px rgba(0, 0, 0, 0.15),
    0 2px 6px rgba(0, 0, 0, 0.1);
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  z-index: 9999;
  backdrop-filter: blur(12px);
  /* 🚀 Enhanced visibility */
  will-change: transform, opacity;
  contain: layout style;
}

.scroll-to-bottom-btn-fixed:hover {
  background: rgba(255, 255, 255, 1);
  border-color: var(--accent-primary, #5865f2);
  color: var(--accent-primary, #5865f2);
  transform: translateY(-2px) scale(1.05);
  box-shadow:
    0 4px 12px rgba(0, 0, 0, 0.15),
    0 2px 6px rgba(0, 0, 0, 0.1);
}

.scroll-to-bottom-btn-fixed:active {
  transform: translateY(0) scale(0.95);
}

.unread-badge {
  position: absolute;
  top: -4px;
  right: -4px;
  background: var(--danger-color, #ed4245);
  color: white;
  border-radius: 12px;
  padding: 3px 8px;
  font-size: 11px;
  font-weight: 700;
  min-width: 20px;
  text-align: center;
  border: 2px solid var(--bg-primary, #36393f);
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
  animation: badgePulse 2s infinite;
}

@keyframes badgePulse {

  0%,
  70%,
  100% {
    transform: scale(1);
  }

  35% {
    transform: scale(1.1);
  }
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

@keyframes pulse {

  0%,
  100% {
    box-shadow: 0 0 0 0 rgba(88, 101, 242, 0.4);
  }

  50% {
    box-shadow: 0 0 0 4px rgba(88, 101, 242, 0);
  }
}

/* Smooth loading text animation */
.load-more-indicator span {
  position: relative;
  overflow: hidden;
}

.load-more-indicator span::after {
  content: '...';
  position: absolute;
  animation: dots 1.5s infinite;
}

@keyframes dots {

  0%,
  20% {
    opacity: 0;
  }

  50% {
    opacity: 1;
  }

  100% {
    opacity: 0;
  }
}

/* 🎯 Smooth scroll position indicator */
.scroll-position-indicator {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 2px;
  background: linear-gradient(90deg,
      var(--accent-primary, #5865f2) 0%,
      rgba(88, 101, 242, 0.6) 50%,
      rgba(88, 101, 242, 0.2) 100%);
  transform: scaleX(0);
  transform-origin: left;
  transition: transform 0.3s ease;
  z-index: 20;
}

.scroll-position-indicator.active {
  transform: scaleX(1);
}

/* Enhanced message highlighting for better reading continuity */
:deep(.message-loading-context) {
  position: relative;
}

:deep(.message-loading-context::before) {
  content: '';
  position: absolute;
  left: -4px;
  top: -2px;
  bottom: -2px;
  width: 3px;
  background: linear-gradient(180deg,
      transparent 0%,
      var(--accent-primary, #5865f2) 20%,
      var(--accent-primary, #5865f2) 80%,
      transparent 100%);
  border-radius: 2px;
  opacity: 0;
  animation: contextHighlight 2s ease-out;
}

@keyframes contextHighlight {
  0% {
    opacity: 0;
    transform: scaleY(0);
  }

  20% {
    opacity: 1;
    transform: scaleY(1);
  }

  80% {
    opacity: 1;
    transform: scaleY(1);
  }

  100% {
    opacity: 0;
    transform: scaleY(1);
  }
}

/* Tablet responsive */
@media (max-width: 1024px) and (min-width: 769px) {
  .messages-container {
    padding: 0 14px 42px 14px;
    /* 🎯 TABLET: 平板端中间值优化 */
  }
}

/* Mobile responsive */
@media (max-width: 768px) {
  .messages-container {
    padding: 0 12px 36px 12px;
    /* 🎯 MOBILE: 移动端优化底部间距，减少左右padding */
  }

  .scroll-to-bottom-btn-fixed {
    bottom: 150px;
    right: 16px;
    width: 40px;
    height: 40px;
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
  .discord-message-wrapper {
    scroll-behavior: auto;
  }

  .typing-dots span {
    animation: none;
  }

  .scroll-to-bottom-btn-fixed {
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
  }

  .date-text:hover {
    color: rgba(220, 221, 222, 0.95);
    background: var(--bg-primary, rgba(47, 49, 54, 0.95));
    box-shadow: 0 4.85px 19.4px rgba(0, 0, 0, 0.2);
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
  }
}
</style>