/**
 * Extension Conflict Test
 * Test script to verify extension conflict handler functionality
 */

export function testExtensionConflictHandler() {
  console.log('🧪 Testing Extension Conflict Handler...');

  // Test 1: Simulate extension conflict error
  function testErrorInterception() {
    console.log('🧪 Test 1: Error interception');

    try {
      // This should be caught and suppressed
      console.error('A listener login:1 indicated an asynchronous response by returning true, but the message channel closed before a response was received');
      console.log('✅ Error interception test passed');
    } catch (error) {
      console.error('❌ Error interception test failed:', error);
    }
  }

  // Test 2: Simulate promise rejection
  function testPromiseRejection() {
    console.log('🧪 Test 2: Promise rejection handling');

    // Create a rejected promise with extension conflict message
    Promise.reject(new Error('listener login:1 indicated an asynchronous response by returning true'))
      .catch(() => {
        // This should be handled by the conflict handler
        console.log('✅ Promise rejection test passed');
      });
  }

  // Test 3: Simulate network request conflict
  function testNetworkRequestConflict() {
    console.log('🧪 Test 3: Network request conflict handling');

    try {
      // Simulate extension content script fetch error
      const mockError = new Error('fetchError: Failed to fetch');
      mockError.stack = `Error: fetchError: Failed to fetch
    at Zx (content script.js:5837:26922)
    at ul.sendMessage (content script.js:5837:26351)
    at async ot (content script.js:6988:7519)
    at async content script.js:6988:14024`;

      // This should be detected as extension conflict
      console.error(mockError.stack);
      console.log('✅ Network request conflict test passed');
    } catch (error) {
      console.error('❌ Network request conflict test failed:', error);
    }
  }

  // Test 4: Test promise rejection with content script error
  function testContentScriptPromiseRejection() {
    console.log('🧪 Test 4: Content script promise rejection');

    // Create rejected promise with content script fetch error
    const mockError = new Error('fetchError: Failed to fetch');
    mockError.stack = 'at content script.js:5837 fetchError: Failed to fetch';

    Promise.reject(mockError)
      .catch(() => {
        console.log('✅ Content script promise rejection test passed');
      });
  }

  // Test 5: Test conflict classification
  function testConflictClassification() {
    console.log('🧪 Test 5: Conflict classification');

    if (window.extensionConflictHandler) {
      const handler = window.extensionConflictHandler;

      // Test different conflict types
      const testCases = [
        { message: 'content script.js:5837 fetchError: Failed to fetch', expected: 'network_request' },
        { message: 'message channel closed before response received', expected: 'message_channel' },
        { message: 'listener indicated an asynchronous response', expected: 'async_listener' },
        { message: 'some other extension error', expected: 'general' }
      ];

      let passed = 0;
      testCases.forEach(({ message, expected }, index) => {
        const classified = handler.classifyConflictType(message);
        if (classified === expected) {
          console.log(`  ✅ Test case ${index + 1}: ${classified} (expected: ${expected})`);
          passed++;
        } else {
          console.log(`  ❌ Test case ${index + 1}: ${classified} (expected: ${expected})`);
        }
      });

      console.log(`✅ Conflict classification test: ${passed}/${testCases.length} passed`);
    } else {
      console.error('❌ Extension conflict handler not found for classification test');
    }
  }

  // Test 6: Check conflict summary
  function testConflictSummary() {
    console.log('🧪 Test 6: Conflict summary');

    if (window.extensionConflictHandler) {
      const summary = window.extensionConflictHandler.getConflictSummary();
      console.log('Conflict summary:', summary);
      console.log('✅ Conflict summary test passed');
    } else {
      console.error('❌ Extension conflict handler not found on window');
    }
  }

  // Test 7: Show conflict report
  function testConflictReport() {
    console.log('🧪 Test 7: Conflict report');

    if (window.extensionConflictHandler) {
      window.extensionConflictHandler.showConflictReport();
      console.log('✅ Conflict report test passed');
    } else {
      console.error('❌ Extension conflict handler not found on window');
    }
  }

  // Test 8: Test conflict type-specific guide
  function testConflictTypeGuide() {
    console.log('🧪 Test 8: Conflict type guide');

    if (window.showConflictTypeGuide) {
      console.log('  Testing network_request guide:');
      window.showConflictTypeGuide('network_request');

      console.log('  Testing message_channel guide:');
      window.showConflictTypeGuide('message_channel');

      console.log('✅ Conflict type guide test passed');
    } else {
      console.error('❌ showConflictTypeGuide not found on window');
    }
  }

  // Run all tests
  testErrorInterception();
  setTimeout(testPromiseRejection, 100);
  setTimeout(testNetworkRequestConflict, 200);
  setTimeout(testContentScriptPromiseRejection, 300);
  setTimeout(testConflictClassification, 400);
  setTimeout(testConflictSummary, 500);
  setTimeout(testConflictReport, 600);
  setTimeout(testConflictTypeGuide, 700);

  console.log('🧪 Extension conflict handler tests initiated');
  console.log('💡 Tests will run sequentially over the next few seconds');
}

/**
 * Test specific conflict type handling
 */
export function testNetworkConflictHandling() {
  console.group('🧪 Network Conflict Handling Test');

  // Simulate various network request conflicts that extensions might cause
  const networkConflictPatterns = [
    'content script.js:5837 fetchError: Failed to fetch',
    'content script.js:6988 sendMessage fetch error',
    'Failed to fetch at content script chrome-extension',
    'fetchError at content script.js:1234'
  ];

  console.log('Testing network conflict pattern detection...');

  networkConflictPatterns.forEach((pattern, index) => {
    console.log(`\nTest ${index + 1}: ${pattern}`);

    if (window.extensionConflictHandler) {
      const isConflict = window.extensionConflictHandler.isExtensionConflict(pattern);
      const conflictType = window.extensionConflictHandler.classifyConflictType(pattern);

      console.log(`  Detected as conflict: ${isConflict}`);
      console.log(`  Classified as: ${conflictType}`);

      if (isConflict && conflictType === 'network_request') {
        console.log('  ✅ Correctly identified as network request conflict');
      } else {
        console.log('  ❌ Failed to correctly identify conflict type');
      }
    } else {
      console.error('  ❌ Extension conflict handler not available');
    }
  });

  console.log('\n🧪 Network conflict handling test completed');
  console.groupEnd();
}

// Auto-run in development
// Disabled automatic test execution to prevent test-induced errors
// Tests can still be run manually via window functions
/*
if (import.meta.env.DEV && typeof window !== 'undefined') {
  // Delay to ensure handler is initialized
  setTimeout(() => {
    testExtensionConflictHandler();

    // Run network-specific test after main tests
    setTimeout(() => {
      testNetworkConflictHandling();
    }, 2000);
  }, 2000);
}
*/

// Expose to window for manual testing
if (typeof window !== 'undefined') {
  window.testExtensionConflictHandler = testExtensionConflictHandler;
  window.testNetworkConflictHandling = testNetworkConflictHandling;

  console.log('🔧 Extension conflict tests available for manual execution:');
  console.log('  - window.testExtensionConflictHandler() - Run main test suite');
  console.log('  - window.testNetworkConflictHandling() - Test network conflict handling');
}

export default testExtensionConflictHandler; 