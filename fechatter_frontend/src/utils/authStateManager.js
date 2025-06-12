/**
 * Auth State Manager - Single Source of Truth
 * Following Frontend Design Principles:
 * - Component State Layer: Single source of truth for auth state
 * - YAGNI: Simple, direct state management
 * - Occam's Razor: Simplest solution that works
 */

class AuthStateManager {
  constructor() {
    this.STORAGE_KEY = 'auth_token';
    this.USER_KEY = 'auth_user';
  }

  /**
   * Get current auth state - SINGLE SOURCE OF TRUTH
   * Token existence determines authentication state
   */
  getAuthState() {
    const token = this.getToken();
    const user = this.getUser();

    // Simple rule: Has token = Authenticated
    const isAuthenticated = !!token;

    return {
      token,
      user,
      isAuthenticated,
      hasToken: !!token,
      hasUser: !!user,
      // Derived state - all based on token existence
      isValid: isAuthenticated && !!user,
      needsRefresh: isAuthenticated && !user
    };
  }

  /**
   * Set auth state - Atomic operation
   */
  setAuthState(token, user) {
    if (token) {
      localStorage.setItem(this.STORAGE_KEY, token);
      if (user) {
        localStorage.setItem(this.USER_KEY, JSON.stringify(user));
      }
    } else {
      // If no token, clear everything
      this.clearAuthState();
    }
  }

  /**
   * Update auth state properties
   * Used by auth store to update state
   */
  updateAuthState(updates) {
    const currentState = this.getAuthState();

    // Handle token update
    if ('token' in updates) {
      this.setAuthState(updates.token, updates.user || currentState.user);
    }

    // Handle user update
    if ('user' in updates && updates.user) {
      localStorage.setItem(this.USER_KEY, JSON.stringify(updates.user));
    }

    // Handle clearing
    if (updates.isAuthenticated === false || updates.user === null) {
      this.clearAuthState();
    }
  }

  /**
   * Clear auth state - Complete cleanup
   */
  clearAuthState() {
    localStorage.removeItem(this.STORAGE_KEY);
    localStorage.removeItem(this.USER_KEY);
    // Also clear any legacy keys
    localStorage.removeItem('token');
    localStorage.removeItem('user');
    sessionStorage.removeItem('auth_token');
    sessionStorage.removeItem('token');
  }

  /**
   * Get token from storage
   */
  getToken() {
    // Check multiple locations for backward compatibility
    return localStorage.getItem(this.STORAGE_KEY) ||
      localStorage.getItem('token') ||
      sessionStorage.getItem('auth_token') ||
      sessionStorage.getItem('token') ||
      null;
  }

  /**
   * Get user from storage
   */
  getUser() {
    try {
      const userStr = localStorage.getItem(this.USER_KEY) ||
        localStorage.getItem('user');
      return userStr ? JSON.parse(userStr) : null;
    } catch {
      return null;
    }
  }

  /**
   * Validate token format
   */
  isValidTokenFormat(token) {
    if (!token || typeof token !== 'string') return false;

    // JWT format: xxx.yyy.zzz
    const parts = token.split('.');
    return parts.length === 3 && parts.every(part => part.length > 0);
  }

  /**
   * Debug current state
   */
  debug() {
    const state = this.getAuthState();
    console.log('🔐 [AuthStateManager] Current State:', {
      ...state,
      tokenPreview: state.token ? `${state.token.substring(0, 20)}...` : null,
      storageKeys: {
        localStorage: Object.keys(localStorage).filter(k =>
          k.includes('auth') || k.includes('token') || k.includes('user')
        ),
        sessionStorage: Object.keys(sessionStorage).filter(k =>
          k.includes('auth') || k.includes('token') || k.includes('user')
        )
      }
    });
    return state;
  }

  /**
   * Migrate legacy auth data
   */
  migrateLegacyAuth() {
    // Check for legacy token locations
    const legacyToken = localStorage.getItem('token') ||
      sessionStorage.getItem('token');

    if (legacyToken && !localStorage.getItem(this.STORAGE_KEY)) {
      console.log('🔄 [AuthStateManager] Migrating legacy token');
      localStorage.setItem(this.STORAGE_KEY, legacyToken);

      // Clean up legacy locations
      localStorage.removeItem('token');
      sessionStorage.removeItem('token');
      sessionStorage.removeItem('auth_token');
    }
  }
}

// Export singleton instance
export const authStateManager = new AuthStateManager();

// Auto-migrate on load
authStateManager.migrateLegacyAuth();

// Export for debugging
if (import.meta.env.DEV) {
  window.authStateManager = authStateManager;
}

export default authStateManager; 