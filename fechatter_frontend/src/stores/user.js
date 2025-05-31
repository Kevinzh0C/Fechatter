import { defineStore } from 'pinia';
import axios from 'axios';

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
        const response = await axios.get('/api/users');
        this.workspaceUsers = response.data;
        return response.data;
      } catch (error) {
        this.error = error.response?.data?.error || 'Failed to fetch workspace users';
        console.error('Failed to fetch workspace users:', error);
        return [];
      } finally {
        this.loading = false;
      }
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