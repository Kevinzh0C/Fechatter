
import { nextTick } from 'vue'

class MessageNavigationManager {
  constructor() {
    this.navigationQueue = []
    this.isNavigating = false
    this.highlightTimeouts = new Map()
    this.scrollContainers = new Map()
    this.chatRouterInstance = null

    // Navigation analytics
    this.analytics = {
      totalNavigations: 0,
      successfulNavigations: 0,
      failedNavigations: 0,
      averageNavigationTime: 0,
      navigationHistory: []
    }

    console.log('[MessageNavigationManager] Initialized')
  }

  /**
   * Core Navigation Function - Entry Point
   */
  async navigateToMessage(params) {
    const startTime = Date.now()

    // Validation
    if (!this.validateNavigationParams(params)) {
      return { success: false, error: 'Invalid navigation parameters' }
    }

    // 🔄 Queue management for concurrent navigations
    const navigationId = `nav_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`

    const navigationTask = {
      id: navigationId,
      params,
      startTime,
      status: 'queued'
    }

    this.navigationQueue.push(navigationTask)
    this.analytics.totalNavigations++

    console.log('[MessageNavigation] Starting navigation:', {
      navigationId,
      messageId: params.messageId,
      chatId: params.chatId,
      source: params.source || 'unknown'
    })

    try {
      const result = await this.executeNavigation(navigationTask)

      // Analytics update
      const duration = Date.now() - startTime
      this.updateAnalytics(result.success, duration)

      return result
    } catch (error) {
      console.error('[MessageNavigation] Navigation failed:', error)
      this.analytics.failedNavigations++
      return { success: false, error: error.message }
    } finally {
      // Clean up navigation queue
      this.navigationQueue = this.navigationQueue.filter(t => t.id !== navigationId)
    }
  }

  /**
   * Step 1: Parameter Validation
   */
  validateNavigationParams(params) {
    const required = ['messageId', 'chatId']
    const missing = required.filter(field => !params[field])

    if (missing.length > 0) {
      console.error('[Validation] Missing required fields:', missing)
      return false
    }

    return true
  }

  /**
   * Step 2: Execute Navigation Process
   */
  async executeNavigation(navigationTask) {
    const { params } = navigationTask

    try {
      navigationTask.status = 'executing'

      // 🔄 Step 2.1: Chat Navigation (if needed)
      const chatNavigationResult = await this.handleChatNavigation(params)
      if (!chatNavigationResult.success) {
        return chatNavigationResult
      }

      // 📜 Step 2.2: Message Context Loading
      const contextResult = await this.ensureMessageContext(params)
      if (!contextResult.success) {
        return contextResult
      }

      // Step 2.3: Scroll to Message
      const scrollResult = await this.performScrollToMessage(params)
      if (!scrollResult.success) {
        return scrollResult
      }

      // ✨ Step 2.4: Highlight Message
      const highlightResult = await this.highlightTargetMessage(params)

      navigationTask.status = 'completed'

      return {
        success: true,
        navigationId: navigationTask.id,
        duration: Date.now() - navigationTask.startTime,
        steps: {
          chatNavigation: chatNavigationResult,
          messageContext: contextResult,
          scroll: scrollResult,
          highlight: highlightResult
        }
      }

    } catch (error) {
      navigationTask.status = 'failed'
      throw error
    }
  }

  /**
   * 🔄 Step 2.1: Chat Navigation Handler
   */
  async handleChatNavigation(params) {
    const currentChatId = this.getCurrentChatId()

    if (params.chatId && params.chatId != currentChatId) {
      console.log('🔄 [ChatNavigation] Switching to chat:', params.chatId)

      try {
        if (this.chatRouterInstance) {
          await this.chatRouterInstance.push(`/chat/${params.chatId}`)

          // Wait for chat to load
          await this.waitForChatLoad(params.chatId)

          return { success: true, action: 'chat_switched' }
        } else {
          return { success: false, error: 'Router instance not available' }
        }
      } catch (error) {
        return { success: false, error: `Chat navigation failed: ${error.message}` }
      }
    }

    return { success: true, action: 'same_chat' }
  }

  /**
   * 📜 Step 2.2: Enhanced Message Context Ensurance
   */
  async ensureMessageContext(params) {
    try {
      // Step 1: Check if message exists in current DOM
      let messageElement = document.querySelector(`[data-message-id="${params.messageId}"]`)

      if (messageElement) {
        console.log('📜 [MessageContext] Message found in DOM:', params.messageId)
        return { success: true, action: 'message_found_in_dom' }
      }

      console.log('📜 [MessageContext] Message not in DOM, loading context for:', params.messageId)

      // Step 2: Try multiple loading strategies
      const strategies = [
        () => this.loadViaChatStore(params),
        () => this.loadViaAPI(params),
        () => this.loadViaMessageService(params),
        () => this.loadViaScrollingStrategy(params)
      ]

      for (let i = 0; i < strategies.length; i++) {
        const strategyName = ['ChatStore', 'API', 'MessageService', 'Scrolling'][i]
        console.log(`📜 [MessageContext] Trying strategy ${i + 1}: ${strategyName}`)

        try {
          const result = await strategies[i]()
          if (result.success) {
            // Wait for DOM update and verify message is loaded
            await nextTick()
            await new Promise(resolve => setTimeout(resolve, 200))

            messageElement = document.querySelector(`[data-message-id="${params.messageId}"]`)
            if (messageElement) {
              console.log(`📜 [MessageContext] Success with ${strategyName}`)
              return { success: true, action: `loaded_via_${strategyName.toLowerCase()}` }
            }
          }
        } catch (error) {
          console.warn(`📜 [MessageContext] Strategy ${strategyName} failed:`, error.message)
        }
      }

      return { success: false, error: 'All loading strategies failed' }

    } catch (error) {
      return { success: false, error: `Context loading failed: ${error.message}` }
    }
  }

  /**
   * 🏪 Load via Chat Store
   */
  async loadViaChatStore(params) {
    const chatStore = await this.getChatStore()
    if (!chatStore) {
      return { success: false, error: 'Chat store not available' }
    }

    // Try different store methods
    const storeMethods = [
      'loadMessageContext',
      'fetchMessageContext',
      'loadHistoryAroundMessage',
      'fetchMoreMessages',
      'loadMessagesContaining'
    ]

    for (const method of storeMethods) {
      if (typeof chatStore[method] === 'function') {
        try {
          await chatStore[method](params.chatId, params.messageId)
          return { success: true, method }
        } catch (error) {
          console.warn(`📜 [ChatStore] Method ${method} failed:`, error.message)
        }
      }
    }

    return { success: false, error: 'No usable chat store methods' }
  }

  /**
   * 🌐 Load via direct API call
   */
  async loadViaAPI(params) {
    try {
      // Try to fetch message context directly
      const response = await fetch(`/api/chats/${params.chatId}/messages/${params.messageId}/context`, {
        headers: {
          'Authorization': `Bearer ${this.getAuthToken()}`,
          'Content-Type': 'application/json'
        }
      })

      if (response.ok) {
        const data = await response.json()
        // Trigger re-render by dispatching custom event
        window.dispatchEvent(new CustomEvent('messagesLoaded', { detail: data }))
        return { success: true, data }
      }

      return { success: false, error: `API response: ${response.status}` }
    } catch (error) {
      return { success: false, error: error.message }
    }
  }

  /**
   * Load via Message Service
   */
  async loadViaMessageService(params) {
    try {
      // Try UnifiedMessageService if available
      const messageService = window.UnifiedMessageService || window.messageService
      if (messageService && typeof messageService.loadMessageContext === 'function') {
        await messageService.loadMessageContext(params.chatId, params.messageId)
        return { success: true, service: 'UnifiedMessageService' }
      }

      // Try other global services
      if (window.chatManager && typeof window.chatManager.loadMessage === 'function') {
        await window.chatManager.loadMessage(params.messageId, params.chatId)
        return { success: true, service: 'ChatManager' }
      }

      return { success: false, error: 'No message service available' }
    } catch (error) {
      return { success: false, error: error.message }
    }
  }

  /**
   * 📜 Load via scrolling strategy (for very old messages)
   */
  async loadViaScrollingStrategy(params) {
    try {
      const scrollContainer = this.getScrollContainer(params.chatId)
      if (!scrollContainer) {
        return { success: false, error: 'No scroll container' }
      }

      console.log('📜 [ScrollingStrategy] Attempting to load historical messages by scrolling')

      // Scroll to top and trigger load more
      scrollContainer.scrollTop = 0

      // Wait for load more to trigger
      await new Promise(resolve => setTimeout(resolve, 500))

      // Try multiple times to load more messages
      for (let attempt = 0; attempt < 10; attempt++) {
        // Trigger scroll event to load more
        scrollContainer.dispatchEvent(new Event('scroll'))

        // Check if message appeared
        await new Promise(resolve => setTimeout(resolve, 300))
        const messageElement = document.querySelector(`[data-message-id="${params.messageId}"]`)
        if (messageElement) {
          return { success: true, attempts: attempt + 1 }
        }

        // Scroll to top again
        scrollContainer.scrollTop = 0
        await new Promise(resolve => setTimeout(resolve, 200))
      }

      return { success: false, error: 'Message not found after scrolling attempts' }
    } catch (error) {
      return { success: false, error: error.message }
    }
  }

  /**
   * 🔑 Get authentication token
   */
  getAuthToken() {
    // Try multiple sources for auth token
    const sources = [
      () => localStorage.getItem('auth_token'),
      () => localStorage.getItem('token'),
      () => sessionStorage.getItem('auth_token'),
      () => document.cookie.match(/auth_token=([^;]+)/)?.[1],
      () => window.authToken
    ]

    for (const source of sources) {
      try {
        const token = source()
        if (token) return token
      } catch (error) {
        // Continue to next source
      }
    }

    return null
  }

  /**
   * Step 2.3: Scroll to Message
   */
  async performScrollToMessage(params) {
    try {
      await nextTick()

      const messageElement = document.querySelector(`[data-message-id="${params.messageId}"]`)

      if (!messageElement) {
        return { success: false, error: 'Message element not found in DOM' }
      }

      // Get scroll container
      const scrollContainer = this.getScrollContainer(params.chatId)

      if (!scrollContainer) {
        return { success: false, error: 'Scroll container not found' }
      }

      // Calculate optimal scroll position
      const scrollPosition = this.calculateScrollPosition(messageElement, scrollContainer, params)

      // Perform smooth scroll
      scrollContainer.scrollTo({
        top: scrollPosition,
        behavior: params.scrollBehavior || 'smooth'
      })

      // Wait for scroll completion
      await this.waitForScrollCompletion(scrollContainer, scrollPosition)

      return {
        success: true,
        action: 'scrolled_to_message',
        scrollPosition,
        messageElement
      }

    } catch (error) {
      return { success: false, error: `Scroll failed: ${error.message}` }
    }
  }

  /**
   * ✨ Step 2.4: Message Highlighting
   */
  async highlightTargetMessage(params) {
    try {
      const messageElement = document.querySelector(`[data-message-id="${params.messageId}"]`)

      if (!messageElement) {
        return { success: false, error: 'Message element not found for highlighting' }
      }

      // Clear any existing highlights
      this.clearExistingHighlights()

      // Apply search highlight if query provided
      if (params.searchQuery) {
        this.applySearchHighlight(messageElement, params.searchQuery)
      }

      // Apply navigation highlight
      this.applyNavigationHighlight(messageElement, params)

      // Set highlight duration
      const duration = params.highlightDuration || 3000
      this.scheduleHighlightRemoval(params.messageId, duration)

      return {
        success: true,
        action: 'message_highlighted',
        duration,
        hasSearchHighlight: !!params.searchQuery
      }

    } catch (error) {
      return { success: false, error: `Highlighting failed: ${error.message}` }
    }
  }

  /**
   * Apply Search Term Highlighting
   */
  applySearchHighlight(messageElement, searchQuery) {
    const contentElements = messageElement.querySelectorAll(
      '.message-content, .message-text, .result-content'
    )

    contentElements.forEach(element => {
      const originalText = element.textContent
      const highlightedHTML = this.highlightSearchTerms(originalText, searchQuery)

      // Store original content for restoration
      element.setAttribute('data-original-content', originalText)
      element.innerHTML = highlightedHTML
    })
  }

  /**
   * ✨ Apply Navigation Highlighting
   */
  applyNavigationHighlight(messageElement, params) {
    // Add navigation highlight class
    messageElement.classList.add('message-navigation-highlight')

    // Add pulse animation for better visibility
    if (params.pulseAnimation !== false) {
      messageElement.classList.add('message-navigation-pulse')

      // Remove pulse after animation
      setTimeout(() => {
        messageElement.classList.remove('message-navigation-pulse')
      }, 1000)
    }

    // Add visual indicator
    this.addNavigationIndicator(messageElement, params)
  }

  /**
   * Add Visual Navigation Indicator
   */
  addNavigationIndicator(messageElement, params) {
    // Remove existing indicators
    const existingIndicator = messageElement.querySelector('.navigation-indicator')
    if (existingIndicator) {
      existingIndicator.remove()
    }

    // Create new indicator
    const indicator = document.createElement('div')
    indicator.className = 'navigation-indicator'
    indicator.innerHTML = `
      <div class="indicator-pulse"></div>
      <div class="indicator-label">${params.source === 'search' ? 'Found' : '📍 Located'}</div>
    `

    // Insert indicator
    messageElement.style.position = 'relative'
    messageElement.appendChild(indicator)
  }

  /**
   * 🧹 Clear Existing Highlights
   */
  clearExistingHighlights() {
    // Clear navigation highlights
    document.querySelectorAll('.message-navigation-highlight').forEach(el => {
      el.classList.remove('message-navigation-highlight', 'message-navigation-pulse')
    })

    // Clear search highlights
    document.querySelectorAll('[data-original-content]').forEach(el => {
      const originalContent = el.getAttribute('data-original-content')
      if (originalContent) {
        el.textContent = originalContent
        el.removeAttribute('data-original-content')
      }
    })

    // Clear navigation indicators
    document.querySelectorAll('.navigation-indicator').forEach(el => {
      el.remove()
    })

    // Clear all highlight timeouts
    this.highlightTimeouts.forEach(timeout => clearTimeout(timeout))
    this.highlightTimeouts.clear()
  }

  /**
   * Highlight Search Terms
   */
  highlightSearchTerms(text, searchQuery) {
    if (!searchQuery || !text) return text

    const terms = searchQuery.split(' ').filter(term => term.length > 0)
    let highlightedText = text

    terms.forEach(term => {
      const regex = new RegExp(`(${this.escapeRegex(term)})`, 'gi')
      highlightedText = highlightedText.replace(regex,
        '<mark class="search-term-highlight">$1</mark>'
      )
    })

    return highlightedText
  }

  /**
   * ⏰ Schedule Highlight Removal
   */
  scheduleHighlightRemoval(messageId, duration) {
    // Clear existing timeout for this message
    if (this.highlightTimeouts.has(messageId)) {
      clearTimeout(this.highlightTimeouts.get(messageId))
    }

    // Set new timeout
    const timeoutId = setTimeout(() => {
      this.removeHighlightForMessage(messageId)
      this.highlightTimeouts.delete(messageId)
    }, duration)

    this.highlightTimeouts.set(messageId, timeoutId)
  }

  /**
   * 🧹 Remove Highlight for Specific Message
   */
  removeHighlightForMessage(messageId) {
    const messageElement = document.querySelector(`[data-message-id="${messageId}"]`)
    if (!messageElement) return

    // Remove navigation highlight
    messageElement.classList.remove('message-navigation-highlight', 'message-navigation-pulse')

    // Restore original content
    messageElement.querySelectorAll('[data-original-content]').forEach(el => {
      const originalContent = el.getAttribute('data-original-content')
      if (originalContent) {
        el.textContent = originalContent
        el.removeAttribute('data-original-content')
      }
    })

    // Remove navigation indicator
    const indicator = messageElement.querySelector('.navigation-indicator')
    if (indicator) {
      indicator.remove()
    }
  }

  /**
   * Utility Functions
   */
  calculateScrollPosition(messageElement, scrollContainer, params) {
    const messageRect = messageElement.getBoundingClientRect()
    const containerRect = scrollContainer.getBoundingClientRect()

    // Calculate position to center the message
    const messageOffset = messageElement.offsetTop
    const containerHeight = containerRect.height
    const messageHeight = messageRect.height

    // Center the message with some offset for better visibility
    const targetPosition = messageOffset - (containerHeight / 2) + (messageHeight / 2)

    return Math.max(0, targetPosition)
  }

  async waitForScrollCompletion(scrollContainer, targetPosition) {
    return new Promise(resolve => {
      let attempts = 0
      const maxAttempts = 50

      const checkScroll = () => {
        attempts++
        const currentScroll = scrollContainer.scrollTop
        const tolerance = 10

        if (Math.abs(currentScroll - targetPosition) <= tolerance || attempts >= maxAttempts) {
          resolve()
        } else {
          setTimeout(checkScroll, 50)
        }
      }

      checkScroll()
    })
  }

  async waitForChatLoad(chatId) {
    return new Promise(resolve => {
      let attempts = 0
      const maxAttempts = 20

      const checkChatLoad = () => {
        attempts++
        const currentChat = this.getCurrentChatId()

        if (currentChat == chatId || attempts >= maxAttempts) {
          resolve()
        } else {
          setTimeout(checkChatLoad, 100)
        }
      }

      setTimeout(checkChatLoad, 50)
    })
  }

  escapeRegex(string) {
    return string.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
  }

  /**
   * Configuration and State Management
   */
  registerScrollContainer(chatId, container) {
    this.scrollContainers.set(chatId, container)
  }

  registerRouter(routerInstance) {
    this.chatRouterInstance = routerInstance
  }

  getScrollContainer(chatId) {
    return this.scrollContainers.get(chatId) ||
      document.querySelector('.simple-message-list, .message-list, .messages-container')
  }

  getCurrentChatId() {
    // This should integrate with your chat store
    const pathMatch = window.location.pathname.match(/\/chat\/(\d+)/)
    return pathMatch ? pathMatch[1] : null
  }

  async getChatStore() {
    try {
      const { useChatStore } = await import('@/stores/chat')
      return useChatStore()
    } catch (error) {
      console.warn('[MessageNavigation] Chat store not available:', error)
      return null
    }
  }

  /**
   * Analytics and Monitoring
   */
  updateAnalytics(success, duration) {
    if (success) {
      this.analytics.successfulNavigations++
    } else {
      this.analytics.failedNavigations++
    }

    // Update average navigation time
    const totalSuccessful = this.analytics.successfulNavigations
    if (totalSuccessful > 0) {
      this.analytics.averageNavigationTime =
        (this.analytics.averageNavigationTime * (totalSuccessful - 1) + duration) / totalSuccessful
    }

    // Keep navigation history (last 50)
    this.analytics.navigationHistory.push({
      timestamp: Date.now(),
      success,
      duration
    })

    if (this.analytics.navigationHistory.length > 50) {
      this.analytics.navigationHistory.shift()
    }
  }

  getAnalytics() {
    return {
      ...this.analytics,
      successRate: this.analytics.totalNavigations > 0
        ? (this.analytics.successfulNavigations / this.analytics.totalNavigations * 100).toFixed(2) + '%'
        : '0%'
    }
  }

  /**
   * 🔄 Public API Methods
   */
  async jumpToMessage(params) {
    return this.navigateToMessage({
      source: 'search',
      scrollBehavior: 'smooth',
      highlightDuration: 3000,
      pulseAnimation: true,
      ...params
    })
  }

  clearAllHighlights() {
    this.clearExistingHighlights()
  }

  destroy() {
    this.clearExistingHighlights()
    this.highlightTimeouts.clear()
    this.scrollContainers.clear()
    this.navigationQueue = []
    console.log('[MessageNavigationManager] Destroyed')
  }
}

// Global Instance
export const messageNavigationManager = new MessageNavigationManager()

// Vue Integration Helper
export function useMessageNavigation() {
  return {
    navigateToMessage: (params) => messageNavigationManager.navigateToMessage(params),
    jumpToMessage: (params) => messageNavigationManager.jumpToMessage(params),
    clearHighlights: () => messageNavigationManager.clearAllHighlights(),
    getAnalytics: () => messageNavigationManager.getAnalytics(),

    // Registration methods for components
    registerScrollContainer: (chatId, container) =>
      messageNavigationManager.registerScrollContainer(chatId, container),
    registerRouter: (router) =>
      messageNavigationManager.registerRouter(router)
  }
}

export default messageNavigationManager 