/**
 * Auth Store
 * 
 * Manages authentication state with token manager integration
 */

import { defineStore } from 'pinia';
import authService from '@/services/auth.service';
import tokenManager from '@/services/tokenManager';
import { useUserStore } from './user';
import { useWorkspaceStore } from './workspace';
import { errorHandler } from '@/utils/errorHandler';

export const useAuthStore = defineStore('auth', {
  state: () => ({
    user: null,
    isAuthenticated: false,
    isLoading: false,
    error: null,
    lastLoginTime: null,
    sessionStartTime: null,
    isInitialized: false,
  }),

  getters: {
    /**
     * Get current user
     */
    currentUser: (state) => state.user,

    /**
     * Check if user is logged in
     */
    isLoggedIn: (state) => state.isAuthenticated,

    /**
     * Get user ID
     */
    userId: (state) => state.user?.id,

    /**
     * Get user email
     */
    userEmail: (state) => state.user?.email,

    /**
     * Get user full name
     */
    userFullName: (state) => state.user?.fullname || state.user?.username || 'User',

    /**
     * Check if user has admin role
     */
    isAdmin: (state) => state.user?.role === 'admin' || state.user?.is_admin === true,

    /**
     * Get session duration
     */
    sessionDuration: (state) => {
      if (!state.sessionStartTime) return 0;
      return Date.now() - state.sessionStartTime;
    },

    /**
     * Get access token (compatibility)
     */
    token: () => tokenManager.getAccessToken(),

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
        return this.isAuthenticated;
      }

      try {
        // Check localStorage for persisted auth
        const storedAuth = localStorage.getItem('auth');
        if (!storedAuth) {
          this.isInitialized = true;
          return false;
        }

        const authData = JSON.parse(storedAuth);

        // Validate stored data
        if (!authData.user || !authData.tokens) {
          this.clearAuth();
          this.isInitialized = true;
          return false;
        }

        // Set user
        this.user = authData.user;
        this.isAuthenticated = true;
        this.lastLoginTime = authData.lastLoginTime;
        this.sessionStartTime = Date.now();

        // Initialize token manager
        tokenManager.setTokens({
          accessToken: authData.tokens.accessToken,
          refreshToken: authData.tokens.refreshToken,
          expiresAt: authData.tokens.expiresAt,
          issuedAt: authData.tokens.issuedAt,
          absoluteExpiry: authData.tokens.absoluteExpiry,
        });

        // Listen to token manager events
        this.setupTokenManagerListeners();

        // Verify token is still valid
        if (tokenManager.isTokenExpired()) {
          console.log('🔐 Token expired, attempting refresh...');
          try {
            await tokenManager.refreshToken();
          } catch (error) {
            console.error('Token refresh failed:', error);
            this.clearAuth();
            this.isInitialized = true;
            return false;
          }
        }

        // Fetch fresh user data
        try {
          await this.fetchCurrentUser();
        } catch (error) {
          console.error('Failed to fetch current user:', error);
          // Don't fail initialization if user fetch fails
        }

        // Track activity
        tokenManager.updateActivity();

        // Mark as initialized
        this.isInitialized = true;

        return true;
      } catch (error) {
        console.error('Failed to initialize auth:', error);
        this.clearAuth();
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

        // Set user data
        this.user = result.user;
        this.isAuthenticated = true;
        this.lastLoginTime = Date.now();
        this.sessionStartTime = Date.now();

        // Setup token manager
        const now = Date.now();
        tokenManager.setTokens({
          accessToken: result.accessToken,
          refreshToken: result.refreshToken,
          expiresAt: now + (result.expiresIn * 1000),
          issuedAt: now,
        });

        // Setup listeners
        this.setupTokenManagerListeners();

        // Store auth data
        this.persistAuth();

        // Initialize other stores
        await this.initializeUserStores();

        // Track activity
        tokenManager.updateActivity();

        // Mark as initialized
        this.isInitialized = true;

        return true;
      } catch (error) {
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

        // Set user data
        this.user = result.user;
        this.isAuthenticated = true;
        this.lastLoginTime = Date.now();
        this.sessionStartTime = Date.now();

        // Setup token manager
        const now = Date.now();
        tokenManager.setTokens({
          accessToken: result.accessToken,
          refreshToken: result.refreshToken,
          expiresAt: now + (result.expiresIn * 1000),
          issuedAt: now,
        });

        // Setup listeners
        this.setupTokenManagerListeners();

        // Store auth data
        this.persistAuth();

        // Initialize other stores
        await this.initializeUserStores();

        return true;
      } catch (error) {
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
        if (window.$router) {
          window.$router.push('/login');
        }
      }
    },

    /**
     * Fetch current user data
     */
    async fetchCurrentUser() {
      try {
        const user = await authService.getCurrentUser();
        this.user = user;
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
        this.user = { ...this.user, ...updatedUser };
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

        // Fetch workspace users
        await userStore.fetchWorkspaceUsers().catch(err => {
          console.warn('Failed to fetch workspace users:', err);
        });

        // Fetch workspace data (will use user data as fallback)
        await workspaceStore.fetchCurrentWorkspace().catch(err => {
          console.warn('Failed to fetch workspace data:', err);
        });

        // Set current workspace if available
        if (this.user?.workspace_id) {
          workspaceStore.setCurrentWorkspaceId(this.user.workspace_id);
        }
      } catch (error) {
        console.error('Failed to initialize user stores:', error);
        // Don't throw - initialization should continue even if some stores fail
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
      // Clear state
      this.user = null;
      this.isAuthenticated = false;
      this.error = null;
      this.lastLoginTime = null;
      this.sessionStartTime = null;
      this.isInitialized = false; // Reset initialization flag

      // Clear token manager
      tokenManager.clearTokens();

      // Clear localStorage
      localStorage.removeItem('auth');

      // Clear other stores
      try {
        const userStore = useUserStore();
        const workspaceStore = useWorkspaceStore();

        userStore.$reset();
        workspaceStore.$reset();
      } catch (error) {
        console.error('Error clearing stores:', error);
      }
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
     * Update user activity
     */
    updateActivity() {
      tokenManager.updateActivity();
    },
  },
}); 