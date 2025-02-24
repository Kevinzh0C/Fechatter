import { defineStore } from 'pinia';
import { userEndpointManager } from '../services/api/userEndpoints.js';

export const useUserStore = defineStore('user', {
  state: () => ({
    workspaceUsers: [],
    loading: false,
    error: null,
    userCache: new Map(),
    hasFetchedAllUsers: false,
    users: [],
    lastFetch: 0,
    fetchPromise: null
  }),

  actions: {
    /**
     * Fetch all users in the workspace, ensuring it only runs once.
     */
    async fetchWorkspaceUsers() {
      // If a fetch is already in progress, return the existing promise
      if (this.loading && this.fetchPromise) {
        return this.fetchPromise;
      }

      const now = Date.now();
      // Only fetch if cache is empty or older than 5 minutes
      if (this.workspaceUsers.length > 0 && (now - this.lastFetch < 300000)) {
        return this.workspaceUsers;
      }

      this.loading = true;
      this.fetchPromise = (async () => {
        try {
          const users = await userEndpointManager.fetchWorkspaceUsers();
          this.workspaceUsers = users;
          this.userCache.clear();
          users.forEach(user => this.userCache.set(user.id, user));
          this.lastFetch = Date.now();
          return users;
        } catch (error) {
          if (import.meta.env.DEV) {
            console.error('[UserStore] Failed to fetch users:', error);
          }
          // Return empty array on failure but still resolve the promise
          return [];
        } finally {
          this.loading = false;
          this.fetchPromise = null;
        }
      })();
      return this.fetchPromise;
    },

    clearError() {
      this.error = null;
    },

    /**
     * Fetch users by their IDs.
     * @param {number[]} userIds - An array of user IDs to fetch.
     */
    async fetchUsersByIds(userIds) {
      if (!userIds || userIds.length === 0) {
        return;
      }
      try {
        const users = await userEndpointManager.fetchUsersByIds(userIds);
        users.forEach(user => {
          this.userCache.set(user.id, user);
        });
      } catch (error) {
        if (import.meta.env.DEV) {
          console.error('[UserStore] Failed to fetch users by IDs:', error);
        }
      }
    },
  },

  getters: {
    getWorkspaceUsers: (state) => state.workspaceUsers,

    getAvailableUsers: (state) => (excludeIds = []) => {
      return state.workspaceUsers.filter(user => !excludeIds.includes(user.id));
    },

    /**
     * Check if a user exists in the store or cache.
     * @param {object} state - The store's state.
     * @returns {function(number): boolean} A function that takes a user ID and returns a boolean.
     */
    hasUser: (state) => (userId) => {
      return state.userCache.has(userId);
    },

    getUserById: (state) => (userId) => {
      return state.userCache.get(userId);
    },
  }
}); 