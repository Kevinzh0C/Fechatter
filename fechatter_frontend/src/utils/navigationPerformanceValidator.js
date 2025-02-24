/**
 * Navigation Performance Validator
 * Validates that navigation fixes are working correctly
 */

class NavigationPerformanceValidator {
  constructor() {
    this.metrics = {
      stateClears: 0,
      messageSetOperations: 0,
      virtualScrollRecalculations: 0,
      cacheHits: 0,
      cacheMisses: 0,
      navigationStartTime: 0,
      navigationEndTime: 0
    };

    this.listeners = [];
    this.isValidating = false;
  }

  /**
   * Start validation for a chat navigation
   */
  startValidation(chatId) {
    this.isValidating = true;
    this.metrics = {
      stateClears: 0,
      messageSetOperations: 0,
      virtualScrollRecalculations: 0,
      cacheHits: 0,
      cacheMisses: 0,
      navigationStartTime: performance.now(),
      navigationEndTime: 0
    };

    if (import.meta.env.DEV) {
      console.log(`🔍 [PERF_VALIDATOR] Starting validation for chat ${chatId}`);
    }

    // Monitor console logs for our performance indicators
    this.setupConsoleMonitoring();

    return this;
  }

  /**
   * Setup console monitoring to track our fixes
   */
  setupConsoleMonitoring() {
    const originalLog = console.log;
    const validator = this;

    console.log = function (...args) {
      if (validator.isValidating) {
        const message = args.join(' ');

        // Track state clear operations
        if (message.includes('🧹 [ChatStore] Cleared messages')) {
          validator.metrics.stateClears++;
        }

        // Track setMessages operations
        if (message.includes('📦 [DEDUP] setMessages')) {
          validator.metrics.messageSetOperations++;
        }

        // Track virtual scroll updates
        if (message.includes('📈 [VIRTUAL] Messages')) {
          validator.metrics.virtualScrollRecalculations++;
        }

        // Track cache usage
        if (message.includes('📦 [PARALLEL_FETCH] Using cached messages')) {
          validator.metrics.cacheHits++;
        }

        if (message.includes('🌐 [PARALLEL_FETCH] No valid cache')) {
          validator.metrics.cacheMisses++;
        }
      }

      return originalLog.apply(console, args);
    };
  }

  /**
   * End validation and generate report
   */
  endValidation() {
    this.isValidating = false;
    this.metrics.navigationEndTime = performance.now();

    const totalTime = this.metrics.navigationEndTime - this.metrics.navigationStartTime;

    const report = {
      totalNavigationTime: Math.round(totalTime),
      metrics: this.metrics,
      performance: this.evaluatePerformance(),
      recommendations: this.generateRecommendations()
    };

    if (import.meta.env.DEV) {
      console.log('🔍 [PERF_VALIDATOR] Navigation Performance Report:', report);
    }

    return report;
  }

  /**
   * Evaluate performance based on metrics
   */
  evaluatePerformance() {
    const score = {
      stateManagement: this.metrics.stateClears <= 1 ? 'GOOD' : 'NEEDS_IMPROVEMENT',
      renderingEfficiency: this.metrics.messageSetOperations <= 2 ? 'GOOD' : 'NEEDS_IMPROVEMENT',
      cacheEfficiency: this.metrics.cacheHits > this.metrics.cacheMisses ? 'GOOD' : 'POOR',
      overall: 'CALCULATING'
    };

    const goodCount = Object.values(score).filter(s => s === 'GOOD').length;
    score.overall = goodCount >= 2 ? 'GOOD' : 'NEEDS_IMPROVEMENT';

    return score;
  }

  /**
   * Generate recommendations based on metrics
   */
  generateRecommendations() {
    const recommendations = [];

    if (this.metrics.stateClears > 1) {
      recommendations.push('Reduce state clearing operations during navigation');
    }

    if (this.metrics.messageSetOperations > 2) {
      recommendations.push('Optimize message setting operations to prevent triple writes');
    }

    if (this.metrics.cacheMisses > this.metrics.cacheHits) {
      recommendations.push('Improve cache hit rate for better performance');
    }

    if (this.metrics.virtualScrollRecalculations > 3) {
      recommendations.push('Reduce virtual scroll recalculations');
    }

    return recommendations;
  }

  /**
   * Quick test method for manual validation
   */
  static async testNavigationPerformance(fromChatId, toChatId) {
    const { useChatStore } = await import('@/stores/chat');
    const chatStore = useChatStore();

    const validator = new NavigationPerformanceValidator();
    validator.startValidation(toChatId);

    try {
      await chatStore.navigateToChat(toChatId);

      // Wait for all async operations to complete
      await new Promise(resolve => setTimeout(resolve, 500));

      return validator.endValidation();
    } catch (error) {
      if (import.meta.env.DEV) {
        console.error('Navigation test failed:', error);
      }
      return validator.endValidation();
    }
  }
}

// Export for use in browser console
if (typeof window !== 'undefined') {
  window.testNavigationPerformance = NavigationPerformanceValidator.testNavigationPerformance;
  window.NavigationPerformanceValidator = NavigationPerformanceValidator;
}

export default NavigationPerformanceValidator; 