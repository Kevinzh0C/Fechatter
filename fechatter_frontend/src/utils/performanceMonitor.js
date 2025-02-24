/**
 * Performance Monitor for Fechatter
 * Tracks and reports on chat switching performance metrics
 */

class PerformanceMonitor {
  constructor() {
    this.metrics = new Map();
    this.enabled = process.env.NODE_ENV === 'development';
    this.thresholds = {
      excellent: 100,   // < 100ms
      good: 300,        // < 300ms
      fair: 1000,       // < 1s
      poor: 3000        // < 3s
    };
    this.initialized = false;
  }

  /**
   * Initialize the performance monitor
   */
  initialize() {
    if (this.initialized) return;

    this.initialized = true;

    if (this.enabled) {
      console.log('📊 Performance Monitor initialized');

      // Start tracking app initialization
      this.startOperation('app-initialization', {
        type: 'system',
        userAgent: navigator.userAgent,
        timestamp: Date.now()
      });
    }
  }

  /**
   * Start tracking a performance metric
   * @param {string} operation - Operation name (e.g., 'chat-switch-5')
   * @param {Object} metadata - Additional metadata
   */
  startOperation(operation, metadata = {}) {
    if (!this.enabled) return;

    this.metrics.set(operation, {
      startTime: performance.now(),
      metadata,
      marks: []
    });
  }

  /**
   * Add a mark/checkpoint to an operation
   * @param {string} operation - Operation name
   * @param {string} mark - Mark name (e.g., 'cache-hit', 'api-start')
   */
  mark(operation, mark) {
    if (!this.enabled) return;

    const metric = this.metrics.get(operation);
    if (!metric) return;

    metric.marks.push({
      name: mark,
      time: performance.now() - metric.startTime
    });
  }

  /**
   * End tracking and report results
   * @param {string} operation - Operation name
   * @param {Object} result - Operation result data
   */
  endOperation(operation, result = {}) {
    if (!this.enabled) return;

    const metric = this.metrics.get(operation);
    if (!metric) return;

    const duration = performance.now() - metric.startTime;
    const rating = this.getRating(duration);

    const report = {
      operation,
      duration: Math.round(duration),
      rating,
      ...metric.metadata,
      ...result,
      marks: metric.marks,
      timestamp: new Date().toISOString()
    };

    // Log performance report
    this.logReport(report);

    // Store for analysis
    this.storeReport(report);

    // Clean up
    this.metrics.delete(operation);

    return report;
  }

  /**
   * Get performance rating based on duration
   * @param {number} duration - Duration in milliseconds
   * @returns {string} Rating (excellent/good/fair/poor)
   */
  getRating(duration) {
    if (duration < this.thresholds.excellent) return 'excellent';
    if (duration < this.thresholds.good) return 'good';
    if (duration < this.thresholds.fair) return 'fair';
    if (duration < this.thresholds.poor) return 'poor';
    return 'critical';
  }

  /**
   * Log performance report with color coding
   * @param {Object} report - Performance report
   */
  logReport(report) {
    const emoji = {
      excellent: '🚀',
      good: '✅',
      fair: '⚡',
      poor: '🐌',
      critical: '❌'
    };

    const color = {
      excellent: 'color: #10b981',
      good: 'color: #3b82f6',
      fair: 'color: #f59e0b',
      poor: 'color: #ef4444',
      critical: 'color: #dc2626'
    };

    console.log(
      `${emoji[report.rating]} [PERF] ${report.operation}: ${report.duration}ms (${report.rating})`,
      color[report.rating]
    );

    if (report.marks.length > 0) {
      console.log('  Breakdown:', report.marks.map(m => `${m.name}: ${Math.round(m.time)}ms`).join(', '));
    }
  }

  /**
   * Store report for later analysis
   * @param {Object} report - Performance report
   */
  storeReport(report) {
    try {
      const stored = JSON.parse(localStorage.getItem('perf_reports') || '[]');
      stored.push(report);

      // Keep only last 100 reports
      if (stored.length > 100) {
        stored.splice(0, stored.length - 100);
      }

      localStorage.setItem('perf_reports', JSON.stringify(stored));
    } catch (error) {
      // Ignore storage errors
    }
  }

  /**
   * Get performance analytics
   * @returns {Object} Analytics summary
   */
  getAnalytics() {
    try {
      const reports = JSON.parse(localStorage.getItem('perf_reports') || '[]');

      if (reports.length === 0) {
        return { message: 'No performance data available' };
      }

      // Group by operation type
      const grouped = reports.reduce((acc, report) => {
        const type = report.operation.split('-')[0];
        if (!acc[type]) acc[type] = [];
        acc[type].push(report);
        return acc;
      }, {});

      // Calculate stats for each type
      const stats = {};
      for (const [type, typeReports] of Object.entries(grouped)) {
        const durations = typeReports.map(r => r.duration);
        stats[type] = {
          count: typeReports.length,
          avg: Math.round(durations.reduce((a, b) => a + b, 0) / durations.length),
          min: Math.min(...durations),
          max: Math.max(...durations),
          ratings: typeReports.reduce((acc, r) => {
            acc[r.rating] = (acc[r.rating] || 0) + 1;
            return acc;
          }, {})
        };
      }

      return {
        totalReports: reports.length,
        stats,
        recentReports: reports.slice(-10).reverse()
      };
    } catch (error) {
      return { error: 'Failed to load analytics' };
    }
  }

  /**
   * Clear all stored reports
   */
  clearReports() {
    localStorage.removeItem('perf_reports');
    if (import.meta.env.DEV) {
      console.log('✅ [PERF] Performance reports cleared');
    }
  }
}

// Create singleton instance
const performanceMonitor = new PerformanceMonitor();

// Export for use in components
export default performanceMonitor;

// Add to window for debugging
if (process.env.NODE_ENV === 'development') {
  window.performanceMonitor = performanceMonitor;
  window.perfAnalytics = () => {
    const analytics = performanceMonitor.getAnalytics();
    if (import.meta.env.DEV) {
      console.log('📊 Performance Analytics:', analytics);
    }
    return analytics;
  };

  console.log('📊 Performance Monitor loaded - use window.perfAnalytics() to view stats');
} 