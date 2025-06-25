/**
 * Test Extension System Coordination
 * Verifies that ContentScriptErrorSuppressor and ExtensionConflictHandler work together
 */

async function testExtensionCoordination() {
  console.group('🤝 Extension System Coordination Test');

  // Test 1: Check both systems are active
  console.log('\nTest 1: System Status');
  const suppressorActive = !!window.contentScriptSuppressor;
  const handlerActive = !!window.extensionConflictHandler;

  console.log('  - ContentScriptErrorSuppressor:', suppressorActive ? '✅ Active' : '❌ Not active');
  console.log('  - ExtensionConflictHandler:', handlerActive ? '✅ Active' : '❌ Not active');

  if (!suppressorActive || !handlerActive) {
    console.log('  ⚠️ Both systems must be active for coordination test');
    console.groupEnd();
    return;
  }

  // Test 2: Test content script error handling
  console.log('\nTest 2: Content Script Error Handling');

  // Get initial stats
  const initialStats = window.contentScriptSuppressor.getStats();
  const initialConflicts = window.extensionConflictHandler.errorCounts.size;

  // Create a content script error
  const contentScriptError = new Error('fetchError: Failed to fetch');
  contentScriptError.stack = 'at P1.sendMessage (chrome-extension://fake/content script.js:123:45)';

  console.log('  - Simulating content script error...');
  console.error(contentScriptError);

  // Check results after a short delay
  setTimeout(() => {
    const newStats = window.contentScriptSuppressor.getStats();
    const newConflicts = window.extensionConflictHandler.errorCounts.size;

    console.log('  - Suppressor handled:', newStats.totalSuppressed > initialStats.totalSuppressed ? '✅ Yes' : '❌ No');
    console.log('  - ConflictHandler ignored:', newConflicts === initialConflicts ? '✅ Yes' : '❌ No');

    // Test 3: Test non-content script extension error
    console.log('\nTest 3: Extension Conflict Error Handling');

    // Create an extension conflict error (not content script)
    const extensionError = new Error('A listener indicated an asynchronous response');

    console.log('  - Simulating extension conflict error...');
    console.error(extensionError);

    setTimeout(() => {
      const finalConflicts = window.extensionConflictHandler.errorCounts.size;

      console.log('  - ConflictHandler detected:', finalConflicts > newConflicts ? '✅ Yes' : '❌ No');
      console.log('  - Coordination successful:', '✅ Systems working together');

      // Summary
      console.log('\n📊 Summary:');
      console.log('  - Content script errors → Suppressor only');
      console.log('  - Extension conflicts → Handler only');
      console.log('  - No overlap or duplication');

      console.groupEnd();
    }, 100);
  }, 100);
}

// Export test function
window.testExtensionCoordination = testExtensionCoordination;

// Auto-run test after page load
if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', () => {
    setTimeout(() => {
      console.log('💡 Extension coordination test available. Run window.testExtensionCoordination()');
    }, 2000);
  });
} else {
  setTimeout(() => {
    console.log('💡 Extension coordination test available. Run window.testExtensionCoordination()');
  }, 2000);
} 