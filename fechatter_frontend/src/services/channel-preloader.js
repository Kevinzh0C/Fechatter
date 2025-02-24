/**
 * 🚀 Channel Preloader Service
 * 频道预加载服务 - 快速首屏加载
 */

import { ref, computed } from 'vue';
import { useChatStore } from '@/stores/chat';
import { useUserStore } from '@/stores/user';
import { useWorkspaceStore } from '@/stores/workspace';
import { useAuthStore } from '@/stores/auth';
import { useNotifications } from '@/composables/useNotifications';
import minimalSSE from '@/services/sse-minimal';
import api from '@/services/api';

class ChannelPreloaderService {
  constructor() {
    this.isPreloading = false;
    this.preloadPromise = null;
    this.preloadedData = {
      channels: [],
      isValid: false,
      timestamp: null,
      cacheKey: 'channel_preload_cache',
      cacheExpiry: 2 * 60 * 1000, // 2分钟
    };

    // 预加载配置
    this.config = {
      preloadTimeout: 10000, // 10秒超时
      cacheTimeout: 2 * 60 * 1000, // 2分钟缓存
      enableBackgroundRefresh: true,
      maxRetries: 3
    };

    // 后台刷新定时器
    this.backgroundRefreshTimer = null;
  }

  /**
   * 开始预加载 - 在用户登录后立即调用
   */
  async startPreloading() {
    if (this.isPreloading && this.preloadPromise) {
      return this.preloadPromise;
    }

    // 检查缓存是否有效
    if (this.isCacheValid()) {
      return Promise.resolve(this.preloadedData);
    }

    this.isPreloading = true;

    this.preloadPromise = this.performPreload();

    try {
      const result = await this.preloadPromise;
      return result;
    } catch (error) {
      if (import.meta.env.DEV) {
        console.error('❌ [PRELOADER] Preloading failed:', error);
      throw error;
    } finally {
      this.isPreloading = false;
      this.preloadPromise = null;
    }

  /**
   * 执行预加载逻辑
   */
  async performPreload() {
    const startTime = performance.now();
    const chatStore = useChatStore();
    const authStore = useAuthStore();

    // 确保用户已认证
    if (!authStore.user || !authStore.token) {
      throw new Error('User not authenticated');
    }

    try {
      // 1. 并行预加载多个数据源
      const promises = [
        this.preloadChannels(chatStore),
        this.preloadSSEConnection(authStore.token),
      ];

      // 使用Promise.allSettled确保部分失败不影响其他数据
      const results = await Promise.allSettled(promises);

      // 处理结果
      const channels = results[0].status === 'fulfilled' ? results[0].value : [];
      const sseConnected = results[1].status === 'fulfilled' ? results[1].value : false;

      // 更新缓存
      this.preloadedData = {
        channels,
        sseConnected,
        lastUpdate: Date.now(),
        isValid: true
      };

      // 开始后台刷新
      this.startBackgroundRefresh();

      const loadTime = performance.now() - startTime;
      return this.preloadedData;

    } catch (error) {
      if (import.meta.env.DEV) {
        console.error('📦 [PRELOADER] Preload failed:', error);
      }

      // 即使失败也返回一个基本结构
      this.preloadedData = {
        channels: [],
        sseConnected: false,
        lastUpdate: Date.now(),
        isValid: false,
        error: error.message
      };

      throw error;
    }

  /**
   * 预加载频道列表
   */
  async preloadChannels(chatStore) {
    try {
      const startTime = performance.now();

      // Preload channels
      if (import.meta.env.DEV) {
        console.log('📦 [PRELOADER] Fetching channels...');
      const workspaceStore = useWorkspaceStore();

      // Use workspace store to fetch chats since chat store doesn't have fetchChats
      const chats = await workspaceStore.fetchWorkspaceChats();

      // Update chat store's chats array if needed
      if (chatStore.chats !== undefined) {
        chatStore.chats = chats;
      }

      // Get unread counts
      const unreadCounts = await chatStore.getAllUnreadCounts();

      // Apply unread counts to chats
      chats.forEach(chat => {
        if (unreadCounts[chat.id] !== undefined) {
          chat.unread_count = unreadCounts[chat.id];
        }
      });

      // 过滤和组织频道
      const channels = Array.isArray(chats) ? chats.filter(chat =>
        chat && (chat.chat_type === 'PublicChannel' || chat.chat_type === 'PrivateChannel')
      ) : [];

      // 🔥 获取真实的未读计数
      let realUnreadCounts = {};
      try {
        const { useAuthStore } = await import('@/stores/auth');
        const authStore = useAuthStore();

        if (authStore.token) {
          const api = await import('@/services/api');
          const response = await api.default.get('/unread-counts');

          // 处理 ApiResponse 包装的数据结构：response.data.data
          const unreadData = response.data?.data || response.data;
          if (unreadData && Array.isArray(unreadData)) {
            realUnreadCounts = unreadData.reduce((acc, item) => {
              acc[item.chat_id] = item.unread_count;
              return acc;
            }, {});

        }
      } catch (error) {
        if (import.meta.env.DEV) {
          console.warn('📋 [PRELOADER] Failed to load unread counts:', error);
        }
        // 继续使用默认值0
      }

      const loadTime = performance.now() - startTime;
      return channels.map(channel => ({
        id: channel.id,
        name: channel.name,
        chat_type: channel.chat_type,
        description: channel.description,
        unread_count: realUnreadCounts[channel.id] || unreadCounts[channel.id] || 0, // 使用真实的未读计数
        member_count: channel.chat_members?.length || 0,
        last_activity: channel.updated_at,
        preloaded: true
      }));

    } catch (error) {
      if (import.meta.env.DEV) {
        console.error('📋 [PRELOADER] Failed to preload channels:', error);
      return [];
    }

  /**
   * 预加载SSE实时连接
   */
  async preloadSSEConnection(token) {
    try {
      // 异步连接SSE，不阻塞主要加载流程
      const connectPromise = minimalSSE.connect(token);

      // 设置超时以避免长时间等待
      const timeoutPromise = new Promise((_, reject) =>
        setTimeout(() => reject(new Error('SSE connection timeout')), 5000)
      );

      await Promise.race([connectPromise, timeoutPromise]);

      return true;

    } catch (error) {
      if (import.meta.env.DEV) {
        console.warn('🔌 [PRELOADER] SSE connection failed, will retry in background:', error);
      }

      // SSE连接失败不应该阻塞主要流程
      // 稍后在后台重试
      setTimeout(() => {
        minimalSSE.connect(token).catch(console.warn);
      }, 3000);

      return false;
    }

  /**
   * 获取预加载的数据
   */
  getPreloadedChannels() {
    if (this.isCacheValid()) {
      return this.preloadedData.channels;
    return null;
  }

  /**
   * 检查缓存是否有效
   */
  isCacheValid() {
    if (!this.preloadedData.isValid || !this.preloadedData.lastUpdate) {
      return false;
    }

    const now = Date.now();
    const age = now - this.preloadedData.lastUpdate;

    return age < this.config.cacheTimeout;
  }

  /**
   * 开始后台刷新
   */
  startBackgroundRefresh() {
    if (!this.config.enableBackgroundRefresh) return;

    this.stopBackgroundRefresh();

    // 每30秒检查一次是否需要刷新
    this.backgroundRefreshTimer = setInterval(() => {
      if (!this.isCacheValid() && !this.isPreloading) {
        this.startPreloading().catch(error => {
          if (import.meta.env.DEV) {
            console.warn('🔄 [PRELOADER] Background refresh failed:', error);
          }
        });
    }, 30000);
  }

  /**
   * 停止后台刷新
   */
  stopBackgroundRefresh() {
    if (this.backgroundRefreshTimer) {
      clearInterval(this.backgroundRefreshTimer);
      this.backgroundRefreshTimer = null;
    }

  /**
   * 手动刷新数据
   */
  async refresh() {
    this.invalidateCache();
    return this.startPreloading();
  }

  /**
   * 使缓存失效
   */
  invalidateCache() {
    this.preloadedData.isValid = false;
    this.preloadedData.lastUpdate = null;
  }

  /**
   * 添加新频道到缓存
   */
  addChannelToCache(channel) {
    if (this.preloadedData.isValid) {
      const formattedChannel = {
        id: channel.id,
        name: channel.name,
        chat_type: channel.chat_type,
        description: channel.description,
        unread_count: 0,
        member_count: 1,
        last_activity: new Date().toISOString(),
        preloaded: false // 标记为非预加载的新频道
      };

      this.preloadedData.channels.unshift(formattedChannel);
    }

  /**
   * 从缓存中移除频道
   */
  removeChannelFromCache(channelId) {
    if (this.preloadedData.isValid) {
      this.preloadedData.channels = this.preloadedData.channels.filter(
        channel => channel.id !== channelId
      );
    }

  /**
   * 更新频道未读计数
   */
  updateChannelUnreadCount(channelId, unreadCount) {
    if (this.preloadedData.isValid) {
      const channel = this.preloadedData.channels.find(c => c.id === channelId);
      if (channel) {
        const oldCount = channel.unread_count;
        channel.unread_count = unreadCount;

      } else {
        if (import.meta.env.DEV) {
          console.warn(`📦 [PRELOADER] Channel ${channelId} not found in preloaded data`);
        }

  /**
   * 增加频道未读计数
   */
  incrementChannelUnreadCount(channelId, increment = 1) {
    if (this.preloadedData.isValid) {
      const channel = this.preloadedData.channels.find(c => c.id === channelId);
      if (channel) {
        const oldCount = channel.unread_count;
        channel.unread_count += increment;

      }

  /**
   * 重置频道未读计数
   */
  resetChannelUnreadCount(channelId) {
    this.updateChannelUnreadCount(channelId, 0);
  }

  /**
   * 获取预加载状态
   */
  getPreloadStatus() {
    return {
      isPreloading: this.isPreloading,
      isValid: this.preloadedData.isValid,
      lastUpdate: this.preloadedData.lastUpdate,
      channelCount: this.preloadedData.channels.length,
      sseConnected: this.preloadedData.sseConnected
    };
  }

  /**
   * 清理资源
   */
  dispose() {
    this.stopBackgroundRefresh();
    this.invalidateCache();
    this.isPreloading = false;
    this.preloadPromise = null;

    // 断开SSE连接
    minimalSSE.disconnect();

  }

  /**
   * 🚀 并行加载：预加载数据 + SSE连接 
   */
  async startParallelLoading(authStore, chatStore) {
    const startTime = performance.now();

    const token = authStore.token;

    // 🔥 并行启动：数据预加载 + SSE连接
    const [dataPromise, connectPromise] = await Promise.allSettled([
      this.preloadChannels(chatStore),
      minimalSSE.connect(token)
    ]);

    // 检查预加载结果
    if (dataPromise.status === 'fulfilled') {
    } else {
      if (import.meta.env.DEV) {
        console.warn('⚠️ [PRELOADER] Data preloading failed:', dataPromise.reason);
      }

    // 检查SSE连接结果
    if (connectPromise.status === 'fulfilled') {
    } else {
      if (import.meta.env.DEV) {
        console.warn('⚠️ [PRELOADER] Real-time connection failed:', connectPromise.reason);
      }
      // SSE连接失败时，仍然可以尝试在后台重连
      minimalSSE.connect(token).catch(console.warn);
    }

    const endTime = performance.now();
    return {
      dataSuccess: dataPromise.status === 'fulfilled',
      sseSuccess: connectPromise.status === 'fulfilled',
      totalTime: endTime - startTime
    };
  }

  /**
   * 断开连接并清理
   */
  disconnect() {
    // 断开SSE连接
    minimalSSE.disconnect();

    // 清除缓存
    this.clearCache();

    if (import.meta.env.DEV) {
      console.log('🧹 [PRELOADER] Cleanup completed');
    }

// 单例实例
const channelPreloaderService = new ChannelPreloaderService();

export default channelPreloaderService; 