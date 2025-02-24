/**
 * 🎯 Navigation Manager - Production-Grade Message Switching
 * 
 * Core Constraints Compliance:
 * - Data Consistency: No message list flashing or clearing
 * - Request Idempotency: Duplicate clicks properly handled  
 * - Cancellation Semantics: Graceful abortion without errors
 * - State Synchronization: UI and data always in sync
 * - User Perception: <200ms switch threshold
 */

import { ref, computed } from 'vue';
import { unifiedMessageService } from '@/services/messageSystem/UnifiedMessageService';

class NavigationManager {
  constructor() {
    // Navigation state
    this.isNavigating = ref(false);
    this.currentNavigation = ref(null);
    this.navigationHistory = [];

    // 🔧 FIXED: Single active navigation to prevent race conditions
    this.activeNavigation = null; // Only one navigation at a time
    this.activeChatId = null;

    // Request management with proper cancellation
    this.activeRequests = new Map(); // chatId -> { controller, type, startTime }

    // Error recovery
    this.retryAttempts = new Map();
    this.maxRetries = 2; // Reduce retries for faster UX

    // Dependencies
    this.router = null;
    this.chatStore = null;
    this.isInitialized = false;
  }

  /**
   * Initialize with proper dependency injection
   */
  initialize(router, chatStore) {
    if (this.isInitialized) return;

    this.router = router;
    this.chatStore = chatStore;
    this.isInitialized = true;

    this.setupNavigationGuards();
    console.log('🧭 Navigation Manager initialized successfully');
  }

  /**
   * Setup navigation guards for cleanup
   */
  setupNavigationGuards() {
    if (!this.router?.beforeEach) return;

    this.router.beforeEach((to, from, next) => {
      // Clear retry attempts when changing routes
      if (from.name === 'chat' && to.name !== 'chat') {
        this.retryAttempts.clear();
      }
      next();
    });
  }

  /**
   * 🔧 CORE FIX: Production-grade navigation with proper concurrency control
   */
  async navigateToChat(chatId, options = {}) {
    if (!this.isInitialized || !this.router || !this.chatStore) {
      console.error('🧭 Navigation Manager not properly initialized');
      throw new Error('Navigation Manager not initialized');
    }

    const normalizedId = parseInt(chatId);

    // 🔧 BALANCED FIX: Check if we're already in chat AND have messages loaded
    const currentRoute = this.router.currentRoute.value;
    const isOnCorrectRoute = currentRoute.params.id && parseInt(currentRoute.params.id) === normalizedId;
    const hasMessages = unifiedMessageService.getMessagesForChat(normalizedId)?.length > 0;
    const isCurrentChat = this.chatStore.currentChatId === normalizedId;

    if (isOnCorrectRoute && hasMessages && isCurrentChat) {
      console.log(`✅ Already in chat ${normalizedId} with messages loaded`);
      return { success: true, cached: true, duration: 0 };
    }

    // 🔧 FIX 2: Cancel any active navigation (only one at a time)
    if (this.activeNavigation && this.activeChatId !== normalizedId) {
      console.log(`🚫 Cancelling navigation to chat ${this.activeChatId}, switching to ${normalizedId}`);
      await this.cancelActiveNavigation();
    }

    // 🔧 FIX 3: If same chat navigation is already in progress, wait for it
    if (this.activeNavigation && this.activeChatId === normalizedId) {
      console.log(`🔄 Already navigating to chat ${normalizedId}, waiting...`);
      return this.activeNavigation;
    }

    // Start new navigation
    this.activeChatId = normalizedId;
    this.activeNavigation = this._performNavigation(normalizedId, options);

    try {
      const result = await this.activeNavigation;
      return result;
    } finally {
      this.activeNavigation = null;
      this.activeChatId = null;
    }
  }

  /**
   * 🔧 FIXED: Graceful cancellation without errors
   */
  async cancelActiveNavigation() {
    if (!this.activeNavigation) return;

    // Cancel all active requests gracefully
    for (const [chatId, requestInfo] of this.activeRequests.entries()) {
      if (requestInfo.controller && !requestInfo.controller.signal.aborted) {
        console.log(`🚫 Gracefully cancelling ${requestInfo.type} request for chat ${chatId}`);
        requestInfo.controller.abort();
      }
    }

    // Clear request tracking
    this.activeRequests.clear();

    // Wait for current navigation to finish (with timeout)
    try {
      await Promise.race([
        this.activeNavigation,
        new Promise((_, reject) =>
          setTimeout(() => reject(new Error('Navigation timeout')), 1000)
        )
      ]);
    } catch (error) {
      // Expected when aborting, ignore gracefully
      console.log('🔄 Navigation cancelled gracefully');
    }

    this.activeNavigation = null;
    this.activeChatId = null;
  }

  /**
   * 🔧 CORE FIX: Perform navigation with proper error handling
   */
  async _performNavigation(chatId, options = {}) {
    const startTime = Date.now();

    try {
      this.isNavigating.value = true;
      this.currentNavigation.value = { chatId, startTime };

      console.log(`🚀 Starting navigation to chat ${chatId}`);

      // 🔧 FIX: Immediate UI state update to prevent flashing
      await this._updateUIState(chatId);

      // 🔧 FIX: Sequential execution to prevent race conditions
      await this._navigateRoute(chatId);
      await this._ensureChatData(chatId);
      await this._loadMessagesGracefully(chatId, options);
      await this._loadMembersNonCritical(chatId);

      const duration = Date.now() - startTime;
      console.log(`✅ Navigation to chat ${chatId} completed in ${duration}ms`);

      this.navigationHistory.push({
        chatId,
        timestamp: Date.now(),
        duration,
        success: true
      });

      return { success: true, duration };

    } catch (error) {
      // 🔧 FIX: Distinguish between cancellation and real errors
      if (this._isCancellationError(error)) {
        console.log(`🔄 Navigation to chat ${chatId} was cancelled`);
        return { success: false, cancelled: true };
      }

      console.error(`❌ Navigation to chat ${chatId} failed:`, error);

      // Only retry for non-cancellation errors
      if (this.shouldRetry(chatId)) {
        console.log(`🔄 Retrying navigation to chat ${chatId}...`);
        return this._retryNavigation(chatId, options);
      }

      throw error;

    } finally {
      this.isNavigating.value = false;
      this.currentNavigation.value = null;

      // Clean up any remaining requests for this chat
      this.activeRequests.delete(chatId);
    }
  }

  /**
   * 🔧 NEW: Immediate UI state update to prevent flashing
   */
  async _updateUIState(chatId) {
    try {
      // Set current chat immediately to update UI
      await this.chatStore.setCurrentChat(chatId);

      // Check if we have cached messages to show immediately
      const cachedMessages = unifiedMessageService.getMessagesForChat(chatId);
      if (cachedMessages && cachedMessages.length > 0) {
        console.log(`⚡ Using cached messages for immediate display: ${cachedMessages.length} messages`);
        // Messages are already available, UI will update automatically
      }
    } catch (error) {
      console.warn('Failed to update UI state:', error);
      // Non-critical, continue navigation
    }
  }

  /**
   * Route navigation with redundancy handling
   */
  async _navigateRoute(chatId) {
    try {
      // 🔧 ENHANCED: Check current route before attempting navigation
      const currentRoute = this.router.currentRoute.value;
      const currentChatId = currentRoute.params.id ? parseInt(currentRoute.params.id) : null;

      // If we're already on the correct route, skip navigation
      if (currentChatId === parseInt(chatId)) {
        console.log(`ℹ️ Already on chat ${chatId} route, skipping navigation`);
        return true;
      }

      await this.router.push(`/chat/${chatId}`);
      return true;
    } catch (error) {
      // 🔧 EXPANDED: Handle all possible Vue Router navigation errors
      if (error.name === 'NavigationDuplicated' ||
        error.message?.includes('redundant navigation') ||
        error.message?.includes('Avoided redundant') ||
        error.toString().includes('redundant')) {
        console.log(`ℹ️ Redundant navigation to chat ${chatId} detected and handled gracefully`);
        return true;
      }

      // 🔧 NEW: Handle navigation cancelled errors
      if (error.name === 'NavigationCancelled' ||
        error.message?.includes('Navigation cancelled')) {
        console.log(`ℹ️ Navigation to chat ${chatId} was cancelled, treating as success`);
        return true;
      }

      console.error(`❌ Failed to navigate to route /chat/${chatId}:`, error);
      throw error;
    }
  }

  /**
   * Ensure chat metadata with caching
   */
  async _ensureChatData(chatId) {
    const chat = this.chatStore.getChatById(chatId);
    if (chat) return chat;

    await this.chatStore.fetchChats();

    const fetchedChat = this.chatStore.getChatById(chatId);
    if (!fetchedChat) {
      throw new Error(`Chat ${chatId} not found`);
    }

    return fetchedChat;
  }

  /**
   * 🔧 FIXED: Load messages with graceful cancellation handling
   */
  async _loadMessagesGracefully(chatId, options = {}) {
    const requestType = 'messages';
    const controller = new AbortController();

    // Track this request
    this.activeRequests.set(chatId, {
      controller,
      type: requestType,
      startTime: Date.now()
    });

    try {
      const messages = await unifiedMessageService.fetchMessages(chatId, {
        limit: options.messageLimit || 15,
        abortSignal: controller.signal,
        isPreload: options.isPreload || false
      });

      // Successfully loaded, remove from tracking
      this.activeRequests.delete(chatId);
      return messages;

    } catch (error) {
      // Remove from tracking regardless of error type
      this.activeRequests.delete(chatId);

      if (this._isCancellationError(error)) {
        console.log(`🚫 Message loading cancelled for chat ${chatId}`);
        // Return empty array instead of throwing for cancellation
        return [];
      }

      // Re-throw non-cancellation errors
      throw error;
    }
  }

  /**
   * Load members (non-critical operation)
   */
  async _loadMembersNonCritical(chatId) {
    try {
      return await this.chatStore.fetchChatMembers(chatId);
    } catch (error) {
      console.warn(`⚠️ Failed to load members for chat ${chatId}:`, error);
      return [];
    }
  }

  /**
   * 🔧 NEW: Check if error is from request cancellation
   */
  _isCancellationError(error) {
    return error.name === 'AbortError' ||
      error.message === 'canceled' ||
      error.message === 'Navigation cancelled' ||
      error.code === 'ERR_CANCELED';
  }

  /**
   * Check if should retry (reduced retry logic)
   */
  shouldRetry(chatId) {
    const attempts = this.retryAttempts.get(chatId) || 0;
    return attempts < this.maxRetries;
  }

  /**
   * Retry with exponential backoff
   */
  async _retryNavigation(chatId, options) {
    const attempts = this.retryAttempts.get(chatId) || 0;
    this.retryAttempts.set(chatId, attempts + 1);

    // Shorter backoff for better UX
    const delay = Math.min(300 * Math.pow(1.5, attempts), 1000);
    await new Promise(resolve => setTimeout(resolve, delay));

    try {
      const result = await this._performNavigation(chatId, options);
      this.retryAttempts.delete(chatId);
      return result;
    } catch (error) {
      if (attempts + 1 >= this.maxRetries) {
        this.retryAttempts.delete(chatId);
      }
      throw error;
    }
  }

  /**
   * Get current navigation state
   */
  getNavigationState() {
    return {
      isNavigating: this.isNavigating.value,
      currentNavigation: this.currentNavigation.value,
      activeRequests: this.activeRequests.size,
      activeChatId: this.activeChatId,
      hasActiveNavigation: !!this.activeNavigation,
      history: this.navigationHistory.slice(-10)
    };
  }

  /**
   * 🔧 ENHANCED: Preload with request tracking
   */
  async preloadChat(chatId) {
    try {
      const normalizedId = parseInt(chatId);

      // Skip if already loaded or currently being loaded
      if (this.activeRequests.has(normalizedId)) {
        console.log(`📦 Chat ${normalizedId} already being loaded`);
        return;
      }

      const messages = unifiedMessageService.getMessagesForChat(normalizedId);
      if (messages && messages.length > 0) {
        return;
      }

      // Preload in background with tracking
      const controller = new AbortController();
      this.activeRequests.set(normalizedId, {
        controller,
        type: 'preload',
        startTime: Date.now()
      });

      try {
        await unifiedMessageService.fetchMessages(normalizedId, {
          limit: 10,
          abortSignal: controller.signal,
          isPreload: true
        });

        console.log(`📦 Preloaded chat ${normalizedId}`);
      } finally {
        this.activeRequests.delete(normalizedId);
      }

    } catch (error) {
      if (!this._isCancellationError(error)) {
        console.debug(`Preload failed for chat ${chatId}:`, error);
      }
    }
  }
}

// Create and export singleton instance
export const navigationManager = new NavigationManager();

// 🔧 FIXED: Simplified factory function that requires pre-initialized manager
export function createNavigationHelper(router, chatStore) {
  // Initialize the manager with proper dependencies
  navigationManager.initialize(router, chatStore);

  return {
    navigateToChat: (chatId, options) => navigationManager.navigateToChat(chatId, options),
    preloadChat: (chatId) => navigationManager.preloadChat(chatId),
    cancelNavigation: () => navigationManager.cancelActiveNavigation(),
    isNavigating: computed(() => navigationManager.isNavigating.value),
    navigationState: computed(() => navigationManager.getNavigationState())
  };
} 