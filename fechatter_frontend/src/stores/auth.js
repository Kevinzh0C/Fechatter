/**
 * Auth Store
 * 
 * Manages authentication state with single source of truth
 */

import { defineStore } from 'pinia';
import authService from '@/services/auth.service';
import tokenManager from '@/services/tokenManager';
import authStateManager from '@/utils/authStateManager';
import { useUserStore } from './user';
import { useWorkspaceStore } from './workspace';
import { errorHandler } from '@/utils/errorHandler';

export const useAuthStore = defineStore('auth', {
  state: () => ({
    isLoading: false,
    error: null,
    lastLoginTime: null,
    sessionStartTime: null,
    isInitialized: false,
  }),

  getters: {
    /**
     * Get current user - from authStateManager
     */
    currentUser: () => authStateManager.getAuthState().user,
    user: () => authStateManager.getAuthState().user,

    /**
     * Check if user is authenticated - SINGLE SOURCE OF TRUTH
     */
    isAuthenticated: () => authStateManager.getAuthState().isAuthenticated,
    isLoggedIn: () => authStateManager.getAuthState().isAuthenticated,

    /**
     * Get user ID
     */
    userId: () => authStateManager.getAuthState().user?.id,

    /**
     * Get user email
     */
    userEmail: () => authStateManager.getAuthState().user?.email,

    /**
     * Get user full name
     */
    userFullName: () => {
      const user = authStateManager.getAuthState().user;
      return user?.fullname || user?.username || 'User';
    },

    /**
     * Check if user has admin role
     */
    isAdmin: () => {
      const user = authStateManager.getAuthState().user;
      return user?.role === 'admin' || user?.is_admin === true;
    },

    /**
     * Get session duration
     */
    sessionDuration: (state) => {
      if (!state.sessionStartTime) return 0;
      return Date.now() - state.sessionStartTime;
    },

    /**
     * Get access token (compatibility)
     * IMPORTANT: Get from tokenManager first (memory) then fallback to authStateManager (localStorage)
     */
    token: () => {
      // First try tokenManager (in-memory, most reliable)
      const tokenFromManager = tokenManager.getAccessToken();
      if (tokenFromManager) {
        return tokenFromManager;
      }

      // Fallback to authStateManager (localStorage)
      return authStateManager.getAuthState().token;
    },

    /**
     * Check if token is expired (compatibility)
     */
    isTokenExpired: () => tokenManager.isTokenExpired(),
  },

  actions: {
    /**
     * Initialize auth state from storage
     */
    async initialize() {
      // Prevent multiple initializations
      if (this.isInitialized) {
        return authStateManager.getAuthState().isAuthenticated;
      }

      // First ensure auth state consistency
      await this.ensureAuthStateConsistency();

      try {
        // Get current auth state from single source of truth
        const authState = authStateManager.getAuthState();

        // Debug log
        console.log('🔐 [AUTH] Initializing with state:', {
          hasToken: authState.hasToken,
          hasUser: authState.hasUser,
          isAuthenticated: authState.isAuthenticated
        });

        // No token = not authenticated
        if (!authState.token) {
          this.isInitialized = true;
          return false;
        }

        // Validate token format
        if (!authStateManager.isValidTokenFormat(authState.token)) {
          console.warn('🔐 [AUTH] Invalid token format, clearing...');
          authStateManager.clearAuthState();
          this.isInitialized = true;
          return false;
        }

        // Set token in tokenManager for API calls
        tokenManager.setTokens({
          accessToken: authState.token,
          refreshToken: authState.token, // Temporary, will be replaced by refresh
          expiresAt: Date.now() + (3600 * 1000), // Default 1 hour
          issuedAt: Date.now(),
        });

        // Check if token is expired
        if (tokenManager.isTokenExpired()) {
          console.warn('🔐 [AUTH] Token is expired, attempting refresh...');

          try {
            await tokenManager.refreshToken();
            console.log('✅ [AUTH] Token refreshed successfully');
          } catch (error) {
            console.error('❌ [AUTH] Token refresh failed:', error);
            authStateManager.clearAuthState();
            this.isInitialized = true;
            return false;
          }
        }

        // Set timestamps
        this.lastLoginTime = Date.now();
        this.sessionStartTime = Date.now();

        // Setup listeners
        this.setupTokenManagerListeners();

        // Fetch fresh user data if needed
        if (!authState.user) {
          try {
            await this.fetchCurrentUser();
          } catch (error) {
            console.warn('Failed to fetch user data:', error);
            // Don't fail init if user fetch fails
          }
        }

        // Mark as initialized
        this.isInitialized = true;

        return true;
      } catch (error) {
        console.error('❌ [AUTH] Failed to initialize auth:', error);
        authStateManager.clearAuthState();
        this.isInitialized = true;
        return false;
      }
    },

    /**
     * Setup token manager event listeners
     */
    setupTokenManagerListeners() {
      // Token refreshed
      tokenManager.on('token-refreshed', (tokens) => {
        this.updateStoredTokens(tokens);
      });

      // Refresh failed
      tokenManager.on('refresh-failed', (error) => {
        console.error('Token refresh failed:', error);
        this.handleAuthError('Session expired. Please login again.');
      });

      // Refresh token expired
      tokenManager.on('refresh-token-expired', () => {
        this.handleAuthError('Your session has expired. Please login again.');
      });

      // Inactivity timeout
      tokenManager.on('inactivity-timeout', () => {
        this.logout('You have been logged out due to inactivity.');
      });
    },

    /**
     * Login user
     */
    async login(email, password) {
      this.isLoading = true;
      this.error = null;

      try {
        // Call auth service
        const result = await authService.login(email, password);

        // Debug log the login result
        console.log('🔐 [AUTH] Login result:', {
          hasAccessToken: !!result.accessToken,
          hasRefreshToken: !!result.refreshToken,
          hasUser: !!result.user,
          expiresIn: result.expiresIn
        });

        // Validate tokens before doing anything
        const now = Date.now();
        const tokenData = {
          accessToken: result.accessToken,
          refreshToken: result.refreshToken,
          expiresAt: now + (result.expiresIn * 1000),
          issuedAt: now,
        };

        if (!tokenData.accessToken || !tokenData.refreshToken) {
          throw new Error('Invalid token data received from login');
        }

        // Set tokens FIRST
        tokenManager.setTokens(tokenData);

        // Verify token was set correctly
        const verifyToken = tokenManager.getAccessToken();
        if (!verifyToken) {
          console.error('❌ [AUTH] Token not set correctly in tokenManager');
          throw new Error('Failed to store authentication token');
        }

        console.log('✅ [AUTH] Token verified in tokenManager:', verifyToken.substring(0, 20) + '...');

        // ONLY NOW set auth state via authStateManager
        // This ensures we never have isAuthenticated=true without a valid token
        authStateManager.setAuthState(result.accessToken, result.user);

        // Set timestamps
        this.lastLoginTime = Date.now();
        this.sessionStartTime = Date.now();

        // Setup listeners after everything is confirmed
        this.setupTokenManagerListeners();

        // Initialize other stores
        await this.initializeUserStores();

        // Track activity
        tokenManager.updateActivity();

        // Mark as initialized
        this.isInitialized = true;

        return true;
      } catch (error) {
        // Clear any partial state on error
        authStateManager.clearAuthState();
        this.error = error.message || 'Login failed';

        errorHandler.handle(error, {
          context: 'Login',
          silent: false,
        });
        throw error;
      } finally {
        this.isLoading = false;
      }
    },

    /**
     * Register new user
     */
    async register(userData) {
      this.isLoading = true;
      this.error = null;

      try {
        // Validate passwords match
        if (userData.password !== userData.confirmPassword) {
          throw new Error('Passwords do not match');
        }

        // Call auth service
        const result = await authService.register({
          fullname: userData.fullname,
          email: userData.email,
          password: userData.password,
          confirm_password: userData.password,
        });

        // Validate and setup tokens FIRST
        const now = Date.now();
        const tokenData = {
          accessToken: result.accessToken,
          refreshToken: result.refreshToken,
          expiresAt: now + (result.expiresIn * 1000),
          issuedAt: now,
        };

        if (!tokenData.accessToken || !tokenData.refreshToken) {
          throw new Error('Invalid token data received from registration');
        }

        tokenManager.setTokens(tokenData);

        // Verify token was set correctly
        const verifyToken = tokenManager.getAccessToken();
        if (!verifyToken) {
          console.error('❌ [AUTH] Token not set correctly in tokenManager after registration');
          throw new Error('Failed to store authentication token');
        }

        console.log('✅ [AUTH] Registration token verified:', verifyToken.substring(0, 20) + '...');

        // ONLY NOW set auth state via authStateManager
        authStateManager.setAuthState(result.accessToken, result.user);

        // Set timestamps
        this.lastLoginTime = Date.now();
        this.sessionStartTime = Date.now();

        // Setup listeners
        this.setupTokenManagerListeners();

        // Initialize other stores
        await this.initializeUserStores();

        return true;
      } catch (error) {
        // Clear any partial state on error
        authStateManager.clearAuthState();
        this.error = error.message || 'Registration failed';

        errorHandler.handle(error, {
          context: 'Register',
          silent: false,
        });
        throw error;
      } finally {
        this.isLoading = false;
      }
    },

    /**
     * Logout user
     */
    async logout(message = null) {
      try {
        // Get refresh token for logout API
        const tokens = tokenManager.getTokens();

        // Call logout API
        if (tokens.refreshToken) {
          await authService.logout(tokens.refreshToken);
        }
      } catch (error) {
        console.error('Logout API error:', error);
      } finally {
        // Always clear local state
        this.clearAuth();

        // Show message if provided
        if (message) {
          errorHandler.showNotification(message, 'info');
        }

        // Redirect to login
        try {
          // Import router dynamically to avoid circular dependency
          const router = await import('@/router').then(m => m.default);
          const currentRoute = router.currentRoute.value;

          if (currentRoute.path !== '/login' && currentRoute.path !== '/register') {
            await router.push('/login');
          }
        } catch (routerError) {
          console.error('Router navigation error:', routerError);
          // Fallback to direct navigation
          if (window.location.pathname !== '/login') {
            window.location.href = '/login';
          }
        }
      }
    },

    /**
     * Fetch current user data
     */
    async fetchCurrentUser() {
      try {
        const user = await authService.getCurrentUser();
        authStateManager.updateAuthState({
          user: user,
        });
        this.persistAuth();
        return user;
      } catch (error) {
        console.error('Failed to fetch current user:', error);
        throw error;
      }
    },

    /**
     * Update user profile
     */
    async updateProfile(profileData) {
      try {
        const updatedUser = await authService.updateProfile(profileData);
        authStateManager.updateAuthState({
          user: { ...this.user, ...updatedUser },
        });
        this.persistAuth();
        return updatedUser;
      } catch (error) {
        errorHandler.handle(error, {
          context: 'Update profile',
          silent: false,
        });
        throw error;
      }
    },

    /**
     * Change password
     */
    async changePassword(currentPassword, newPassword) {
      try {
        await authService.changePassword(currentPassword, newPassword);
        errorHandler.showNotification('Password changed successfully', 'success');
        return true;
      } catch (error) {
        errorHandler.handle(error, {
          context: 'Change password',
          silent: false,
        });
        throw error;
      }
    },

    /**
     * Initialize user-related stores
     */
    async initializeUserStores() {
      try {
        const userStore = useUserStore();
        const workspaceStore = useWorkspaceStore();

        // Fetch workspace users - don't fail if this errors
        await userStore.fetchWorkspaceUsers().catch(err => {
          console.warn('Failed to fetch workspace users:', err);
        });

        // Fetch workspace data - always provide a default workspace
        try {
          await workspaceStore.fetchCurrentWorkspace();
        } catch (err) {
          console.warn('Failed to fetch workspace data, using defaults:', err);
          // Ensure we have a default workspace even if fetch fails
          workspaceStore.setCurrentWorkspaceId(this.user?.workspace_id || 1);
        }

        // Set current workspace ID if available
        const workspaceId = this.user?.workspace_id || 1;
        workspaceStore.setCurrentWorkspaceId(workspaceId);

        console.log('✅ [AUTH] User stores initialized with workspace:', workspaceId);
      } catch (error) {
        console.error('Failed to initialize user stores:', error);
        // Don't throw - initialization should continue even if some stores fail
        // Ensure we have minimal workspace setup
        try {
          const workspaceStore = useWorkspaceStore();
          workspaceStore.setCurrentWorkspaceId(1);
        } catch (e) {
          console.error('Failed to set default workspace:', e);
        }
      }
    },

    /**
     * Persist auth data to localStorage
     */
    persistAuth() {
      if (!this.isAuthenticated || !this.user) {
        return;
      }

      const tokens = tokenManager.getTokens();
      const authData = {
        user: this.user,
        tokens: tokens,
        lastLoginTime: this.lastLoginTime,
        timestamp: Date.now(),
      };

      localStorage.setItem('auth', JSON.stringify(authData));
    },

    /**
     * Update stored tokens
     */
    updateStoredTokens(tokens) {
      const authData = localStorage.getItem('auth');
      if (authData) {
        const parsed = JSON.parse(authData);
        parsed.tokens = tokens;
        parsed.timestamp = Date.now();
        localStorage.setItem('auth', JSON.stringify(parsed));
      }
    },

    /**
     * Clear auth state
     */
    clearAuth() {
      console.log('🧹 [AUTH] Clearing auth state...');

      // Clear state via authStateManager
      authStateManager.clearAuthState();

      // Clear local state
      this.$patch({
        user: null,
        isAuthenticated: false,
        lastLoginTime: null,
        sessionStartTime: null,
        error: null,
        isInitialized: false
      });

      // Clear token manager
      tokenManager.clearTokens();

      // Clear all auth-related localStorage items
      const keysToRemove = ['auth', 'auth_token', 'fechatter_access_token', 'token_expires_at', 'fechatter_token_expiry', 'remember_me'];
      keysToRemove.forEach(key => {
        localStorage.removeItem(key);
        sessionStorage.removeItem(key);
      });

      // Clear other stores
      try {
        const { useUserStore } = require('@/stores/user');
        const { useWorkspaceStore } = require('@/stores/workspace');
        const { useChatStore } = require('@/stores/chat');

        const userStore = useUserStore();
        const workspaceStore = useWorkspaceStore();
        const chatStore = useChatStore();

        userStore.$reset();
        workspaceStore.$reset();
        chatStore.$reset();
      } catch (error) {
        console.error('Error clearing stores:', error);
      }

      console.log('✅ [AUTH] Auth state cleared');
    },

    /**
     * Handle auth errors
     */
    handleAuthError(message) {
      this.error = message;
      this.clearAuth();

      // Redirect to login
      if (window.$router && window.$router.currentRoute.value.path !== '/login') {
        window.$router.push({
          path: '/login',
          query: { redirect: window.$router.currentRoute.value.fullPath }
        });
      }
    },

    /**
     * Get access token (for API interceptor)
     */
    getAccessToken() {
      return tokenManager.getAccessToken();
    },

    /**
     * Check and refresh token if needed
     */
    async checkAndRefreshToken() {
      if (tokenManager.shouldRefreshToken()) {
        try {
          await tokenManager.refreshToken();
          return true;
        } catch (error) {
          console.error('Token refresh failed:', error);
          return false;
        }
      }
      return true;
    },

    /**
     * Validate and synchronize auth state
     */
    async validateAndSyncAuthState() {
      try {
        // Check if we have inconsistent state
        const hasAuthState = this.isAuthenticated && this.user;
        const hasValidToken = tokenManager.getAccessToken() && !tokenManager.isTokenExpired();

        console.log('🔧 [AUTH] Validating auth state:', {
          hasAuthState,
          hasValidToken,
          isAuthenticated: this.isAuthenticated,
          hasUser: !!this.user,
          hasToken: !!tokenManager.getAccessToken(),
          isTokenExpired: tokenManager.isTokenExpired()
        });

        if (hasAuthState && !hasValidToken) {
          console.warn('🔧 [AUTH] Detected auth state inconsistency, attempting to fix...');

          // Try to restore from localStorage
          const storedAuth = localStorage.getItem('auth');
          if (storedAuth) {
            try {
              const authData = JSON.parse(storedAuth);

              console.log('🔧 [AUTH] Found stored auth data:', {
                hasUser: !!authData.user,
                hasTokens: !!authData.tokens,
                hasAccessToken: !!authData.tokens?.accessToken,
                tokenLength: authData.tokens?.accessToken?.length || 0
              });

              if (authData.tokens && authData.tokens.accessToken) {
                // Reinitialize token manager
                tokenManager.setTokens({
                  accessToken: authData.tokens.accessToken,
                  refreshToken: authData.tokens.refreshToken,
                  expiresAt: authData.tokens.expiresAt,
                  issuedAt: authData.tokens.issuedAt,
                  absoluteExpiry: authData.tokens.absoluteExpiry,
                });

                // Verify token was restored
                const restoredToken = tokenManager.getAccessToken();
                if (!restoredToken) {
                  console.error('❌ [AUTH] Failed to restore token to tokenManager');
                  this.clearAuth();
                  return false;
                }

                console.log('✅ [AUTH] Token restored:', restoredToken.substring(0, 20) + '...');

                // Check if token needs refresh
                if (tokenManager.isTokenExpired()) {
                  console.log('🔄 [AUTH] Restored token is expired, attempting refresh...');
                  await tokenManager.refreshToken();
                }

                console.log('✅ [AUTH] Auth state synchronized successfully');
                return true;
              } else {
                console.error('❌ [AUTH] No valid tokens found in stored auth data');
              }
            } catch (error) {
              console.error('❌ [AUTH] Failed to restore auth state:', error);
            }
          } else {
            console.error('❌ [AUTH] No stored auth data found');
          }

          // If restoration failed, clear inconsistent state
          console.log('🧹 [AUTH] Clearing inconsistent auth state');
          this.clearAuth();
          return false;
        }

        return hasValidToken;
      } catch (error) {
        console.error('❌ [AUTH] Error during auth state validation:', error);
        this.clearAuth();
        return false;
      }
    },

    /**
     * Update user activity
     */
    updateActivity() {
      tokenManager.updateActivity();
    },

    /**
     * Ensure auth state consistency between tokenManager and authStateManager
     */
    async ensureAuthStateConsistency() {
      // Get token from tokenManager (memory)
      const tokenFromManager = tokenManager.getAccessToken();

      // Get auth state from authStateManager (localStorage)
      const authState = authStateManager.getAuthState();

      console.log('🔧 [AUTH] Checking auth state consistency:', {
        hasTokenInManager: !!tokenFromManager,
        hasTokenInStorage: !!authState.token,
        tokensMatch: tokenFromManager === authState.token
      });

      // If tokenManager has token but authStateManager doesn't, sync it
      if (tokenFromManager && !authState.token) {
        console.log('🔧 [AUTH] Syncing token from tokenManager to authStateManager');
        authStateManager.setAuthState(tokenFromManager, authState.user);
      }

      // If authStateManager has token but tokenManager doesn't, restore it
      else if (!tokenFromManager && authState.token) {
        console.log('🔧 [AUTH] Restoring token from authStateManager to tokenManager');

        // Try to restore token data from localStorage
        const storedAuth = localStorage.getItem('auth');
        if (storedAuth) {
          try {
            const authData = JSON.parse(storedAuth);
            if (authData.tokens) {
              tokenManager.setTokens(authData.tokens);
            }
          } catch (error) {
            console.error('Failed to restore token data:', error);
          }
        }
      }

      // If tokens don't match, prefer tokenManager's version
      else if (tokenFromManager && authState.token && tokenFromManager !== authState.token) {
        console.warn('🔧 [AUTH] Token mismatch detected, using tokenManager version');
        authStateManager.setAuthState(tokenFromManager, authState.user);
      }
    },
  },
}); 