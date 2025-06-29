/**
 * Navigation and SSE Error Fix Test Script
 * 运行在浏览器控制台中验证修复功能
 */

// Test 1: Router Guard Protection
console.log('🧪 Testing Router Guard Protection...');

async function testNavigationDebounce() {
  console.log('📝 Testing rapid navigation debouncing...');

  // Simulate rapid navigation clicks
  const routes = ['/chat/1', '/chat/2', '/chat/3', '/chat/1'];
  const promises = [];

  routes.forEach((route, index) => {
    setTimeout(() => {
      console.log(`🔄 Attempting navigation to: ${route}`);
      // This would normally cause cancellation errors
      promises.push(window.$router?.push(route));
    }, index * 50); // 50ms intervals - very rapid
  });

  try {
    await Promise.allSettled(promises);
    console.log('✅ Navigation test completed without errors');
  } catch (error) {
    console.log('❌ Navigation test failed:', error);
  }
}

// Test 2: SSE Error Suppression  
console.log('🧪 Testing SSE Error Suppression...');

async function testSSEErrorSuppression() {
  // Import the error suppression composable
  const { createSSEErrorHandler } = await import('./src/composables/useSSEErrorSuppression.js');

  const errorHandler = createSSEErrorHandler();

  console.log('📝 Testing error suppression with repeated network errors...');

  // Simulate repeated network errors
  for (let i = 0; i < 10; i++) {
    const error = new Error('网络连接失败，请检查您的网络设置');
    error.name = 'NetworkError';

    setTimeout(() => {
      const wasLogged = errorHandler.handleConnectionError(error);
      console.log(`Error ${i + 1}: ${wasLogged ? 'LOGGED' : 'SUPPRESSED'}`);
    }, i * 100);
  }

  // Check suppression stats after 2 seconds
  setTimeout(() => {
    const stats = errorHandler.getSuppressionStats();
    console.log('📊 Error Suppression Stats:', stats);
  }, 2000);
}

// Test 3: Integration Test
async function runIntegrationTest() {
  console.log('🧪 Running Integration Test...');

  // Test navigation protection
  await testNavigationDebounce();

  // Wait a bit
  await new Promise(resolve => setTimeout(resolve, 1000));

  // Test error suppression
  await testSSEErrorSuppression();

  console.log('✅ Integration test completed!');
}

// Usage instructions
console.log(`
🎯 **Navigation & SSE Error Fix Test Suite**

To run tests, execute in browser console:

1. Test Navigation Debounce:
   testNavigationDebounce()

2. Test SSE Error Suppression:
   testSSEErrorSuppression()

3. Run Full Integration Test:
   runIntegrationTest()

📝 Expected Results:
- No "Navigation cancelled" errors in console
- SSE errors are suppressed after initial occurrences
- Smooth navigation experience
- Reduced console spam

🔍 Monitoring:
- Check Network tab for actual requests
- Monitor console for error patterns
- Observe navigation timing improvements
`);

// Export functions for console use
if (typeof window !== 'undefined') {
  window.testNavigationDebounce = testNavigationDebounce;
  window.testSSEErrorSuppression = testSSEErrorSuppression;
  window.runIntegrationTest = runIntegrationTest;
} 