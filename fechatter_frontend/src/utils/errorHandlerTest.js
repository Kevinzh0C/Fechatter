/**
 * Error Handler Test
 * Test the missing showNotification method and related functionality
 */

import { errorHandler } from './errorHandler';

export class ErrorHandlerTest {
  constructor() {
    this.testResults = [];
  }

  /**
   * Run comprehensive error handler tests
   */
  async runTests() {
    console.group('🧪 Error Handler Test Suite');
    console.log('Testing error handler showNotification method...');

    this.testResults = [];

    // Test 1: Method Existence
    await this.testMethodExistence();

    // Test 2: Basic Notification Types
    await this.testBasicNotificationTypes();

    // Test 3: Fallback Behavior
    await this.testFallbackBehavior();

    // Test 4: Options Handling
    await this.testOptionsHandling();

    // Test 5: Integration Test
    await this.testIntegration();

    // Generate report
    this.generateTestReport();

    console.groupEnd();
    return this.testResults;
  }

  /**
   * Test if showNotification method exists
   */
  async testMethodExistence() {
    console.log('\n🔍 Test 1: Method Existence');

    try {
      const hasMethod = typeof errorHandler.showNotification === 'function';

      if (hasMethod) {
        console.log('  ✅ showNotification method exists');
        this.recordResult('method_existence', true, 'showNotification method is available');
      } else {
        console.log('  ❌ showNotification method missing');
        this.recordResult('method_existence', false, 'showNotification method not found');
      }
    } catch (error) {
      console.log('  ❌ Error checking method existence:', error.message);
      this.recordResult('method_existence', false, error.message);
    }
  }

  /**
   * Test basic notification types
   */
  async testBasicNotificationTypes() {
    console.log('\n📝 Test 2: Basic Notification Types');

    const testTypes = ['success', 'info', 'warning', 'error'];
    let passed = 0;

    for (const type of testTypes) {
      try {
        console.log(`  Testing ${type} notification...`);

        // Try to call the method
        errorHandler.showNotification(`Test ${type} message`, type);

        console.log(`  ✅ ${type} notification sent successfully`);
        passed++;
      } catch (error) {
        console.log(`  ❌ ${type} notification failed:`, error.message);
      }
    }

    const success = passed === testTypes.length;
    this.recordResult('basic_notification_types', success, `${passed}/${testTypes.length} notification types working`);
  }

  /**
   * Test fallback behavior when notifications are not available
   */
  async testFallbackBehavior() {
    console.log('\n🔄 Test 3: Fallback Behavior');

    try {
      // Store original console.log
      const originalConsoleLog = console.log;
      let fallbackCalled = false;

      // Override console.log to detect fallback
      console.log = (...args) => {
        const message = args.join(' ');
        if (message.includes('[NOTIFICATION')) {
          fallbackCalled = true;
          console.debug('🔇 [TEST] Fallback notification detected:', message);
        }
        // Call original for other logs
        originalConsoleLog.apply(console, args);
      };

      // Test with uninitialized notifications (simulate edge case)
      const tempNotifications = errorHandler.notifications;
      errorHandler.notifications = null;

      errorHandler.showNotification('Test fallback message', 'info');

      // Restore
      errorHandler.notifications = tempNotifications;
      console.log = originalConsoleLog;

      if (fallbackCalled) {
        console.log('  ✅ Fallback behavior working');
        this.recordResult('fallback_behavior', true, 'Console fallback activated when notifications unavailable');
      } else {
        console.log('  ⚠️ Fallback behavior not detected');
        this.recordResult('fallback_behavior', false, 'Fallback behavior not triggered');
      }
    } catch (error) {
      console.log('  ❌ Fallback test failed:', error.message);
      this.recordResult('fallback_behavior', false, error.message);
    }
  }

  /**
   * Test options handling
   */
  async testOptionsHandling() {
    console.log('\n⚙️ Test 4: Options Handling');

    try {
      // Test with custom options
      const testOptions = {
        title: 'Custom Title',
        duration: 3000,
        actions: [{ label: 'Test Action', handler: () => { } }]
      };

      errorHandler.showNotification('Test message with options', 'success', testOptions);

      console.log('  ✅ Options handling successful');
      this.recordResult('options_handling', true, 'Custom options processed without error');
    } catch (error) {
      console.log('  ❌ Options handling failed:', error.message);
      this.recordResult('options_handling', false, error.message);
    }
  }

  /**
   * Test integration with actual auth store scenarios
   */
  async testIntegration() {
    console.log('\n🔗 Test 5: Integration Test');

    try {
      // Simulate the exact calls from auth.js
      console.log('  Simulating auth.js scenario 1...');
      errorHandler.showNotification('Test info message', 'info');

      console.log('  Simulating auth.js scenario 2...');
      errorHandler.showNotification('Password changed successfully', 'success');

      console.log('  ✅ Integration test successful');
      this.recordResult('integration_test', true, 'Auth store scenarios working correctly');
    } catch (error) {
      console.log('  ❌ Integration test failed:', error.message);
      this.recordResult('integration_test', false, error.message);
    }
  }

  /**
   * Record test result
   */
  recordResult(testName, success, details) {
    this.testResults.push({
      test: testName,
      success,
      details,
      timestamp: new Date().toISOString()
    });
  }

  /**
   * Generate comprehensive test report
   */
  generateTestReport() {
    console.log('\n📊 Error Handler Test Report');

    const passed = this.testResults.filter(r => r.success).length;
    const total = this.testResults.length;
    const successRate = total > 0 ? (passed / total) * 100 : 0;

    console.log(`Overall Success Rate: ${successRate.toFixed(1)}% (${passed}/${total})`);

    this.testResults.forEach(result => {
      const status = result.success ? '✅ PASS' : '❌ FAIL';
      console.log(`  ${status} ${result.test}: ${result.details}`);
    });

    if (successRate === 100) {
      console.log('\n🎉 All error handler tests passed!');
      console.log('💡 The showNotification method is working correctly');
      console.log('🔧 The original TypeError should be resolved');
    } else {
      console.log('\n⚠️ Some tests failed - error handler may need additional fixes');
      console.log('🔧 Check the failed tests above');
    }

    return {
      passed,
      total,
      successRate,
      results: this.testResults
    };
  }
}

/**
 * Test the specific error case from user report
 */
export function testCurrentError() {
  console.group('🧪 Current Error Fix Test');

  console.log('Testing the specific TypeError that was reported...');
  console.log('Error: errorHandler.showNotification is not a function');

  try {
    // Test the exact scenarios that were failing
    console.log('\n1. Testing info notification:');
    errorHandler.showNotification('Test info message', 'info');
    console.log('   ✅ Info notification successful');

    console.log('\n2. Testing success notification:');
    errorHandler.showNotification('Password changed successfully', 'success');
    console.log('   ✅ Success notification successful');

    console.log('\n✅ Current error has been fixed!');
    console.log('💡 errorHandler.showNotification is now a function');
    console.log('🔧 The unhandled promise rejection should be resolved');

  } catch (error) {
    console.log('\n❌ Current error still exists:', error.message);
    console.log('🔧 Additional fixes may be needed');
  }

  console.groupEnd();
}

/**
 * Run error handler test suite
 */
export async function runErrorHandlerTest() {
  const tester = new ErrorHandlerTest();
  return await tester.runTests();
}

// Auto-run in development
// Disabled automatic test execution to prevent test-induced side effects
// Tests can still be run manually via window functions
/*
if (import.meta.env.DEV && typeof window !== 'undefined') {
  setTimeout(() => {
    console.log('🔧 [DEV] Running error handler test...');
    testCurrentError();

    setTimeout(() => {
      runErrorHandlerTest();
    }, 1000);
  }, 2000);
}
*/

// Expose to window for manual testing
if (typeof window !== 'undefined') {
  window.runErrorHandlerTest = runErrorHandlerTest;
  window.testCurrentError = testCurrentError;
  window.ErrorHandlerTest = ErrorHandlerTest;

  console.log('🛠️ Error handler tests available for manual execution:');
  console.log('  - window.testCurrentError() - Test specific error fix');
  console.log('  - window.runErrorHandlerTest() - Run full test suite');
}

export default ErrorHandlerTest; 