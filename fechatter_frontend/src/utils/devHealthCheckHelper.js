/**
 * Development Health Check Helper
 * 
 * Provides utilities for debugging and monitoring health check issues
 */

class DevHealthCheckHelper {
  constructor() {
    this.checkHistory = [];
    this.maxHistorySize = 50;
    this.isMonitoring = false;
    this.monitorInterval = null;
  }

  /**
   * Run a single health check and display results
   */
  async runSingleCheck(checkId) {
    try {
      const healthCheck = await import('./healthCheck.js');
      const result = await healthCheck.default.runCheck(checkId);

      this.logCheckResult(checkId, result);
      return result;
    } catch (error) {
      console.error(`❌ Failed to run check ${checkId}:`, error);
      return null;
    }
  }

  /**
   * Run all health checks and display summary
   */
  async runAllChecks() {
    try {
      const healthCheck = await import('./healthCheck.js');
      const results = await healthCheck.default.runAllChecksSafely();

      console.group('🏥 Health Check Results');
      console.log('Summary:', results.summary);

      // Group results by status
      const passed = results.results.filter(r => r.success);
      const failed = results.results.filter(r => !r.success);

      if (passed.length > 0) {
        console.group('✅ Passed Checks');
        passed.forEach(r => {
          console.log(`${r.checkName}:`, r.details);
        });
        console.groupEnd();
      }

      if (failed.length > 0) {
        console.group('❌ Failed Checks');
        failed.forEach(r => {
          console.error(`${r.checkName}:`, r.error || r.details);
        });
        console.groupEnd();
      }

      console.groupEnd();

      return results;
    } catch (error) {
      console.error('❌ Failed to run health checks:', error);
      return null;
    }
  }

  /**
   * Monitor specific checks continuously
   */
  startMonitoring(checkIds = ['stores_functional', 'sse_connection'], intervalMs = 5000) {
    if (this.isMonitoring) {
      console.warn('⚠️ Monitoring already active');
      return;
    }

    this.isMonitoring = true;
    console.log(`🔍 Starting health check monitoring for: ${checkIds.join(', ')}`);
    console.log(`📊 Interval: ${intervalMs}ms`);

    this.monitorInterval = setInterval(async () => {
      console.group(`🔄 Health Check Monitor - ${new Date().toLocaleTimeString()}`);

      for (const checkId of checkIds) {
        const result = await this.runSingleCheck(checkId);
        if (result) {
          const status = result.success ? '✅' : '❌';
          console.log(`${status} ${checkId}:`, result.details || result.error);
        }
      }

      console.groupEnd();
    }, intervalMs);
  }

  /**
   * Stop monitoring
   */
  stopMonitoring() {
    if (this.monitorInterval) {
      clearInterval(this.monitorInterval);
      this.monitorInterval = null;
      this.isMonitoring = false;
      console.log('🛑 Health check monitoring stopped');
    }
  }

  /**
   * Log check result to history
   */
  logCheckResult(checkId, result) {
    const entry = {
      checkId,
      timestamp: new Date().toISOString(),
      success: result.success,
      details: result.details,
      error: result.error
    };

    this.checkHistory.unshift(entry);
    if (this.checkHistory.length > this.maxHistorySize) {
      this.checkHistory = this.checkHistory.slice(0, this.maxHistorySize);
    }
  }

  /**
   * Get check history
   */
  getHistory(checkId = null) {
    if (checkId) {
      return this.checkHistory.filter(h => h.checkId === checkId);
    }
    return this.checkHistory;
  }

  /**
   * Analyze common issues
   */
  analyzeIssues() {
    console.group('🔍 Health Check Issue Analysis');

    // Check app initialization
    console.log('App Status:', {
      hasApp: !!window.app,
      hasPinia: !!window.pinia,
      hasRouter: !!window.$router,
      hasSSE: !!window.realtimeCommunicationService
    });

    // Check authentication
    if (window.pinia) {
      try {
        const authStore = window.pinia._s.get('auth');
        console.log('Auth Status:', {
          hasToken: !!authStore?.token,
          hasUser: !!authStore?.user,
          isAuthenticated: authStore?.isAuthenticated
        });
      } catch (e) {
        console.warn('Could not access auth store:', e.message);
      }
    }

    // Check SSE status
    if (window.realtimeCommunicationService) {
      const sseState = window.realtimeCommunicationService.getConnectionState();
      console.log('SSE Status:', sseState);
    }

    console.groupEnd();
  }

  /**
   * Fix common issues
   */
  async attemptFixes() {
    console.group('🔧 Attempting Common Fixes');

    // Fix 1: Ensure config is loaded
    try {
      const { initializeConfig } = await import('./configLoader.js');
      await initializeConfig();
      console.log('✅ Configuration initialized');
    } catch (e) {
      console.error('❌ Failed to initialize config:', e.message);
    }

    // Fix 2: Try to connect SSE if authenticated
    if (window.realtimeCommunicationService && window.pinia) {
      try {
        const authStore = window.pinia._s.get('auth');
        if (authStore?.token && !window.realtimeCommunicationService.isConnected) {
          console.log('🔌 Attempting SSE connection...');
          await window.realtimeCommunicationService.connect(authStore.token);
          console.log('✅ SSE connection attempted');
        }
      } catch (e) {
        console.error('❌ Failed to connect SSE:', e.message);
      }
    }

    console.groupEnd();
  }
}

// Create singleton instance
const devHealthCheckHelper = new DevHealthCheckHelper();

// Expose to window for easy access in development
if (import.meta.env.DEV && typeof window !== 'undefined') {
  window.healthHelper = {
    run: (checkId) => devHealthCheckHelper.runSingleCheck(checkId),
    runAll: () => devHealthCheckHelper.runAllChecks(),
    monitor: (checkIds, interval) => devHealthCheckHelper.startMonitoring(checkIds, interval),
    stopMonitor: () => devHealthCheckHelper.stopMonitoring(),
    history: (checkId) => devHealthCheckHelper.getHistory(checkId),
    analyze: () => devHealthCheckHelper.analyzeIssues(),
    fix: () => devHealthCheckHelper.attemptFixes(),

    // Shortcuts
    stores: () => devHealthCheckHelper.runSingleCheck('stores_functional'),
    sse: () => devHealthCheckHelper.runSingleCheck('sse_connection'),
    api: () => devHealthCheckHelper.runSingleCheck('api_connection')
  };

  console.log('💡 Health Check Helper available at window.healthHelper');
  console.log('   Commands: run(), runAll(), monitor(), analyze(), fix()');
  console.log('   Shortcuts: stores(), sse(), api()');
}

export default devHealthCheckHelper; 