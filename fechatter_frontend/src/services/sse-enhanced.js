/**
 * Enhanced SSE Service - Production Grade Real-time Communication
 * Built with Rust backend compatibility and Vue 3 reactivity
 */

// 🚨 IMPORTANT: This service is temporarily disabled to prevent double message handling
// Only minimalSSE should be active to fix message duplication issues
const DISABLE_ENHANCED_SSE = true;

import { ref, computed } from 'vue';
import { useChatStore } from '@/stores/chat';
import { useAuthStore } from '@/stores/auth';
import { errorHandler } from '@/utils/errorHandler';
import { createSSEErrorHandler } from '@/composables/useSSEErrorSuppression';

// 智能重连策略配置
const RECONNECT_STRATEGIES = {
  NETWORK_ERROR: {
    initialDelay: 5000,      // 5秒起始 (vs 原来的1秒)
    maxDelay: 300000,        // 5分钟最大
    backoffFactor: 1.5,      // 温和增长
    maxAttempts: 8
  },
  AUTH_ERROR: {
    initialDelay: 30000,     // 30秒起始
    maxDelay: 600000,        // 10分钟最大  
    backoffFactor: 2.0,
    maxAttempts: 5
  },
  SERVER_ERROR: {
    initialDelay: 10000,     // 10秒起始
    maxDelay: 180000,        // 3分钟最大
    backoffFactor: 1.8,
    maxAttempts: 6
  },
  PROXY_ERROR: {            // Pingora相关错误
    initialDelay: 15000,     // 15秒起始
    maxDelay: 900000,        // 15分钟最大
    backoffFactor: 2.5,      // 快速增长
    maxAttempts: 4
  }
};

// 连接质量评估标准
const CONNECTION_QUALITY = {
  EXCELLENT: { threshold: 100, reconnectMultiplier: 0.5 },
  GOOD: { threshold: 500, reconnectMultiplier: 1.0 },
  FAIR: { threshold: 2000, reconnectMultiplier: 1.5 },
  POOR: { threshold: 5000, reconnectMultiplier: 3.0 }
};

class EnhancedRealtimeCommunicationService {
  constructor() {
    this.eventSource = null;
    this.isConnected = false;
    this.reconnectAttempts = 0;
    this.maxReconnectAttempts = 8;  // 减少最大短期重试次数
    this.reconnectDelay = 1000;
    this.messageQueue = [];
    this.eventListeners = new Map();

    // Connection state
    this.connectionState = 'disconnected';
    this.lastPingTime = null;
    this.latency = null;

    // Enhanced: 连接质量监控
    this.connectionQuality = 'GOOD';
    this.latencyHistory = [];
    this.errorHistory = [];
    this.connectionStartTime = null;
    this.connectionAttempts = 0;

    // Enhanced: 智能重连配置
    this.reconnectStrategies = RECONNECT_STRATEGIES;
    this.currentStrategy = null;
    this.lastErrorType = null;

    // 新增：增强的重试控制
    this.retryControl = {
      totalAttempts: 0,              // 总尝试次数
      maxTotalAttempts: 15,          // 增强版允许更多总尝试次数
      consecutiveFailures: 0,        // 连续失败次数
      maxConsecutiveFailures: 5,     // 增强版允许更多连续失败
      permanentFailure: false,       // 是否永久失败
      lastAttemptTime: null,         // 最后尝试时间
      errorTypeHistory: [],          // 错误类型历史
      qualityBasedRetries: true      // 基于连接质量的重试策略
    };

    // Enhanced: Pingora兼容性检测
    this.proxyType = 'unknown';
    this.sseEndpointFallbacks = [
      '/events',
      '/stream',
      '/api/events',
      '/realtime/events'
    ];
    this.fallbackIndex = 0;

    // 网络状态检测
    this.networkStatus = {
      isOnline: navigator.onLine,
      lastOnlineTime: Date.now(),
      offlineStartTime: null
    };

    // 心跳机制 - 优化间隔
    this.heartbeat = {
      interval: null,
      intervalMs: 30000, // 减少到30秒，匹配后端25秒keep-alive
      missedBeats: 0,
      maxMissedBeats: 3  // 增加到3次，提供更好的网络容错性（90秒总容忍时间）
    };

    // 长期重连策略
    this.longTermReconnect = {
      enabled: false,
      intervalMinutes: 10, // 增加到10分钟 (vs 原来的5分钟)
      maxIntervalMinutes: 60, // 增加到60分钟 (vs 原来的30分钟)
      attempts: 0,
      timeout: null
    };

    // 活动时间跟踪
    this.lastActivityTime = Date.now();

    // 增强的错误处理器
    this.errorHandler = createSSEErrorHandler({
      maxErrorsPerMinute: 1,      // 更严格：每分钟1个错误
      suppressionDurationMs: 300000, // 5分钟抑制期
      resetCounterMs: 900000      // 15分钟重置计数器
    });

    // 绑定事件处理器方法
    this.boundHandlers = {
      handleNetworkOnline: this.handleNetworkOnline.bind(this),
      handleNetworkOffline: this.handleNetworkOffline.bind(this),
      handleVisibilityChange: this.handleVisibilityChange.bind(this),
      handleWindowFocus: this.handleWindowFocus.bind(this),
      handleWindowBlur: this.handleWindowBlur.bind(this),
      handleBeforeUnload: this.handleBeforeUnload.bind(this)
    };

    // 设置网络和页面状态监听器
    if (typeof window !== 'undefined') {
      window.addEventListener('online', this.boundHandlers.handleNetworkOnline);
      window.addEventListener('offline', this.boundHandlers.handleNetworkOffline);
      document.addEventListener('visibilitychange', this.boundHandlers.handleVisibilityChange);
      window.addEventListener('focus', this.boundHandlers.handleWindowFocus);
      window.addEventListener('blur', this.boundHandlers.handleWindowBlur);
      window.addEventListener('beforeunload', this.boundHandlers.handleBeforeUnload);
    }

    /**
     * Enhanced: 智能错误分类
     */
    classifyError(error, response = null) {
      // HTTP状态码检测
      if (response?.status === 404) {
        return 'PROXY_ERROR'; // 可能是Pingora路由问题
        if (response?.status === 502 || response?.status === 503) {
          return 'PROXY_ERROR';
          if (response?.status === 401 || response?.status === 403) {
            return 'AUTH_ERROR';
            if (response?.status >= 500) {
              return 'SERVER_ERROR';
            }

            // 错误消息检测
            const message = error.message?.toLowerCase() || '';
            if (message.includes('network') || message.includes('connection')) {
              return 'NETWORK_ERROR';
              if (message.includes('stream closed') || message.includes('premature') ||
                message.includes('event stream')) {
                return 'PROXY_ERROR'; // Pingora相关
                if (message.includes('unauthorized') || message.includes('forbidden')) {
                  return 'AUTH_ERROR';
                }

                // EventSource特定错误检测
                if (error.target && error.target.readyState === EventSource.CLOSED) {
                  if (this.connectionAttempts > 3) {
                    return 'PROXY_ERROR'; // 多次连接失败，可能是代理问题
                    return 'NETWORK_ERROR';
                  }

                  return 'NETWORK_ERROR'; // 默认
                }

                /**
                 * Enhanced: 连接质量评估
                 */
                assessConnectionQuality() {
                  if (this.latencyHistory.length < 3) return;

                  const avgLatency = this.latencyHistory
                    .slice(-10)
                    .reduce((a, b) => a + b, 0) / Math.min(this.latencyHistory.length, 10);

                  const recentErrors = this.errorHistory
                    .filter(time => Date.now() - time < 300000); // 5分钟内

                  const errorRate = recentErrors.length / Math.max(this.connectionAttempts, 1);

                  // 连接成功率评估
                  const connectionDuration = this.connectionStartTime ?
                    Date.now() - this.connectionStartTime : 0;
                  const connectionStability = connectionDuration > 60000 ? 1.0 :
                    connectionDuration / 60000; // 连接稳定性因子

                  if (import.meta.env.DEV) {
                    console.log(`[SSE] Quality assessment: latency=${avgLatency.toFixed(0)}ms, errors=${recentErrors.length}, stability=${connectionStability.toFixed(2)}`);
                  }

                  if (avgLatency < 100 && errorRate < 0.1 && connectionStability > 0.8) {
                    this.connectionQuality = 'EXCELLENT';
                  } else if (avgLatency < 500 && errorRate < 0.3 && connectionStability > 0.5) {
                    this.connectionQuality = 'GOOD';
                  } else if (avgLatency < 2000 && errorRate < 0.5 && connectionStability > 0.2) {
                    this.connectionQuality = 'FAIR';
                  } else {
                    this.connectionQuality = 'POOR';
                  }

  /**
   * Enhanced: Pingora兼容性检测和端点选择
   */
  async detectProxyAndSelectEndpoint() {
                    if (import.meta.env.DEV) {
                      console.log('[SSE] Detecting proxy type and testing endpoints...');
                    }

                    for (let i = 0; i < this.sseEndpointFallbacks.length; i++) {
                      const endpoint = this.sseEndpointFallbacks[i];

                      try {
                        const startTime = Date.now();
                        const response = await fetch(endpoint, {
                          method: 'HEAD',
                          headers: {
                            'Accept': 'text/event-stream',
                            'Cache-Control': 'no-cache'
                          },
                          timeout: 5000
                        });

                        const latency = Date.now() - startTime;
                        this.latencyHistory.push(latency);

                        // 检测代理类型
                        const server = response.headers.get('server') || '';
                        const via = response.headers.get('via') || '';

                        if (server.includes('pingora') || via.includes('pingora')) {
                          this.proxyType = 'pingora';
                          if (import.meta.env.DEV) {
                            console.log('[SSE] Detected Pingora proxy');
                          }
                        } else if (server.includes('nginx')) {
                          this.proxyType = 'nginx';
                        } else if (server.includes('cloudflare')) {
                          this.proxyType = 'cloudflare';
                        }

                        // 检查端点可用性
                        if (response.ok || response.status === 401) {
                          if (import.meta.env.DEV) {
                            console.log(`[SSE] Found working endpoint: ${endpoint} (${latency}ms)`);
                            this.fallbackIndex = i;
                            return endpoint;
                          }

                          if (import.meta.env.DEV) {
                            console.log(`WARNING: [SSE] Endpoint ${endpoint} returned ${response.status}`);
                          }
                        } catch (error) {
                          if (import.meta.env.DEV) {
                            console.log(`ERROR: [SSE] Endpoint ${endpoint} failed:`, error.message);
                            continue;
                          }

                          // 如果所有SSE端点都失败，考虑WebSocket降级
                          if (import.meta.env.DEV) {
                            console.warn('🔄 [SSE] All SSE endpoints failed, considering WebSocket fallback');
                            return this.enableWebSocketFallback();
                          }

                          /**
                           * Enhanced: WebSocket降级方案
                           */
                          enableWebSocketFallback() {
                            if (import.meta.env.DEV) {
                              console.log('🔄 [SSE] WebSocket fallback not implemented yet');
                            }
                            // TODO: 实现WebSocket作为SSE的降级方案
                            return null;
                          }

  /**
   * Enhanced connection method with endpoint detection
   */
  async connect(token) {
                            if (DISABLE_ENHANCED_SSE) {
                              if (import.meta.env.DEV) {
                                console.log('🚨 Enhanced SSE service is disabled to prevent message duplication');
                                return Promise.resolve();
                              }

                              if (this.isConnected || this.connectionState === 'connecting') {
                                return;
                              }

                              this.retryControl.lastAttemptTime = Date.now();

                              try {
                                // 首先确保配置已加载
                                const { getApiConfig, getConfig, initializeConfig } = await import('@/utils/configLoader');

                                let config = getConfig();
                                if (!config) {
                                  if (import.meta.env.DEV) {
                                    console.log('SSE: Configuration not loaded yet, initializing...');
                                    await initializeConfig();
                                    config = getConfig();
                                  }

                                  const apiConfig = getApiConfig();

                                  // 检查是否禁用SSE连接（默认在开发环境下禁用）
                                  const shouldDisableSSE = apiConfig.disable_sse ||
                                    (config?.app?.environment === 'development' && apiConfig.disable_sse !== false);

                                  if (shouldDisableSSE) {
                                    if (import.meta.env.DEV) {
                                      console.log('[SSE] SSE连接已在开发模式下禁用，使用模拟模式');
                                      this.isConnected = true;
                                      this.connectionState = 'connected';
                                      this.emit('connected');
                                      // 启动模拟心跳以避免错误
                                      this.startMockHeartbeat();
                                      return;
                                    }

                                    if (import.meta.env.DEV) {
                                      console.log('🔌 [SSE] SSE连接已启用，正在建立连接...');
                                    }
                                  } catch (configError) {
                                    if (import.meta.env.DEV) {
                                      console.error('[SSE] 配置加载失败，默认禁用SSE:', configError);
                                      this.isConnected = true;
                                      this.connectionState = 'connected';
                                      this.emit('connected');
                                      this.startMockHeartbeat();
                                      return;
                                    }

                                    this.connectionState = 'connecting';
                                    this.connectionAttempts++;
                                    this.connectionStartTime = Date.now();

                                    try {
                                      // Import config loader and ensure config is loaded
                                      const { getApiConfig, getConfig, initializeConfig } = await import('@/utils/configLoader');

                                      let config = getConfig();
                                      if (!config) {
                                        if (import.meta.env.DEV) {
                                          console.log('SSE: Configuration not loaded yet, initializing...');
                                          await initializeConfig();
                                          config = getConfig();
                                        }

                                        const apiConfig = getApiConfig();

                                        // Enhanced: 智能端点选择
                                        let sseUrl;
                                        if (this.fallbackIndex === 0) {
                                          // 首次连接或检测端点
                                          const detectedEndpoint = await this.detectProxyAndSelectEndpoint();
                                          if (!detectedEndpoint) {
                                            throw new Error('No working SSE endpoint found');
                                            sseUrl = `${apiConfig.gateway_url || 'http://127.0.0.1:8080'}${detectedEndpoint}`;
                                          } else {
                                            // 使用之前检测到的端点
                                            const endpoint = this.sseEndpointFallbacks[this.fallbackIndex];
                                            sseUrl = `${apiConfig.gateway_url || 'http://127.0.0.1:8080'}${endpoint}`;
                                          }

                                          // Ensure token is valid
                                          if (!token || typeof token !== 'string') {
                                            throw new Error('Invalid authentication token provided for SSE connection');
                                          }

                                          // Build complete SSE URL with proper token encoding
                                          const fullSseUrl = `${sseUrl}?access_token=${encodeURIComponent(token)}`;

                                          if (import.meta.env.DEV) {
                                            console.log(`🔌 SSE: Connecting to ${sseUrl} (attempt ${this.connectionAttempts})`);
                                          }

                                          this.eventSource = new EventSource(fullSseUrl);

                                          this.eventSource.onopen = this.handleOpen.bind(this);
                                          this.eventSource.onmessage = this.handleMessage.bind(this);
                                          this.eventSource.onerror = this.handleError.bind(this);

                                          // Listen for specific event types
                                          this.setupEventListeners();

                                        } catch (error) {
                                          if (import.meta.env.DEV) {
                                            console.error('🔌 SSE: Connection setup failed:', error);
                                          }

                                          const errorType = this.classifyError(error);
                                          this.lastErrorType = errorType;

                                          this.errorHandler.handleConnectionError(error);

                                          this.connectionState = 'disconnected';
                                          this.scheduleIntelligentReconnect(errorType, error);
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
                                              this.errorHandler.handleMessageError(error, 'NewMessage');
                                            }
                                          });

                                          // Typing status event
                                          this.eventSource.addEventListener('TypingStatus', (event) => {
                                            try {
                                              const data = JSON.parse(event.data);
                                              this.handleTypingStatus(data);
                                            } catch (error) {
                                              this.errorHandler.handleMessageError(error, 'TypingStatus');
                                            }
                                          });

                                          // Message read event
                                          this.eventSource.addEventListener('MessageRead', (event) => {
                                            try {
                                              const data = JSON.parse(event.data);
                                              this.handleMessageStatus(data);
                                            } catch (error) {
                                              this.errorHandler.handleMessageError(error, 'MessageRead');
                                            }
                                          });

                                          // User presence event
                                          this.eventSource.addEventListener('UserPresence', (event) => {
                                            try {
                                              const data = JSON.parse(event.data);
                                              this.handleUserPresence(data);
                                            } catch (error) {
                                              this.errorHandler.handleMessageError(error, 'UserPresence');
                                            }
                                          });

                                          // Chat member events
                                          this.eventSource.addEventListener('UserJoinedChat', (event) => {
                                            try {
                                              const data = JSON.parse(event.data);
                                              this.emit('user_joined_chat', data);
                                            } catch (error) {
                                              this.errorHandler.handleMessageError(error, 'UserJoinedChat');
                                            }
                                          });

                                          this.eventSource.addEventListener('UserLeftChat', (event) => {
                                            try {
                                              const data = JSON.parse(event.data);
                                              this.emit('user_left_chat', data);
                                            } catch (error) {
                                              this.errorHandler.handleMessageError(error, 'UserLeftChat');
                                            }
                                          });

                                          /**
                                           * Handle connection open
                                           */
                                          handleOpen(event) {
                                            this.isConnected = true;
                                            this.connectionState = 'connected';
                                            this.reconnectAttempts = 0;
                                            this.reconnectDelay = 1000;
                                            this.lastActivityTime = Date.now();

                                            // 重置重试控制
                                            this.retryControl.consecutiveFailures = 0;
                                            if (import.meta.env.DEV) {
                                              console.log(`[SSE Enhanced] Connected successfully (Total attempts: ${this.retryControl.totalAttempts})`);
                                            }

                                            // 记录连接成功的延迟
                                            if (this.connectionStartTime) {
                                              const connectionLatency = Date.now() - this.connectionStartTime;
                                              this.latencyHistory.push(connectionLatency);
                                              if (import.meta.env.DEV) {
                                                console.log(`[SSE] Connected successfully in ${connectionLatency}ms`);
                                              }

                                              // 重置长期重连策略
                                              this.resetLongTermReconnect();

                                              // 启动心跳检测
                                              this.startHeartbeat();

                                              // Send presence update on connection
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
                                                this.errorHandler.handleMessageError(error, 'GenericMessage');
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
                                                  realtime: true,
                                                  status: 'delivered' // SSE messages are delivered by definition
                                                };

                                                // CRITICAL FIX: Check if this is a delivery confirmation for our own message
                                                const authStore = useAuthStore();
                                                const isOwnMessage = formattedMessage.sender_id === authStore.user?.id;

                                                if (isOwnMessage) {
                                                  // Try to update existing message status first (delivery confirmation)
                                                  const updated = chatStore.updateRealtimeMessage(formattedMessage.id, {
                                                    status: 'delivered',
                                                    delivered_at: formattedMessage.created_at,
                                                    server_id: formattedMessage.id
                                                  });

                                                  if (updated) {
                                                    if (import.meta.env.DEV) {
                                                      console.log(`[SSE] Own message ${formattedMessage.id} marked as delivered via SSE`);
                                                    }
                                                    // Don't add as new message if we successfully updated existing one
                                                    this.emit('message_delivered', formattedMessage);
                                                    return;
                                                  }
                                                }

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
                                                      this.errorHandler.handleGeneralError(error, 'Mark message as read');
                                                    });
                                                }).catch(err => {
                                                  this.errorHandler.handleGeneralError(err, 'Import API module for read receipt');
                                                });

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
   * Enhanced Error handler with intelligent classification
   */
  async handleError(error) {
                                                    // 更新重试控制
                                                    this.retryControl.totalAttempts++;
                                                    this.retryControl.consecutiveFailures++;

                                                    const errorType = this.classifyError(error);
                                                    this.lastErrorType = errorType;
                                                    this.errorHistory.push(Date.now());
                                                    this.retryControl.errorTypeHistory.push(errorType);

                                                    if (import.meta.env.DEV) {
                                                      console.error(`🔌 SSE Enhanced: Connection error (Attempt ${this.retryControl.totalAttempts}/${this.retryControl.maxTotalAttempts}, ` +
    }
                                                    `Consecutive failures: ${this.retryControl.consecutiveFailures}/${this.retryControl.maxConsecutiveFailures})`,
                                                      { type: errorType, error });

                                                    // 先检查是否达到永久失败条件
                                                    if (this.retryControl.totalAttempts >= this.retryControl.maxTotalAttempts ||
                                                      this.retryControl.consecutiveFailures >= this.retryControl.maxConsecutiveFailures) {
                                                      this.retryControl.permanentFailure = true;
                                                      if (import.meta.env.DEV) {
                                                        console.error('🔌 SSE Enhanced: Maximum retry attempts reached, connection permanently failed');
                                                      }

                                                      // 立即关闭EventSource以防止自动重连
                                                      if (this.eventSource) {
                                                        this.eventSource.close();
                                                        this.eventSource = null;
                                                      }

                                                      // 使用增强的错误处理器处理永久失败（需要导入）
                                                      try {
                                                        const { errorHandler: importedErrorHandler } = await import('@/utils/errorHandler');
                                                        importedErrorHandler.handleSSERetryError(error, {
                                                          totalAttempts: this.retryControl.totalAttempts,
                                                          maxTotalAttempts: this.retryControl.maxTotalAttempts,
                                                          consecutiveFailures: this.retryControl.consecutiveFailures,
                                                          maxConsecutiveFailures: this.retryControl.maxConsecutiveFailures,
                                                          errorTypeHistory: this.retryControl.errorTypeHistory,
                                                          context: 'SSE Enhanced Service'
                                                        });
                                                      } catch (importError) {
                                                        if (import.meta.env.DEV) {
                                                          console.warn('Failed to import enhanced error handler for permanent failure');
                                                        }

                                                        this.isConnected = false;
                                                        this.connectionState = 'permanently_failed';
                                                        this.stopHeartbeat();
                                                        this.emit('permanently_failed', {
                                                          totalAttempts: this.retryControl.totalAttempts,
                                                          consecutiveFailures: this.retryControl.consecutiveFailures,
                                                          errorTypeHistory: this.retryControl.errorTypeHistory
                                                        });
                                                        return; // 立即返回，不再尝试重连
                                                      }

                                                      // 如果还没达到限制，使用内部错误处理器
                                                      const wasLogged = this.errorHandler.handleConnectionError(error);

                                                      if (wasLogged) {
                                                        if (import.meta.env.DEV) {
                                                          console.log(`🔌 [SSE] Error classified as: ${errorType}`);
                                                        }

                                                        this.isConnected = false;
                                                        this.connectionState = 'disconnected';

                                                        // 停止心跳检测
                                                        this.stopHeartbeat();

                                                        // Emit disconnected event
                                                        this.emit('disconnected', { error, errorType });

                                                        // 使用智能重连策略
                                                        this.scheduleIntelligentReconnect(errorType, error);
                                                      }

                                                      /**
                                                       * Enhanced: 智能重连调度
                                                       */
                                                      scheduleIntelligentReconnect(errorType, error) {
                                                        // 检查是否已永久失败
                                                        if (this.retryControl.permanentFailure) {
                                                          if (import.meta.env.DEV) {
                                                            console.warn('🔌 SSE Enhanced: Skipping reconnect due to permanent failure');
                                                            return;
                                                          }

                                                          // 检查网络状态 - 如果离线则暂停重连
                                                          if (!this.networkStatus.isOnline) {
                                                            if (import.meta.env.DEV) {
                                                              console.log('🔌 [SSE] Network offline, pausing reconnection');
                                                              return;
                                                            }

                                                            this.currentStrategy = this.reconnectStrategies[errorType];
                                                            this.assessConnectionQuality();

                                                            const qualityMultiplier = CONNECTION_QUALITY[this.connectionQuality].reconnectMultiplier;

                                                            // 计算基础延迟
                                                            const baseDelay = Math.min(
                                                              this.currentStrategy.initialDelay * Math.pow(
                                                                this.currentStrategy.backoffFactor,
                                                                this.reconnectAttempts
                                                              ),
                                                              this.currentStrategy.maxDelay
                                                            );

                                                            // 应用连接质量调整
                                                            const adjustedDelay = Math.floor(baseDelay * qualityMultiplier);

                                                            if (import.meta.env.DEV) {
                                                              console.log(`🔄 [SSE] Smart reconnect scheduled: ${errorType}, quality: ${this.connectionQuality}, delay: ${Math.round(adjustedDelay / 1000)}s (attempt ${this.reconnectAttempts + 1}/${this.currentStrategy.maxAttempts})`);
                                                            }

                                                            // 检查是否超过最大重试次数
                                                            if (this.reconnectAttempts >= this.currentStrategy.maxAttempts) {
                                                              if (import.meta.env.DEV) {
                                                                console.warn('🔌 [SSE] Max reconnection attempts reached, switching to long-term strategy');
                                                                this.enableLongTermReconnect();
                                                                return;
                                                              }

                                                              this.connectionState = 'reconnecting';
                                                              this.reconnectAttempts++;

                                                              setTimeout(() => {
                                                                // 在执行重连时再次检查网络状态
                                                                if (!this.networkStatus.isOnline) {
                                                                  if (import.meta.env.DEV) {
                                                                    console.log('🔌 [SSE] Network went offline during reconnect delay');
                                                                    return;
                                                                  }

                                                                  const authStore = useAuthStore();
                                                                  if (authStore.token) {
                                                                    this.connect(authStore.token);
                                                                  }
                                                                }, adjustedDelay);
                                                            }

                                                            /**
                                                             * 启用长期重连策略（分钟级别）
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
                                                             * 安排长期重连
                                                             */
                                                            scheduleLongTermReconnect() {
                                                              if (!this.longTermReconnect.enabled) {
                                                                return;
                                                              }

                                                              this.longTermReconnect.attempts++;
                                                              const intervalMs = this.longTermReconnect.intervalMinutes * 60 * 1000;

                                                              if (import.meta.env.DEV) {
                                                                console.log(`🕐 [SSE] Long-term reconnect scheduled in ${this.longTermReconnect.intervalMinutes} minutes (attempt ${this.longTermReconnect.attempts})`);
                                                              }

                                                              this.longTermReconnect.timeout = setTimeout(() => {
                                                                if (!this.isConnected && this.longTermReconnect.enabled) {
                                                                  const authStore = useAuthStore();
                                                                  if (authStore.token) {
                                                                    // 临时重置短期重试计数器以尝试连接
                                                                    const originalAttempts = this.reconnectAttempts;
                                                                    this.reconnectAttempts = 0;
                                                                    this.reconnectDelay = this.currentStrategy?.initialDelay || 5000;

                                                                    this.connect(authStore.token).then(() => {
                                                                      // 连接成功会通过 handleOpen 重置状态
                                                                    }).catch(() => {
                                                                      // 连接失败，恢复原来的状态并继续长期重连
                                                                      this.reconnectAttempts = originalAttempts;
                                                                      this.increaseLongTermInterval();
                                                                      this.scheduleLongTermReconnect();
                                                                    });
                                                                  }, intervalMs);
                                                            }

                                                            /**
                                                             * 增加长期重连间隔
                                                             */
                                                            increaseLongTermInterval() {
                                                              const currentInterval = this.longTermReconnect.intervalMinutes;
                                                              // 逐步增加间隔：10分钟 -> 20分钟 -> 30分钟 -> 60分钟（最大）
                                                              if (currentInterval < 20) {
                                                                this.longTermReconnect.intervalMinutes = 20;
                                                              } else if (currentInterval < 30) {
                                                                this.longTermReconnect.intervalMinutes = 30;
                                                              } else if (currentInterval < 60) {
                                                                this.longTermReconnect.intervalMinutes = 60;
                                                              }

                                                              /**
                                                               * 重置长期重连策略
                                                               */
                                                              resetLongTermReconnect() {
                                                                if (this.longTermReconnect.timeout) {
                                                                  clearTimeout(this.longTermReconnect.timeout);
                                                                  this.longTermReconnect.timeout = null;
                                                                }

                                                                this.longTermReconnect.enabled = false;
                                                                this.longTermReconnect.attempts = 0;
                                                                this.longTermReconnect.intervalMinutes = 10; // 重置为10分钟
                                                              }

  /**
   * Send presence update via HTTP API
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

                                                                  if (import.meta.env.DEV) {
                                                                    console.log(`Presence updated to: ${status}`);
                                                                  }
                                                                } catch (error) {
                                                                  // Don't use errorHandler here to avoid circular dependency
                                                                  if (import.meta.env.DEV) {
                                                                    console.warn('Failed to send presence update:', error.message);
                                                                  }

                                                                  // For development/testing, emit a fake presence event
                                                                  if (import.meta.env.DEV) {
                                                                    const authStore = useAuthStore();
                                                                    if (authStore.user) {
                                                                      this.emit('user_presence', {
                                                                        user_id: authStore.user.id,
                                                                        status,
                                                                        timestamp: new Date().toISOString(),
                                                                        last_seen: new Date().toISOString()
                                                                      });

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

                                                                          /**
                                                                           * Disconnect
                                                                           */
                                                                          /**
                                                                           * Reset retry control
                                                                           */
                                                                          resetRetryControl() {
                                                                            this.retryControl = {
                                                                              totalAttempts: 0,
                                                                              maxTotalAttempts: 15,
                                                                              consecutiveFailures: 0,
                                                                              maxConsecutiveFailures: 5,
                                                                              permanentFailure: false,
                                                                              lastAttemptTime: null,
                                                                              errorTypeHistory: [],
                                                                              qualityBasedRetries: true
                                                                            };
                                                                            this.reconnectAttempts = 0;
                                                                            this.reconnectDelay = 1000;
                                                                            if (import.meta.env.DEV) {
                                                                              console.log('🔌 SSE Enhanced: Retry control reset');
                                                                            }

                                                                            disconnect() {
                                                                              // Send offline status before disconnecting
                                                                              this.sendPresenceUpdate('offline');

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

                                                                              // 停止心跳检测
                                                                              this.stopHeartbeat();

                                                                              // 清理长期重连策略
                                                                              this.resetLongTermReconnect();

                                                                              // 清理事件监听器
                                                                              if (typeof window !== 'undefined' && this.boundHandlers) {
                                                                                window.removeEventListener('online', this.boundHandlers.handleNetworkOnline);
                                                                                window.removeEventListener('offline', this.boundHandlers.handleNetworkOffline);
                                                                                document.removeEventListener('visibilitychange', this.boundHandlers.handleVisibilityChange);
                                                                                window.removeEventListener('focus', this.boundHandlers.handleWindowFocus);
                                                                                window.removeEventListener('blur', this.boundHandlers.handleWindowBlur);
                                                                                window.removeEventListener('beforeunload', this.boundHandlers.handleBeforeUnload);
                                                                              }

                                                                              /**
                                                                               * Add event listener
                                                                               */
                                                                              on(event, callback) {
                                                                                if (!this.eventListeners.has(event)) {
                                                                                  this.eventListeners.set(event, new Set());
                                                                                  this.eventListeners.get(event).add(callback);
                                                                                }

                                                                                /**
                                                                                 * Remove event listener
                                                                                 */
                                                                                off(event, callback) {
                                                                                  if (this.eventListeners.has(event)) {
                                                                                    this.eventListeners.get(event).delete(callback);
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
                                                                                          this.errorHandler.handleGeneralError(error, `Event callback for ${event}`);
                                                                                        }
                                                                                      });

                                                                                      /**
                                                                                       * 网络状态处理：网络恢复时
                                                                                       */
                                                                                      handleNetworkOnline() {
                                                                                        this.networkStatus.isOnline = true;
                                                                                        this.networkStatus.lastOnlineTime = Date.now();

                                                                                        if (this.networkStatus.offlineStartTime) {
                                                                                          const offlineDuration = Date.now() - this.networkStatus.offlineStartTime;
                                                                                          if (import.meta.env.DEV) {
                                                                                            console.log(`🌐 [SSE] Network back online after ${Math.round(offlineDuration / 1000)}s`);
                                                                                            this.networkStatus.offlineStartTime = null;
                                                                                          }

                                                                                          // 网络恢复时，使用智能重连策略
                                                                                          if (!this.isConnected) {
                                                                                            const errorType = this.lastErrorType || 'NETWORK_ERROR';
                                                                                            this.scheduleIntelligentReconnect(errorType, new Error('Network restored'));
                                                                                          }

                                                                                          /**
                                                                                           * 网络状态处理：网络断开时
                                                                                           */
                                                                                          handleNetworkOffline() {
                                                                                            this.networkStatus.isOnline = false;
                                                                                            this.networkStatus.offlineStartTime = Date.now();
                                                                                            if (import.meta.env.DEV) {
                                                                                              console.log('🌐 [SSE] Network went offline');
                                                                                            }

                                                                                            // 停止心跳检测
                                                                                            this.stopHeartbeat();

                                                                                            // 暂停重连尝试
                                                                                            if (this.longTermReconnect.timeout) {
                                                                                              clearTimeout(this.longTermReconnect.timeout);
                                                                                              this.longTermReconnect.timeout = null;
                                                                                            }

                                                                                            /**
                                                                                             * 启动心跳检测 - 优化版本
                                                                                             */
                                                                                            startHeartbeat() {
                                                                                              if (this.heartbeat.interval) {
                                                                                                clearInterval(this.heartbeat.interval);
                                                                                              }

                                                                                              this.heartbeat.missedBeats = 0;
                                                                                              this.heartbeat.interval = setInterval(() => {
                                                                                                if (!this.isConnected) {
                                                                                                  this.stopHeartbeat();
                                                                                                  return;
                                                                                                }

                                                                                                // 检查连接是否仍然活跃
                                                                                                const timeSinceLastActivity = Date.now() - (this.lastActivityTime || 0);
                                                                                                if (timeSinceLastActivity > this.heartbeat.intervalMs * 1.5) {
                                                                                                  this.heartbeat.missedBeats++;
                                                                                                  if (import.meta.env.DEV) {
                                                                                                    console.warn(`💔 [SSE] Heartbeat missed (${this.heartbeat.missedBeats}/${this.heartbeat.maxMissedBeats}), last activity: ${Math.round(timeSinceLastActivity / 1000)}s ago`);
                                                                                                  }

                                                                                                  if (this.heartbeat.missedBeats >= this.heartbeat.maxMissedBeats) {
                                                                                                    if (import.meta.env.DEV) {
                                                                                                      console.error('💔 [SSE] Too many missed heartbeats, reconnecting...');
                                                                                                      this.handleError(new Error('Heartbeat timeout'));
                                                                                                    }
                                                                                                  } else {
                                                                                                    this.heartbeat.missedBeats = 0;
                                                                                                  }
                                                                                                }, this.heartbeat.intervalMs);
                                                                                            }

                                                                                            /**
                                                                                             * 停止心跳检测
                                                                                             */
                                                                                            stopHeartbeat() {
                                                                                              if (this.heartbeat.interval) {
                                                                                                clearInterval(this.heartbeat.interval);
                                                                                                this.heartbeat.interval = null;
                                                                                                this.heartbeat.missedBeats = 0;
                                                                                              }

                                                                                              /**
                                                                                               * 处理页面可见性变化
                                                                                               */
                                                                                              handleVisibilityChange() {
                                                                                                // Occam's Razor: Don't change presence based on tab visibility
                                                                                                // Modern chat apps don't mark users as away just for switching tabs

                                                                                                // Old logic that was too aggressive:
                                                                                                // if (document.hidden) {
                                                                                                //   this.sendPresenceUpdate('away');
                                                                                                // } else {
                                                                                                //   this.sendPresenceUpdate('online');
                                                                                                // }

                                                                                                // Only check connection when page becomes visible
                                                                                                if (!document.hidden) {
                                                                                                  // 页面重新可见时，检查连接状态
                                                                                                  if (!this.isConnected && this.networkStatus.isOnline) {
                                                                                                    if (import.meta.env.DEV) {
                                                                                                      console.log('👁️ [SSE] Page visible again, checking connection...');
                                                                                                      const errorType = this.lastErrorType || 'NETWORK_ERROR';
                                                                                                      this.scheduleIntelligentReconnect(errorType, new Error('Page visibility changed'));
                                                                                                    }

                                                                                                    /**
                                                                                                     * 处理窗口获得焦点
                                                                                                     */
                                                                                                    handleWindowFocus() {
                                                                                                      // Keep user online when window gains focus
                                                                                                      this.sendPresenceUpdate('online');
                                                                                                    }

                                                                                                    /**
                                                                                                     * 处理窗口失去焦点
                                                                                                     */
                                                                                                    handleWindowBlur() {
                                                                                                      // Occam's Razor: Don't set away when user switches windows
                                                                                                      // They might be multitasking or referencing other documents

                                                                                                      // Disabled aggressive away detection:
                                                                                                      // setTimeout(() => {
                                                                                                      //   if (!document.hasFocus()) {
                                                                                                      //     this.sendPresenceUpdate('away');
                                                                                                      //   }
                                                                                                      // }, 2000);
                                                                                                    }

                                                                                                    /**
                                                                                                     * 处理页面卸载
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
                                                                                                          if (import.meta.env.DEV) {
                                                                                                            console.warn('Failed to send offline status via beacon:', error);
                                                                                                          }

                                                                                                          /**
                                                                                                           * Enhanced: Get connection state with quality metrics
                                                                                                           */
                                                                                                          getConnectionState() {
                                                                                                            return {
                                                                                                              isConnected: this.isConnected,
                                                                                                              state: this.connectionState,
                                                                                                              reconnectAttempts: this.reconnectAttempts,
                                                                                                              latency: this.latency,
                                                                                                              connectionType: 'SSE Enhanced',
                                                                                                              proxyType: this.proxyType,
                                                                                                              fallbackIndex: this.fallbackIndex,
                                                                                                              connectionQuality: this.connectionQuality,
                                                                                                              lastErrorType: this.lastErrorType,
                                                                                                              avgLatency: this.latencyHistory.length > 0 ?
                                                                                                                this.latencyHistory.reduce((a, b) => a + b, 0) / this.latencyHistory.length : 0,
                                                                                                              errorCount: this.errorHistory.filter(time => Date.now() - time < 300000).length,
                                                                                                              networkStatus: {
                                                                                                                isOnline: this.networkStatus.isOnline,
                                                                                                                lastOnlineTime: this.networkStatus.lastOnlineTime,
                                                                                                                offlineStartTime: this.networkStatus.offlineStartTime
                                                                                                              },
                                                                                                              heartbeat: {
                                                                                                                isActive: !!this.heartbeat.interval,
                                                                                                                missedBeats: this.heartbeat.missedBeats,
                                                                                                                maxMissedBeats: this.heartbeat.maxMissedBeats,
                                                                                                                intervalMs: this.heartbeat.intervalMs
                                                                                                              },
                                                                                                              longTermReconnect: {
                                                                                                                enabled: this.longTermReconnect.enabled,
                                                                                                                attempts: this.longTermReconnect.attempts,
                                                                                                                intervalMinutes: this.longTermReconnect.intervalMinutes
                                                                                                              }
                                                                                                            };
                                                                                                          }

                                                                                                          /**
                                                                                                           * Enhanced: Debug information
                                                                                                           */
                                                                                                          getDebugInfo() {
                                                                                                            return {
                                                                                                              ...this.getConnectionState(),
                                                                                                              latencyHistory: this.latencyHistory.slice(-20), // 最近20次延迟
                                                                                                              errorHistory: this.errorHistory.slice(-10).map(time => new Date(time).toISOString()),
                                                                                                              currentStrategy: this.currentStrategy,
                                                                                                              sseEndpointFallbacks: this.sseEndpointFallbacks,
                                                                                                              connectionAttempts: this.connectionAttempts
                                                                                                            };
                                                                                                          }

                                                                                                          /**
                                                                                                           * 模拟心跳 - 用于开发模式下SSE禁用时
                                                                                                           */
                                                                                                          startMockHeartbeat() {
                                                                                                            if (import.meta.env.DEV) {
                                                                                                              console.log('[SSE] 启动模拟心跳 (开发模式)');
                                                                                                            }

                                                                                                            // 模拟定期更新活动时间
                                                                                                            setInterval(() => {
                                                                                                              this.lastActivityTime = Date.now();
                                                                                                              if (import.meta.env.DEV) {
                                                                                                                console.log('HEARTBEAT: [SSE] 模拟心跳 - 连接正常');
                                                                                                              }
                                                                                                            }, 10000); // 每10秒更新一次

                                                                                                            // 启动正常的心跳检测（但由于活动时间会被更新，不会触发重连）
                                                                                                            this.startHeartbeat();
                                                                                                          }

                                                                                                          // Singleton instance
                                                                                                          const enhancedRealtimeCommunicationService = new EnhancedRealtimeCommunicationService();

                                                                                                          // Expose globally for health checks and debugging
                                                                                                          if (typeof window !== 'undefined') {
                                                                                                            window.enhancedSSE = enhancedRealtimeCommunicationService;
                                                                                                          }

                                                                                                          export default enhancedRealtimeCommunicationService; 