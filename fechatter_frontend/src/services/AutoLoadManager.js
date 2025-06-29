/**
 * AutoLoadManager - Reliable historical message auto-load manager
 * 
 * Ensures complete lifecycle management for "Loading earlier messages..." functionality:
 * 1. Automatically detect and load historical messages
 * 2. Guarantee to stop after all messages are loaded
 * 3. Provide user-friendly completion notifications
 * 4. Prevent users from continuing to attempt refresh
 */

export class AutoLoadManager {
  constructor() {
    this.states = {
      IDLE: 'idle',
      DETECTING_NEED: 'detecting_need',
      LOADING: 'loading',
      PROCESSING: 'processing',
      ALL_LOADED: 'all_loaded',
      COMPLETED: 'completed',
      ERROR: 'error',
      DISABLED: 'disabled',
      WAITING_FOR_SCROLL: 'waiting_for_scroll' // Waiting for user scroll state
    };

    this.currentState = this.states.IDLE;
    this.chatId = null;
    this.loadCallback = null;
    this.messageCount = 0;
    this.totalLoadedMessages = 0;
    this.loadingAttempts = 0;
    this.maxRetries = 3;
    this.isActive = false;

    // Loading mode configuration (default scroll-triggered mode)
    this.loadingMode = 'scroll-triggered'; // 'auto' | 'scroll-triggered'
    this.waitingForScroll = false;
    this.scrollTriggerCallback = null;
    this.batchSize = 20; // Default 20 messages per batch
    this.userControlEnabled = true; // User control switch

    // User interaction state
    this.userNotified = false;
    this.completionShown = false;
    this.userDismissed = false;

    // Performance and debugging
    this.startTime = 0;
    this.metrics = {
      totalSessions: 0,
      successfulSessions: 0,
      totalMessagesLoaded: 0,
      averageLoadTime: 0,
      errorCount: 0
    };

    // Event system
    this.eventListeners = new Map();

    if (import.meta.env.DEV) {
      console.log('[AutoLoadManager] Initialization complete');
    }
  }

  /**
   * Start auto-load session
   */
  async startAutoLoadSession({
    chatId,
    hasMoreMessages,
    loadCallback,
    onProgress = null,
    onComplete = null,
    onError = null,
    // Loading mode configuration (default scroll-triggered mode)
    loadingMode = 'scroll-triggered', // 'auto' | 'scroll-triggered'
    onScrollNeeded = null, // Callback for scroll-triggered mode
    batchSize = 20, // Number of messages per batch (performance control)
    userControlEnabled = true // User control switch
  }) {
    // Prevent duplicate activation
    if (this.isActive && this.chatId === chatId) {
      if (import.meta.env.DEV) {
        console.warn('[AutoLoadManager] Session already active, ignoring duplicate start');
      }
      return false;
    }

    // Reset state
    this.reset();
    this.chatId = chatId;
    this.loadCallback = loadCallback;
    this.isActive = true;
    this.startTime = Date.now();
    this.metrics.totalSessions++;

    // Set loading mode and user control configuration
    this.loadingMode = loadingMode;
    this.scrollTriggerCallback = onScrollNeeded;
    this.waitingForScroll = false;
    this.batchSize = batchSize;
    this.userControlEnabled = userControlEnabled;

    // Register callbacks
    if (onProgress) this.on('progress', onProgress);
    if (onComplete) this.on('complete', onComplete);
    if (onError) this.on('error', onError);

    if (import.meta.env.DEV) {
      console.log(`[AutoLoadManager] Starting auto-load session - Chat ${chatId}`);
    }

    try {
      // Step 1: Detect if loading is needed
      await this.detectLoadingNeed(hasMoreMessages);

      // Step 2: Start loading loop
      if (this.currentState === this.states.LOADING) {
        await this.executeLoadingLoop();
      }

      return this.currentState === this.states.COMPLETED;

    } catch (error) {
      console.error('[AutoLoadManager] Auto-load session failed:', error);
      this.transitionTo(this.states.ERROR);
      this.emit('error', { error: error.message, chatId });
      return false;
    }
  }

  /**
   * Phase 1: Detect loading need
   */
  async detectLoadingNeed(hasMoreMessages) {
    this.transitionTo(this.states.DETECTING_NEED);

    if (!hasMoreMessages) {
      if (import.meta.env.DEV) {
        console.log('[AutoLoadManager] No more messages detected, completing directly');
      }
      this.transitionTo(this.states.ALL_LOADED);
      await this.handleAllLoaded();
      return;
    }

    if (import.meta.env.DEV) {
      console.log('[AutoLoadManager] More messages detected, starting load');
    }
    this.transitionTo(this.states.LOADING);
  }

  /**
   * Phase 2: Execute loading loop
   */
  async executeLoadingLoop() {
    while (this.currentState === this.states.LOADING && this.isActive) {
      try {
        // Execute single load
        const result = await this.performSingleLoad();

        if (!result.success) {
          throw new Error(result.error || 'Load failed');
        }

        // Process load result
        await this.processLoadResult(result);

        // Check if continuation is needed
        if (!result.hasMore) {
          break;
        }

        // Decide whether to continue based on loading mode
        if (this.loadingMode === 'scroll-triggered') {
          // Scroll-triggered mode: wait for user to scroll to top
          await this.waitForUserScroll();

          // If user doesn't continue scrolling or session is cancelled, exit loop
          if (!this.isActive || this.currentState !== this.states.LOADING) {
            break;
          }
        } else {
          // Auto mode: continue loading directly
          await this.wait(100); // Avoid rapid consecutive requests
        }

      } catch (error) {
        this.loadingAttempts++;

        if (this.loadingAttempts >= this.maxRetries) {
          throw error;
        }

        if (import.meta.env.DEV) {
          console.warn(`[AutoLoadManager] Load failed, retrying ${this.loadingAttempts}/${this.maxRetries}:`, error);
        }

        await this.wait(1000 * this.loadingAttempts); // Incremental delay
      }
    }
  }

  /**
   * Execute single load
   */
  async performSingleLoad() {
    if (!this.loadCallback) {
      throw new Error('Load callback not set');
    }

    this.emit('progress', {
      chatId: this.chatId,
      phase: 'loading',
      attempt: this.loadingAttempts + 1
    });

    if (import.meta.env.DEV) {
      console.log(`[AutoLoadManager] Executing load attempt ${this.loadingAttempts + 1}`);
    }

    // Call external load function
    const result = await this.loadCallback();

    // Validate result format
    if (!this.isValidLoadResult(result)) {
      throw new Error('Load callback returned invalid result format');
    }

    return result;
  }

  /**
   * Process load result
   */
  async processLoadResult(result) {
    this.transitionTo(this.states.PROCESSING);

    const { messages = [], hasMore = false, totalCount = 0 } = result;

    // Update counters
    this.messageCount += messages.length;
    this.totalLoadedMessages += messages.length;
    this.metrics.totalMessagesLoaded += messages.length;

    this.emit('progress', {
      chatId: this.chatId,
      phase: 'processing',
      newMessages: messages.length,
      totalLoaded: this.totalLoadedMessages,
      hasMore
    });

    if (import.meta.env.DEV) {
      console.log(`[AutoLoadManager] Processing load result: +${messages.length} messages, total: ${this.totalLoadedMessages}, has more: ${hasMore}`);
    }

    // Check if all loading is complete
    if (!hasMore || messages.length === 0) {
      this.transitionTo(this.states.ALL_LOADED);
      await this.handleAllLoaded();
    } else {
      // Continue loading
      this.transitionTo(this.states.LOADING);
    }
  }

  /**
   * Wait for user to scroll to top (user-controlled loading)
   */
  async waitForUserScroll() {
    return new Promise((resolve, reject) => {
      if (import.meta.env.DEV) {
        console.log(`[AutoLoadManager] Waiting for user control - ${this.totalLoadedMessages} messages loaded`);
      }

      // Transition to waiting for scroll state
      this.transitionTo(this.states.WAITING_FOR_SCROLL);
      this.waitingForScroll = true;

      // Notify UI layer that user control is needed (enhance user control feeling)
      this.emit('scroll-needed', {
        chatId: this.chatId,
        totalLoaded: this.totalLoadedMessages,
        batchSize: this.batchSize,
        userControlEnabled: this.userControlEnabled,
        message: `Loaded ${this.totalLoadedMessages} messages, continue loading next ${this.batchSize}?`,
        options: {
          canScroll: true,    // User can scroll to continue
          canClick: true,     // User can click to continue  
          canStop: true,      // User can stop at any time
          canAdjustBatch: true // User can adjust batch size
        }
      });

      // Call external scroll callback
      if (this.scrollTriggerCallback) {
        try {
          this.scrollTriggerCallback({
            chatId: this.chatId,
            totalLoaded: this.totalLoadedMessages,
            continueLoading: () => this.resumeFromScroll(resolve),
            cancelLoading: () => this.cancelFromScroll(reject)
          });
        } catch (error) {
          console.error('🚨 [AutoLoadManager] 滚动回调执行失败:', error);
          this.cancelFromScroll(reject);
        }
      }

      // 设置较长的超时时间，给用户更多控制时间
      const scrollTimeout = setTimeout(() => {
        if (this.waitingForScroll) {
          if (import.meta.env.DEV) {
            console.log('⏰ [AutoLoadManager] 用户控制超时，自动保存当前进度');
          }
          // 不强制取消，而是保存进度让用户选择
          this.emit('user-timeout', {
            chatId: this.chatId,
            totalLoaded: this.totalLoadedMessages,
            message: '已为您保存当前加载进度，可稍后继续'
          });
        }
      }, 300000); // 5分钟超时，更人性化

      // 存储超时ID以便清理
      this._scrollTimeout = scrollTimeout;
    });
  }

  /**
   * 🔄 NEW: 从滚动等待状态恢复
   */
  resumeFromScroll(resolve) {
    if (!this.waitingForScroll) {
      return;
    }

    if (import.meta.env.DEV) {
      console.log('[AutoLoadManager] 用户滚动检测到，继续加载');
    }

    this.waitingForScroll = false;

    // 清理超时
    if (this._scrollTimeout) {
      clearTimeout(this._scrollTimeout);
      this._scrollTimeout = null;
    }

    // 恢复到加载状态
    this.transitionTo(this.states.LOADING);

    // 发送恢复事件
    this.emit('scroll-resumed', {
      chatId: this.chatId,
      totalLoaded: this.totalLoadedMessages
    });

    resolve();
  }

  /**
 * 🔄 ENHANCED: 从滚动等待状态取消 (用户友好式停止)
 */
  cancelFromScroll(reject, reason = 'timeout') {
    if (!this.waitingForScroll) {
      return;
    }

    if (import.meta.env.DEV) {
      console.log(`🛑 [AutoLoadManager] 用户控制停止: ${reason}`);
    }

    this.waitingForScroll = false;

    // 清理超时
    if (this._scrollTimeout) {
      clearTimeout(this._scrollTimeout);
      this._scrollTimeout = null;
    }

    // 根据停止原因采取不同行动
    if (reason === 'user-stop') {
      // 用户主动停止 - 保存进度，显示友好提示
      this.transitionTo(this.states.COMPLETED);
      this.emit('user-stopped', {
        chatId: this.chatId,
        totalLoaded: this.totalLoadedMessages,
        message: `已为您加载 ${this.totalLoadedMessages} 条历史消息`,
        canResume: true // 用户可以稍后恢复
      });
    } else {
      // 其他原因停止
      this.stopAutoLoad(reason);
      this.emit('scroll-cancelled', {
        chatId: this.chatId,
        totalLoaded: this.totalLoadedMessages,
        reason
      });
    }

    reject(new Error(`加载已停止: ${reason}`));
  }

  /**
   * NEW: 用户主动停止加载 (任意位置停止)
   */
  userStopLoading() {
    if (import.meta.env.DEV) {
      console.log('USER: [AutoLoadManager] 用户主动停止加载');
    }

    if (this.waitingForScroll) {
      // 正在等待用户操作时停止
      this.cancelFromScroll(() => { }, 'user-stop');
    } else if (this.isActive) {
      // 正在加载时停止
      this.stopAutoLoad('user-stop');
      this.emit('user-stopped', {
        chatId: this.chatId,
        totalLoaded: this.totalLoadedMessages,
        message: `已为您加载 ${this.totalLoadedMessages} 条历史消息`,
        canResume: true
      });
    }

    return {
      stopped: true,
      totalLoaded: this.totalLoadedMessages,
      canResume: true
    };
  }

  /**
   * NEW: 调整批次大小 (性能控制)
   */
  adjustBatchSize(newBatchSize) {
    if (newBatchSize > 0 && newBatchSize <= 100) {
      this.batchSize = newBatchSize;

      if (import.meta.env.DEV) {
        console.log(`[AutoLoadManager] 批次大小调整为: ${newBatchSize}`);
      }

      this.emit('batch-size-changed', {
        chatId: this.chatId,
        oldBatchSize: this.batchSize,
        newBatchSize: newBatchSize,
        reason: '用户调整性能参数'
      });

      return true;
    }
    return false;
  }

  /**
   * 处理全部加载完成
   */
  async handleAllLoaded() {
    if (import.meta.env.DEV) {
      console.log('[AutoLoadManager] 所有消息已加载完成');
    }

    this.transitionTo(this.states.COMPLETED);

    // 记录成功
    this.metrics.successfulSessions++;
    const duration = Date.now() - this.startTime;
    this.updateAverageLoadTime(duration);

    // 发送完成事件
    this.emit('complete', {
      chatId: this.chatId,
      totalLoaded: this.totalLoadedMessages,
      duration,
      success: true
    });

    // 显示用户完成提示
    await this.showCompletionNotification();

    // 标记会话结束
    this.isActive = false;
  }

  /**
   * 显示完成通知
   */
  async showCompletionNotification() {
    if (this.completionShown || this.userDismissed) {
      return;
    }

    this.completionShown = true;

    // 发送UI更新事件
    this.emit('ui-update', {
      type: 'completion',
      data: {
        chatId: this.chatId,
        totalLoaded: this.totalLoadedMessages,
        message: `已加载全部 ${this.totalLoadedMessages} 条历史消息`
      }
    });

    if (import.meta.env.DEV) {
      console.log(`[AutoLoadManager] 显示完成通知: ${this.totalLoadedMessages} 条消息`);
    }
  }

  /**
   * 🔄 状态转换
   */
  transitionTo(newState) {
    const oldState = this.currentState;
    this.currentState = newState;

    if (import.meta.env.DEV) {
      console.log(`🔄 [AutoLoadManager] 状态转换: ${oldState} → ${newState}`);
    }

    this.emit('state-change', {
      from: oldState,
      to: newState,
      chatId: this.chatId
    });
  }

  /**
   * 🛑 停止自动加载
   */
  stopAutoLoad(reason = 'manual') {
    if (!this.isActive) {
      return;
    }

    if (import.meta.env.DEV) {
      console.log(`🛑 [AutoLoadManager] 停止自动加载: ${reason}`);
    }

    this.isActive = false;
    this.transitionTo(this.states.DISABLED);

    this.emit('stopped', {
      chatId: this.chatId,
      reason,
      totalLoaded: this.totalLoadedMessages
    });
  }

  /**
   * 🔄 重置状态
   */
  reset() {
    this.currentState = this.states.IDLE;
    this.chatId = null;
    this.loadCallback = null;
    this.messageCount = 0;
    this.totalLoadedMessages = 0;
    this.loadingAttempts = 0;
    this.isActive = false;
    this.userNotified = false;
    this.completionShown = false;
    this.userDismissed = false;
    this.startTime = 0;
    this.eventListeners.clear();
  }

  /**
   * 验证加载结果格式
   */
  isValidLoadResult(result) {
    return (
      result &&
      typeof result === 'object' &&
      Array.isArray(result.messages) &&
      typeof result.hasMore === 'boolean'
    );
  }

  /**
   * ⏰ 等待指定时间
   */
  wait(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
  }

  /**
   * 更新平均加载时间
   */
  updateAverageLoadTime(duration) {
    const totalTime = this.metrics.averageLoadTime * (this.metrics.successfulSessions - 1) + duration;
    this.metrics.averageLoadTime = totalTime / this.metrics.successfulSessions;
  }

  /**
   * 🎧 事件系统
   */
  on(event, callback) {
    if (!this.eventListeners.has(event)) {
      this.eventListeners.set(event, []);
    }
    this.eventListeners.get(event).push(callback);
  }

  off(event, callback) {
    if (this.eventListeners.has(event)) {
      const listeners = this.eventListeners.get(event);
      const index = listeners.indexOf(callback);
      if (index > -1) {
        listeners.splice(index, 1);
      }
    }
  }

  emit(event, data) {
    if (this.eventListeners.has(event)) {
      this.eventListeners.get(event).forEach(callback => {
        try {
          callback(data);
        } catch (error) {
          console.error(`事件回调失败 [${event}]:`, error);
        }
      });
    }
  }

  /**
   * 📈 获取性能指标
   */
  getMetrics() {
    const successRate = this.metrics.totalSessions > 0
      ? ((this.metrics.successfulSessions / this.metrics.totalSessions) * 100).toFixed(1)
      : '0.0';

    return {
      ...this.metrics,
      successRate: `${successRate}%`,
      currentState: this.currentState,
      isActive: this.isActive,
      currentChatId: this.chatId
    };
  }

  /**
   * 获取当前状态
   */
  getState() {
    return {
      current: this.currentState,
      chatId: this.chatId,
      isActive: this.isActive,
      totalLoaded: this.totalLoadedMessages,
      attempts: this.loadingAttempts,
      completionShown: this.completionShown,
      userDismissed: this.userDismissed
    };
  }

  /**
   * USER: 用户交互方法
   */
  userDismissCompletion() {
    this.userDismissed = true;
    this.emit('user-dismissed', { chatId: this.chatId });

    if (import.meta.env.DEV) {
      console.log('USER: [AutoLoadManager] 用户关闭了完成提示');
    }
  }

  /**
   * 🔄 用户请求重试
   */
  async userRetry() {
    if (this.currentState !== this.states.ERROR) {
      return false;
    }

    if (import.meta.env.DEV) {
      console.log('🔄 [AutoLoadManager] 用户请求重试');
    }

    this.loadingAttempts = 0;
    this.transitionTo(this.states.LOADING);

    try {
      await this.executeLoadingLoop();
      return true;
    } catch (error) {
      this.transitionTo(this.states.ERROR);
      this.emit('error', { error: error.message, chatId: this.chatId });
      return false;
    }
  }
}

// 🌍 创建全局单例
export const autoLoadManager = new AutoLoadManager();

// 开发环境调试函数
if (typeof window !== 'undefined' && import.meta.env.DEV) {
  window.autoLoadManager = autoLoadManager;

  window.debugAutoLoad = () => {
    console.log('🔄 [AutoLoadManager] 调试信息:', autoLoadManager.getMetrics());
    console.log('🔄 [AutoLoadManager] 当前状态:', autoLoadManager.getState());
  };

  window.stopAutoLoad = (reason = 'debug') => {
    autoLoadManager.stopAutoLoad(reason);
  };

  console.log('🔄 AutoLoadManager 调试功能已加载');
} 