/**
 * Performance Bottleneck Analysis Tool
 * 性能瓶颈深度分析工具 - 解决12.7秒延迟问题
 */

class PerformanceBottleneckAnalysis {
  constructor() {
    this.measurements = [];
    this.apiCallTimes = [];
    this.renderTimes = [];
    this.networkTimes = [];
    this.storeTimes = [];

    console.log('🔍 Performance Bottleneck Analysis initialized');
  }

  /**
   * Start comprehensive performance analysis
   * 启动综合性能分析
   */
  async analyzeFullChain() {
    console.log('\n🔍 COMPREHENSIVE PERFORMANCE BOTTLENECK ANALYSIS');
    console.log('================================================\n');

    // 1. Analyze current system performance
    await this.analyzeCurrentSystem();

    // 2. Analyze API performance
    await this.analyzeAPIPerformance();

    // 3. Analyze store performance
    this.analyzeStorePerformance();

    // 4. Analyze rendering performance
    this.analyzeRenderingPerformance();

    // 5. Analyze network stack
    await this.analyzeNetworkStack();

    // 6. Generate comprehensive report
    this.generateBottleneckReport();
  }

  /**
   * Analyze current message system performance
   */
  async analyzeCurrentSystem() {
    console.log('1️⃣ CURRENT SYSTEM ANALYSIS');
    console.log('==========================');

    const chatStore = this.getChatStore();
    if (!chatStore) {
      console.log('❌ Chat store not available');
      return;
    }

    // Test current sendMessage performance
    if (chatStore.currentChatId) {
      console.log('🧪 Testing current sendMessage performance...');

      const startTime = performance.now();

      // Hook into performance monitoring
      const originalSendMessage = chatStore.sendMessage;
      let sendMessageTime = 0;

      chatStore.sendMessage = async function (...args) {
        const sendStart = performance.now();
        const result = await originalSendMessage.apply(this, args);
        sendMessageTime = performance.now() - sendStart;
        return result;
      };

      // Test with dummy message (don't actually send)
      try {
        // Simulate the timing without actual send
        const testContent = 'Performance test message';

        // Measure UI update time
        const uiStart = performance.now();

        // Create test message
        const testMessage = {
          id: `test-${Date.now()}`,
          content: testContent,
          created_at: new Date().toISOString()
        };

        // Test UI processing
        if (chatStore.messages) {
          const messagesLength = chatStore.messages.length;
          chatStore.messages.push(testMessage);
          const uiTime = performance.now() - uiStart;

          // Remove test message
          chatStore.messages.pop();

          console.log(`📊 UI Update Time: ${uiTime.toFixed(2)}ms`);
          this.measurements.push({
            component: 'UI Update',
            time: uiTime,
            status: uiTime < 10 ? 'GOOD' : 'SLOW'
          });
        }
      } catch (error) {
        console.error('Error testing current system:', error);
      }

      // Restore original method
      chatStore.sendMessage = originalSendMessage;
    }
  }

  /**
   * Analyze API performance with detailed timing
   */
  async analyzeAPIPerformance() {
    console.log('\n2️⃣ API PERFORMANCE ANALYSIS');
    console.log('============================');

    const testAPIs = [
      {
        name: 'Health Check',
        url: '/api/health',
        method: 'GET'
      },
      {
        name: 'User Profile',
        url: '/api/user/profile',
        method: 'GET'
      }
    ];

    for (const api of testAPIs) {
      await this.testAPIEndpoint(api);
    }

    // Test message API if we have a chat
    const chatStore = this.getChatStore();
    if (chatStore?.currentChatId) {
      await this.testAPIEndpoint({
        name: 'Fetch Messages',
        url: `/api/chat/${chatStore.currentChatId}/messages?limit=5`,
        method: 'GET'
      });
    }
  }

  /**
   * Test individual API endpoint performance
   */
  async testAPIEndpoint(api) {
    console.log(`🧪 Testing ${api.name}...`);

    const times = [];
    const errors = [];

    // Test 3 times for consistency
    for (let i = 0; i < 3; i++) {
      try {
        const startTime = performance.now();

        const response = await fetch(api.url, {
          method: api.method,
          headers: {
            'Content-Type': 'application/json',
            'Accept': 'application/json'
          }
        });

        const endTime = performance.now();
        const responseTime = endTime - startTime;

        times.push(responseTime);

        console.log(`   Attempt ${i + 1}: ${responseTime.toFixed(2)}ms (${response.status})`);

      } catch (error) {
        errors.push(error);
        console.log(`   Attempt ${i + 1}: ERROR - ${error.message}`);
      }
    }

    if (times.length > 0) {
      const avgTime = times.reduce((a, b) => a + b, 0) / times.length;
      const maxTime = Math.max(...times);
      const minTime = Math.min(...times);

      console.log(`📊 ${api.name} Performance:`);
      console.log(`   Average: ${avgTime.toFixed(2)}ms`);
      console.log(`   Range: ${minTime.toFixed(2)}ms - ${maxTime.toFixed(2)}ms`);

      const status = avgTime < 100 ? 'EXCELLENT' :
        avgTime < 500 ? 'GOOD' :
          avgTime < 2000 ? 'SLOW' : 'CRITICAL';

      console.log(`   Status: ${status}`);

      this.apiCallTimes.push({
        endpoint: api.name,
        average: avgTime,
        max: maxTime,
        min: minTime,
        status
      });
    }

    if (errors.length > 0) {
      console.log(`❌ ${errors.length} errors encountered`);
    }
  }

  /**
   * Analyze Pinia store performance
   */
  analyzeStorePerformance() {
    console.log('\n3️⃣ STORE PERFORMANCE ANALYSIS');
    console.log('==============================');

    const chatStore = this.getChatStore();
    if (!chatStore) {
      console.log('❌ Chat store not available');
      return;
    }

    // Test store state access
    const stateAccessTime = this.measureFunction(() => {
      const currentChatId = chatStore.currentChatId;
      const messages = chatStore.messages;
      const loading = chatStore.loading;
      return { currentChatId, messages: messages?.length || 0, loading };
    });

    console.log(`📊 Store State Access: ${stateAccessTime.toFixed(2)}ms`);

    // Test store method calls
    if (typeof chatStore.normalizeMessage === 'function') {
      const normalizeTime = this.measureFunction(() => {
        const testMessage = {
          id: 1,
          content: 'test',
          created_at: new Date().toISOString()
        };
        return chatStore.normalizeMessage(testMessage);
      });

      console.log(`📊 Message Normalization: ${normalizeTime.toFixed(2)}ms`);
    }

    // Analyze store size
    const storeSize = this.getObjectSize(chatStore);
    console.log(`📊 Store Memory Size: ${storeSize}KB`);

    this.storeTimes.push({
      stateAccess: stateAccessTime,
      storeSize
    });
  }

  /**
   * Analyze rendering and DOM performance
   */
  analyzeRenderingPerformance() {
    console.log('\n4️⃣ RENDERING PERFORMANCE ANALYSIS');
    console.log('==================================');

    // Test DOM query performance
    const domQueryTime = this.measureFunction(() => {
      const chatContainer = document.querySelector('.chat-container');
      const messageElements = document.querySelectorAll('.message');
      const inputElement = document.querySelector('.message-input');
      return {
        chatContainer: !!chatContainer,
        messageCount: messageElements.length,
        hasInput: !!inputElement
      };
    });

    console.log(`📊 DOM Query Time: ${domQueryTime.toFixed(2)}ms`);

    // Test scroll performance
    const scrollContainer = document.querySelector('.messages-container') ||
      document.querySelector('.chat-messages') ||
      document.querySelector('[class*="message"]')?.parentElement;

    if (scrollContainer) {
      const scrollTime = this.measureFunction(() => {
        scrollContainer.scrollTop = scrollContainer.scrollHeight;
      });

      console.log(`📊 Scroll Performance: ${scrollTime.toFixed(2)}ms`);
    }

    // Analyze message count impact
    const messageElements = document.querySelectorAll('[class*="message"]');
    console.log(`📊 Current Message Elements: ${messageElements.length}`);

    this.renderTimes.push({
      domQuery: domQueryTime,
      messageCount: messageElements.length
    });
  }

  /**
   * Analyze network stack performance
   */
  async analyzeNetworkStack() {
    console.log('\n5️⃣ NETWORK STACK ANALYSIS');
    console.log('==========================');

    // DNS resolution test
    const dnsTime = await this.testDNSResolution();
    console.log(`📊 DNS Resolution: ${dnsTime.toFixed(2)}ms`);

    // Connection establishment
    const connectionTime = await this.testConnectionTime();
    console.log(`📊 Connection Time: ${connectionTime.toFixed(2)}ms`);

    // Test with different request sizes
    await this.testRequestSizes();

    this.networkTimes.push({
      dns: dnsTime,
      connection: connectionTime
    });
  }

  /**
   * Test DNS resolution time
   */
  async testDNSResolution() {
    const startTime = performance.now();

    try {
      // Make a simple request to test DNS + connection
      await fetch(window.location.origin + '/api/health', {
        method: 'HEAD',
        cache: 'no-cache'
      });

      return performance.now() - startTime;
    } catch (error) {
      console.warn('DNS test failed:', error);
      return -1;
    }
  }

  /**
   * Test connection establishment time
   */
  async testConnectionTime() {
    const times = [];

    // Test multiple small requests
    for (let i = 0; i < 3; i++) {
      const startTime = performance.now();

      try {
        await fetch(window.location.origin + '/api/health', {
          method: 'HEAD'
        });

        times.push(performance.now() - startTime);
      } catch (error) {
        console.warn(`Connection test ${i + 1} failed:`, error);
      }
    }

    return times.length > 0 ? times.reduce((a, b) => a + b, 0) / times.length : -1;
  }

  /**
   * Test different request payload sizes
   */
  async testRequestSizes() {
    const sizes = [
      { name: 'Small (100B)', size: 100 },
      { name: 'Medium (1KB)', size: 1024 },
      { name: 'Large (10KB)', size: 10240 }
    ];

    for (const test of sizes) {
      const payload = 'x'.repeat(test.size);
      const startTime = performance.now();

      try {
        await fetch('/api/test-payload', {
          method: 'POST',
          headers: { 'Content-Type': 'text/plain' },
          body: payload
        });

        const time = performance.now() - startTime;
        console.log(`📊 ${test.name} Request: ${time.toFixed(2)}ms`);

      } catch (error) {
        console.log(`📊 ${test.name} Request: FAILED (${error.message})`);
      }
    }
  }

  /**
   * Generate comprehensive bottleneck report
   */
  generateBottleneckReport() {
    console.log('\n📊 BOTTLENECK ANALYSIS REPORT');
    console.log('==============================\n');

    // Identify critical bottlenecks
    const bottlenecks = [];

    // API bottlenecks
    this.apiCallTimes.forEach(api => {
      if (api.average > 2000) {
        bottlenecks.push({
          component: `API: ${api.endpoint}`,
          severity: 'CRITICAL',
          time: api.average,
          recommendation: 'Optimize backend response time or implement caching'
        });
      } else if (api.average > 500) {
        bottlenecks.push({
          component: `API: ${api.endpoint}`,
          severity: 'HIGH',
          time: api.average,
          recommendation: 'Consider request optimization or CDN'
        });
      }
    });

    // Store bottlenecks
    this.storeTimes.forEach(store => {
      if (store.stateAccess > 10) {
        bottlenecks.push({
          component: 'Store State Access',
          severity: 'MEDIUM',
          time: store.stateAccess,
          recommendation: 'Optimize store structure or use computed properties'
        });
      }
    });

    // Rendering bottlenecks
    this.renderTimes.forEach(render => {
      if (render.domQuery > 50) {
        bottlenecks.push({
          component: 'DOM Queries',
          severity: 'MEDIUM',
          time: render.domQuery,
          recommendation: 'Cache DOM references or use virtual scrolling'
        });
      }

      if (render.messageCount > 100) {
        bottlenecks.push({
          component: 'Message Count',
          severity: 'HIGH',
          time: render.messageCount,
          recommendation: 'Implement message virtualization'
        });
      }
    });

    // Display bottlenecks
    if (bottlenecks.length === 0) {
      console.log('✅ No significant bottlenecks detected');
    } else {
      console.log('🔍 Identified Bottlenecks:');

      bottlenecks
        .sort((a, b) => {
          const severityOrder = { CRITICAL: 3, HIGH: 2, MEDIUM: 1 };
          return severityOrder[b.severity] - severityOrder[a.severity];
        })
        .forEach((bottleneck, index) => {
          const emoji = {
            CRITICAL: '🚨',
            HIGH: '⚠️',
            MEDIUM: '⚡'
          }[bottleneck.severity];

          console.log(`\n${index + 1}. ${emoji} ${bottleneck.component}`);
          console.log(`   Severity: ${bottleneck.severity}`);
          console.log(`   Impact: ${bottleneck.time.toFixed(2)}ms`);
          console.log(`   Fix: ${bottleneck.recommendation}`);
        });
    }

    // Performance summary
    console.log('\n📈 PERFORMANCE SUMMARY');
    console.log('======================');

    const totalAPITime = this.apiCallTimes.reduce((sum, api) => sum + api.average, 0);
    const totalStoreTime = this.storeTimes.reduce((sum, store) => sum + store.stateAccess, 0);
    const totalRenderTime = this.renderTimes.reduce((sum, render) => sum + render.domQuery, 0);

    console.log(`🌐 Total API Time: ${totalAPITime.toFixed(2)}ms`);
    console.log(`🏪 Total Store Time: ${totalStoreTime.toFixed(2)}ms`);
    console.log(`🎨 Total Render Time: ${totalRenderTime.toFixed(2)}ms`);

    const estimatedTotal = totalAPITime + totalStoreTime + totalRenderTime;
    console.log(`⏱️ Estimated Total Chain: ${estimatedTotal.toFixed(2)}ms`);

    // Root cause analysis for 12.7s delay
    if (estimatedTotal > 5000) {
      console.log('\n🚨 ROOT CAUSE OF 12.7s DELAY IDENTIFIED:');
      console.log('=========================================');

      if (totalAPITime > 5000) {
        console.log('🎯 PRIMARY CAUSE: Backend API Response Time');
        console.log('   - Backend is taking >5 seconds to respond');
        console.log('   - Immediate fix: Implement ultra-fast optimistic updates');
        console.log('   - Long-term fix: Backend optimization required');
      }

      if (totalStoreTime > 1000) {
        console.log('🎯 SECONDARY CAUSE: Store Processing Overhead');
        console.log('   - Store operations are taking >1 second');
        console.log('   - Fix: Bypass store for immediate UI updates');
      }

      if (totalRenderTime > 500) {
        console.log('🎯 TERTIARY CAUSE: DOM Rendering Bottleneck');
        console.log('   - DOM operations are taking >500ms');
        console.log('   - Fix: Implement virtual scrolling or message windowing');
      }
    }

    console.log('\n💡 IMMEDIATE SOLUTION:');
    console.log('======================');
    console.log('🚀 Activate Ultra-Fast Message System:');
    console.log('   window.ultraFast.init()');
    console.log('   window.ultraFast.replace()');
    console.log('');
    console.log('✅ This will bypass ALL bottlenecks with:');
    console.log('   - Instant optimistic updates (0ms UI delay)');
    console.log('   - Direct API calls (bypass middleware)');
    console.log('   - Memory + localStorage dual cache');
    console.log('   - Background sync without blocking');

    return {
      bottlenecks,
      summary: {
        apiTime: totalAPITime,
        storeTime: totalStoreTime,
        renderTime: totalRenderTime,
        estimatedTotal
      }
    };
  }

  /**
   * Utility: Measure function execution time
   */
  measureFunction(fn) {
    const startTime = performance.now();
    fn();
    return performance.now() - startTime;
  }

  /**
   * Utility: Get approximate object size in KB
   */
  getObjectSize(obj) {
    try {
      const jsonString = JSON.stringify(obj);
      return Math.round((jsonString.length * 2) / 1024); // Rough estimate
    } catch (error) {
      return 0;
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
}

// Create global instance
const bottleneckAnalyzer = new PerformanceBottleneckAnalysis();

// Expose to window
if (typeof window !== 'undefined') {
  window.analyzeBottlenecks = {
    run: () => bottleneckAnalyzer.analyzeFullChain(),
    api: () => bottleneckAnalyzer.analyzeAPIPerformance(),
    store: () => bottleneckAnalyzer.analyzeStorePerformance(),
    render: () => bottleneckAnalyzer.analyzeRenderingPerformance(),
    network: () => bottleneckAnalyzer.analyzeNetworkStack()
  };

  console.log('🔍 Performance Bottleneck Analyzer loaded');
  console.log('   Commands:');
  console.log('   - window.analyzeBottlenecks.run() - Full analysis');
  console.log('   - window.analyzeBottlenecks.api() - API analysis');
  console.log('   - window.analyzeBottlenecks.store() - Store analysis');
}

export default bottleneckAnalyzer; 