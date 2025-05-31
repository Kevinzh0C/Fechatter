import { describe, it, expect, beforeEach } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';
import { useAuthStore } from '../../stores/auth';
import axios from 'axios';
import { vi } from 'vitest';

vi.mock('axios');

describe('Auth Store', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    localStorage.clear();
  });

  describe('login', () => {
    it('should successfully login with super user', async () => {
      const store = useAuthStore();
      const result = await store.login('super@test.com', 'super123');

      expect(result).toBe(true);
      expect(store.token).toBe('super_token_123');
      expect(store.refreshToken).toBe('super_refresh_token_123');
      expect(localStorage.setItem).toHaveBeenCalledWith('token', 'super_token_123');
      expect(localStorage.setItem).toHaveBeenCalledWith('refreshToken', 'super_refresh_token_123');
      expect(axios.post).not.toHaveBeenCalled();
    });

    it('should successfully login with regular user', async () => {
      const store = useAuthStore();
      const mockResponse = {
        data: {
          access_token: 'access123',
          refresh_token: 'refresh123'
        }
      };
      axios.post.mockResolvedValueOnce(mockResponse);

      const result = await store.login('test@example.com', 'password');

      expect(result).toBe(true);
      expect(store.token).toBe('access123');
      expect(store.refreshToken).toBe('refresh123');
      expect(localStorage.setItem).toHaveBeenCalledWith('token', 'access123');
      expect(localStorage.setItem).toHaveBeenCalledWith('refreshToken', 'refresh123');
    });

    it('should handle login failure', async () => {
      const store = useAuthStore();
      axios.post.mockRejectedValueOnce({ response: { data: { message: 'Invalid credentials' } } });

      const result = await store.login('test@example.com', 'wrong');

      expect(result).toBe(false);
      expect(store.error).toBe('Invalid credentials');
      expect(store.token).toBeNull();
    });
  });

  describe('register', () => {
    it('should successfully register', async () => {
      const store = useAuthStore();
      const mockResponse = {
        data: {
          access_token: 'access123',
          refresh_token: 'refresh123'
        }
      };
      axios.post.mockResolvedValueOnce(mockResponse);

      const result = await store.register('Test User', 'test@example.com', 'password', 'workspace');

      expect(result).toBe(true);
      expect(store.token).toBe('access123');
      expect(store.refreshToken).toBe('refresh123');
      expect(localStorage.setItem).toHaveBeenCalledWith('token', 'access123');
      expect(localStorage.setItem).toHaveBeenCalledWith('refreshToken', 'refresh123');
    });
  });

  describe('logout', () => {
    it('should clear tokens on super user logout without API call', async () => {
      const store = useAuthStore();
      store.token = 'super_token_123';
      store.refreshToken = 'super_refresh_token_123';

      await store.logout();

      expect(store.token).toBeNull();
      expect(store.refreshToken).toBeNull();
      expect(localStorage.removeItem).toHaveBeenCalledWith('token');
      expect(localStorage.removeItem).toHaveBeenCalledWith('refreshToken');
      expect(axios.post).not.toHaveBeenCalled();
    });

    it('should clear tokens on regular user logout with API call', async () => {
      const store = useAuthStore();
      store.token = 'access123';
      store.refreshToken = 'refresh123';

      await store.logout();

      expect(store.token).toBeNull();
      expect(store.refreshToken).toBeNull();
      expect(localStorage.removeItem).toHaveBeenCalledWith('token');
      expect(localStorage.removeItem).toHaveBeenCalledWith('refreshToken');
      expect(axios.post).toHaveBeenCalled();
    });
  });

  describe('refreshAccessToken', () => {
    it('should return super token for super user without API call', async () => {
      const store = useAuthStore();
      store.refreshToken = 'super_refresh_token_123';

      const result = await store.refreshAccessToken();

      expect(result).toBe('super_token_123');
      expect(store.token).toBe('super_token_123');
      expect(axios.post).not.toHaveBeenCalled();
    });

    it('should refresh tokens for regular user with API call', async () => {
      const store = useAuthStore();
      const mockResponse = {
        data: {
          access_token: 'new_access123',
          refresh_token: 'new_refresh123'
        }
      };
      axios.post.mockResolvedValueOnce(mockResponse);
      store.refreshToken = 'old_refresh123';

      const result = await store.refreshAccessToken();

      expect(result).toBe('new_access123');
      expect(store.token).toBe('new_access123');
      expect(store.refreshToken).toBe('new_refresh123');
      expect(axios.post).toHaveBeenCalled();
    });
  });
});