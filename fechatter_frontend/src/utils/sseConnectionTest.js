/**
 * SSE Connection Test Utility
 * Test and diagnose SSE connection issues after fixes
 */

export class SSEConnectionTester {
  constructor() {
    this.testResults = [];
    this.isRunning = false;
  }

  /**
   * Run comprehensive SSE connection tests
   */
  async runTests() {
    if (this.isRunning) {
      console.warn('🧪 SSE tests already running');
      return;
    }

    this.isRunning = true;
    this.testResults = [];

    console.log('🧪 Starting SSE Connection Tests...');

    try {
      // Test 1: Global Manager State
      await this.testGlobalManagerState();

      // Test 2: Service Singleton State
      await this.testServiceState();

      // Test 3: Connection Attempt
      await this.testConnectionAttempt();

      // Test 4: State Consistency
      await this.testStateConsistency();

    } catch (error) {
      this.addResult('FATAL', 'Test execution failed', error.message);
    } finally {
      this.isRunning = false;
      this.printReport();
    }
  }

  /**
   * Test Global Manager State
   */
  async testGlobalManagerState() {
    try {
      if (!window.sseGlobalManager) {
        this.addResult('ERROR', 'Global Manager', 'Not found in window object');
        return;
      }

      const status = window.sseGlobalManager.getStatus();
      this.addResult('INFO', 'Global Manager', `Found with ${status.connections.length} connections`);

      if (status.sessionStats.permanentlyBanned.length > 0) {
        this.addResult('WARNING', 'Banned URLs', status.sessionStats.permanentlyBanned.join(', '));
      }

      if (Object.keys(status.sessionStats.bannedUrls).length > 0) {
        this.addResult('WARNING', 'Cooling Down URLs', Object.keys(status.sessionStats.bannedUrls).join(', '));
      }

    } catch (error) {
      this.addResult('ERROR', 'Global Manager Test', error.message);
    }
  }

  /**
   * Test Service State
   */
  async testServiceState() {
    try {
      if (!window.realtimeCommunicationService) {
        this.addResult('ERROR', 'Service', 'Not found in window object');
        return;
      }

      const service = window.realtimeCommunicationService;
      const state = service.getConnectionState();

      this.addResult('INFO', 'Service State', `${state.state} - Connected: ${state.isConnected}`);
      this.addResult('INFO', 'Retry Control', `Total: ${state.retryControl.totalAttempts}/${state.retryControl.maxTotalAttempts}`);

      if (state.retryControl.permanentFailure) {
        this.addResult('ERROR', 'Service', 'Permanently failed');
      }

    } catch (error) {
      this.addResult('ERROR', 'Service Test', error.message);
    }
  }

  /**
   * Test Connection Attempt
   */
  async testConnectionAttempt() {
    try {
      // Get token for connection
      const { default: tokenManager } = await import('@/services/tokenManager');
      const tokens = tokenManager.getTokens();

      if (!tokens.accessToken) {
        this.addResult('WARNING', 'Connection Test', 'No access token available');
        return;
      }

      const service = window.realtimeCommunicationService;

      // Check current state before attempting connection
      const beforeState = service.getConnectionState();
      this.addResult('INFO', 'Before Connection', `State: ${beforeState.state}`);

      // Attempt connection (this will test our fixes)
      await service.connect(tokens.accessToken);

      // Wait a moment for connection to establish
      await new Promise(resolve => setTimeout(resolve, 2000));

      const afterState = service.getConnectionState();
      this.addResult('INFO', 'After Connection', `State: ${afterState.state}, Connected: ${afterState.isConnected}`);

    } catch (error) {
      this.addResult('ERROR', 'Connection Test', error.message);
    }
  }

  /**
   * Test State Consistency
   */
  async testStateConsistency() {
    try {
      const service = window.realtimeCommunicationService;
      const globalManager = window.sseGlobalManager;

      if (!service || !globalManager) {
        this.addResult('ERROR', 'Consistency Test', 'Missing service or global manager');
        return;
      }

      const serviceState = service.getConnectionState();
      const globalStatus = globalManager.getStatus();

      // Check if service reports connected but no active connections in global manager
      if (serviceState.isConnected && globalStatus.connections.length === 0) {
        this.addResult('WARNING', 'State Inconsistency', 'Service connected but no global connections');
      }

      // Check if global manager has connections but service not connected
      if (!serviceState.isConnected && globalStatus.connections.length > 0) {
        this.addResult('WARNING', 'State Inconsistency', 'Global connections exist but service not connected');
      }

      if (serviceState.isConnected && globalStatus.connections.length > 0) {
        this.addResult('SUCCESS', 'State Consistency', 'Service and global manager states align');
      }

    } catch (error) {
      this.addResult('ERROR', 'Consistency Test', error.message);
    }
  }

  /**
   * Add test result
   */
  addResult(level, category, message) {
    const result = {
      timestamp: new Date().toISOString(),
      level,
      category,
      message
    };
    this.testResults.push(result);

    const emoji = {
      'SUCCESS': '✅',
      'INFO': 'ℹ️',
      'WARNING': '⚠️',
      'ERROR': '❌',
      'FATAL': '💥'
    }[level] || '🔸';

    console.log(`${emoji} [${category}] ${message}`);
  }

  /**
   * Print test report
   */
  printReport() {
    console.log('\n🧪 =============== SSE Test Report ===============');

    const summary = {
      SUCCESS: 0,
      INFO: 0,
      WARNING: 0,
      ERROR: 0,
      FATAL: 0
    };

    this.testResults.forEach(result => {
      summary[result.level]++;
    });

    console.log(`✅ Success: ${summary.SUCCESS}`);
    console.log(`ℹ️ Info: ${summary.INFO}`);
    console.log(`⚠️ Warnings: ${summary.WARNING}`);
    console.log(`❌ Errors: ${summary.ERROR}`);
    console.log(`💥 Fatal: ${summary.FATAL}`);

    if (summary.ERROR === 0 && summary.FATAL === 0) {
      console.log('🎉 All tests passed! SSE connection should be working properly.');
    } else {
      console.log('🔧 Issues detected. Review the errors above.');
    }

    console.log('===============================================\n');

    return {
      summary,
      results: this.testResults,
      passed: summary.ERROR === 0 && summary.FATAL === 0
    };
  }

  /**
   * Quick test - just check current state
   */
  quickTest() {
    console.log('🔍 Quick SSE State Check:');

    if (window.sseGlobalManager) {
      const status = window.sseGlobalManager.getStatus();
      console.log(`📊 Global Manager: ${status.connections.length} connections`);
    }

    if (window.realtimeCommunicationService) {
      const state = window.realtimeCommunicationService.getConnectionState();
      console.log(`🔌 Service: ${state.state} (Connected: ${state.isConnected})`);
    }
  }
}

// Create global instance for easy access
const sseTester = new SSEConnectionTester();

// Expose to window for debugging
if (typeof window !== 'undefined') {
  window.sseTester = sseTester;
}

export default sseTester; 