/**
 * SSE Connection Fix
 * Handles SSE 401 errors and token refresh issues
 * Production-grade solution following Occam's Razor principle
 */

class SSEConnectionFix {
  constructor() {
    this.isFixing = false;
    this.lastFixTime = 0;
    this.fixDebounceMs = 5000; // 5 seconds debounce
  }

  /**
   * Main fix method - handles SSE 401 errors
   */
  async fixSSEConnection() {
    // Prevent excessive fix attempts
    const now = Date.now();
    if (this.isFixing || (now - this.lastFixTime) < this.fixDebounceMs) {
      if (import.meta.env.DEV) {
        console.log('🔧 SSE fix already in progress or too recent, skipping');
      return false;
    }

    this.isFixing = true;
    this.lastFixTime = now;

    try {
      if (import.meta.env.DEV) {
        console.log('🔧 Starting SSE connection fix...');
      }

      // Step 1: Check if we have a valid token
      const { default: tokenManager } = await import('@/services/tokenManager');
      let tokens = tokenManager.getTokens();

      if (!tokens.accessToken) {
        if (import.meta.env.DEV) {
          console.error('❌ No access token available for SSE fix');
        return false;
      }

      // Step 2: Check if token is expired or about to expire
      const isExpired = tokenManager.isTokenExpired();
      const shouldRefresh = tokenManager.shouldRefreshToken();

      if (isExpired || shouldRefresh) {
        if (import.meta.env.DEV) {
          console.log('🔐 Token is expired/expiring, refreshing...');
        }

        try {
          await tokenManager.refreshToken();
          tokens = tokenManager.getTokens(); // Get fresh tokens
          if (import.meta.env.DEV) {
            console.log('✅ Token refreshed successfully');
          }
        } catch (refreshError) {
          if (import.meta.env.DEV) {
            console.error('❌ Token refresh failed:', refreshError);
          }

          // If refresh fails, redirect to login
          this.handleAuthFailure();
          return false;
        }

      // Step 3: Fix SSE connection with fresh token
      if (window.realtimeCommunicationService) {
        const service = window.realtimeCommunicationService;
        const currentState = service.getConnectionState();

        // Disconnect existing failed connection
        if (currentState.connectionState === 'disconnected' ||
          currentState.connectionState === 'error' ||
          !currentState.isConnected) {

          if (import.meta.env.DEV) {
            console.log('🔌 Disconnecting failed SSE connection...');
          service.disconnect();

          // Wait a moment before reconnecting
          await new Promise(resolve => setTimeout(resolve, 1000));

          if (import.meta.env.DEV) {
            console.log('🔌 Reconnecting SSE with fresh token...');
          await service.connect(tokens.accessToken);

          if (import.meta.env.DEV) {
            console.log('✅ SSE connection fix completed');
          return true;
        } else {
          if (import.meta.env.DEV) {
            console.log('✅ SSE connection is already healthy');
          return true;
        }
      } else {
        if (import.meta.env.DEV) {
          console.warn('⚠️ SSE service not available');
        return false;
      }

    } catch (error) {
      if (import.meta.env.DEV) {
        console.error('❌ SSE connection fix failed:', error);
      return false;
    } finally {
      this.isFixing = false;
    }

  /**
   * Handle authentication failure
   */
  handleAuthFailure() {
    if (import.meta.env.DEV) {
      console.log('🔐 Authentication failed, redirecting to login...');
    }

    // Clear all auth data
    localStorage.clear();
    sessionStorage.clear();

    // Redirect to login
    if (window.location.pathname !== '/login') {
      window.location.href = '/login';
    }

  /**
   * Test SSE endpoint with current token
   */
  async testSSEEndpoint() {
    try {
      const { default: tokenManager } = await import('@/services/tokenManager');
      const tokens = tokenManager.getTokens();

      if (!tokens.accessToken) {
        if (import.meta.env.DEV) {
          console.error('❌ No token available for SSE test');
        return false;
      }

      // Get SSE URL from config
      const { getApiConfig } = await import('@/utils/configLoader');
      const apiConfig = getApiConfig();
      const sseUrl = apiConfig.sse_url || '/events';

      const testUrl = `${sseUrl}?access_token=${encodeURIComponent(tokens.accessToken)}`;

      if (import.meta.env.DEV) {
        console.log('🧪 Testing SSE endpoint:', sseUrl);
      }

      const response = await fetch(testUrl, {
        method: 'GET',
        headers: {
          'Accept': 'text/event-stream',
          'Cache-Control': 'no-cache'
        }
      });

      if (response.status === 200) {
        if (import.meta.env.DEV) {
          console.log('✅ SSE endpoint test passed');
        return true;
      } else if (response.status === 401) {
        if (import.meta.env.DEV) {
          console.error('❌ SSE endpoint returned 401 - token issue');
        return false;
      } else {
        if (import.meta.env.DEV) {
          console.warn('⚠️ SSE endpoint returned unexpected status:', response.status);
        return false;
      }

    } catch (error) {
      if (import.meta.env.DEV) {
        console.error('❌ SSE endpoint test failed:', error);
      return false;
    }

  /**
   * Auto-fix SSE connection issues
   */
  async autoFix() {
    if (import.meta.env.DEV) {
      console.log('🔧 Running SSE auto-fix...');
    }

    // Test endpoint first
    const endpointOk = await this.testSSEEndpoint();

    if (!endpointOk) {
      if (import.meta.env.DEV) {
        console.log('🔧 Endpoint test failed, attempting connection fix...');
      const fixResult = await this.fixSSEConnection();

      if (fixResult) {
        // Test again after fix
        const retestResult = await this.testSSEEndpoint();
        if (import.meta.env.DEV) {
          console.log('🔧 Auto-fix result:', retestResult ? 'SUCCESS' : 'FAILED');
        return retestResult;
      } else {
        if (import.meta.env.DEV) {
          console.log('🔧 Auto-fix failed');
        return false;
      }
    } else {
      if (import.meta.env.DEV) {
        console.log('✅ SSE endpoint is healthy, no fix needed');
      return true;
    }

  /**
   * Monitor SSE connection and auto-fix issues
   */
  startMonitoring() {
    // Monitor every 30 seconds
    setInterval(async () => {
      if (window.realtimeCommunicationService) {
        const service = window.realtimeCommunicationService;
        const state = service.getConnectionState();

        // Auto-fix if disconnected or failed
        if (!state.isConnected &&
          state.connectionState !== 'connecting' &&
          state.connectionState !== 'permanently_failed') {

          if (import.meta.env.DEV) {
            console.log('🔧 SSE monitoring detected issue, auto-fixing...');
          await this.autoFix();
        }
    }, 30000); // 30 seconds

    if (import.meta.env.DEV) {
      console.log('👁️ SSE connection monitoring started');
    }

// Create singleton instance
const sseConnectionFix = new SSEConnectionFix();

// Export for global use
if (typeof window !== 'undefined') {
  window.sseConnectionFix = sseConnectionFix;

  // Auto-start monitoring after page load
  window.addEventListener('load', () => {
    setTimeout(() => {
      sseConnectionFix.startMonitoring();
    }, 5000); // Start monitoring after 5 seconds
  });

  if (import.meta.env.DEV) {
    console.log('🔧 SSE Connection Fix loaded - use window.sseConnectionFix');
  }

export default sseConnectionFix; 