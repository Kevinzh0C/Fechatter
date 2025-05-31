import { defineStore } from 'pinia';
import axios from 'axios';

export const useWorkspaceStore = defineStore('workspace', {
  state: () => ({
    currentWorkspace: null,
    workspaces: [],
    workspaceUsers: [],
    workspaceChats: [],
    loading: false,
    error: null,
    statsRefreshInterval: null,
  }),

  getters: {
    workspaceName: (state) => {
      return state.currentWorkspace?.name || 'Fechatter';
    },
    
    workspaceOwner: (state) => {
      if (!state.currentWorkspace) return null;
      return state.workspaceUsers.find(user => user.id === state.currentWorkspace.owner_id);
    },
    
    isWorkspaceOwner: (state) => (userId) => {
      return state.currentWorkspace?.owner_id === userId;
    },
    
    memberCount: (state) => {
      return state.workspaceUsers.length;
    },

    chatStats: (state) => {
      return state.workspaceChats.map(chat => ({
        ...chat,
        activity_level: chat.activity_score > 50 ? 'high' : chat.activity_score > 20 ? 'medium' : 'low'
      }));
    }
  },

  actions: {
    async fetchCurrentWorkspace() {
      try {
        this.loading = true;
        this.error = null;
        
        // Get current user's workspace info
        const response = await axios.get('/api/workspaces');
        this.currentWorkspace = response.data;
        
        return response.data;
      } catch (error) {
        this.error = error.response?.data?.message || 'Failed to fetch workspace';
        console.error('Failed to fetch workspace:', error);
        throw error;
      } finally {
        this.loading = false;
      }
    },

    async fetchWorkspaceUsers() {
      try {
        this.loading = true;
        this.error = null;
        
        const response = await axios.get('/api/users');
        this.workspaceUsers = response.data;
        
        return response.data;
      } catch (error) {
        this.error = error.response?.data?.message || 'Failed to fetch workspace users';
        console.error('Failed to fetch workspace users:', error);
        throw error;
      } finally {
        this.loading = false;
      }
    },

    async fetchWorkspaceChats() {
      try {
        this.loading = true;
        this.error = null;
        
        const response = await axios.get('/api/workspace/chats');
        this.workspaceChats = response.data;
        
        return response.data;
      } catch (error) {
        this.error = error.response?.data?.message || 'Failed to fetch workspace chats';
        console.error('Failed to fetch workspace chats:', error);
        throw error;
      } finally {
        this.loading = false;
      }
    },

    async updateWorkspace(name, description = '') {
      try {
        this.loading = true;
        this.error = null;
        
        const payload = { name };
        if (description) payload.description = description;
        
        const response = await axios.patch(`/api/workspace/${this.currentWorkspace.id}`, payload);
        
        // Update local state
        this.currentWorkspace = { ...this.currentWorkspace, ...response.data };
        
        return response.data;
      } catch (error) {
        this.error = error.response?.data?.message || 'Failed to update workspace';
        throw error;
      } finally {
        this.loading = false;
      }
    },

    async inviteUserToWorkspace(email, role = 'member') {
      try {
        this.loading = true;
        this.error = null;
        
        const response = await axios.post('/api/workspace/invite', {
          email,
          role
        });
        
        // Refresh workspace users
        await this.fetchWorkspaceUsers();
        
        return response.data;
      } catch (error) {
        this.error = error.response?.data?.message || 'Failed to invite user';
        throw error;
      } finally {
        this.loading = false;
      }
    },

    async removeUserFromWorkspace(userId) {
      try {
        this.loading = true;
        this.error = null;
        
        await axios.delete(`/api/workspace/users/${userId}`);
        
        // Remove user from local state
        this.workspaceUsers = this.workspaceUsers.filter(user => user.id !== userId);
        
        return true;
      } catch (error) {
        this.error = error.response?.data?.message || 'Failed to remove user';
        throw error;
      } finally {
        this.loading = false;
      }
    },

    async transferWorkspaceOwnership(newOwnerId) {
      try {
        this.loading = true;
        this.error = null;
        
        const response = await axios.patch(`/api/workspace/${this.currentWorkspace.id}/transfer`, {
          new_owner_id: newOwnerId
        });
        
        // Update local state
        this.currentWorkspace.owner_id = newOwnerId;
        
        return response.data;
      } catch (error) {
        this.error = error.response?.data?.message || 'Failed to transfer ownership';
        throw error;
      } finally {
        this.loading = false;
      }
    },

    // Initialize workspace data
    async initializeWorkspace() {
      try {
        await Promise.all([
          this.fetchCurrentWorkspace(),
          this.fetchWorkspaceUsers(),
          this.fetchWorkspaceChats()
        ]);
        
        // 设置定时刷新统计数据（每30秒）
        this.startStatsRefresh();
      } catch (error) {
        console.error('Failed to initialize workspace:', error);
      }
    },

    // 新增：启动统计数据定时刷新
    startStatsRefresh() {
      // 清除现有的定时器
      if (this.statsRefreshInterval) {
        clearInterval(this.statsRefreshInterval);
      }
      
      // 每30秒刷新一次统计数据
      this.statsRefreshInterval = setInterval(async () => {
        try {
          await this.fetchWorkspaceChats();
        } catch (error) {
          console.warn('Failed to refresh workspace stats:', error);
        }
      }, 30000);
    },

    // 新增：停止统计数据刷新
    stopStatsRefresh() {
      if (this.statsRefreshInterval) {
        clearInterval(this.statsRefreshInterval);
        this.statsRefreshInterval = null;
      }
    },

    // 新增：手动刷新统计数据
    async refreshStats() {
      try {
        await this.fetchWorkspaceChats();
      } catch (error) {
        console.error('Failed to refresh stats:', error);
        throw error;
      }
    },

    clearError() {
      this.error = null;
    },

    // Utility methods
    getUserById(userId) {
      return this.workspaceUsers.find(user => user.id === userId);
    },

    getUserByEmail(email) {
      return this.workspaceUsers.find(user => user.email === email);
    },

    getAvailableUsers(excludeIds = []) {
      return this.workspaceUsers.filter(user => !excludeIds.includes(user.id));
    },

    async updateWorkspaceChat(chatId, updates) {
      try {
        this.loading = true;
        this.error = null;
        
        const response = await axios.patch(`/api/workspace/chats/${chatId}`, updates);
        
        // 更新本地状态
        const chatIndex = this.workspaceChats.findIndex(chat => chat.id === chatId);
        if (chatIndex !== -1) {
          this.workspaceChats[chatIndex] = response.data;
        }
        
        return response.data;
      } catch (error) {
        this.error = error.response?.data?.message || 'Failed to update workspace chat';
        throw error;
      } finally {
        this.loading = false;
      }
    },

    async archiveWorkspaceChat(chatId, archived = true) {
      try {
        this.loading = true;
        this.error = null;
        
        const response = await axios.post(`/api/workspace/chats/${chatId}/archive`, {
          archived
        });
        
        // 刷新聊天统计数据
        await this.fetchWorkspaceChats();
        
        return response.data;
      } catch (error) {
        this.error = error.response?.data?.message || `Failed to ${archived ? 'archive' : 'unarchive'} chat`;
        throw error;
      } finally {
        this.loading = false;
      }
    },
  }
}); 