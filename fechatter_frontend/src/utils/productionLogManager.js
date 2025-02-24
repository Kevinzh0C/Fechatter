/**
 * Production-Level Log Manager
 * 
 * Intelligently filters and batches logs to reduce console noise
 * while maintaining essential debugging capabilities in development
 */

export class ProductionLogManager {
  constructor() {
    this.isDev = import.meta.env.DEV;
    this.logCounts = new Map(); // Track frequency of log messages
    this.suppressedLogs = new Set(); // Track which log types are suppressed
    this.batchedLogs = new Map(); // Batch similar logs together
    this.lastBatchFlush = Date.now();
    this.batchInterval = 2000; // Flush batches every 2 seconds

    // Configure log levels
    this.logLevels = {
      CRITICAL: 0,
      ERROR: 1,
      WARN: 2,
      INFO: 3,
      DEBUG: 4,
      VERBOSE: 5
    };

    this.currentLevel = this.isDev ? this.logLevels.DEBUG : this.logLevels.WARN;

    // Start batch processing
    this.startBatchProcessor();
  }

  /**
   * Smart log function that reduces noise
   */
  log(level, category, message, data = null, options = {}) {
    if (!this.shouldLog(level, category, message)) {
      return;
    }

    const logKey = `${category}:${message}`;

    // Check if this is a repetitive log
    if (this.isRepetitive(logKey)) {
      this.addToBatch(level, category, message, data, options);
      return;
    }

    // Log immediately for important messages
    this.outputLog(level, category, message, data, options);
    this.trackLogCount(logKey);
  }

  /**
   * Determine if log should be output
   */
  shouldLog(level, category, message) {
    // Always log critical and error messages
    if (level <= this.logLevels.ERROR) return true;

    // Check log level
    if (level > this.currentLevel) return false;

    // Check if category is suppressed
    if (this.suppressedLogs.has(category)) return false;

    return true;
  }

  /**
   * Check if message is repetitive
   */
  isRepetitive(logKey) {
    const count = this.logCounts.get(logKey) || 0;

    // Consider repetitive if seen more than 3 times in short period
    return count > 3;
  }

  /**
   * Add log to batch for later output
   */
  addToBatch(level, category, message, data, options) {
    const batchKey = `${level}:${category}:${message}`;

    if (!this.batchedLogs.has(batchKey)) {
      this.batchedLogs.set(batchKey, {
        level,
        category,
        message,
        count: 0,
        firstSeen: Date.now(),
        lastData: data,
        options
      });
    }

    this.batchedLogs.get(batchKey).count++;
    this.batchedLogs.get(batchKey).lastData = data;
  }

  /**
   * Output actual log message
   */
  outputLog(level, category, message, data, options = {}) {
    const prefix = this.getLogPrefix(level, category);
    const { skipConsole = false, skipCount = false } = options;

    if (skipConsole) return;

    if (data) {
      console.log(`${prefix} ${message}`, data);
    } else {
      console.log(`${prefix} ${message}`);
    }

    if (!skipCount) {
      this.trackLogCount(`${category}:${message}`);
    }
  }

  /**
   * Get appropriate log prefix with emoji
   */
  getLogPrefix(level, category) {
    const levelPrefixes = {
      [this.logLevels.CRITICAL]: '🚨',
      [this.logLevels.ERROR]: '❌',
      [this.logLevels.WARN]: '⚠️',
      [this.logLevels.INFO]: 'ℹ️',
      [this.logLevels.DEBUG]: '🔧',
      [this.logLevels.VERBOSE]: '📝'
    };

    return `${levelPrefixes[level]} [${category}]`;
  }

  /**
   * Track log frequency
   */
  trackLogCount(logKey) {
    const count = this.logCounts.get(logKey) || 0;
    this.logCounts.set(logKey, count + 1);

    // Auto-suppress if too frequent
    if (count > 10) {
      const [category] = logKey.split(':');
      this.suppressCategory(category, 5000); // Suppress for 5 seconds
    }
  }

  /**
   * Temporarily suppress a category
   */
  suppressCategory(category, duration) {
    this.suppressedLogs.add(category);

    setTimeout(() => {
      this.suppressedLogs.delete(category);
      console.log(`🔧 [LogManager] Re-enabled logging for category: ${category}`);
    }, duration);

    console.log(`🔇 [LogManager] Temporarily suppressing noisy category: ${category} for ${duration}ms`);
  }

  /**
   * Start batch processor
   */
  startBatchProcessor() {
    setInterval(() => {
      this.flushBatches();
    }, this.batchInterval);
  }

  /**
   * Flush batched logs
   */
  flushBatches() {
    if (this.batchedLogs.size === 0) return;

    console.group('📦 Batched Log Summary');

    for (const [key, batch] of this.batchedLogs.entries()) {
      if (batch.count > 1) {
        console.log(`${this.getLogPrefix(batch.level, batch.category)} ${batch.message} (${batch.count}x in ${this.batchInterval}ms)`, batch.lastData);
      }
    }

    console.groupEnd();

    this.batchedLogs.clear();
    this.lastBatchFlush = Date.now();
  }

  /**
   * Helper methods for different log levels
   */
  critical(category, message, data, options) {
    this.log(this.logLevels.CRITICAL, category, message, data, options);
  }

  error(category, message, data, options) {
    this.log(this.logLevels.ERROR, category, message, data, options);
  }

  warn(category, message, data, options) {
    this.log(this.logLevels.WARN, category, message, data, options);
  }

  info(category, message, data, options) {
    this.log(this.logLevels.INFO, category, message, data, options);
  }

  debug(category, message, data, options) {
    this.log(this.logLevels.DEBUG, category, message, data, options);
  }

  verbose(category, message, data, options) {
    this.log(this.logLevels.VERBOSE, category, message, data, options);
  }

  /**
   * Set log level
   */
  setLevel(level) {
    this.currentLevel = level;
    console.log(`🔧 [LogManager] Log level set to: ${Object.keys(this.logLevels)[level]}`);
  }

  /**
   * Enable/disable specific categories
   */
  enableCategory(category) {
    this.suppressedLogs.delete(category);
    console.log(`🔧 [LogManager] Enabled logging for category: ${category}`);
  }

  disableCategory(category) {
    this.suppressedLogs.add(category);
    console.log(`🔧 [LogManager] Disabled logging for category: ${category}`);
  }

  /**
   * Get logging statistics
   */
  getStats() {
    return {
      totalCategories: new Set(Array.from(this.logCounts.keys()).map(k => k.split(':')[0])).size,
      suppressedCategories: Array.from(this.suppressedLogs),
      batchedLogsCount: this.batchedLogs.size,
      currentLevel: Object.keys(this.logLevels)[this.currentLevel],
      mostFrequentLogs: Array.from(this.logCounts.entries())
        .sort((a, b) => b[1] - a[1])
        .slice(0, 5)
    };
  }
}

// Create singleton instance
export const productionLogManager = new ProductionLogManager();

// Global access for debugging
window.logManager = productionLogManager; 