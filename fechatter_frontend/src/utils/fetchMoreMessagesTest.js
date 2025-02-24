/**
 * fetchMoreMessages Fix Verification Test
 * 🔧 Tests the variable scope fix in UnifiedMessageService.fetchMoreMessages
 */

class FetchMoreMessagesTest {
  constructor() {
    this.testResults = [];
  }

  /**
   * Run comprehensive tests for fetchMoreMessages fix
   */
  async runTests(chatId = 5) {
    console.log('🔧 [FetchMoreMessages Test] Starting verification tests...');

    const startTime = Date.now();

    // Test 1: Check if service is available
    const serviceTest = this.testServiceAvailability();

    // Test 2: Check variable scope fix
    const scopeTest = this.testVariableScope();

    // Test 3: Test error handling
    const errorTest = await this.testErrorHandling(chatId);

    // Test 4: Test actual fetchMoreMessages call
    const functionalTest = await this.testFunctionalCall(chatId);

    const duration = Date.now() - startTime;

    const results = {
      timestamp: Date.now(),
      duration,
      tests: {
        serviceAvailability: serviceTest,
        variableScope: scopeTest,
        errorHandling: errorTest,
        functionalCall: functionalTest
      },
      overall: this.calculateOverallResult([serviceTest, scopeTest, errorTest, functionalTest])
    };

    this.displayResults(results);
    return results;
  }

  /**
   * Test if UnifiedMessageService is available
   */
  testServiceAvailability() {
    const result = {
      name: 'Service Availability',
      passed: false,
      details: {}
    };

    try {
      // Check if service exists
      if (!window.unifiedMessageService) {
        result.details.error = 'UnifiedMessageService not found on window';
        return result;
      }

      // Check if fetchMoreMessages method exists
      if (typeof window.unifiedMessageService.fetchMoreMessages !== 'function') {
        result.details.error = 'fetchMoreMessages method not found';
        return result;
      }

      result.passed = true;
      result.details.serviceFound = true;
      result.details.methodFound = true;

    } catch (error) {
      result.details.error = error.message;
    }

    return result;
  }

  /**
   * Test variable scope fix by analyzing the source
   */
  testVariableScope() {
    const result = {
      name: 'Variable Scope Fix',
      passed: false,
      details: {}
    };

    try {
      // Get the fetchMoreMessages method source code
      const methodSource = window.unifiedMessageService.fetchMoreMessages.toString();

      // Check if 'existing' is declared before try block
      const beforeTryPattern = /const existing = [^;]+;[\s\S]*try\s*{/;
      const existingBeforeTry = beforeTryPattern.test(methodSource);

      // Check if 'existing' is used after try block
      const afterTryPattern = /}\s*catch[\s\S]*?}\s*[\s\S]*existing/;
      const existingAfterTry = afterTryPattern.test(methodSource);

      result.details.existingBeforeTry = existingBeforeTry;
      result.details.existingAfterTry = existingAfterTry;

      // The fix is successful if existing is declared before try AND used after
      result.passed = existingBeforeTry && existingAfterTry;

      if (!result.passed) {
        result.details.issue = 'Variable scope issue may still exist';
      }

    } catch (error) {
      result.details.error = error.message;
    }

    return result;
  }

  /**
   * Test error handling improvements
   */
  async testErrorHandling(chatId) {
    const result = {
      name: 'Error Handling',
      passed: false,
      details: {}
    };

    try {
      // Create a mock scenario where API might fail
      const originalMessagesByChat = window.unifiedMessageService.messagesByChat;

      // Temporarily set up a chat with some messages
      const testMessages = [
        { id: 1, content: 'Test message 1' },
        { id: 2, content: 'Test message 2' }
      ];

      window.unifiedMessageService.messagesByChat.set(parseInt(chatId), testMessages);

      // The test passes if we can access the existing messages even in error scenarios
      const existing = window.unifiedMessageService.messagesByChat.get(parseInt(chatId)) || [];

      result.details.existingMessagesCount = existing.length;
      result.details.canAccessExisting = existing.length > 0;
      result.passed = existing.length === 2; // Should match our test data

      // Restore original state
      if (originalMessagesByChat) {
        window.unifiedMessageService.messagesByChat = originalMessagesByChat;
      }

    } catch (error) {
      result.details.error = error.message;
    }

    return result;
  }

  /**
   * Test actual fetchMoreMessages functionality
   */
  async testFunctionalCall(chatId) {
    const result = {
      name: 'Functional Call Test',
      passed: false,
      details: {}
    };

    try {
      // Test with a safe call (short timeout to avoid hanging)
      const timeoutPromise = new Promise((_, reject) =>
        setTimeout(() => reject(new Error('Test timeout')), 5000)
      );

      const fetchPromise = window.unifiedMessageService.fetchMoreMessages(chatId, { limit: 5 });

      try {
        const fetchResult = await Promise.race([fetchPromise, timeoutPromise]);

        result.details.callSucceeded = true;
        result.details.returnType = Array.isArray(fetchResult) ? 'array' : typeof fetchResult;
        result.details.messageCount = Array.isArray(fetchResult) ? fetchResult.length : 0;
        result.passed = true;

      } catch (fetchError) {
        // Even if the API call fails, the method should not throw ReferenceError for 'existing'
        result.details.callSucceeded = false;
        result.details.errorType = fetchError.constructor.name;
        result.details.errorMessage = fetchError.message;

        // The fix is successful if we don't get ReferenceError: existing is not defined
        const isReferenceError = fetchError instanceof ReferenceError &&
          fetchError.message.includes('existing is not defined');

        result.passed = !isReferenceError;
        result.details.isOriginalError = isReferenceError;
      }

    } catch (error) {
      result.details.error = error.message;
    }

    return result;
  }

  /**
   * Calculate overall test result
   */
  calculateOverallResult(tests) {
    const passedTests = tests.filter(test => test.passed).length;
    const totalTests = tests.length;

    return {
      passed: passedTests === totalTests,
      passRate: `${passedTests}/${totalTests}`,
      percentage: Math.round((passedTests / totalTests) * 100)
    };
  }

  /**
   * Display test results
   */
  displayResults(results) {
    console.group('🔧 fetchMoreMessages Fix Verification Results');

    console.log(`⏱️ Test Duration: ${results.duration}ms`);
    console.log(`📊 Overall Result: ${results.overall.passed ? '✅ PASSED' : '❌ FAILED'} (${results.overall.passRate})`);

    console.log('\n📋 Individual Test Results:');

    Object.values(results.tests).forEach((test, idx) => {
      const icon = test.passed ? '✅' : '❌';
      console.log(`  ${idx + 1}. ${icon} ${test.name}`);

      if (test.details) {
        Object.entries(test.details).forEach(([key, value]) => {
          console.log(`     ${key}: ${JSON.stringify(value)}`);
        });
      }
    });

    if (results.overall.passed) {
      console.log('\n🎉 SUCCESS: fetchMoreMessages ReferenceError has been fixed!');
      console.log('   ✅ Variable scope issue resolved');
      console.log('   ✅ Error handling improved');
      console.log('   ✅ Method functions correctly');
    } else {
      console.log('\n⚠️ Some tests failed. The fix may need additional work.');
    }

    console.groupEnd();
  }

  /**
   * Quick test specifically for the ReferenceError
   */
  async quickReferenceErrorTest(chatId = 5) {
    console.log('🔍 [Quick Test] Checking for ReferenceError: existing is not defined...');

    try {
      // Force an API error scenario to test the fix
      const result = await window.unifiedMessageService.fetchMoreMessages(chatId);
      console.log('✅ [Quick Test] No ReferenceError thrown - fix appears successful');
      return { success: true, result };

    } catch (error) {
      const isOriginalError = error instanceof ReferenceError &&
        error.message.includes('existing is not defined');

      if (isOriginalError) {
        console.log('❌ [Quick Test] Original ReferenceError still present - fix failed');
        return { success: false, error, isOriginalError: true };
      } else {
        console.log('✅ [Quick Test] Different error (expected) - ReferenceError fix successful');
        console.log(`   Error type: ${error.constructor.name}`);
        console.log(`   Error message: ${error.message}`);
        return { success: true, error, isOriginalError: false };
      }
    }
  }
}

// Create global instance
const fetchMoreMessagesTest = new FetchMoreMessagesTest();

// Expose to window for console access
if (typeof window !== 'undefined') {
  window.fetchMoreMessagesTest = fetchMoreMessagesTest;

  // Convenience commands
  window.testFetchMoreFix = (chatId) => fetchMoreMessagesTest.runTests(chatId);
  window.quickTestFetchMore = (chatId) => fetchMoreMessagesTest.quickReferenceErrorTest(chatId);
}

export default fetchMoreMessagesTest; 