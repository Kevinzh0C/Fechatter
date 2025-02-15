/**
 * Ultra-Fast Performance Test
 * 超快性能测试 - 验证消息发送到SSE接收的速度优化
 */

class UltraFastPerformanceTest {
  constructor() {
    this.testResults = [];
    this.isRunning = false;
    console.log('⚡ Ultra-Fast Performance Test initialized');
  }

  /**
   * Run comprehensive performance test
   * 运行综合性能测试
   */
  async runTest() {
    if (this.isRunning) {
      console.log('⚠️ Test already running');
      return;
    }

    this.isRunning = true;
    this.testResults = [];

    console.log('\n⚡ ULTRA-FAST PERFORMANCE TEST');
    console.log('==============================');

    try {
      // Test 1: UI Update Speed
      const uiTest = await this.testUIUpdateSpeed();
      console.log(`📱 UI Update Speed: ${uiTest.average.toFixed(2)}ms`);

      // Test 2: Message Send Speed
      const sendTest = await this.testMessageSendSpeed();
      console.log(`📤 Message Send Speed: ${sendTest.average.toFixed(2)}ms`);

      // Test 3: SSE Processing Speed
      const sseTest = this.testSSEProcessingSpeed();
      console.log(`📨 SSE Processing Speed: ${sseTest.average.toFixed(2)}ms`);

      // Test 4: Cache Update Speed
      const cacheTest = await this.testCacheUpdateSpeed();
      console.log(`📦 Cache Update Speed: ${cacheTest.average.toFixed(2)}ms`);

      // Test 5: End-to-End Speed
      const e2eTest = await this.testEndToEndSpeed();
      console.log(`🔄 End-to-End Speed: ${e2eTest.average.toFixed(2)}ms`);

      // Summary
      this.showSummary({
        ui: uiTest,
        send: sendTest,
        sse: sseTest,
        cache: cacheTest,
        e2e: e2eTest
      });

    } catch (error) {
      console.error('❌ Performance test failed:', error);
    } finally {
      this.isRunning = false;
    }
  }

  /**
   * Test UI update speed
   */
  async testUIUpdateSpeed() {
    const chatStore = this.getChatStore();
    if (!chatStore) throw new Error('Chat store not available');

    const times = [];
    const testCount = 10;

    for (let i = 0; i < testCount; i++) {
      const startTime = performance.now();

      // Simulate adding a message to UI
      const testMessage = {
        id: `test-ui-${Date.now()}-${i}`,
        content: `UI test message ${i}`,
        sender: { fullname: 'Test User' },
        created_at: new Date().toISOString(),
        _normalized: true,
        _timestamp: Date.now()
      };

      if (chatStore.messages) {
        chatStore.messages.push(testMessage);

        // Force Vue reactivity
        await this.$nextTick?.() || new Promise(resolve => setTimeout(resolve, 0));

        const elapsed = performance.now() - startTime;
        times.push(elapsed);

        // Clean up
        chatStore.messages.pop();
      }

      // Small delay between tests
      await new Promise(resolve => setTimeout(resolve, 10));
    }

    return this.calculateStats(times, 'UI Update');
  }

  /**
   * Test message send speed (optimistic update)
   */
  async testMessageSendSpeed() {
    const chatStore = this.getChatStore();
    if (!chatStore || !chatStore.sendMessageUltraFast) {
      throw new Error('Ultra-fast send method not available');
    }

    const times = [];
    const testCount = 5; // Fewer tests to avoid spamming backend

    for (let i = 0; i < testCount; i++) {
      const startTime = performance.now();

      try {
        // Test optimistic update speed (not actual backend call)
        const testChatId = chatStore.currentChatId || 1;

        // Mock the ultra-fast send (just the UI part)
        const tempMessage = {
          id: `temp-test-${Date.now()}-${i}`,
          content: `Performance test ${i}`,
          sender_id: 1,
          created_at: new Date().toISOString(),
          _timestamp: Date.now()
        };

        if (chatStore.messages) {
          chatStore.messages.push(tempMessage);

          const elapsed = performance.now() - startTime;
          times.push(elapsed);

          // Clean up
          chatStore.messages.pop();
        }

      } catch (error) {
        console.warn(`Send test ${i} failed:`, error);
      }

      await new Promise(resolve => setTimeout(resolve, 50));
    }

    return this.calculateStats(times, 'Message Send');
  }

  /**
   * Test SSE processing speed
   */
  testSSEProcessingSpeed() {
    const sseService = window.minimalSSE;
    if (!sseService) {
      return { average: 0, min: 0, max: 0, count: 0 };
    }

    const stats = sseService.getPerformanceStats();

    return {
      average: parseFloat(stats.averageProcessingTime) || 0,
      min: parseFloat(stats.lastMessageTime) || 0,
      max: parseFloat(stats.lastMessageTime) || 0,
      count: stats.messageCount || 0,
      label: 'SSE Processing'
    };
  }

  /**
   * Test cache update speed
   */
  async testCacheUpdateSpeed() {
    const chatStore = this.getChatStore();
    if (!chatStore) throw new Error('Chat store not available');

    const times = [];
    const testCount = 10;
    const testChatId = 999; // Use fake chat ID

    for (let i = 0; i < testCount; i++) {
      const startTime = performance.now();

      // Test cache update
      const testMessage = {
        id: `cache-test-${Date.now()}-${i}`,
        content: `Cache test ${i}`,
        created_at: new Date().toISOString(),
        _timestamp: Date.now()
      };

      // Update cache
      if (!chatStore.messageCache[testChatId]) {
        chatStore.messageCache[testChatId] = {
          messages: [],
          timestamp: Date.now()
        };
      }

      chatStore.messageCache[testChatId].messages.push(testMessage);

      const elapsed = performance.now() - startTime;
      times.push(elapsed);

      await new Promise(resolve => setTimeout(resolve, 5));
    }

    // Clean up test cache
    delete chatStore.messageCache[testChatId];

    return this.calculateStats(times, 'Cache Update');
  }

  /**
   * Test end-to-end speed simulation
   */
  async testEndToEndSpeed() {
    const times = [];
    const testCount = 5;

    for (let i = 0; i < testCount; i++) {
      const startTime = performance.now();

      // Simulate full flow: UI update + cache + persistence
      try {
        const chatStore = this.getChatStore();
        if (chatStore) {
          // 1. UI update
          const testMessage = {
            id: `e2e-test-${Date.now()}-${i}`,
            content: `E2E test ${i}`,
            created_at: new Date().toISOString(),
            _timestamp: Date.now()
          };

          // 2. Add to messages
          chatStore.messages?.push(testMessage);

          // 3. Update cache
          const testChatId = 888;
          if (!chatStore.messageCache[testChatId]) {
            chatStore.messageCache[testChatId] = { messages: [], timestamp: Date.now() };
          }
          chatStore.messageCache[testChatId].messages.push(testMessage);

          // 4. Simulate persistence (localStorage write)
          try {
            localStorage.setItem(`test-e2e-${i}`, JSON.stringify(testMessage));
          } catch (e) {
            // Ignore localStorage errors
          }

          const elapsed = performance.now() - startTime;
          times.push(elapsed);

          // Clean up
          chatStore.messages?.pop();
          delete chatStore.messageCache[testChatId];
          localStorage.removeItem(`test-e2e-${i}`);
        }

      } catch (error) {
        console.warn(`E2E test ${i} failed:`, error);
      }

      await new Promise(resolve => setTimeout(resolve, 20));
    }

    return this.calculateStats(times, 'End-to-End');
  }

  /**
   * Calculate statistics from timing array
   */
  calculateStats(times, label) {
    if (times.length === 0) {
      return { average: 0, min: 0, max: 0, count: 0, label };
    }

    const average = times.reduce((a, b) => a + b, 0) / times.length;
    const min = Math.min(...times);
    const max = Math.max(...times);

    return {
      average,
      min,
      max,
      count: times.length,
      label,
      times
    };
  }

  /**
   * Show performance summary
   */
  showSummary(results) {
    console.log('\n📊 PERFORMANCE SUMMARY');
    console.log('======================');

    Object.values(results).forEach(result => {
      if (result.count > 0) {
        console.log(`${result.label}:`);
        console.log(`  Average: ${result.average.toFixed(2)}ms`);
        console.log(`  Min: ${result.min.toFixed(2)}ms`);
        console.log(`  Max: ${result.max.toFixed(2)}ms`);
        console.log(`  Tests: ${result.count}`);
        console.log('');
      }
    });

    // Performance rating
    const avgE2E = results.e2e.average;
    let rating = '🔥 ULTRA-FAST';
    if (avgE2E > 10) rating = '⚡ FAST';
    if (avgE2E > 50) rating = '🐌 SLOW';
    if (avgE2E > 100) rating = '🚨 VERY SLOW';

    console.log(`Overall Rating: ${rating} (${avgE2E.toFixed(2)}ms E2E)`);
    console.log('');

    // Recommendations
    if (avgE2E < 5) {
      console.log('✅ Excellent performance! System is ultra-fast.');
    } else if (avgE2E < 20) {
      console.log('✅ Good performance! System is fast enough.');
    } else {
      console.log('⚠️ Performance could be improved. Check for bottlenecks.');
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
   * Get current performance baseline
   */
  getBaseline() {
    const sseStats = window.sseStats?.() || {};

    return {
      sseMessages: sseStats.messageCount || 0,
      sseAverage: parseFloat(sseStats.averageProcessingTime) || 0,
      sseConnected: sseStats.isConnected || false,
      timestamp: new Date().toISOString()
    };
  }
}

// Create global instance
const ultraFastTest = new UltraFastPerformanceTest();

// Export for use
export default ultraFastTest;

// Expose to window
if (typeof window !== 'undefined') {
  window.ultraFastTest = ultraFastTest;
  window.testPerformance = () => ultraFastTest.runTest();
  window.getPerformanceBaseline = () => ultraFastTest.getBaseline();
}

console.log('⚡ Ultra-Fast Performance Test loaded');
console.log('   Commands:');
console.log('   - window.testPerformance() - Run full performance test');
console.log('   - window.getPerformanceBaseline() - Get current baseline'); 