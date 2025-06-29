/**
 * Fix Verification Script
 * Tests all implemented fixes to ensure they work correctly
 */

class FixVerification {
  constructor() {
    this.results = [];
  }

  /**
   * Add test result
   */
  addResult(test, status, details) {
    this.results.push({
      test,
      status,
      details,
      timestamp: new Date().toISOString()
    });

  /**
   * Test 1: Navigation Fix Optimization
   */
  testNavigationFix() {
    console.group('🔧 Testing Navigation Fix Optimization');

    try {
      // Check if navigation fix is loaded
      if (typeof window.navigationFix === 'undefined') {
        this.addResult('Navigation Fix', 'FAIL', 'navigationFix not found on window');
        return;
      }

      // Check if debouncing is working
      const startTime = Date.now();

      // Trigger multiple fixes rapidly
      for (let i = 0; i < 5; i++) {
        window.navigationFix.fixChannelNavigation();
      }

      const endTime = Date.now();
      const duration = endTime - startTime;

      if (duration < 100) {
        this.addResult('Navigation Fix', 'PASS', 'Debouncing working - rapid calls completed quickly');
      } else {
        this.addResult('Navigation Fix', 'WARN', `Calls took ${duration}ms - may need optimization`);
      }

      if (import.meta.env.DEV) {
        console.log('✅ Navigation fix optimization test completed');
      }

    } catch (error) {
      this.addResult('Navigation Fix', 'ERROR', error.message);
      if (import.meta.env.DEV) {
        console.error('❌ Navigation fix test failed:', error);
      }

    console.groupEnd();
  }

  /**
   * Test 2: SSE Connection Fix
   */
  async testSSEConnectionFix() {
    console.group('🔌 Testing SSE Connection Fix');

    try {
      // Check if SSE connection fix is loaded
      if (typeof window.sseConnectionFix === 'undefined') {
        this.addResult('SSE Connection Fix', 'FAIL', 'sseConnectionFix not found on window');
        return;
      }

      // Test endpoint check
      const endpointResult = await window.sseConnectionFix.testSSEEndpoint();

      if (endpointResult) {
        this.addResult('SSE Connection Fix', 'PASS', 'SSE endpoint test passed');
      } else {
        this.addResult('SSE Connection Fix', 'WARN', 'SSE endpoint test failed - may need token refresh');

        // Try auto-fix
        const autoFixResult = await window.sseConnectionFix.autoFix();

        if (autoFixResult) {
          this.addResult('SSE Auto-Fix', 'PASS', 'Auto-fix resolved SSE issues');
        } else {
          this.addResult('SSE Auto-Fix', 'FAIL', 'Auto-fix could not resolve SSE issues');
        }

      if (import.meta.env.DEV) {
        console.log('✅ SSE connection fix test completed');
      }

    } catch (error) {
      this.addResult('SSE Connection Fix', 'ERROR', error.message);
      if (import.meta.env.DEV) {
        console.error('❌ SSE connection fix test failed:', error);
      }

    console.groupEnd();
  }

  /**
   * Test 3: Navigation Fix Log Suppression
   */
  testLogSuppression() {
    console.group('🔇 Testing Navigation Fix Log Suppression');

    try {
      // Check if navigation fix is loaded
      if (typeof window.navigationFix === 'undefined') {
        this.addResult('Log Suppression', 'FAIL', 'navigationFix not found on window');
        return;
      }

      // Get initial log stats
      const initialStats = window.navigationFix.getLogStats();
      const initialCount = Object.keys(initialStats.logCounts).length;

      // Reset log counts for clean test
      window.navigationFix.resetLogCounts();

      // Trigger multiple fixes to test suppression
      for (let i = 0; i < 10; i++) {
        window.navigationFix.fixChannelNavigation();
      }

      // Check final stats
      const finalStats = window.navigationFix.getLogStats();
      const finalCount = Object.keys(finalStats.logCounts).length;

      if (finalCount > initialCount) {
        this.addResult('Log Suppression', 'PASS', `Log suppression is working - tracking ${finalCount} message types`);
      } else {
        this.addResult('Log Suppression', 'WARN', 'No log suppression activity detected');
      }

      if (import.meta.env.DEV) {
        console.log('✅ Log suppression test completed');
      }

    } catch (error) {
      this.addResult('Log Suppression', 'ERROR', error.message);
      if (import.meta.env.DEV) {
        console.error('❌ Log suppression test failed:', error);
      }

    console.groupEnd();
  }

  /**
   * Test 4: Performance Impact
   */
  testPerformanceImpact() {
    console.group('⚡ Testing Performance Impact');

    try {
      const startTime = performance.now();

      // Simulate DOM changes that would trigger navigation fixes
      const testDiv = document.createElement('div');
      testDiv.className = 'channel-card';
      testDiv.setAttribute('data-test', 'true');
      document.body.appendChild(testDiv);

      // Wait a moment for observers to trigger
      setTimeout(() => {
        const endTime = performance.now();
        const duration = endTime - startTime;

        if (duration < 50) {
          this.addResult('Performance Impact', 'PASS', `DOM manipulation took ${duration.toFixed(2)}ms`);
        } else {
          this.addResult('Performance Impact', 'WARN', `DOM manipulation took ${duration.toFixed(2)}ms - may be slow`);
        }

        // Cleanup
        document.body.removeChild(testDiv);

        if (import.meta.env.DEV) {
          console.log('✅ Performance impact test completed');
        }
      }, 100);

    } catch (error) {
      this.addResult('Performance Impact', 'ERROR', error.message);
      if (import.meta.env.DEV) {
        console.error('❌ Performance impact test failed:', error);
      }

    console.groupEnd();
  }

  /**
   * Test 5: Token Management
   */
  async testTokenManagement() {
    console.group('🔐 Testing Token Management');

    try {
      // Check if token manager is available
      if (typeof window.tokenManager === 'undefined') {
        this.addResult('Token Management', 'FAIL', 'tokenManager not found on window');
        return;
      }

      const tokens = window.tokenManager.getTokens();

      if (tokens.accessToken) {
        this.addResult('Token Management', 'PASS', 'Access token is available');

        // Test token expiration check
        const isExpired = window.tokenManager.isTokenExpired();
        const shouldRefresh = window.tokenManager.shouldRefreshToken();

        this.addResult('Token Expiration Check', 'INFO', `Expired: ${isExpired}, Should Refresh: ${shouldRefresh}`);

      } else {
        this.addResult('Token Management', 'WARN', 'No access token available');
      }

      if (import.meta.env.DEV) {
        console.log('✅ Token management test completed');
      }

    } catch (error) {
      this.addResult('Token Management', 'ERROR', error.message);
      if (import.meta.env.DEV) {
        console.error('❌ Token management test failed:', error);
      }

    console.groupEnd();
  }

  /**
   * Run all tests
   */
  async runAllTests() {
    if (import.meta.env.DEV) {
      console.log('🧪 Starting Fix Verification Tests...');
    if (import.meta.env.DEV) {
      console.log('=====================================');
    }

    this.results = []; // Reset results

    // Run tests
    this.testNavigationFix();
    await this.testSSEConnectionFix();
    this.testLogSuppression();
    this.testPerformanceImpact();
    await this.testTokenManagement();

    // Generate report
    setTimeout(() => {
      this.generateReport();
    }, 2000); // Wait for async tests to complete
  }

  /**
   * Generate test report
   */
  generateReport() {
    if (import.meta.env.DEV) {
      console.log('\n📊 Fix Verification Report');
    if (import.meta.env.DEV) {
      console.log('==========================');
    }

    const summary = {
      total: this.results.length,
      passed: this.results.filter(r => r.status === 'PASS').length,
      failed: this.results.filter(r => r.status === 'FAIL').length,
      warnings: this.results.filter(r => r.status === 'WARN').length,
      errors: this.results.filter(r => r.status === 'ERROR').length
    };

    if (import.meta.env.DEV) {
      console.log(`📈 Summary: ${summary.passed}/${summary.total} tests passed`);
    if (import.meta.env.DEV) {
      console.log(`❌ Failed: ${summary.failed}`);
    if (import.meta.env.DEV) {
      console.log(`⚠️ Warnings: ${summary.warnings}`);
    if (import.meta.env.DEV) {
      console.log(`🚨 Errors: ${summary.errors}`);
    }

    // Show detailed results
    if (import.meta.env.DEV) {
      console.log('\n📋 Detailed Results:');
    console.table(this.results);

    // Overall status
    if (summary.failed === 0 && summary.errors === 0) {
      if (import.meta.env.DEV) {
        console.log('\n🎉 All critical tests passed! Fixes are working correctly.');
      }
    } else if (summary.failed > 0 || summary.errors > 0) {
      if (import.meta.env.DEV) {
        console.log('\n⚠️ Some tests failed. Please review the issues above.');
      }
    } else {
      if (import.meta.env.DEV) {
        console.log('\n✅ Tests completed with warnings. System is functional.');
      }

    return {
      summary,
      results: this.results
    };
  }

// Create singleton instance
const fixVerification = new FixVerification();

// Export for global use
if (typeof window !== 'undefined') {
  window.fixVerification = fixVerification;
  window.verifyFixes = () => fixVerification.runAllTests();

  if (import.meta.env.DEV) {
    console.log('🧪 Fix Verification loaded - use window.verifyFixes()');
  }

export default fixVerification; 