/**
 * Test Manager
 * Centralized control for all development tests
 * Following Occam's Razor: Tests should be run manually, not automatically
 */

export class TestManager {
  constructor() {
    this.availableTests = new Map();
    this.testResults = new Map();
    this.isInitialized = false;
  }

  /**
   * Initialize test manager
   */
  initialize() {
    if (this.isInitialized) return;

    console.group('🧪 Test Manager Initialization');
    console.log('Registering available tests...');

    // Register all available tests
    this.registerTest('errorHandler', {
      name: 'Error Handler Tests',
      description: 'Test errorHandler.showNotification and related functionality',
      commands: [
        { name: 'testCurrentError', desc: 'Test specific error fix' },
        { name: 'runErrorHandlerTest', desc: 'Run full test suite' }
      ]
    });

    this.registerTest('extensionConflict', {
      name: 'Extension Conflict Tests',
      description: 'Test browser extension conflict detection and handling',
      commands: [
        { name: 'testExtensionConflictHandler', desc: 'Run main test suite' },
        { name: 'testNetworkConflictHandling', desc: 'Test network conflicts' }
      ]
    });

    this.registerTest('networkConflict', {
      name: 'Network Conflict Tests',
      description: 'Test network request conflict handling',
      commands: [
        { name: 'testCurrentError', desc: 'Test specific error pattern' },
        { name: 'runNetworkConflictTest', desc: 'Run full test suite' }
      ]
    });

    this.registerTest('security', {
      name: 'Security Tests',
      description: 'Test JWT token security and protection',
      commands: [
        { name: 'testSecurity', desc: 'Run security test suite' }
      ]
    });

    console.log(`✅ Registered ${this.availableTests.size} test suites`);
    console.groupEnd();

    this.isInitialized = true;
  }

  /**
   * Register a test suite
   */
  registerTest(id, config) {
    this.availableTests.set(id, {
      id,
      ...config,
      lastRun: null,
      lastResult: null
    });
  }

  /**
   * Run a specific test
   */
  async runTest(testId, commandName) {
    const test = this.availableTests.get(testId);
    if (!test) {
      console.error(`❌ Test suite '${testId}' not found`);
      return null;
    }

    const command = test.commands.find(cmd => cmd.name === commandName);
    if (!command) {
      console.error(`❌ Command '${commandName}' not found in test suite '${testId}'`);
      return null;
    }

    console.log(`🧪 Running ${test.name} - ${command.desc}...`);

    try {
      if (window[commandName]) {
        const result = await window[commandName]();
        test.lastRun = new Date();
        test.lastResult = { success: true, result };
        return result;
      } else {
        throw new Error(`Function ${commandName} not available`);
      }
    } catch (error) {
      console.error(`❌ Test failed:`, error);
      test.lastResult = { success: false, error: error.message };
      return null;
    }
  }

  /**
   * Show available tests
   */
  showAvailableTests() {
    console.group('🧪 Available Test Suites');

    this.availableTests.forEach((test) => {
      console.group(`📦 ${test.name}`);
      console.log(`Description: ${test.description}`);
      console.log('Commands:');
      test.commands.forEach(cmd => {
        console.log(`  • window.${cmd.name}() - ${cmd.desc}`);
      });
      if (test.lastRun) {
        console.log(`Last run: ${test.lastRun.toLocaleString()}`);
        console.log(`Last result:`, test.lastResult);
      }
      console.groupEnd();
    });

    console.log('\n💡 Tip: Use window.tests.run(testId, commandName) for structured execution');
    console.log('Example: window.tests.run("errorHandler", "testCurrentError")');
    console.groupEnd();
  }

  /**
   * Run all tests (with confirmation)
   */
  async runAll() {
    console.log('⚠️ Running all tests can produce console noise and side effects.');
    console.log('💡 It is recommended to run tests individually as needed.');
    console.log('Type window.tests.runAllConfirmed() to proceed.');
  }

  /**
   * Actually run all tests
   */
  async runAllConfirmed() {
    console.group('🧪 Running All Test Suites');

    for (const [testId, test] of this.availableTests) {
      console.group(`📦 ${test.name}`);
      for (const command of test.commands) {
        await this.runTest(testId, command.name);
        // Add delay between tests
        await new Promise(resolve => setTimeout(resolve, 500));
      }
      console.groupEnd();
    }

    console.groupEnd();
  }

  /**
   * Clear test results
   */
  clearResults() {
    this.testResults.clear();
    this.availableTests.forEach(test => {
      test.lastRun = null;
      test.lastResult = null;
    });
    console.log('✅ Test results cleared');
  }
}

// Create singleton instance
const testManager = new TestManager();

// Initialize in development
if (import.meta.env.DEV) {
  // Wait for all test modules to load
  setTimeout(() => {
    testManager.initialize();

    // Create convenient API
    window.tests = {
      show: () => testManager.showAvailableTests(),
      run: (testId, commandName) => testManager.runTest(testId, commandName),
      runAll: () => testManager.runAll(),
      runAllConfirmed: () => testManager.runAllConfirmed(),
      clear: () => testManager.clearResults(),
      manager: testManager
    };

    console.log('🧪 Test Manager ready - use window.tests.show() to see available tests');
  }, 4000); // Wait for test modules to register their functions
}

export default testManager; 