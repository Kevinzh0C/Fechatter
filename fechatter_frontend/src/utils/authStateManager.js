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

    // 🔧 ENHANCED: More robust authentication check
    // Check multiple conditions for a more reliable auth state
    const hasValidToken = !!token && this.isValidTokenFormat(token);
    const hasValidUser = !!user && typeof user === 'object' && user.id;

    // 🔧 CRITICAL FIX: Enhanced authentication logic - ensure boolean return
    // User must have BOTH valid token AND valid user to be authenticated
    const isAuthenticated = Boolean(hasValidToken && hasValidUser);

    // 🔧 DEBUG: Development logging for troubleshooting
    if (import.meta.env.DEV && token) {
      console.log('🔐 [AuthStateManager] Auth state calculation:', {
        hasToken: !!token,
        hasValidToken,
        hasUser: !!user,
        hasValidUser,
        isAuthenticated,
        tokenPreview: token ? `${token.substring(0, 10)}...` : null,
        userId: user?.id || null
      });
    }

    return {
      token,
      user,
      isAuthenticated,
      hasToken: !!token,
      hasUser: !!user,
      // Derived state - all based on comprehensive checks
      isValid: isAuthenticated,
      needsRefresh: hasValidToken && !hasValidUser,
      // 🔧 NEW: Additional diagnostic information
      tokenValid: hasValidToken,
      userValid: hasValidUser,
      lastChecked: Date.now()
    };
  }

  /**
   * Set auth state - Atomic operation
   * This is the primary method for persisting authentication data.
   */
  setAuthState(token, user) {
    // Always clear previous state for consistency
    this.clearAuthState();

    if (token) {
      localStorage.setItem(this.STORAGE_KEY, token);
    }
    if (user) {
      localStorage.setItem(this.USER_KEY, JSON.stringify(user));
    }
  }

  /**
   * Update auth state properties
   * Used by auth store to update state without full replacement.
   */
  updateAuthState(updates) {
    // Handle user update separately
    if ('user' in updates) {
      if (updates.user) {
        localStorage.setItem(this.USER_KEY, JSON.stringify(updates.user));
      } else {
        localStorage.removeItem(this.USER_KEY);
      }
    }

    // Handle token update - this should ideally be part of a full setAuthState call
    if ('token' in updates) {
      if (updates.token) {
        localStorage.setItem(this.STORAGE_KEY, updates.token);
      } else {
        localStorage.removeItem(this.STORAGE_KEY);
      }
    }
  }

  /**
   * Clear auth state - Complete cleanup
   */
  clearAuthState() {
    const keysToRemove = [
      this.STORAGE_KEY,
      this.USER_KEY,
      'token', // Legacy
      'user',  // Legacy
      'redirectPath', // From router
      'fechatter_access_token', // Legacy
      'token_expires_at', // Legacy
      'fechatter_token_expiry', // Legacy
    ];

    keysToRemove.forEach(key => {
      localStorage.removeItem(key);
      sessionStorage.removeItem(key); // Clear from both storages
    });
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
   * Validate token format - 🔧 ENHANCED: More tolerant validation
   */
  isValidTokenFormat(token) {
    if (!token || typeof token !== 'string') return false;

    // 🔧 RELAXED: Accept any non-empty string token with reasonable length
    // Don't enforce strict JWT format as it may break with mock tokens or different auth systems
    if (token.length < 10) return false; // Too short to be meaningful
    if (token.length > 2000) return false; // Unreasonably long

    // 🔧 PRACTICAL: Basic token format validation
    // Accept JWT format (xxx.yyy.zzz) OR other reasonable token formats
    const hasBasicStructure = token.includes('.') || token.length >= 20;

    // 🔧 ENHANCED: Check for obviously invalid tokens
    const notObviouslyInvalid =
      !token.includes(' ') && // No spaces
      !token.includes('\n') && // No newlines
      !token.includes('\t') && // No tabs
      !/^[0-9]+$/.test(token); // Not just numbers

    const isValid = hasBasicStructure && notObviouslyInvalid;

    if (import.meta.env.DEV && !isValid) {
      console.warn('⚠️ [AuthStateManager] Token validation failed:', {
        tokenLength: token.length,
        hasBasicStructure,
        notObviouslyInvalid,
        tokenPreview: token.substring(0, 20) + '...'
      });
    }

    return isValid;
  }

  /**
   * Debug current state
   */
  debug() {
    const state = this.getAuthState();
    if (import.meta.env.DEV) {
      console.log('🔐 [AuthStateManager] Current State:', {
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
    }
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
      if (import.meta.env.DEV) {
        console.log('🔄 [AuthStateManager] Migrating legacy token');
      }
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