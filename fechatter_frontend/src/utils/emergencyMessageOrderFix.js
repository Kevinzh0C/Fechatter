/**
 * Emergency Message Order Fix
 * 紧急消息排序修复 - 处理API连接失败和缓存数据排序
 */

class EmergencyMessageOrderFix {
  constructor() {
    this.hasAppliedFix = false;
    console.log('🚨 Emergency Message Order Fix initialized');
  }

  /**
   * Complete emergency fix for message ordering
   * 完整的紧急消息排序修复
   */
  applyCompleteFix() {
    console.log('\n🚨 APPLYING EMERGENCY MESSAGE ORDER FIX');
    console.log('========================================');

    let fixCount = 0;

    // Fix 1: Chat Store Messages
    fixCount += this.fixChatStoreMessages();

    // Fix 2: LocalStorage Cache
    fixCount += this.fixLocalStorageCache();

    // Fix 3: Session Storage
    fixCount += this.fixSessionStorage();

    // Fix 4: Install Permanent Interceptors
    fixCount += this.installPermanentInterceptors();

    // Fix 5: Fix Current DOM
    fixCount += this.fixCurrentDOM();

    console.log(`\n✅ Applied ${fixCount} emergency fixes`);
    console.log('🔄 Refresh the page to see all fixes applied');

    this.hasAppliedFix = true;
    return fixCount;
  }

  /**
   * Fix chat store messages
   */
  fixChatStoreMessages() {
    console.log('🔧 Fixing Chat Store messages...');

    try {
      const chatStore = this.getChatStore();
      if (!chatStore) {
        console.log('⚠️ Chat store not found');
        return 0;
      }

      let fixCount = 0;

      // Fix current messages
      if (chatStore.messages && Array.isArray(chatStore.messages) && chatStore.messages.length > 0) {
        const originalOrder = chatStore.messages.map(m => this.formatTime(m.created_at)).join(' → ');

        chatStore.messages.sort((a, b) => {
          const timeA = new Date(a.created_at).getTime();
          const timeB = new Date(b.created_at).getTime();
          return timeA - timeB; // Ascending: oldest first
        });

        const newOrder = chatStore.messages.map(m => this.formatTime(m.created_at)).join(' → ');

        console.log(`   Original: ${originalOrder}`);
        console.log(`   Fixed:    ${newOrder}`);
        fixCount++;
      }

      // Fix message cache
      if (chatStore.messageCache && typeof chatStore.messageCache === 'object') {
        Object.keys(chatStore.messageCache).forEach(chatId => {
          const cache = chatStore.messageCache[chatId];
          if (cache && cache.messages && Array.isArray(cache.messages)) {
            cache.messages.sort((a, b) => {
              const timeA = new Date(a.created_at).getTime();
              const timeB = new Date(b.created_at).getTime();
              return timeA - timeB;
            });
            fixCount++;
          }
        });
        console.log(`   Fixed ${Object.keys(chatStore.messageCache).length} cached chats`);
      }

      console.log(`✅ Chat store: ${fixCount} fixes applied`);
      return fixCount;

    } catch (error) {
      console.error('❌ Error fixing chat store:', error);
      return 0;
    }
  }

  /**
   * Fix localStorage cache
   */
  fixLocalStorageCache() {
    console.log('🔧 Fixing localStorage cache...');

    try {
      let fixCount = 0;

      // Check for various possible cache keys
      const possibleKeys = [
        'fechatter_messages',
        'chat_messages',
        'messages_cache',
        'messageCache',
        'vuex_messages',
        'pinia_chat'
      ];

      possibleKeys.forEach(key => {
        try {
          const cached = localStorage.getItem(key);
          if (cached) {
            const data = JSON.parse(cached);

            if (this.fixCachedData(data, key)) {
              localStorage.setItem(key, JSON.stringify(data));
              fixCount++;
            }
          }
        } catch (error) {
          // Ignore individual key errors
        }
      });

      console.log(`✅ localStorage: ${fixCount} caches fixed`);
      return fixCount;

    } catch (error) {
      console.error('❌ Error fixing localStorage:', error);
      return 0;
    }
  }

  /**
   * Fix sessionStorage
   */
  fixSessionStorage() {
    console.log('🔧 Fixing sessionStorage...');

    try {
      let fixCount = 0;

      for (let i = 0; i < sessionStorage.length; i++) {
        const key = sessionStorage.key(i);
        if (key && (key.includes('message') || key.includes('chat'))) {
          try {
            const cached = sessionStorage.getItem(key);
            const data = JSON.parse(cached);

            if (this.fixCachedData(data, key)) {
              sessionStorage.setItem(key, JSON.stringify(data));
              fixCount++;
            }
          } catch (error) {
            // Ignore individual key errors
          }
        }
      }

      console.log(`✅ sessionStorage: ${fixCount} caches fixed`);
      return fixCount;

    } catch (error) {
      console.error('❌ Error fixing sessionStorage:', error);
      return 0;
    }
  }

  /**
   * Fix cached data structure
   */
  fixCachedData(data, key) {
    let fixed = false;

    if (Array.isArray(data)) {
      // Direct array of messages
      if (data.length > 0 && data[0].created_at) {
        data.sort((a, b) => {
          const timeA = new Date(a.created_at).getTime();
          const timeB = new Date(b.created_at).getTime();
          return timeA - timeB;
        });
        console.log(`   Fixed array in ${key}: ${data.length} messages`);
        fixed = true;
      }
    } else if (data && typeof data === 'object') {
      // Object containing messages
      Object.keys(data).forEach(prop => {
        if (Array.isArray(data[prop]) && data[prop].length > 0 && data[prop][0].created_at) {
          data[prop].sort((a, b) => {
            const timeA = new Date(a.created_at).getTime();
            const timeB = new Date(b.created_at).getTime();
            return timeA - timeB;
          });
          console.log(`   Fixed ${prop} in ${key}: ${data[prop].length} messages`);
          fixed = true;
        }
      });
    }

    return fixed;
  }

  /**
   * Install permanent interceptors
   */
  installPermanentInterceptors() {
    console.log('🔧 Installing permanent sorting interceptors...');

    try {
      let fixCount = 0;

      const chatStore = this.getChatStore();
      if (chatStore) {
        // Intercept fetchMessagesWithSignal
        if (chatStore.fetchMessagesWithSignal && !chatStore._emergencyFixInstalled) {
          const originalFetch = chatStore.fetchMessagesWithSignal;

          chatStore.fetchMessagesWithSignal = async function (...args) {
            let result;

            try {
              result = await originalFetch.apply(this, args);
            } catch (error) {
              console.log('🚨 API fetch failed, using emergency fallback');
              result = this.messages || [];
            }

            // Always sort result chronologically
            if (result && Array.isArray(result) && result.length > 1) {
              result.sort((a, b) => {
                const timeA = new Date(a.created_at).getTime();
                const timeB = new Date(b.created_at).getTime();
                return timeA - timeB;
              });
              console.log(`🔧 Auto-sorted ${result.length} messages chronologically`);
            }

            return result;
          };

          chatStore._emergencyFixInstalled = true;
          fixCount++;
        }

        // Intercept messages setter
        this.interceptMessagesSetter(chatStore);
        fixCount++;
      }

      console.log(`✅ Interceptors: ${fixCount} installed`);
      return fixCount;

    } catch (error) {
      console.error('❌ Error installing interceptors:', error);
      return 0;
    }
  }

  /**
   * Intercept messages setter
   */
  interceptMessagesSetter(chatStore) {
    if (chatStore._messageSetterIntercepted) return;

    let internalMessages = chatStore.messages || [];

    Object.defineProperty(chatStore, 'messages', {
      get() {
        return internalMessages;
      },
      set(newMessages) {
        if (newMessages && Array.isArray(newMessages) && newMessages.length > 1) {
          // Auto-sort before setting
          newMessages.sort((a, b) => {
            const timeA = new Date(a.created_at).getTime();
            const timeB = new Date(b.created_at).getTime();
            return timeA - timeB;
          });
          console.log('🔧 Auto-sorted messages on setter');
        }
        internalMessages = newMessages;
      },
      configurable: true,
      enumerable: true
    });

    chatStore._messageSetterIntercepted = true;
  }

  /**
   * Fix current DOM
   */
  fixCurrentDOM() {
    console.log('🔧 Analyzing current DOM...');

    try {
      const messageElements = document.querySelectorAll('[data-message-id], .message-item, .message');

      if (messageElements.length === 0) {
        console.log('   No message elements found in DOM');
        return 0;
      }

      console.log(`   Found ${messageElements.length} message elements`);

      // Analyze current DOM order
      const domMessages = [];
      messageElements.forEach((el, index) => {
        const timeElement = el.querySelector('.message-timestamp, [class*="timestamp"], .time');
        const contentElement = el.querySelector('.message-content, [class*="content"], .text');

        if (timeElement) {
          domMessages.push({
            index,
            element: el,
            timestamp: timeElement.textContent || timeElement.title || timeElement.getAttribute('title'),
            content: contentElement?.textContent?.substring(0, 20) || 'No content'
          });
        }
      });

      if (domMessages.length > 0) {
        console.log('   Current DOM order:');
        domMessages.forEach((msg, i) => {
          console.log(`     [${i}] ${msg.timestamp} - "${msg.content}..."`);
        });
      }

      console.log('✅ DOM: Analysis complete (no direct DOM manipulation)');
      return 1;

    } catch (error) {
      console.error('❌ Error analyzing DOM:', error);
      return 0;
    }
  }

  /**
   * Quick diagnostic
   */
  diagnose() {
    console.log('\n🔍 EMERGENCY DIAGNOSTIC');
    console.log('=======================');

    // Check Chat Store
    const chatStore = this.getChatStore();
    if (chatStore && chatStore.messages) {
      console.log(`📊 Chat Store: ${chatStore.messages.length} messages`);

      if (chatStore.messages.length > 1) {
        const first = chatStore.messages[0];
        const last = chatStore.messages[chatStore.messages.length - 1];
        const firstTime = new Date(first.created_at).getTime();
        const lastTime = new Date(last.created_at).getTime();

        if (firstTime < lastTime) {
          console.log('✅ Store order: CORRECT (oldest→newest)');
        } else {
          console.log('❌ Store order: WRONG (newest→oldest)');
        }
      }
    } else {
      console.log('⚠️ Chat Store: No messages found');
    }

    // Check localStorage
    const hasLocalStorage = localStorage.length > 0;
    console.log(`📦 localStorage: ${hasLocalStorage ? 'Has data' : 'Empty'}`);

    // Check API connectivity
    console.log('🌐 API Status: Checking...');
    fetch('/api/health').then(response => {
      console.log(`✅ API: Connected (${response.status})`);
    }).catch(error => {
      console.log('❌ API: Connection failed (using cached data)');
    });

    return this;
  }

  /**
   * Get chat store reference
   */
  getChatStore() {
    try {
      return window.app?._instance?.proxy?.$pinia?._s?.get('chat') ||
        window.app?._instance?.setupState?.chatStore ||
        window.$nuxt?.$store?.state?.chat;
    } catch (error) {
      return null;
    }
  }

  /**
   * Format time for display
   */
  formatTime(timestamp) {
    try {
      return new Date(timestamp).toLocaleTimeString();
    } catch (error) {
      return timestamp;
    }
  }
}

// Create global instance
const emergencyFix = new EmergencyMessageOrderFix();

// Export for use
export default emergencyFix;

// Expose to window for immediate use
if (typeof window !== 'undefined') {
  window.emergencyFix = {
    fix: () => emergencyFix.applyCompleteFix(),
    diagnose: () => emergencyFix.diagnose(),
    run: () => {
      emergencyFix.diagnose();
      return emergencyFix.applyCompleteFix();
    }
  };

  console.log('🚨 Emergency Message Order Fix loaded');
  console.log('   Commands:');
  console.log('   - window.emergencyFix.run() - Full diagnostic + fix');
  console.log('   - window.emergencyFix.fix() - Apply all fixes');
  console.log('   - window.emergencyFix.diagnose() - Check current state');
} 