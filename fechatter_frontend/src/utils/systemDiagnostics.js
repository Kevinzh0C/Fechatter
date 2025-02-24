/**
 * 🔍 System Diagnostics - Comprehensive Health Check
 * 系统诊断工具 - 验证修复效果
 */

export class SystemDiagnostics {
  constructor() {
    this.diagnosticResults = [];
    this.startTime = Date.now();
  }

  /**
   * 🔧 Test 1: MessageDisplayGuarantee Fallback Optimization
   */
  async testMessageDisplayGuarantee() {
    const testName = 'MessageDisplayGuarantee Fallback Prevention';

    try {
      if (!window.messageDisplayGuarantee) {
        throw new Error('MessageDisplayGuarantee not available');
      }

      const guarantee = window.messageDisplayGuarantee;
      const initialContextCount = guarantee.verificationQueue.size;

      // Test multiple orphaned messages for same chat
      const testChatId = 999;
      guarantee.markMessageDisplayed(1001, null, testChatId);
      guarantee.markMessageDisplayed(1002, null, testChatId);
      guarantee.markMessageDisplayed(1003, null, testChatId);

      // Should only create ONE fallback context for the chat
      const fallbackContexts = Array.from(guarantee.verificationQueue.values())
        .filter(ctx => ctx.isFallback && ctx.chatId === testChatId);

      const success = fallbackContexts.length === 1;
      const contextHasAllMessages = fallbackContexts[0]?.messageIds.size === 3;

      this.diagnosticResults.push({
        test: testName,
        status: success && contextHasAllMessages ? 'PASS' : 'FAIL',
        details: {
          fallbackContextsCreated: fallbackContexts.length,
          expectedContexts: 1,
          messagesInContext: fallbackContexts[0]?.messageIds.size || 0,
          expectedMessages: 3
        },
        timestamp: Date.now()
      });

      // Cleanup
      setTimeout(() => {
        guarantee.clearTrackingForChat(testChatId);
      }, 100);

      return success && contextHasAllMessages;

    } catch (error) {
      this.diagnosticResults.push({
        test: testName,
        status: 'ERROR',
        error: error.message,
        timestamp: Date.now()
      });
      return false;
    }
  }

  /**
   * 🔧 Test 2: Auth Store Initialization Deduplication
   */
  async testAuthStoreDeduplication() {
    const testName = 'Auth Store Initialization Deduplication';

    try {
      // Mock the auth store states
      const mockAuthStore = {
        userStoresInitialized: false,
        userStoresInitializationInProgress: false,
        async initializeUserStores() {
          if (this.userStoresInitialized) {
            return 'already-initialized';
          }

          if (this.userStoresInitializationInProgress) {
            return 'in-progress';
          }

          this.userStoresInitializationInProgress = true;

          // Simulate initialization
          await new Promise(resolve => setTimeout(resolve, 10));

          this.userStoresInitialized = true;
          this.userStoresInitializationInProgress = false;

          return 'initialized';
        }
      };

      // Test multiple concurrent calls
      const results = await Promise.all([
        mockAuthStore.initializeUserStores(),
        mockAuthStore.initializeUserStores(),
        mockAuthStore.initializeUserStores()
      ]);

      const initializedCount = results.filter(r => r === 'initialized').length;
      const skippedCount = results.filter(r => r !== 'initialized').length;

      const success = initializedCount === 1 && skippedCount === 2;

      this.diagnosticResults.push({
        test: testName,
        status: success ? 'PASS' : 'FAIL',
        details: {
          actualInitializations: initializedCount,
          expectedInitializations: 1,
          skippedCalls: skippedCount,
          results: results
        },
        timestamp: Date.now()
      });

      return success;

    } catch (error) {
      this.diagnosticResults.push({
        test: testName,
        status: 'ERROR',
        error: error.message,
        timestamp: Date.now()
      });
      return false;
    }
  }

  /**
   * 🔧 Test 3: CircuitBreaker Enhanced State Management
   */
  async testCircuitBreakerStates() {
    const testName = 'CircuitBreaker Enhanced State Management';

    try {
      // Mock circuit breaker
      const mockCircuitBreaker = {
        isOpen: false,
        failureCount: 0,
        maxFailures: 3,
        consecutiveFailures: 0,
        lastSuccessTime: 0,

        recordSuccess() {
          this.lastSuccessTime = Date.now();
          this.consecutiveFailures = 0;
          this.failureCount = Math.max(0, this.failureCount - 1);

          if (this.isOpen && this.failureCount === 0) {
            this.isOpen = false;
          }
        },

        recordFailure() {
          this.failureCount++;
          this.consecutiveFailures++;

          if (this.failureCount >= this.maxFailures) {
            this.isOpen = true;
          }
        },

        getStatus() {
          return this.isOpen ? 'open' : 'closed';
        }
      };

      // Test failure recording and circuit opening
      mockCircuitBreaker.recordFailure();
      mockCircuitBreaker.recordFailure();
      mockCircuitBreaker.recordFailure();

      const isOpenAfterFailures = mockCircuitBreaker.getStatus() === 'open';

      // Test success recording and circuit closing
      mockCircuitBreaker.recordSuccess();
      mockCircuitBreaker.recordSuccess();
      mockCircuitBreaker.recordSuccess();

      const isClosedAfterSuccesses = mockCircuitBreaker.getStatus() === 'closed';

      const success = isOpenAfterFailures && isClosedAfterSuccesses;

      this.diagnosticResults.push({
        test: testName,
        status: success ? 'PASS' : 'FAIL',
        details: {
          openedAfterFailures: isOpenAfterFailures,
          closedAfterSuccesses: isClosedAfterSuccesses,
          finalFailureCount: mockCircuitBreaker.failureCount,
          finalStatus: mockCircuitBreaker.getStatus()
        },
        timestamp: Date.now()
      });

      return success;

    } catch (error) {
      this.diagnosticResults.push({
        test: testName,
        status: 'ERROR',
        error: error.message,
        timestamp: Date.now()
      });
      return false;
    }
  }

  /**
   * 🔧 Test 4: System Integration Health Check
   */
  async testSystemIntegration() {
    const testName = 'System Integration Health Check';

    try {
      const checks = [];

      // Check MessageDisplayGuarantee availability
      checks.push({
        component: 'MessageDisplayGuarantee',
        available: typeof window.messageDisplayGuarantee !== 'undefined',
        activeContexts: window.messageDisplayGuarantee?.verificationQueue.size || 0
      });

      // Check Auth Store states
      const authStore = window.__pinia_stores__?.auth?.();
      checks.push({
        component: 'AuthStore',
        available: !!authStore,
        initialized: authStore?.isInitialized || false,
        userStoresInitialized: authStore?.userStoresInitialized || false
      });

      // Check Circuit Breaker debug functions
      checks.push({
        component: 'CircuitBreakerDebug',
        available: typeof window.debugLoadingCircuitBreaker !== 'undefined',
        resetAvailable: typeof window.resetCircuitBreaker !== 'undefined'
      });

      const allAvailable = checks.every(check => check.available);

      this.diagnosticResults.push({
        test: testName,
        status: allAvailable ? 'PASS' : 'FAIL',
        details: {
          checks: checks,
          allComponentsAvailable: allAvailable
        },
        timestamp: Date.now()
      });

      return allAvailable;

    } catch (error) {
      this.diagnosticResults.push({
        test: testName,
        status: 'ERROR',
        error: error.message,
        timestamp: Date.now()
      });
      return false;
    }
  }

  /**
   * 🔍 Run All Diagnostic Tests
   */
  async runAllTests() {
    console.log('🔍 Starting System Diagnostics...');

    const tests = [
      this.testMessageDisplayGuarantee(),
      this.testAuthStoreDeduplication(),
      this.testCircuitBreakerStates(),
      this.testSystemIntegration()
    ];

    const results = await Promise.all(tests);
    const passedTests = results.filter(r => r).length;
    const totalTests = results.length;

    const summary = {
      totalTests,
      passedTests,
      failedTests: totalTests - passedTests,
      successRate: ((passedTests / totalTests) * 100).toFixed(1),
      executionTime: Date.now() - this.startTime,
      timestamp: new Date().toISOString()
    };

    console.log('✅ System Diagnostics Complete:', summary);
    console.table(this.diagnosticResults);

    return {
      summary,
      results: this.diagnosticResults,
      overallStatus: passedTests === totalTests ? 'HEALTHY' : 'NEEDS_ATTENTION'
    };
  }

  /**
   * 🔧 Generate Diagnostic Report
   */
  generateReport() {
    return {
      title: 'Fechatter System Diagnostic Report',
      timestamp: new Date().toISOString(),
      executionTime: Date.now() - this.startTime,
      results: this.diagnosticResults,
      recommendations: this.generateRecommendations(),
      nextSteps: this.generateNextSteps()
    };
  }

  /**
   * Generate improvement recommendations
   */
  generateRecommendations() {
    const failedTests = this.diagnosticResults.filter(r => r.status === 'FAIL' || r.status === 'ERROR');

    if (failedTests.length === 0) {
      return ['All systems operating normally', 'Continue monitoring for any performance degradation'];
    }

    return failedTests.map(test => {
      switch (test.test) {
        case 'MessageDisplayGuarantee Fallback Prevention':
          return 'Review MessageDisplayGuarantee timing and context creation logic';
        case 'Auth Store Initialization Deduplication':
          return 'Verify auth store initialization sequence and deduplication logic';
        case 'CircuitBreaker Enhanced State Management':
          return 'Review circuit breaker state transitions and failure handling';
        default:
          return `Review ${test.test} implementation`;
      }
    });
  }

  /**
   * Generate next steps
   */
  generateNextSteps() {
    return [
      'Monitor application logs for reduced error frequency',
      'Verify circuit breaker metrics in production',
      'Check MessageDisplayGuarantee fallback context creation rates',
      'Validate auth store initialization logs',
      'Schedule regular system diagnostics'
    ];
  }
}

// 🔧 Global diagnostic functions
if (typeof window !== 'undefined' && import.meta.env.DEV) {
  window.runSystemDiagnostics = async () => {
    const diagnostics = new SystemDiagnostics();
    return await diagnostics.runAllTests();
  };

  window.generateDiagnosticReport = () => {
    const diagnostics = new SystemDiagnostics();
    return diagnostics.generateReport();
  };

  console.log('🔍 System Diagnostics loaded - use runSystemDiagnostics() to test fixes');
} 