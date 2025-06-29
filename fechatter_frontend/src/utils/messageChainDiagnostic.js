/**
 * Message Chain Diagnostic
 * 消息调用链深度诊断 - 追踪整个消息处理流程找出排序问题根源
 */

class MessageChainDiagnostic {
  constructor() {
    this.traceLog = [];
    this.originalMethods = {};
    this.isTracing = false;

    if (import.meta.env.DEV) {
      console.log('Message Chain Diagnostic initialized');
    }

  /**
   * Start comprehensive chain tracing
   * 开始全面的调用链追踪
   */
  startTracing() {
    if (import.meta.env.DEV) {
      console.log('\nSTARTING MESSAGE CHAIN TRACING');
    if (import.meta.env.DEV) {
      console.log('================================\n');
    }

    this.isTracing = true;
    this.traceLog = [];

    // Hook into all relevant methods
    this.hookChatStoreMethods();
    this.hookAPIResponse();
    this.hookMessageNormalization();
    this.hookUIUpdates();

    if (import.meta.env.DEV) {
      console.log('Tracing hooks installed');
    if (import.meta.env.DEV) {
      console.log('Send a message or switch chats to see the full trace');
    }

  /**
   * Stop tracing and generate report
   */
  stopTracing() {
    if (import.meta.env.DEV) {
      console.log('\n🛑 STOPPING MESSAGE CHAIN TRACING');
    if (import.meta.env.DEV) {
      console.log('=================================\n');
    }

    this.isTracing = false;
    this.restoreOriginalMethods();
    this.generateChainReport();
  }

  /**
   * Hook into chat store methods
   */
  hookChatStoreMethods() {
    const chatStore = this.getChatStore();
    if (!chatStore) return;

    // Hook fetchMessagesWithSignal
    if (chatStore.fetchMessagesWithSignal && !this.originalMethods.fetchMessagesWithSignal) {
      this.originalMethods.fetchMessagesWithSignal = chatStore.fetchMessagesWithSignal;

      chatStore.fetchMessagesWithSignal = async (...args) => {
        this.log('STORE', 'fetchMessagesWithSignal', `Started for chat ${args[0]}`);

        const result = await this.originalMethods.fetchMessagesWithSignal.apply(chatStore, args);

        if (result && result.length > 0) {
          this.log('STORE', 'fetchMessagesWithSignal', `Returned ${result.length} messages`);
          this.analyzeMessageOrder(result, 'API Response');
        }

        return result;
      };
    }

    // Hook normalizeMessage
    if (chatStore.normalizeMessage && !this.originalMethods.normalizeMessage) {
      this.originalMethods.normalizeMessage = chatStore.normalizeMessage;

      chatStore.normalizeMessage = (message) => {
        const result = this.originalMethods.normalizeMessage.apply(chatStore, [message]);

        if (result) {
          this.log('STORE', 'normalizeMessage', `Normalized message ${result.id} - ${result.created_at}`);
        }

        return result;
      };
    }

    // Hook messages setter
    this.hookMessagesProperty(chatStore);
  }

  /**
   * Hook the messages property to track updates
   */
  hookMessagesProperty(chatStore) {
    if (chatStore._messagesHooked) return;

    let internalMessages = chatStore.messages || [];
    const diagnosticInstance = this;

    Object.defineProperty(chatStore, 'messages', {
      get() {
        return internalMessages;
      },
      set(newMessages) {
        if (diagnosticInstance.isTracing) {
          diagnosticInstance.log('STORE', 'messages.setter', `Setting ${newMessages.length} messages`);

          if (newMessages.length > 0) {
            diagnosticInstance.analyzeMessageOrder(newMessages, 'Store Assignment');
          }

        internalMessages = newMessages;
      },
      configurable: true,
      enumerable: true
    });

    chatStore._messagesHooked = true;
  }

  /**
   * Hook API response processing
   */
  hookAPIResponse() {
    // Hook fetch to intercept API responses
    if (!this.originalMethods.fetch) {
      this.originalMethods.fetch = window.fetch;

      window.fetch = async (...args) => {
        const response = await this.originalMethods.fetch.apply(window, args);

        // Only trace message API calls
        if (args[0] && args[0].includes('/messages')) {
          this.log('API', 'fetch', `Response for ${args[0]}`);

          // Clone response to read it
          const clonedResponse = response.clone();
          try {
            const data = await clonedResponse.json();
            const messages = data.data || data || [];

            if (Array.isArray(messages) && messages.length > 0) {
              this.analyzeMessageOrder(messages, 'Raw API Response');
            }
          } catch (error) {
            // Ignore JSON parse errors
          }

        return response;
      };
    }

  /**
   * Hook message normalization
   */
  hookMessageNormalization() {
    // This is already covered in hookChatStoreMethods
  }

  /**
   * Hook UI updates (Vue reactivity)
   */
  hookUIUpdates() {
    // Monitor DOM mutations for message list changes
    const observer = new MutationObserver((mutations) => {
      if (!this.isTracing) return;

      mutations.forEach((mutation) => {
        if (mutation.type === 'childList' &&
          mutation.target.classList &&
          (mutation.target.classList.contains('messages-stable') ||
            mutation.target.classList.contains('message-list'))) {

          this.log('UI', 'DOM', `Message DOM updated - ${mutation.addedNodes.length} added, ${mutation.removedNodes.length} removed`);

          // Analyze current DOM order
          this.analyzeDOMMessageOrder();
        }
      });
    });

    // Start observing the document
    observer.observe(document.body, {
      childList: true,
      subtree: true
    });

    this.domObserver = observer;
  }

  /**
   * Analyze message order at different stages
   */
  analyzeMessageOrder(messages, stage) {
    if (!messages || messages.length < 2) return;

    const first = messages[0];
    const last = messages[messages.length - 1];

    const firstTime = new Date(first.created_at).getTime();
    const lastTime = new Date(last.created_at).getTime();

    let orderType = 'unknown';
    if (firstTime < lastTime) {
      orderType = 'CHRONOLOGICAL (oldest→newest) ';
    } else if (firstTime > lastTime) {
      orderType = 'REVERSE (newest→oldest) ❌';
    } else {
      orderType = 'SAME_TIME';
    }

    this.log('ORDER', stage, `${messages.length} messages - ${orderType}`);
    this.log('ORDER', stage, `First: ${this.formatTime(first.created_at)} | Last: ${this.formatTime(last.created_at)}`);

    // Detailed analysis for small arrays
    if (messages.length <= 10) {
      messages.forEach((msg, index) => {
        this.log('ORDER', `${stage}[${index}]`, `${this.formatTime(msg.created_at)} - "${msg.content?.substring(0, 30)}..."`);
      });

  /**
   * Analyze DOM message order
   */
  analyzeDOMMessageOrder() {
    const messageElements = document.querySelectorAll('[data-message-id]');

    if (messageElements.length < 2) return;

    const domMessages = [];
    messageElements.forEach((el) => {
      const messageId = el.getAttribute('data-message-id');
      const timeElement = el.querySelector('.message-timestamp, [class*="timestamp"]');
      const contentElement = el.querySelector('.message-content, [class*="content"]');

      if (timeElement) {
        domMessages.push({
          id: messageId,
          timestamp: timeElement.textContent || timeElement.title,
          content: contentElement?.textContent?.substring(0, 30) || 'No content'
        });
    });

    if (domMessages.length > 0) {
      this.log('UI', 'DOM Order', `${domMessages.length} messages in DOM`);
      domMessages.forEach((msg, index) => {
        this.log('UI', `DOM[${index}]`, `${msg.timestamp} - "${msg.content}..."`);
      });

  /**
   * Log trace entry
   */
  log(category, method, message) {
    if (!this.isTracing) return;

    const timestamp = new Date().toISOString().substring(11, 23);
    const entry = {
      timestamp,
      category,
      method,
      message
    };

    this.traceLog.push(entry);
    if (import.meta.env.DEV) {
      console.log(`[${timestamp}] ${category}:${method} - ${message}`);
    }

  /**
   * Generate comprehensive chain report
   */
  generateChainReport() {
    if (import.meta.env.DEV) {
      console.log('\nMESSAGE CHAIN ANALYSIS REPORT');
    if (import.meta.env.DEV) {
      console.log('=================================\n');
    }

    if (this.traceLog.length === 0) {
      if (import.meta.env.DEV) {
        console.log('WARNING: No trace data collected');
      return;
    }

    // Group by category
    const categories = {};
    this.traceLog.forEach(entry => {
      if (!categories[entry.category]) {
        categories[entry.category] = [];
      categories[entry.category].push(entry);
    });

    // Analyze each category
    Object.keys(categories).forEach(category => {
      if (import.meta.env.DEV) {
        console.log(`\n📋 ${category} ANALYSIS:`);
      if (import.meta.env.DEV) {
        console.log(`${'='.repeat(category.length + 10)}`);
      }

      categories[category].forEach(entry => {
        if (import.meta.env.DEV) {
          console.log(`   ${entry.timestamp} ${entry.method}: ${entry.message}`);
        }
      });
    });

    // Find ordering issues
    this.identifyOrderingIssues();

    // Generate recommendations
    this.generateRecommendations();
  }

  /**
   * Identify where ordering goes wrong
   */
  identifyOrderingIssues() {
    if (import.meta.env.DEV) {
      console.log('\n🚨 ORDERING ISSUE ANALYSIS:');
    if (import.meta.env.DEV) {
      console.log('===========================');
    }

    const orderEntries = this.traceLog.filter(entry => entry.category === 'ORDER');

    if (orderEntries.length === 0) {
      if (import.meta.env.DEV) {
        console.log('WARNING: No order analysis data found');
      return;
    }

    // Track order changes through the pipeline
    const stages = {};
    orderEntries.forEach(entry => {
      const stage = entry.method;
      if (!stages[stage]) {
        stages[stage] = [];
      stages[stage].push(entry);
    });

    if (import.meta.env.DEV) {
      console.log('\nOrder changes through pipeline:');
    Object.keys(stages).forEach(stage => {
      const stageEntries = stages[stage];
      const orderInfo = stageEntries.find(e => e.message.includes('CHRONOLOGICAL') || e.message.includes('REVERSE'));

      if (orderInfo) {
        const isCorrect = orderInfo.message.includes('');
        const status = isCorrect ? 'CORRECT' : 'ERROR: WRONG';
        if (import.meta.env.DEV) {
          console.log(`   ${stage}: ${status}`);
        }

        if (!isCorrect) {
          if (import.meta.env.DEV) {
            console.log(`      Issue: ${orderInfo.message}`);
          }
    });

  /**
   * Generate fix recommendations
   */
  generateRecommendations() {
    if (import.meta.env.DEV) {
      console.log('\nFIX RECOMMENDATIONS:');
    if (import.meta.env.DEV) {
      console.log('========================');
    }

    const hasAPIIssue = this.traceLog.some(e =>
      e.category === 'ORDER' &&
      e.method.includes('API') &&
      e.message.includes('REVERSE')
    );

    const hasStoreIssue = this.traceLog.some(e =>
      e.category === 'ORDER' &&
      e.method.includes('Store') &&
      e.message.includes('REVERSE')
    );

    const hasDOMIssue = this.traceLog.some(e =>
      e.category === 'UI' &&
      e.method.includes('DOM')
    );

    if (hasAPIIssue) {
      if (import.meta.env.DEV) {
        console.log('BACKEND ISSUE: API returns messages in wrong order');
      if (import.meta.env.DEV) {
        console.log('   Fix: Backend should sort by created_at ASC');
      if (import.meta.env.DEV) {
        console.log('   Temp fix: Sort on frontend after API response');
      }

    if (hasStoreIssue) {
      if (import.meta.env.DEV) {
        console.log('STORE ISSUE: Messages sorted incorrectly in store');
      if (import.meta.env.DEV) {
        console.log('   Fix: Update chat store sorting logic');
      if (import.meta.env.DEV) {
        console.log('   Code: messages.sort((a,b) => new Date(a.created_at) - new Date(b.created_at))');
      }

    if (hasDOMIssue) {
      if (import.meta.env.DEV) {
        console.log('UI ISSUE: DOM rendering problem');
      if (import.meta.env.DEV) {
        console.log('   Fix: Check Vue component message rendering order');
      }

    if (import.meta.env.DEV) {
      console.log('\nIMMEDIATE ACTION:');
    if (import.meta.env.DEV) {
      console.log('   1. Run: window.messageChain.fixIdentifiedIssues()');
    if (import.meta.env.DEV) {
      console.log('   2. Test: Send a message and verify order');
    if (import.meta.env.DEV) {
      console.log('   3. Verify: Check all stages show CORRECT');
    }

  /**
   * Apply fixes based on identified issues
   */
  fixIdentifiedIssues() {
    if (import.meta.env.DEV) {
      console.log('\nAPPLYING CHAIN FIXES');
    if (import.meta.env.DEV) {
      console.log('=======================');
    }

    const chatStore = this.getChatStore();
    if (!chatStore) {
      if (import.meta.env.DEV) {
        console.log('ERROR: Cannot fix - chat store not available');
      return;
    }

    let fixCount = 0;

    // Fix 1: Sort current messages
    if (chatStore.messages && chatStore.messages.length > 0) {
      if (import.meta.env.DEV) {
        console.log('Fixing current messages...');
      chatStore.messages.sort((a, b) => {
        const timeA = new Date(a.created_at).getTime();
        const timeB = new Date(b.created_at).getTime();
        return timeA - timeB; // Ascending order
      });
      fixCount++;
      if (import.meta.env.DEV) {
        console.log('Current messages sorted chronologically');
      }

    // Fix 2: Fix message cache
    if (chatStore.messageCache) {
      if (import.meta.env.DEV) {
        console.log('Fixing message cache...');
      Object.keys(chatStore.messageCache).forEach(chatId => {
        const cache = chatStore.messageCache[chatId];
        if (cache && cache.messages) {
          cache.messages.sort((a, b) => {
            const timeA = new Date(a.created_at).getTime();
            const timeB = new Date(b.created_at).getTime();
            return timeA - timeB;
          });
      });
      fixCount++;
      if (import.meta.env.DEV) {
        console.log('Message cache sorted chronologically');
      }

    // Fix 3: Install permanent sorting interceptor
    this.installPermanentSortingFix(chatStore);
    fixCount++;

    if (import.meta.env.DEV) {
      console.log(`\nApplied ${fixCount} fixes to the message chain`);
    if (import.meta.env.DEV) {
      console.log('🔄 Refresh the page to see the corrected order');
    }

  /**
   * Install permanent sorting fix
   */
  installPermanentSortingFix(chatStore) {
    if (import.meta.env.DEV) {
      console.log('Installing permanent sorting interceptor...');
    }

    // Hook into fetchMessagesWithSignal permanently
    if (chatStore.fetchMessagesWithSignal && !chatStore._sortingFixInstalled) {
      const originalFetch = chatStore.fetchMessagesWithSignal;

      chatStore.fetchMessagesWithSignal = async function (...args) {
        const result = await originalFetch.apply(this, args);

        // Always sort messages chronologically after fetch
        if (result && Array.isArray(result) && result.length > 1) {
          result.sort((a, b) => {
            const timeA = new Date(a.created_at).getTime();
            const timeB = new Date(b.created_at).getTime();
            return timeA - timeB;
          });

          if (import.meta.env.DEV) {
            console.log('Auto-sorted fetched messages chronologically');
          }

        return result;
      };

      chatStore._sortingFixInstalled = true;
      if (import.meta.env.DEV) {
        console.log('Permanent sorting fix installed');
      }

  /**
   * Restore original methods
   */
  restoreOriginalMethods() {
    const chatStore = this.getChatStore();

    // Restore chat store methods
    if (chatStore && this.originalMethods.fetchMessagesWithSignal) {
      chatStore.fetchMessagesWithSignal = this.originalMethods.fetchMessagesWithSignal;
      delete this.originalMethods.fetchMessagesWithSignal;
    }

    if (chatStore && this.originalMethods.normalizeMessage) {
      chatStore.normalizeMessage = this.originalMethods.normalizeMessage;
      delete this.originalMethods.normalizeMessage;
    }

    // Restore fetch
    if (this.originalMethods.fetch) {
      window.fetch = this.originalMethods.fetch;
      delete this.originalMethods.fetch;
    }

    // Stop DOM observer
    if (this.domObserver) {
      this.domObserver.disconnect();
      this.domObserver = null;
    }

  /**
   * Format timestamp for display
   */
  formatTime(timestamp) {
    try {
      const date = new Date(timestamp);
      return date.toLocaleTimeString();
    } catch (error) {
      return timestamp;
    }

  /**
   * Get chat store reference
   */
  getChatStore() {
    try {
      return window.app?._instance?.proxy?.$pinia?._s?.get('chat');
    } catch (error) {
      return null;
    }

// Create global instance
const messageChainDiagnostic = new MessageChainDiagnostic();

// Expose to window
if (typeof window !== 'undefined') {
  window.messageChain = {
    start: () => messageChainDiagnostic.startTracing(),
    stop: () => messageChainDiagnostic.stopTracing(),
    fix: () => messageChainDiagnostic.fixIdentifiedIssues(),
    analyze: () => {
      messageChainDiagnostic.startTracing();
      if (import.meta.env.DEV) {
        console.log('Tracing started - switch chats or send messages to collect data');
      if (import.meta.env.DEV) {
        console.log('⏹️ Run window.messageChain.stop() when ready to analyze');
      }
  };

  if (import.meta.env.DEV) {
    console.log('Message Chain Diagnostic loaded');
  if (import.meta.env.DEV) {
    console.log('   Commands:');
  if (import.meta.env.DEV) {
    console.log('   - window.messageChain.start() - Start tracing');
  if (import.meta.env.DEV) {
    console.log('   - window.messageChain.stop() - Stop and analyze');
  if (import.meta.env.DEV) {
    console.log('   - window.messageChain.fix() - Apply fixes');
  if (import.meta.env.DEV) {
    console.log('   - window.messageChain.analyze() - Quick analyze');
  }

export default messageChainDiagnostic; 