/**
 * Auth Service
 * 
 * Handles authentication API calls
 */

import api from '@/services/api';
import { errorHandler } from '@/utils/errorHandler';

class AuthService {
  /**
   * Refresh access token
   */
  async refreshToken(refreshToken) {
    try {
      const response = await api.post('/refresh', {}, {
        headers: {
          'Authorization': `Bearer ${refreshToken}`,
        },
        // Don't use interceptors to avoid infinite loop
        skipAuthRefresh: true,
      });

      const data = response.data?.data || response.data;

      return {
        accessToken: data.access_token,
        refreshToken: data.refresh_token || refreshToken, // Use old if not rotated
        expiresIn: data.expires_in || 3600, // Default 1 hour
        tokenType: data.token_type || 'Bearer',
        absoluteExpiry: data.absolute_expiry,
      };
    } catch (error) {
      // Handle specific error cases
      if (error.response?.status === 401) {
        throw new Error('Invalid or expired refresh token');
      }

      errorHandler.handle(error, {
        context: 'Token refresh',
        silent: true,
      });

      throw error;
    }
  }

  /**
   * Login user
   */
  async login(email, password) {
    try {
      const response = await api.post('/signin', {
        email,
        password,
      });

      const data = response.data?.data || response.data;

      return {
        user: data.user,
        accessToken: data.access_token,
        refreshToken: data.refresh_token,
        expiresIn: data.expires_in || 3600,
        tokenType: data.token_type || 'Bearer',
      };
    } catch (error) {
      errorHandler.handle(error, {
        context: 'Login',
        silent: false,
      });

      throw error;
    }
  }

  /**
   * Register user
   */
  async register(userData) {
    try {
      const response = await api.post('/signup', userData);

      const data = response.data?.data || response.data;

      return {
        user: data.user,
        accessToken: data.access_token,
        refreshToken: data.refresh_token,
        expiresIn: data.expires_in || 3600,
        tokenType: data.token_type || 'Bearer',
      };
    } catch (error) {
      errorHandler.handle(error, {
        context: 'Register',
        silent: false,
      });

      throw error;
    }
  }

  /**
   * Logout user
   */
  async logout(refreshToken) {
    try {
      await api.post('/logout', {}, {
        headers: refreshToken ? {
          'Authorization': `Bearer ${refreshToken}`,
        } : {},
      });

      return true;
    } catch (error) {
      // Logout should always succeed locally even if API fails
      console.error('Logout API error:', error);
      return true;
    }
  }

  /**
   * Get current user
   */
  async getCurrentUser() {
    try {
      const response = await api.get('/users/profile');
      const data = response.data?.data || response.data;
      return data;
    } catch (error) {
      errorHandler.handle(error, {
        context: 'Get current user',
        silent: true,
      });

      throw error;
    }
  }

  /**
   * Update user profile
   */
  async updateProfile(profileData) {
    try {
      const response = await api.put('/users/profile', profileData);
      const data = response.data?.data || response.data;
      return data;
    } catch (error) {
      errorHandler.handle(error, {
        context: 'Update profile',
        silent: false,
      });

      throw error;
    }
  }

  /**
   * Change password
   */
  async changePassword(currentPassword, newPassword) {
    try {
      const response = await api.post('/users/change-password', {
        current_password: currentPassword,
        new_password: newPassword,
      });

      return response.data?.data || response.data;
    } catch (error) {
      errorHandler.handle(error, {
        context: 'Change password',
        silent: false,
      });

      throw error;
    }
  }
}

// Create singleton instance
const authService = new AuthService();

export default authService;