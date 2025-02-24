/**
 * First Principle Message Optimization System
 * 第一性原理消息优化系统 - 最短路径消息传递
 */

class FirstPrincipleMessageOptimization {
  constructor() {
    this.messageCache = new Map(); // In-memory ultra-fast cache
    this.pendingMessages = new Map(); // Pending message tracking
    this.optimizedAPI = null;
    this.directPersistence = null;

    if (import.meta.env.DEV) {
      console.log('🚀 First Principle Message Optimization initialized');
    }

  /**
   * Initialize ultra-optimized message system
   * 初始化超优化消息系统
   */
  initialize() {
    this.setupOptimizedAPI();
    this.setupDirectPersistence();
    this.setupInstantUI();
    this.setupBypassChain();

    if (import.meta.env.DEV) {
      console.log('⚡ Ultra-optimized message system ready');
    }

  /**
   * Setup optimized API with minimal overhead
   * 设置最小开销的优化API
   */
  setupOptimizedAPI() {
    this.optimizedAPI = {
      // Direct fetch without middleware overhead
      fastFetch: async (url, options = {}) => {
        const startTime = performance.now();

        try {
          const response = await fetch(url, {
            ...options,
            headers: {
              'Content-Type': 'application/json',
              'Accept': 'application/json',
              // Bypass unnecessary headers
              ...options.headers
            }
          });

          const endTime = performance.now();
          if (import.meta.env.DEV) {
            console.log(`⚡ Fast API call: ${endTime - startTime}ms`);
          }

          return response;
        } catch (error) {
          if (import.meta.env.DEV) {
            console.error('❌ Fast API failed:', error);
          throw error;
        }
      },

      // Optimized message sending
      sendMessageDirect: async (chatId, content) => {
        const url = `${window.location.origin}/api/chat/${chatId}/messages`;

        return this.fastFetch(url, {
          method: 'POST',
          body: JSON.stringify({
            content,
            message_type: 'text'
          })
        });
      },

      // Optimized message fetching
      fetchMessagesDirect: async (chatId, limit = 50) => {
        const url = `${window.location.origin}/api/chat/${chatId}/messages?limit=${limit}`;
        return this.fastFetch(url);
      }
    };
  }

  /**
   * Setup direct persistence bypassing layers
   * 设置绕过层级的直接持久化
   */
  setupDirectPersistence() {
    this.directPersistence = {
      // Ultra-fast localStorage operations
      save: (key, data) => {
        try {
          const serialized = JSON.stringify(data);
          localStorage.setItem(key, serialized);
          return true;
        } catch (error) {
          if (import.meta.env.DEV) {
            console.warn('Direct persistence save failed:', error);
          return false;
        }
      },

      get: (key) => {
        try {
          const item = localStorage.getItem(key);
          return item ? JSON.parse(item) : null;
        } catch (error) {
          if (import.meta.env.DEV) {
            console.warn('Direct persistence get failed:', error);
          return null;
        }
      },

      // Chat-specific operations
      saveMessages: (chatId, messages) => {
        const key = `fast_chat_${chatId}`;
        return this.save(key, {
          messages,
          timestamp: Date.now(),
          chatId
        });
      },

      getMessages: (chatId) => {
        const key = `fast_chat_${chatId}`;
        const data = this.get(key);
        return data ? data.messages : [];
      }
    };
  }

  /**
   * Setup instant UI updates
   * 设置即时UI更新
   */
  setupInstantUI() {
    this.instantUI = {
      // Add message to UI instantly
      addMessage: (chatId, message) => {
        const chatStore = this.getChatStore();
        if (chatStore && chatStore.currentChatId === chatId) {
          chatStore.messages.push(message);

          // Update cache instantly
          this.messageCache.set(chatId, [...chatStore.messages]);

          // Persist instantly
          this.directPersistence.saveMessages(chatId, chatStore.messages);
        }
      },

      // Replace optimistic message with real one
      replaceMessage: (chatId, tempId, realMessage) => {
        const chatStore = this.getChatStore();
        if (chatStore && chatStore.currentChatId === chatId) {
          const index = chatStore.messages.findIndex(m =>
            m.temp_id === tempId || m.id === tempId
          );

          if (index !== -1) {
            chatStore.messages.splice(index, 1, realMessage);

            // Update cache and persistence
            this.messageCache.set(chatId, [...chatStore.messages]);
            this.directPersistence.saveMessages(chatId, chatStore.messages);
          }
      },

      // Load messages instantly on chat switch
      loadMessagesInstantly: (chatId) => {
        const chatStore = this.getChatStore();
        if (!chatStore) return;

        // Try memory cache first (fastest)
        if (this.messageCache.has(chatId)) {
          chatStore.messages = [...this.messageCache.get(chatId)];
          chatStore.loading = false;
          if (import.meta.env.DEV) {
            console.log(`⚡ Loaded from memory cache: ${chatStore.messages.length} messages`);
          return;
        }

        // Try localStorage (fast)
        const persistedMessages = this.directPersistence.getMessages(chatId);
        if (persistedMessages.length > 0) {
          chatStore.messages = [...persistedMessages];
          chatStore.loading = false;
          this.messageCache.set(chatId, persistedMessages);
          if (import.meta.env.DEV) {
            console.log(`📦 Loaded from localStorage: ${persistedMessages.length} messages`);
          return;
        }

        // No cache available, will need to fetch
        chatStore.messages = [];
        chatStore.loading = true;
      }
    };
  }

  /**
   * Setup bypass chain - shortest path messaging
   * 设置绕过链 - 最短路径消息传递
   */
  setupBypassChain() {
    this.bypassChain = {
      // Ultra-optimized send message
      sendMessageUltraFast: async (chatId, content) => {
        const startTime = performance.now();

        // 1. Create optimistic message instantly
        const tempId = `temp-${Date.now()}-${Math.random()}`;
        const optimisticMessage = {
          id: tempId,
          temp_id: tempId,
          content: content.trim(),
          sender_id: this.getCurrentUserId(),
          sender: this.getCurrentUserInfo(),
          created_at: new Date().toISOString(),
          chat_id: chatId,
          status: 'sent',
          isOptimistic: true,
          _normalized: true,
          _timestamp: Date.now()
        };

        // 2. Add to UI instantly (0ms delay)
        this.instantUI.addMessage(chatId, optimisticMessage);

        // Track pending message
        this.pendingMessages.set(tempId, {
          chatId,
          content,
          startTime,
          optimisticMessage
        });

        if (import.meta.env.DEV) {
          console.log(`⚡ Optimistic message added in ${performance.now() - startTime}ms`);
        }

        // 3. Send to backend in parallel (non-blocking)
        this.sendToBackendAsync(chatId, content, tempId);

        return optimisticMessage;
      },

      // Async backend sending
      sendToBackendAsync: async (chatId, content, tempId) => {
        try {
          const response = await this.optimizedAPI.sendMessageDirect(chatId, content);
          const data = await response.json();

          const realMessage = data.data || data;
          if (realMessage) {
            // Replace optimistic with real message
            this.instantUI.replaceMessage(chatId, tempId, realMessage);

            // Clean up pending
            this.pendingMessages.delete(tempId);

            const pending = this.pendingMessages.get(tempId);
            if (pending) {
              const totalTime = performance.now() - pending.startTime;
              if (import.meta.env.DEV) {
                console.log(`✅ Message sent successfully in ${totalTime}ms`);
              }
        } catch (error) {
          if (import.meta.env.DEV) {
            console.error('❌ Backend send failed:', error);
          }

          // Mark message as failed
          const chatStore = this.getChatStore();
          if (chatStore) {
            const msg = chatStore.messages.find(m => m.temp_id === tempId);
            if (msg) {
              msg.status = 'failed';
              msg.error = error.message;
            }

          this.pendingMessages.delete(tempId);
        }
      },

      // Ultra-fast chat loading
      loadChatUltraFast: async (chatId) => {
        const startTime = performance.now();

        // 1. Load messages instantly from cache
        this.instantUI.loadMessagesInstantly(chatId);

        if (import.meta.env.DEV) {
          console.log(`⚡ Chat loaded instantly in ${performance.now() - startTime}ms`);
        }

        // 2. Fetch latest in background
        this.fetchLatestInBackground(chatId);
      },

      // Background fetching without blocking UI
      fetchLatestInBackground: async (chatId) => {
        try {
          const response = await this.optimizedAPI.fetchMessagesDirect(chatId);
          const data = await response.json();

          const messages = data.data || data || [];
          if (messages.length > 0) {
            // Update cache and persistence
            this.messageCache.set(chatId, messages);
            this.directPersistence.saveMessages(chatId, messages);

            // Update UI if still on this chat
            const chatStore = this.getChatStore();
            if (chatStore && chatStore.currentChatId === chatId) {
              chatStore.messages = [...messages];
            }

            if (import.meta.env.DEV) {
              console.log(`🔄 Background sync completed: ${messages.length} messages`);
            }
        } catch (error) {
          if (import.meta.env.DEV) {
            console.warn('Background fetch failed:', error);
          }
    };
  }

  /**
   * Get chat store reference
   */
  getChatStore() {
    try {
      return window.app?._instance?.proxy?.$pinia?._s?.get('chat');
    } catch (error) {
      if (import.meta.env.DEV) {
        console.warn('Failed to get chat store:', error);
      return null;
    }

  /**
   * Get current user ID
   */
  getCurrentUserId() {
    try {
      const authStore = window.app?._instance?.proxy?.$pinia?._s?.get('auth');
      return authStore?.user?.id || 1;
    } catch (error) {
      return 1;
    }

  /**
   * Get current user info
   */
  getCurrentUserInfo() {
    try {
      const authStore = window.app?._instance?.proxy?.$pinia?._s?.get('auth');
      const user = authStore?.user;
      return {
        id: user?.id || 1,
        fullname: user?.fullname || 'You',
        email: user?.email || '',
        avatar_url: user?.avatar_url || null
      };
    } catch (error) {
      return {
        id: 1,
        fullname: 'You',
        email: '',
        avatar_url: null
      };
    }

  /**
   * Performance diagnostics
   */
  diagnosePerformance() {
    if (import.meta.env.DEV) {
      console.log('\n⚡ FIRST PRINCIPLE PERFORMANCE DIAGNOSTICS');
    if (import.meta.env.DEV) {
      console.log('==========================================');
    }

    // Memory cache stats
    if (import.meta.env.DEV) {
      console.log(`📊 Memory cache: ${this.messageCache.size} chats cached`);
    }

    // Pending messages
    if (import.meta.env.DEV) {
      console.log(`⏳ Pending messages: ${this.pendingMessages.size} messages`);
    }

    // localStorage usage
    const usage = this.getStorageUsage();
    if (import.meta.env.DEV) {
      console.log(`💾 localStorage usage: ${usage.used}KB / ${usage.quota}KB`);
    }

    // Test basic operations
    this.testPerformance();
  }

  /**
   * Get storage usage statistics
   */
  getStorageUsage() {
    try {
      let used = 0;
      for (let key in localStorage) {
        if (localStorage.hasOwnProperty(key)) {
          used += localStorage[key].length;
        }

      return {
        used: Math.round(used / 1024),
        quota: 5 * 1024 // ~5MB typical quota
      };
    } catch (error) {
      return { used: 0, quota: 0 };
    }

  /**
   * Test performance of core operations
   */
  testPerformance() {
    if (import.meta.env.DEV) {
      console.log('\n🔬 Testing core operation performance:');
    }

    // Test localStorage save/get
    const testData = { test: 'performance', timestamp: Date.now() };

    const saveStart = performance.now();
    this.directPersistence.save('perf_test', testData);
    const saveTime = performance.now() - saveStart;

    const getStart = performance.now();
    const retrieved = this.directPersistence.get('perf_test');
    const getTime = performance.now() - getStart;

    if (import.meta.env.DEV) {
      console.log(`   💾 localStorage save: ${saveTime.toFixed(2)}ms`);
    if (import.meta.env.DEV) {
      console.log(`   📖 localStorage get: ${getTime.toFixed(2)}ms`);
    }

    // Clean up test data
    localStorage.removeItem('perf_test');

    // Performance targets
    if (saveTime < 1 && getTime < 1) {
      if (import.meta.env.DEV) {
        console.log('   ✅ Performance: EXCELLENT');
      }
    } else if (saveTime < 5 && getTime < 5) {
      if (import.meta.env.DEV) {
        console.log('   👍 Performance: GOOD');
      }
    } else {
      if (import.meta.env.DEV) {
        console.log('   ⚠️ Performance: NEEDS OPTIMIZATION');
      }

  /**
   * Replace existing message system
   */
  replaceExistingSystem() {
    const chatStore = this.getChatStore();
    if (!chatStore) {
      if (import.meta.env.DEV) {
        console.error('❌ Cannot replace system - chat store not found');
      return;
    }

    // Backup original methods
    chatStore._originalSendMessage = chatStore.sendMessage;
    chatStore._originalSetCurrentChat = chatStore.setCurrentChat;

    // Replace with ultra-fast versions
    chatStore.sendMessage = (chatId, messageData) => {
      return this.bypassChain.sendMessageUltraFast(chatId, messageData.content);
    };

    chatStore.setCurrentChat = (chatId) => {
      chatStore.currentChatId = chatId;
      return this.bypassChain.loadChatUltraFast(chatId);
    };

    if (import.meta.env.DEV) {
      console.log('🔄 Message system replaced with ultra-fast version');
    }

  /**
   * Restore original system
   */
  restoreOriginalSystem() {
    const chatStore = this.getChatStore();
    if (!chatStore) return;

    if (chatStore._originalSendMessage) {
      chatStore.sendMessage = chatStore._originalSendMessage;
      delete chatStore._originalSendMessage;
    }

    if (chatStore._originalSetCurrentChat) {
      chatStore.setCurrentChat = chatStore._originalSetCurrentChat;
      delete chatStore._originalSetCurrentChat;
    }

    if (import.meta.env.DEV) {
      console.log('🔄 Original message system restored');
    }

// Create global instance
const firstPrincipleOptimizer = new FirstPrincipleMessageOptimization();

// Expose to window for debugging and control
if (typeof window !== 'undefined') {
  window.ultraFast = {
    init: () => firstPrincipleOptimizer.initialize(),
    replace: () => firstPrincipleOptimizer.replaceExistingSystem(),
    restore: () => firstPrincipleOptimizer.restoreOriginalSystem(),
    diagnose: () => firstPrincipleOptimizer.diagnosePerformance(),
    send: (chatId, content) => firstPrincipleOptimizer.bypassChain.sendMessageUltraFast(chatId, content),
    load: (chatId) => firstPrincipleOptimizer.bypassChain.loadChatUltraFast(chatId)
  };

  if (import.meta.env.DEV) {
    console.log('⚡ Ultra-Fast Message System loaded');
  if (import.meta.env.DEV) {
    console.log('   Commands:');
  if (import.meta.env.DEV) {
    console.log('   - window.ultraFast.init() - Initialize ultra-fast system');
  if (import.meta.env.DEV) {
    console.log('   - window.ultraFast.replace() - Replace existing system');
  if (import.meta.env.DEV) {
    console.log('   - window.ultraFast.restore() - Restore original system');
  if (import.meta.env.DEV) {
    console.log('   - window.ultraFast.diagnose() - Performance diagnostics');
  }

export default firstPrincipleOptimizer; 