/**
 * 🔧 System Monitor - Comprehensive debugging and performance monitoring
 * 
 * Provides real-time monitoring of:
 * - Message service health
 * - Cache performance
 * - Component states
 * - Memory usage
 * - Performance metrics
 */

class SystemMonitor {
  constructor() {
    this.isEnabled = import.meta.env.DEV;
    this.metrics = {
      messageServiceCalls: 0,
      cacheHits: 0,
      cacheMisses: 0,
      autoLoadTriggers: 0,
      componentMounts: 0,
      lastActivity: Date.now()
    };

    if (this.isEnabled) {
      this._setupGlobalFunctions();
      this._startPeriodicReporting();
    }
  }

  /**
   * Set up global debugging functions
   */
  _setupGlobalFunctions() {
    // Main system status check
    window.debugSystem = () => {
      return this.getSystemStatus();
    };

    // Message service diagnostics
    window.debugMessageService = () => {
      return this.getMessageServiceStatus();
    };

    // Cache performance analysis
    window.debugCache = () => {
      return this.getCacheStatus();
    };

    // Component health check
    window.debugComponents = () => {
      return this.getComponentStatus();
    };

    // Performance metrics
    window.debugPerformance = () => {
      return this.getPerformanceMetrics();
    };

    // Memory usage
    window.debugMemory = () => {
      return this.getMemoryUsage();
    };

    // Full system report
    window.debugFullReport = () => {
      const report = {
        timestamp: new Date().toISOString(),
        system: this.getSystemStatus(),
        messageService: this.getMessageServiceStatus(),
        cache: this.getCacheStatus(),
        components: this.getComponentStatus(),
        performance: this.getPerformanceMetrics(),
        memory: this.getMemoryUsage()
      };

      console.log('🔍 [SystemMonitor] Full System Report:', report);
      return report;
    };
  }

  /**
   * Get overall system status
   */
  getSystemStatus() {
    return {
      isHealthy: this._checkSystemHealth(),
      uptime: Date.now() - this.metrics.lastActivity,
      environment: import.meta.env.MODE,
      timestamp: new Date().toISOString(),
      services: {
        messageService: !!window.unifiedMessageService,
        messageDisplayGuarantee: !!window.messageDisplayGuarantee,
        authStore: !!window.__pinia_stores__?.auth,
        chatStore: !!window.__pinia_stores__?.chat
      }
    };
  }

  /**
   * Get message service status
   */
  getMessageServiceStatus() {
    if (!window.unifiedMessageService) {
      return { error: 'UnifiedMessageService not available' };
    }

    const service = window.unifiedMessageService;
    const cacheKeys = Object.keys(service.messageCache || {});
    const totalMessages = Array.from(service.messagesByChat?.values() || [])
      .reduce((total, messages) => total + (messages?.length || 0), 0);

    return {
      isInitialized: service.isInitialized?.value || false,
      isOnline: service.isOnline?.value || false,
      cachedChats: cacheKeys.length,
      totalCachedMessages: totalMessages,
      hasMoreFlags: service.hasMoreByChat?.size || 0,
      stats: service.stats || {},
      cacheConfig: service.cacheConfig || {}
    };
  }

  /**
   * Get cache performance status
   */
  getCacheStatus() {
    if (!window.unifiedMessageService) {
      return { error: 'UnifiedMessageService not available' };
    }

    const service = window.unifiedMessageService;
    const now = Date.now();
    const caches = service.messageCache || {};

    const cacheAnalysis = Object.entries(caches).map(([chatId, cache]) => ({
      chatId,
      messageCount: cache.messages?.length || 0,
      age: Math.round((now - (cache.timestamp || 0)) / 1000), // seconds
      isFresh: cache.isFresh || false,
      isStale: (now - (cache.timestamp || 0)) > (5 * 60 * 1000) // 5 minutes
    }));

    return {
      totalCaches: cacheAnalysis.length,
      freshCaches: cacheAnalysis.filter(c => c.isFresh).length,
      staleCaches: cacheAnalysis.filter(c => c.isStale).length,
      averageAge: cacheAnalysis.length > 0
        ? Math.round(cacheAnalysis.reduce((sum, c) => sum + c.age, 0) / cacheAnalysis.length)
        : 0,
      totalCachedMessages: cacheAnalysis.reduce((sum, c) => sum + c.messageCount, 0),
      cacheDetails: cacheAnalysis,
      hitRate: this.metrics.cacheHits / (this.metrics.cacheHits + this.metrics.cacheMisses) || 0
    };
  }

  /**
   * Get component status
   */
  getComponentStatus() {
    const components = {
      simpleMessageList: document.querySelectorAll('.simple-message-list').length,
      messageItems: document.querySelectorAll('.message-wrapper').length,
      loadingIndicators: document.querySelectorAll('.auto-load-indicator').length,
      visibleMessages: document.querySelectorAll('.message-wrapper:not([style*="display: none"])').length
    };

    return {
      mountedComponents: this.metrics.componentMounts,
      domElements: components,
      circuitBreakerStates: this._getCircuitBreakerStates()
    };
  }

  /**
   * Get performance metrics
   */
  getPerformanceMetrics() {
    return {
      messageServiceCalls: this.metrics.messageServiceCalls,
      autoLoadTriggers: this.metrics.autoLoadTriggers,
      cacheHitRate: this.metrics.cacheHits / (this.metrics.cacheHits + this.metrics.cacheMisses) || 0,
      averageResponseTime: this._calculateAverageResponseTime(),
      lastActivity: new Date(this.metrics.lastActivity).toLocaleTimeString()
    };
  }

  /**
   * Get memory usage information
   */
  getMemoryUsage() {
    if (!performance.memory) {
      return { error: 'Memory API not available' };
    }

    return {
      usedJSHeapSize: Math.round(performance.memory.usedJSHeapSize / 1024 / 1024), // MB
      totalJSHeapSize: Math.round(performance.memory.totalJSHeapSize / 1024 / 1024), // MB
      jsHeapSizeLimit: Math.round(performance.memory.jsHeapSizeLimit / 1024 / 1024), // MB
      memoryPressure: performance.memory.usedJSHeapSize / performance.memory.jsHeapSizeLimit
    };
  }

  /**
   * Check overall system health with improved resilience
   */
  _checkSystemHealth() {
    const checks = [
      {
        name: 'unifiedMessageService',
        test: !!window.unifiedMessageService,
        critical: true
      },
      {
        name: 'messageDisplayGuarantee',
        test: !!window.messageDisplayGuarantee,
        critical: true
      },
      {
        name: 'simple-message-list-dom',
        test: document.querySelectorAll('.simple-message-list').length > 0,
        critical: false // Not critical - user might not be on chat page
      }
    ];

    const failedChecks = checks.filter(check => !check.test);
    const criticalFailures = failedChecks.filter(check => check.critical);

    // Only report unhealthy if critical services are missing
    const isHealthy = criticalFailures.length === 0;

    if (!isHealthy && import.meta.env.DEV) {
      console.log('🔍 [SystemMonitor] Failed health checks:', failedChecks.map(c => c.name));
    }

    return isHealthy;
  }

  /**
   * Get circuit breaker states
   */
  _getCircuitBreakerStates() {
    // This would need to be implemented based on actual circuit breaker instances
    return {
      // Placeholder for circuit breaker monitoring
      autoLoadCircuitBreaker: 'unknown'
    };
  }

  /**
   * Calculate average response time
   */
  _calculateAverageResponseTime() {
    // Placeholder - would need actual timing measurements
    return 0;
  }

  /**
   * Start periodic reporting in development
   */
  _startPeriodicReporting() {
    if (!this.isEnabled) return;

    let consecutiveHealthyCount = 0;
    let consecutiveUnhealthyCount = 0;

    // Report system status every 30 seconds in development
    setInterval(() => {
      const status = this.getSystemStatus();

      if (status.isHealthy) {
        consecutiveHealthyCount++;
        consecutiveUnhealthyCount = 0;

        // Report recovery after being unhealthy
        if (consecutiveHealthyCount === 1) {
          console.log('✅ [SystemMonitor] System health restored');
        }
      } else {
        consecutiveUnhealthyCount++;
        consecutiveHealthyCount = 0;

        // Only report first few failures to avoid spam
        if (consecutiveUnhealthyCount <= 3) {
          console.warn('⚠️ [SystemMonitor] System health check failed:', status);

          if (consecutiveUnhealthyCount === 3) {
            console.log('🔇 [SystemMonitor] Further health check failures will be suppressed until recovery');
          }
        }
      }
    }, 30 * 1000);
  }

  /**
   * Record metric
   */
  recordMetric(type, value = 1) {
    if (this.metrics.hasOwnProperty(type)) {
      this.metrics[type] += value;
    }
    this.metrics.lastActivity = Date.now();
  }

  /**
   * Log performance event
   */
  logPerformanceEvent(event, details = {}) {
    if (!this.isEnabled) return;

    console.log(`📊 [SystemMonitor] ${event}:`, {
      timestamp: new Date().toISOString(),
      ...details
    });
  }
}

// Create global instance
const systemMonitor = new SystemMonitor();

// Export for module usage
export { systemMonitor };

// Make available globally in development
if (import.meta.env.DEV) {
  window.systemMonitor = systemMonitor;
} 