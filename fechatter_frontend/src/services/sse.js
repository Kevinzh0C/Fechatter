/**
 * 🔄 Real-time Communication Service (SSE-based)
 */

// Import required stores and utilities
import { useChatStore } from '@/stores/chat';
import { useAuthStore } from '@/stores/auth';
import { errorHandler } from '@/utils/errorHandler';
import { createSSEErrorHandler } from '@/composables/useSSEErrorSuppression';
import sseGlobalManager from '@/utils/SSEGlobalManager';

class RealtimeCommunicationService {
  constructor() {
    this.eventSource = null;
    this.isConnected = false;
    this.reconnectAttempts = 0;
    this.maxReconnectAttempts = 5; // 限制最大重试次数为5次
    this.reconnectDelay = 3000; // 从3秒开始
    this.messageQueue = [];
    this.eventListeners = new Map();

    // 🚨 全局管理器集成
    this.connectionId = null;
    this.isGloballyBanned = false;
    this.isPaused = false;

    // 新增：重试控制配置
    this.retryControl = {
      totalAttempts: 0,              // 总尝试次数
      maxTotalAttempts: 10,          // 生命周期内最大总尝试次数
      consecutiveFailures: 0,        // 连续失败次数
      maxConsecutiveFailures: 3,     // 最大连续失败次数
      permanentFailure: false,       // 是否永久失败
      lastAttemptTime: null          // 最后尝试时间
    };

    // Connection state
    this.connectionState = 'disconnected'; // disconnected, connecting, connected, reconnecting
    this.lastPingTime = null;
    this.latency = null;

    // 🔧 新增：长期重连策略配置
    this.longTermReconnect = {
      enabled: false,
      intervalMinutes: 10, // 分钟级重连间隔，从10分钟开始
      maxIntervalMinutes: 60, // 最大间隔60分钟
      attempts: 0,
      timeout: null
    };

    // 🔧 新增：网络状态检测
    this.networkStatus = {
      isOnline: navigator.onLine,
      lastOnlineTime: Date.now(),
      offlineStartTime: null
    };

    // 🔧 新增：心跳机制
    this.heartbeat = {
      interval: null,
      intervalMs: 60000, // 60秒心跳间隔
      missedBeats: 0,
      maxMissedBeats: 5 // 允许错过5次心跳
    };

    // 🔧 新增：活动时间跟踪
    this.lastActivityTime = Date.now();

    // 🔧 新增：增强的错误处理器
    this.errorHandler = createSSEErrorHandler({
      maxErrorsPerMinute: 2,      // 更严格的错误限制
      suppressionDurationMs: 120000, // 2分钟抑制期
      resetCounterMs: 600000      // 10分钟重置计数器
    });

    // 🔧 绑定事件处理器方法
    this.boundHandlers = {
      handleNetworkOnline: this.handleNetworkOnline.bind(this),
      handleNetworkOffline: this.handleNetworkOffline.bind(this),
      handleVisibilityChange: this.handleVisibilityChange.bind(this),
      handleWindowFocus: this.handleWindowFocus.bind(this),
      handleWindowBlur: this.handleWindowBlur.bind(this),
      handleBeforeUnload: this.handleBeforeUnload.bind(this)
    };

    // 🔧 设置网络和页面状态监听器
    if (typeof window !== 'undefined') {
      // 监听网络状态变化
      window.addEventListener('online', this.boundHandlers.handleNetworkOnline);
      window.addEventListener('offline', this.boundHandlers.handleNetworkOffline);

      // 🔧 监听页面可见性变化
      document.addEventListener('visibilitychange', this.boundHandlers.handleVisibilityChange);

      // 🔧 监听窗口焦点变化
      window.addEventListener('focus', this.boundHandlers.handleWindowFocus);
      window.addEventListener('blur', this.boundHandlers.handleWindowBlur);

      // 🔧 监听页面卸载 - 发送离线状态
      window.addEventListener('beforeunload', this.boundHandlers.handleBeforeUnload);
    }

    // 🔧 Token refresh timer
    this.tokenRefreshTimer = null;
    this.tokenRefreshInterval = 4 * 60 * 1000; // Refresh every 4 minutes (before 5 min expiry buffer)
  }

  /**
   * Connect to notify_server using SSE instead of WebSocket
   */
  async connect(token) {
    try {
      // 🚨 检查全局管理器状态
      if (this.isGloballyBanned) {
        console.warn('🚨 SSE: Connection is globally banned, refusing to connect');
        throw new Error('SSE connection is globally banned');
      }

      if (this.isPaused) {
        console.warn('🚨 SSE: Connection is paused, refusing to connect');
        throw new Error('SSE connection is paused');
      }

      // Check if already connected
      if (this.isConnected && this.eventSource) {
        console.log('🔌 SSE: Already connected');
        return;
      }

      // 🔧 新增：在连接前检查并刷新 token
      const authStore = useAuthStore();

      // Check if token is about to expire (within 5 minutes)
      if (authStore.shouldRefreshToken || authStore.tokenExpiresIn < 300000) {
        console.log('🔐 SSE: Token is about to expire, refreshing before connection...');
        try {
          await authStore.refreshToken();
          // Use the new token after refresh
          token = authStore.token;
          console.log('✅ SSE: Token refreshed successfully');
        } catch (error) {
          console.error('❌ SSE: Failed to refresh token:', error);
          throw new Error('Failed to refresh token for SSE connection');
        }
      }

      // Check if token is already expired
      if (authStore.isTokenExpired) {
        console.error('❌ SSE: Token is expired, cannot establish connection');
        throw new Error('Token is expired');
      }

      // Close existing connection if any
      if (this.eventSource) {
        this.eventSource.close();
        this.eventSource = null;
      }

      this.connectionState = 'connecting';
      this.retryControl.lastAttemptTime = Date.now();

      try {
        // Import config loader and ensure config is loaded
        const { getApiConfig, getConfig, initializeConfig } = await import('@/utils/configLoader');

        // Ensure configuration is loaded before proceeding
        let config = getConfig();
        if (!config) {
          console.log('🔧 SSE: Configuration not loaded yet, initializing...');
          await initializeConfig();
          config = getConfig();
        }

        const apiConfig = getApiConfig();

        // Validate SSE URL configuration
        let sseUrl = apiConfig.sse_url || 'http://127.0.0.1:8080/events';

        if (!sseUrl) {
          throw new Error('SSE URL not configured');
        }

        // Ensure token is valid
        if (!token || typeof token !== 'string') {
          throw new Error('Invalid authentication token provided for SSE connection');
        }

        // Build complete SSE URL with proper token encoding
        const fullSseUrl = `${sseUrl}?access_token=${encodeURIComponent(token)}`;

        console.log('🔌 SSE: Connecting to', sseUrl);
        console.log('🔧 SSE: Using configuration from', config?.app?.environment || 'default');

        this.eventSource = new EventSource(fullSseUrl);

        // 🚨 向全局管理器注册连接
        try {
          this.connectionId = sseGlobalManager.registerConnection(fullSseUrl, this.eventSource, this);
          console.log(`🚨 SSE connection registered with global manager: ${this.connectionId}`);
        } catch (error) {
          console.error('🚨 Failed to register with global manager:', error.message);
          this.isGloballyBanned = true;
          throw error;
        }

        this.eventSource.onopen = this.handleOpen.bind(this);
        this.eventSource.onmessage = this.handleMessage.bind(this);
        this.eventSource.onerror = this.handleError.bind(this);

        // Listen for specific event types
        this.setupEventListeners();

      } catch (error) {
        console.error('🔌 SSE: Connection setup failed:', error);

        errorHandler.handle(error, {
          context: 'SSE Gateway connection setup',
          silent: false
        });

        this.connectionState = 'disconnected';
        this.scheduleReconnect();
      }
    } catch (error) {
      console.error('🔌 SSE: Connection setup failed:', error);
      errorHandler.handle(error, {
        context: 'SSE Connection setup',
        silent: false
      });
      this.connectionState = 'disconnected';
      this.scheduleReconnect();
    }
  }

  /**
   * Set up SSE event listeners
   */
  setupEventListeners() {
    // New message event
    this.eventSource.addEventListener('NewMessage', (event) => {
      try {
        const data = JSON.parse(event.data);
        this.handleChatMessage(data);
      } catch (error) {
        errorHandler.handle(error, {
          context: 'Parse NewMessage event',
          silent: true
        });
      }
    });

    // Typing status event
    this.eventSource.addEventListener('TypingStatus', (event) => {
      try {
        const data = JSON.parse(event.data);
        this.handleTypingStatus(data);
      } catch (error) {
        errorHandler.handle(error, {
          context: 'Parse TypingStatus event',
          silent: true
        });
      }
    });

    // Message read event
    this.eventSource.addEventListener('MessageRead', (event) => {
      try {
        const data = JSON.parse(event.data);
        this.handleMessageStatus(data);
      } catch (error) {
        errorHandler.handle(error, {
          context: 'Parse MessageRead event',
          silent: true
        });
      }
    });

    // User presence event
    this.eventSource.addEventListener('UserPresence', (event) => {
      try {
        const data = JSON.parse(event.data);
        this.handleUserPresence(data);
      } catch (error) {
        errorHandler.handle(error, {
          context: 'Parse UserPresence event',
          silent: true
        });
      }
    });

    // Chat member events
    this.eventSource.addEventListener('UserJoinedChat', (event) => {
      try {
        const data = JSON.parse(event.data);
        this.emit('user_joined_chat', data);
      } catch (error) {
        errorHandler.handle(error, {
          context: 'Parse UserJoinedChat event',
          silent: true
        });
      }
    });

    this.eventSource.addEventListener('UserLeftChat', (event) => {
      try {
        const data = JSON.parse(event.data);
        this.emit('user_left_chat', data);
      } catch (error) {
        errorHandler.handle(error, {
          context: 'Parse UserLeftChat event',
          silent: true
        });
      }
    });
  }

  /**
   * Handle connection open
   */
  handleOpen(event) {
    this.isConnected = true;
    this.connectionState = 'connected';
    this.reconnectAttempts = 0;
    this.reconnectDelay = 3000; // 重置为初始延迟
    this.lastActivityTime = Date.now(); // 记录活动时间

    // 重置重试控制
    this.retryControl.consecutiveFailures = 0;
    console.log(`✅ SSE: Connection established (Total attempts: ${this.retryControl.totalAttempts})`);

    // 🔧 重置长期重连策略
    this.resetLongTermReconnect();

    // 🔧 启动心跳检测
    this.startHeartbeat();

    // 🔧 启动 token 刷新定时器
    this.startTokenRefreshTimer();

    // 🔧 Send presence update on connection
    this.sendPresenceUpdate('online');

    // Emit connection event
    this.emit('connected');
  }

  /**
   * Default message handler (fallback)
   */
  handleMessage(event) {
    try {
      // 更新活动时间
      this.lastActivityTime = Date.now();

      const data = JSON.parse(event.data);
      // Emit generic message event
      this.emit('message', data);

    } catch (error) {
      errorHandler.handle(error, {
        context: 'Parse SSE message',
        silent: true
      });
    }
  }

  /**
   * Handle chat message
   */
  handleChatMessage(message) {
    const chatStore = useChatStore();

    // Ensure message has all required fields
    const formattedMessage = {
      id: parseInt(message.id),
      chat_id: message.chat_id,
      chat_type: message.chat_type,
      sender_id: message.sender_id || message.user_id,
      content: message.content,
      files: message.files || [],
      created_at: message.created_at ? new Date(message.created_at).toISOString() : new Date().toISOString(),
      sender_name: message.sender_name,
      sender_fullname: message.sender_fullname,
      sender: message.sender,
      realtime: true
    };

    // Add to store (handles both current chat and other chats)
    chatStore.addRealtimeMessage(formattedMessage);

    // Only mark as read for DM messages (group chats do not need read receipts)
    if (chatStore.currentChatId === message.chat_id &&
      message.chat_type === 'Single') {
      this.markCurrentChatMessageRead(message.id, message.chat_id);
    }

    // Emit new message event (for other listeners like notification system)
    this.emit('new_message', formattedMessage);
  }

  /**
   * Mark the current chat's message as read
   */
  markCurrentChatMessageRead(messageId, chatId) {
    // Call backend API asynchronously, do not block main flow
    import('@/services/api').then(({ default: api }) => {
      api.post(`/realtime/chat/${chatId}/messages/${messageId}/read`, {})
        .catch(error => {
          errorHandler.handle(error, {
            context: `Mark message ${messageId} as read`,
            silent: true
          });
        });
    }).catch(err => {
      errorHandler.handle(err, {
        context: 'Import API module for read receipt',
        silent: true
      });
    });
  }

  /**
   * Handle typing status
   */
  handleTypingStatus(typing) {
    this.emit('typing_status', {
      chatId: typing.chat_id,
      userId: typing.user_id,
      isTyping: typing.is_typing,
      userName: typing.user_name,
      timestamp: Date.now()
    });
  }

  /**
   * Handle user presence
   */
  handleUserPresence(presence) {
    this.emit('user_presence', presence);
  }

  /**
   * Handle message status
   */
  handleMessageStatus(status) {
    const chatStore = useChatStore();

    // Update message status (read/delivered)
    chatStore.updateRealtimeMessage(status.message_id, {
      status: 'read',
      read_at: status.read_at
    });

    this.emit('message_status', status);
  }

  /**
   * Error handler
   */
  handleError(error) {
    // 🚨 检查是否被全局管理器禁用
    if (this.isGloballyBanned) {
      console.error('🚨 SSE: Connection is globally banned, refusing to process error');
      return;
    }

    // 🔧 特殊处理 401 错误 - Token 过期
    if (error.type === 'error' && this.eventSource && this.eventSource.readyState === EventSource.CLOSED) {
      // SSE doesn't provide status codes directly, but we can infer 401 from the connection state
      console.warn('🔐 SSE: Connection closed, possibly due to authentication failure');
      
      // Try to refresh token before giving up
      const authStore = useAuthStore();
      if (authStore.tokens.refreshToken && !authStore.isRefreshing) {
        console.log('🔐 SSE: Attempting to refresh token after connection failure...');
        
        authStore.refreshToken().then(() => {
          console.log('✅ SSE: Token refreshed after 401, will reconnect with new token');
          // Reset consecutive failures since this is a token issue, not a network issue
          this.retryControl.consecutiveFailures = 0;
          // Continue with normal reconnection flow
          this.scheduleReconnect();
        }).catch((refreshError) => {
          console.error('❌ SSE: Failed to refresh token after 401:', refreshError);
          // Continue with normal error handling
          this.handleErrorInternal(error);
        });
        
        // Stop further processing while we try to refresh
        return;
      }
    }

    // Normal error handling
    this.handleErrorInternal(error);
  }

  /**
   * Internal error handler (after 401 check)
   */
  handleErrorInternal(error) {
    // 更新重试控制
    this.retryControl.totalAttempts++;
    this.retryControl.consecutiveFailures++;

    const isNetworkError = !navigator.onLine || error.type === 'error';
    const errorType = isNetworkError ? 'network' : 'server';

    console.error(`🔌 SSE: Connection error (Attempt ${this.retryControl.totalAttempts}/${this.retryControl.maxTotalAttempts}, ` +
      `Consecutive failures: ${this.retryControl.consecutiveFailures}/${this.retryControl.maxConsecutiveFailures})`,
      { type: errorType, error });

    // 🚨 向全局管理器报告错误
    if (this.connectionId) {
      const result = sseGlobalManager.recordConnectionError(this.connectionId, error);
      if (result && result.terminate) {
        console.error(`🚨 Global manager requested termination: ${result.reason}`);
        this.forceStop(`全局管理器终止: ${result.reason}`);
        return;
      }
    }

    // 先检查是否达到永久失败条件
    if (this.retryControl.totalAttempts >= this.retryControl.maxTotalAttempts ||
      this.retryControl.consecutiveFailures >= this.retryControl.maxConsecutiveFailures) {
      this.retryControl.permanentFailure = true;
      console.error('🔌 SSE: Maximum retry attempts reached, connection permanently failed');

      // 立即关闭EventSource以防止自动重连
      if (this.eventSource) {
        this.eventSource.close();
        this.eventSource = null;
      }

      // 使用增强的SSE错误处理器处理永久失败
      errorHandler.handleSSERetryError(error, {
        totalAttempts: this.retryControl.totalAttempts,
        maxTotalAttempts: this.retryControl.maxTotalAttempts,
        consecutiveFailures: this.retryControl.consecutiveFailures,
        maxConsecutiveFailures: this.retryControl.maxConsecutiveFailures,
        context: 'SSE Basic Service'
      });

      this.isConnected = false;
      this.connectionState = 'permanently_failed';
      this.stopHeartbeat();
      this.stopTokenRefreshTimer(); // 🔧 停止 token 刷新
      this.emit('permanently_failed', {
        totalAttempts: this.retryControl.totalAttempts,
        consecutiveFailures: this.retryControl.consecutiveFailures
      });
      return; // 立即返回，不再尝试重连
    }

    // 如果还没达到限制，使用普通错误处理
    errorHandler.handle(error, {
      context: `SSE connection error (Attempt ${this.retryControl.totalAttempts})`,
      silent: this.retryControl.totalAttempts > 2 // 前两次显示错误，之后静默
    });

    this.isConnected = false;
    this.connectionState = 'disconnected';

    // 🔧 停止心跳检测
    this.stopHeartbeat();

    // 🔧 停止 token 刷新定时器
    this.stopTokenRefreshTimer();

    // Emit disconnected event
    this.emit('disconnected', { error });

    // SSE will auto-reconnect, but we can implement custom logic
    this.scheduleReconnect();
  }

  /**
   * 🚨 Note: SSE is one-way, cannot send messages.
   * The following methods are kept for interface compatibility but will warn.
   */
  send(data) {
    console.warn('🔌 ⚠️ SSE is read-only. Cannot send messages. Use HTTP API instead.');
    console.warn('🔌 💡 For real-time features like typing, use HTTP API to fechatter_server');
    return false;
  }

  sendTypingStatus(chatId, isTyping) {
    console.warn('🔌 ⚠️ Use HTTP API for typing status:', { chatId, isTyping });
    // You can call HTTP API here to implement typing status
    return false;
  }

  markMessageRead(messageId, chatId) {
    console.warn('🔌 ⚠️ Use HTTP API for marking messages read:', { messageId, chatId });
    // You can call HTTP API here to implement read receipt
    return false;
  }

  /**
   * 🔧 Send presence update via HTTP API
   */
  async sendPresenceUpdate(status = 'online') {
    try {
      const authStore = useAuthStore();
      if (!authStore.token) return;

      // Import API service
      const { default: api } = await import('@/services/api');

      // Send presence update to server
      await api.post('/realtime/presence', {
        status,
        timestamp: new Date().toISOString(),
        device_info: {
          userAgent: navigator.userAgent,
          platform: navigator.platform,
          language: navigator.language
        }
      });

      console.log(`✅ Presence updated to: ${status}`);
    } catch (error) {
      // Don't use errorHandler here to avoid circular dependency
      console.warn('🔧 Failed to send presence update:', error.message);

      // For development/testing, emit a fake presence event
      if (process.env.NODE_ENV === 'development') {
        const authStore = useAuthStore();
        if (authStore.user) {
          this.emit('user_presence', {
            user_id: authStore.user.id,
            status,
            timestamp: new Date().toISOString(),
            last_seen: new Date().toISOString()
          });
        }
      }
    }
  }

  /**
   * 🚨 强制停止连接 (由全局管理器调用)
   */
  forceStop(reason = '外部终止') {
    console.error(`🚨 SSE: Force stopping connection - ${reason}`);

    this.isGloballyBanned = true;
    this.retryControl.permanentFailure = true;
    this.connectionState = 'force_stopped';

    // 立即关闭EventSource
    if (this.eventSource) {
      try {
        this.eventSource.close();
      } catch (error) {
        console.warn('Error closing EventSource in forceStop:', error);
      }
      this.eventSource = null;
    }

    // 停止所有定时器
    this.stopHeartbeat();
    this.resetLongTermReconnect();

    this.isConnected = false;

    // 显示错误消息
    errorHandler.handleSSERetryError(new Error(`SSE连接被强制终止: ${reason}`), {
      totalAttempts: this.retryControl.totalAttempts,
      maxTotalAttempts: this.retryControl.maxTotalAttempts,
      consecutiveFailures: this.retryControl.consecutiveFailures,
      maxConsecutiveFailures: this.retryControl.maxConsecutiveFailures,
      context: 'SSE Force Stop'
    });

    this.emit('force_stopped', { reason });
  }

  /**
   * 🚨 暂停连接 (由全局管理器调用)
   */
  pause() {
    console.log('🚨 SSE: Pausing connection');
    this.isPaused = true;

    if (this.eventSource) {
      try {
        this.eventSource.close();
      } catch (error) {
        console.warn('Error closing EventSource in pause:', error);
      }
      this.eventSource = null;
    }

    this.stopHeartbeat();
    this.isConnected = false;
    this.connectionState = 'paused';
  }

  /**
   * 🚨 恢复连接 (由全局管理器调用)
   */
  resume() {
    console.log('🚨 SSE: Resuming connection');
    this.isPaused = false;

    if (!this.isGloballyBanned && !this.retryControl.permanentFailure) {
      const authStore = useAuthStore();
      if (authStore.token) {
        this.connect(authStore.token);
      }
    }
  }

  /**
   * Schedule reconnection
   */
  scheduleReconnect() {
    // 🚨 检查全局状态
    if (this.isGloballyBanned) {
      console.warn('🚨 SSE: Skipping reconnect due to global ban');
      return;
    }

    if (this.isPaused) {
      console.warn('🚨 SSE: Skipping reconnect due to pause state');
      return;
    }

    // 检查是否已永久失败
    if (this.retryControl.permanentFailure) {
      console.warn('🔌 SSE: Skipping reconnect due to permanent failure');
      return;
    }

    // 🔧 检查网络状态 - 如果离线则暂停重连
    if (!this.networkStatus.isOnline) {
      console.warn('🔌 SSE: Network offline, pausing reconnection');
      return;
    }

    // 🔧 短期重试策略（前5次尝试）
    if (this.reconnectAttempts < this.maxReconnectAttempts) {
      this.connectionState = 'reconnecting';
      this.reconnectAttempts++;

      const delay = this.reconnectDelay;
      console.log(`🔌 SSE: Scheduling reconnect attempt ${this.reconnectAttempts}/${this.maxReconnectAttempts} in ${delay / 1000}s`);

      setTimeout(() => {
        // 在执行重连时再次检查网络状态和永久失败状态
        if (!this.networkStatus.isOnline || this.retryControl.permanentFailure) {
          return;
        }

        const authStore = useAuthStore();
        if (authStore.token) {
          this.connect(authStore.token);
        }
      }, delay);

      // 更温和的指数退避，最大限制在5分钟
      this.reconnectDelay = Math.min(this.reconnectDelay * 1.5, 300000); // 最大5分钟
    } else {
      // 🔧 切换到长期重连策略
      console.warn('🔌 Max short-term reconnection attempts reached, switching to long-term strategy');
      this.enableLongTermReconnect();
    }
  }

  /**
   * 🔧 启用长期重连策略（分钟级别）
   */
  enableLongTermReconnect() {
    if (this.longTermReconnect.enabled) {
      return; // 避免重复启用
    }

    this.longTermReconnect.enabled = true;
    this.longTermReconnect.attempts = 0;
    this.connectionState = 'reconnecting_long_term';

    this.emit('long_term_reconnect_started');

    this.scheduleLongTermReconnect();
  }

  /**
   * 🔧 安排长期重连
   */
  scheduleLongTermReconnect() {
    if (!this.longTermReconnect.enabled) {
      return;
    }

    this.longTermReconnect.attempts++;
    const intervalMs = this.longTermReconnect.intervalMinutes * 60 * 1000;

    this.longTermReconnect.timeout = setTimeout(() => {
      if (!this.isConnected && this.longTermReconnect.enabled) {
        const authStore = useAuthStore();
        if (authStore.token) {
          // 临时重置短期重试计数器以尝试连接
          const originalAttempts = this.reconnectAttempts;
          this.reconnectAttempts = 0;
          this.reconnectDelay = 1000;

          this.connect(authStore.token).then(() => {
            // 连接成功会通过 handleOpen 重置状态
          }).catch(() => {
            // 连接失败，恢复原来的状态并继续长期重连
            this.reconnectAttempts = originalAttempts;
            this.increaseLongTermInterval();
            this.scheduleLongTermReconnect();
          });
        }
      }
    }, intervalMs);
  }

  /**
   * 🔧 增加长期重连间隔
   */
  increaseLongTermInterval() {
    const currentInterval = this.longTermReconnect.intervalMinutes;
    // 逐步增加间隔：5分钟 -> 10分钟 -> 15分钟 -> 30分钟（最大）
    if (currentInterval < 10) {
      this.longTermReconnect.intervalMinutes = 10;
    } else if (currentInterval < 15) {
      this.longTermReconnect.intervalMinutes = 15;
    } else if (currentInterval < 30) {
      this.longTermReconnect.intervalMinutes = 30;
    }

  }

  /**
   * 🔧 重置长期重连策略
   */
  resetLongTermReconnect() {
    if (this.longTermReconnect.timeout) {
      clearTimeout(this.longTermReconnect.timeout);
      this.longTermReconnect.timeout = null;
    }

    this.longTermReconnect.enabled = false;
    this.longTermReconnect.attempts = 0;
    this.longTermReconnect.intervalMinutes = 5; // 重置为5分钟

  }

  /**
   * Update unread count
   */
  updateUnreadCount(chatId) {
    // Get the current chat ID the user is viewing
    const chatStore = useChatStore();
    const isCurrentChat = chatStore.currentChatId === chatId;

    // If not the current chat, increment unread count
    if (!isCurrentChat) {
      this.emit('unread_count_changed', {
        chatId,
        increment: 1,
        action: 'increment'
      });
    } else {
      // If current chat, reset unread count
      this.emit('unread_count_changed', {
        chatId,
        unreadCount: 0,
        action: 'reset'
      });
    }
  }

  /**
   * Reset retry control
   */
  resetRetryControl() {
    this.retryControl = {
      totalAttempts: 0,
      maxTotalAttempts: 10,
      consecutiveFailures: 0,
      maxConsecutiveFailures: 3,
      permanentFailure: false,
      lastAttemptTime: null
    };
    this.reconnectAttempts = 0;
    this.reconnectDelay = 3000;
    console.log('🔌 SSE: Retry control reset');
  }

  /**
   * Disconnect
   */
  disconnect() {
    // 🔧 Send offline status before disconnecting
    this.sendPresenceUpdate('offline');

    // 🚨 通知全局管理器
    if (this.connectionId) {
      console.log(`🚨 SSE: Notifying global manager of disconnect: ${this.connectionId}`);
      // 全局管理器会自动处理连接清理
    }

    if (this.eventSource) {
      this.eventSource.close();
      this.eventSource = null;
    }

    this.isConnected = false;
    this.connectionState = 'disconnected';
    this.reconnectAttempts = 0;
    this.messageQueue = [];

    // 重置重试控制
    this.resetRetryControl();

    // 🔧 停止心跳检测
    this.stopHeartbeat();

    // 🔧 停止 token 刷新定时器
    this.stopTokenRefreshTimer();

    // 🔧 清理长期重连策略
    this.resetLongTermReconnect();

    // 🚨 重置全局状态
    this.connectionId = null;
    this.isGloballyBanned = false;
    this.isPaused = false;

    // 🔧 清理事件监听器
    if (typeof window !== 'undefined' && this.boundHandlers) {
      window.removeEventListener('online', this.boundHandlers.handleNetworkOnline);
      window.removeEventListener('offline', this.boundHandlers.handleNetworkOffline);
      document.removeEventListener('visibilitychange', this.boundHandlers.handleVisibilityChange);
      window.removeEventListener('focus', this.boundHandlers.handleWindowFocus);
      window.removeEventListener('blur', this.boundHandlers.handleWindowBlur);
      window.removeEventListener('beforeunload', this.boundHandlers.handleBeforeUnload);
    }
  }

  /**
   * Add event listener
   */
  on(event, callback) {
    if (!this.eventListeners.has(event)) {
      this.eventListeners.set(event, new Set());
    }
    this.eventListeners.get(event).add(callback);
  }

  /**
   * Remove event listener
   */
  off(event, callback) {
    if (this.eventListeners.has(event)) {
      this.eventListeners.get(event).delete(callback);
    }
  }

  /**
   * Emit event
   */
  emit(event, data) {
    if (this.eventListeners.has(event)) {
      this.eventListeners.get(event).forEach(callback => {
        try {
          callback(data);
        } catch (error) {
          errorHandler.handle(error, {
            context: `Event callback for ${event}`,
            silent: true
          });
        }
      });
    }
  }

  /**
   * 🔧 网络状态处理：网络恢复时
   */
  handleNetworkOnline() {
    this.networkStatus.isOnline = true;
    this.networkStatus.lastOnlineTime = Date.now();

    if (this.networkStatus.offlineStartTime) {
      const offlineDuration = Date.now() - this.networkStatus.offlineStartTime;
      this.networkStatus.offlineStartTime = null;
    }

    // 网络恢复时立即尝试重连
    if (!this.isConnected) {
      this.scheduleReconnect();
    }
  }

  /**
   * 🔧 网络状态处理：网络断开时
   */
  handleNetworkOffline() {
    this.networkStatus.isOnline = false;
    this.networkStatus.offlineStartTime = Date.now();

    // 停止心跳检测
    this.stopHeartbeat();

    // 暂停重连尝试
    if (this.longTermReconnect.timeout) {
      clearTimeout(this.longTermReconnect.timeout);
      this.longTermReconnect.timeout = null;
    }
  }

  /**
   * 🔧 启动心跳检测
   */
  startHeartbeat() {
    if (this.heartbeat.interval) {
      clearInterval(this.heartbeat.interval);
    }

    this.heartbeat.missedBeats = 0;
    // 每10秒检查一次心跳状态，而不是每30秒
    this.heartbeat.interval = setInterval(() => {
      if (!this.isConnected) {
        this.stopHeartbeat();
        return;
      }

      // 检查连接是否仍然活跃
      // SSE本身不支持ping，所以我们检查最近是否收到任何消息
      const timeSinceLastActivity = Date.now() - (this.lastActivityTime || 0);

      // 放宽心跳检测条件：3分钟内没有活动才算丢失
      const heartbeatTimeoutMs = this.heartbeat.intervalMs * 3; // 180秒（3分钟）

      if (timeSinceLastActivity > heartbeatTimeoutMs) {
        this.heartbeat.missedBeats++;
        console.warn(`🔧 Heartbeat missed (${this.heartbeat.missedBeats}/${this.heartbeat.maxMissedBeats}) - ${Math.round(timeSinceLastActivity / 1000)}s since last activity`);

        if (this.heartbeat.missedBeats >= this.heartbeat.maxMissedBeats) {
          console.error('🔧 Too many missed heartbeats, reconnecting...');
          this.handleError(new Error('Heartbeat timeout'));
        }
      } else {
        // 重置错过的心跳计数
        if (this.heartbeat.missedBeats > 0) {
          console.log(`🔧 Heartbeat recovered - ${Math.round(timeSinceLastActivity / 1000)}s since last activity`);
          this.heartbeat.missedBeats = 0;
        }
      }
    }, 10000); // 每10秒检查一次
  }

  /**
   * 🔧 停止心跳检测
   */
  stopHeartbeat() {
    if (this.heartbeat.interval) {
      clearInterval(this.heartbeat.interval);
      this.heartbeat.interval = null;
    }
    this.heartbeat.missedBeats = 0;
  }

  /**
   * 🔧 处理页面可见性变化
   */
  handleVisibilityChange() {
    if (document.hidden) {
      // 页面隐藏 - 设置为away状态
      this.sendPresenceUpdate('away');
    } else {
      // 页面可见 - 设置为online状态
      this.sendPresenceUpdate('online');
    }
  }

  /**
   * 🔧 处理窗口获得焦点
   */
  handleWindowFocus() {
    this.sendPresenceUpdate('online');
  }

  /**
   * 🔧 处理窗口失去焦点
   */
  handleWindowBlur() {
    // 短暂延迟后设置为away，避免快速切换
    setTimeout(() => {
      if (!document.hasFocus()) {
        this.sendPresenceUpdate('away');
      }
    }, 1000);
  }

  /**
   * 🔧 处理页面卸载
   */
  handleBeforeUnload() {
    // 同步发送离线状态（使用beacon API避免被阻塞）
    const authStore = useAuthStore();
    if (authStore.token && navigator.sendBeacon) {
      try {
        const data = JSON.stringify({
          status: 'offline',
          timestamp: new Date().toISOString()
        });

        // 尝试使用beacon API发送离线状态
        navigator.sendBeacon('/api/realtime/presence', data);
      } catch (error) {
        console.warn('Failed to send offline status via beacon:', error);
      }
    }
  }

  /**
   * Get connection state
   */
  getConnectionState() {
    return {
      isConnected: this.isConnected,
      state: this.connectionState,
      reconnectAttempts: this.reconnectAttempts,
      latency: this.latency,
      connectionType: 'SSE',
      retryControl: {
        totalAttempts: this.retryControl.totalAttempts,
        maxTotalAttempts: this.retryControl.maxTotalAttempts,
        consecutiveFailures: this.retryControl.consecutiveFailures,
        maxConsecutiveFailures: this.retryControl.maxConsecutiveFailures,
        permanentFailure: this.retryControl.permanentFailure,
        remainingAttempts: Math.max(0, this.retryControl.maxTotalAttempts - this.retryControl.totalAttempts)
      },
      networkStatus: {
        isOnline: this.networkStatus.isOnline,
        lastOnlineTime: this.networkStatus.lastOnlineTime,
        offlineStartTime: this.networkStatus.offlineStartTime
      },
      heartbeat: {
        isActive: !!this.heartbeat.interval,
        missedBeats: this.heartbeat.missedBeats,
        maxMissedBeats: this.heartbeat.maxMissedBeats
      },
      longTermReconnect: {
        enabled: this.longTermReconnect.enabled,
        attempts: this.longTermReconnect.attempts,
        intervalMinutes: this.longTermReconnect.intervalMinutes
      }
    };
  }

  /**
   * 🔧 Start token refresh timer
   */
  startTokenRefreshTimer() {
    // Clear existing timer
    this.stopTokenRefreshTimer();

    // Set up periodic token refresh
    this.tokenRefreshTimer = setInterval(async () => {
      const authStore = useAuthStore();

      // Only refresh if connected and token is about to expire
      if (this.isConnected && authStore.shouldRefreshToken) {
        console.log('🔐 SSE: Refreshing token to maintain connection...');

        try {
          await authStore.refreshToken();
          console.log('✅ SSE: Token refreshed successfully during active connection');

          // Note: We cannot update the existing SSE connection's token
          // The connection will need to be re-established when current one fails
          // But at least we'll have a fresh token ready for reconnection
        } catch (error) {
          console.error('❌ SSE: Failed to refresh token during active connection:', error);
          // Don't disconnect immediately, wait for actual 401 error
        }
      }
    }, this.tokenRefreshInterval);

    console.log('⏰ SSE: Token refresh timer started (4 min interval)');
  }

  /**
   * 🔧 Stop token refresh timer
   */
  stopTokenRefreshTimer() {
    if (this.tokenRefreshTimer) {
      clearInterval(this.tokenRefreshTimer);
      this.tokenRefreshTimer = null;
      console.log('⏰ SSE: Token refresh timer stopped');
    }
  }
}

// Singleton instance
const realtimeCommunicationService = new RealtimeCommunicationService();

// Expose globally for health checks and debugging
if (typeof window !== 'undefined') {
  window.realtimeCommunicationService = realtimeCommunicationService;
}

export default realtimeCommunicationService; 