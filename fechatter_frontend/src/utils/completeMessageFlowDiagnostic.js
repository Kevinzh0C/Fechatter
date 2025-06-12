/**
 * Complete Message Flow Diagnostic
 * 完整消息流程诊断 - 从UI点击到API调用到显示的全流程分析
 */

class CompleteMessageFlowDiagnostic {
  constructor() {
    this.traceLog = [];
    this.interceptors = [];
    this.isTracing = false;
    this.expectedOrder = null;

    console.log('🔬 Complete Message Flow Diagnostic initialized');
    console.log('📋 Will trace: UI Click → Store → API → Processing → Display');
  }

  /**
   * Start complete flow tracing
   * 开始完整流程追踪
   */
  startCompleteTrace() {
    console.log('\n🔬 STARTING COMPLETE MESSAGE FLOW TRACE');
    console.log('=======================================');
    console.log('🎯 Tracing from channel click to message display');
    console.log('📊 Will analyze every function and statement');

    this.isTracing = true;
    this.traceLog = [];
    this.expectedOrder = null;

    // Install all interceptors
    this.interceptUIEvents();
    this.interceptStoreActions();
    this.interceptAPIRequests();
    this.interceptDataProcessing();
    this.interceptUIRendering();

    console.log('✅ All interceptors installed');
    console.log('👆 Now click on a channel to start tracing...');

    return this;
  }

  /**
   * Intercept UI events (channel clicks)
   */
  interceptUIEvents() {
    console.log('🔧 Installing UI event interceptors...');

    // Intercept channel click events
    const originalAddEventListener = Element.prototype.addEventListener;
    Element.prototype.addEventListener = function (type, listener, options) {
      if (type === 'click' && this.classList.contains('channel-item')) {
        const wrappedListener = (event) => {
          this.logStep('UI_EVENT', 'channel_click', {
            channelId: this.getAttribute('data-channel-id'),
            timestamp: Date.now(),
            element: this.className
          });

          return listener.call(this, event);
        };

        return originalAddEventListener.call(this, type, wrappedListener, options);
      }

      return originalAddEventListener.call(this, type, listener, options);
    }.bind(this);

    // Intercept router navigation
    const router = window.$router;
    if (router) {
      const originalPush = router.push;
      router.push = function (location) {
        this.logStep('UI_NAVIGATION', 'router_push', {
          to: location,
          from: router.currentRoute.value.path,
          timestamp: Date.now()
        });

        return originalPush.call(this, location);
      }.bind(this);
    }
  }

  /**
   * Intercept store actions
   */
  interceptStoreActions() {
    console.log('🔧 Installing store interceptors...');

    const chatStore = this.getChatStore();
    if (!chatStore) return;

    // Intercept setCurrentChat
    if (chatStore.setCurrentChat) {
      const original = chatStore.setCurrentChat;
      chatStore.setCurrentChat = function (chatId) {
        this.logStep('STORE_ACTION', 'setCurrentChat', {
          chatId,
          previousChat: chatStore.currentChatId,
          timestamp: Date.now()
        });

        const result = original.call(this, chatId);

        this.logStep('STORE_ACTION', 'setCurrentChat_complete', {
          chatId,
          newCurrentChat: chatStore.currentChatId,
          timestamp: Date.now()
        });

        return result;
      }.bind(this);
    }

    // Intercept fetchMessagesWithSignal with detailed analysis
    if (chatStore.fetchMessagesWithSignal) {
      const original = chatStore.fetchMessagesWithSignal;
      chatStore.fetchMessagesWithSignal = async function (...args) {
        const startTime = Date.now();

        this.logStep('STORE_ACTION', 'fetchMessagesWithSignal_start', {
          args,
          chatId: args[0],
          timestamp: startTime
        });

        try {
          const result = await original.apply(this, args);

          this.logStep('STORE_ACTION', 'fetchMessagesWithSignal_success', {
            chatId: args[0],
            messageCount: result?.length || 0,
            firstMessage: result?.[0] ? {
              id: result[0].id,
              created_at: result[0].created_at,
              content: result[0].content?.substring(0, 50)
            } : null,
            lastMessage: result?.[result.length - 1] ? {
              id: result[result.length - 1].id,
              created_at: result[result.length - 1].created_at,
              content: result[result.length - 1].content?.substring(0, 50)
            } : null,
            duration: Date.now() - startTime,
            timestamp: Date.now()
          });

          // Analyze order immediately after fetch
          if (result && result.length > 1) {
            this.analyzeMessageOrder(result, 'FETCH_RESULT');
          }

          return result;

        } catch (error) {
          this.logStep('STORE_ACTION', 'fetchMessagesWithSignal_error', {
            chatId: args[0],
            error: error.message,
            duration: Date.now() - startTime,
            timestamp: Date.now()
          });

          throw error;
        }
      }.bind(this);
    }

    // Intercept messages property setter
    this.interceptMessagesProperty(chatStore);
  }

  /**
   * Intercept messages property with detailed tracking
   */
  interceptMessagesProperty(chatStore) {
    if (chatStore._completeTraceHooked) return;

    let internalMessages = chatStore.messages || [];
    const diagnosticInstance = this;

    Object.defineProperty(chatStore, 'messages', {
      get() {
        return internalMessages;
      },
      set(newMessages) {
        diagnosticInstance.logStep('STORE_PROPERTY', 'messages_setter', {
          messageCount: newMessages?.length || 0,
          previousCount: internalMessages?.length || 0,
          timestamp: Date.now()
        });

        if (newMessages && newMessages.length > 1) {
          diagnosticInstance.analyzeMessageOrder(newMessages, 'STORE_SETTER');
        }

        internalMessages = newMessages;

        diagnosticInstance.logStep('STORE_PROPERTY', 'messages_setter_complete', {
          finalCount: internalMessages?.length || 0,
          timestamp: Date.now()
        });
      },
      configurable: true,
      enumerable: true
    });

    chatStore._completeTraceHooked = true;
  }

  /**
   * Intercept API requests
   */
  interceptAPIRequests() {
    console.log('🔧 Installing API interceptors...');

    // Intercept fetch with detailed request/response analysis
    const originalFetch = window.fetch;
    window.fetch = async function (...args) {
      const url = args[0];
      const startTime = Date.now();

      if (url && url.includes('/messages')) {
        this.logStep('API_REQUEST', 'fetch_start', {
          url,
          method: args[1]?.method || 'GET',
          timestamp: startTime
        });

        try {
          const response = await originalFetch.apply(window, args);
          const clonedResponse = response.clone();

          this.logStep('API_REQUEST', 'fetch_response', {
            url,
            status: response.status,
            statusText: response.statusText,
            duration: Date.now() - startTime,
            timestamp: Date.now()
          });

          // Analyze raw API response
          try {
            const data = await clonedResponse.json();
            const messages = data.data || data || [];

            this.logStep('API_RESPONSE', 'data_parsed', {
              url,
              messageCount: messages.length,
              rawStructure: Array.isArray(data) ? 'Array' : 'Object',
              timestamp: Date.now()
            });

            if (Array.isArray(messages) && messages.length > 1) {
              this.analyzeMessageOrder(messages, 'RAW_API_RESPONSE');
              this.setExpectedOrder(messages);
            }

          } catch (parseError) {
            this.logStep('API_RESPONSE', 'parse_error', {
              url,
              error: parseError.message,
              timestamp: Date.now()
            });
          }

          return response;

        } catch (fetchError) {
          this.logStep('API_REQUEST', 'fetch_error', {
            url,
            error: fetchError.message,
            duration: Date.now() - startTime,
            timestamp: Date.now()
          });

          throw fetchError;
        }
      }

      return originalFetch.apply(window, args);
    }.bind(this);
  }

  /**
   * Intercept data processing
   */
  interceptDataProcessing() {
    console.log('🔧 Installing data processing interceptors...');

    // Intercept Array.prototype.sort to catch any sorting operations
    const originalSort = Array.prototype.sort;
    Array.prototype.sort = function (...args) {
      // Only intercept if this looks like a message array
      if (this.length > 0 && this[0] && typeof this[0] === 'object' && this[0].created_at) {
        this.logStep('DATA_PROCESSING', 'array_sort', {
          arrayLength: this.length,
          sortFunction: args[0] ? 'Custom' : 'Default',
          beforeFirst: this[0] ? {
            id: this[0].id,
            created_at: this[0].created_at
          } : null,
          beforeLast: this[this.length - 1] ? {
            id: this[this.length - 1].id,
            created_at: this[this.length - 1].created_at
          } : null,
          timestamp: Date.now()
        });
      }

      const result = originalSort.apply(this, args);

      // Log after sort if it's a message array
      if (this.length > 0 && this[0] && typeof this[0] === 'object' && this[0].created_at) {
        this.logStep('DATA_PROCESSING', 'array_sort_complete', {
          arrayLength: this.length,
          afterFirst: this[0] ? {
            id: this[0].id,
            created_at: this[0].created_at
          } : null,
          afterLast: this[this.length - 1] ? {
            id: this[this.length - 1].id,
            created_at: this[this.length - 1].created_at
          } : null,
          timestamp: Date.now()
        });

        this.analyzeMessageOrder(this, 'AFTER_SORT');
      }

      return result;
    }.bind(this);
  }

  /**
   * Intercept UI rendering
   */
  interceptUIRendering() {
    console.log('🔧 Installing UI rendering interceptors...');

    // Monitor DOM mutations for message rendering
    const observer = new MutationObserver((mutations) => {
      if (!this.isTracing) return;

      mutations.forEach((mutation) => {
        if (mutation.type === 'childList' && mutation.addedNodes.length > 0) {
          // Check if message elements were added
          const messageElements = Array.from(mutation.addedNodes).filter(node =>
            node.nodeType === 1 &&
            (node.classList?.contains('message') ||
              node.querySelector?.('.message') ||
              node.getAttribute?.('data-message-id'))
          );

          if (messageElements.length > 0) {
            this.logStep('UI_RENDERING', 'messages_rendered', {
              addedCount: messageElements.length,
              targetElement: mutation.target.className,
              timestamp: Date.now()
            });

            // Analyze DOM order
            setTimeout(() => {
              this.analyzeDOMOrder();
            }, 100);
          }
        }
      });
    });

    observer.observe(document.body, {
      childList: true,
      subtree: true
    });

    this.domObserver = observer;
  }

  /**
   * Analyze message order at any stage
   */
  analyzeMessageOrder(messages, stage) {
    if (!messages || messages.length < 2) return;

    const orderAnalysis = {
      stage,
      messageCount: messages.length,
      timestamps: [],
      isChronological: true,
      violations: [],
      timestamp: Date.now()
    };

    // Collect all timestamps
    for (let i = 0; i < messages.length; i++) {
      const timestamp = new Date(messages[i].created_at).getTime();
      orderAnalysis.timestamps.push({
        index: i,
        messageId: messages[i].id,
        timestamp,
        created_at: messages[i].created_at,
        content: messages[i].content?.substring(0, 30)
      });
    }

    // Check chronological order
    for (let i = 1; i < orderAnalysis.timestamps.length; i++) {
      const prev = orderAnalysis.timestamps[i - 1];
      const curr = orderAnalysis.timestamps[i];

      if (prev.timestamp > curr.timestamp) {
        orderAnalysis.isChronological = false;
        orderAnalysis.violations.push({
          position: i,
          prevTimestamp: prev.timestamp,
          currTimestamp: curr.timestamp,
          prevMessage: prev.messageId,
          currMessage: curr.messageId
        });
      }
    }

    this.logStep('ORDER_ANALYSIS', 'message_order', orderAnalysis);

    return orderAnalysis;
  }

  /**
   * Set expected order from API response
   */
  setExpectedOrder(messages) {
    if (!messages || messages.length === 0) return;

    // Expected order: chronological (oldest first)
    const sorted = [...messages].sort((a, b) => {
      return new Date(a.created_at).getTime() - new Date(b.created_at).getTime();
    });

    this.expectedOrder = sorted.map(msg => ({
      id: msg.id,
      created_at: msg.created_at,
      timestamp: new Date(msg.created_at).getTime()
    }));

    this.logStep('ORDER_ANALYSIS', 'expected_order_set', {
      messageCount: this.expectedOrder.length,
      oldestFirst: this.expectedOrder[0],
      newestLast: this.expectedOrder[this.expectedOrder.length - 1],
      timestamp: Date.now()
    });
  }

  /**
   * Analyze DOM order
   */
  analyzeDOMOrder() {
    const messageElements = document.querySelectorAll('[data-message-id], .message');

    if (messageElements.length < 2) return;

    const domOrder = [];
    messageElements.forEach((el, index) => {
      const messageId = el.getAttribute('data-message-id') ||
        el.querySelector('[data-message-id]')?.getAttribute('data-message-id');
      const timeElement = el.querySelector('.message-timestamp, [class*="timestamp"], .time');
      const contentElement = el.querySelector('.message-content, [class*="content"], .text');

      domOrder.push({
        domIndex: index,
        messageId,
        timestamp: timeElement?.textContent || timeElement?.title,
        content: contentElement?.textContent?.substring(0, 30) || 'No content'
      });
    });

    this.logStep('UI_RENDERING', 'dom_order_analysis', {
      elementCount: domOrder.length,
      domOrder,
      timestamp: Date.now()
    });

    return domOrder;
  }

  /**
   * Log a step in the flow
   */
  logStep(category, action, data) {
    if (!this.isTracing) return;

    const entry = {
      timestamp: Date.now(),
      category,
      action,
      data,
      sequenceNumber: this.traceLog.length
    };

    this.traceLog.push(entry);

    console.log(`🔬 [${entry.sequenceNumber}] ${category}:${action}`, data);
  }

  /**
   * Stop tracing and generate comprehensive report
   */
  stopAndAnalyze() {
    console.log('\n📊 STOPPING TRACE AND GENERATING COMPREHENSIVE REPORT');
    console.log('====================================================');

    this.isTracing = false;
    this.restoreInterceptors();

    if (this.traceLog.length === 0) {
      console.log('⚠️ No trace data collected. Make sure to click on a channel after starting trace.');
      return;
    }

    this.generateCompleteReport();
    this.generateRootCauseAnalysis();
    this.generateFixRecommendations();

    return this.traceLog;
  }

  /**
   * Generate complete flow report
   */
  generateCompleteReport() {
    console.log('\n📋 COMPLETE FLOW REPORT');
    console.log('=======================');

    // Group by category
    const categories = {};
    this.traceLog.forEach(entry => {
      if (!categories[entry.category]) {
        categories[entry.category] = [];
      }
      categories[entry.category].push(entry);
    });

    // Report each category
    Object.keys(categories).forEach(category => {
      console.log(`\n📂 ${category}:`);
      console.log(`${'='.repeat(category.length + 5)}`);

      categories[category].forEach(entry => {
        console.log(`   [${entry.sequenceNumber}] ${entry.action}:`, entry.data);
      });
    });
  }

  /**
   * Generate root cause analysis
   */
  generateRootCauseAnalysis() {
    console.log('\n🔍 ROOT CAUSE ANALYSIS');
    console.log('======================');

    // Find all order analyses
    const orderAnalyses = this.traceLog.filter(entry =>
      entry.category === 'ORDER_ANALYSIS' && entry.action === 'message_order'
    );

    if (orderAnalyses.length === 0) {
      console.log('⚠️ No order analysis data found');
      return;
    }

    console.log('\n📊 Order violations found:');
    orderAnalyses.forEach(analysis => {
      const data = analysis.data;
      const status = data.isChronological ? '✅ CORRECT' : '❌ WRONG';

      console.log(`\n🔍 Stage: ${data.stage} - ${status}`);
      console.log(`   Messages: ${data.messageCount}`);

      if (!data.isChronological) {
        console.log(`   Violations: ${data.violations.length}`);
        data.violations.forEach((violation, i) => {
          console.log(`     [${i + 1}] Position ${violation.position}: Message ${violation.currMessage} (${new Date(violation.currTimestamp).toLocaleTimeString()}) should come after ${violation.prevMessage} (${new Date(violation.prevTimestamp).toLocaleTimeString()})`);
        });
      }
    });

    // Identify the exact point where order breaks
    this.identifyOrderBreakPoint(orderAnalyses);
  }

  /**
   * Identify exact point where order breaks
   */
  identifyOrderBreakPoint(orderAnalyses) {
    console.log('\n🎯 ORDER BREAK POINT ANALYSIS');
    console.log('==============================');

    let firstCorrect = null;
    let firstIncorrect = null;

    for (const analysis of orderAnalyses) {
      if (analysis.data.isChronological && !firstCorrect) {
        firstCorrect = analysis;
      } else if (!analysis.data.isChronological && !firstIncorrect) {
        firstIncorrect = analysis;
        break;
      }
    }

    if (firstCorrect && firstIncorrect) {
      console.log(`🔍 Order breaks between:`);
      console.log(`   ✅ Last correct: ${firstCorrect.data.stage}`);
      console.log(`   ❌ First incorrect: ${firstIncorrect.data.stage}`);
      console.log(`\n💡 Root cause likely in the processing between these stages`);
    } else if (!firstCorrect) {
      console.log(`🚨 Order is wrong from the very beginning (${orderAnalyses[0]?.data.stage})`);
      console.log(`💡 Root cause: API response or data source`);
    } else {
      console.log(`✅ All analyzed stages show correct order`);
      console.log(`💡 Issue might be in UI rendering or untraced processing`);
    }
  }

  /**
   * Generate fix recommendations
   */
  generateFixRecommendations() {
    console.log('\n💡 FIX RECOMMENDATIONS');
    console.log('=======================');

    // Analyze where the problem occurs
    const orderAnalyses = this.traceLog.filter(entry =>
      entry.category === 'ORDER_ANALYSIS' && entry.action === 'message_order'
    );

    const apiWrong = orderAnalyses.some(a => a.data.stage === 'RAW_API_RESPONSE' && !a.data.isChronological);
    const storeWrong = orderAnalyses.some(a => a.data.stage.includes('STORE') && !a.data.isChronological);
    const sortWrong = orderAnalyses.some(a => a.data.stage === 'AFTER_SORT' && !a.data.isChronological);

    if (apiWrong) {
      console.log('🔧 API LEVEL FIX NEEDED:');
      console.log('   Backend API returns messages in wrong order');
      console.log('   Solution: Fix backend SQL query ORDER BY created_at ASC');
      console.log('   Temp fix: Sort on frontend after API response');
    }

    if (storeWrong) {
      console.log('🔧 STORE LEVEL FIX NEEDED:');
      console.log('   Store processing corrupts message order');
      console.log('   Solution: Fix store message processing logic');
    }

    if (sortWrong) {
      console.log('🔧 SORTING LOGIC FIX NEEDED:');
      console.log('   Array sort function has wrong logic');
      console.log('   Solution: Fix sort comparator function');
    }

    console.log('\n🚀 IMMEDIATE ACTIONS:');
    console.log('   1. Apply emergency fix: window.completeFlow.emergencyFix()');
    console.log('   2. Re-run trace to verify: window.completeFlow.startCompleteTrace()');
    console.log('   3. Monitor specific stage: Check logs above');
  }

  /**
   * Apply emergency fix based on analysis
   */
  emergencyFix() {
    console.log('\n🚨 APPLYING EMERGENCY FIX BASED ON ANALYSIS');
    console.log('===========================================');

    const chatStore = this.getChatStore();
    if (!chatStore) {
      console.log('❌ Cannot fix: chat store not available');
      return;
    }

    // Fix current messages
    if (chatStore.messages && chatStore.messages.length > 0) {
      console.log('🔧 Fixing current messages...');
      const before = chatStore.messages.map(m => this.formatTime(m.created_at)).join(' → ');

      chatStore.messages.sort((a, b) => {
        return new Date(a.created_at).getTime() - new Date(b.created_at).getTime();
      });

      const after = chatStore.messages.map(m => this.formatTime(m.created_at)).join(' → ');
      console.log(`   Before: ${before}`);
      console.log(`   After:  ${after}`);
    }

    // Install permanent fix
    this.installPermanentFix(chatStore);

    console.log('✅ Emergency fix applied');
    console.log('🔄 Refresh page or switch channels to test');
  }

  /**
   * Install permanent fix
   */
  installPermanentFix(chatStore) {
    if (chatStore._permanentOrderFixInstalled) return;

    // Override fetchMessagesWithSignal to always sort
    if (chatStore.fetchMessagesWithSignal) {
      const original = chatStore.fetchMessagesWithSignal;
      chatStore.fetchMessagesWithSignal = async function (...args) {
        const result = await original.apply(this, args);

        if (result && Array.isArray(result) && result.length > 1) {
          result.sort((a, b) => {
            return new Date(a.created_at).getTime() - new Date(b.created_at).getTime();
          });
          console.log('🔧 Auto-sorted messages chronologically');
        }

        return result;
      };

      chatStore._permanentOrderFixInstalled = true;
      console.log('✅ Permanent sorting fix installed');
    }
  }

  /**
   * Restore interceptors
   */
  restoreInterceptors() {
    // Stop DOM observer
    if (this.domObserver) {
      this.domObserver.disconnect();
      this.domObserver = null;
    }
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
const completeFlowDiagnostic = new CompleteMessageFlowDiagnostic();

// Export for use
export default completeFlowDiagnostic;

// Expose to window
if (typeof window !== 'undefined') {
  window.completeFlow = {
    trace: () => completeFlowDiagnostic.startCompleteTrace(),
    stop: () => completeFlowDiagnostic.stopAndAnalyze(),
    fix: () => completeFlowDiagnostic.emergencyFix(),
    run: () => {
      console.log('🔬 Starting complete flow trace...');
      console.log('👆 Click on a channel after this message to trace the full flow');
      return completeFlowDiagnostic.startCompleteTrace();
    }
  };

  console.log('🔬 Complete Message Flow Diagnostic loaded');
  console.log('   Commands:');
  console.log('   - window.completeFlow.run() - Start tracing (then click channel)');
  console.log('   - window.completeFlow.stop() - Stop and analyze');
  console.log('   - window.completeFlow.fix() - Apply emergency fix');
} 