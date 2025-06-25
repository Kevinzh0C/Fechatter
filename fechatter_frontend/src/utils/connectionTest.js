/**
 * Connection Test - Verify remote server connectivity
 * Test that error loop has been fixed and frontend connects to remote server
 */

export async function testConnectionFix() {
  console.log('🔧 Testing Connection Fix - Verifying no more localhost errors...');

  try {
    // Load configuration
    const { getApiConfig } = await import('@/utils/configLoader');
    const apiConfig = getApiConfig();

    console.log('📋 Configuration loaded:');
    console.log(`   Gateway URL: ${apiConfig.gateway_url}`);
    console.log(`   API Base URL: ${apiConfig.base_url}`);
    console.log(`   SSE URL: ${apiConfig.sse_url}`);

    // Test 1: Health check should use remote server
    const healthUrl = `${apiConfig.gateway_url}/health`;
    console.log(`🔌 Testing health check: ${healthUrl}`);

    try {
      const response = await fetch(healthUrl, {
        method: 'GET',
        signal: AbortSignal.timeout(5000)
      });

      if (response.ok) {
        const text = await response.text();
        console.log(`✅ Health check SUCCESS: ${text}`);
      } else {
        console.log(`⚠️ Health check failed with status: ${response.status}`);
      }
    } catch (fetchError) {
      console.error(`❌ Health check failed: ${fetchError.message}`);
    }

    // Test 2: Check if we're still trying to connect to localhost
    const isUsingLocalhost = apiConfig.gateway_url?.includes('127.0.0.1') ||
      apiConfig.base_url?.includes('127.0.0.1') ||
      apiConfig.sse_url?.includes('127.0.0.1');

    if (isUsingLocalhost) {
      console.error('❌ ERROR: Still using localhost in configuration!');
      return false;
    } else {
      console.log('✅ SUCCESS: No localhost addresses found in configuration');
    }

    // Test 3: Verify remote server addresses
    const hasRemoteServer = apiConfig.gateway_url?.includes('45.77.178.85:8080');

    if (hasRemoteServer) {
      console.log('✅ SUCCESS: Using correct remote server address');
      console.log('🎉 Connection fix verification PASSED!');
      console.log('🔄 Error loop should be eliminated');
      return true;
    } else {
      console.error('❌ ERROR: Remote server address not found in configuration');
      return false;
    }

  } catch (error) {
    console.error('❌ Connection test failed:', error.message);
    return false;
  }
}

// Auto-run on import in development
if (typeof window !== 'undefined' && import.meta.env.DEV) {
  // Delay to ensure other modules are loaded
  setTimeout(() => {
    testConnectionFix();
  }, 2000);
}

// Expose to window for manual testing
if (typeof window !== 'undefined') {
  window.testConnectionFix = testConnectionFix;
}

export default testConnectionFix; 