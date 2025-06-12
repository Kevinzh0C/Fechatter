/**
 * Test SSE Connection with Real Token
 * Verify that JWT key synchronization fixed the 401 issue
 */

export async function testSSEWithRealToken() {
  console.log('🔐 Testing SSE with real JWT token...');

  try {
    // Get real token from tokenManager
    const { default: tokenManager } = await import('@/services/tokenManager');
    const tokens = tokenManager.getTokens();

    if (!tokens.accessToken) {
      console.error('❌ No access token available');
      return false;
    }

    console.log('🔑 Using token:', tokens.accessToken.substring(0, 50) + '...');

    // Get SSE URL
    const { getApiConfig } = await import('@/utils/configLoader');
    const apiConfig = getApiConfig();
    const sseUrl = apiConfig.sse_url || '/events';

    const testUrl = `${sseUrl}?access_token=${encodeURIComponent(tokens.accessToken)}`;
    console.log('🔌 Testing SSE URL:', sseUrl);

    // Test with fetch first
    try {
      const response = await fetch(testUrl, {
        method: 'GET',
        headers: {
          'Accept': 'text/event-stream',
          'Cache-Control': 'no-cache'
        }
      });

      console.log('📊 Response status:', response.status);
      console.log('📊 Response headers:', Object.fromEntries(response.headers.entries()));

      if (response.status === 200) {
        console.log('✅ SSE endpoint accepts the token! 401 issue fixed!');
        return true;
      } else if (response.status === 401) {
        const errorText = await response.text();
        console.error('❌ Still getting 401:', errorText);
        return false;
      } else {
        console.warn('⚠️ Unexpected status:', response.status);
        const responseText = await response.text();
        console.log('Response body:', responseText);
        return false;
      }

    } catch (fetchError) {
      console.error('❌ Fetch failed:', fetchError.message);
      return false;
    }

  } catch (error) {
    console.error('❌ Test failed:', error.message);
    return false;
  }
}

/**
 * Test actual SSE connection
 */
export function testActualSSEConnection() {
  console.log('🧪 Testing actual SSE connection...');

  return new Promise((resolve) => {
    // Use the SSE service to test connection
    if (window.realtimeCommunicationService) {
      const service = window.realtimeCommunicationService;

      // Add one-time event listeners
      const onConnected = () => {
        console.log('✅ SSE connection established successfully!');
        service.off('connected', onConnected);
        service.off('permanently_failed', onFailed);
        resolve(true);
      };

      const onFailed = (data) => {
        console.error('❌ SSE connection failed:', data);
        service.off('connected', onConnected);
        service.off('permanently_failed', onFailed);
        resolve(false);
      };

      service.on('connected', onConnected);
      service.on('permanently_failed', onFailed);

      // Test connection with current token
      const { default: tokenManager } = import('@/services/tokenManager').then((module) => {
        const tokens = module.default.getTokens();
        if (tokens.accessToken) {
          service.connect(tokens.accessToken);
        } else {
          console.error('❌ No token available for SSE test');
          resolve(false);
        }
      });

      // Timeout after 10 seconds
      setTimeout(() => {
        service.off('connected', onConnected);
        service.off('permanently_failed', onFailed);
        console.warn('⏰ SSE connection test timed out');
        resolve(false);
      }, 10000);

    } else {
      console.error('❌ SSE service not available');
      resolve(false);
    }
  });
}

// Expose to window for easy testing
if (typeof window !== 'undefined') {
  window.testSSEWithRealToken = testSSEWithRealToken;
  window.testActualSSEConnection = testActualSSEConnection;
}

export default {
  testSSEWithRealToken,
  testActualSSEConnection
}; 