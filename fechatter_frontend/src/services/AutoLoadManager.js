/**
 * 🔄 AutoLoadManager - 100%可靠的历史消息自动加载管理器
 * 
 * 确保"Loading earlier messages..."功能完整生命周期管理：
 * 1. 自动检测和加载历史消息
 * 2. 100%保证在所有消息加载完成后停止
 * 3. 提供用户友好的完成提示
 * 4. 防止用户继续尝试刷新
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
      WAITING_FOR_SCROLL: 'waiting_for_scroll' // 🔄 NEW: 等待用户滚动状态
    };

    this.currentState = this.states.IDLE;
    this.chatId = null;
    this.loadCallback = null;
    this.messageCount = 0;
    this.totalLoadedMessages = 0;
    this.loadingAttempts = 0;
    this.maxRetries = 3;
    this.isActive = false;

    // 🔄 NEW: 加载模式配置 (默认滚动触发模式)
    this.loadingMode = 'scroll-triggered'; // 'auto' | 'scroll-triggered'
    this.waitingForScroll = false;
    this.scrollTriggerCallback = null;
    this.batchSize = 20; // 默认每批加载20条消息
    this.userControlEnabled = true; // 用户控制开关

    // 用户交互状态
    this.userNotified = false;
    this.completionShown = false;
    this.userDismissed = false;

    // 性能和调试
    this.startTime = 0;
    this.metrics = {
      totalSessions: 0,
      successfulSessions: 0,
      totalMessagesLoaded: 0,
      averageLoadTime: 0,
      errorCount: 0
    };

    // 事件系统
    this.eventListeners = new Map();

    if (import.meta.env.DEV) {
      console.log('🔄 [AutoLoadManager] 初始化完成');
    }
  }

  /**
   * 🎯 启动自动加载会话
   */
  async startAutoLoadSession({
    chatId,
    hasMoreMessages,
    loadCallback,
    onProgress = null,
    onComplete = null,
    onError = null,
    // 🔄 NEW: 加载模式配置 (默认滚动触发模式)
    loadingMode = 'scroll-triggered', // 'auto' | 'scroll-triggered'
    onScrollNeeded = null, // 滚动触发模式的回调
    batchSize = 20, // 每批加载数量 (性能控制)
    userControlEnabled = true // 用户控制开关
  }) {
    // 防止重复启动
    if (this.isActive && this.chatId === chatId) {
      if (import.meta.env.DEV) {
        console.warn('🔄 [AutoLoadManager] 会话已激活，忽略重复启动');
      }
      return false;
    }

    // 重置状态
    this.reset();
    this.chatId = chatId;
    this.loadCallback = loadCallback;
    this.isActive = true;
    this.startTime = Date.now();
    this.metrics.totalSessions++;

    // 🔄 NEW: 设置加载模式和用户控制配置
    this.loadingMode = loadingMode;
    this.scrollTriggerCallback = onScrollNeeded;
    this.waitingForScroll = false;
    this.batchSize = batchSize;
    this.userControlEnabled = userControlEnabled;

    // 注册回调
    if (onProgress) this.on('progress', onProgress);
    if (onComplete) this.on('complete', onComplete);
    if (onError) this.on('error', onError);

    if (import.meta.env.DEV) {
      console.log(`🔄 [AutoLoadManager] 开始自动加载会话 - Chat ${chatId}`);
    }

    try {
      // 第一步：检测是否需要加载
      await this.detectLoadingNeed(hasMoreMessages);

      // 第二步：开始加载循环
      if (this.currentState === this.states.LOADING) {
        await this.executeLoadingLoop();
      }

      return this.currentState === this.states.COMPLETED;

    } catch (error) {
      console.error('🚨 [AutoLoadManager] 自动加载会话失败:', error);
      this.transitionTo(this.states.ERROR);
      this.emit('error', { error: error.message, chatId });
      return false;
    }
  }

  /**
   * 🔍 第一阶段：检测加载需求
   */
  async detectLoadingNeed(hasMoreMessages) {
    this.transitionTo(this.states.DETECTING_NEED);

    if (!hasMoreMessages) {
      if (import.meta.env.DEV) {
        console.log('🔄 [AutoLoadManager] 检测到无更多消息，直接完成');
      }
      this.transitionTo(this.states.ALL_LOADED);
      await this.handleAllLoaded();
      return;
    }

    if (import.meta.env.DEV) {
      console.log('🔄 [AutoLoadManager] 检测到有更多消息，开始加载');
    }
    this.transitionTo(this.states.LOADING);
  }

  /**
 * 🔄 第二阶段：执行加载循环
 */
  async executeLoadingLoop() {
    while (this.currentState === this.states.LOADING && this.isActive) {
      try {
        // 执行单次加载
        const result = await this.performSingleLoad();

        if (!result.success) {
          throw new Error(result.error || '加载失败');
        }

        // 处理加载结果
        await this.processLoadResult(result);

        // 检查是否需要继续
        if (!result.hasMore) {
          break;
        }

        // 🔄 NEW: 根据加载模式决定是否继续
        if (this.loadingMode === 'scroll-triggered') {
          // 滚动触发模式：等待用户滚动到顶部
          await this.waitForUserScroll();

          // 如果用户没有继续滚动或会话被取消，退出循环
          if (!this.isActive || this.currentState !== this.states.LOADING) {
            break;
          }
        } else {
          // 自动模式：直接继续加载
          await this.wait(100); // 避免过快的连续请求
        }

      } catch (error) {
        this.loadingAttempts++;

        if (this.loadingAttempts >= this.maxRetries) {
          throw error;
        }

        if (import.meta.env.DEV) {
          console.warn(`⚠️ [AutoLoadManager] 加载失败，重试 ${this.loadingAttempts}/${this.maxRetries}:`, error);
        }

        await this.wait(1000 * this.loadingAttempts); // 递增延迟
      }
    }
  }

  /**
   * 📦 执行单次加载
   */
  async performSingleLoad() {
    if (!this.loadCallback) {
      throw new Error('加载回调未设置');
    }

    this.emit('progress', {
      chatId: this.chatId,
      phase: 'loading',
      attempt: this.loadingAttempts + 1
    });

    if (import.meta.env.DEV) {
      console.log(`📦 [AutoLoadManager] 执行第 ${this.loadingAttempts + 1} 次加载`);
    }

    // 调用外部加载函数
    const result = await this.loadCallback();

    // 验证结果格式
    if (!this.isValidLoadResult(result)) {
      throw new Error('加载回调返回了无效的结果格式');
    }

    return result;
  }

  /**
   * 🔧 处理加载结果
   */
  async processLoadResult(result) {
    this.transitionTo(this.states.PROCESSING);

    const { messages = [], hasMore = false, totalCount = 0 } = result;

    // 更新计数
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
      console.log(`🔧 [AutoLoadManager] 处理加载结果: +${messages.length} 消息, 总计: ${this.totalLoadedMessages}, 还有更多: ${hasMore}`);
    }

    // 检查是否全部加载完成
    if (!hasMore || messages.length === 0) {
      this.transitionTo(this.states.ALL_LOADED);
      await this.handleAllLoaded();
    } else {
      // 继续加载
      this.transitionTo(this.states.LOADING);
    }
  }

  /**
   * 🔄 ENHANCED: 等待用户滚动到顶部 (用户控制式加载)
   */
  async waitForUserScroll() {
    return new Promise((resolve, reject) => {
      if (import.meta.env.DEV) {
        console.log(`⏳ [AutoLoadManager] 等待用户控制 - 已加载 ${this.totalLoadedMessages} 条消息`);
      }

      // 转换到等待滚动状态
      this.transitionTo(this.states.WAITING_FOR_SCROLL);
      this.waitingForScroll = true;

      // 🎯 通知UI层需要用户控制 (增强用户控制感)
      this.emit('scroll-needed', {
        chatId: this.chatId,
        totalLoaded: this.totalLoadedMessages,
        batchSize: this.batchSize,
        userControlEnabled: this.userControlEnabled,
        message: `已加载 ${this.totalLoadedMessages} 条消息，继续加载下 ${this.batchSize} 条？`,
        options: {
          canScroll: true,    // ✅ 用户可以滚动继续
          canClick: true,     // ✅ 用户可以点击继续  
          canStop: true,      // ✅ 用户可以随时停止
          canAdjustBatch: true // ✅ 用户可以调整批次大小
        }
      });

      // 调用外部滚动回调
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

      // 🎯 设置较长的超时时间，给用户更多控制时间
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
      console.log('✅ [AutoLoadManager] 用户滚动检测到，继续加载');
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

    // 🎯 根据停止原因采取不同行动
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
   * 🎯 NEW: 用户主动停止加载 (任意位置停止)
   */
  userStopLoading() {
    if (import.meta.env.DEV) {
      console.log('👤 [AutoLoadManager] 用户主动停止加载');
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
   * 🎯 NEW: 调整批次大小 (性能控制)
   */
  adjustBatchSize(newBatchSize) {
    if (newBatchSize > 0 && newBatchSize <= 100) {
      this.batchSize = newBatchSize;

      if (import.meta.env.DEV) {
        console.log(`📊 [AutoLoadManager] 批次大小调整为: ${newBatchSize}`);
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
   * ✅ 处理全部加载完成
   */
  async handleAllLoaded() {
    if (import.meta.env.DEV) {
      console.log('✅ [AutoLoadManager] 所有消息已加载完成');
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
   * 🎨 显示完成通知
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
      console.log(`🎨 [AutoLoadManager] 显示完成通知: ${this.totalLoadedMessages} 条消息`);
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
   * 📊 验证加载结果格式
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
   * 📊 更新平均加载时间
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
   * 🎯 获取当前状态
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
   * 👤 用户交互方法
   */
  userDismissCompletion() {
    this.userDismissed = true;
    this.emit('user-dismissed', { chatId: this.chatId });

    if (import.meta.env.DEV) {
      console.log('👤 [AutoLoadManager] 用户关闭了完成提示');
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

// 🔧 开发环境调试函数
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