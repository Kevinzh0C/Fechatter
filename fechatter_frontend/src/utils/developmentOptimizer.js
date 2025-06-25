/**
 * Development Environment Optimizer
 * 
 * 🎯 Purpose: Reduce development noise and improve developer experience
 * 🔧 Features: Smart error filtering, health check optimization, performance monitoring
 */

class DevelopmentOptimizer {
  constructor() {
    this.isDevelopment = import.meta.env.DEV;
    this.config = this.getOptimizedConfig();

    if (this.isDevelopment) {
      this.initializeDevelopmentMode();
    }
  }

  /**
   * Get optimized configuration for development environment
   */
  getOptimizedConfig() {
    return {
      // Health check optimization
      healthCheck: {
        enabled: this.isDevelopment,
        startDelayMs: 30000, // 30 seconds delay
        intervalMs: 300000,  // 5 minutes interval
        timeoutMs: 10000,    // 10 seconds timeout
        retryAttempts: 1,    // Reduced retries
        silentFailures: true // Don't log network failures
      },

      // Error monitoring optimization
      errorMonitoring: {
        filterNetworkErrors: true,
        filterHealthCheckErrors: true,
        filterHMRErrors: true,
        maxErrorsToShow: 5,
        suppressCriticalAlerts: false
      },

      // Performance monitoring
      performance: {
        enableDetailedLogs: false,
        trackRenderTimes: false,
        trackMemoryUsage: false,
        enableProfiler: false
      },

      // API configuration
      api: {
        extendedTimeouts: true,
        retryNetworkErrors: true,
        gracefulDegradation: true,
        mockUnavailableServices: false
      }
    };
  }

  /**
   * Initialize development mode optimizations
   */
  initializeDevelopmentMode() {
    console.log('🔧 [DEV_OPTIMIZER] Development mode optimizations enabled');

    // Setup global error filtering
    this.setupGlobalErrorFiltering();

    // Setup console enhancement
    this.enhanceConsoleLogging();

    // Setup development shortcuts
    this.setupDevelopmentShortcuts();

    // Monitor backend service availability
    this.startBackendServiceMonitoring();
  }

  /**
   * Setup intelligent error filtering for development
   */
  setupGlobalErrorFiltering() {
    const originalConsoleError = console.error;
    const originalConsoleWarn = console.warn;

    // Filter console errors
    console.error = (...args) => {
      if (this.shouldSuppressError(args)) {
        console.debug('🔇 [DEV] Suppressed error:', ...args);
        return;
      }
      originalConsoleError.apply(console, args);
    };

    // Filter console warnings
    console.warn = (...args) => {
      if (this.shouldSuppressWarning(args)) {
        console.debug('🔇 [DEV] Suppressed warning:', ...args);
        return;
      }
      originalConsoleWarn.apply(console, args);
    };
  }

  /**
   * Check if error should be suppressed in development
   */
  shouldSuppressError(args) {
    const errorStr = args.join(' ').toLowerCase();

    const suppressPatterns = [
      /failed to fetch/i,
      /network error/i,
      /connection refused/i,
      /health.*check.*failed/i,
      /gateway.*not.*running/i,
      /cors.*error/i,
      /timeout.*error/i
    ];

    return suppressPatterns.some(pattern => pattern.test(errorStr));
  }

  /**
   * Check if warning should be suppressed in development
   */
  shouldSuppressWarning(args) {
    const warningStr = args.join(' ').toLowerCase();

    const suppressPatterns = [
      /backend.*not.*started/i,
      /services.*not.*available/i,
      /health.*monitoring/i,
      /connection.*failed/i
    ];

    return suppressPatterns.some(pattern => pattern.test(warningStr));
  }

  /**
   * Enhance console logging for development
   */
  enhanceConsoleLogging() {
    // Add timestamp and context to logs
    const originalLog = console.log;
    console.log = (...args) => {
      const timestamp = new Date().toLocaleTimeString();
      originalLog.apply(console, [`[${timestamp}]`, ...args]);
    };

    // Create developer-friendly log levels
    console.dev = (...args) => {
      console.log('🔧 [DEV]', ...args);
    };

    console.health = (...args) => {
      console.log('🏥 [HEALTH]', ...args);
    };

    console.api = (...args) => {
      console.log('🌐 [API]', ...args);
    };
  }

  /**
   * Setup development shortcuts and helpers
   */
  setupDevelopmentShortcuts() {
    // Global development helpers
    window.dev = {
      // Health check helpers
      runHealthCheck: () => {
        import('./healthCheck.js').then(({ runHealthCheck }) => {
          runHealthCheck();
        });
      },

      // Error monitoring helpers
      clearErrors: () => {
        import('./errorMonitor.js').then(({ clearErrors }) => {
          clearErrors();
        });
      },

      // API testing helpers
      testAPI: async () => {
        try {
          const { default: api } = await import('../services/api.js');
          const response = await api.get('/health');
          console.api('API Test Success:', response.status, response.data);
        } catch (error) {
          console.api('API Test Failed:', error.message);
        }
      },

      // Configuration helpers
      showConfig: () => {
        console.dev('Current Configuration:', this.config);
      },

      // Performance helpers
      memoryUsage: () => {
        if (performance.memory) {
          console.dev('Memory Usage:', {
            used: Math.round(performance.memory.usedJSHeapSize / 1024 / 1024) + ' MB',
            total: Math.round(performance.memory.totalJSHeapSize / 1024 / 1024) + ' MB',
            limit: Math.round(performance.memory.jsHeapSizeLimit / 1024 / 1024) + ' MB'
          });
        }
      }
    };

    console.dev('Development helpers available at window.dev');
  }

  /**
   * Monitor backend service availability
   */
  startBackendServiceMonitoring() {
    const checkInterval = 60000; // 1 minute
    let consecutiveFailures = 0;
    let notificationShown = false;

    const checkBackendServices = async () => {
      try {
        const response = await fetch('http://127.0.0.1:8080/health', {
          method: 'GET',
          signal: AbortSignal.timeout(5000)
        });

        if (response.ok) {
          consecutiveFailures = 0;
          if (notificationShown) {
            console.dev('✅ Backend services are now available');
            notificationShown = false;
          }
        } else {
          throw new Error(`Gateway returned ${response.status}`);
        }
      } catch (error) {
        consecutiveFailures++;

        // Show notification after 3 consecutive failures
        if (consecutiveFailures >= 3 && !notificationShown) {
          console.dev('🔌 Backend services unavailable. Run ./scripts/start-dev.sh to start them.');
          notificationShown = true;
        }
      }
    };

    // Initial check after delay
    setTimeout(checkBackendServices, 10000);

    // Periodic checks
    setInterval(checkBackendServices, checkInterval);
  }

  /**
   * Get configuration for specific component
   */
  getConfig(component) {
    return this.config[component] || {};
  }

  /**
   * Check if feature should be enabled in development
   */
  shouldEnable(feature) {
    return this.isDevelopment && this.config[feature]?.enabled !== false;
  }

  /**
   * Get optimized timeout for development
   */
  getTimeout(defaultTimeout = 5000) {
    return this.isDevelopment ? Math.max(defaultTimeout, 10000) : defaultTimeout;
  }
}

// Create singleton instance
const developmentOptimizer = new DevelopmentOptimizer();

// Export convenience methods
export default developmentOptimizer;
export const getDevConfig = (component) => developmentOptimizer.getConfig(component);
export const shouldEnable = (feature) => developmentOptimizer.shouldEnable(feature);
export const getDevTimeout = (defaultTimeout) => developmentOptimizer.getTimeout(defaultTimeout); 