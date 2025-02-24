import axios from 'axios';

/**
 * Production-level API client with authentication and error handling
 */

// Create axios instance
const api = axios.create({
  baseURL: '/api',
  timeout: 30000,
  headers: {
    'Content-Type': 'application/json',
    'Accept': 'application/json'
  }
});

/**
 * Request interceptor - Add authentication headers
 */
api.interceptors.request.use(
  async (config) => {
    try {
      // Import tokenManager dynamically to avoid circular dependencies
      const { default: tokenManager } = await import('./tokenManager');

      // Get current token
      const tokens = tokenManager.getTokens();

      if (tokens.accessToken && !config.skipAuthRefresh) {
        config.headers.Authorization = `Bearer ${tokens.accessToken}`;
      }

      if (import.meta.env.DEV) {
        console.log(`🔗 API Request: ${config.method?.toUpperCase()} ${config.url}`);
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
            console.log('✅ Token refreshed, retrying request');
          }

          return api(originalRequest);
        }

      } catch (refreshError) {
        if (import.meta.env.DEV) {
          console.error('❌ Token refresh failed:', refreshError);
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