/**
 * Console Monitor
 * Real-time monitoring of console activities to debug error suppression
 * Production-grade implementation following Occam's Razor principle
 */

class ConsoleMonitor {
  constructor() {
    this.logs = [];
    this.maxLogs = 100;
    this.listeners = new Set();
    this.originalMethods = {};
    this.isActive = false;

    // Categories to track
    this.categories = {
      errors: 0,
      warnings: 0,
      logs: 0,
      debugs: 0,
      suppressedErrors: 0
    };
  }

  /**
   * Start monitoring console
   */
  start() {
    if (this.isActive) return;

    // Save original console methods
    this.originalMethods = {
      log: console.log,
      warn: console.warn,
      debug: console.debug
    };

    // Register error monitoring with unified handler
    if (window.unifiedErrorHandler) {
      window.unifiedErrorHandler.registerHandler(
        'consoleMonitor',
        ({ args, errorString }) => {
          // Record the log
          const entry = {
            type: 'error',
            timestamp: new Date().toISOString(),
            args: args,
            stack: new Error().stack,
            suppressed: false
          };

          // Check if this was suppressed
          if (window.contentScriptSuppressor?.isContentScriptError(errorString)) {
            entry.suppressed = true;
            this.categories.suppressedErrors++;
          }

          // Update categories
          this.categories.errors++;

          // Add to logs
          this.logs.push(entry);
          if (this.logs.length > this.maxLogs) {
            this.logs.shift();
          }

          // Notify listeners
          this.notifyListeners(entry);

          return 'pass'; // Always pass through for monitoring
        },
        90 // Low priority - just monitoring
      );
    }

    // Override other console methods (not error)
    console.log = this.createWrapper('log', this.originalMethods.log);
    console.warn = this.createWrapper('warn', this.originalMethods.warn);
    console.debug = this.createWrapper('debug', this.originalMethods.debug);

    this.isActive = true;
    if (import.meta.env.DEV) {
      console.log('Console Monitor started');
    }

  /**
   * Create wrapper for console method
   */
  createWrapper(type, originalMethod) {
    return (...args) => {
      // Record the log
      const entry = {
        type,
        timestamp: new Date().toISOString(),
        args: args,
        stack: new Error().stack,
        suppressed: false
      };

      // Check if this was suppressed
      if (type === 'error') {
        const errorString = args.join(' ');
        if (window.contentScriptSuppressor?.isContentScriptError(errorString)) {
          entry.suppressed = true;
          this.categories.suppressedErrors++;
        }

      // Update categories
      this.categories[type + 's']++;

      // Add to logs
      this.logs.push(entry);
      if (this.logs.length > this.maxLogs) {
        this.logs.shift();
      }

      // Notify listeners
      this.notifyListeners(entry);

      // Call original method
      originalMethod.apply(console, args);
    };
  }

  /**
   * Stop monitoring
   */
  stop() {
    if (!this.isActive) return;

    // Restore original console methods
    Object.keys(this.originalMethods).forEach(method => {
      console[method] = this.originalMethods[method];
    });

    this.isActive = false;
    this.originalMethods.log('Console Monitor stopped');
  }

  /**
   * Get recent logs
   */
  getRecentLogs(count = 10, filter = null) {
    let logs = this.logs.slice(-count);

    if (filter) {
      logs = logs.filter(log => {
        if (filter.type && log.type !== filter.type) return false;
        if (filter.suppressed !== undefined && log.suppressed !== filter.suppressed) return false;
        if (filter.pattern && !log.args.some(arg => String(arg).includes(filter.pattern))) return false;
        return true;
      });

    return logs;
  }

  /**
   * Get statistics
   */
  getStats() {
    return {
      ...this.categories,
      totalLogs: this.logs.length,
      isActive: this.isActive,
      suppressionRate: this.categories.errors > 0
        ? (this.categories.suppressedErrors / this.categories.errors * 100).toFixed(2) + '%'
        : '0%'
    };
  }

  /**
   * Add listener for console events
   */
  addListener(callback) {
    this.listeners.add(callback);
    return () => this.listeners.delete(callback);
  }

  /**
   * Notify listeners
   */
  notifyListeners(entry) {
    this.listeners.forEach(callback => {
      try {
        callback(entry);
      } catch (e) {
        // Ignore listener errors
      }
    });

  /**
   * Generate report
   */
  generateReport() {
    const stats = this.getStats();
    const recentErrors = this.getRecentLogs(10, { type: 'error' });
    const suppressedErrors = this.getRecentLogs(10, { suppressed: true });

    console.group('Console Monitor Report');
    if (import.meta.env.DEV) {
      console.log('Statistics:', stats);
    if (import.meta.env.DEV) {
      console.log('\nRecent Errors:');
    recentErrors.forEach((log, i) => {
      if (import.meta.env.DEV) {
        console.log(`  ${i + 1}. [${log.timestamp}] ${log.suppressed ? '🔇' : '❌'}`, log.args[0]);
      }
    });
    if (import.meta.env.DEV) {
      console.log('\nSuppressed Errors:');
    suppressedErrors.forEach((log, i) => {
      if (import.meta.env.DEV) {
        console.log(`  ${i + 1}. [${log.timestamp}]`, log.args[0]);
      }
    });
    console.groupEnd();
  }

  /**
   * Clear logs
   */
  clear() {
    this.logs = [];
    Object.keys(this.categories).forEach(key => {
      this.categories[key] = 0;
    });

// Create singleton instance
const consoleMonitor = new ConsoleMonitor();

// Auto-start in development
if (import.meta.env.DEV && typeof window !== 'undefined') {
  // Wait a bit to ensure other systems are initialized
  setTimeout(() => {
    consoleMonitor.start();
    if (import.meta.env.DEV) {
      console.log('Console Monitor available: window.consoleMonitor');
    }
  }, 100);
}

// Expose globally
if (typeof window !== 'undefined') {
  window.consoleMonitor = consoleMonitor;
}

export default consoleMonitor; 