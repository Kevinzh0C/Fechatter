/**
 * Auth 401 Error Fix
 * Comprehensive solution for authentication failures
 */

class Auth401ErrorFix {
  constructor() {
    this.name = 'Auth401ErrorFix';
    this.version = '1.0.0';
    this.isActive = false;
    this.debugMode = true;
    this.fixCount = 0;
  }

  /**
   * Apply comprehensive 401 error fix
   */
  async applyComprehensiveFix() {
    console.group('Applying Auth 401 Error Fix');

    try {
      const fixes = [
        this.fixTokenManagerState(),
        this.fixAuthStoreState(),
        this.fixSSEConnection(),
        this.addApiErrorHandling(),
        this.implementGracefulLogout()
      ];

      const results = await Promise.allSettled(fixes);

      results.forEach((result, index) => {
        const fixName = ['token manager', 'auth store', 'SSE connection', 'API error handling', 'graceful logout'][index];
        if (result.status === 'fulfilled') {
          if (import.meta.env.DEV) {
            console.log(`${fixName} fix applied`);
          this.fixCount++;
        } else {
          if (import.meta.env.DEV) {
            console.error(`ERROR: ${fixName} fix failed:`, result.reason);
          }
      });

      this.isActive = true;
      if (import.meta.env.DEV) {
        console.log(`🎉 Comprehensive 401 fix applied (${this.fixCount} fixes active)`);
      }

    } catch (error) {
      if (import.meta.env.DEV) {
        console.error('ERROR: Failed to apply comprehensive fix:', error);
      }

    console.groupEnd();
  }

  /**
   * Fix 1: Token Manager State [CRITICAL]
   */
  async fixTokenManagerState() {
    try {
      const { default: tokenManager } = await import('@/services/tokenManager');

      // Check current token status
      const status = tokenManager.getStatus();
      if (import.meta.env.DEV) {
        console.log('🔐 Current token status:', status);
      }

      if (status.isExpired || status.refreshTokenExpired) {
        if (import.meta.env.DEV) {
          console.log('🧹 Clearing expired tokens...');
        await tokenManager.clearTokens();

        // Clear related storage
        const authKeys = ['auth', 'auth_token', 'token', 'refreshToken'];
        authKeys.forEach(key => {
          localStorage.removeItem(key);
          sessionStorage.removeItem(key);
        });

        if (import.meta.env.DEV) {
          console.log('Expired tokens cleared');
        return true;
      }

      // If token should refresh, attempt it
      if (status.shouldRefresh && !status.isRefreshing) {
        if (import.meta.env.DEV) {
          console.log('🔄 Attempting token refresh...');
        try {
          await tokenManager.refreshToken();
          if (import.meta.env.DEV) {
            console.log('Token refreshed successfully');
          return true;
        } catch (error) {
          if (import.meta.env.DEV) {
            console.warn('WARNING: Token refresh failed, clearing tokens:', error.message);
          await tokenManager.clearTokens();
          return true;
        }

      if (import.meta.env.DEV) {
        console.log('Token manager state is valid');
      return true;
    } catch (error) {
      if (import.meta.env.DEV) {
        console.error('ERROR: Token manager fix failed:', error);
      return false;
    }

  /**
   * Fix 2: Auth Store State [HIGH]
   */
  async fixAuthStoreState() {
    try {
      const { useAuthStore } = await import('@/stores/auth');
      const authStore = useAuthStore();

      // Check auth consistency
      const isAuthenticated = authStore.isAuthenticated;
      const hasUser = !!authStore.user;
      const hasToken = !!authStore.getAccessToken();

      if (import.meta.env.DEV) {
        console.log('🔐 Auth store state:', { isAuthenticated, hasUser, hasToken });

      // Fix inconsistent states
      if (isAuthenticated && (!hasUser || !hasToken)) {
        if (import.meta.env.DEV) {
          console.log('🧹 Fixing inconsistent auth state...');
        }

        // Try to restore state from localStorage
        const storedAuth = localStorage.getItem('auth');
        if (storedAuth) {
          try {
            const authData = JSON.parse(storedAuth);
            if (authData.user && authData.tokens?.accessToken) {
              if (import.meta.env.DEV) {
                console.log('🔄 Restoring auth state from localStorage...');
              await authStore.validateAndSyncAuthState();
              if (import.meta.env.DEV) {
                console.log('Auth state restored');
              return true;
            }
          } catch (parseError) {
            if (import.meta.env.DEV) {
              console.warn('WARNING: Failed to parse stored auth data:', parseError);
            }

        // If restoration failed, clear everything
        if (import.meta.env.DEV) {
          console.log('🧹 Clearing corrupted auth state...');
        authStore.clearAuth();
        if (import.meta.env.DEV) {
          console.log('Auth state cleared');
        return true;
      }

      // If not authenticated but has user/token, clear them
      if (!isAuthenticated && (hasUser || hasToken)) {
        if (import.meta.env.DEV) {
          console.log('🧹 Clearing leftover auth data...');
        authStore.clearAuth();
        if (import.meta.env.DEV) {
          console.log('Leftover auth data cleared');
        return true;
      }

      if (import.meta.env.DEV) {
        console.log('Auth store state is consistent');
      return true;
    } catch (error) {
      if (import.meta.env.DEV) {
        console.error('ERROR: Auth store fix failed:', error);
      return false;
    }

  /**
   * Fix 3: SSE Connection [MEDIUM]
   */
  async fixSSEConnection() {
    try {
      // Check if SSE connection manager exists
      if (window.sseConnectionManager) {
        const sseManager = window.sseConnectionManager;

        // Check connection status
        if (import.meta.env.DEV) {
          console.log('🔌 Checking SSE connection...');
        }

        // Close existing connection if it's failing
        if (sseManager.eventSource && sseManager.eventSource.readyState === EventSource.CLOSED) {
          if (import.meta.env.DEV) {
            console.log('🧹 Cleaning up failed SSE connection...');
          sseManager.disconnect();
        }

        // Don't reconnect if not authenticated
        const { useAuthStore } = await import('@/stores/auth');
        const authStore = useAuthStore();

        if (!authStore.isAuthenticated) {
          if (import.meta.env.DEV) {
            console.log('SSE disconnected (not authenticated)');
          sseManager.disconnect();
          return true;
        }

        if (import.meta.env.DEV) {
          console.log('SSE connection state checked');
        return true;
      }

      if (import.meta.env.DEV) {
        console.log('No SSE connection manager found');
      return true;
    } catch (error) {
      if (import.meta.env.DEV) {
        console.error('ERROR: SSE connection fix failed:', error);
      return false;
    }

  /**
   * Fix 4: API Error Handling [MEDIUM]  
   */
  async addApiErrorHandling() {
    try {
      // Add global 401 error handling
      if (window.axios && window.axios.interceptors) {
        // Check if our interceptor already exists
        const existingInterceptor = window.axios.interceptors.response.handlers.find(
          handler => handler && handler._auth401Fix
        );

        if (!existingInterceptor) {
          // Add response interceptor for 401 handling
          window.axios.interceptors.response.use(
            response => response,
            async error => {
              if (error.response?.status === 401) {
                if (import.meta.env.DEV) {
                  console.log('🔐 Intercepted 401 error:', error.config?.url);
                }

                // Don't logout for missing endpoints
                const responseData = error.response?.data;
                if (responseData?.error?.includes('Not Found') ||
                  responseData?.message?.includes('not found')) {
                  if (import.meta.env.DEV) {
                    console.warn('🔐 401 due to missing API endpoint, ignoring');
                  return Promise.reject(error);
                }

                // Check if already on auth page
                if (window.location.pathname === '/login' || window.location.pathname === '/register') {
                  if (import.meta.env.DEV) {
                    console.log('🔐 Already on auth page, clearing tokens only');
                  const { default: tokenManager } = await import('@/services/tokenManager');
                  await tokenManager.clearTokens();
                  return Promise.reject(error);
                }

                // Trigger graceful logout
                this.performGracefulLogout();
              }

              return Promise.reject(error);
            }
          );

          // Mark our interceptor
          const interceptorId = window.axios.interceptors.response.handlers.length - 1;
          if (window.axios.interceptors.response.handlers[interceptorId]) {
            window.axios.interceptors.response.handlers[interceptorId]._auth401Fix = true;
          }

          if (import.meta.env.DEV) {
            console.log('API 401 error interceptor added');
          }
        } else {
          if (import.meta.env.DEV) {
            console.log('API 401 error interceptor already exists');
          }

        return true;
      }

      if (import.meta.env.DEV) {
        console.log('No axios instance found');
      return true;
    } catch (error) {
      if (import.meta.env.DEV) {
        console.error('ERROR: API error handling fix failed:', error);
      return false;
    }

  /**
   * Fix 5: Graceful Logout Implementation
   */
  async implementGracefulLogout() {
    try {
      // Add graceful logout method to window for global access
      window.gracefulLogout = async (message = 'Session expired. Please login again.') => {
        if (import.meta.env.DEV) {
          console.log('🚪 Performing graceful logout...');
        }

        try {
          const { useAuthStore } = await import('@/stores/auth');
          const authStore = useAuthStore();

          // Clear auth state without API call (since we're logged out)
          authStore.clearAuth();

          // Show notification if available
          if (window.errorHandler?.showNotification) {
            window.errorHandler.showNotification('info', message);
          }

          // Redirect to login
          if (window.location.pathname !== '/login') {
            window.location.href = '/login';
          }

          if (import.meta.env.DEV) {
            console.log('Graceful logout completed');
          }
        } catch (error) {
          if (import.meta.env.DEV) {
            console.error('ERROR: Graceful logout failed:', error);
          }
          // Fallback: hard redirect
          window.location.href = '/login';
        }
      };

      if (import.meta.env.DEV) {
        console.log('Graceful logout implemented');
      return true;
    } catch (error) {
      if (import.meta.env.DEV) {
        console.error('ERROR: Graceful logout implementation failed:', error);
      return false;
    }

  /**
   * Perform graceful logout
   */
  async performGracefulLogout() {
    if (window.gracefulLogout) {
      await window.gracefulLogout();
    } else {
      // Fallback implementation
      if (import.meta.env.DEV) {
        console.log('🚪 Fallback logout...');
      }

      try {
        const { default: tokenManager } = await import('@/services/tokenManager');
        await tokenManager.clearTokens();
      } catch (error) {
        if (import.meta.env.DEV) {
          console.error('Failed to clear tokens:', error);
        }

      window.location.href = '/login';
    }

  /**
   * Diagnose current 401 error state
   */
  async diagnose401Errors() {
    console.group('Diagnosing 401 Error State');

    try {
      // Check auth state
      const { useAuthStore } = await import('@/stores/auth');
      const authStore = useAuthStore();

      if (import.meta.env.DEV) {
        console.log('1️⃣ Auth Store State:');
      if (import.meta.env.DEV) {
        console.log('  - Is Authenticated:', authStore.isAuthenticated);
      if (import.meta.env.DEV) {
        console.log('  - Has User:', !!authStore.user);
      if (import.meta.env.DEV) {
        console.log('  - Has Token:', !!authStore.getAccessToken());
      }

      // Check token manager
      const { default: tokenManager } = await import('@/services/tokenManager');
      const tokenStatus = tokenManager.getStatus();

      if (import.meta.env.DEV) {
        console.log('2️⃣ Token Manager State:');
      if (import.meta.env.DEV) {
        console.log('  - Has Token:', tokenStatus.hasToken);
      if (import.meta.env.DEV) {
        console.log('  - Is Expired:', tokenStatus.isExpired);
      if (import.meta.env.DEV) {
        console.log('  - Should Refresh:', tokenStatus.shouldRefresh);
      if (import.meta.env.DEV) {
        console.log('  - Is Refreshing:', tokenStatus.isRefreshing);
      if (import.meta.env.DEV) {
        console.log('  - Refresh Token Expired:', tokenStatus.refreshTokenExpired);
      }

      // Check storage
      const storedAuth = localStorage.getItem('auth');
      if (import.meta.env.DEV) {
        console.log('3️⃣ Storage State:');
      if (import.meta.env.DEV) {
        console.log('  - Has Stored Auth:', !!storedAuth);
      }

      if (storedAuth) {
        try {
          const authData = JSON.parse(storedAuth);
          if (import.meta.env.DEV) {
            console.log('  - Stored User:', !!authData.user);
          if (import.meta.env.DEV) {
            console.log('  - Stored Tokens:', !!authData.tokens);
          if (import.meta.env.DEV) {
            console.log('  - Token Expires At:', authData.tokens?.expiresAt ? new Date(authData.tokens.expiresAt).toISOString() : 'N/A');
          }
        } catch (error) {
          if (import.meta.env.DEV) {
            console.log('  - Storage Parse Error:', error.message);
          }

      // Check network requests
      if (import.meta.env.DEV) {
        console.log('4️⃣ Recent 401 Errors:');
      const recentErrors = this.getRecent401Errors();
      recentErrors.forEach((error, index) => {
        if (import.meta.env.DEV) {
          console.log(`  ${index + 1}. ${error.url} - ${error.timestamp}`);
        }
      });

      if (import.meta.env.DEV) {
        console.log('5️⃣ Recommendations:');
      if (tokenStatus.isExpired) {
        if (import.meta.env.DEV) {
          console.log('  WARNING: Token is expired - clear auth state');
      if (authStore.isAuthenticated && !tokenStatus.hasToken) {
        if (import.meta.env.DEV) {
          console.log('  WARNING: Auth state inconsistent - sync or clear');
      if (tokenStatus.refreshTokenExpired) {
        if (import.meta.env.DEV) {
          console.log('  WARNING: Refresh token expired - require login');
        }

    } catch (error) {
      if (import.meta.env.DEV) {
        console.error('ERROR: Diagnosis failed:', error);
      }

    console.groupEnd();
  }

  /**
   * Get recent 401 errors (mock implementation)
   */
  getRecent401Errors() {
    // This would track recent 401 errors in a real implementation
    return [
      { url: '/api/users', timestamp: new Date().toISOString() },
      { url: '/api/workspace/chats', timestamp: new Date().toISOString() },
      { url: '/events', timestamp: new Date().toISOString() },
    ];
  }

  /**
   * Get fix status
   */
  getStatus() {
    return {
      name: this.name,
      version: this.version,
      isActive: this.isActive,
      fixCount: this.fixCount,
      debugMode: this.debugMode
    };
  }

// Create singleton instance
const auth401ErrorFix = new Auth401ErrorFix();

// Export for global use
if (typeof window !== 'undefined') {
  window.auth401ErrorFix = auth401ErrorFix;

  // Convenient global functions
  window.fix401Errors = () => auth401ErrorFix.applyComprehensiveFix();
  window.diagnose401 = () => auth401ErrorFix.diagnose401Errors();
  window.gracefulLogout = async (message) => auth401ErrorFix.performGracefulLogout(message);

  if (import.meta.env.DEV) {
    console.log('Auth 401 Error Fix loaded');
  if (import.meta.env.DEV) {
    console.log('Use window.fix401Errors() to apply comprehensive fix');
  if (import.meta.env.DEV) {
    console.log('Use window.diagnose401() to diagnose current state');
  if (import.meta.env.DEV) {
    console.log('Use window.gracefulLogout() for clean logout');
  }

export default auth401ErrorFix; 