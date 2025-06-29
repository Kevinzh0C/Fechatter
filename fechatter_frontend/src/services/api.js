import axios from 'axios';

/**
 * Production-level API client with authentication and error handling
 */

// Create axios instance
const api = axios.create({
  baseURL: '/api',
  timeout: 30000,
  headers: {
    'Accept': 'application/json'
    // CRITICAL FIX: Removed default 'Content-Type': 'application/json'
    // This allows axios to automatically set the correct Content-Type based on request body:
    // - FormData → multipart/form-data; boundary=...
    // - Object → application/json
    // - String → text/plain
  }
});

/**
 * Request interceptor - Add authentication headers
 */
api.interceptors.request.use(
  async (config) => {
    try {
      // Import dependencies dynamically to avoid circular dependencies
      const { default: tokenManager } = await import('./tokenManager');
      const { default: authStateManager } = await import('../utils/authStateManager');

      // CRITICAL FIX: Get token with fallback mechanism
      // Priority 1: tokenManager (in-memory, fast)
      let token = tokenManager.getAccessToken();

      // Priority 2: authStateManager (localStorage, persistent)
      if (!token) {
        const authState = authStateManager.getAuthState();
        token = authState.token;

        // RECOVERY: If authStateManager has token but tokenManager doesn't,
        // restore tokenManager with the token for future requests
        if (token) {
          await tokenManager.setTokens({
            accessToken: token,
            refreshToken: token, // Using same token as refresh for stub implementation
            expiresAt: Date.now() + (3600 * 1000), // 1 hour default
            issuedAt: Date.now(),
          });

          if (import.meta.env.DEV) {
            console.log('🔄 [API] Restored tokenManager from authStateManager');
          }
        }
      }

      // Add Authorization header if token is available
      if (token && !config.skipAuthRefresh) {
        config.headers.Authorization = `Bearer ${token}`;
      }

      // CRITICAL FIX: Smart Content-Type handling for FormData
      // If body is FormData, remove any existing Content-Type to let browser set it automatically
      if (config.data instanceof FormData) {
        delete config.headers['Content-Type'];
        if (import.meta.env.DEV) {
          console.log('[API] Removed Content-Type header for FormData - browser will auto-set boundary');
        }
      }

      if (import.meta.env.DEV) {
        const hasToken = !!token;
        console.log(`🔗 API Request: ${config.method?.toUpperCase()} ${config.url}${hasToken ? ' (with auth)' : ' (no auth)'}`);
      }

      return config;
    } catch (error) {
      if (import.meta.env.DEV) {
        console.warn('Request interceptor error:', error);
      }
      return config;
    }
  },
  (error) => {
    return Promise.reject(error);
  }
);

/**
 * Response interceptor - Handle 401 errors and token refresh
 */
api.interceptors.response.use(
  (response) => {
    return response;
  },
  async (error) => {
    const originalRequest = error.config;

    // Handle 401 Unauthorized errors
    if (error.response?.status === 401 && !originalRequest._retry) {
      originalRequest._retry = true;

      if (import.meta.env.DEV) {
        console.warn(`🔐 401 Error on ${originalRequest.url}`);
      }

      // Skip token refresh for auth endpoints to avoid infinite loops
      if (originalRequest.skipAuthRefresh ||
        originalRequest.url?.includes('/signin') ||
        originalRequest.url?.includes('/signup') ||
        originalRequest.url?.includes('/refresh')) {
        return Promise.reject(error);
      }

      try {
        // Import tokenManager dynamically
        const { default: tokenManager } = await import('./tokenManager');

        // Check if we have a refresh token
        const tokens = tokenManager.getTokens();
        if (!tokens.refreshToken) {
          throw new Error('No refresh token available');
        }

        // Attempt to refresh the token
        await tokenManager.refreshToken();

        // Get the new token and retry the original request
        const newTokens = tokenManager.getTokens();
        if (newTokens.accessToken) {
          originalRequest.headers.Authorization = `Bearer ${newTokens.accessToken}`;

          if (import.meta.env.DEV) {
            console.log('Token refreshed, retrying request');
          }

          return api(originalRequest);
        }

      } catch (refreshError) {
        if (import.meta.env.DEV) {
          console.error('ERROR: Token refresh failed:', refreshError);
        }

        // Clear tokens and redirect to login
        try {
          const { default: tokenManager } = await import('./tokenManager');
          await tokenManager.clearTokens();
        } catch (clearError) {
          console.error('Error clearing tokens:', clearError);
        }

        // Redirect to login if not already there
        if (typeof window !== 'undefined' &&
          !window.location.pathname.includes('/login') &&
          !window.location.pathname.includes('/register')) {

          // Try to use router if available
          try {
            const { useRouter } = await import('vue-router');
            const router = useRouter();
            router.push('/login');
          } catch (routerError) {
            // Fallback to window.location
            window.location.href = '/login';
          }
        }

        return Promise.reject(error);
      }
    }

    // Handle other errors
    if (import.meta.env.DEV) {
      const status = error.response?.status;
      const url = error.config?.url;
      console.error(`🚨 API Error ${status}: ${url}`, error.response?.data);
    }

    return Promise.reject(error);
  }
);

export default api; 
