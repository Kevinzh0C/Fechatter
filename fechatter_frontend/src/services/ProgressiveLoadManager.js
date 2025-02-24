/**
 * 🔄 ProgressiveLoadManager - 渐进式预加载管理器
 * 
 * 解决问题：
 * 1. 消息加载过于突兀，不是匀速匀质的随用户滑动出现
 * 2. 检测点触发算法需要改进，实现更自然的加载体验
 * 
 * 核心特性：
 * - 智能滚动速度检测
 * - 预测性加载机制
 * - 分层渐进式加载
 * - 平滑的视觉过渡
 */

export class ProgressiveLoadManager {
  constructor() {
    // 状态管理
    this.isActive = false;
    this.chatId = null;
    this.loadCallback = null;

    // 滚动行为分析
    this.scrollMetrics = {
      velocity: 0,
      direction: 'none', // 'up', 'down', 'none'
      acceleration: 0,
      lastPosition: 0,
      lastTimestamp: 0,
      samples: [] // 最近10个滚动样本
    };

    // 加载策略配置
    this.loadingStrategy = {
      // 预加载区域 (多层次)
      preloadZones: {
        immediate: 150,    // 立即加载区 (150px)
        anticipatory: 300, // 预测加载区 (300px)
        background: 600    // 背景加载区 (600px)
      },

      // 分层加载数量
      batchSizes: {
        micro: 3,    // 微量加载 (3条消息)
        small: 5,    // 小批量加载 (5条消息)
        medium: 10,  // 中等批量加载 (10条消息)
        large: 15    // 大批量加载 (15条消息)
      },

      // 加载间隔 (毫秒)
      intervals: {
        rapid: 100,     // 快速滚动时
        normal: 300,    // 正常滚动时
        slow: 800,      // 慢速滚动时
        predictive: 50  // 预测性加载时
      }
    };

    // 当前加载状态
    this.loadingState = {
      isLoading: false,
      lastLoadTime: 0,
      pendingLoads: new Set(),
      loadQueue: [],
      totalLoaded: 0
    };

    // 性能优化
    this.performanceConfig = {
      maxSamples: 10,           // 最大滚动样本数
      velocityThreshold: 50,    // 速度阈值 (px/s)
      accelerationThreshold: 20, // 加速度阈值
      debounceTime: 16,         // 16ms debounce (60fps)
      preloadCooldown: 200      // 预加载冷却时间
    };

    // 事件系统
    this.eventListeners = new Map();

    if (import.meta.env.DEV) {
      console.log('🔄 [ProgressiveLoadManager] 渐进式加载管理器已初始化');
    }
  }

  /**
   * 初始化管理器
   */
  initialize(chatId, loadCallback) {
    this.chatId = chatId;
    this.loadCallback = loadCallback;
    this.isActive = true;

    // 重置状态
    this.resetScrollMetrics();
    this.resetLoadingState();

    if (import.meta.env.DEV) {
      console.log(`🔄 [ProgressiveLoadManager] 已为 chat ${chatId} 初始化`);
    }

    this.emit('initialized', { chatId });
  }

  /**
   * 🧠 智能滚动分析 - 核心算法
   */
  analyzeScrollBehavior(scrollTop, scrollHeight, clientHeight, timestamp) {
    if (!this.isActive) return null;

    const now = timestamp || Date.now();
    const deltaTime = now - this.scrollMetrics.lastTimestamp;
    const deltaPosition = scrollTop - this.scrollMetrics.lastPosition;

    // 计算瞬时速度 (px/ms)
    const velocity = deltaTime > 0 ? deltaPosition / deltaTime : 0;

    // 计算加速度
    const acceleration = this.scrollMetrics.velocity !== 0
      ? (velocity - this.scrollMetrics.velocity) / deltaTime
      : 0;

    // 确定滚动方向
    let direction = 'none';
    if (Math.abs(deltaPosition) > 1) {
      direction = deltaPosition > 0 ? 'down' : 'up';
    }

    // 更新滚动样本
    this.scrollMetrics.samples.push({
      position: scrollTop,
      velocity: velocity * 1000, // 转换为 px/s
      acceleration: acceleration * 1000000, // 转换为 px/s²
      timestamp: now,
      direction
    });

    // 保持样本数量限制
    if (this.scrollMetrics.samples.length > this.performanceConfig.maxSamples) {
      this.scrollMetrics.samples.shift();
    }

    // 更新当前指标
    this.scrollMetrics.velocity = velocity * 1000;
    this.scrollMetrics.direction = direction;
    this.scrollMetrics.acceleration = acceleration * 1000000;
    this.scrollMetrics.lastPosition = scrollTop;
    this.scrollMetrics.lastTimestamp = now;

    // 计算平均速度和方向趋势
    const avgVelocity = this.calculateAverageVelocity();
    const directionTrend = this.calculateDirectionTrend();
    const isStabilizing = this.isScrollStabilizing();

    return {
      currentVelocity: this.scrollMetrics.velocity,
      averageVelocity: avgVelocity,
      direction: this.scrollMetrics.direction,
      directionTrend,
      acceleration: this.scrollMetrics.acceleration,
      isStabilizing,
      distanceFromTop: scrollTop,
      distanceFromBottom: scrollHeight - scrollTop - clientHeight,
      scrollProgress: scrollTop / (scrollHeight - clientHeight)
    };
  }

  /**
   * 🎯 智能加载决策引擎
   */
  shouldLoadMore(scrollAnalysis, hasMoreMessages) {
    if (!this.isActive || !hasMoreMessages || this.loadingState.isLoading) {
      return { shouldLoad: false, reason: 'blocked' };
    }

    const now = Date.now();
    const timeSinceLastLoad = now - this.loadingState.lastLoadTime;

    // 冷却期检查
    if (timeSinceLastLoad < this.performanceConfig.preloadCooldown) {
      return { shouldLoad: false, reason: 'cooldown' };
    }

    const {
      distanceFromTop,
      currentVelocity,
      averageVelocity,
      direction,
      directionTrend,
      isStabilizing
    } = scrollAnalysis;

    // 🔍 多层次触发区域检测
    const triggers = this.evaluateLoadTriggers(scrollAnalysis);

    if (triggers.immediate) {
      return {
        shouldLoad: true,
        reason: 'immediate',
        strategy: this.selectLoadingStrategy(scrollAnalysis),
        priority: 'high'
      };
    }

    if (triggers.anticipatory && direction === 'up') {
      return {
        shouldLoad: true,
        reason: 'anticipatory',
        strategy: this.selectLoadingStrategy(scrollAnalysis),
        priority: 'medium'
      };
    }

    if (triggers.predictive && this.isPredictiveLoadingBeneficial(scrollAnalysis)) {
      return {
        shouldLoad: true,
        reason: 'predictive',
        strategy: this.selectLoadingStrategy(scrollAnalysis),
        priority: 'low'
      };
    }

    return { shouldLoad: false, reason: 'no-trigger' };
  }

  /**
   * 📊 评估加载触发器
   */
  evaluateLoadTriggers(scrollAnalysis) {
    const { distanceFromTop } = scrollAnalysis;
    const zones = this.loadingStrategy.preloadZones;

    return {
      immediate: distanceFromTop <= zones.immediate,
      anticipatory: distanceFromTop <= zones.anticipatory,
      predictive: distanceFromTop <= zones.background
    };
  }

  /**
   * 🧮 选择最佳加载策略
   */
  selectLoadingStrategy(scrollAnalysis) {
    const { currentVelocity, averageVelocity, direction, isStabilizing } = scrollAnalysis;
    const absVelocity = Math.abs(currentVelocity);

    // 快速滚动 - 预测用户意图，小批量频繁加载
    if (absVelocity > this.performanceConfig.velocityThreshold && direction === 'up') {
      return {
        batchSize: this.loadingStrategy.batchSizes.micro,
        interval: this.loadingStrategy.intervals.rapid,
        type: 'rapid'
      };
    }

    // 稳定滚动 - 正常批量加载
    if (isStabilizing && direction === 'up') {
      return {
        batchSize: this.loadingStrategy.batchSizes.small,
        interval: this.loadingStrategy.intervals.normal,
        type: 'stable'
      };
    }

    // 慢速滚动 - 更大批量，减少频率
    if (absVelocity < 20 && direction === 'up') {
      return {
        batchSize: this.loadingStrategy.batchSizes.medium,
        interval: this.loadingStrategy.intervals.slow,
        type: 'slow'
      };
    }

    // 预测性加载 - 微量加载
    return {
      batchSize: this.loadingStrategy.batchSizes.micro,
      interval: this.loadingStrategy.intervals.predictive,
      type: 'predictive'
    };
  }

  /**
   * 🔮 执行渐进式加载 - 增强错误处理
   */
  async executeProgressiveLoad(strategy) {
    if (this.loadingState.isLoading) {
      if (import.meta.env.DEV) {
        console.log(`🔄 [ProgressiveLoadManager] 跳过重复加载请求 (${strategy.type})`);
      }
      return { success: false, reason: 'already_loading' };
    }

    this.loadingState.isLoading = true;
    this.loadingState.lastLoadTime = Date.now();

    if (import.meta.env.DEV) {
      console.log(`🔄 [ProgressiveLoadManager] 执行 ${strategy.type} 加载策略:`, {
        batchSize: strategy.batchSize,
        interval: strategy.interval
      });
    }

    try {
      // 通知开始加载
      this.emit('load-start', {
        chatId: this.chatId,
        strategy,
        batchSize: strategy.batchSize
      });

      // 执行实际加载
      const result = await this.loadCallback({
        limit: strategy.batchSize,
        type: strategy.type,
        progressive: true
      });

      // 🔧 改进结果处理：支持各种成功情况
      if (result.success) {
        const messageCount = result.messageCount || 0;
        this.loadingState.totalLoaded += messageCount;

        // 通知加载完成
        this.emit('load-complete', {
          chatId: this.chatId,
          strategy,
          messageCount,
          totalLoaded: this.loadingState.totalLoaded,
          reason: result.reason || 'success'
        });

        if (import.meta.env.DEV) {
          if (messageCount > 0) {
            console.log(`✅ [ProgressiveLoadManager] ${strategy.type} 加载完成: +${messageCount} 条消息`);
          } else {
            console.log(`ℹ️ [ProgressiveLoadManager] ${strategy.type} 加载完成但无新消息 (${result.reason || '可能已到底部'})`);
          }
        }

        return { success: true, messageCount, reason: result.reason };
      } else {
        // 🔧 处理加载失败但不抛出错误的情况
        if (import.meta.env.DEV) {
          console.log(`⚠️ [ProgressiveLoadManager] ${strategy.type} 加载未成功:`, result.reason || 'unknown');
        }

        this.emit('load-complete', {
          chatId: this.chatId,
          strategy,
          messageCount: 0,
          totalLoaded: this.loadingState.totalLoaded,
          reason: result.reason || 'no_success'
        });

        return { success: false, reason: result.reason || 'no_success' };
      }

    } catch (error) {
      // 🔧 区分不同类型的错误
      const isTimeoutError = error.message.includes('timeout');
      const isNetworkError = error.message.includes('network') || error.message.includes('fetch');

      if (import.meta.env.DEV) {
        if (isTimeoutError) {
          console.log(`⏰ [ProgressiveLoadManager] ${strategy.type} 加载超时，可能已无更多消息`);
        } else if (isNetworkError) {
          console.warn(`🌐 [ProgressiveLoadManager] ${strategy.type} 网络错误:`, error.message);
        } else {
          console.error(`❌ [ProgressiveLoadManager] ${strategy.type} 加载失败:`, error);
        }
      }

      // 🔧 根据错误类型决定是否继续尝试
      const shouldRetry = !isTimeoutError && !error.message.includes('no more messages');

      this.emit('load-error', {
        chatId: this.chatId,
        strategy,
        error: error.message,
        errorType: isTimeoutError ? 'timeout' : isNetworkError ? 'network' : 'unknown',
        shouldRetry
      });

      return {
        success: false,
        error: error.message,
        errorType: isTimeoutError ? 'timeout' : 'error',
        shouldRetry
      };

    } finally {
      // 延迟重置加载状态，根据策略间隔
      setTimeout(() => {
        this.loadingState.isLoading = false;
      }, Math.min(strategy.interval, 1000)); // 最大1秒延迟，避免卡住
    }
  }

  /**
   * 📈 计算平均速度
   */
  calculateAverageVelocity() {
    if (this.scrollMetrics.samples.length < 2) return 0;

    const recentSamples = this.scrollMetrics.samples.slice(-5);
    const totalVelocity = recentSamples.reduce((sum, sample) => sum + Math.abs(sample.velocity), 0);
    return totalVelocity / recentSamples.length;
  }

  /**
   * 🧭 计算方向趋势
   */
  calculateDirectionTrend() {
    if (this.scrollMetrics.samples.length < 3) return 'none';

    const recentSamples = this.scrollMetrics.samples.slice(-3);
    const upCount = recentSamples.filter(s => s.direction === 'up').length;
    const downCount = recentSamples.filter(s => s.direction === 'down').length;

    if (upCount > downCount) return 'up';
    if (downCount > upCount) return 'down';
    return 'mixed';
  }

  /**
   * ⚖️ 判断滚动是否趋于稳定
   */
  isScrollStabilizing() {
    if (this.scrollMetrics.samples.length < 5) return false;

    const recentSamples = this.scrollMetrics.samples.slice(-5);
    const velocities = recentSamples.map(s => Math.abs(s.velocity));

    // 检查速度方差，低方差表示稳定
    const avg = velocities.reduce((a, b) => a + b) / velocities.length;
    const variance = velocities.reduce((sum, v) => sum + Math.pow(v - avg, 2), 0) / velocities.length;

    return variance < 500; // 经验阈值
  }

  /**
   * 🤖 判断是否应该进行预测性加载
   */
  isPredictiveLoadingBeneficial(scrollAnalysis) {
    const { direction, directionTrend, averageVelocity } = scrollAnalysis;

    // 只有向上滚动趋势明显时才预测性加载
    return direction === 'up' &&
      directionTrend === 'up' &&
      averageVelocity > 10 &&
      averageVelocity < 100;
  }

  /**
   * 🔄 重置滚动指标
   */
  resetScrollMetrics() {
    this.scrollMetrics = {
      velocity: 0,
      direction: 'none',
      acceleration: 0,
      lastPosition: 0,
      lastTimestamp: Date.now(),
      samples: []
    };
  }

  /**
   * 🧹 重置加载状态
   */
  resetLoadingState() {
    this.loadingState = {
      isLoading: false,
      lastLoadTime: 0,
      pendingLoads: new Set(),
      loadQueue: [],
      totalLoaded: 0
    };
  }

  /**
   * 🛑 停止管理器
   */
  stop() {
    this.isActive = false;
    this.resetScrollMetrics();
    this.resetLoadingState();

    if (import.meta.env.DEV) {
      console.log('🛑 [ProgressiveLoadManager] 已停止');
    }

    this.emit('stopped', { chatId: this.chatId });
  }

  /**
   * 📊 获取性能指标
   */
  getMetrics() {
    return {
      isActive: this.isActive,
      chatId: this.chatId,
      scrollMetrics: { ...this.scrollMetrics },
      loadingState: { ...this.loadingState },
      sampleCount: this.scrollMetrics.samples.length,
      avgVelocity: this.calculateAverageVelocity(),
      directionTrend: this.calculateDirectionTrend(),
      isStabilizing: this.isScrollStabilizing()
    };
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
      const callbacks = this.eventListeners.get(event);
      const index = callbacks.indexOf(callback);
      if (index > -1) {
        callbacks.splice(index, 1);
      }
    }
  }

  emit(event, data) {
    if (this.eventListeners.has(event)) {
      this.eventListeners.get(event).forEach(callback => {
        try {
          callback(data);
        } catch (error) {
          console.error(`🚨 [ProgressiveLoadManager] Event callback error:`, error);
        }
      });
    }
  }
}

// 创建全局单例
export const progressiveLoadManager = new ProgressiveLoadManager();

// 开发环境下全局可用
if (import.meta.env.DEV) {
  window.progressiveLoadManager = progressiveLoadManager;
  console.log('🔄 ProgressiveLoadManager 全局可用');
} 