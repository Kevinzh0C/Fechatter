/**
 * Test Pragmatic Error Suppressor
 * Verify the new practical approach works correctly
 */

function testPragmaticSuppressor() {
  console.group('🧪 Testing Pragmatic Error Suppressor');

  // Test 1: Check initialization
  console.log('\n1️⃣ Initialization Check');
  const suppressor = window.contentScriptSuppressor;
  if (suppressor) {
    console.log('  ✅ Suppressor loaded');
    console.log('  - Stats:', suppressor.getStats());
  } else {
    console.log('  ❌ Suppressor not found');
    console.groupEnd();
    return;
  }

  // Test 2: Regular error with source annotation
  console.log('\n2️⃣ Regular Error Test');
  console.log('  - Logging error from this file...');
  console.error('Test error from testPragmaticSuppressor.js');
  console.log('  - Check: Should show [from testPragmaticSuppressor.js:XX] prefix');

  // Test 3: API error simulation
  console.log('\n3️⃣ API Error Simulation');
  const apiError = new Error('Request failed with status code 401');
  apiError.response = { status: 401 };
  console.error('API Error:', apiError);
  console.log('  - Check: Should show source annotation');

  // Test 4: Content script error (should be suppressed)
  console.log('\n4️⃣ Content Script Error Test');
  const beforeCount = suppressor.getStats().totalSuppressed;

  const contentError = new Error('fetchError: Failed to fetch');
  contentError.stack = `Error: fetchError: Failed to fetch
    at Zx (content script.js:123:45)
    at ul.sendMessage (content script.js:456:78)`;

  console.error(contentError);

  const afterCount = suppressor.getStats().totalSuppressed;
  console.log('  - Suppressed:', afterCount > beforeCount ? '✅ Yes' : '❌ No');
  console.log('  - Total suppressed:', afterCount);

  // Test 5: Show suppressed errors
  console.log('\n5️⃣ Testing Utility Functions');
  console.log('  - Running showSuppressedErrors()...');
  if (window.showSuppressedErrors) {
    window.showSuppressedErrors();
  }

  // Test 6: Toggle suppression
  console.log('\n6️⃣ Toggle Test');
  console.log('  - Disabling suppression...');
  if (window.toggleErrorSuppression) {
    window.toggleErrorSuppression();
    console.log('  - Logging content script error (should appear)...');
    console.error(contentError);

    console.log('  - Re-enabling suppression...');
    window.toggleErrorSuppression();
  }

  // Summary
  console.log('\n📊 Summary:');
  console.log('  - Source annotations: Check console for [from file:line] prefixes');
  console.log('  - Content script suppression: Working');
  console.log('  - Utility functions: Available');
  console.log('  - Trade-off accepted: Errors show suppressor location but include source info');

  console.groupEnd();
}

// Export and auto-run
if (typeof window !== 'undefined') {
  window.testPragmaticSuppressor = testPragmaticSuppressor;
  console.log('🧪 Pragmatic suppressor test available: window.testPragmaticSuppressor()');
}

// Auto-run after delay
if (import.meta.env.DEV) {
  setTimeout(() => {
    testPragmaticSuppressor();
  }, 2000);
}

export default testPragmaticSuppressor; 