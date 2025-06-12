/**
 * Test Error Source Preservation
 * Verify that error sources are preserved when passing through content script suppressor
 * Production-grade test following Occam's Razor principle
 */

async function testErrorSourcePreservation() {
  console.group('🔍 Error Source Preservation Test');

  // Test 1: Direct console.error call
  console.log('\nTest 1: Direct console.error');
  console.log('  - Calling console.error from this file...');
  console.error('Test error from testErrorSourcePreservation.js');
  console.log('  - Check console: Should show this file as source, not contentScriptErrorSuppressor.js');

  // Test 2: API error simulation
  console.log('\nTest 2: API Error Simulation');
  try {
    // Create a mock API error similar to what api.js would produce
    const apiError = new Error('Request failed with status code 401');
    apiError.response = { status: 401, data: {} };
    apiError.config = { url: '/api/test' };

    console.error('🚨 API Error (via Gateway):', {
      message: apiError.message,
      code: 'ERR_BAD_REQUEST',
      response: apiError.response,
      status: 401,
      config: apiError.config
    });

    console.log('  - Check console: Should preserve original call location');
  } catch (e) {
    console.log('  - Error in test:', e.message);
  }

  // Test 3: Stack trace preservation
  console.log('\nTest 3: Stack Trace Test');
  function innerFunction() {
    console.error('Error from nested function');
  }

  function outerFunction() {
    innerFunction();
  }

  outerFunction();
  console.log('  - Check console: Stack should show innerFunction → outerFunction');

  // Test 4: Content script error (should be suppressed)
  console.log('\nTest 4: Content Script Error (Should be suppressed)');
  const contentScriptError = new Error('fetchError: Failed to fetch');
  contentScriptError.stack = `Error: fetchError: Failed to fetch
    at Zx (content script.js:5837:26922)
    at ul.sendMessage (content script.js:5837:26351)`;

  console.log('  - Attempting to log content script error...');
  console.error(contentScriptError);
  console.log('  - This error should have been suppressed');

  // Summary
  console.log('\n📊 Summary:');
  console.log('  - Non-content-script errors should show real source');
  console.log('  - Content script errors should be suppressed');
  console.log('  - No errors should show contentScriptErrorSuppressor.js as source');

  console.groupEnd();
}

// Export for manual testing
if (typeof window !== 'undefined') {
  window.testErrorSourcePreservation = testErrorSourcePreservation;
  console.log('🧪 Test available: window.testErrorSourcePreservation()');
}

// Auto-run in development after a delay
if (import.meta.env.DEV) {
  setTimeout(() => {
    console.log('🔧 Running error source preservation test...');
    testErrorSourcePreservation();
  }, 2000);
}

export default testErrorSourcePreservation; 