<template>
  <!-- 🎯 Perfect Search Modal - Jobs-inspired Design -->
  <teleport to="body">
    <transition name="modal-fade">
      <div v-if="isOpen" class="search-backdrop" @click="handleClose" @keydown.esc="handleClose">
        <transition name="modal-scale">
          <div class="search-modal" @click.stop ref="modalRef">
            <!-- Minimalist Header -->
            <header class="modal-header">
              <div class="search-branding">
                <div class="search-icon-elegant">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor">
                    <circle cx="11" cy="11" r="8" />
                    <path d="m21 21-4.35-4.35" />
                  </svg>
                </div>
                <h1 class="search-title">Search</h1>
              </div>

              <button @click="handleClose" class="close-button" aria-label="Close">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor">
                  <path d="M18 6L6 18M6 6l12 12" />
                </svg>
              </button>
            </header>

            <!-- 🔍 Modern Search Input -->
            <section class="search-section">
              <div class="search-input-container">
                <!-- Search Icon - Always Visible -->
                <div class="search-icon-wrapper">
                  <svg class="search-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <circle cx="11" cy="11" r="8" />
                    <path d="m21 21-4.35-4.35" />
                  </svg>
                </div>

                <input ref="searchInput" v-model="searchQuery" @input="handleSearch" @keydown.enter="executeSearch"
                  @keydown.escape="handleClose" @keydown.up="navigateResults(-1)" @keydown.down="navigateResults(1)"
                  type="text" placeholder="Search messages, files, and more..." class="search-input" autocomplete="off"
                  spellcheck="false" :disabled="isSearching" />

                <!-- Search Actions -->
                <div class="search-actions">
                  <!-- Clear Button -->
                  <button v-if="searchQuery && !isSearching" @click="clearSearch" class="clear-button"
                    aria-label="Clear search">
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                      <line x1="18" y1="6" x2="6" y2="18" />
                      <line x1="6" y1="6" x2="18" y2="18" />
                    </svg>
                  </button>

                  <!-- Loading Indicator -->
                  <div v-if="isSearching" class="loading-indicator">
                    <div class="loading-spinner">
                      <svg class="spinner" viewBox="0 0 24 24">
                        <circle cx="12" cy="12" r="10" fill="none" stroke="currentColor" stroke-width="2"
                          stroke-linecap="round" stroke-dasharray="31.416" stroke-dashoffset="31.416">
                          <animate attributeName="stroke-dasharray" dur="2s" values="0 31.416;15.708 15.708;0 31.416"
                            repeatCount="indefinite" />
                          <animate attributeName="stroke-dashoffset" dur="2s" values="0;-15.708;-31.416"
                            repeatCount="indefinite" />
                        </circle>
                      </svg>
                    </div>
                  </div>

                  <!-- Search Button - Always Visible -->
                  <button @click="executeSearch" class="search-button" :class="{ active: searchQuery }"
                    :disabled="!searchQuery || isSearching" aria-label="Search">
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                      <circle cx="11" cy="11" r="8" />
                      <path d="m21 21-4.35-4.35" />
                    </svg>
                  </button>
                </div>
              </div>

              <!-- Advanced Search Strategies -->
              <div class="strategies-section" v-if="hasSearched">
                <div class="strategies-header">
                  <button @click="showAdvancedStrategies = !showAdvancedStrategies" class="strategies-toggle">
                    <span class="toggle-icon">🔧</span>
                    <span>Advanced Search</span>
                    <svg class="chevron-icon" :class="{ rotated: showAdvancedStrategies }" viewBox="0 0 24 24"
                      fill="none" stroke="currentColor">
                      <path d="M6 9l6 6 6-6" />
                    </svg>
                  </button>
                </div>

                <transition name="strategies-slide">
                  <div v-if="showAdvancedStrategies" class="strategies-container">
                    <div class="strategy-pills">
                      <button v-for="strategy in searchStrategies" :key="strategy.id" @click="toggleStrategy(strategy)"
                        class="strategy-pill" :class="{ active: activeFilters.includes(strategy.id) }"
                        :title="strategy.description">
                        <span class="strategy-icon">{{ strategy.icon }}</span>
                        <span class="strategy-label">{{ strategy.label }}</span>
                      </button>
                    </div>
                  </div>
                </transition>
              </div>
            </section>

            <!-- Beautiful Results -->
            <section class="results-section" v-if="hasSearched">
              <div class="results-header" v-if="searchResults.length > 0">
                <div class="results-info">
                  <span class="results-count">{{ searchResults.length }} results</span>
                  <span class="search-source" v-if="searchSource">
                    {{ getSearchSourceLabel(searchSource) }}
                  </span>
                </div>
                <span class="search-time" v-if="searchTime">{{ searchTime }}ms</span>
              </div>

              <!-- Premium Results List with Ergonomic Design -->
              <div class="results-container" v-if="searchResults.length > 0">
                <div v-for="(result, index) in searchResults" :key="result.id" @click="jumpToMessage(result)"
                  class="result-item" :class="{ focused: focusedIndex === index }"
                  :aria-selected="focusedIndex === index" role="button" tabindex="0"
                  @keydown.enter="jumpToMessage(result)">
                  <div class="result-message-container">
                    <!-- User Avatar with Color -->
                    <div class="result-avatar"
                      :style="getAvatarStyle(result.sender_name || result.sender_fullname || result.sender?.fullname || 'Unknown')">
                      {{ getInitials(result.sender_name || result.sender_fullname || result.sender?.fullname ||
                        'Unknown') }}
                    </div>

                    <!-- Message Content Area -->
                    <div class="result-content-area">
                      <!-- Message Header -->
                      <div class="result-header">
                        <span class="result-sender">
                          {{ result.sender_name || result.sender_fullname || result.sender?.fullname || 'Unknown User'
                          }}
                        </span>
                        <span class="result-time">{{ formatTime(result.created_at) }}</span>
                      </div>

                      <!-- Channel Information -->
                      <div class="result-chat" v-if="result.chat_name">
                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                          <path
                            d="M21 11.5a8.38 8.38 0 01-.9 3.8 8.5 8.5 0 01-7.6 4.7 8.38 8.38 0 01-3.8-.9L3 21l1.9-5.7a8.38 8.38 0 01-.9-3.8 8.5 8.5 0 014.7-7.6 8.38 8.38 0 013.8-.9h.5a8.48 8.48 0 018 8v.5z" />
                        </svg>
                        <span>{{ result.chat_name }}</span>
                      </div>

                      <!-- Message Content with Enhanced Typography -->
                      <div class="result-content"
                        v-html="highlightText(result.content || result.text || '', searchQuery)"
                        :title="result.content || result.text || ''"></div>
                    </div>
                  </div>
                </div>
              </div>

              <!-- Error State -->
              <div v-else-if="searchError" class="error-state">
                <div class="error-icon">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor">
                    <path
                      d="M12 9v3.75m-9.303 3.376c-.866 1.5.217 3.374 1.948 3.374h14.71c1.73 0 2.813-1.874 1.948-3.374L13.949 3.378c-.866-1.5-3.032-1.5-3.898 0L2.697 16.126zM12 15.75h.007v.008H12v-.008z" />
                  </svg>
                </div>
                <h3 class="error-title">Search temporarily unavailable</h3>
                <p class="error-description">{{ searchError }}</p>
                <button @click="handleSearch" class="retry-button">
                  Try Again
                </button>
              </div>

              <!-- Empty State -->
              <div v-else-if="searchQuery && !isSearching && searchResults.length === 0" class="empty-state">
                <div class="empty-icon">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor">
                    <circle cx="11" cy="11" r="8" />
                    <path d="m21 21-4.35-4.35" />
                  </svg>
                </div>
                <h3 class="empty-title">No results found</h3>
                <p class="empty-description">Try different keywords or check your spelling</p>
              </div>
            </section>

            <!-- Initial State -->
            <section v-else class="welcome-section">
              <div class="welcome-content">
                <div class="welcome-icon">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor">
                    <circle cx="11" cy="11" r="8" />
                    <path d="m21 21-4.35-4.35" />
                  </svg>
                </div>
                <h2 class="welcome-title">Find anything, instantly</h2>
                <p class="welcome-subtitle">Search through all your messages and files</p>

                <div class="quick-suggestions">
                  <span class="suggestions-label">Popular searches:</span>
                  <div class="suggestion-tags">
                    <button v-for="suggestion in suggestions" :key="suggestion" @click="applySuggestion(suggestion)"
                      class="suggestion-tag">
                      {{ suggestion }}
                    </button>
                  </div>
                </div>
              </div>
            </section>

            <!-- Keyboard Hints -->
            <footer class="modal-footer">
              <div class="keyboard-hints">
                <kbd>⌘K</kbd> to search
                <kbd>↵</kbd> to select
                <kbd>esc</kbd> to close
              </div>
            </footer>
          </div>
        </transition>
      </div>
    </transition>
  </teleport>
</template>

<script setup>
import { ref, computed, watch, onMounted, nextTick } from 'vue'
import { useDebounceFn } from '@vueuse/core'
import { useRouter } from 'vue-router'

// Import navigation styles
import '@/styles/messageNavigation.css'

// 🔧 Router instance for navigation
const router = useRouter()

// Props
const props = defineProps({
  isOpen: {
    type: Boolean,
    default: false
  },
  chatId: {
    type: [String, Number],
    default: null
  }
})

// Emits
const emit = defineEmits(['close', 'navigate'])

// State
const searchInput = ref(null)
const modalRef = ref(null)
const searchQuery = ref('')
const searchResults = ref([])
const isSearching = ref(false)
const hasSearched = ref(false)
const searchTime = ref(null)
const focusedIndex = ref(-1)
const activeFilters = ref([])
const searchError = ref(null)
const searchSource = ref(null)

// 🔍 Advanced Search Strategies
const searchStrategies = [
  {
    id: 'semantic',
    label: 'Semantic Search',
    icon: '🧠',
    description: 'AI-powered meaning-based search'
  },
  {
    id: 'exact',
    label: 'Exact Match',
    icon: '🎯',
    description: 'Find exact phrase matches'
  },
  {
    id: 'fuzzy',
    label: 'Fuzzy Search',
    icon: '🔍',
    description: 'Tolerates typos and variations'
  },
  {
    id: 'recent',
    label: 'Recent First',
    icon: '⏰',
    description: 'Prioritize recent messages'
  },
  {
    id: 'user',
    label: 'User Focused',
    icon: '👤',
    description: 'Search by specific users'
  },
  {
    id: 'files',
    label: 'Files & Media',
    icon: '📎',
    description: 'Find shared files and images'
  }
]

// Show advanced strategies toggle
const showAdvancedStrategies = ref(false)

// Dynamic suggestions from search service
const suggestions = ref([
  'project updates',
  'meeting notes',
  'deadlines',
  'files shared'
])

// Methods
const handleClose = () => {
  emit('close')
}

// 🔐 Authentication-aware search with graceful fallback
const handleSearch = useDebounceFn(async () => {
  if (!searchQuery.value.trim()) {
    searchResults.value = []
    hasSearched.value = false
    searchError.value = null
    searchSource.value = null
    return
  }

  isSearching.value = true
  hasSearched.value = true
  searchError.value = null

  try {
    // 🛡️ Step 1: Check authentication status before attempting API search
    const authStore = (await import('@/stores/auth')).useAuthStore()
    const isAuthenticated = authStore?.isAuthenticated || false
    const hasValidToken = authStore?.token && authStore?.isTokenValid?.() !== false

    console.log('🔍 [PerfectSearch] Auth check:', {
      isAuthenticated,
      hasValidToken,
      tokenExists: !!authStore?.token
    })

    let searchResponse = null
    let currentSearchSource = 'unknown'

    // 🚀 Step 2: Try API search if authenticated
    if (isAuthenticated && hasValidToken) {
      try {
        const searchService = (await import('@/services/searchService')).default

        const searchParams = {
          query: searchQuery.value.trim(),
          chatId: props.chatId,
          limit: 50,
          offset: 0,
          // 🚀 Apply advanced search strategies
          ...buildSearchStrategies()
        }

        console.log('🔍 [PerfectSearch] Attempting API search:', searchParams)
        searchResponse = await searchService.intelligentSearch(searchParams)
        currentSearchSource = 'api'

      } catch (apiError) {
        console.warn('🔍 [PerfectSearch] API search failed, trying fallback:', apiError.message)

        // 🔄 Step 3: Handle specific authentication errors gracefully
        if (apiError.message?.includes('401') ||
          apiError.message?.includes('Unauthorized') ||
          apiError.message?.includes('token') ||
          apiError.message?.includes('expired')) {

          // Don't throw auth errors, just fall back to local search
          console.log('🔍 [PerfectSearch] Authentication issue detected, using local search')
          searchResponse = await performLocalSearch()
          currentSearchSource = 'local_auth_fallback'
        } else {
          // For non-auth errors, still try local search
          searchResponse = await performLocalSearch()
          currentSearchSource = 'local_error_fallback'
        }
      }
    } else {
      // 🏠 Step 4: Use local search if not authenticated
      console.log('🔍 [PerfectSearch] Not authenticated, using local search')
      searchResponse = await performLocalSearch()
      currentSearchSource = 'local_no_auth'
    }

    // 📊 Step 5: Process results regardless of source
    if (searchResponse) {
      searchResults.value = searchResponse.hits || searchResponse.results || []
      searchTime.value = searchResponse.took_ms || searchResponse.duration || 0
      searchSource.value = currentSearchSource

      console.log('🔍 [PerfectSearch] Search completed:', {
        source: currentSearchSource,
        resultCount: searchResults.value.length,
        duration: searchTime.value
      })
    } else {
      throw new Error('No search results available from any source')
    }

  } catch (error) {
    console.error('🔍 [PerfectSearch] Search completely failed:', error)

    // 🎯 Final fallback: show user-friendly error
    if (error.message?.includes('auth') || error.message?.includes('token')) {
      searchError.value = 'Search requires authentication. Please check your connection.'
    } else {
      searchError.value = 'Search temporarily unavailable. Please try again.'
    }

    searchResults.value = []
  } finally {
    isSearching.value = false
  }
}, 300)

// 🏠 Local search implementation for authentication fallback
const performLocalSearch = async () => {
  try {
    console.log('🏠 [LocalSearch] Starting enhanced local message search...')

    // 🔧 增强：多种消息获取方式
    let localMessages = []
    let messageSource = 'unknown'

    // 方案1: 尝试 UnifiedMessageService
    try {
      const UnifiedMessageService = window.unifiedMessageService || window.UnifiedMessageService
      if (UnifiedMessageService && typeof UnifiedMessageService.getMessagesForChat === 'function') {
        localMessages = UnifiedMessageService.getMessagesForChat(props.chatId) || []
        messageSource = 'UnifiedMessageService'
        console.log(`🏠 [LocalSearch] ✅ Got ${localMessages.length} messages from UnifiedMessageService`)
      }
    } catch (error) {
      console.warn('🏠 [LocalSearch] UnifiedMessageService failed:', error.message)
    }

    // 方案2: 尝试 ChatStore
    if (localMessages.length === 0) {
      try {
        const { useChatStore } = await import('@/stores/chat')
        const chatStore = useChatStore()
        if (chatStore && chatStore.messages) {
          localMessages = Object.values(chatStore.messages).filter(msg =>
            msg.chat_id == props.chatId || msg.chatId == props.chatId
          )
          messageSource = 'ChatStore'
          console.log(`🏠 [LocalSearch] ✅ Got ${localMessages.length} messages from ChatStore`)
        }
      } catch (error) {
        console.warn('🏠 [LocalSearch] ChatStore failed:', error.message)
      }
    }

    // 方案3: 尝试从DOM提取
    if (localMessages.length === 0) {
      try {
        const messageElements = document.querySelectorAll('[data-message-id]')
        localMessages = Array.from(messageElements).map(el => {
          const messageId = el.getAttribute('data-message-id')
          const textContent = el.querySelector('.message-content, .message-text')?.textContent || ''
          const senderElement = el.querySelector('.message-sender, .sender-name, .user-name')
          const senderName = senderElement?.textContent || 'Unknown'
          const timestampElement = el.querySelector('.message-timestamp, .timestamp')
          const timestamp = timestampElement?.getAttribute('data-timestamp') ||
            timestampElement?.textContent ||
            Date.now()

          return {
            id: parseInt(messageId),
            content: textContent,
            text: textContent,
            sender_name: senderName,
            sender_fullname: senderName,
            chat_id: props.chatId,
            chatId: props.chatId,
            created_at: timestamp,
            timestamp: timestamp,
            chat_name: `Chat ${props.chatId}`
          }
        }).filter(msg => msg.id && msg.content)

        messageSource = 'DOM_Extraction'
        console.log(`🏠 [LocalSearch] ✅ Got ${localMessages.length} messages from DOM extraction`)
      } catch (error) {
        console.warn('🏠 [LocalSearch] DOM extraction failed:', error.message)
      }
    }

    // 方案4: 尝试从浏览器缓存
    if (localMessages.length === 0) {
      try {
        const cacheKey = `chat_messages_${props.chatId}`
        const cached = localStorage.getItem(cacheKey) || sessionStorage.getItem(cacheKey)
        if (cached) {
          const parsedMessages = JSON.parse(cached)
          if (Array.isArray(parsedMessages)) {
            localMessages = parsedMessages
            messageSource = 'Browser_Cache'
            console.log(`🏠 [LocalSearch] ✅ Got ${localMessages.length} messages from browser cache`)
          }
        }
      } catch (error) {
        console.warn('🏠 [LocalSearch] Browser cache failed:', error.message)
      }
    }

    // 检查是否有可搜索的消息
    if (localMessages.length === 0) {
      console.warn('🏠 [LocalSearch] ⚠️ No messages found from any source')
      return {
        hits: [],
        took_ms: 0,
        source: 'local_no_messages',
        message_sources_tried: ['UnifiedMessageService', 'ChatStore', 'DOM_Extraction', 'Browser_Cache'],
        chatId: props.chatId
      }
    }

    const startTime = Date.now()
    const query = searchQuery.value.trim().toLowerCase()

    console.log(`🏠 [LocalSearch] 🔍 Searching ${localMessages.length} messages from ${messageSource} for: "${query}"`)

    // 🔍 Advanced local search algorithm with strategy support
    const strategies = buildSearchStrategies()

    const searchResults = localMessages
      .filter(message => {
        // 🔧 更强的内容提取
        const content = (
          message.content ||
          message.text ||
          message.body ||
          message.message ||
          ''
        ).toLowerCase()

        const senderName = (
          message.sender_name ||
          message.sender_fullname ||
          message.sender?.fullname ||
          message.sender?.name ||
          message.user_name ||
          message.username ||
          'Unknown'
        ).toLowerCase()

        const chatName = (
          message.chat_name ||
          message.channel_name ||
          ''
        ).toLowerCase()

        // 基础过滤：确保有内容可搜索
        if (!content && !senderName) {
          return false
        }

        // 🎯 Apply exact match strategy
        if (strategies.exactMatch) {
          return content.includes(query) || senderName.includes(query)
        }

        // 🔍 Apply fuzzy search strategy
        if (strategies.fuzzySearch) {
          return fuzzyMatch(content, query) ||
            fuzzyMatch(senderName, query) ||
            content.includes(query) ||
            senderName.includes(query)
        }

        // 👤 Apply user-focused strategy
        if (strategies.userFocused) {
          return senderName.includes(query) || content.includes(query)
        }

        // Default: semantic-like search
        return content.includes(query) ||
          senderName.includes(query) ||
          chatName.includes(query)
      })
      .map(message => ({
        id: message.id,
        content: message.content || message.text || message.body || '',
        sender_name: message.sender_name || message.sender_fullname || message.sender?.fullname || 'Unknown',
        chat_name: message.chat_name || message.channel_name || `Chat ${props.chatId}`,
        created_at: message.created_at || message.timestamp || Date.now(),
        chat_id: message.chat_id || message.chatId || props.chatId,
        score: calculateLocalScore(message, query, strategies)
      }))
      .sort((a, b) => {
        // 🕐 Apply recent-first strategy
        if (strategies.sortBy === 'date_desc') {
          return new Date(b.created_at) - new Date(a.created_at)
        }
        return b.score - a.score // Default: sort by relevance
      })
      .slice(0, 50) // Limit results

    const duration = Date.now() - startTime

    console.log(`🏠 [LocalSearch] ✅ Completed in ${duration}ms, found ${searchResults.length} results from ${messageSource}`)

    return {
      hits: searchResults,
      took_ms: duration,
      source: `local_${messageSource.toLowerCase()}`,
      total: searchResults.length,
      message_source: messageSource,
      original_messages_count: localMessages.length,
      chatId: props.chatId
    }

  } catch (error) {
    console.error('🏠 [LocalSearch] ❌ All fallback methods failed:', error)

    // 🔧 最终备用：返回空结果但不完全失败
    return {
      hits: [],
      took_ms: 0,
      source: 'local_fallback_failed',
      error: error.message,
      chatId: props.chatId,
      fallback_attempted: true
    }
  }
}

// 🔍 Fuzzy string matching for typo tolerance
const fuzzyMatch = (text, query, maxDistance = 2) => {
  if (!text || !query) return false

  // Simple fuzzy matching using edit distance concept
  const words = text.split(' ')
  const queryWords = query.split(' ')

  return queryWords.some(queryWord => {
    return words.some(word => {
      // Check if word is close enough to query word
      const distance = levenshteinDistance(word.toLowerCase(), queryWord.toLowerCase())
      return distance <= maxDistance
    })
  })
}

// 📏 Calculate Levenshtein distance for fuzzy matching
const levenshteinDistance = (str1, str2) => {
  const matrix = []

  for (let i = 0; i <= str2.length; i++) {
    matrix[i] = [i]
  }

  for (let j = 0; j <= str1.length; j++) {
    matrix[0][j] = j
  }

  for (let i = 1; i <= str2.length; i++) {
    for (let j = 1; j <= str1.length; j++) {
      if (str2.charAt(i - 1) === str1.charAt(j - 1)) {
        matrix[i][j] = matrix[i - 1][j - 1]
      } else {
        matrix[i][j] = Math.min(
          matrix[i - 1][j - 1] + 1, // substitution
          matrix[i][j - 1] + 1,     // insertion
          matrix[i - 1][j] + 1      // deletion
        )
      }
    }
  }

  return matrix[str2.length][str1.length]
}

// 📊 Calculate relevance score for local search with strategy support
const calculateLocalScore = (message, query, strategies = {}) => {
  const content = (message.content || message.text || '').toLowerCase()
  const senderName = (message.sender_name || '').toLowerCase()

  let score = 0

  // 🎯 Exact phrase match gets highest score
  if (content.includes(query)) {
    score += strategies.exactMatch ? 200 : 100
  }

  // 👤 Boost user-focused searches
  if (strategies.userFocused && senderName.includes(query)) {
    score += 150
  }

  // Word matches
  const queryWords = query.split(' ')
  queryWords.forEach(word => {
    if (content.includes(word)) {
      score += strategies.boostUserMatches ? 15 : 10
    }
    if (senderName.includes(word)) {
      score += strategies.userFocused ? 25 : 5
    }
  })

  // 🔍 Fuzzy match bonus
  if (strategies.fuzzySearch) {
    if (fuzzyMatch(content, query)) score += 30
    if (fuzzyMatch(senderName, query)) score += 15
  }

  // 🕐 Boost recent messages (if not using date sort)
  if (strategies.sortBy !== 'date_desc') {
    const messageAge = Date.now() - new Date(message.created_at || 0).getTime()
    const dayInMs = 24 * 60 * 60 * 1000
    if (messageAge < dayInMs) score += strategies.timeRange ? 40 : 20
    else if (messageAge < dayInMs * 7) score += strategies.timeRange ? 20 : 10
  }

  return score
}

const executeSearch = () => {
  // If there's a focused result, navigate to it
  if (focusedIndex.value >= 0 && searchResults.value[focusedIndex.value]) {
    jumpToMessage(searchResults.value[focusedIndex.value])
    return
  }

  // Otherwise, perform search
  handleSearch()
}

// 🧹 Clear search input and results
const clearSearch = () => {
  searchQuery.value = ''
  searchResults.value = []
  hasSearched.value = false
  searchError.value = null
  searchSource.value = null
  focusedIndex.value = -1

  // Focus back to input
  nextTick(() => {
    searchInput.value?.focus()
  })
}

// ⌨️ Navigate search results with keyboard
const navigateResults = (direction) => {
  if (!searchResults.value.length) return

  if (direction === -1) {
    // Up arrow
    focusedIndex.value = focusedIndex.value <= 0
      ? searchResults.value.length - 1
      : focusedIndex.value - 1
  } else {
    // Down arrow
    focusedIndex.value = focusedIndex.value >= searchResults.value.length - 1
      ? 0
      : focusedIndex.value + 1
  }

  // Scroll focused item into view
  nextTick(() => {
    const focusedElement = document.querySelector('.result-item.focused')
    if (focusedElement) {
      focusedElement.scrollIntoView({
        behavior: 'smooth',
        block: 'nearest'
      })
    }
  })
}

// 🔧 Build advanced search parameters based on selected strategies
const buildSearchStrategies = () => {
  const strategies = {}

  if (activeFilters.value.includes('semantic')) {
    strategies.useSemanticSearch = true
    strategies.strategy = 'semantic'
  }
  if (activeFilters.value.includes('exact')) {
    strategies.exactMatch = true
    strategies.strategy = 'exact'
  }
  if (activeFilters.value.includes('fuzzy')) {
    strategies.fuzzySearch = true
    strategies.tolerance = 2 // Allow up to 2 character differences
  }
  if (activeFilters.value.includes('recent')) {
    strategies.timeRange = 'week'
    strategies.sortBy = 'date_desc'
  }
  if (activeFilters.value.includes('user')) {
    strategies.userFocused = true
    strategies.boostUserMatches = true
  }
  if (activeFilters.value.includes('files')) {
    strategies.fileType = 'any'
    strategies.includeAttachments = true
  }

  return strategies
}

// 🎯 Toggle search strategy selection
const toggleStrategy = (strategy) => {
  const index = activeFilters.value.indexOf(strategy.id)
  if (index > -1) {
    activeFilters.value.splice(index, 1)
  } else {
    // For exclusive strategies like search type, remove conflicting ones
    if (['semantic', 'exact', 'fuzzy'].includes(strategy.id)) {
      activeFilters.value = activeFilters.value.filter(id => !['semantic', 'exact', 'fuzzy'].includes(id))
    }
    activeFilters.value.push(strategy.id)
  }

  console.log('🎯 [Strategies] Active strategies:', activeFilters.value)

  // Re-search with new strategies
  handleSearch()
}

const applySuggestion = (suggestion) => {
  searchQuery.value = suggestion
  handleSearch()
}

// 🎯 Enhanced Production-grade message navigation with timeout and error recovery
const jumpToMessage = async (result) => {
  console.log('🎯 [MessageJump] 🎯 DAG-Enhanced: Starting navigation chain:', {
    messageId: result.id,
    chatId: result.chat_id || result.chatId,
    sender: result.sender_name,
    source: 'perfect_search_modal'
  })

  // 🔧 新增：运行DAG诊断
  const diagnostics = await runDAGDiagnostics(result)
  console.log('🔍 [MessageJump] 🎯 DAG-Diagnostics: System state:', diagnostics)

  try {
    // 🔧 预检查：验证基本参数
    if (!result.id || !result.chat_id) {
      throw new Error('Invalid search result: missing id or chat_id')
    }

    // 🔧 增强：保存搜索结果上下文用于时间戳估算
    window.lastSearchResults = searchResults.value
    console.log('📜 [PerfectSearch] 🎯 DAG: Saved search context for enhanced navigation', {
      totalResults: searchResults.value.length,
      searchSource: searchSource.value
    })

    console.log('🎯 [PerfectSearch] 🎯 DAG: Executing perfect navigation with enhanced error handling')

    // Import Perfect Navigation Controller
    const { perfectNavigationController } = await import('@/utils/PerfectNavigationController')

    // Close modal first for better UX
    handleClose()

    // 🔧 Enhanced navigation with timeout and fallback
    const navigationPromise = perfectNavigationController.navigateToMessage({
      messageId: result.id,
      chatId: result.chat_id || result.chatId,
      searchQuery: searchQuery.value,

      // Perfect Navigation Settings
      scrollBehavior: 'smooth',
      highlightDuration: 4000, // 增加到4秒
      pulseAnimation: true,
      showIndicator: true,

      // Analytics context
      source: 'perfect_search',
      resultIndex: searchResults.value.findIndex(r => r.id === result.id),
      totalResults: searchResults.value.length,
      searchSource: searchSource.value,
      searchTime: searchTime.value,

      // 🔧 新增：诊断信息
      diagnostics: diagnostics
    })

    // 🔧 设置15秒超时（增加时间）
    const timeoutPromise = new Promise((_, reject) =>
      setTimeout(() => reject(new Error('Navigation timeout after 15 seconds')), 15000)
    )

    const navigationResult = await Promise.race([navigationPromise, timeoutPromise])

    if (navigationResult.success) {
      // 🔧 增强：分析导航结果，显示策略信息
      const strategy = navigationResult.pipeline?.stages?.enhanced_message_context?.strategy
      const loadedMessages = navigationResult.pipeline?.stages?.enhanced_message_context?.loadedMessagesCount

      console.log('✅ [PerfectSearch] 🎯 DAG: Enhanced navigation completed:', {
        navigationId: navigationResult.navigationId,
        duration: navigationResult.duration,
        strategy: strategy,
        loadedMessages: loadedMessages,
        stages: Object.keys(navigationResult.pipeline?.stages || {})
      })

      // 🎯 显示成功反馈（如果使用了历史加载）
      if (strategy && strategy !== 'already_loaded' && loadedMessages > 0) {
        console.log(`🎯 [PerfectSearch] 🎯 DAG: Used ${strategy} to load ${loadedMessages} messages for stable navigation`)
      }

      emit('navigate', {
        messageId: result.id,
        chatId: result.chat_id || result.chatId,
        searchQuery: searchQuery.value,
        success: true,
        perfect: true,
        enhanced: true,
        strategy: strategy,
        loadedMessages: loadedMessages,
        navigationResult,
        diagnostics: diagnostics
      })

    } else {
      console.warn('⚠️ [PerfectSearch] 🎯 DAG: Enhanced navigation failed, attempting graceful fallback')
      await performGracefulFallback(result)
    }

  } catch (error) {
    console.error('❌ [PerfectSearch] 🎯 DAG: Navigation error:', error)

    // 🔧 智能错误处理
    if (error.message.includes('Chat data loading timeout')) {
      await handleChatTimeoutError(result, error)
    } else if (error.message.includes('Navigation timeout')) {
      await handleNavigationTimeoutError(result, error)
    } else {
      await performGracefulFallback(result, error)
    }
  }

  // Analytics tracking
  try {
    const analytics = window.analytics || window.gtag
    if (analytics) {
      analytics('track', 'search_result_clicked', {
        message_id: result.id,
        chat_id: result.chat_id || result.chatId,
        search_query: searchQuery.value,
        result_position: searchResults.value.findIndex(r => r.id === result.id) + 1,
        search_source: searchSource.value
      })
    }
  } catch (error) {
    console.warn('🎨 Analytics tracking failed:', error)
  }
}

// 🔧 增强：Chat超时错误处理（智能版）
const handleChatTimeoutError = async (result, error) => {
  console.log('🔄 [PerfectSearch] Handling chat timeout with enhanced recovery')

  try {
    // 🔧 智能错误分析
    if (error.message.includes('does not exist')) {
      // Chat不存在 - 显示友好提示
      showUserFriendlyError(result, `Chat ${result.chat_id} is no longer available`)
      return
    }

    if (error.message.includes('no access permission')) {
      // 无权限访问 - 显示权限提示
      showUserFriendlyError(result, `You don't have permission to access this chat`)
      return
    }

    // 其他超时错误 - 尝试基础导航
    const safeRouter = getSafeRouter()

    if (safeRouter) {
      console.log('🔄 [PerfectSearch] Using router for timeout recovery')
      await safeRouter.push(`/chat/${result.chat_id}`)

      // 给页面时间加载，然后检查
      setTimeout(async () => {
        const chatPageLoaded = document.querySelector('.chat-container, .simple-message-list, .messages-container')

        if (chatPageLoaded) {
          const messageElement = document.querySelector(`[data-message-id="${result.id}"]`)
          if (messageElement) {
            scrollToMessageSafely(messageElement)

            emit('navigate', {
              messageId: result.id,
              chatId: result.chat_id,
              success: true,
              fallback: 'direct_navigation',
              reason: 'chat_timeout_recovery'
            })
          } else {
            console.log(`🔄 [PerfectSearch] Message not visible, attempting history load`)
            await waitForChatLoadAndHistoryMessages(result)
          }
        } else {
          showUserFriendlyError(result, 'Unable to access the chat')
        }
      }, 2000)
    } else {
      // 🔧 修复：避免window.location.href
      console.warn('⚠️ [PerfectSearch] Router unavailable for timeout recovery')
      showUserFriendlyError(result, 'Navigation system temporarily unavailable')
    }

  } catch (fallbackError) {
    console.error('❌ [PerfectSearch] Enhanced recovery failed:', fallbackError)
    showUserFriendlyError(result, 'Chat is temporarily unavailable')
  }
}

// 🔧 新增：导航超时错误处理
const handleNavigationTimeoutError = async (result, error) => {
  console.log('⏰ [PerfectSearch] Navigation timeout, performing emergency fallback')

  try {
    const safeRouter = getSafeRouter()

    if (safeRouter) {
      console.log('⏰ [PerfectSearch] Using router for timeout fallback')
      await safeRouter.push(`/chat/${result.chat_id}`)

      // 🔧 新增：超时后也尝试等待和查找消息
      setTimeout(async () => {
        await waitForChatLoadAndHistoryMessages(result)
      }, 1000)

      emit('navigate', {
        messageId: result.id,
        chatId: result.chat_id,
        success: true,
        emergency: true,
        reason: 'navigation_timeout'
      })
    } else {
      // 🔧 修复：避免window.location.href
      console.warn('⚠️ [PerfectSearch] Router unavailable for emergency fallback')
      showUserFriendlyError(result, 'Navigation system temporarily unavailable')
    }

  } catch (emergencyError) {
    console.error('❌ [PerfectSearch] Emergency fallback failed:', emergencyError)
    showUserFriendlyError(result, 'Navigation system temporarily unavailable')
  }
}

// 🔧 新增：优雅降级处理 - 修复 router.push 错误
const performGracefulFallback = async (result, error = null) => {
  console.log('🔄 [PerfectSearch] Performing graceful fallback navigation')

  try {
    // 🔧 修复：直接使用setup函数内的router，避免异步import问题
    const safeRouter = getSafeRouter()

    if (safeRouter) {
      console.log('✅ [PerfectSearch] Using router for navigation')
      await safeRouter.push(`/chat/${result.chat_id}`)

      // 🔧 新增：等待页面加载并处理历史消息
      await waitForChatLoadAndHistoryMessages(result)

      emit('navigate', {
        messageId: result.id,
        chatId: result.chat_id,
        searchQuery: searchQuery.value,
        success: true,
        perfect: false,
        fallback: true,
        reason: error?.message || 'fallback'
      })
    } else {
      // 🔧 修复：避免window.location.href，使用事件通知代替
      console.warn('⚠️ [PerfectSearch] Router unavailable, dispatching navigation event')

      window.dispatchEvent(new CustomEvent('navigate-to-chat', {
        detail: {
          chatId: result.chat_id,
          messageId: result.id,
          searchQuery: searchQuery.value,
          source: 'search_fallback'
        }
      }))

      emit('navigate', {
        messageId: result.id,
        chatId: result.chat_id,
        searchQuery: searchQuery.value,
        success: true,
        perfect: false,
        fallback: 'event_dispatch',
        reason: error?.message || 'router_unavailable'
      })
    }

  } catch (fallbackError) {
    console.error('❌ [PerfectSearch] Graceful fallback failed:', fallbackError)
    showUserFriendlyError(result, 'Unable to open chat')
  }
}

// 🔧 新增：基于DAG分析的智能等待聊天加载和历史消息处理
const waitForChatLoadAndHistoryMessages = async (result) => {
  const maxWaitTime = 10000 // 增加到10秒最大等待时间
  const startTime = Date.now()

  console.log(`🔄 [PerfectSearch] 🎯 DAG-Enhanced: Starting intelligent message loading for ${result.id}`)

  try {
    // Step 1: 等待基础页面元素加载
    await waitForChatPageElements()

    // Step 2: 检查消息是否已经在DOM中
    let messageElement = document.querySelector(`[data-message-id="${result.id}"]`)

    if (messageElement) {
      console.log(`✅ [PerfectSearch] 🎯 DAG: Message ${result.id} already visible, scrolling`)
      scrollToMessageSafely(messageElement)
      return { success: true, method: 'already_loaded' }
    }

    // Step 3: 智能策略选择 - 基于DAG分析的多策略方法
    console.log(`🔄 [PerfectSearch] 🎯 DAG: Message not visible, starting intelligent loading strategies`)

    const strategies = [
      { name: 'UnifiedMessageService', execute: () => loadViaUnifiedMessageService(result) },
      { name: 'ScrollTrigger', execute: () => loadViaScrollTrigger(result) },
      { name: 'DirectAPI', execute: () => loadViaDirectAPI(result) },
      { name: 'EventSystem', execute: () => loadViaEventSystem(result) }
    ]

    for (const [index, strategy] of strategies.entries()) {
      if ((Date.now() - startTime) > maxWaitTime) {
        console.warn(`⏰ [PerfectSearch] 🎯 DAG: Timeout reached, stopping strategy execution`)
        break
      }

      console.log(`📋 [PerfectSearch] 🎯 DAG: Trying strategy ${index + 1}/${strategies.length}: ${strategy.name}`)

      const strategyResult = await strategy.execute()

      if (strategyResult.success) {
        // 等待DOM更新并验证
        await waitForDOMStabilization()

        messageElement = document.querySelector(`[data-message-id="${result.id}"]`)
        if (messageElement) {
          console.log(`✅ [PerfectSearch] 🎯 DAG: Strategy ${strategy.name} succeeded`)
          scrollToMessageSafely(messageElement)
          return { success: true, method: strategy.name }
        }
      }

      // 策略间等待时间，让DOM有时间更新
      await new Promise(resolve => setTimeout(resolve, 500))
    }

    console.warn(`⚠️ [PerfectSearch] 🎯 DAG: All strategies failed for message ${result.id}`)
    return { success: false, error: 'All loading strategies failed' }

  } catch (error) {
    console.error(`❌ [PerfectSearch] 🎯 DAG: Wait for chat load failed:`, error)
    return { success: false, error: error.message }
  }
}

// 🔧 新增：通过UnifiedMessageService加载策略
const loadViaUnifiedMessageService = async (result) => {
  try {
    const unifiedMessageService = window.unifiedMessageService
    if (!unifiedMessageService) {
      return { success: false, error: 'UnifiedMessageService not available' }
    }

    console.log(`📦 [PerfectSearch] 🎯 DAG-UMS: Loading via UnifiedMessageService for chat ${result.chat_id}`)

    // 检查是否还有更多消息可以加载
    const hasMore = unifiedMessageService.hasMoreMessages(result.chat_id)
    if (!hasMore) {
      console.log(`📦 [PerfectSearch] 🎯 DAG-UMS: No more messages to load`)
      return { success: false, error: 'No more messages available' }
    }

    // 尝试加载更多消息
    const currentMessages = unifiedMessageService.getMessagesForChat(result.chat_id) || []
    const beforeCount = currentMessages.length

    console.log(`📦 [PerfectSearch] 🎯 DAG-UMS: Current message count: ${beforeCount}`)

    // 使用chatStore的fetchMoreMessages方法
    const chatStore = window.chatStore || (await import('@/stores/chat')).useChatStore()
    if (chatStore && typeof chatStore.fetchMoreMessages === 'function') {
      await chatStore.fetchMoreMessages(result.chat_id, 20) // 加载20条消息

      const afterMessages = unifiedMessageService.getMessagesForChat(result.chat_id) || []
      const afterCount = afterMessages.length

      console.log(`📦 [PerfectSearch] 🎯 DAG-UMS: After loading: ${afterCount} messages (+${afterCount - beforeCount})`)

      return {
        success: afterCount > beforeCount,
        loaded: afterCount - beforeCount,
        method: 'UnifiedMessageService'
      }
    }

    return { success: false, error: 'ChatStore fetchMoreMessages not available' }

  } catch (error) {
    console.error(`❌ [PerfectSearch] 🎯 DAG-UMS: UnifiedMessageService loading failed:`, error)
    return { success: false, error: error.message }
  }
}

// 🔧 新增：通过滚动触发加载策略
const loadViaScrollTrigger = async (result) => {
  try {
    console.log(`📜 [PerfectSearch] 🎯 DAG-Scroll: Triggering load via scroll for chat ${result.chat_id}`)

    const scrollContainer = document.querySelector('.simple-message-list, .messages-container, .message-list')
    if (!scrollContainer) {
      return { success: false, error: 'Scroll container not found' }
    }

    // 保存当前滚动位置
    const originalScrollTop = scrollContainer.scrollTop
    const originalScrollHeight = scrollContainer.scrollHeight

    // 滚动到顶部触发加载
    scrollContainer.scrollTop = 0

    // 等待加载触发
    await new Promise(resolve => setTimeout(resolve, 500))

    // 检查是否有新内容加载
    let attempts = 0
    const maxScrollAttempts = 5

    while (attempts < maxScrollAttempts) {
      await new Promise(resolve => setTimeout(resolve, 800))

      const newScrollHeight = scrollContainer.scrollHeight
      if (newScrollHeight > originalScrollHeight) {
        console.log(`📜 [PerfectSearch] 🎯 DAG-Scroll: New content loaded, height: ${originalScrollHeight} → ${newScrollHeight}`)

        // 恢复滚动位置（调整高度差）
        const heightDifference = newScrollHeight - originalScrollHeight
        scrollContainer.scrollTop = originalScrollTop + heightDifference

        return { success: true, method: 'ScrollTrigger', heightDifference }
      }

      attempts++
    }

    // 恢复原始滚动位置
    scrollContainer.scrollTop = originalScrollTop

    return { success: false, error: 'No new content loaded via scroll' }

  } catch (error) {
    console.error(`❌ [PerfectSearch] 🎯 DAG-Scroll: Scroll trigger loading failed:`, error)
    return { success: false, error: error.message }
  }
}

// 🔧 新增：通过直接API加载策略
const loadViaDirectAPI = async (result) => {
  try {
    console.log(`🌐 [PerfectSearch] 🎯 DAG-API: Loading via direct API for message ${result.id}`)

    const api = (await import('@/services/api')).default

    // 尝试获取消息上下文
    const response = await api.get(`/chat/${result.chat_id}/messages`, {
      params: {
        around_message_id: result.id,
        limit: 50 // 获取目标消息周围的50条消息
      }
    })

    if (response.data && response.data.length > 0) {
      console.log(`🌐 [PerfectSearch] 🎯 DAG-API: Loaded ${response.data.length} messages around target`)

      // 触发消息显示更新（如果有消息服务的话）
      const unifiedMessageService = window.unifiedMessageService
      if (unifiedMessageService && typeof unifiedMessageService.processMessages === 'function') {
        unifiedMessageService.processMessages(result.chat_id, response.data)
      }

      return { success: true, method: 'DirectAPI', count: response.data.length }
    }

    return { success: false, error: 'No messages returned from API' }

  } catch (error) {
    console.error(`❌ [PerfectSearch] 🎯 DAG-API: Direct API loading failed:`, error)
    return { success: false, error: error.message }
  }
}

// 🔧 新增：通过事件系统加载策略
const loadViaEventSystem = async (result) => {
  try {
    console.log(`📡 [PerfectSearch] 🎯 DAG-Event: Triggering load via event system for message ${result.id}`)

    // 发送加载更多消息事件
    window.dispatchEvent(new CustomEvent('load-more-messages', {
      detail: {
        chatId: result.chat_id,
        messageId: result.id,
        source: 'search_navigation',
        limit: 30
      }
    }))

    // 也尝试点击加载更多按钮
    const loadMoreButton = document.querySelector('.load-more-button, [data-load-more], .load-earlier-messages')
    if (loadMoreButton && !loadMoreButton.disabled) {
      console.log(`📡 [PerfectSearch] 🎯 DAG-Event: Clicking load more button`)
      loadMoreButton.click()
    }

    // 等待事件处理
    await new Promise(resolve => setTimeout(resolve, 1000))

    return { success: true, method: 'EventSystem' }

  } catch (error) {
    console.error(`❌ [PerfectSearch] 🎯 DAG-Event: Event system loading failed:`, error)
    return { success: false, error: error.message }
  }
}

// 🔧 新增：DOM稳定化等待
const waitForDOMStabilization = async () => {
  console.log(`🔄 [PerfectSearch] 🎯 DAG-DOM: Waiting for DOM stabilization`)

  // Step 1: Vue nextTick
  await nextTick()

  // Step 2: 浏览器渲染帧
  await new Promise(resolve => requestAnimationFrame(resolve))

  // Step 3: 额外等待时间让DOM完全稳定
  await new Promise(resolve => setTimeout(resolve, 300))

  console.log(`✅ [PerfectSearch] 🎯 DAG-DOM: DOM stabilization complete`)
}

// 🔧 新增：触发加载更多消息
const triggerLoadMoreMessages = async () => {
  try {
    // 方法1: 触发滚动到顶部来加载历史消息
    const scrollContainer = document.querySelector('.simple-message-list, .messages-container, .message-list')
    if (scrollContainer) {
      // 保存当前滚动位置
      const currentScrollTop = scrollContainer.scrollTop

      // 滚动到顶部触发加载
      scrollContainer.scrollTop = 0

      // 等待加载触发
      await new Promise(resolve => setTimeout(resolve, 200))

      return
    }

    // 方法2: 查找并点击"加载更多"按钮
    const loadMoreButton = document.querySelector('.load-more-button, [data-load-more], .load-earlier-messages')
    if (loadMoreButton && !loadMoreButton.disabled) {
      loadMoreButton.click()
      await new Promise(resolve => setTimeout(resolve, 300))
      return
    }

    // 方法3: 通过事件触发加载
    window.dispatchEvent(new CustomEvent('load-more-messages', {
      detail: { source: 'search_navigation' }
    }))

    await new Promise(resolve => setTimeout(resolve, 200))

  } catch (error) {
    console.warn(`⚠️ [PerfectSearch] Failed to trigger load more:`, error.message)
  }
}

// 🔧 新增：等待加载完成
const waitForLoadingComplete = async () => {
  const maxWait = 2000
  const startTime = Date.now()

  while ((Date.now() - startTime) < maxWait) {
    // 检查加载指示器
    const isLoading = document.querySelector('.loading-indicator, .message-loading, .spinner')

    if (!isLoading) {
      // 额外等待DOM更新
      await new Promise(resolve => setTimeout(resolve, 300))
      return
    }

    await new Promise(resolve => setTimeout(resolve, 100))
  }

  console.log(`⏰ [PerfectSearch] Loading wait timeout, proceeding`)
}

// 🔧 新增：安全滚动到消息
const scrollToMessageSafely = (messageElement) => {
  try {
    console.log(`🎯 [PerfectSearch] 🎯 DAG-Scroll: Starting safe scroll to message`)

    // 清除之前的高亮
    document.querySelectorAll('.message-highlight, .search-target, .blue-pulse-beam-highlight').forEach(el => {
      el.classList.remove('message-highlight', 'search-target', 'blue-pulse-beam-highlight',
        'blue-beam-fast', 'blue-beam-intense', 'navigation-target')
      // 清除蓝色光束相关元素
      const scanner = el.querySelector('.blue-beam-scanner')
      const indicator = el.querySelector('.blue-beam-indicator')
      if (scanner) scanner.remove()
      if (indicator) indicator.remove()
    })

    // 🔵 使用蓝色脉冲光束高亮 - 替代传统高亮
    console.log(`🔵 [BlueBeam] 🚀 Applying blue pulse beam effect to message ${messageElement.dataset.messageId}`)

    // 🔵 简化：仅添加边框转动蓝色光束
    messageElement.classList.add('blue-pulse-beam-highlight', 'blue-beam-fast', 'blue-beam-intense')

    // 计算最佳滚动位置
    const rect = messageElement.getBoundingClientRect()
    const containerRect = messageElement.closest('.simple-message-list, .messages-container')?.getBoundingClientRect()

    let scrollOptions = {
      behavior: 'smooth',
      block: 'center',
      inline: 'nearest'
    }

    // 如果消息已经基本可见，使用更温和的滚动
    if (containerRect && rect.top >= containerRect.top && rect.bottom <= containerRect.bottom) {
      scrollOptions.block = 'nearest'
    }

    // 执行滚动
    messageElement.scrollIntoView(scrollOptions)

    // 添加滚动完成监听
    const scrollContainer = messageElement.closest('.simple-message-list, .messages-container')
    if (scrollContainer) {
      const onScrollEnd = () => {
        console.log(`✅ [PerfectSearch] 🎯 DAG-Scroll: Scroll completed, message highlighted with blue pulse beam`)
        scrollContainer.removeEventListener('scrollend', onScrollEnd)
      }

      scrollContainer.addEventListener('scrollend', onScrollEnd)

      // 备用计时器（如果scrollend不支持）
      setTimeout(onScrollEnd, 1000)
    }

    // 🔵 蓝色光束清理 - 延迟移除高亮
    setTimeout(() => {
      // 移除蓝色光束类
      messageElement.classList.remove('blue-pulse-beam-highlight', 'blue-beam-fast', 'blue-beam-intense')

      console.log(`🔵 [BlueBeam] 🚀 Simplified border rotation removed from message ${messageElement.dataset.messageId}`)
      console.log(`🔄 [PerfectSearch] 🎯 DAG-Scroll: Blue pulse beam highlight removed after 4 seconds`)
    }, 4000)

    return { success: true, scrolled: true, effect: 'blue_pulse_beam' }

  } catch (error) {
    console.error(`❌ [PerfectSearch] 🎯 DAG-Scroll: Scroll to message failed:`, error)
    return { success: false, error: error.message }
  }
}

// 🔧 增强：用户友好错误显示 - 修复 actions 数组问题
const showUserFriendlyError = (result, message) => {
  // 🔧 安全的 actions 数组创建
  const actions = []

  try {
    actions.push({
      label: 'Try Again',
      action: () => jumpToMessage(result)
    })

    actions.push({
      label: 'Open Chat',
      action: async () => {
        try {
          // 🔧 修复：使用setup函数内的router，避免页面刷新
          const safeRouter = getSafeRouter()
          if (safeRouter) {
            console.log('🔄 [UserFriendlyError] Using router for navigation')
            await safeRouter.push(`/chat/${result.chat_id}`)
          } else {
            // 🔧 修复：避免window.location.href，使用事件通知
            console.warn('⚠️ [UserFriendlyError] Router unavailable, dispatching event')
            window.dispatchEvent(new CustomEvent('navigate-to-chat', {
              detail: {
                chatId: result.chat_id,
                source: 'error_recovery'
              }
            }))
          }
        } catch (error) {
          console.error('Action navigation failed:', error)
          // 最后备用方案：显示更友好的错误
          window.dispatchEvent(new CustomEvent('show-notification', {
            detail: {
              type: 'error',
              title: 'Navigation Failed',
              message: 'Unable to navigate to chat. Please try refreshing the page.',
              duration: 5000
            }
          }))
        }
      }
    })
  } catch (actionError) {
    console.warn('⚠️ [PerfectSearch] Failed to create recovery actions:', actionError)
  }

  // 创建用户友好的通知
  const notification = {
    type: 'warning',
    title: 'Navigation Issue',
    message: message,
    duration: 5000,
    actions: actions
  }

  // 安全的事件分发
  try {
    window.dispatchEvent(new CustomEvent('show-notification', {
      detail: notification
    }))
  } catch (eventError) {
    console.warn('⚠️ [PerfectSearch] Failed to dispatch notification:', eventError)
  }

  console.error(`🚨 [PerfectSearch] User error: ${message}`, result)

  emit('navigate', {
    messageId: result.id,
    chatId: result.chat_id,
    success: false,
    error: message,
    userFriendly: true,
    recoveryOptions: actions
  })
}

// 🔧 修复：直接使用setup函数内的router实例，避免页面刷新
const getSafeRouter = () => {
  // 直接返回setup函数内的router实例
  if (router && typeof router.push === 'function') {
    return router
  }

  // 备用方案：从全局实例获取
  const fallbackRouter = window.$router || window.app?.$router
  if (fallbackRouter && typeof fallbackRouter.push === 'function') {
    return fallbackRouter
  }

  console.warn('⚠️ [PerfectSearch] No router instance available')
  return null
}

const getInitials = (name) => {
  return name?.split(' ').map(n => n[0]).join('').toUpperCase() || '?'
}

// 🎨 Generate beautiful avatar colors based on username
const getAvatarStyle = (username) => {
  const colors = [
    { bg: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)', color: '#ffffff' },
    { bg: 'linear-gradient(135deg, #f093fb 0%, #f5576c 100%)', color: '#ffffff' },
    { bg: 'linear-gradient(135deg, #4facfe 0%, #00f2fe 100%)', color: '#ffffff' },
    { bg: 'linear-gradient(135deg, #43e97b 0%, #38f9d7 100%)', color: '#ffffff' },
    { bg: 'linear-gradient(135deg, #fa709a 0%, #fee140 100%)', color: '#ffffff' },
    { bg: 'linear-gradient(135deg, #a8edea 0%, #fed6e3 100%)', color: '#2d3748' },
    { bg: 'linear-gradient(135deg, #ffecd2 0%, #fcb69f 100%)', color: '#2d3748' },
    { bg: 'linear-gradient(135deg, #ff8a80 0%, #ffcc02 100%)', color: '#ffffff' },
    { bg: 'linear-gradient(135deg, #8fd3f4 0%, #84fab0 100%)', color: '#2d3748' },
    { bg: 'linear-gradient(135deg, #d299c2 0%, #fef9d7 100%)', color: '#2d3748' },
    { bg: 'linear-gradient(135deg, #89f7fe 0%, #66a6ff 100%)', color: '#ffffff' },
    { bg: 'linear-gradient(135deg, #fdbb2d 0%, #22c1c3 100%)', color: '#ffffff' },
    { bg: 'linear-gradient(135deg, #e0c3fc 0%, #9bb5ff 100%)', color: '#2d3748' },
    { bg: 'linear-gradient(135deg, #ffeaa7 0%, #fab1a0 100%)', color: '#2d3748' },
    { bg: 'linear-gradient(135deg, #74b9ff 0%, #e17055 100%)', color: '#ffffff' }
  ]

  // Generate deterministic color based on username hash
  let hash = 0
  for (let i = 0; i < username.length; i++) {
    const char = username.charCodeAt(i)
    hash = ((hash << 5) - hash) + char
    hash = hash & hash // Convert to 32-bit integer
  }

  const colorIndex = Math.abs(hash) % colors.length
  const selectedColor = colors[colorIndex]

  return {
    background: selectedColor.bg,
    color: selectedColor.color,
    boxShadow: '0 4px 12px rgba(0, 0, 0, 0.15), inset 0 1px 0 rgba(255, 255, 255, 0.2)',
    border: '2px solid rgba(255, 255, 255, 0.3)'
  }
}

const formatTime = (timestamp) => {
  const date = new Date(timestamp)
  const now = new Date()
  const diffInHours = (now - date) / (1000 * 60 * 60)

  if (diffInHours < 24) {
    return date.toLocaleTimeString('zh-CN', {
      hour: '2-digit',
      minute: '2-digit',
      hour12: false
    })
  } else if (diffInHours < 168) { // 7 days
    return date.toLocaleDateString('zh-CN', {
      weekday: 'short',
      hour: '2-digit',
      minute: '2-digit',
      hour12: false
    })
  } else {
    return date.toLocaleDateString('zh-CN', {
      month: 'short',
      day: 'numeric',
      year: 'numeric'
    })
  }
}

const highlightText = (text, query) => {
  if (!query) return text

  const escapedQuery = query.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
  const regex = new RegExp(`(${escapedQuery})`, 'gi')
  return text.replace(regex, '<mark>$1</mark>')
}

// 🏷️ Get user-friendly label for search source
const getSearchSourceLabel = (source) => {
  const sourceLabels = {
    'api': '🌐 Live Search',
    'local_no_auth': '💾 Local Cache',
    'local_auth_fallback': '💾 Offline Mode',
    'local_error_fallback': '💾 Fallback Mode',
    'local': '💾 Local Search',
    'local_empty': '💾 Local (No Data)',
    'local_error': '⚠️ Search Error'
  }

  return sourceLabels[source] || '🔍 Search'
}

// 移除已删除的 perfectSearchService 相关代码

// Watchers
watch(() => props.isOpen, (isOpen) => {
  if (isOpen) {
    nextTick(() => {
      searchInput.value?.focus()
    })
  } else {
    // Reset state
    searchQuery.value = ''
    searchResults.value = []
    hasSearched.value = false
    activeFilters.value = []
    searchSource.value = null
    searchError.value = null
    showAdvancedStrategies.value = false
  }
})

// Lifecycle
onMounted(() => {
  // 🔧 DEBUG: Confirm PerfectSearchModal is loading
  console.log('🎯 PerfectSearchModal mounted successfully!')
  console.log('🎯 Props received:', { isOpen: props.isOpen, chatId: props.chatId })

  // Focus search input when modal opens
  if (props.isOpen) {
    nextTick(() => {
      searchInput.value?.focus()
    })
  }
})

// 🔧 新增：等待聊天页面基础元素 - DAG增强版
const waitForChatPageElements = async () => {
  const maxWait = 5000 // 增加到5秒
  const startTime = Date.now()

  console.log(`🔄 [PerfectSearch] 🎯 DAG-Elements: Waiting for chat page elements`)

  while ((Date.now() - startTime) < maxWait) {
    const chatContainer = document.querySelector('.chat-container, .simple-message-list, .messages-container')
    const messageElements = document.querySelectorAll('[data-message-id]')

    if (chatContainer && messageElements.length > 0) {
      console.log(`✅ [PerfectSearch] 🎯 DAG-Elements: Chat page loaded with ${messageElements.length} messages`)
      return { success: true, elements: messageElements.length }
    }

    await new Promise(resolve => setTimeout(resolve, 100))
  }

  console.warn(`⚠️ [PerfectSearch] 🎯 DAG-Elements: Chat page elements loading timeout`)
  return { success: false, error: 'Chat page elements loading timeout' }
}

// 🔧 新增：DAG诊断工具
const runDAGDiagnostics = async (result) => {
  const diagnostics = {
    timestamp: new Date().toISOString(),
    messageId: result.id,
    chatId: result.chat_id,
    steps: {}
  }

  console.log(`🔍 [PerfectSearch] 🎯 DAG-Diagnostics: Running full system diagnostics`)

  try {
    // 1. 检查路由系统
    const router = getSafeRouter()
    diagnostics.steps.router = {
      available: !!router,
      type: router ? 'vue-router' : 'none',
      canPush: router && typeof router.push === 'function'
    }

    // 2. 检查UnifiedMessageService
    const unifiedMessageService = window.unifiedMessageService
    diagnostics.steps.messageService = {
      available: !!unifiedMessageService,
      hasGetMessages: unifiedMessageService && typeof unifiedMessageService.getMessagesForChat === 'function',
      hasMoreMessages: unifiedMessageService && typeof unifiedMessageService.hasMoreMessages === 'function',
      messageCount: unifiedMessageService ? (unifiedMessageService.getMessagesForChat(result.chat_id) || []).length : 0
    }

    // 3. 检查DOM状态
    const chatContainer = document.querySelector('.chat-container, .simple-message-list, .messages-container')
    const messageElements = document.querySelectorAll('[data-message-id]')
    const targetElement = document.querySelector(`[data-message-id="${result.id}"]`)

    diagnostics.steps.dom = {
      chatContainer: !!chatContainer,
      messageCount: messageElements.length,
      targetFound: !!targetElement,
      containerType: chatContainer?.className || 'none'
    }

    // 4. 检查API可用性
    try {
      const api = (await import('@/services/api')).default
      diagnostics.steps.api = {
        available: !!api,
        type: typeof api
      }
    } catch (error) {
      diagnostics.steps.api = {
        available: false,
        error: error.message
      }
    }

    // 5. 检查chatStore
    try {
      const chatStore = window.chatStore || (await import('@/stores/chat')).useChatStore()
      diagnostics.steps.chatStore = {
        available: !!chatStore,
        hasFetchMore: chatStore && typeof chatStore.fetchMoreMessages === 'function',
        currentChatId: chatStore?.currentChatId
      }
    } catch (error) {
      diagnostics.steps.chatStore = {
        available: false,
        error: error.message
      }
    }

    console.log(`🔍 [PerfectSearch] 🎯 DAG-Diagnostics: Complete diagnostics:`, diagnostics)

    return diagnostics

  } catch (error) {
    console.error(`❌ [PerfectSearch] 🎯 DAG-Diagnostics: Diagnostics failed:`, error)
    diagnostics.error = error.message
    return diagnostics
  }
}
</script>

<style scoped>
/* 🎨 PERFECT SEARCH MODAL - JOBS MASTERPIECE EDITION
   ✨ The epitome of digital craftsmanship and aesthetic perfection */

/* Modal Backdrop - Cinematic Experience */
.search-backdrop {
  position: fixed;
  inset: 0;
  background:
    radial-gradient(ellipse at center, rgba(0, 122, 255, 0.12) 0%, rgba(0, 0, 0, 0.45) 70%),
    linear-gradient(135deg, rgba(0, 0, 0, 0.3) 0%, rgba(0, 0, 0, 0.55) 100%);
  backdrop-filter: blur(24px) saturate(200%) brightness(1.1);
  display: flex;
  align-items: flex-start;
  justify-content: center;
  padding: 32px;
  z-index: 1000;
  overflow-y: auto;
  animation: backdropFadeIn 400ms cubic-bezier(0.23, 1, 0.32, 1);
}

@keyframes backdropFadeIn {
  from {
    opacity: 0;
    backdrop-filter: blur(0px) saturate(100%) brightness(1);
  }

  to {
    opacity: 1;
    backdrop-filter: blur(24px) saturate(200%) brightness(1.1);
  }
}

/* Modal Container - Modern Design */
.search-modal {
  background:
    linear-gradient(145deg, rgba(255, 255, 255, 0.98) 0%, rgba(248, 250, 252, 0.95) 100%);
  backdrop-filter: blur(20px) saturate(140%);
  border-radius: 16px;
  box-shadow:
    0 20px 40px rgba(0, 0, 0, 0.15),
    0 8px 16px rgba(0, 0, 0, 0.1),
    0 4px 8px rgba(0, 0, 0, 0.05),
    inset 0 1px 0 rgba(255, 255, 255, 0.9);
  width: 100%;
  max-width: 640px;
  max-height: calc(100vh - 80px);
  margin-top: 10vh;
  overflow: hidden;
  border: 1px solid rgba(0, 0, 0, 0.08);

  display: flex;
  flex-direction: column;
  position: relative;
  animation: modalSlideIn 400ms cubic-bezier(0.25, 0.8, 0.25, 1);
  transform-origin: center top;
}

@keyframes modalSlideIn {
  from {
    opacity: 0;
    transform: scale(0.94) translateY(-20px);
    filter: blur(4px);
  }

  to {
    opacity: 1;
    transform: scale(1) translateY(0);
    filter: blur(0px);
  }
}

/* ✨ Modern Clean Header */
.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 20px 24px;
  background: rgba(255, 255, 255, 0.95);
  border-bottom: 1px solid rgba(0, 0, 0, 0.08);
  backdrop-filter: blur(10px);
}

.search-branding {
  display: flex;
  align-items: center;
  gap: 12px;
}

.search-icon-elegant {
  width: 20px;
  height: 20px;
  color: #007aff;
  filter: drop-shadow(0 1px 2px rgba(0, 122, 255, 0.3));
}

.search-title {
  font-size: 16px;
  font-weight: 600;
  color: #1f2937;
  margin: 0;
  letter-spacing: -0.01em;
  line-height: 1.3;
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  font-family: -apple-system, BlinkMacSystemFont, 'SF Pro Text', system-ui, sans-serif;
}



.close-button {
  width: 32px;
  height: 32px;
  border: none;
  background: rgba(0, 0, 0, 0.05);
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #6b7280;
  cursor: pointer;
  transition: all 200ms ease;
}

.close-button:hover {
  background: rgba(0, 0, 0, 0.1);
  color: #374151;
  transform: scale(1.1);
}

.close-button svg {
  width: 16px;
  height: 16px;
}

/* 🔍 Modern Search Section */
.search-section {
  padding: 0 24px 24px;
  background: rgba(248, 250, 252, 0.3);
}

.search-input-container {
  position: relative;
  display: flex;
  align-items: center;
  background: rgba(255, 255, 255, 0.9);
  border: 1px solid rgba(0, 0, 0, 0.1);
  border-radius: 12px;
  transition: all 250ms cubic-bezier(0.25, 0.8, 0.25, 1);
  box-shadow:
    0 2px 8px rgba(0, 0, 0, 0.04),
    inset 0 1px 0 rgba(255, 255, 255, 0.8);
  overflow: hidden;
}

.search-input-container:focus-within {
  background: rgba(255, 255, 255, 0.98);
  border-color: #007aff;
  box-shadow:
    0 0 0 3px rgba(0, 122, 255, 0.1),
    0 4px 16px rgba(0, 122, 255, 0.12),
    inset 0 1px 0 rgba(255, 255, 255, 0.9);
  transform: translateY(-1px);
}

/* Search Icon - Left Side */
.search-icon-wrapper {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0 16px;
  flex-shrink: 0;
}

.search-icon {
  width: 20px;
  height: 20px;
  color: #6b7280;
  transition: color 250ms ease;
}

.search-input-container:focus-within .search-icon {
  color: #007aff;
}

/* Main Search Input */
.search-input {
  flex: 1;
  height: 48px;
  padding: 0;
  border: none;
  background: transparent;
  font-size: 16px;
  font-weight: 400;
  color: #1f2937;
  outline: none;
  font-family: -apple-system, BlinkMacSystemFont, 'SF Pro Text', system-ui, sans-serif;
  letter-spacing: -0.01em;
}

.search-input::placeholder {
  color: #9ca3af;
  font-weight: 400;
}

.search-input:focus::placeholder {
  color: #d1d5db;
}

.search-input:disabled {
  opacity: 0.7;
  cursor: not-allowed;
}

/* Search Actions - Right Side */
.search-actions {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 0 12px;
  flex-shrink: 0;
}

/* Clear Button */
.clear-button {
  width: 24px;
  height: 24px;
  border: none;
  background: none;
  color: #9ca3af;
  border-radius: 6px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: all 200ms ease;
}

.clear-button:hover {
  color: #6b7280;
  background: rgba(0, 0, 0, 0.05);
  transform: scale(1.1);
}

.clear-button svg {
  width: 14px;
  height: 14px;
}

/* Search Button - Always Visible with Magnifying Glass */
.search-button {
  width: 32px;
  height: 32px;
  border: none;
  background: #f3f4f6;
  color: #6b7280;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: all 250ms cubic-bezier(0.25, 0.8, 0.25, 1);
  box-shadow:
    0 1px 3px rgba(0, 0, 0, 0.1),
    inset 0 1px 0 rgba(255, 255, 255, 0.8);
}

.search-button:hover:not(:disabled) {
  background: #e5e7eb;
  color: #374151;
  transform: translateY(-1px);
  box-shadow:
    0 2px 6px rgba(0, 0, 0, 0.15),
    inset 0 1px 0 rgba(255, 255, 255, 0.9);
}

.search-button.active {
  background: linear-gradient(135deg, #007aff 0%, #0056cc 100%);
  color: #ffffff;
  box-shadow:
    0 2px 8px rgba(0, 122, 255, 0.3),
    inset 0 1px 0 rgba(255, 255, 255, 0.2);
}

.search-button.active:hover:not(:disabled) {
  background: linear-gradient(135deg, #0056cc 0%, #003d99 100%);
  transform: translateY(-1px) scale(1.02);
  box-shadow:
    0 4px 12px rgba(0, 122, 255, 0.4),
    inset 0 1px 0 rgba(255, 255, 255, 0.3);
}

.search-button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
  transform: none;
}

.search-button svg {
  width: 16px;
  height: 16px;
}

/* 🔄 Modern Loading Animation */
.loading-indicator {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
}

.loading-spinner {
  display: flex;
  align-items: center;
  justify-content: center;
}

.spinner {
  width: 20px;
  height: 20px;
  color: #007aff;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  0% {
    transform: rotate(0deg);
  }

  100% {
    transform: rotate(360deg);
  }
}

/* 🔍 Advanced Search Strategies Section */
.strategies-section {
  margin-top: 20px;
  padding: 0 4px;
}

.strategies-header {
  margin-bottom: 12px;
}

.strategies-toggle {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 16px;
  border: 1px solid rgba(0, 0, 0, 0.08);
  background:
    linear-gradient(145deg, rgba(255, 255, 255, 0.95) 0%, rgba(248, 250, 252, 0.85) 100%);
  border-radius: 12px;
  font-size: 13px;
  font-weight: 500;
  color: #4b5563;
  cursor: pointer;
  transition: all 250ms cubic-bezier(0.23, 1, 0.32, 1);
  box-shadow:
    0 2px 6px rgba(0, 0, 0, 0.04),
    inset 0 1px 0 rgba(255, 255, 255, 0.8);
  width: fit-content;
}

.strategies-toggle:hover {
  border-color: rgba(0, 122, 255, 0.2);
  color: #007aff;
  background:
    linear-gradient(145deg, rgba(255, 255, 255, 0.98) 0%, rgba(240, 249, 255, 0.9) 100%);
  transform: translateY(-1px);
  box-shadow:
    0 4px 10px rgba(0, 122, 255, 0.08),
    inset 0 1px 0 rgba(255, 255, 255, 0.9);
}

.toggle-icon {
  font-size: 14px;
}

.chevron-icon {
  width: 16px;
  height: 16px;
  transition: transform 250ms cubic-bezier(0.23, 1, 0.32, 1);
}

.chevron-icon.rotated {
  transform: rotate(180deg);
}

.strategies-container {
  overflow: hidden;
}

.strategy-pills {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
  padding: 4px 0;
}

.strategy-pill {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 14px;
  border: 1px solid rgba(0, 0, 0, 0.06);
  background:
    linear-gradient(145deg, rgba(255, 255, 255, 0.9) 0%, rgba(248, 250, 252, 0.8) 100%);
  border-radius: 20px;
  font-size: 12px;
  font-weight: 500;
  color: #4b5563;
  cursor: pointer;
  transition: all 250ms cubic-bezier(0.23, 1, 0.32, 1);
  box-shadow:
    0 2px 6px rgba(0, 0, 0, 0.03),
    inset 0 1px 0 rgba(255, 255, 255, 0.7);
  position: relative;
  overflow: hidden;
}

.strategy-pill::before {
  content: '';
  position: absolute;
  inset: 0;
  background: linear-gradient(135deg, rgba(0, 122, 255, 0.1) 0%, rgba(88, 86, 214, 0.08) 100%);
  opacity: 0;
  transition: opacity 250ms ease;
}

.strategy-pill:hover {
  border-color: rgba(0, 122, 255, 0.25);
  color: #007aff;
  background:
    linear-gradient(145deg, rgba(255, 255, 255, 0.95) 0%, rgba(240, 249, 255, 0.9) 100%);
  transform: translateY(-1px);
  box-shadow:
    0 4px 10px rgba(0, 122, 255, 0.1),
    inset 0 1px 0 rgba(255, 255, 255, 0.8);
}

.strategy-pill:hover::before {
  opacity: 1;
}

.strategy-pill.active {
  background:
    linear-gradient(135deg, #007aff 0%, #5856d6 100%);
  border-color: transparent;
  color: #ffffff;
  box-shadow:
    0 4px 14px rgba(0, 122, 255, 0.3),
    inset 0 1px 0 rgba(255, 255, 255, 0.25);
  transform: translateY(-1px);
}

.strategy-pill.active::before {
  background: linear-gradient(135deg, rgba(255, 255, 255, 0.1) 0%, rgba(255, 255, 255, 0.05) 100%);
  opacity: 1;
}

.strategy-icon {
  font-size: 12px;
  filter: drop-shadow(0 1px 2px rgba(0, 0, 0, 0.1));
}

.strategy-label {
  font-weight: 500;
  white-space: nowrap;
}

/* Smooth slide animation for strategies */
.strategies-slide-enter-active,
.strategies-slide-leave-active {
  transition: all 300ms cubic-bezier(0.23, 1, 0.32, 1);
  transform-origin: top;
}

.strategies-slide-enter-from,
.strategies-slide-leave-to {
  opacity: 0;
  transform: scaleY(0) translateY(-10px);
  max-height: 0;
}

.strategies-slide-enter-to,
.strategies-slide-leave-from {
  opacity: 1;
  transform: scaleY(1) translateY(0);
  max-height: 200px;
}

/* Refined Results Section - Ergonomic Design */
.results-section {
  flex: 1;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  background:
    linear-gradient(180deg, rgba(255, 255, 255, 0.95) 0%, rgba(248, 250, 252, 0.8) 100%);
}

.results-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 24px 32px 16px;
  border-top: 1px solid rgba(0, 0, 0, 0.06);
  border-bottom: 1px solid rgba(0, 0, 0, 0.04);
  background:
    linear-gradient(180deg, rgba(255, 255, 255, 0.98) 0%, rgba(248, 250, 252, 0.85) 100%);
  position: sticky;
  top: 0;
  z-index: 10;
  backdrop-filter: blur(20px) saturate(180%);
}

.results-info {
  display: flex;
  align-items: center;
  gap: 12px;
}

.results-count {
  font-size: 15px;
  font-weight: 650;
  color: #111827;
  letter-spacing: -0.02em;
  display: flex;
  align-items: center;
  gap: 8px;
}

.results-count::before {
  content: '🔍';
  font-size: 14px;
}

.search-source {
  font-size: 12px;
  font-weight: 500;
  color: #6b7280;
  background: rgba(0, 122, 255, 0.08);
  padding: 3px 8px;
  border-radius: 12px;
  border: 1px solid rgba(0, 122, 255, 0.12);
  display: flex;
  align-items: center;
  gap: 4px;
  opacity: 0.9;
  font-family: -apple-system, BlinkMacSystemFont, 'SF Pro Text', system-ui, sans-serif;
}

.search-time {
  font-size: 13px;
  color: #6b7280;
  font-weight: 500;
  background: rgba(0, 122, 255, 0.08);
  padding: 4px 10px;
  border-radius: 12px;
  border: 1px solid rgba(0, 122, 255, 0.12);
}

/* Premium Results Container with Custom Scrollbar */
.results-container {
  flex: 1;
  overflow-y: auto;
  padding: 16px 24px 32px;
  scroll-behavior: smooth;

  /* Custom Scrollbar Design */
  scrollbar-width: thin;
  scrollbar-color: rgba(0, 122, 255, 0.3) rgba(0, 0, 0, 0.05);
}

.results-container::-webkit-scrollbar {
  width: 8px;
}

.results-container::-webkit-scrollbar-track {
  background: rgba(0, 0, 0, 0.03);
  border-radius: 10px;
  margin: 8px 0;
}

.results-container::-webkit-scrollbar-thumb {
  background: linear-gradient(180deg, rgba(0, 122, 255, 0.4) 0%, rgba(88, 86, 214, 0.3) 100%);
  border-radius: 10px;
  border: 2px solid transparent;
  background-clip: padding-box;
  transition: all 250ms ease;
}

.results-container::-webkit-scrollbar-thumb:hover {
  background: linear-gradient(180deg, rgba(0, 122, 255, 0.6) 0%, rgba(88, 86, 214, 0.5) 100%);
  transform: scaleX(1.2);
}

.results-container::-webkit-scrollbar-corner {
  background: transparent;
}

/* Ergonomic Message Container Design */
.result-item {
  margin-bottom: 16px;
  cursor: pointer;
  transition: all 400ms cubic-bezier(0.23, 1, 0.32, 1);
  border-radius: 18px;
  background:
    linear-gradient(145deg, rgba(255, 255, 255, 0.95) 0%, rgba(248, 250, 252, 0.8) 100%);
  border: 1.5px solid rgba(0, 0, 0, 0.04);
  box-shadow:
    0 2px 12px rgba(0, 0, 0, 0.06),
    0 1px 4px rgba(0, 0, 0, 0.04),
    inset 0 1px 0 rgba(255, 255, 255, 0.9);
  position: relative;
  overflow: hidden;
  backdrop-filter: blur(20px) saturate(120%);
}

/* Message Accent Line */
.result-item::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 3px;
  background: linear-gradient(90deg,
      rgba(0, 122, 255, 0.0) 0%,
      rgba(0, 122, 255, 0.4) 30%,
      rgba(88, 86, 214, 0.5) 70%,
      rgba(88, 86, 214, 0.0) 100%);
  opacity: 0;
  transition: opacity 400ms ease;
}

/* Message Inner Container */
.result-message-container {
  padding: 20px 24px;
  display: flex;
  gap: 16px;
  align-items: flex-start;
}

/* 🎨 Colorful User Avatar */
.result-avatar {
  width: 44px;
  height: 44px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: 700;
  font-size: 16px;
  flex-shrink: 0;
  transition: all 300ms cubic-bezier(0.23, 1, 0.32, 1);
  font-family: -apple-system, BlinkMacSystemFont, 'SF Pro Display', system-ui, sans-serif;
  letter-spacing: -0.02em;
  /* Background and colors will be set dynamically via JavaScript */
}

/* Message Content Area */
.result-content-area {
  flex: 1;
  min-width: 0;
}

/* Message Header with Optimal Typography */
.result-header {
  display: flex;
  justify-content: space-between;
  align-items: baseline;
  margin-bottom: 8px;
  gap: 12px;
}

.result-sender {
  font-weight: 700;
  color: #0f172a;
  font-size: 15px;
  letter-spacing: -0.02em;
  line-height: 1.3;
  font-family: -apple-system, BlinkMacSystemFont, 'SF Pro Display', system-ui, sans-serif;
}

.result-time {
  font-size: 12px;
  color: #64748b;
  font-weight: 500;
  white-space: nowrap;
  background: rgba(0, 0, 0, 0.04);
  padding: 2px 8px;
  border-radius: 8px;
  letter-spacing: 0.01em;
}

/* Channel Information */
.result-chat {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  color: #475569;
  margin-bottom: 12px;
  font-weight: 500;
  background: rgba(0, 122, 255, 0.06);
  padding: 4px 10px;
  border-radius: 10px;
  width: fit-content;
  border: 1px solid rgba(0, 122, 255, 0.08);
}

.result-chat svg {
  color: #64748b;
  width: 14px;
  height: 14px;
}

/* Optimized Message Content for Reading */
.result-content {
  font-size: 15px;
  color: #1e293b;
  line-height: 1.65;
  font-weight: 400;
  letter-spacing: -0.01em;
  overflow: hidden;
  text-overflow: ellipsis;
  display: -webkit-box;
  -webkit-line-clamp: 3;
  -webkit-box-orient: vertical;
  font-family: -apple-system, BlinkMacSystemFont, 'SF Pro Text', system-ui, sans-serif;
  word-spacing: 0.02em;

  /* Improved text rendering */
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  text-rendering: optimizeLegibility;
}

/* Enhanced Highlight Styling */
.result-content :deep(mark) {
  background:
    linear-gradient(135deg, rgba(255, 193, 7, 0.85) 0%, rgba(255, 235, 59, 0.75) 100%);
  color: #0f172a;
  padding: 3px 6px;
  border-radius: 6px;
  font-weight: 600;
  box-shadow: 0 1px 3px rgba(255, 193, 7, 0.2);
  border: 1px solid rgba(255, 193, 7, 0.3);
}

/* Hover Effects - Refined Animation */
.result-item:hover {
  background:
    linear-gradient(145deg, rgba(255, 255, 255, 0.98) 0%, rgba(240, 249, 255, 0.9) 100%);
  transform: translateY(-3px) scale(1.005);
  box-shadow:
    0 12px 28px rgba(0, 122, 255, 0.12),
    0 6px 16px rgba(0, 0, 0, 0.08),
    inset 0 1px 0 rgba(255, 255, 255, 0.95);
  border-color: rgba(0, 122, 255, 0.15);
}

.result-item:hover::before {
  opacity: 1;
}

.result-item:hover .result-avatar {
  transform: scale(1.05);
  filter: brightness(1.1) saturate(1.2);
}

/* Active/Focused State */
.result-item.focused {
  background:
    linear-gradient(135deg, #007aff 0%, #5856d6 100%);
  color: #ffffff;
  box-shadow:
    0 16px 40px rgba(0, 122, 255, 0.3),
    inset 0 1px 0 rgba(255, 255, 255, 0.25);
  transform: translateY(-4px) scale(1.01);
  border-color: rgba(255, 255, 255, 0.2);
}

.result-item.focused::before {
  background: linear-gradient(90deg,
      rgba(255, 255, 255, 0.0) 0%,
      rgba(255, 255, 255, 0.4) 30%,
      rgba(255, 255, 255, 0.6) 70%,
      rgba(255, 255, 255, 0.0) 100%);
  opacity: 1;
}

.result-item.focused .result-avatar {
  background: rgba(255, 255, 255, 0.25) !important;
  color: #ffffff !important;
  border: 2px solid rgba(255, 255, 255, 0.4) !important;
  box-shadow:
    0 6px 20px rgba(255, 255, 255, 0.2),
    inset 0 1px 0 rgba(255, 255, 255, 0.3) !important;
}

.result-item.focused .result-sender {
  color: #ffffff;
}

.result-item.focused .result-time {
  color: rgba(255, 255, 255, 0.8);
  background: rgba(255, 255, 255, 0.15);
}

.result-item.focused .result-chat {
  color: rgba(255, 255, 255, 0.9);
  background: rgba(255, 255, 255, 0.15);
  border-color: rgba(255, 255, 255, 0.2);
}

.result-item.focused .result-chat svg {
  color: rgba(255, 255, 255, 0.7);
}

.result-item.focused .result-content {
  color: rgba(255, 255, 255, 0.95);
}

.result-item.focused .result-content :deep(mark) {
  background: rgba(255, 235, 59, 0.9);
  color: #1e293b;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

/* Scroll Indicator */
.results-container::after {
  content: '';
  position: sticky;
  bottom: -16px;
  left: 0;
  right: 0;
  height: 16px;
  background: linear-gradient(180deg,
      rgba(248, 250, 252, 0.0) 0%,
      rgba(248, 250, 252, 0.8) 100%);
  pointer-events: none;
  z-index: 5;
}

/* Loading State for Results */
.result-item.loading {
  background:
    linear-gradient(90deg,
      rgba(248, 250, 252, 0.8) 25%,
      rgba(241, 245, 249, 0.9) 50%,
      rgba(248, 250, 252, 0.8) 75%);
  background-size: 200% 100%;
  animation: shimmerLoad 2s infinite;
}

@keyframes shimmerLoad {
  0% {
    background-position: 200% 0;
  }

  100% {
    background-position: -200% 0;
  }
}

/* Empty State Enhancement */
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 80px 32px 64px;
  text-align: center;
  background:
    radial-gradient(ellipse at center, rgba(248, 250, 252, 0.8) 0%, rgba(255, 255, 255, 0.4) 70%);
  animation: emptyStateFadeIn 800ms cubic-bezier(0.23, 1, 0.32, 1);
}

@keyframes emptyStateFadeIn {
  from {
    opacity: 0;
    transform: translateY(30px) scale(0.96);
  }

  to {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
}

.empty-icon {
  width: 72px;
  height: 72px;
  color: #cbd5e1;
  margin-bottom: 24px;
  filter: drop-shadow(0 4px 8px rgba(0, 0, 0, 0.06));
  animation: emptyIconFloat 4s ease-in-out infinite;
}

@keyframes emptyIconFloat {

  0%,
  100% {
    transform: translateY(0px);
  }

  50% {
    transform: translateY(-8px);
  }
}

.empty-title {
  font-size: 22px;
  font-weight: 700;
  background: linear-gradient(135deg, #1f2937 0%, #4b5563 100%);
  background-clip: text;
  -webkit-background-clip: text;
  color: transparent;
  margin: 0 0 12px 0;
  letter-spacing: -0.02em;
  line-height: 1.3;
}

.empty-description {
  font-size: 16px;
  color: #6b7280;
  margin: 0;
  line-height: 1.6;
  max-width: 320px;
}

/* Results Performance Optimization */
.results-container {
  contain: layout style;
  will-change: scroll-position;
}

.result-item {
  contain: layout style;
  will-change: transform, box-shadow;
}

/* Accessibility Improvements */
.result-item:focus-visible {
  outline: 2px solid #007aff;
  outline-offset: 3px;
}

.result-item[aria-selected="true"] {
  border-color: #007aff;
  box-shadow:
    0 0 0 3px rgba(0, 122, 255, 0.1),
    0 8px 24px rgba(0, 122, 255, 0.15);
}

/* ✨ Modern Welcome Section */
.welcome-section {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 48px 24px;
  background: rgba(248, 250, 252, 0.4);
}

.welcome-content {
  text-align: center;
  max-width: 400px;
  animation: welcomeFadeIn 400ms ease-out;
}

@keyframes welcomeFadeIn {
  from {
    opacity: 0;
    transform: translateY(16px);
  }

  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.welcome-icon {
  width: 48px;
  height: 48px;
  color: #9ca3af;
  margin: 0 auto 20px;
}

.welcome-title {
  font-size: 20px;
  font-weight: 600;
  color: #1f2937;
  margin: 0 0 8px 0;
  letter-spacing: -0.01em;
  line-height: 1.3;
}

.welcome-subtitle {
  font-size: 14px;
  color: #6b7280;
  margin: 0 0 32px 0;
  font-weight: 400;
  line-height: 1.5;
}

.quick-suggestions {
  text-align: center;
}

.suggestions-label {
  font-size: 12px;
  color: #9ca3af;
  margin-bottom: 12px;
  display: block;
  font-weight: 500;
}

.suggestion-tags {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
  justify-content: center;
}

.suggestion-tag {
  padding: 6px 12px;
  background: rgba(255, 255, 255, 0.8);
  border: 1px solid rgba(0, 0, 0, 0.08);
  border-radius: 16px;
  font-size: 12px;
  color: #4b5563;
  cursor: pointer;
  transition: all 200ms ease;
  font-weight: 500;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.05);
}

.suggestion-tag:hover {
  background: #007aff;
  color: #ffffff;
  transform: translateY(-1px);
  box-shadow: 0 2px 8px rgba(0, 122, 255, 0.3);
}

/* Premium Error State */
.error-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 64px 32px;
  text-align: center;
  background:
    radial-gradient(ellipse at center, rgba(254, 242, 242, 0.6) 0%, rgba(255, 255, 255, 0.3) 70%);
}

.error-icon {
  width: 56px;
  height: 56px;
  color: #ef4444;
  margin-bottom: 20px;
  filter: drop-shadow(0 4px 8px rgba(239, 68, 68, 0.2));
}

.error-title {
  font-size: 20px;
  font-weight: 650;
  color: #1f2937;
  margin: 0 0 12px 0;
  letter-spacing: -0.01em;
}

.error-description {
  font-size: 15px;
  color: #6b7280;
  margin: 0 0 28px 0;
  line-height: 1.5;
}

.retry-button {
  padding: 12px 24px;
  background:
    linear-gradient(135deg, #ef4444 0%, #dc2626 100%);
  color: #ffffff;
  border: none;
  border-radius: 12px;
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  transition: all 250ms cubic-bezier(0.23, 1, 0.32, 1);
  box-shadow:
    0 4px 16px rgba(239, 68, 68, 0.3),
    inset 0 1px 0 rgba(255, 255, 255, 0.2);
}

.retry-button:hover {
  background:
    linear-gradient(135deg, #dc2626 0%, #b91c1c 100%);
  transform: translateY(-1px);
  box-shadow:
    0 6px 20px rgba(239, 68, 68, 0.4),
    inset 0 1px 0 rgba(255, 255, 255, 0.3);
}

/* Graceful Empty State */
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 64px 32px;
  text-align: center;
  background:
    radial-gradient(ellipse at center, rgba(248, 250, 252, 0.6) 0%, rgba(255, 255, 255, 0.3) 70%);
}

.empty-icon {
  width: 56px;
  height: 56px;
  color: #cbd5e1;
  margin-bottom: 20px;
  filter: drop-shadow(0 4px 8px rgba(0, 0, 0, 0.05));
}

.empty-title {
  font-size: 20px;
  font-weight: 650;
  color: #1f2937;
  margin: 0 0 12px 0;
  letter-spacing: -0.01em;
}

.empty-description {
  font-size: 15px;
  color: #6b7280;
  margin: 0;
  line-height: 1.5;
}

/* 🎯 Modern Modal Footer */
.modal-footer {
  padding: 16px 24px;
  border-top: 1px solid rgba(0, 0, 0, 0.06);
  background: rgba(248, 250, 252, 0.6);
  backdrop-filter: blur(10px);
}

.keyboard-hints {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 16px;
  font-size: 12px;
  color: #6b7280;
  font-weight: 500;
}

.keyboard-hints kbd {
  padding: 3px 6px;
  background: rgba(255, 255, 255, 0.8);
  border: 1px solid rgba(0, 0, 0, 0.1);
  border-radius: 4px;
  font-size: 10px;
  font-family: 'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono', Consolas, monospace;
  color: #374151;
  font-weight: 600;
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.1);
}

/* Liquid Transitions */
.modal-fade-enter-active,
.modal-fade-leave-active {
  transition: all 400ms cubic-bezier(0.23, 1, 0.32, 1);
}

.modal-fade-enter-from,
.modal-fade-leave-to {
  opacity: 0;
  backdrop-filter: blur(0px) saturate(100%) brightness(1);
}

.modal-scale-enter-active,
.modal-scale-leave-active {
  transition: all 500ms cubic-bezier(0.23, 1, 0.32, 1);
}

.modal-scale-enter-from,
.modal-scale-leave-to {
  opacity: 0;
  transform: scale(0.92) translateY(-30px);
  filter: blur(8px);
}

/* Responsive Refinement */
@media (max-width: 640px) {
  .search-backdrop {
    padding: 20px;
  }

  .search-modal {
    margin-top: 5vh;
    max-width: none;
    border-radius: 20px;
  }

  .modal-header,
  .search-section,
  .results-header {
    padding-left: 24px;
    padding-right: 24px;
  }

  .results-container {
    padding: 8px 24px 24px;
  }

  .search-title {
    font-size: 20px;
  }

  .search-input {
    font-size: 16px;
    height: 56px;
  }
}

/* Micro-interactions for perfection */
.search-modal * {
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

/* 🔧 新增：搜索消息高亮样式 */
:global(.message-highlight) {
  background: linear-gradient(135deg, rgba(255, 193, 7, 0.15) 0%, rgba(255, 235, 59, 0.1) 100%) !important;
  border: 2px solid rgba(255, 193, 7, 0.4) !important;
  border-radius: 8px !important;
  box-shadow:
    0 0 0 3px rgba(255, 193, 7, 0.1),
    0 4px 12px rgba(255, 193, 7, 0.2) !important;
  animation: messageHighlight 3s ease-out;
  position: relative;
  z-index: 10;
}

:global(.search-target) {
  transform: scale(1.02);
  transition: all 0.3s cubic-bezier(0.25, 0.8, 0.25, 1);
}

:global(.search-target::before) {
  content: '🎯';
  position: absolute;
  top: -8px;
  right: -8px;
  background: #ff9800;
  color: white;
  border-radius: 50%;
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 12px;
  box-shadow: 0 2px 8px rgba(255, 152, 0, 0.3);
  animation: searchTargetPulse 2s ease-in-out infinite;
}

@keyframes messageHighlight {
  0% {
    background: rgba(255, 193, 7, 0.3);
    transform: scale(1.05);
  }

  50% {
    background: rgba(255, 193, 7, 0.2);
    transform: scale(1.02);
  }

  100% {
    background: rgba(255, 193, 7, 0.1);
    transform: scale(1);
  }
}

@keyframes searchTargetPulse {

  0%,
  100% {
    transform: scale(1);
    opacity: 1;
  }

  50% {
    transform: scale(1.2);
    opacity: 0.8;
  }
}
</style>