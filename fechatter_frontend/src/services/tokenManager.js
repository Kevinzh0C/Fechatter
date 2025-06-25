/**
 * Token Management Service
 * 
 * Provides sliding window token management with:
 * - Automatic token refresh
 * - Concurrent refresh prevention
 * - Activity-based refresh
 * - Token lifecycle tracking
 * - Event-driven architecture
 */

import { errorHandler } from '@/utils/errorHandler';

class TokenManager {
  constructor() {
    // Token storage
    this.tokens = {
      accessToken: null,
      refreshToken: null,
      expiresAt: null,
      issuedAt: null,
      absoluteExpiry: null, // Refresh token absolute expiry
    };

    // Configuration
    this.config = {
      refreshThreshold: 5 * 60 * 1000, // 5 minutes before expiry
      refreshPercentage: 0.5, // Refresh when 50% of lifetime passed
      clockSkewBuffer: 30 * 1000, // 30 seconds clock skew buffer
      maxRefreshRetries: 3,
      refreshBackoffMs: 1000, // Initial backoff
      maxBackoffMs: 30000, // Max 30 seconds
      inactivityTimeout: 30 * 60 * 1000, // 30 minutes
      activityCheckInterval: 60 * 1000, // Check every minute
    };

    // State management
    this.state = {
      isRefreshing: false,
      refreshPromise: null,
      refreshFailureCount: 0,
      lastRefreshTime: null,
      lastActivityTime: Date.now(),
      activityCheckTimer: null,
      refreshTimer: null,
    };

    // Event listeners
    this.eventListeners = new Map();

    // Bind methods
    this.handleTokenRefreshed = this.handleTokenRefreshed.bind(this);
    this.handleActivityUpdate = this.handleActivityUpdate.bind(this);
    this.checkActivity = this.checkActivity.bind(this);

    // Initialize
    this.initialize();
  }

  /**
   * Initialize the token manager
   */
  initialize() {
    // Listen for global events
    if (typeof window !== 'undefined') {
      window.addEventListener('auth:token-refreshed', this.handleTokenRefreshed);
      window.addEventListener('user:activity', this.handleActivityUpdate);

      // Start activity monitoring
      this.startActivityMonitoring();
    }

    console.log('🔐 Token Manager initialized');
  }

  /**
   * Set tokens (called by auth store)
   */
  setTokens(tokenData) {
    const now = Date.now();

    this.tokens = {
      accessToken: tokenData.accessToken,
      refreshToken: tokenData.refreshToken,
      expiresAt: tokenData.expiresAt || (now + (tokenData.expiresIn * 1000)),
      issuedAt: tokenData.issuedAt || now,
      absoluteExpiry: tokenData.absoluteExpiry || (now + (30 * 24 * 60 * 60 * 1000)), // 30 days
    };

    // Reset refresh state
    this.state.refreshFailureCount = 0;
    this.state.lastRefreshTime = now;

    // Schedule next refresh
    this.scheduleTokenRefresh();

    // Emit token updated event
    this.emit('tokens-updated', this.tokens);

    console.log('🔐 Tokens updated:', {
      expiresIn: Math.round((this.tokens.expiresAt - now) / 1000) + 's',
      lifetime: Math.round((this.tokens.expiresAt - this.tokens.issuedAt) / 1000) + 's',
    });
  }

  /**
   * Get current access token
   */
  getAccessToken() {
    return this.tokens.accessToken;
  }

  /**
   * Get all tokens
   */
  getTokens() {
    return { ...this.tokens };
  }

  /**
   * Check if token is expired
   */
  isTokenExpired() {
    if (!this.tokens.expiresAt) return true;

    // Add clock skew buffer
    return Date.now() + this.config.clockSkewBuffer > this.tokens.expiresAt;
  }

  /**
   * Check if token should be refreshed
   */
  shouldRefreshToken() {
    if (!this.tokens.expiresAt || this.state.isRefreshing) return false;

    const now = Date.now();
    const timeUntilExpiry = this.tokens.expiresAt - now;

    // Check if within threshold
    if (timeUntilExpiry < this.config.refreshThreshold) {
      return true;
    }

    // Check if passed percentage of lifetime
    const lifetime = this.tokens.expiresAt - this.tokens.issuedAt;
    const timeSinceIssued = now - this.tokens.issuedAt;
    const percentageUsed = timeSinceIssued / lifetime;

    return percentageUsed > this.config.refreshPercentage;
  }

  /**
   * Check if refresh token is expired
   */
  isRefreshTokenExpired() {
    if (!this.tokens.absoluteExpiry) return false;
    return Date.now() > this.tokens.absoluteExpiry;
  }

  /**
   * Refresh token with sliding window mechanism
   */
  async refreshToken() {
    // Check if already refreshing
    if (this.state.isRefreshing && this.state.refreshPromise) {
      console.log('🔐 Token refresh already in progress, waiting...');
      return this.state.refreshPromise;
    }

    // Check if refresh token expired
    if (this.isRefreshTokenExpired()) {
      const error = new Error('Refresh token expired');
      this.emit('refresh-token-expired', error);
      throw error;
    }

    // Check retry limit
    if (this.state.refreshFailureCount >= this.config.maxRefreshRetries) {
      const error = new Error('Max refresh retries exceeded');
      this.emit('refresh-failed', error);
      throw error;
    }

    try {
      this.state.isRefreshing = true;
      this.state.refreshPromise = this._performRefresh();

      const result = await this.state.refreshPromise;

      // Reset failure count on success
      this.state.refreshFailureCount = 0;

      return result;
    } catch (error) {
      // Increment failure count
      this.state.refreshFailureCount++;

      // Calculate backoff
      const backoff = Math.min(
        this.config.refreshBackoffMs * Math.pow(2, this.state.refreshFailureCount - 1),
        this.config.maxBackoffMs
      );

      console.error(`❌ Token refresh failed (attempt ${this.state.refreshFailureCount}), next retry in ${backoff}ms`);

      // Schedule retry with backoff
      if (this.state.refreshFailureCount < this.config.maxRefreshRetries) {
        setTimeout(() => {
          if (this.shouldRefreshToken()) {
            this.refreshToken().catch(console.error);
          }
        }, backoff);
      }

      throw error;
    } finally {
      this.state.isRefreshing = false;
      this.state.refreshPromise = null;
    }
  }

  /**
   * Internal refresh implementation
   */
  async _performRefresh() {
    if (!this.tokens.refreshToken) {
      throw new Error('No refresh token available');
    }

    console.log('🔐 Performing token refresh...');

    // Import auth service dynamically to avoid circular dependency
    const { default: authService } = await import('@/services/auth.service');

    const result = await authService.refreshToken(this.tokens.refreshToken);

    // Update tokens
    const now = Date.now();
    this.tokens.accessToken = result.accessToken;
    this.tokens.issuedAt = now;
    this.tokens.expiresAt = now + (result.expiresIn * 1000);

    // Update refresh token if rotated
    if (result.refreshToken) {
      this.tokens.refreshToken = result.refreshToken;
      // Assume 30 day lifetime if not provided
      this.tokens.absoluteExpiry = result.absoluteExpiry || (now + (30 * 24 * 60 * 60 * 1000));
    }

    this.state.lastRefreshTime = now;

    // Schedule next refresh
    this.scheduleTokenRefresh();

    // Emit events
    this.emit('token-refreshed', this.tokens);

    // Dispatch global event for other services
    window.dispatchEvent(new CustomEvent('auth:token-refreshed', {
      detail: { token: this.tokens.accessToken }
    }));

    console.log('✅ Token refreshed successfully');

    return true;
  }

  /**
   * Schedule token refresh
   */
  scheduleTokenRefresh() {
    // Clear existing timer
    if (this.state.refreshTimer) {
      clearTimeout(this.state.refreshTimer);
      this.state.refreshTimer = null;
    }

    if (!this.tokens.expiresAt) return;

    const now = Date.now();
    const lifetime = this.tokens.expiresAt - this.tokens.issuedAt;
    const timeUntilExpiry = this.tokens.expiresAt - now;

    // Calculate optimal refresh time
    const refreshAtPercentage = lifetime * this.config.refreshPercentage;
    const refreshAtThreshold = timeUntilExpiry - this.config.refreshThreshold;
    const refreshIn = Math.min(refreshAtPercentage, refreshAtThreshold);

    if (refreshIn > 0) {
      console.log(`⏰ Next token refresh scheduled in ${Math.round(refreshIn / 1000)}s`);

      this.state.refreshTimer = setTimeout(() => {
        if (this.shouldRefreshToken()) {
          this.refreshToken().catch(error => {
            console.error('❌ Scheduled token refresh failed:', error);
            this.emit('refresh-failed', error);
          });
        }
      }, refreshIn);
    }
  }

  /**
   * Update user activity
   */
  updateActivity() {
    this.state.lastActivityTime = Date.now();

    // Check if token needs refresh on activity
    if (this.shouldRefreshToken() && !this.state.isRefreshing) {
      this.refreshToken().catch(error => {
        console.error('❌ Activity-triggered token refresh failed:', error);
      });
    }
  }

  /**
   * Start activity monitoring
   */
  startActivityMonitoring() {
    // Clear existing timer
    this.stopActivityMonitoring();

    // Check activity periodically
    this.state.activityCheckTimer = setInterval(this.checkActivity, this.config.activityCheckInterval);
  }

  /**
   * Stop activity monitoring
   */
  stopActivityMonitoring() {
    if (this.state.activityCheckTimer) {
      clearInterval(this.state.activityCheckTimer);
      this.state.activityCheckTimer = null;
    }
  }

  /**
   * Check user activity
   */
  checkActivity() {
    const now = Date.now();
    const timeSinceLastActivity = now - this.state.lastActivityTime;

    // Check for inactivity timeout
    if (timeSinceLastActivity > this.config.inactivityTimeout) {
      console.warn('⏰ User inactive for too long');
      this.emit('inactivity-timeout');
      return;
    }

    // Check if token needs refresh
    if (this.shouldRefreshToken() && !this.state.isRefreshing) {
      this.refreshToken().catch(error => {
        console.error('❌ Periodic token refresh failed:', error);
      });
    }
  }

  /**
   * Handle external token refresh
   */
  handleTokenRefreshed(event) {
    if (event.detail && event.detail.token) {
      // Update our token if it's different
      if (event.detail.token !== this.tokens.accessToken) {
        console.log('🔐 External token refresh detected, updating...');
        // Note: We should get full token data, not just access token
        // This is a simplified handler
      }
    }
  }

  /**
   * Handle activity update
   */
  handleActivityUpdate() {
    this.updateActivity();
  }

  /**
   * Clear tokens
   */
  clearTokens() {
    this.tokens = {
      accessToken: null,
      refreshToken: null,
      expiresAt: null,
      issuedAt: null,
      absoluteExpiry: null,
    };

    // Clear timers
    if (this.state.refreshTimer) {
      clearTimeout(this.state.refreshTimer);
      this.state.refreshTimer = null;
    }

    // Reset state
    this.state.isRefreshing = false;
    this.state.refreshPromise = null;
    this.state.refreshFailureCount = 0;

    this.emit('tokens-cleared');
  }

  /**
   * Destroy the token manager
   */
  destroy() {
    // Remove event listeners
    if (typeof window !== 'undefined') {
      window.removeEventListener('auth:token-refreshed', this.handleTokenRefreshed);
      window.removeEventListener('user:activity', this.handleActivityUpdate);
    }

    // Stop monitoring
    this.stopActivityMonitoring();

    // Clear tokens
    this.clearTokens();

    // Clear listeners
    this.eventListeners.clear();

    console.log('🔐 Token Manager destroyed');
  }

  /**
   * Event emitter methods
   */
  on(event, callback) {
    if (!this.eventListeners.has(event)) {
      this.eventListeners.set(event, new Set());
    }
    this.eventListeners.get(event).add(callback);

    // Return unsubscribe function
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
          console.error(`Error in token manager event listener for ${event}:`, error);
        }
      });
    }
  }

  /**
   * Get token status
   */
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