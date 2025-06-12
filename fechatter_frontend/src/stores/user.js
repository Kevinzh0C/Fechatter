import { defineStore } from 'pinia';
import api from '@/services/api';

export const useUserStore = defineStore('user', {
  state: () => ({
    users: [],
    workspaceUsers: [],
    loading: false,
    error: null,
  }),

  actions: {
    async fetchWorkspaceUsers() {
      try {
        this.loading = true;
        this.error = null;
        
        // Call the new /users endpoint
        const response = await api.get('/users');
        
        // Handle API response
        let usersData = [];
        const responseData = response.data;
        
        if (responseData) {
          if (responseData.data && Array.isArray(responseData.data)) {
            // ApiResponse format: { data: [...] }
            usersData = responseData.data;
          } else if (Array.isArray(responseData)) {
            // Direct array format
            usersData = responseData;
          } else {
            console.warn('Unexpected users response format:', responseData);
          }
        }
        
        this.workspaceUsers = usersData;
        return this.workspaceUsers;
      } catch (error) {
        this.error = error.response?.data?.error || 'Failed to fetch workspace users';
        console.error('Failed to fetch workspace users:', error);
        
        // Fallback: return current user if available
        try {
          const authStore = await import('@/stores/auth').then(m => m.useAuthStore());
          const currentUser = authStore.user;
          
          if (currentUser) {
            this.workspaceUsers = [{
              id: currentUser.id,
              fullname: currentUser.fullname || currentUser.email,
              email: currentUser.email,
              username: currentUser.username || currentUser.email.split('@')[0],
              status: 'active'
            }];
            return this.workspaceUsers;
          }
        } catch (fallbackError) {
          console.warn('Fallback user data failed:', fallbackError);
        }
        
        return [];
      } finally {
        this.loading = false;
      }
    },

    async getWorkspaceUsers() {
      // If we already have users cached, return them
      if (this.workspaceUsers.length > 0) {
        return this.workspaceUsers;
      }
      // Otherwise fetch them
      return await this.fetchWorkspaceUsers();
    },

    clearError() {
      this.error = null;
    },

    getUserById(userId) {
      return this.workspaceUsers.find(user => user.id === userId);
    },

    getUserByEmail(email) {
      return this.workspaceUsers.find(user => user.email === email);
    }
  },

  getters: {
    getWorkspaceUsersCount: (state) => {
      return state.workspaceUsers.length;
    },

    getAvailableUsers: (state) => (excludeIds = []) => {
      return state.workspaceUsers.filter(user => !excludeIds.includes(user.id));
    }
  }
}); 