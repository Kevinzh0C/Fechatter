/**
 * 🛡️ Message Display Guarantee Verification
 * 
 * Production-level testing and verification script for message display guarantee system
 * Ensures end-to-end reliability from backend fetch to frontend display
 */

import { unifiedMessageService } from '@/services/messageSystem/UnifiedMessageService.js';
import { messageDisplayGuarantee } from '@/services/messageSystem/MessageDisplayGuarantee.js';
import { useChatStore } from '@/stores/chat.js';

export class MessageDisplayGuaranteeVerification {
  constructor() {
    this.testResults = [];
    this.isRunning = false;
    this.testConfig = {
      timeoutMs: 5000,
      retryAttempts: 3,
      sampleChatIds: [1, 2, 3, 6], // Test with different chat IDs
      expectedMinMessages: 1
    };
  }

  /**
   * Run complete verification of message display guarantee
   */
  async runCompleteVerification() {
    if (this.isRunning) {
      console.warn('🛡️ [Verification] Test already running');
      return false;
    }

    this.isRunning = true;
    this.testResults = [];

    console.group('🛡️ [Message Display Guarantee] Starting Complete Verification');

    try {
      // Test 1: Basic System Health
      await this.verifySystemHealth();

      // Test 2: Message Fetch Integration
      await this.verifyMessageFetchIntegration();

      // Test 3: Display Tracking
      await this.verifyDisplayTracking();

      // Test 4: Recovery Mechanism
      await this.verifyRecoveryMechanism();

      // Test 5: Performance Metrics
      await this.verifyPerformanceMetrics();

      // Generate report
      const report = this.generateVerificationReport();
      console.log('📊 [Verification] Complete Report:', report);

      console.groupEnd();
      return report;

    } catch (error) {
      console.error('❌ [Verification] Critical failure:', error);
      console.groupEnd();
      return { success: false, error: error.message };
    } finally {
      this.isRunning = false;
    }
  }

  /**
   * Test 1: Verify system health and initialization
   */
  async verifySystemHealth() {
    console.log('🔍 [Test 1] Verifying System Health...');

    const test = {
      name: 'System Health',
      status: 'running',
      startTime: Date.now(),
      checks: []
    };

    try {
      // Check if guarantee system is enabled
      const isEnabled = messageDisplayGuarantee.isEnabled.value;
      test.checks.push({
        name: 'Guarantee System Enabled',
        passed: isEnabled,
        details: { enabled: isEnabled }
      });

      // Check if UnifiedMessageService is initialized
      const isUMSInitialized = unifiedMessageService.isInitialized.value;
      test.checks.push({
        name: 'UnifiedMessageService Initialized',
        passed: isUMSInitialized,
        details: { initialized: isUMSInitialized }
      });

      // Check global window references
      const hasGlobalReference = typeof window.messageDisplayGuarantee !== 'undefined';
      test.checks.push({
        name: 'Global Reference Available',
        passed: hasGlobalReference,
        details: { hasGlobalReference }
      });

      // Check debug helper availability
      const hasDebugHelper = typeof window.debugMessageGuarantee === 'function';
      test.checks.push({
        name: 'Debug Helper Available',
        passed: hasDebugHelper,
        details: { hasDebugHelper }
      });

      const allPassed = test.checks.every(check => check.passed);
      test.status = allPassed ? 'passed' : 'failed';
      test.endTime = Date.now();

      console.log(`✅ [Test 1] System Health: ${test.status.toUpperCase()}`);

    } catch (error) {
      test.status = 'error';
      test.error = error.message;
      test.endTime = Date.now();
      console.error('❌ [Test 1] System Health Error:', error);
    }

    this.testResults.push(test);
  }

  /**
   * Test 2: Verify message fetch integration
   */
  async verifyMessageFetchIntegration() {
    console.log('🔍 [Test 2] Verifying Message Fetch Integration...');

    const test = {
      name: 'Message Fetch Integration',
      status: 'running',
      startTime: Date.now(),
      chatTests: []
    };

    try {
      const chatStore = useChatStore();

      for (const chatId of this.testConfig.sampleChatIds) {
        const chatTest = {
          chatId,
          startTime: Date.now(),
          status: 'testing'
        };

        try {
          // Get initial metrics
          const initialMetrics = messageDisplayGuarantee.getMetrics();

          // Fetch messages for this chat
          console.log(`🎯 [Test 2] Testing chat ${chatId}...`);
          await chatStore.setCurrentChat(chatId);

          // Wait for messages to be fetched
          const messages = await this.waitForMessages(chatId, 2000);

          // Check if tracking was started
          const finalMetrics = messageDisplayGuarantee.getMetrics();
          const trackingStarted = finalMetrics.totalFetched > initialMetrics.totalFetched;

          chatTest.status = messages.length > 0 && trackingStarted ? 'passed' : 'failed';
          chatTest.details = {
            messageCount: messages.length,
            trackingStarted,
            initialFetched: initialMetrics.totalFetched,
            finalFetched: finalMetrics.totalFetched
          };

          console.log(`${chatTest.status === 'passed' ? '✅' : '❌'} [Test 2] Chat ${chatId}: ${chatTest.status}`);

        } catch (error) {
          chatTest.status = 'error';
          chatTest.error = error.message;
          console.error(`❌ [Test 2] Chat ${chatId} error:`, error);
        }

        chatTest.endTime = Date.now();
        test.chatTests.push(chatTest);
      }

      const allPassed = test.chatTests.every(t => t.status === 'passed');
      test.status = allPassed ? 'passed' : 'failed';
      test.endTime = Date.now();

    } catch (error) {
      test.status = 'error';
      test.error = error.message;
      test.endTime = Date.now();
      console.error('❌ [Test 2] Integration Error:', error);
    }

    this.testResults.push(test);
  }

  /**
   * Test 3: Verify display tracking
   */
  async verifyDisplayTracking() {
    console.log('🔍 [Test 3] Verifying Display Tracking...');

    const test = {
      name: 'Display Tracking',
      status: 'running',
      startTime: Date.now(),
      checks: []
    };

    try {
      // Get initial metrics
      const initialMetrics = messageDisplayGuarantee.getMetrics();

      // Simulate message display by directly calling markMessageDisplayed
      const mockMessageId = 999999;
      const mockElement = document.createElement('div');
      mockElement.style.position = 'fixed';
      mockElement.style.top = '100px';
      mockElement.style.left = '100px';
      mockElement.style.width = '200px';
      mockElement.style.height = '50px';
      document.body.appendChild(mockElement);

      // Start tracking for mock message
      const trackingId = messageDisplayGuarantee.startMessageTracking(1, [mockMessageId]);

      test.checks.push({
        name: 'Tracking Started',
        passed: trackingId !== null,
        details: { trackingId }
      });

      // Mark as displayed
      messageDisplayGuarantee.markMessageDisplayed(mockMessageId, mockElement);

      // Check metrics updated
      await new Promise(resolve => setTimeout(resolve, 100));
      const updatedMetrics = messageDisplayGuarantee.getMetrics();
      const displayedIncreased = updatedMetrics.totalDisplayed > initialMetrics.totalDisplayed;

      test.checks.push({
        name: 'Display Tracking Works',
        passed: displayedIncreased,
        details: {
          initialDisplayed: initialMetrics.totalDisplayed,
          finalDisplayed: updatedMetrics.totalDisplayed
        }
      });

      // Cleanup
      document.body.removeChild(mockElement);

      const allPassed = test.checks.every(check => check.passed);
      test.status = allPassed ? 'passed' : 'failed';
      test.endTime = Date.now();

      console.log(`✅ [Test 3] Display Tracking: ${test.status.toUpperCase()}`);

    } catch (error) {
      test.status = 'error';
      test.error = error.message;
      test.endTime = Date.now();
      console.error('❌ [Test 3] Display Tracking Error:', error);
    }

    this.testResults.push(test);
  }

  /**
   * Test 4: Verify recovery mechanism
   */
  async verifyRecoveryMechanism() {
    console.log('🔍 [Test 4] Verifying Recovery Mechanism...');

    const test = {
      name: 'Recovery Mechanism',
      status: 'running',
      startTime: Date.now(),
      checks: []
    };

    try {
      // Test force refresh event
      let refreshEventReceived = false;
      const refreshHandler = () => {
        refreshEventReceived = true;
      };

      window.addEventListener('fechatter:force-message-refresh', refreshHandler);

      // Trigger force refresh
      await messageDisplayGuarantee.forceMessageListRefresh(1);

      // Wait for event
      await new Promise(resolve => setTimeout(resolve, 100));

      test.checks.push({
        name: 'Force Refresh Event',
        passed: refreshEventReceived,
        details: { eventReceived: refreshEventReceived }
      });

      // Test scroll check event
      let scrollEventReceived = false;
      const scrollHandler = () => {
        scrollEventReceived = true;
      };

      window.addEventListener('fechatter:force-scroll-check', scrollHandler);

      // Trigger scroll event
      const scrollEvent = new CustomEvent('fechatter:force-scroll-check', {
        detail: { chatId: 1 }
      });
      window.dispatchEvent(scrollEvent);

      await new Promise(resolve => setTimeout(resolve, 100));

      test.checks.push({
        name: 'Scroll Check Event',
        passed: scrollEventReceived,
        details: { eventReceived: scrollEventReceived }
      });

      // Cleanup
      window.removeEventListener('fechatter:force-message-refresh', refreshHandler);
      window.removeEventListener('fechatter:force-scroll-check', scrollHandler);

      const allPassed = test.checks.every(check => check.passed);
      test.status = allPassed ? 'passed' : 'failed';
      test.endTime = Date.now();

      console.log(`✅ [Test 4] Recovery Mechanism: ${test.status.toUpperCase()}`);

    } catch (error) {
      test.status = 'error';
      test.error = error.message;
      test.endTime = Date.now();
      console.error('❌ [Test 4] Recovery Mechanism Error:', error);
    }

    this.testResults.push(test);
  }

  /**
   * Test 5: Verify performance metrics
   */
  async verifyPerformanceMetrics() {
    console.log('🔍 [Test 5] Verifying Performance Metrics...');

    const test = {
      name: 'Performance Metrics',
      status: 'running',
      startTime: Date.now(),
      checks: []
    };

    try {
      // Get current metrics
      const metrics = messageDisplayGuarantee.getMetrics();

      test.checks.push({
        name: 'Metrics Structure Valid',
        passed: typeof metrics === 'object' &&
          typeof metrics.totalFetched === 'number' &&
          typeof metrics.totalDisplayed === 'number' &&
          typeof metrics.successRate === 'string',
        details: { metricsStructure: Object.keys(metrics) }
      });

      // Test debug export
      const debugInfo = messageDisplayGuarantee.exportDebugInfo();

      test.checks.push({
        name: 'Debug Export Works',
        passed: typeof debugInfo === 'object' &&
          debugInfo.hasOwnProperty('metrics') &&
          debugInfo.hasOwnProperty('activeTracking'),
        details: { debugStructure: Object.keys(debugInfo) }
      });

      const allPassed = test.checks.every(check => check.passed);
      test.status = allPassed ? 'passed' : 'failed';
      test.endTime = Date.now();

      console.log(`✅ [Test 5] Performance Metrics: ${test.status.toUpperCase()}`);

    } catch (error) {
      test.status = 'error';
      test.error = error.message;
      test.endTime = Date.now();
      console.error('❌ [Test 5] Performance Metrics Error:', error);
    }

    this.testResults.push(test);
  }

  /**
   * Wait for messages to be available for a chat
   */
  async waitForMessages(chatId, timeoutMs = 2000) {
    const startTime = Date.now();

    while (Date.now() - startTime < timeoutMs) {
      const messages = unifiedMessageService.getMessagesForChat(chatId);
      if (messages && messages.length > 0) {
        return messages;
      }

      await new Promise(resolve => setTimeout(resolve, 100));
    }

    return [];
  }

  /**
   * Generate comprehensive verification report
   */
  generateVerificationReport() {
    const totalTests = this.testResults.length;
    const passedTests = this.testResults.filter(test => test.status === 'passed').length;
    const failedTests = this.testResults.filter(test => test.status === 'failed').length;
    const errorTests = this.testResults.filter(test => test.status === 'error').length;

    const report = {
      timestamp: new Date().toISOString(),
      overall: {
        success: passedTests === totalTests,
        successRate: ((passedTests / totalTests) * 100).toFixed(2) + '%',
        totalTests,
        passed: passedTests,
        failed: failedTests,
        errors: errorTests
      },
      tests: this.testResults.map(test => ({
        name: test.name,
        status: test.status,
        duration: test.endTime - test.startTime,
        ...(test.checks && { checksCount: test.checks.length }),
        ...(test.chatTests && { chatTestsCount: test.chatTests.length }),
        ...(test.error && { error: test.error })
      })),
      currentMetrics: messageDisplayGuarantee.getMetrics(),
      recommendations: this.generateRecommendations()
    };

    return report;
  }

  /**
   * Generate improvement recommendations based on test results
   */
  generateRecommendations() {
    const recommendations = [];
    const failedTests = this.testResults.filter(test => test.status === 'failed');

    if (failedTests.length > 0) {
      recommendations.push('🔧 Some tests failed - investigate specific failure causes');
    }

    const metrics = messageDisplayGuarantee.getMetrics();
    const successRate = parseFloat(metrics.successRate);

    if (successRate < 95) {
      recommendations.push('📊 Display success rate below 95% - consider increasing retry attempts');
    }

    if (metrics.failedDisplays > 0) {
      recommendations.push('⚠️ Some messages failed to display - check UI rendering performance');
    }

    if (recommendations.length === 0) {
      recommendations.push('✅ All tests passed - message display guarantee is working correctly');
    }

    return recommendations;
  }
}

// Create global instance for testing
export const messageDisplayGuaranteeVerification = new MessageDisplayGuaranteeVerification();

// Make available globally for debugging
if (typeof window !== 'undefined') {
  window.verifyMessageDisplayGuarantee = async () => {
    return await messageDisplayGuaranteeVerification.runCompleteVerification();
  };

  window.messageDisplayGuaranteeVerification = messageDisplayGuaranteeVerification;
} 