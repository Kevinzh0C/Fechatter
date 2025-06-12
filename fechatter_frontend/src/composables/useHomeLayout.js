import { ref, readonly, computed, onMounted, onUnmounted } from 'vue';
import { useRouter } from 'vue-router';
import { useAuthStore } from '@/stores/auth';
import { useChatStore } from '@/stores/chat';
import { useWorkspaceStore } from '@/stores/workspace';
import realtimeCommunicationService from '@/services/sse';
import channelPreloaderService from '@/services/channel-preloader';
import api from '@/services/api';

export function useHomeLayout() {
  const router = useRouter();
  const authStore = useAuthStore();
  const chatStore = useChatStore();
  const workspaceStore = useWorkspaceStore();

  // State - optimized for performance
  const state = ref({
    layoutState: 'INIT',
    sidebarState: 'open', // Default to open for better UX
    channels: [],
    loading: false, // Start with false since we use preloaded data
    error: null,
    dataLoaded: false,
    lastLoadTime: null,
    isPreloadedData: false, // Track if using preloaded data
  });

  // Modal states
  const showCreateChannelModal = ref(false);
  const showCreateDMModal = ref(false);

  // SSE connection state (renamed from wsConnectionState for clarity)
  const sseConnectionState = ref({
    isConnected: false,
    latency: null,
    reconnectAttempts: 0,
    connectionType: 'SSE'
  });

  // Computed - memoized for performance
  const currentUser = computed(() => {
    const user = authStore.user;
    return user ? {
      id: user.id,
      fullname: user.fullname || 'User',
      email: user.email || '',
      initials: getInitials(user.fullname || user.email || 'U'),
      status: user.status || 'Active',
      ...user
    } : null;
  });

  const currentWorkspace = computed(() => {
    // Extract workspace from user data or use defaults
    const user = authStore.user;
    return {
      id: user?.workspace_id || 1,
      name: user?.workspace?.name || 'Fechatter Workspace',
      description: 'Your team workspace'
    };
  });

  const isLoading = computed(() => state.value.loading);
  const channels = computed(() => state.value.channels);
  const isSidebarOpen = computed(() => state.value.sidebarState === 'open');

  // Utility functions
  const getInitials = (name) => {
    if (!name) return '?';
    return name
      .split(' ')
      .map(n => n.charAt(0))
      .join('')
      .toUpperCase()
      .slice(0, 2);
  };

  // ✨ 快速加载 - 使用预加载数据
  const loadWithPreloadedData = async () => {
    const startTime = performance.now();

    try {
      // 1. 首先尝试获取预加载的数据
      const preloadedChannels = channelPreloaderService.getPreloadedChannels();

      if (preloadedChannels && preloadedChannels.length > 0) {
        // 立即使用预加载的数据
        state.value.channels = preloadedChannels;
        state.value.isPreloadedData = true;
        state.value.dataLoaded = true;

        const loadTime = performance.now() - startTime;
        // 在后台验证和刷新数据
        backgroundRefreshData();

        return preloadedChannels;
      } else {
        // 如果没有预加载数据，回退到常规加载
        return await loadFreshData();
      }

    } catch (error) {
      console.error('🏠 [HOME] Failed to load with preloaded data:', error);
      // 回退到常规加载
      return await loadFreshData();
    }
  };

  // 🔄 常规数据加载
  const loadFreshData = async () => {
    const startTime = performance.now();
    try {
      state.value.loading = true;
      state.value.error = null;

      // Fast parallel data loading
      const promises = [
        loadUserChats(),
        connectSSE(),
      ];

      const results = await Promise.allSettled(promises);

      // Handle results
      const channels = results[0].status === 'fulfilled' ? results[0].value : [];
      const sseConnected = results[1].status === 'fulfilled' ? results[1].value : false;

      state.value.channels = channels;
      state.value.dataLoaded = true;
      state.value.lastLoadTime = Date.now();
      state.value.isPreloadedData = false;

      const loadTime = performance.now() - startTime;
      return channels;

    } catch (error) {
      console.error('🏠 [HOME] Load error:', error);
      state.value.error = 'Failed to load home data';
      throw error;
    } finally {
      state.value.loading = false;
    }
  };

  // 🔄 后台刷新数据
  const backgroundRefreshData = async () => {
    try {
      // 不显示loading状态，静默刷新
      const freshChannels = await loadUserChats();

      // 比较并更新数据
      if (JSON.stringify(freshChannels) !== JSON.stringify(state.value.channels)) {
        state.value.channels = freshChannels;
        state.value.lastLoadTime = Date.now();
      }

    } catch (error) {
      console.warn('🏠 [HOME] Background refresh failed:', error);
      // 后台刷新失败不影响用户体验
    }
  };

  // Fast user chats loading
  const loadUserChats = async () => {
    try {
      const startTime = performance.now();

      // Use chat store with optimized loading - method now exists
      const chats = await chatStore.fetchChats();

      // Sync unread counts - method now exists
      await chatStore.syncUnreadCounts();

      // Filter channels - use chat store's processed data
      const channels = chats.filter(chat =>
        chat && (chat.chat_type === 'PublicChannel' || chat.chat_type === 'PrivateChannel')
      );

      const loadTime = performance.now() - startTime;
      console.log(`📋 [HOME] Loaded ${channels.length} channels in ${loadTime.toFixed(2)}ms`);
      return channels;
    } catch (error) {
      console.error('📋 [HOME] Failed to load chats:', error);
      return [];
    }
  };

  // 🔌 SSE连接管理
  const connectSSE = async () => {
    const authStore = useAuthStore();
    const token = authStore.token;

    if (!token) {
      console.warn('🔌 [HOME] No authentication token available for SSE connection');
      return false;
    }

    // 检查是否已经连接
    const currentState = realtimeCommunicationService.getConnectionState();
    if (currentState.isConnected || currentState.connectionState === 'connecting') {
      console.log('🔌 [HOME] SSE already connected or connecting, skipping');
      return true;
    }

    try {
      console.log('🔌 [HOME] Initializing configuration for SSE...');
      // Import configuration system
      const { getConfig, initializeConfig } = await import('@/utils/configLoader');

      // Ensure configuration is loaded
      let config = getConfig();
      if (!config) {
        console.log('🔌 [HOME] Initializing configuration for SSE...');
        await initializeConfig();
        config = getConfig();
      }

      console.log('🔌 [HOME] Connecting to real-time communication service...');
      await realtimeCommunicationService.connect(authStore.token);

      console.log('✅ [HOME] Real-time communication connected successfully');
      return true;
    } catch (error) {
      console.error('❌ [HOME] Real-time communication failed:', error);

      // Don't throw error to prevent blocking the main loading flow
      // SSE connection can be retried in the background
      return false;
    }
  };

  // 🔌 设置SSE事件监听器
  const setupSSEListeners = () => {
    // 新消息事件
    realtimeCommunicationService.on('new_message', (message) => {
      // 更新聊天的最后消息
      const chat = chatStore.chats.find(c => c.id === message.chat_id);
      if (chat) {
        // Update last_message with normalized format
        chat.last_message = {
          id: message.id,
          content: message.content,
          created_at: message.created_at,
          sender_id: message.sender_id,
          sender_name: message.sender_name || message.sender?.fullname
        };
        chat.updated_at = message.created_at;

        // 如果是DM，移到列表顶部
        if (chat.chat_type === 'Single') {
          const chatIndex = chatStore.chats.findIndex(c => c.id === message.chat_id);
          if (chatIndex > 0) {
            chatStore.chats.splice(chatIndex, 1);
            chatStore.chats.unshift(chat);
          }
        }

        // Update unread count if not current chat
        if (chatStore.currentChatId !== message.chat_id) {
          chat.unread_count = (chat.unread_count || 0) + 1;
        }
      } else {
        // If chat not in list, might be a closed DM that needs to be re-added
        if (message.chat_type === 'Single') {
          // Fetch the chat and add it to the list
          api.get(`/chat/${message.chat_id}`).then(response => {
            const chatData = response.data?.data || response.data;
            if (chatData) {
              // Check if chat already exists before adding
              const existingChat = chatStore.chats.find(c => c.id === chatData.id);
              if (!existingChat) {
                const normalizedChat = chatStore._normalizeChat(chatData);
                normalizedChat.last_message = {
                  id: message.id,
                  content: message.content,
                  created_at: message.created_at,
                  sender_id: message.sender_id
                };
                normalizedChat.unread_count = 1;
                chatStore.chats.unshift(normalizedChat);
              } else {
                // Update existing chat instead
                existingChat.last_message = {
                  id: message.id,
                  content: message.content,
                  created_at: message.created_at,
                  sender_id: message.sender_id
                };
                existingChat.unread_count = (existingChat.unread_count || 0) + 1;
                // Move to top if it's a DM
                const chatIndex = chatStore.chats.findIndex(c => c.id === existingChat.id);
                if (chatIndex > 0) {
                  chatStore.chats.splice(chatIndex, 1);
                  chatStore.chats.unshift(existingChat);
                }
              }
            }
          }).catch(err => {
            console.error('Failed to fetch chat for new message:', err);
          });
        }
      }
    });

    // 连接状态事件
    realtimeCommunicationService.on('connected', () => {
      sseConnectionState.value.isConnected = true;
      sseConnectionState.value.reconnectAttempts = 0;
    });

    realtimeCommunicationService.on('disconnected', (data) => {
      sseConnectionState.value.isConnected = false;
    });

    // 未读计数更新事件
    realtimeCommunicationService.on('unread_count_changed', (data) => {
      const { chatId, increment, unreadCount, action } = data;

      if (action === 'increment' && increment) {
        chatStore.incrementChatUnreadCount(chatId);
        updateChannelUnreadCount(chatId, increment);
      } else if (action === 'reset') {
        chatStore.resetChatUnreadCount(chatId);
        resetChannelUnreadCount(chatId);
      } else if (typeof unreadCount === 'number') {
        chatStore.updateChatUnreadCount(chatId, unreadCount);
        setChannelUnreadCount(chatId, unreadCount);
      }
    });
  };

  // 🔌 获取SSE连接状态
  const getSSEStatus = () => {
    const connectionState = realtimeCommunicationService.getConnectionState();
    sseConnectionState.value = {
      ...sseConnectionState.value,
      ...connectionState
    };
    return sseConnectionState.value;
  };

  // 更新频道未读计数
  const updateChannelUnreadCount = (chatId, increment = 1) => {
    const channelIndex = state.value.channels.findIndex(c => c.id === chatId);
    if (channelIndex !== -1) {
      state.value.channels[channelIndex].unread_count += increment;

      // 同时更新预加载服务的缓存
      channelPreloaderService.updateChannelUnreadCount(
        chatId,
        state.value.channels[channelIndex].unread_count
      );
    }
  };

  // 重置频道未读计数
  const resetChannelUnreadCount = (chatId) => {
    const channelIndex = state.value.channels.findIndex(c => c.id === chatId);
    if (channelIndex !== -1) {
      state.value.channels[channelIndex].unread_count = 0;

      // 同时更新预加载服务的缓存
      channelPreloaderService.updateChannelUnreadCount(chatId, 0);
    }
  };

  // 设置频道未读计数
  const setChannelUnreadCount = (chatId, count) => {
    const channelIndex = state.value.channels.findIndex(c => c.id === chatId);
    if (channelIndex !== -1) {
      state.value.channels[channelIndex].unread_count = count;

      // 同时更新预加载服务的缓存
      channelPreloaderService.updateChannelUnreadCount(chatId, count);
    }
  };

  // State machine transition - simplified and faster
  const transition = (event) => {
    const currentState = state.value.layoutState;

    switch (currentState) {
      case 'INIT':
        if (event.type === 'LOAD') {
          state.value.layoutState = 'HOME';
          loadWithPreloadedData(); // 使用优化的加载策略
        }
        break;

      case 'HOME':
        if (event.type === 'SELECT_DEFAULT') {
          state.value.layoutState = 'DEFAULT_CHANNEL';
          selectDefaultChannel();
        } else if (event.type === 'CLOSE') {
          state.value.layoutState = 'CLOSED';
        }
        break;

      case 'DEFAULT_CHANNEL':
        if (event.type === 'CLOSE') {
          state.value.layoutState = 'CLOSED';
        }
        break;

      default:
        console.warn(`🏠 [HOME] Invalid transition from ${currentState} with event ${event.type}`);
    }
  };

  // Auto-select default channel for better UX
  const selectDefaultChannel = async () => {
    try {
      // Find a suitable default channel
      const defaultChannel = state.value.channels.find(c =>
        c.name.toLowerCase().includes('general') ||
        c.name.toLowerCase().includes('random') ||
        c.chat_type === 'PublicChannel'
      ) || state.value.channels[0];

      if (defaultChannel) {
        await router.push(`/chat/${defaultChannel.id}`);
      } else {
      }
    } catch (error) {
      console.error('🎯 [HOME] Failed to select default channel:', error);
    }
  };

  // Sidebar state management
  const toggleSidebar = (newState) => {
    state.value.sidebarState = newState;
  };

  const openSidebar = () => toggleSidebar('open');
  const closeSidebar = () => toggleSidebar('closed');

  // Logout with proper cleanup
  const logout = async () => {
    try {
      // Clear local state
      state.value.channels = [];
      state.value.dataLoaded = false;

      // 清理SSE连接
      realtimeCommunicationService.disconnect();

      // 清理预加载服务
      channelPreloaderService.dispose();

      // Use auth store logout (which now handles navigation)
      await authStore.logout();
    } catch (error) {
      console.error('🚪 [HOME] Logout error:', error);
      // Force navigation using window location as fallback
      if (typeof window !== 'undefined') {
        window.location.href = '/login';
      }
    }
  };

  // Modal handlers
  const onChannelCreated = (channel) => {
    showCreateChannelModal.value = false;

    // Add to local state for immediate feedback
    if (channel) {
      const newChannel = {
        id: channel.id,
        name: channel.name,
        chat_type: channel.chat_type,
        description: channel.description,
        unread_count: 0,
        member_count: 1,
      };

      state.value.channels.unshift(newChannel);

      // 同时更新预加载服务缓存
      channelPreloaderService.addChannelToCache(channel);

      // Navigate to new channel
      router.push(`/chat/${channel.id}`);
    }
  };

  const onDMCreated = (dm) => {
    showCreateDMModal.value = false;

    if (dm) {
      // Navigate to new DM
      router.push(`/chat/${dm.id}`);
    }
  };

  // 手动刷新数据
  const refresh = async () => {
    try {
      state.value.loading = true;

      // 刷新预加载服务的数据
      await channelPreloaderService.refresh();

      // 重新加载数据
      await loadWithPreloadedData();

    } catch (error) {
      console.error('🔄 [HOME] Manual refresh failed:', error);
      state.value.error = 'Failed to refresh data';
    } finally {
      state.value.loading = false;
    }
  };

  // Lifecycle - optimized initialization
  onMounted(() => {
    // Check authentication using a more lenient approach
    // Only check for token existence, not user data
    if (!authStore.token || authStore.isTokenExpired) {
      console.warn('🏠 [HOME] No valid token found, redirecting to login');
      console.warn('🔍 [HOME] Debug info:', {
        hasToken: !!authStore.token,
        isTokenExpired: authStore.isTokenExpired,
        isAuthenticated: authStore.isAuthenticated,
        hasUser: !!authStore.user,
        tokenValue: authStore.token ? authStore.token.substring(0, 20) + '...' : 'null'
      });

      router.push('/login');
      return;
    }

    // If we have a token but no user data yet, that's OK
    // The auth store initialization will handle loading user data
    if (!authStore.user && authStore.token) {
      // Continue with loading - user data will be available soon
    }

    // Start loading immediately with optimized strategy
    transition({ type: 'LOAD' });

    // 设置SSE事件监听器
    setupSSEListeners();
  });

  onUnmounted(() => {
    // 清理SSE事件监听器
    cleanupEventListeners();
  });

  // 🧹 清理函数
  const cleanup = () => {
    console.log('🧹 [HOME] Cleaning up home layout...');

    // 断开SSE连接
    realtimeCommunicationService.disconnect();

    // 清理定时器
    if (refreshTimer.value) {
      clearInterval(refreshTimer.value);
      refreshTimer.value = null;
    }

    // 清理事件监听器
    cleanupEventListeners();

  };

  // 🧹 清理事件监听器
  const cleanupEventListeners = () => {
    realtimeCommunicationService.off('new_message');
    realtimeCommunicationService.off('connected');
    realtimeCommunicationService.off('disconnected');
    realtimeCommunicationService.off('unread_count_changed');
  };

  // Return optimized API
  return {
    // State - readonly for external consumers
    state: readonly(state),
    currentUser: readonly(currentUser),
    currentWorkspace: readonly(currentWorkspace),
    channels: readonly(channels),
    isLoading: readonly(isLoading),
    isSidebarOpen: readonly(isSidebarOpen),
    wsConnectionState: readonly(sseConnectionState), // Keep the old name for backward compatibility

    // Modal states
    showCreateChannelModal,
    showCreateDMModal,

    // Actions - only expose necessary methods
    openSidebar,
    closeSidebar,
    logout,
    onChannelCreated,
    onDMCreated,
    refresh,
    getSSEStatus,

    // Expose cleanup for external use
    cleanup,

    // Expose loading status for debugging
    getLoadingStatus: () => ({
      loading: state.value.loading,
      dataLoaded: state.value.dataLoaded,
      isPreloadedData: state.value.isPreloadedData,
      lastLoadTime: state.value.lastLoadTime
    })
  };
}