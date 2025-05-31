import { defineStore } from 'pinia';
import api from '@/services/api';

export const useAuthStore = defineStore('auth', {
  state: () => ({
    token: localStorage.getItem('token') || null,
    refreshToken: localStorage.getItem('refreshToken') || null,
    user: JSON.parse(localStorage.getItem('user') || 'null'),
    loading: false,
    error: null,
    isAuthenticated: false,
  }),

  getters: {
    isLoggedIn: (state) => !!state.token && !!state.refreshToken,
  },

  actions: {
    async login(email, password) {
      try {
        this.loading = true;
        this.error = null;
        
        // Super user bypass for testing
        if (email === 'super@test.com' && password === 'super123') {
          const mockToken = 'super_token_123';
          const mockRefreshToken = 'super_refresh_token_123';
          const mockUser = { id: 1, email, fullname: 'Super User' };
          
          this.token = mockToken;
          this.refreshToken = mockRefreshToken;
          this.user = mockUser;
          this.isAuthenticated = true;
          
          localStorage.setItem('token', mockToken);
          localStorage.setItem('refreshToken', mockRefreshToken);
          localStorage.setItem('user', JSON.stringify(mockUser));
          localStorage.setItem('auth_token', mockToken);
          
          return true;
        }

        // Use the configured api instance with proper baseURL
        const response = await api.post('/signin', { 
          email: email.trim(), 
          password 
        });
        
        const { access_token, refresh_token, user } = response.data;
        
        if (!access_token || !refresh_token) {
          throw new Error('Invalid response from server');
        }
        
        this.token = access_token;
        this.refreshToken = refresh_token;
        this.user = user;
        this.isAuthenticated = true;
        
        // Store in localStorage
        localStorage.setItem('token', access_token);
        localStorage.setItem('refreshToken', refresh_token);
        localStorage.setItem('auth_token', access_token);
        if (user) {
          localStorage.setItem('user', JSON.stringify(user));
        }
        
        return true;
      } catch (error) {
        console.error('Login error:', error);
        this.error = this.getErrorMessage(error);
        this.isAuthenticated = false;
        return false;
      } finally {
        this.loading = false;
      }
    },

    async register(fullname, email, password, workspace) {
      try {
        this.loading = true;
        this.error = null;
        
        const response = await api.post('/signup', { 
          fullname: fullname.trim(),
          email: email.trim(), 
          password,
          workspace: workspace?.trim()
        });
        
        const { access_token, refresh_token, user } = response.data;
        
        if (!access_token || !refresh_token) {
          throw new Error('Invalid response from server');
        }
        
        this.token = access_token;
        this.refreshToken = refresh_token;
        this.user = user;
        this.isAuthenticated = true;
        
        // Store in localStorage
        localStorage.setItem('token', access_token);
        localStorage.setItem('refreshToken', refresh_token);
        localStorage.setItem('auth_token', access_token);
        if (user) {
          localStorage.setItem('user', JSON.stringify(user));
        }
        
        return true;
      } catch (error) {
        console.error('Registration error:', error);
        this.error = this.getErrorMessage(error);
        this.isAuthenticated = false;
        return false;
      } finally {
        this.loading = false;
      }
    },

    async logout() {
      try {
        // Don't attempt logout for super user
        if (this.token !== 'super_token_123' && this.refreshToken) {
          try {
            await api.post('/logout', null, {
              headers: {
                Authorization: `Bearer ${this.refreshToken}`
              }
            });
          } catch (error) {
            // Ignore logout errors - still clear local state
            console.warn('Logout request failed:', error);
          }
        }
      } finally {
        this.clearAuthState();
      }
    },

    async logoutAll() {
      try {
        if (this.token !== 'super_token_123') {
          try {
            await api.post('/logout_all');
          } catch (error) {
            console.warn('LogoutAll request failed:', error);
          }
        }
      } finally {
        this.clearAuthState();
      }
    },

    async refreshAccessToken() {
      try {
        if (this.refreshToken === 'super_refresh_token_123') {
          this.token = 'super_token_123';
          localStorage.setItem('token', this.token);
          localStorage.setItem('auth_token', this.token);
          return this.token;
        }

        if (!this.refreshToken) {
          throw new Error('No refresh token available');
        }

        const response = await api.post('/refresh', null, {
          headers: {
            Authorization: `Bearer ${this.refreshToken}`
          }
        });
        
        const { access_token, refresh_token } = response.data;
        
        this.token = access_token;
        if (refresh_token) {
          this.refreshToken = refresh_token;
          localStorage.setItem('refreshToken', refresh_token);
        }
        
        localStorage.setItem('token', access_token);
        localStorage.setItem('auth_token', access_token);
        
        return access_token;
      } catch (error) {
        console.error('Token refresh failed:', error);
        this.clearAuthState();
        throw error;
      }
    },

    clearAuthState() {
      this.token = null;
      this.refreshToken = null;
      this.user = null;
      this.isAuthenticated = false;
      this.error = null;
      
      // Clear localStorage
      localStorage.removeItem('token');
      localStorage.removeItem('refreshToken');
      localStorage.removeItem('auth_token');
      localStorage.removeItem('user');
    },

    getErrorMessage(error) {
      if (error.response?.data?.message) {
        return error.response.data.message;
      }
      if (error.response?.data?.error) {
        return error.response.data.error;
      }
      if (error.response?.status === 401) {
        return 'Invalid credentials';
      }
      if (error.response?.status === 404) {
        return 'Service unavailable';
      }
      if (error.response?.status >= 500) {
        return 'Server error, please try again later';
      }
      if (error.code === 'NETWORK_ERROR' || error.message?.includes('Network Error')) {
        return 'Network error, please check your connection';
      }
      return error.message || 'An unexpected error occurred';
    },

    // Initialize auth state from localStorage
    initializeAuth() {
      const token = localStorage.getItem('token');
      const refreshToken = localStorage.getItem('refreshToken');
      const user = localStorage.getItem('user');
      
      if (token && refreshToken) {
        this.token = token;
        this.refreshToken = refreshToken;
        this.isAuthenticated = true;
        if (user) {
          try {
            this.user = JSON.parse(user);
          } catch (e) {
            console.warn('Failed to parse user data from localStorage');
          }
        }
      }
    }
  },
});