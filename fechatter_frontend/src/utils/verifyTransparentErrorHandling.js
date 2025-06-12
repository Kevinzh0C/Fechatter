/**
 * Verify Transparent Error Handling
 * Ensures error sources are preserved through our suppressor
 * Production-grade verification following Occam's Razor
 */

function verifyTransparentErrorHandling() {
  console.group('🔬 Transparent Error Handling Verification');

  // Get current suppressor stats
  const stats = window.contentScriptSuppressor?.getStats() || {};
  console.log('📊 Initial suppressor stats:', stats);

  // Test 1: Simple error from this file
  console.log('\n1️⃣ Test: Simple console.error');
  console.error('Direct error from verifyTransparentErrorHandling.js');
  console.log('   ↳ Check: Should show THIS file as source, not suppressor');

  // Test 2: Error object with stack
  console.log('\n2️⃣ Test: Error object with stack');
  const testError = new Error('Test error with stack trace');
  console.error(testError);
  console.log('   ↳ Check: Stack should start from THIS file');

  // Test 3: Complex API error simulation
  console.log('\n3️⃣ Test: API error simulation');
  const apiError = {
    message: 'Request failed with status code 401',
    code: 'ERR_BAD_REQUEST',
    response: { status: 401, data: {} },
    config: { url: '/api/test' }
  };
  console.error('🚨 API Error (via Gateway):', apiError);
  console.log('   ↳ Check: Should show THIS line as source');

  // Test 4: Nested function calls
  console.log('\n4️⃣ Test: Nested function calls');
  function level3() {
    console.error('Error from level3 function');
  }
  function level2() { level3(); }
  function level1() { level2(); }
  level1();
  console.log('   ↳ Check: Stack should show level3 → level2 → level1');

  // Test 5: Content script error (should be suppressed)
  console.log('\n5️⃣ Test: Content script suppression');
  const beforeSuppress = window.contentScriptSuppressor?.getStats().totalSuppressed || 0;

  const contentError = new Error('fetchError: Failed to fetch');
  contentError.stack = 'Error: fetchError: Failed to fetch\n    at Zx (content script.js:123:45)';
  console.error(contentError);

  const afterSuppress = window.contentScriptSuppressor?.getStats().totalSuppressed || 0;
  console.log('   ↳ Suppressed:', afterSuppress > beforeSuppress ? '✅ Yes' : '❌ No');

  // Test 6: Multiple rapid errors
  console.log('\n6️⃣ Test: Rapid error sequence');
  console.error('Error 1 - rapid');
  console.error('Error 2 - rapid');
  console.error('Error 3 - rapid');
  console.log('   ↳ Check: Each should show correct line number');

  // Summary
  console.log('\n📋 Summary:');
  console.log('  • Regular errors should show real source location');
  console.log('  • No contentScriptErrorSuppressor.js in stack traces');
  console.log('  • Content script errors should be suppressed');
  console.log('  • Error handling should be completely transparent');

  // Final stats
  const finalStats = window.contentScriptSuppressor?.getStats() || {};
  console.log('\n📊 Final suppressor stats:', finalStats);

  console.groupEnd();
}

// Export and auto-run
if (typeof window !== 'undefined') {
  window.verifyTransparentErrorHandling = verifyTransparentErrorHandling;
  console.log('🔬 Verification test available: window.verifyTransparentErrorHandling()');
}

// Auto-run after delay in development
if (import.meta.env.DEV) {
  setTimeout(() => {
    verifyTransparentErrorHandling();
  }, 3000);
}

export default verifyTransparentErrorHandling; 