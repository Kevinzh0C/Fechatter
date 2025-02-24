/**
 * 🧪 MessageDisplayGuarantee Validation & Testing Script
 * Production-grade validation for tracking context fixes
 */

export class MessageDisplayGuaranteeValidator {
  constructor() {
    this.testResults = [];
    this.guarantee = null;
    this.mockChatId = 999;
    this.mockMessageIds = [1001, 1002, 1003, 1004, 1005];
  }

  async initialize() {
    // Import the guarantee system
    try {
      const { messageDisplayGuarantee } = await import('./messageSystem/MessageDisplayGuarantee.js');
      this.guarantee = messageDisplayGuarantee;

      // Enable debug mode for testing
      this.guarantee.enableDebugMode();

      console.log('✅ MessageDisplayGuarantee validation initialized');
      return true;
    } catch (error) {
      console.error('❌ Failed to initialize validation:', error);
      return false;
    }
  }

  /**
   * Test 1: Basic tracking context creation and cleanup
   */
  async testBasicTracking() {
    console.log('🧪 Test 1: Basic tracking context creation');

    try {
      // Create tracking context
      const trackingId = this.guarantee.startMessageTracking(this.mockChatId, this.mockMessageIds);

      if (!trackingId) {
        throw new Error('No tracking ID returned');
      }

      // Verify context exists
      const hasContext = this.guarantee.verificationQueue.has(trackingId);
      if (!hasContext) {
        throw new Error('Tracking context not found in queue');
      }

      // Mark some messages as displayed
      this.guarantee.markMessageDisplayed(1001, null, this.mockChatId);
      this.guarantee.markMessageDisplayed(1002, null, this.mockChatId);

      // Check progress
      const context = this.guarantee.verificationQueue.get(trackingId);
      const displayedCount = context.displayedIds.size;

      this.testResults.push({
        test: 'Basic Tracking',
        status: displayedCount === 2 ? 'PASS' : 'FAIL',
        details: `Displayed ${displayedCount}/2 expected messages`
      });

      // Cleanup
      this.guarantee.clearTrackingForChat(this.mockChatId);

      return displayedCount === 2;
    } catch (error) {
      this.testResults.push({
        test: 'Basic Tracking',
        status: 'FAIL',
        error: error.message
      });
      return false;
    }
  }

  /**
   * Test 2: Fallback context creation for orphaned messages
   */
  async testFallbackContext() {
    console.log('🧪 Test 2: Fallback context creation');

    try {
      // Mark message as displayed WITHOUT creating tracking context first
      this.guarantee.markMessageDisplayed(2001, null, this.mockChatId);

      // Check if fallback context was created
      let fallbackFound = false;
      for (const [trackingId, context] of this.guarantee.verificationQueue.entries()) {
        if (context.isFallback && context.chatId === this.mockChatId) {
          fallbackFound = true;
          break;
        }
      }

      this.testResults.push({
        test: 'Fallback Context',
        status: fallbackFound ? 'PASS' : 'FAIL',
        details: fallbackFound ? 'Fallback context created successfully' : 'No fallback context found'
      });

      // Cleanup
      this.guarantee.clearTrackingForChat(this.mockChatId);

      return fallbackFound;
    } catch (error) {
      this.testResults.push({
        test: 'Fallback Context',
        status: 'FAIL',
        error: error.message
      });
      return false;
    }
  }

  /**
   * Test 3: Graceful cleanup during chat switching
   */
  async testGracefulCleanup() {
    console.log('🧪 Test 3: Graceful cleanup during chat switching');

    try {
      // Create tracking context with partial progress
      const trackingId = this.guarantee.startMessageTracking(this.mockChatId, this.mockMessageIds);

      // Mark some messages as displayed (partial progress)
      this.guarantee.markMessageDisplayed(1001, null, this.mockChatId);
      this.guarantee.markMessageDisplayed(1002, null, this.mockChatId);

      const initialContext = this.guarantee.verificationQueue.get(trackingId);
      const initialDisplayed = initialContext.displayedIds.size;

      // Trigger graceful cleanup
      const clearedCount = this.guarantee.clearTrackingForChat(this.mockChatId);

      // Context should still exist due to partial progress
      const contextStillExists = this.guarantee.verificationQueue.has(trackingId);

      this.testResults.push({
        test: 'Graceful Cleanup',
        status: contextStillExists ? 'PASS' : 'FAIL',
        details: `Context preserved due to partial progress (${initialDisplayed} messages displayed)`
      });

      // Force cleanup after test
      setTimeout(() => {
        this.guarantee.clearAllTracking();
      }, 100);

      return contextStillExists;
    } catch (error) {
      this.testResults.push({
        test: 'Graceful Cleanup',
        status: 'FAIL',
        error: error.message
      });
      return false;
    }
  }

  /**
   * Test 4: Error throttling mechanism
   */
  async testErrorThrottling() {
    console.log('🧪 Test 4: Error throttling mechanism');

    try {
      const originalConsoleError = console.error;
      let errorCount = 0;

      // Mock console.error to count calls
      console.error = (...args) => {
        if (args[0] && args[0].includes('NO TRACKING CONTEXT FOUND')) {
          errorCount++;
        }
        originalConsoleError.apply(console, args);
      };

      // Trigger multiple "orphaned" message display events
      for (let i = 0; i < 10; i++) {
        this.guarantee.markMessageDisplayed(3000 + i, null, this.mockChatId);
      }

      // Restore console.error
      console.error = originalConsoleError;

      // Error should be throttled (not logged 10 times)
      const throttlingWorking = errorCount < 10;

      this.testResults.push({
        test: 'Error Throttling',
        status: throttlingWorking ? 'PASS' : 'FAIL',
        details: `Logged ${errorCount}/10 errors (throttling ${throttlingWorking ? 'working' : 'not working'})`
      });

      return throttlingWorking;
    } catch (error) {
      this.testResults.push({
        test: 'Error Throttling',
        status: 'FAIL',
        error: error.message
      });
      return false;
    }
  }

  /**
   * Test 5: Cross-chat context isolation
   */
  async testChatIsolation() {
    console.log('🧪 Test 5: Cross-chat context isolation');

    try {
      const chatId1 = 100;
      const chatId2 = 200;

      // Create tracking contexts for different chats
      const trackingId1 = this.guarantee.startMessageTracking(chatId1, [1001, 1002]);
      const trackingId2 = this.guarantee.startMessageTracking(chatId2, [2001, 2002]);

      // Mark message in chat1 from chat2 context (should not interfere)
      this.guarantee.markMessageDisplayed(1001, null, chatId2);

      // Check that chat1 context was not affected
      const context1 = this.guarantee.verificationQueue.get(trackingId1);
      const chat1Displayed = context1.displayedIds.size;

      // Mark message in correct chat
      this.guarantee.markMessageDisplayed(1001, null, chatId1);
      const context1After = this.guarantee.verificationQueue.get(trackingId1);
      const chat1DisplayedAfter = context1After.displayedIds.size;

      const isolationWorking = chat1Displayed === 0 && chat1DisplayedAfter === 1;

      this.testResults.push({
        test: 'Chat Isolation',
        status: isolationWorking ? 'PASS' : 'FAIL',
        details: `Cross-chat interference ${isolationWorking ? 'prevented' : 'detected'}`
      });

      // Cleanup
      this.guarantee.clearTrackingForChat(chatId1);
      this.guarantee.clearTrackingForChat(chatId2);

      return isolationWorking;
    } catch (error) {
      this.testResults.push({
        test: 'Chat Isolation',
        status: 'FAIL',
        error: error.message
      });
      return false;
    }
  }

  /**
   * Run all validation tests
   */
  async runAllTests() {
    console.log('🧪 Starting MessageDisplayGuarantee validation tests...');

    const initialized = await this.initialize();
    if (!initialized) {
      return { success: false, error: 'Failed to initialize' };
    }

    const tests = [
      this.testBasicTracking,
      this.testFallbackContext,
      this.testGracefulCleanup,
      this.testErrorThrottling,
      this.testChatIsolation
    ];

    let passCount = 0;

    for (const test of tests) {
      try {
        const result = await test.call(this);
        if (result) passCount++;

        // Small delay between tests
        await new Promise(resolve => setTimeout(resolve, 100));
      } catch (error) {
        console.error('Test execution error:', error);
      }
    }

    const totalTests = tests.length;
    const success = passCount === totalTests;

    console.log(`🧪 Validation complete: ${passCount}/${totalTests} tests passed`);

    return {
      success,
      passCount,
      totalTests,
      results: this.testResults,
      summary: this.generateSummary()
    };
  }

  /**
   * Generate test summary
   */
  generateSummary() {
    const passed = this.testResults.filter(r => r.status === 'PASS').length;
    const failed = this.testResults.filter(r => r.status === 'FAIL').length;

    return {
      total: this.testResults.length,
      passed,
      failed,
      successRate: ((passed / this.testResults.length) * 100).toFixed(1) + '%'
    };
  }

  /**
   * Export detailed test report
   */
  exportReport() {
    return {
      timestamp: new Date().toISOString(),
      tests: this.testResults,
      summary: this.generateSummary(),
      systemInfo: this.getSystemInfo()
    };
  }

  /**
   * Get system information for debugging
   */
  getSystemInfo() {
    return {
      userAgent: navigator.userAgent,
      timestamp: Date.now(),
      guaranteeMetrics: this.guarantee ? this.guarantee.getMetrics() : null,
      activeContexts: this.guarantee ? this.guarantee.verificationQueue.size : 0
    };
  }
}

// Create global instance for testing
export const messageDisplayValidator = new MessageDisplayGuaranteeValidator();

// Make available globally for debugging
if (typeof window !== 'undefined') {
  window.validateMessageDisplayGuarantee = async () => {
    return await messageDisplayValidator.runAllTests();
  };

  window.messageDisplayValidator = messageDisplayValidator;
}

export default messageDisplayValidator; 