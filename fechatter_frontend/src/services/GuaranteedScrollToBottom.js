/**
 * 🎯 GuaranteedScrollToBottom - 100%可靠的滚动到底部状态机
 * 
 * 解决第一次进入channel时不能确保滚动到最新消息位置的问题
 * 通过状态机管理整个滚动过程，包含重试和验证机制
 */

export class GuaranteedScrollToBottom {
  constructor() {
    this.states = {
      IDLE: 'idle',
      WAITING_FOR_MESSAGES: 'waiting_for_messages',
      WAITING_FOR_DOM: 'waiting_for_dom',
      CALCULATING_SCROLL: 'calculating_scroll',
      SCROLLING: 'scrolling',
      VERIFYING: 'verifying',
      COMPLETED: 'completed',
      FAILED: 'failed'
    };

    this.currentState = this.states.IDLE;
    this.retryCount = 0;
    this.maxRetries = 3;
    this.chatId = null;
    this.scrollContainer = null;
    this.targetElement = null;
    this.onComplete = null;
    this.onFailed = null;

    // 性能监控
    this.startTime = 0;
    this.metrics = {
      totalAttempts: 0,
      successfulScrolls: 0,
      failedScrolls: 0,
      averageTime: 0
    };

    if (import.meta.env.DEV) {
      console.log('🎯 [GuaranteedScrollToBottom] 状态机已初始化');
    }
  }

  /**
   * 🎯 主入口：启动滚动保证流程
   */
  async guaranteeScrollToBottom({
    chatId,
    scrollContainer,
    messages = [],
    onComplete = null,
    onFailed = null,
    forceSmooth = false
  }) {
    // 防止重复调用
    if (this.currentState !== this.states.IDLE && this.currentState !== this.states.COMPLETED && this.currentState !== this.states.FAILED) {
      if (import.meta.env.DEV) {
        console.warn('🎯 [GuaranteedScrollToBottom] 状态机正在运行，忽略重复调用');
      }
      return false;
    }

    this.startTime = Date.now();
    this.metrics.totalAttempts++;
    this.chatId = chatId;
    this.scrollContainer = scrollContainer;
    this.onComplete = onComplete;
    this.onFailed = onFailed;
    this.retryCount = 0;
    this.forceSmooth = forceSmooth;

    if (import.meta.env.DEV) {
      console.log(`🎯 [GuaranteedScrollToBottom] 开始为chat ${chatId}保证滚动到底部`);
    }

    try {
      // 第一阶段：等待消息
      await this.waitForMessages(messages);

      // 第二阶段：等待DOM
      await this.waitForDOM();

      // 第三阶段：执行滚动循环（包含重试）
      await this.executeScrollLoop();

      return this.currentState === this.states.COMPLETED;

    } catch (error) {
      console.error('🚨 [GuaranteedScrollToBottom] 状态机执行失败:', error);
      this.transitionTo(this.states.FAILED);
      this.callOnFailed(error.message);
      return false;
    }
  }

  /**
   * 🔄 状态转换
   */
  transitionTo(newState) {
    const oldState = this.currentState;
    this.currentState = newState;

    if (import.meta.env.DEV) {
      console.log(`🔄 [GuaranteedScrollToBottom] 状态转换: ${oldState} → ${newState}`);
    }
  }

  /**
   * 📦 第一阶段：等待消息加载
   */
  async waitForMessages(messages) {
    this.transitionTo(this.states.WAITING_FOR_MESSAGES);

    // 检查消息是否已存在
    if (messages && messages.length > 0) {
      if (import.meta.env.DEV) {
        console.log(`✅ [GuaranteedScrollToBottom] 消息已存在 (${messages.length}条)`);
      }
      return Promise.resolve();
    }

    // 等待消息加载，最多等待5秒
    return new Promise((resolve, reject) => {
      let attempts = 0;
      const maxAttempts = 50; // 5秒 (100ms * 50)

      const checkMessages = () => {
        attempts++;

        // 通过DOM检查消息是否已渲染
        const messageElements = this.scrollContainer?.querySelectorAll('[data-message-id]');

        if (messageElements && messageElements.length > 0) {
          if (import.meta.env.DEV) {
            console.log(`✅ [GuaranteedScrollToBottom] 检测到${messageElements.length}条消息`);
          }
          resolve();
        } else if (attempts >= maxAttempts) {
          reject(new Error('等待消息加载超时'));
        } else {
          setTimeout(checkMessages, 100);
        }
      };

      checkMessages();
    });
  }

  /**
   * 🏗️ 第二阶段：等待DOM渲染完成
   */
  async waitForDOM() {
    this.transitionTo(this.states.WAITING_FOR_DOM);

    return new Promise((resolve) => {
      // 使用双重requestAnimationFrame确保DOM完全渲染
      requestAnimationFrame(() => {
        requestAnimationFrame(() => {
          // 额外等待10ms确保浏览器完成所有布局计算
          setTimeout(() => {
            if (import.meta.env.DEV) {
              console.log('✅ [GuaranteedScrollToBottom] DOM渲染完成');
            }
            resolve();
          }, 10);
        });
      });
    });
  }

  /**
   * 🔄 第三阶段：执行滚动循环（包含重试）
   */
  async executeScrollLoop() {
    while (this.retryCount <= this.maxRetries) {
      try {
        // 计算滚动位置
        await this.calculateScrollTarget();

        // 执行滚动
        await this.performScroll();

        // 验证滚动结果
        const isSuccess = await this.verifyScrollPosition();

        if (isSuccess) {
          this.transitionTo(this.states.COMPLETED);
          this.recordSuccess();
          this.callOnComplete();
          return;
        }

        // 如果失败且未达到最大重试次数，继续重试
        this.retryCount++;
        if (this.retryCount <= this.maxRetries) {
          if (import.meta.env.DEV) {
            console.warn(`⚠️ [GuaranteedScrollToBottom] 滚动验证失败，重试 ${this.retryCount}/${this.maxRetries}`);
          }
          await this.wait(100); // 等待100ms后重试
        }

      } catch (error) {
        this.retryCount++;
        if (this.retryCount <= this.maxRetries) {
          if (import.meta.env.DEV) {
            console.warn(`⚠️ [GuaranteedScrollToBottom] 滚动执行失败，重试 ${this.retryCount}/${this.maxRetries}:`, error);
          }
          await this.wait(100);
        } else {
          throw error;
        }
      }
    }

    // 所有重试都失败，执行最后的强制滚动
    this.transitionTo(this.states.FAILED);
    await this.performForceScroll();
    this.recordFailure();
    this.callOnFailed('滚动验证失败，已执行强制滚动');
  }

  /**
   * 📐 计算滚动目标位置
   */
  async calculateScrollTarget() {
    this.transitionTo(this.states.CALCULATING_SCROLL);

    if (!this.scrollContainer) {
      throw new Error('滚动容器不存在');
    }

    // 确保容器有内容
    const scrollHeight = this.scrollContainer.scrollHeight;
    const clientHeight = this.scrollContainer.clientHeight;

    if (scrollHeight <= clientHeight) {
      if (import.meta.env.DEV) {
        console.log('📐 [GuaranteedScrollToBottom] 容器高度不足，无需滚动');
      }
      return;
    }

    // 计算目标滚动位置（底部）
    this.targetScrollTop = scrollHeight - clientHeight;

    if (import.meta.env.DEV) {
      console.log(`📐 [GuaranteedScrollToBottom] 计算滚动目标: ${this.targetScrollTop}px (总高度: ${scrollHeight}px)`);
    }
  }

  /**
   * 📜 执行滚动
   */
  async performScroll() {
    this.transitionTo(this.states.SCROLLING);

    if (!this.scrollContainer || this.targetScrollTop === undefined) {
      return;
    }

    return new Promise((resolve) => {
      const scrollOptions = {
        top: this.targetScrollTop,
        behavior: this.forceSmooth ? 'smooth' : 'instant'
      };

      this.scrollContainer.scrollTo(scrollOptions);

      // 如果是平滑滚动，等待滚动完成
      if (this.forceSmooth) {
        setTimeout(resolve, 300); // 等待平滑滚动完成
      } else {
        // 立即滚动，稍等确保浏览器处理完成
        setTimeout(resolve, 50);
      }
    });
  }

  /**
   * ✅ 验证滚动位置
   */
  async verifyScrollPosition() {
    this.transitionTo(this.states.VERIFYING);

    return new Promise((resolve) => {
      // 等待一帧确保滚动位置更新
      requestAnimationFrame(() => {
        const container = this.scrollContainer;
        if (!container) {
          resolve(false);
          return;
        }

        const { scrollTop, scrollHeight, clientHeight } = container;
        const maxScrollTop = scrollHeight - clientHeight;
        const tolerance = 5; // 5px容错范围

        const isAtBottom = Math.abs(scrollTop - maxScrollTop) <= tolerance;

        if (import.meta.env.DEV) {
          console.log(`✅ [GuaranteedScrollToBottom] 验证滚动位置:`, {
            scrollTop: Math.round(scrollTop),
            maxScrollTop: Math.round(maxScrollTop),
            difference: Math.round(Math.abs(scrollTop - maxScrollTop)),
            isAtBottom,
            tolerance
          });
        }

        resolve(isAtBottom);
      });
    });
  }

  /**
   * 💪 强制滚动（最后手段）
   */
  async performForceScroll() {
    if (!this.scrollContainer) return;

    // 多次尝试强制滚动
    for (let i = 0; i < 3; i++) {
      this.scrollContainer.scrollTop = this.scrollContainer.scrollHeight;
      await this.wait(50);
    }

    if (import.meta.env.DEV) {
      console.warn('💪 [GuaranteedScrollToBottom] 已执行强制滚动');
    }
  }

  /**
   * ⏰ 等待指定时间
   */
  wait(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
  }

  /**
   * 📊 记录成功
   */
  recordSuccess() {
    this.metrics.successfulScrolls++;
    const duration = Date.now() - this.startTime;
    this.updateAverageTime(duration);

    if (import.meta.env.DEV) {
      console.log(`🎉 [GuaranteedScrollToBottom] 滚动成功完成 (${duration}ms, 重试${this.retryCount}次)`);
    }
  }

  /**
   * 📊 记录失败
   */
  recordFailure() {
    this.metrics.failedScrolls++;
    const duration = Date.now() - this.startTime;
    this.updateAverageTime(duration);

    console.error(`💥 [GuaranteedScrollToBottom] 滚动失败 (${duration}ms, 重试${this.retryCount}次)`);
  }

  /**
   * 📊 更新平均时间
   */
  updateAverageTime(duration) {
    const totalTime = this.metrics.averageTime * (this.metrics.totalAttempts - 1) + duration;
    this.metrics.averageTime = totalTime / this.metrics.totalAttempts;
  }

  /**
   * 📞 调用完成回调
   */
  callOnComplete() {
    if (this.onComplete && typeof this.onComplete === 'function') {
      try {
        this.onComplete({
          chatId: this.chatId,
          duration: Date.now() - this.startTime,
          retryCount: this.retryCount
        });
      } catch (error) {
        console.error('回调函数执行失败:', error);
      }
    }
  }

  /**
   * 📞 调用失败回调
   */
  callOnFailed(reason) {
    if (this.onFailed && typeof this.onFailed === 'function') {
      try {
        this.onFailed({
          chatId: this.chatId,
          reason,
          duration: Date.now() - this.startTime,
          retryCount: this.retryCount
        });
      } catch (error) {
        console.error('失败回调函数执行失败:', error);
      }
    }
  }

  /**
   * 📊 获取性能指标
   */
  getMetrics() {
    const successRate = this.metrics.totalAttempts > 0
      ? ((this.metrics.successfulScrolls / this.metrics.totalAttempts) * 100).toFixed(1)
      : '0.0';

    return {
      ...this.metrics,
      successRate: `${successRate}%`,
      currentState: this.currentState
    };
  }

  /**
   * 🔄 重置状态机
   */
  reset() {
    this.currentState = this.states.IDLE;
    this.retryCount = 0;
    this.chatId = null;
    this.scrollContainer = null;
    this.targetElement = null;
    this.onComplete = null;
    this.onFailed = null;
  }
}

// 🌍 创建全局单例
export const guaranteedScrollToBottom = new GuaranteedScrollToBottom();

// 🔧 开发环境调试函数
if (typeof window !== 'undefined' && import.meta.env.DEV) {
  window.debugGuaranteedScroll = () => {
    console.log('🎯 [GuaranteedScrollToBottom] 调试信息:', guaranteedScrollToBottom.getMetrics());
  };

  window.testGuaranteedScroll = (chatId, container) => {
    return guaranteedScrollToBottom.guaranteeScrollToBottom({
      chatId,
      scrollContainer: container,
      messages: container?.querySelectorAll('[data-message-id]') || [],
      onComplete: (result) => console.log('✅ 测试滚动成功:', result),
      onFailed: (result) => console.error('❌ 测试滚动失败:', result)
    });
  };

  console.log('🎯 GuaranteedScrollToBottom调试功能已加载');
} 