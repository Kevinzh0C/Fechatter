import { defineStore } from 'pinia';
import api from '../services/api.js';
import { useAuthStore } from './auth.js';

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

        // Backend doesn't have a /workspaces endpoint
        // We need to construct workspace info from user data
        const authStore = useAuthStore();
        const user = authStore.user;

        if (!user || !user.workspace_id) {
          throw new Error('No workspace information available');
        }

        // Create workspace object from user data
        this.currentWorkspace = {
          id: user.workspace_id,
          name: 'Fechatter Workspace',
          description: 'Default workspace',
          owner_id: user.id,
          member_count: 1, // Will be updated when fetching users
          created_at: new Date().toISOString()
        };

        return this.currentWorkspace;
      } catch (error) {
        // If API fails, use default workspace
        console.warn('Failed to fetch workspace, using default:', error);
        this.currentWorkspace = {
          id: 'default',
          name: 'Fechatter',
          description: 'Default workspace',
          owner_id: null,
          member_count: 1,
          created_at: new Date().toISOString()
        };
        return this.currentWorkspace;
      } finally {
        this.loading = false;
      }
    },

    async fetchWorkspaceUsers() {
      try {
        this.loading = true;
        this.error = null;

        // Use the correct endpoint that exists in backend
        const response = await api.get('/users');
        const data = response.data?.data || response.data || [];

        this.workspaceUsers = Array.isArray(data) ? data : [];

        return this.workspaceUsers;
      } catch (error) {
        // If API fails, return empty array
        console.warn('Failed to fetch workspace users:', error);
        this.workspaceUsers = [];
        return this.workspaceUsers;
      } finally {
        this.loading = false;
      }
    },

    async fetchWorkspaceChats() {
      try {
        this.loading = true;
        this.error = null;

        const response = await api.get('/workspace/chats');
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

        // Workspace update endpoint doesn't exist in backend
        console.warn('Workspace update endpoint not implemented in backend');
        throw new Error('Workspace update not implemented');

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

    /*
    // TODO: Backend endpoint POST /workspace/invite is not implemented yet.
    // This function is commented out to prevent frontend errors.
    // See API_CONTRACT.md for more details.
    async inviteUserToWorkspace(email, role = 'member') {
      try {
        this.loading = true;
        this.error = null;
        
        const response = await api.post('/workspace/invite', {
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
    */

    /*
    // TODO: Backend endpoint DELETE /workspace/users/{userId} is not implemented yet.
    // This function is commented out to prevent frontend errors.
    // See API_CONTRACT.md for more details.
    async removeUserFromWorkspace(userId) {
      try {
        this.loading = true;
        this.error = null;
        
        await api.delete(`/workspace/users/${userId}`);
        
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
    */

    async transferWorkspaceOwnership(newOwnerId) {
      try {
        this.loading = true;
        this.error = null;

        // Workspace transfer endpoint doesn't exist in backend
        console.warn('Workspace transfer endpoint not implemented in backend');
        throw new Error('Workspace transfer not implemented');

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
        // Fetch workspace data (will use user data as fallback)
        await this.fetchCurrentWorkspace();
        
        // Fetch users and chats in parallel
        await Promise.all([
          this.fetchWorkspaceUsers().catch(err => {
            console.warn('Failed to fetch users:', err);
            return [];
          }),
          this.fetchWorkspaceChats().catch(err => {
            console.warn('Failed to fetch chats:', err);
            return [];
          })
        ]);

        // Update member count based on fetched users
        if (this.currentWorkspace && this.workspaceUsers.length > 0) {
          this.currentWorkspace.member_count = this.workspaceUsers.length;
        }

        // 设置定时刷新统计数据（每30秒）
        this.startStatsRefresh();
      } catch (error) {
        console.error('Failed to initialize workspace:', error);
        // Even if initialization fails, we should have a default workspace
        if (!this.currentWorkspace) {
          this.currentWorkspace = {
            id: 'default',
            name: 'Fechatter',
            description: 'Default workspace',
            owner_id: null,
            member_count: 0,
            created_at: new Date().toISOString()
          };
        }
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

        const response = await api.patch(`/workspace/chats/${chatId}`, updates);

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

        const response = await api.post(`/workspace/chats/${chatId}/archive`, {
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

    // ===== 健康检查兼容性方法 =====
    // 为健康检查系统提供期望的方法名
    async fetchWorkspaces() {
      // 别名：调用 fetchCurrentWorkspace
      return await this.fetchCurrentWorkspace();
    },

    setActiveWorkspace(workspace) {
      // 设置当前活动的工作区
      this.currentWorkspace = workspace;
      return workspace;
    },

    /**
     * Set current workspace ID
     * This method is called by auth store after login
     */
    setCurrentWorkspaceId(workspaceId) {
      // If we already have the workspace loaded, just update the ID
      if (this.currentWorkspace) {
        this.currentWorkspace.id = workspaceId;
      } else {
        // Create a minimal workspace object
        this.currentWorkspace = {
          id: workspaceId,
          name: 'Workspace',
          description: 'User workspace',
          owner_id: null
        };
      }

      // Optionally fetch full workspace data
      // this.fetchCurrentWorkspace().catch(console.error);

      return this.currentWorkspace;
    },
  }
}); 