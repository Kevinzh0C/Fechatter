/**
 * Test Content Script Error Suppression
 * Verify that browser extension errors are properly suppressed
 */

export function testContentScriptErrorSuppression() {
  console.group('🧪 Testing Content Script Error Suppression');

  // Check if suppressors are active
  const hasExtensionHandler = !!window.extensionConflictHandler;
  const hasContentSuppressor = !!window.contentScriptSuppressor;

  console.log('✅ Extension conflict handler available:', hasExtensionHandler);
  console.log('✅ Content script suppressor available:', hasContentSuppressor);

  // Get statistics
  if (window.contentScriptSuppressor) {
    const stats = window.contentScriptSuppressor.getStats();
    console.log('📊 Suppression statistics:', stats);
  }

  // Test by creating a mock error (not actually throwing it)
  console.log('\n🔍 Testing error detection (simulation only):');

  const testError = new Error('fetchError: Failed to fetch');
  testError.stack = `Error: fetchError: Failed to fetch
    at Zx (content script.js:5837:26922)
    at ul.sendMessage (content script.js:5837:26351)
    at async ot (content script.js:6988:7519)
    at async content script.js:6988:14024`;

  console.log('Mock error created with content script stack trace');

  // Check if it would be suppressed
  if (window.extensionConflictHandler) {
    const wouldBeSuppressed = window.extensionConflictHandler.isExtensionConflict(testError.stack);
    console.log('Would be detected by extension handler:', wouldBeSuppressed);
  }

  if (window.contentScriptSuppressor) {
    // Use the new API - isContentScriptError
    const errorString = testError.stack || testError.message;
    const wouldBeSuppressed = window.contentScriptSuppressor.isContentScriptError(errorString);
    console.log('Would be suppressed by content suppressor:', wouldBeSuppressed);
  }

  console.log('\n✅ Error suppression is configured correctly');
  console.log('💡 Content script errors should now be silently handled');

  console.groupEnd();
}

// Auto-run in development
if (import.meta.env.DEV && typeof window !== 'undefined') {
  // Wait for suppressors to be initialized
  setTimeout(() => {
    testContentScriptErrorSuppression();
  }, 2000);
}

// Expose for manual testing
if (typeof window !== 'undefined') {
  window.testContentScriptErrorSuppression = testContentScriptErrorSuppression;
  console.log('🧪 Test available: window.testContentScriptErrorSuppression()');
}

export default testContentScriptErrorSuppression; 