/**
 * Test Unified Error Handler
 * Verify the single interception point works correctly
 */

function testUnifiedErrorHandler() {
  console.group('🧪 Testing Unified Error Handler');

  // Test 1: Check initialization
  console.log('\n1️⃣ System Status');
  const handler = window.unifiedErrorHandler;
  if (!handler) {
    console.error('❌ Unified error handler not found!');
    console.groupEnd();
    return;
  }

  const stats = handler.getStats();
  console.log('  ✅ Handler initialized:', stats.initialized);
  console.log('  - Registered handlers:', stats.handlers);
  console.log('  - Total handlers:', stats.totalHandlers);

  // Test 2: Simple error (should show real source)
  console.log('\n2️⃣ Simple Error Test');
  console.log('  - Logging error from this file...');
  console.error('Test error from testUnifiedErrorHandler.js');
  console.log('  - Check: Should show THIS file as source with annotation');

  // Test 3: API error simulation
  console.log('\n3️⃣ API Error Test');
  try {
    throw new Error('Simulated API error');
  } catch (error) {
    console.error('API Error:', error);
  }
  console.log('  - Check: Should show source annotation');

  // Test 4: Content script error (should be suppressed)
  console.log('\n4️⃣ Content Script Error Test');
  const beforeStats = handler.getStats();

  const contentError = new Error('fetchError: Failed to fetch');
  contentError.stack = `Error: fetchError: Failed to fetch
    at Zx (content script.js:123:45)
    at ul.sendMessage (content script.js:456:78)`;

  console.error('Extension error:', contentError);

  setTimeout(() => {
    const afterStats = handler.getStats();
    console.log('  - Suppressed:', afterStats.totalSuppressed > beforeStats.totalSuppressed ? '✅ Yes' : '❌ No');
    console.log('  - Total suppressed so far:', afterStats.totalSuppressed);
  }, 100);

  // Test 5: Check call stack depth
  console.log('\n5️⃣ Call Stack Test');
  function level1() { level2(); }
  function level2() { level3(); }
  function level3() {
    console.error('Error from nested function');
  }
  level1();
  console.log('  - Check: Should show level3 as source');

  // Test 6: Development optimizer filter
  console.log('\n6️⃣ Development Filter Test');
  console.error('Failed to fetch something');
  console.log('  - Check: Might be suppressed by development optimizer');

  // Summary
  setTimeout(() => {
    console.log('\n📊 Final Summary');
    const finalStats = handler.getStats();
    console.log('  - Total handlers:', finalStats.totalHandlers);
    console.log('  - Total suppressed:', finalStats.totalSuppressed);
    console.log('  - Unique errors:', finalStats.uniqueErrors);

    console.log('\n💡 Available commands:');
    console.log('  - unifiedErrorHandler.showSuppressedErrors()');
    console.log('  - unifiedErrorHandler.getStats()');
    console.log('  - unifiedErrorHandler.restore() - Restore original console.error');

    console.groupEnd();
  }, 200);
}

// Export and auto-run
if (typeof window !== 'undefined') {
  window.testUnifiedErrorHandler = testUnifiedErrorHandler;
  console.log('🧪 Unified error handler test available: window.testUnifiedErrorHandler()');

  // Auto-run after a delay
  setTimeout(() => {
    testUnifiedErrorHandler();
  }, 1000);
}

export default testUnifiedErrorHandler; 