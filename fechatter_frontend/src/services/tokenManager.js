/**
 * Token Manager - Minimal Stub
 * 
 * Temporary stub to restore build; will be reimplemented in sub-tasks
 */

import { authEventBus, AuthEvents } from './auth-events';

class TokenManager {
  constructor() {
    this.tokens = {
      accessToken: null,
      refreshToken: null,
      expiresAt: null,
      issuedAt: null,
      absoluteExpiry: null,
    };

    this.config = {
      refreshThreshold: 5 * 60 * 1000,
      refreshPercentage: 0.5,
      clockSkewBuffer: 30 * 1000,
      maxRefreshRetries: 3,
      refreshBackoffMs: 1000,
      maxBackoffMs: 30000,
      inactivityTimeout: 30 * 60 * 1000,
      activityCheckInterval: 60 * 1000,
    };

    this.state = {
      isRefreshing: false,
      refreshPromise: null,
      refreshFailureCount: 0,
      lastRefreshTime: null,
      lastActivityTime: Date.now(),
      activityCheckTimer: null,
      refreshTimer: null,
    };

    this.eventListeners = new Map();
    this.authState = {
      lastAuthErrorTime: null,
      authErrorDebounceMs: 5000,
      isLoggingOut: false,
      inactivityErrorSent: false,
    };

    // Store auth service reference to avoid circular imports
    this._authService = null;
  }

  initialize() {
    // Stub implementation
  }

  // Set auth service reference to avoid circular dependencies
  setAuthService(authService) {
    this._authService = authService;
  }

  async setTokens(tokenData) {
    const now = Date.now();
    this.tokens = {
      accessToken: tokenData.accessToken,
      refreshToken: tokenData.refreshToken,
      expiresAt: tokenData.expiresAt || (now + (tokenData.expiresIn * 1000)),
      issuedAt: tokenData.issuedAt || now,
      absoluteExpiry: tokenData.absoluteExpiry || (now + (30 * 24 * 60 * 60 * 1000)),
    };
  }

  getAccessToken() {
    return this.tokens.accessToken;
  }

  getRefreshToken() {
    return this.tokens.refreshToken;
  }

  getTokens() {
    return { ...this.tokens };
  }

  isTokenExpired() {
    if (!this.tokens.expiresAt) return true;
    return Date.now() + this.config.clockSkewBuffer > this.tokens.expiresAt;
  }

  shouldRefreshToken() {
    if (!this.tokens.accessToken || !this.tokens.expiresAt) return false;

    const now = Date.now();
    const timeUntilExpiry = this.tokens.expiresAt - now;

    // Refresh if token expires within threshold (5 minutes)
    return timeUntilExpiry <= this.config.refreshThreshold;
  }

  isRefreshTokenExpired() {
    if (!this.tokens.absoluteExpiry) return false;
    return Date.now() > this.tokens.absoluteExpiry;
  }

  async refreshToken() {
    // Prevent concurrent refresh attempts
    if (this.state.isRefreshing) {
      return this.state.refreshPromise;
    }

    // Check if refresh token is expired
    if (this.isRefreshTokenExpired()) {
      const error = new Error('Refresh token has expired');
      this.emit('refresh-token-expired', error);
      throw error;
    }

    if (!this.tokens.refreshToken) {
      const error = new Error('No refresh token available');
      this.emit('refresh-failed', error);
      throw error;
    }

    this.state.isRefreshing = true;
    this.state.refreshPromise = this._performTokenRefresh();

    try {
      const result = await this.state.refreshPromise;
      this.state.refreshFailureCount = 0;
      this.state.lastRefreshTime = Date.now();

      if (import.meta.env.DEV) {
        console.log('Token refreshed successfully');
      }

      this.emit('token-refreshed', this.tokens);
      return result;
    } catch (error) {
      this.state.refreshFailureCount++;

      if (import.meta.env.DEV) {
        console.error('ERROR: Token refresh failed:', error);
      }

      this.emit('refresh-failed', error);

      // If refresh fails multiple times, clear tokens
      if (this.state.refreshFailureCount >= this.config.maxRefreshRetries) {
        await this.clearTokens();
        this.emit('refresh-token-expired', error);
      }

      throw error;
    } finally {
      this.state.isRefreshing = false;
      this.state.refreshPromise = null;
    }
  }

  async _performTokenRefresh() {
    try {
      // Use stored auth service reference or fallback to dynamic import
      let authService = this._authService;

      if (!authService) {
        // Fallback to dynamic import only if auth service not set
        const { default: importedAuthService } = await import('./auth.service');
        authService = importedAuthService;
      }

      const result = await authService.refreshToken(this.tokens.refreshToken);

      // Update tokens with the new ones
      await this.setTokens({
        accessToken: result.accessToken,
        refreshToken: result.refreshToken,
        expiresAt: Date.now() + (result.expiresIn * 1000),
        issuedAt: Date.now(),
        absoluteExpiry: result.absoluteExpiry || this.tokens.absoluteExpiry,
      });

      // Update stored auth state
      try {
        const storedAuth = localStorage.getItem('auth');
        if (storedAuth) {
          const authData = JSON.parse(storedAuth);
          authData.tokens = this.tokens;
          localStorage.setItem('auth', JSON.stringify(authData));
        }
      } catch (storageError) {
        if (import.meta.env.DEV) {
          console.warn('Failed to update stored auth data:', storageError);
        }
      }

      return this.tokens;
    } catch (error) {
      if (import.meta.env.DEV) {
        console.error('Token refresh API call failed:', error);
      }
      throw error;
    }
  }

  updateActivity() {
    this.state.lastActivityTime = Date.now();
  }

  startActivityMonitoring() {
    // Stub implementation
  }

  stopActivityMonitoring() {
    if (this.state.activityCheckTimer) {
      clearInterval(this.state.activityCheckTimer);
      this.state.activityCheckTimer = null;
    }
  }

  resetAuthState() {
    this.authState.lastAuthErrorTime = null;
    this.authState.isLoggingOut = false;
    this.authState.inactivityErrorSent = false;
  }

  async clearTokens() {
    this.tokens = {
      accessToken: null,
      refreshToken: null,
      expiresAt: null,
      issuedAt: null,
      absoluteExpiry: null,
    };

    // Clear from localStorage
    try {
      localStorage.removeItem('auth');
      localStorage.removeItem('auth_token');
      localStorage.removeItem('token');
      localStorage.removeItem('refresh_token');

      if (import.meta.env.DEV) {
        console.log('🧹 Tokens cleared from storage');
      }
    } catch (error) {
      if (import.meta.env.DEV) {
        console.warn('Failed to clear tokens from storage:', error);
      }
    }

    // Reset state
    this.resetAuthState();
    this.state.refreshFailureCount = 0;
    this.state.lastRefreshTime = null;

    // Emit event
    this.emit('tokens-cleared');
  }

  destroy() {
    this.stopActivityMonitoring();
    this.clearTokens();
    this.eventListeners.clear();
  }

  on(event, callback) {
    if (!this.eventListeners.has(event)) {
      this.eventListeners.set(event, new Set());
    }
    this.eventListeners.get(event).add(callback);
    return () => this.off(event, callback);
  }

  off(event, callback) {
    if (this.eventListeners.has(event)) {
      this.eventListeners.get(event).delete(callback);
    }
  }

  emit(event, data) {
    if (this.eventListeners.has(event)) {
      this.eventListeners.get(event).forEach(callback => {
        try {
          callback(data);
        } catch (error) {
          if (import.meta.env.DEV) {
            console.error(`Error in token manager event listener for ${event}:`, error);
          }
        }
      });
    }
  }

  getStatus() {
    const now = Date.now();
    return {
      hasToken: !!this.tokens.accessToken,
      isExpired: this.isTokenExpired(),
      shouldRefresh: this.shouldRefreshToken(),
      isRefreshing: this.state.isRefreshing,
      expiresIn: this.tokens.expiresAt ? Math.max(0, this.tokens.expiresAt - now) : 0,
      refreshTokenExpired: this.isRefreshTokenExpired(),
      refreshFailures: this.state.refreshFailureCount,
      lastRefresh: this.state.lastRefreshTime,
      lastActivity: this.state.lastActivityTime,
    };
  }
}

// Create singleton instance
const tokenManager = new TokenManager();

// Export for use in other modules
export default tokenManager;

// Also export class for testing
export { TokenManager }; 