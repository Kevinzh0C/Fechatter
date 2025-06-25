/**
 * Analytics Integration Test Utility
 * Tests the complete analytics pipeline from frontend to analytics_server
 */

import { analytics } from '../lib/analytics.ts';
import { getApiConfig } from './config.js';

class AnalyticsIntegrationTest {
  constructor() {
    this.results = [];
    this.apiConfig = getApiConfig();
    this.startTime = Date.now();
  }

  /**
   * Log test result
   */
  log(testName, success, message, data = null) {
    const result = {
      testName,
      success,
      message,
      data,
      timestamp: new Date().toISOString(),
      duration: Date.now() - this.startTime,
    };

    this.results.push(result);

    const emoji = success ? '✅' : '❌';
    console.log(`${emoji} [ANALYTICS_TEST] ${testName}: ${message}`);

    if (data) {
      console.log('  📊 Data:', data);
    }
  }

  /**
   * Test analytics client initialization
   */
  testAnalyticsClientInit() {
    try {
      const isEnabled = analytics.config?.enabled;
      const hasEndpoint = analytics.config?.endpoint;
      const hasClientId = analytics.client_id;

      this.log(
        'Analytics Client Initialization',
        isEnabled && hasEndpoint && hasClientId,
        isEnabled && hasEndpoint && hasClientId
          ? 'Analytics client initialized successfully'
          : 'Analytics client initialization failed',
        {
          enabled: isEnabled,
          endpoint: hasEndpoint,
          clientId: hasClientId,
          config: analytics.config
        }
      );
    } catch (error) {
      this.log(
        'Analytics Client Initialization',
        false,
        `Failed to initialize analytics client: ${error.message}`,
        { error: error.message }
      );
    }
  }

  /**
   * Test Gateway analytics route connectivity
   */
  async testGatewayConnectivity() {
    try {
      const analyticsEndpoint = analytics.config?.endpoint;
      if (!analyticsEndpoint) {
        throw new Error('Analytics endpoint not configured');
      }

      // Test health endpoint through Gateway
      const healthUrl = `${analyticsEndpoint.replace('/api/analytics', '')}/health`;
      const response = await fetch(healthUrl, {
        method: 'GET',
        headers: {
          'Accept': 'application/json',
        }
      });

      const success = response.ok;
      const responseData = await response.text();

      this.log(
        'Gateway Analytics Route Connectivity',
        success,
        success
          ? 'Gateway successfully routes to analytics services'
          : `Gateway routing failed: ${response.status}`,
        {
          healthUrl,
          status: response.status,
          response: responseData
        }
      );
    } catch (error) {
      this.log(
        'Gateway Analytics Route Connectivity',
        false,
        `Gateway connectivity test failed: ${error.message}`,
        { error: error.message }
      );
    }
  }

  /**
   * Test analytics server direct connectivity
   */
  async testAnalyticsServerConnectivity() {
    try {
      // Test analytics server health directly
      const response = await fetch('http://127.0.0.1:6690/health', {
        method: 'GET',
        headers: {
          'Accept': 'application/json',
        }
      });

      const success = response.ok;
      const responseData = await response.text();

      this.log(
        'Analytics Server Direct Connectivity',
        success,
        success
          ? 'Analytics server is running and healthy'
          : `Analytics server unreachable: ${response.status}`,
        {
          status: response.status,
          response: responseData
        }
      );
    } catch (error) {
      this.log(
        'Analytics Server Direct Connectivity',
        false,
        `Analytics server connectivity test failed: ${error.message}`,
        { error: error.message }
      );
    }
  }

  /**
   * Test sending a sample analytics event
   */
  async testEventSending() {
    try {
      // Create a test event
      const testEvent = {
        context: {
          client_id: 'test_client_123',
          user_id: 'test_user_123',
          app_version: '1.0.0',
          client_ts: Date.now(),
          server_ts: 0,
          user_agent: navigator.userAgent,
          ip: '',
          system: {
            os: 'test',
            arch: 'x64',
            locale: 'en-US',
            timezone: 'UTC',
            browser: 'Test',
            browser_version: '1.0',
          },
        },
        event_type: {
          navigation: {
            from: '/test-from',
            to: '/test-to',
            duration_ms: 100,
          },
        },
      };

      // Send event through Gateway
      const response = await fetch(`${analytics.config.endpoint}/event`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(testEvent),
      });

      const success = response.ok;
      const responseData = response.ok ? await response.json() : await response.text();

      this.log(
        'Analytics Event Sending',
        success,
        success
          ? 'Test analytics event sent successfully'
          : `Failed to send analytics event: ${response.status}`,
        {
          status: response.status,
          response: responseData,
          event: testEvent
        }
      );
    } catch (error) {
      this.log(
        'Analytics Event Sending',
        false,
        `Analytics event sending test failed: ${error.message}`,
        { error: error.message }
      );
    }
  }

  /**
   * Test analytics client method functionality
   */
  async testAnalyticsClientMethods() {
    try {
      // Test setting user ID
      analytics.setUserId('test_user_123');

      // Test message sent tracking (without actually sending)
      await analytics.trackMessageSent('test_chat_123', 'Test message content', []);

      // Test navigation tracking
      await analytics.trackNavigation('/test-from', '/test-to', Date.now() - 100);

      // Test error tracking
      await analytics.trackError(
        new Error('Test error'),
        'Analytics integration test',
        'TestError'
      );

      // Test search tracking
      await analytics.trackSearch('messages', 'test query', 5, 250, false);

      this.log(
        'Analytics Client Methods',
        true,
        'All analytics client methods executed successfully',
        {
          methods: ['setUserId', 'trackMessageSent', 'trackNavigation', 'trackError', 'trackSearch']
        }
      );
    } catch (error) {
      this.log(
        'Analytics Client Methods',
        false,
        `Analytics client methods test failed: ${error.message}`,
        { error: error.message }
      );
    }
  }

  /**
   * Test analytics batch functionality
   */
  async testBatchFunctionality() {
    try {
      // Force batch size to be small for testing
      const originalBatchSize = analytics.config.batch_size;
      analytics.config.batch_size = 2;

      // Add multiple events to trigger batch sending
      await analytics.trackNavigation('/batch-test-1', '/batch-test-2', Date.now() - 50);
      await analytics.trackNavigation('/batch-test-2', '/batch-test-3', Date.now() - 30);

      // Restore original batch size
      analytics.config.batch_size = originalBatchSize;

      // Force flush any remaining events
      await analytics.flush();

      this.log(
        'Analytics Batch Functionality',
        true,
        'Analytics batch functionality tested successfully',
        {
          batchSize: originalBatchSize,
          testBatchSize: 2
        }
      );
    } catch (error) {
      this.log(
        'Analytics Batch Functionality',
        false,
        `Analytics batch functionality test failed: ${error.message}`,
        { error: error.message }
      );
    }
  }

  /**
   * Run all tests
   */
  async runAllTests() {
    console.log('🧪 Starting Analytics Integration Tests...');
    this.startTime = Date.now();

    // Test 1: Client initialization
    this.testAnalyticsClientInit();

    // Test 2: Gateway connectivity
    await this.testGatewayConnectivity();

    // Test 3: Analytics server connectivity
    await this.testAnalyticsServerConnectivity();

    // Test 4: Event sending
    await this.testEventSending();

    // Test 5: Client methods
    await this.testAnalyticsClientMethods();

    // Test 6: Batch functionality
    await this.testBatchFunctionality();

    // Generate report
    const report = this.generateReport();
    console.table(this.results);

    return report;
  }

  /**
   * Generate test report
   */
  generateReport() {
    const totalTests = this.results.length;
    const passedTests = this.results.filter(r => r.success).length;
    const failedTests = totalTests - passedTests;
    const overallSuccess = failedTests === 0;

    const report = {
      summary: {
        total: totalTests,
        passed: passedTests,
        failed: failedTests,
        successRate: (passedTests / totalTests * 100).toFixed(1) + '%',
        overallSuccess,
        duration: Date.now() - this.startTime + 'ms'
      },
      details: this.results,
      recommendations: this.generateRecommendations()
    };

    return report;
  }

  /**
   * Generate recommendations based on test results
   */
  generateRecommendations() {
    const recommendations = [];
    const failedTests = this.results.filter(r => !r.success);

    if (failedTests.length === 0) {
      recommendations.push('✅ All analytics tests passed! The integration is working correctly.');
    } else {
      recommendations.push('❌ Some analytics tests failed. Please check the following:');

      failedTests.forEach(test => {
        switch (test.testName) {
          case 'Analytics Client Initialization':
            recommendations.push('• Check analytics client configuration in config.js');
            break;
          case 'Gateway Analytics Route Connectivity':
            recommendations.push('• Verify Gateway is running on port 8080');
            recommendations.push('• Check Gateway configuration for analytics routes');
            break;
          case 'Analytics Server Direct Connectivity':
            recommendations.push('• Verify analytics_server is running on port 6690');
            recommendations.push('• Check analytics_server health endpoint');
            break;
          case 'Analytics Event Sending':
            recommendations.push('• Check Gateway routing to analytics_server');
            recommendations.push('• Verify analytics_server API endpoints');
            break;
          case 'Analytics Client Methods':
            recommendations.push('• Check analytics client implementation');
            break;
          case 'Analytics Batch Functionality':
            recommendations.push('• Check batch processing logic');
            break;
        }
      });
    }

    return recommendations;
  }
}

// Export for use in development
export const analyticsTest = new AnalyticsIntegrationTest();

// Auto-run in development mode
if (process.env.NODE_ENV === 'development') {
  // Add to global scope for easy access in console
  window.analyticsTest = analyticsTest;
}

export default AnalyticsIntegrationTest; 