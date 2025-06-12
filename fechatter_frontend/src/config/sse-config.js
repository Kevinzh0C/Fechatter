/**
 * SSE Configuration Manager
 * Manages switching between original and enhanced SSE implementations
 */

// Enhanced SSE service import
import enhancedSSE from '@/services/sse-enhanced';
import originalSSE from '@/services/sse';

// Configuration flags
const SSE_CONFIG = {
  // Feature flags
  USE_ENHANCED_SSE: true,           // 启用优化版SSE
  ENABLE_PERFORMANCE_MONITORING: true, // 启用性能监控
  ENABLE_PINGORA_DETECTION: true,   // 启用Pingora检测
  ENABLE_DEBUG_LOGGING: true,       // 启用调试日志

  // Performance settings
  ENABLE_INTELLIGENT_RECONNECT: true,  // 智能重连
  ENABLE_CONNECTION_QUALITY_ASSESSMENT: true, // 连接质量评估
  ENABLE_ENDPOINT_FALLBACK: true,      // 端点降级
  ENABLE_ERROR_SUPPRESSION: true,     // 错误抑制

  // Retry Control Settings
  RETRY_LIMITS: {
    // Basic SSE Service
    BASIC_SSE: {
      maxTotalAttempts: 10,          // 基础版最大总尝试次数
      maxConsecutiveFailures: 3,     // 基础版最大连续失败次数
      maxShortTermAttempts: 5,       // 短期重试最大次数
      initialDelay: 3000,            // 初始延迟（毫秒）
      maxDelay: 300000,              // 最大延迟（5分钟）
      backoffFactor: 1.5             // 退避因子
    },
    // Enhanced SSE Service
    ENHANCED_SSE: {
      maxTotalAttempts: 15,          // 增强版最大总尝试次数
      maxConsecutiveFailures: 5,     // 增强版最大连续失败次数
      maxShortTermAttempts: 8,       // 短期重试最大次数
      initialDelay: 1000,            // 初始延迟（毫秒）
      qualityBasedRetries: true,     // 基于连接质量的重试
      useIntelligentStrategies: true // 使用智能重试策略
    }
  },

  // Error Handling
  ERROR_HANDLING: {
    silentAfterAttempts: 2,          // 多少次尝试后错误变为静默
    logDetailedErrors: true,         // 记录详细错误信息
    trackErrorTypes: true,           // 跟踪错误类型
    enableUserNotifications: true   // 启用用户通知
  },

  // Experimental features
  ENABLE_WEBSOCKET_FALLBACK: false,   // WebSocket降级 (未实现)
  ENABLE_HTTP3_UPGRADE: false,        // HTTP/3升级 (未实现)

  // Development settings
  MOCK_NETWORK_CONDITIONS: false,     // 模拟网络条件
  FORCE_ERROR_SCENARIOS: false,       // 强制错误场景
};

// Environment-specific overrides
if (process.env.NODE_ENV === 'development') {
  SSE_CONFIG.ENABLE_DEBUG_LOGGING = true;
  SSE_CONFIG.ENABLE_PERFORMANCE_MONITORING = true;
  // 开发环境下减少重试次数，快速失败便于调试
  SSE_CONFIG.RETRY_LIMITS.BASIC_SSE.maxTotalAttempts = 5;
  SSE_CONFIG.RETRY_LIMITS.BASIC_SSE.maxConsecutiveFailures = 2;
  SSE_CONFIG.RETRY_LIMITS.ENHANCED_SSE.maxTotalAttempts = 8;
  SSE_CONFIG.RETRY_LIMITS.ENHANCED_SSE.maxConsecutiveFailures = 3;
}

if (process.env.NODE_ENV === 'production') {
  SSE_CONFIG.ENABLE_DEBUG_LOGGING = false;
  SSE_CONFIG.FORCE_ERROR_SCENARIOS = false;
  // 生产环境下增加重试次数，提高稳定性
  SSE_CONFIG.RETRY_LIMITS.BASIC_SSE.maxTotalAttempts = 15;
  SSE_CONFIG.RETRY_LIMITS.BASIC_SSE.maxConsecutiveFailures = 5;
  SSE_CONFIG.RETRY_LIMITS.ENHANCED_SSE.maxTotalAttempts = 20;
  SSE_CONFIG.RETRY_LIMITS.ENHANCED_SSE.maxConsecutiveFailures = 8;
  SSE_CONFIG.ERROR_HANDLING.silentAfterAttempts = 3;
}

/**
 * SSE Service Factory
 */
class SSEServiceFactory {
  constructor() {
    this.currentService = null;
    this.performanceMonitor = null;
    this.switchInProgress = false;
  }

  /**
   * Apply retry configuration to SSE service
   */
  applyRetryConfiguration(service) {
    const isEnhanced = SSE_CONFIG.USE_ENHANCED_SSE;
    const config = isEnhanced ? 
      SSE_CONFIG.RETRY_LIMITS.ENHANCED_SSE : 
      SSE_CONFIG.RETRY_LIMITS.BASIC_SSE;

    if (service.retryControl) {
      // 应用配置到服务
      service.retryControl.maxTotalAttempts = config.maxTotalAttempts;
      service.retryControl.maxConsecutiveFailures = config.maxConsecutiveFailures;
      
      if (config.maxShortTermAttempts) {
        service.maxReconnectAttempts = config.maxShortTermAttempts;
      }
      
      if (config.initialDelay) {
        service.reconnectDelay = config.initialDelay;
      }

      console.log(`🔧 [SSE_FACTORY] Applied ${isEnhanced ? 'Enhanced' : 'Basic'} retry configuration:`, {
        maxTotalAttempts: config.maxTotalAttempts,
        maxConsecutiveFailures: config.maxConsecutiveFailures,
        maxShortTermAttempts: config.maxShortTermAttempts || service.maxReconnectAttempts
      });
    }
  }

  /**
   * Get the appropriate SSE service instance
   */
  getSSEService() {
    if (this.currentService) {
      return this.currentService;
    }

    if (SSE_CONFIG.USE_ENHANCED_SSE) {
      console.log('🚀 [SSE_FACTORY] Using Enhanced SSE service');
      this.currentService = enhancedSSE;
    } else {
      console.log('📡 [SSE_FACTORY] Using Original SSE service');
      this.currentService = originalSSE;
    }

    // 应用重试配置
    this.applyRetryConfiguration(this.currentService);

    // 启用性能监控
    if (SSE_CONFIG.ENABLE_PERFORMANCE_MONITORING) {
      this.enablePerformanceMonitoring();
    }

    return this.currentService;
  }

  /**
   * Switch between SSE implementations
   */
  async switchSSEImplementation(useEnhanced = true) {
    if (this.switchInProgress) {
      console.warn('🔄 [SSE_FACTORY] Switch already in progress');
      return false;
    }

    this.switchInProgress = true;
    console.log(`🔄 [SSE_FACTORY] Switching to ${useEnhanced ? 'Enhanced' : 'Original'} SSE...`);

    try {
      // Disconnect current service
      if (this.currentService && this.currentService.isConnected) {
        this.currentService.disconnect();
      }

      // Update config
      SSE_CONFIG.USE_ENHANCED_SSE = useEnhanced;
      this.currentService = null;

      // Get new service
      const newService = this.getSSEService();

      // Reconnect if we have a token
      const { useAuthStore } = await import('@/stores/auth');
      const authStore = useAuthStore();

      if (authStore.token) {
        await newService.connect(authStore.token);
      }

      console.log(`✅ [SSE_FACTORY] Successfully switched to ${useEnhanced ? 'Enhanced' : 'Original'} SSE`);
      return true;

    } catch (error) {
      console.error('❌ [SSE_FACTORY] Failed to switch SSE implementation:', error);
      return false;
    } finally {
      this.switchInProgress = false;
    }
  }

  /**
   * Enable performance monitoring
   */
  enablePerformanceMonitoring() {
    if (this.performanceMonitor || !this.currentService) {
      return;
    }

    this.performanceMonitor = new SSEPerformanceMonitor(this.currentService);
    this.performanceMonitor.startMonitoring();
  }

  /**
   * Get performance metrics
   */
  getPerformanceMetrics() {
    if (!this.performanceMonitor) {
      return null;
    }

    return this.performanceMonitor.getMetrics();
  }

  /**
   * Compare performance between implementations
   */
  async runPerformanceComparison(durationMs = 60000) {
    console.log('📊 [SSE_FACTORY] Starting performance comparison...');

    const results = {
      original: null,
      enhanced: null,
      improvement: null
    };

    // Test original implementation
    console.log('📡 Testing Original SSE...');
    await this.switchSSEImplementation(false);
    await this.runPerformanceTest(durationMs / 2);
    results.original = this.getPerformanceMetrics();

    // Test enhanced implementation  
    console.log('🚀 Testing Enhanced SSE...');
    await this.switchSSEImplementation(true);
    await this.runPerformanceTest(durationMs / 2);
    results.enhanced = this.getPerformanceMetrics();

    // Calculate improvements
    if (results.original && results.enhanced) {
      results.improvement = this.calculateImprovement(results.original, results.enhanced);
    }

    console.log('📊 [SSE_FACTORY] Performance comparison completed:', results);
    return results;
  }

  /**
   * Run performance test for specified duration
   */
  async runPerformanceTest(durationMs) {
    return new Promise((resolve) => {
      if (this.performanceMonitor) {
        this.performanceMonitor.resetMetrics();
      }

      setTimeout(() => {
        resolve();
      }, durationMs);
    });
  }

  /**
   * Calculate performance improvement percentage
   */
  calculateImprovement(original, enhanced) {
    const improvements = {};

    // Calculate reconnection improvement
    if (original.avgReconnectDelay && enhanced.avgReconnectDelay) {
      improvements.reconnectDelay =
        ((original.avgReconnectDelay - enhanced.avgReconnectDelay) / original.avgReconnectDelay * 100).toFixed(1);
    }

    // Calculate error rate improvement
    if (original.errorRate && enhanced.errorRate) {
      improvements.errorRate =
        ((original.errorRate - enhanced.errorRate) / original.errorRate * 100).toFixed(1);
    }

    // Calculate connection success rate improvement
    if (original.connectionSuccessRate && enhanced.connectionSuccessRate) {
      improvements.connectionSuccessRate =
        ((enhanced.connectionSuccessRate - original.connectionSuccessRate) / original.connectionSuccessRate * 100).toFixed(1);
    }

    return improvements;
  }
}

/**
 * SSE Performance Monitor
 */
class SSEPerformanceMonitor {
  constructor(sseService) {
    this.sseService = sseService;
    this.metrics = {
      connectionAttempts: 0,
      successfulConnections: 0,
      failedConnections: 0,
      totalReconnectDelay: 0,
      reconnectCount: 0,
      errorCount: 0,
      startTime: null,
      lastConnectionTime: null,
      averageLatency: 0,
      minLatency: Infinity,
      maxLatency: 0,
      latencyMeasurements: []
    };

    this.isMonitoring = false;
    this.eventListeners = [];
  }

  /**
   * Start monitoring
   */
  startMonitoring() {
    if (this.isMonitoring) return;

    this.isMonitoring = true;
    this.metrics.startTime = Date.now();

    // Listen to SSE events
    this.sseService.on('connected', this.handleConnected.bind(this));
    this.sseService.on('disconnected', this.handleDisconnected.bind(this));
    this.sseService.on('reconnecting', this.handleReconnecting.bind(this));

    console.log('📊 [PERFORMANCE_MONITOR] Started monitoring SSE performance');
  }

  /**
   * Stop monitoring
   */
  stopMonitoring() {
    if (!this.isMonitoring) return;

    this.isMonitoring = false;

    // Remove event listeners
    this.sseService.off('connected', this.handleConnected);
    this.sseService.off('disconnected', this.handleDisconnected);
    this.sseService.off('reconnecting', this.handleReconnecting);

    console.log('📊 [PERFORMANCE_MONITOR] Stopped monitoring SSE performance');
  }

  /**
   * Handle connection success
   */
  handleConnected(data) {
    this.metrics.connectionAttempts++;
    this.metrics.successfulConnections++;
    this.metrics.lastConnectionTime = Date.now();

    // Measure latency if available
    const state = this.sseService.getConnectionState?.();
    if (state?.avgLatency) {
      this.updateLatencyMetrics(state.avgLatency);
    }
  }

  /**
   * Handle disconnection
   */
  handleDisconnected(data) {
    this.metrics.failedConnections++;

    if (data?.error) {
      this.metrics.errorCount++;
    }
  }

  /**
   * Handle reconnection attempt
   */
  handleReconnecting(data) {
    if (data?.delay) {
      this.metrics.totalReconnectDelay += data.delay;
      this.metrics.reconnectCount++;
    }
  }

  /**
   * Update latency metrics
   */
  updateLatencyMetrics(latency) {
    this.metrics.latencyMeasurements.push(latency);
    this.metrics.minLatency = Math.min(this.metrics.minLatency, latency);
    this.metrics.maxLatency = Math.max(this.metrics.maxLatency, latency);

    // Calculate average
    const total = this.metrics.latencyMeasurements.reduce((a, b) => a + b, 0);
    this.metrics.averageLatency = total / this.metrics.latencyMeasurements.length;
  }

  /**
   * Reset metrics
   */
  resetMetrics() {
    this.metrics = {
      connectionAttempts: 0,
      successfulConnections: 0,
      failedConnections: 0,
      totalReconnectDelay: 0,
      reconnectCount: 0,
      errorCount: 0,
      startTime: Date.now(),
      lastConnectionTime: null,
      averageLatency: 0,
      minLatency: Infinity,
      maxLatency: 0,
      latencyMeasurements: []
    };
  }

  /**
   * Get performance metrics
   */
  getMetrics() {
    const runtime = Date.now() - this.metrics.startTime;

    return {
      // Connection metrics
      connectionAttempts: this.metrics.connectionAttempts,
      successfulConnections: this.metrics.successfulConnections,
      failedConnections: this.metrics.failedConnections,
      connectionSuccessRate: this.metrics.connectionAttempts > 0 ?
        (this.metrics.successfulConnections / this.metrics.connectionAttempts * 100).toFixed(1) : 0,

      // Reconnection metrics
      avgReconnectDelay: this.metrics.reconnectCount > 0 ?
        (this.metrics.totalReconnectDelay / this.metrics.reconnectCount).toFixed(0) : 0,
      totalReconnects: this.metrics.reconnectCount,

      // Error metrics
      errorCount: this.metrics.errorCount,
      errorRate: runtime > 0 ?
        (this.metrics.errorCount / (runtime / 60000)).toFixed(2) : 0, // errors per minute

      // Latency metrics
      averageLatency: this.metrics.averageLatency.toFixed(0),
      minLatency: this.metrics.minLatency === Infinity ? 0 : this.metrics.minLatency,
      maxLatency: this.metrics.maxLatency,
      latencyMeasurements: this.metrics.latencyMeasurements.length,

      // Runtime
      runtimeMs: runtime,
      runtimeMinutes: (runtime / 60000).toFixed(1)
    };
  }
}

// Create singleton factory
const sseFactory = new SSEServiceFactory();

// Expose for debugging
if (typeof window !== 'undefined') {
  window.SSE_CONFIG = SSE_CONFIG;
  window.sseFactory = sseFactory;
}

export { SSE_CONFIG, sseFactory, SSEPerformanceMonitor };
export default sseFactory; 