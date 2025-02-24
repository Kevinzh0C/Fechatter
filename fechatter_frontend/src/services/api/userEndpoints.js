/**
 * User API Endpoints with Production Fallback Mechanisms
 * 
 * Handles multiple endpoint strategies to ensure compatibility
 * with different backend configurations and deployment stages
 */

import api from '../api.js';
import { productionLogManager } from '@/utils/productionLogManager.js';

export class UserEndpointManager {
  constructor() {
    this.endpointStrategy = 'auto'; // auto, workspace, profile, mock
    this.fallbackEndpoints = [
      '/users',              // Primary: Correct backend endpoint (requires auth + workspace)
      '/users/profile',      // Secondary: User profile endpoint
    ];
    this.lastSuccessfulEndpoint = null;
    this.userCache = new Map();
    this.cacheExpiry = 5 * 60 * 1000; // 5 minutes

    // 🔧 NEW: Enhanced auth monitoring
    this.authRetryCount = 0;
    this.maxAuthRetries = 2;
  }

  /**
   * Fetch workspace users with automatic endpoint detection
   */
  async fetchWorkspaceUsers() {
    // Try cached result first
    const cached = this.getCachedUsers();
    if (cached) {
      productionLogManager.debug('UserEndpoints', 'Returning cached workspace users', { count: cached.length });
      return cached;
    }

    // Try endpoints in order until one succeeds
    for (const endpoint of this.fallbackEndpoints) {
      try {
        productionLogManager.debug('UserEndpoints', `Attempting endpoint: ${endpoint}`);

        const response = await this.tryEndpoint(endpoint);
        if (response && response.data) {
          this.lastSuccessfulEndpoint = endpoint;
          const users = this.normalizeUserData(response.data);
          this.cacheUsers(users);

          productionLogManager.info('UserEndpoints', `Successfully fetched users from ${endpoint}`, { count: users.length });
          return users;
        }
      } catch (error) {
        productionLogManager.warn('UserEndpoints', `Endpoint ${endpoint} failed: ${error.message}`);
        continue;
      }
    }

    // All endpoints failed, return mock data in development
    if (import.meta.env.DEV) {
      productionLogManager.warn('UserEndpoints', 'All endpoints failed, using mock data');
      return this.getMockUsers();
    }

    throw new Error('All user endpoints failed');
  }

  /**
   * Try a specific endpoint with appropriate method and auth handling
   */
  async tryEndpoint(endpoint) {
    try {
      const response = await api.get(endpoint);

      // Reset auth retry count on success
      this.authRetryCount = 0;

      return response;
    } catch (error) {
      // Handle 401 Unauthorized errors
      if (error.response?.status === 401) {
        productionLogManager.warn('UserEndpoints', `Authentication failed for ${endpoint}: ${error.message}`);

        // Try to refresh token if we haven't exceeded retry limit
        if (this.authRetryCount < this.maxAuthRetries) {
          this.authRetryCount++;

          try {
            // Force token refresh through auth store
            await this.attemptTokenRefresh();

            // Retry the request with refreshed token
            productionLogManager.info('UserEndpoints', `Retrying ${endpoint} after token refresh (attempt ${this.authRetryCount})`);
            return await api.get(endpoint);
          } catch (refreshError) {
            productionLogManager.error('UserEndpoints', `Token refresh failed: ${refreshError.message}`);
            throw error; // Throw original 401 error
          }
        }
      }

      throw error;
    }
  }

  /**
   * Attempt to refresh authentication token
   */
  async attemptTokenRefresh() {
    // Try to get auth store and refresh token
    try {
      // Check if we're in a browser environment with access to stores
      if (typeof window !== 'undefined' && window.__pinia_stores__) {
        const authStore = window.__pinia_stores__.auth();
        if (authStore && typeof authStore.refreshToken === 'function') {
          await authStore.refreshToken();
          productionLogManager.info('UserEndpoints', 'Token refreshed successfully via auth store');
          return;
        }
      }

      // Fallback: direct API call to refresh endpoint
      const refreshToken = localStorage.getItem('refresh_token');
      if (refreshToken) {
        const response = await fetch('/api/refresh', {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: JSON.stringify({ refresh_token: refreshToken })
        });

        if (response.ok) {
          const data = await response.json();
          // Update localStorage with new tokens
          localStorage.setItem('access_token', data.access_token);
          if (data.refresh_token) {
            localStorage.setItem('refresh_token', data.refresh_token);
          }
          productionLogManager.info('UserEndpoints', 'Token refreshed successfully via direct API');
          return;
        }
      }

      throw new Error('No valid refresh token available');
    } catch (error) {
      productionLogManager.error('UserEndpoints', `Token refresh attempt failed: ${error.message}`);
      throw error;
    }
  }

  /**
   * Normalize user data from different endpoint formats
   */
  normalizeUserData(data) {
    // Handle different response formats
    let users = [];

    if (Array.isArray(data)) {
      users = data;
    } else if (data.data && Array.isArray(data.data)) {
      users = data.data;
    } else if (data.users && Array.isArray(data.users)) {
      users = data.users;
    } else if (data.id) {
      // Single user from profile endpoint
      users = [data];
    } else {
      users = [];
    }

    // Normalize user objects
    return users.map(user => ({
      id: user.id,
      fullname: user.fullname || user.full_name || user.name || 'Unknown User',
      email: user.email || '',
      avatar_url: user.avatar_url || null,
      status: user.status || 'Active',
      title: user.title || '',
      department: user.department || '',
      last_seen_at: user.last_seen_at || user.last_active_at,
      created_at: user.created_at || new Date().toISOString(),
      workspace_id: user.workspace_id || 1
    }));
  }

  /**
   * Cache management
   */
  cacheUsers(users) {
    this.userCache.set('workspace_users', {
      data: users,
      timestamp: Date.now()
    });
  }

  getCachedUsers() {
    const cached = this.userCache.get('workspace_users');
    if (cached && (Date.now() - cached.timestamp < this.cacheExpiry)) {
      return cached.data;
    }
    return null;
  }

  clearCache() {
    this.userCache.clear();
  }

  /**
   * Mock users for development fallback
   */
  getMockUsers() {
    return [
      {
        id: 1,
        fullname: 'Development User',
        email: 'dev@fechatter.local',
        status: 'Active',
        avatar_url: null,
        title: 'Developer',
        department: 'Engineering',
        created_at: new Date().toISOString(),
        workspace_id: 1
      },
      {
        id: 2,
        fullname: 'Test User',
        email: 'test@fechatter.local',
        status: 'Active',
        avatar_url: null,
        title: 'Tester',
        department: 'QA',
        created_at: new Date().toISOString(),
        workspace_id: 1
      }
    ];
  }

  /**
   * Fetch users by IDs with fallback
   */
  async fetchUsersByIds(userIds) {
    if (!userIds || userIds.length === 0) {
      return [];
    }

    try {
      // Try batch endpoint first
      const response = await api.post('/users/batch', { user_ids: userIds });
      return this.normalizeUserData(response.data);
    } catch (error) {
      productionLogManager.warn('UserEndpoints', 'Batch fetch failed, using workspace users fallback');

      // Fallback: get from workspace users
      const allUsers = await this.fetchWorkspaceUsers();
      return allUsers.filter(user => userIds.includes(user.id));
    }
  }

  /**
   * Get current user profile
   */
  async getCurrentUserProfile() {
    try {
      const response = await api.get('/users/profile');
      return this.normalizeUserData(response.data)[0];
    } catch (error) {
      productionLogManager.error('UserEndpoints', 'Failed to fetch current user profile', error);
      throw error;
    }
  }

  /**
   * Set endpoint strategy manually
   */
  setEndpointStrategy(strategy) {
    this.endpointStrategy = strategy;
    productionLogManager.info('UserEndpoints', `Endpoint strategy set to: ${strategy}`);
  }

  /**
   * Get comprehensive diagnostics information including auth status
   */
  getDiagnostics() {
    const authToken = localStorage.getItem('access_token');
    const refreshToken = localStorage.getItem('refresh_token');

    return {
      lastSuccessfulEndpoint: this.lastSuccessfulEndpoint,
      strategy: this.endpointStrategy,
      cacheSize: this.userCache.size,
      cachedUsers: this.getCachedUsers()?.length || 0,
      endpointsToTry: this.fallbackEndpoints,
      authStatus: {
        hasAccessToken: !!authToken,
        hasRefreshToken: !!refreshToken,
        authRetryCount: this.authRetryCount,
        maxAuthRetries: this.maxAuthRetries,
        tokenExpiry: this.getTokenExpiry(authToken)
      }
    };
  }

  /**
   * Get token expiry information
   */
  getTokenExpiry(token) {
    if (!token) return null;

    try {
      const payload = JSON.parse(atob(token.split('.')[1]));
      return {
        exp: payload.exp,
        iat: payload.iat,
        isExpired: Date.now() / 1000 > payload.exp,
        expiresIn: Math.max(0, payload.exp - Date.now() / 1000)
      };
    } catch (error) {
      return { error: 'Invalid token format' };
    }
  }

  /**
   * Force auth refresh for debugging
   */
  async forceAuthRefresh() {
    productionLogManager.info('UserEndpoints', 'Forcing authentication refresh for debugging');
    this.authRetryCount = 0; // Reset retry count
    return await this.attemptTokenRefresh();
  }

  /**
   * Test endpoint connectivity and auth status
   */
  async testEndpoints() {
    const results = {};

    for (const endpoint of this.fallbackEndpoints) {
      try {
        productionLogManager.info('UserEndpoints', `Testing endpoint: ${endpoint}`);
        const startTime = Date.now();
        const response = await this.tryEndpoint(endpoint);
        results[endpoint] = {
          status: 'success',
          data: response.data ? response.data.length : 0,
          responseTime: Date.now() - startTime
        };
      } catch (error) {
        results[endpoint] = {
          status: 'failed',
          error: error.message,
          statusCode: error.response?.status,
          responseTime: Date.now() - startTime
        };
      }
    }

    return results;
  }
}

// Export singleton instance
export const userEndpointManager = new UserEndpointManager();

// Global access for debugging
if (import.meta.env.DEV) {
  window.userEndpoints = userEndpointManager;

  // 🔧 Enhanced debugging commands
  window.diagnoseDUserAPI = () => {
    console.group('🔬 User API Diagnostics');
    const diagnostics = userEndpointManager.getDiagnostics();
    console.table(diagnostics);
    console.log('Full diagnostics:', diagnostics);
    console.groupEnd();
    return diagnostics;
  };

  window.testDUserEndpoints = async () => {
    console.log('🧪 Testing user endpoints...');
    const results = await userEndpointManager.testEndpoints();
    console.table(results);
    return results;
  };

  window.fixDUserAuth = async () => {
    console.log('🔧 Attempting to fix user authentication...');
    try {
      await userEndpointManager.forceAuthRefresh();
      console.log('✅ Authentication refresh completed');

      // Test endpoints after refresh
      const results = await userEndpointManager.testEndpoints();
      console.log('🧪 Post-refresh test results:');
      console.table(results);

      return { success: true, results };
    } catch (error) {
      console.error('❌ Authentication fix failed:', error);
      return { success: false, error: error.message };
    }
  };
} 