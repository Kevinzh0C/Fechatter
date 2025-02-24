/**
 * 🔄 REFACTOR: Message Tracking Logger Service
 * 
 * Separates logging concerns from business logic:
 * - Cross-cutting concern: Centralized logging
 * - Environment-aware: Production vs Development
 * - Configurable: Different log levels and outputs
 * - Testable: Can be mocked and disabled
 */

export class MessageTrackingLogger {
  constructor(config = {}) {
    this.config = {
      environment: config.environment || (import.meta.env.DEV ? 'development' : 'production'),
      enableConsoleLogging: config.enableConsoleLogging ?? import.meta.env.DEV,
      enableMetricsLogging: config.enableMetricsLogging ?? true,
      enableErrorReporting: config.enableErrorReporting ?? true,
      logLevel: config.logLevel || 'info', // 'debug', 'info', 'warn', 'error'
      ...config
    };

    this.logBuffer = [];
    this.maxBufferSize = 1000;
    this.metricsCollector = new MessageMetricsCollector();

    // 🔧 Structured logging categories
    this.categories = {
      FETCH: 'message_fetch',
      TRACKING: 'message_tracking',
      VISIBILITY: 'visibility_detection',
      PERFORMANCE: 'performance',
      ERROR: 'error',
      CLEANUP: 'cleanup'
    };
  }

  /**
   * 🛡️ FETCHING LOGS: Track message fetching lifecycle
   */
  logFetchStart(chatId, options) {
    if (!this.shouldLog('fetching', 'info')) return;

    this.log('info', this.categories.FETCH, 'fetch-start', {
      message: `📥 Fetching messages for chat ${chatId}`,
      chatId: parseInt(chatId),
      options: this.sanitizeOptions(options),
      timestamp: Date.now()
    });
  }

  logFetchSuccess(chatId, messageCount, duration) {
    if (!this.shouldLog('fetching', 'info')) return;

    this.log('info', this.categories.FETCH, 'fetch-success', {
      message: `✅ Fetched ${messageCount} messages for chat ${chatId} in ${duration}ms`,
      chatId: parseInt(chatId),
      messageCount,
      duration,
      performance: this.getPerformanceMetrics(duration, messageCount),
      timestamp: Date.now()
    });
  }

  logFetchSkipped(chatId, reason, messageCount = 0) {
    if (!this.shouldLog('fetching', 'info')) return;

    this.log('info', this.categories.FETCH, 'fetch-skipped', {
      message: `⚠️ Skipping tracking for chat ${chatId}: ${reason}`,
      chatId: parseInt(chatId),
      reason,
      messageCount,
      timestamp: Date.now()
    });
  }

  /**
   * 🛡️ TRACKING LOGS: Track message display guarantee lifecycle
   */
  logTrackingStart(chatId, messageIds, trackingId) {
    if (!this.shouldLog('tracking', 'info')) return;

    this.log('info', this.categories.TRACKING, 'tracking-start', {
      message: `🛡️ Started tracking ${messageIds.length} messages for chat ${chatId}`,
      chatId: parseInt(chatId),
      messageIds: messageIds.slice(0, 10), // Limit logged IDs for performance
      messageCount: messageIds.length,
      trackingId,
      timestamp: Date.now()
    });

    this.metricsCollector.recordTrackingStart(chatId, messageIds.length);
  }

  logTrackingSuccess(trackingId, chatId, displayedCount, totalCount) {
    if (!this.shouldLog('tracking', 'info')) return;

    this.log('info', this.categories.TRACKING, 'tracking-success', {
      message: `✅ Successfully tracked ${displayedCount}/${totalCount} messages for chat ${chatId}`,
      trackingId,
      chatId: parseInt(chatId),
      displayedCount,
      totalCount,
      completionRate: ((displayedCount / totalCount) * 100).toFixed(1) + '%',
      timestamp: Date.now()
    });
  }

  logTrackingContext(contextData) {
    if (!this.shouldLog('tracking', 'debug')) return;

    this.log('debug', this.categories.TRACKING, 'context-state', {
      message: `🔍 Active tracking contexts`,
      activeContexts: contextData.map(ctx => ({
        trackingId: ctx.trackingId,
        chatId: ctx.chatId,
        messageCount: ctx.messageIds?.length || 0,
        displayedCount: ctx.displayedIds?.length || 0,
        status: ctx.status,
        age: ctx.age
      })),
      totalContexts: contextData.length,
      timestamp: Date.now()
    });
  }

  logTrackingNotFound(messageId, chatId, activeContexts) {
    if (!this.shouldLog('tracking', 'error')) return;

    this.log('error', this.categories.TRACKING, 'context-not-found', {
      message: `❌ No tracking context found for message ${messageId} in chat ${chatId}`,
      messageId: parseInt(messageId),
      chatId: parseInt(chatId),
      activeContexts: activeContexts.map(ctx => ({
        trackingId: ctx.trackingId,
        chatId: ctx.chatId,
        messageCount: ctx.messageIds?.length || 0,
        status: ctx.status
      })),
      troubleshooting: {
        totalActiveContexts: activeContexts.length,
        hasContextsForThisChat: activeContexts.some(ctx => ctx.chatId === parseInt(chatId)),
        possibleCauses: this.generateTroubleshootingHints(messageId, chatId, activeContexts)
      },
      timestamp: Date.now()
    });
  }

  /**
   * 🧹 CLEARING LOGS: Track chat switching and cleanup
   */
  logClearingStart(chatId, callStack) {
    if (!this.shouldLog('clearing', 'info')) return;

    this.log('info', this.categories.CLEANUP, 'clear-start', {
      message: `🧹 Clearing messages and tracking for chat ${chatId}`,
      chatId: parseInt(chatId),
      callStack: this.sanitizeCallStack(callStack),
      timestamp: Date.now()
    });
  }

  logClearingComplete(chatId, clearedContexts) {
    if (!this.shouldLog('clearing', 'info')) return;

    this.log('info', this.categories.CLEANUP, 'clear-complete', {
      message: `✅ Cleared ${clearedContexts} tracking contexts for chat ${chatId}`,
      chatId: parseInt(chatId),
      clearedContexts,
      timestamp: Date.now()
    });
  }

  /**
   * ⚠️ ERROR LOGS: Centralized error tracking
   */
  logError(category, error, context = {}) {
    if (!this.shouldLog('errors', 'error')) return;

    this.log('error', this.categories.ERROR, 'error-occurred', {
      message: `❌ Error in ${category}: ${error.message}`,
      category,
      error: {
        name: error.name,
        message: error.message,
        stack: this.config.environment === 'development' ? error.stack : undefined
      },
      context: this.sanitizeContext(context),
      timestamp: Date.now()
    });
  }

  /**
   * 🎯 CORE LOGGING METHOD
   */
  log(level, category, eventType, data) {
    if (!this.shouldLog(level)) return;

    const logEntry = {
      level,
      category,
      eventType,
      timestamp: Date.now(),
      ...data,
      environment: this.config.environment
    };

    // Add to buffer
    this.addToBuffer(logEntry);

    // Output based on configuration
    if (this.config.enableConsoleLogging) {
      this.outputToConsole(logEntry);
    }

    if (this.config.enableMetricsLogging) {
      this.outputToMetrics(logEntry);
    }

    if (level === 'error' && this.config.enableErrorReporting) {
      this.reportError(logEntry);
    }
  }

  /**
   * 🎨 CONSOLE OUTPUT: Formatted for development
   */
  outputToConsole(logEntry) {
    if (!this.config.enableConsoleLogging) return;

    const formatted = this.formatForConsole(logEntry);
    const consoleMethod = this.getConsoleMethod(logEntry.level);

    if (logEntry.level === 'error') {
      console.error(formatted.message, formatted.data);
    } else {
      console.log(formatted.message, formatted.data);
    }
  }

  formatForConsole(logEntry) {
    const emoji = this.getEmojiForEvent(logEntry.eventType);
    const prefix = `${emoji} [${logEntry.category.toUpperCase()}]`;

    return {
      message: `${prefix} ${logEntry.message}`,
      data: logEntry.troubleshooting || logEntry.context || {}
    };
  }

  /**
   * 📊 BUFFER OUTPUT: For production batch sending
   */
  outputToBuffer(logEntry) {
    this.logBuffer.push(logEntry);

    if (this.logBuffer.length >= this.maxBufferSize) {
      this.flushBuffer();
    }
  }

  flushBuffer() {
    if (this.logBuffer.length === 0) return;

    // In production, send to monitoring service
    if (this.config.environment === 'production') {
      this.sendToMonitoring(this.logBuffer);
    }

    this.logBuffer = [];
  }

  /**
   * 🔧 UTILITY METHODS
   */
  shouldLog(level) {
    const levels = {
      debug: 0,
      info: 1,
      warn: 2,
      error: 3
    };

    return levels[level] >= levels[this.config.logLevel];
  }

  getEmojiForEvent(eventType) {
    const emojiMap = {
      'fetch-start': '📥',
      'fetch-success': '✅',
      'fetch-skipped': '⚠️',
      'tracking-start': '🛡️',
      'tracking-success': '✅',
      'context-state': '🔍',
      'context-not-found': '❌',
      'clear-start': '🧹',
      'clear-complete': '✅',
      'error-occurred': '❌'
    };
    return emojiMap[eventType] || '📋';
  }

  getConsoleMethod(level) {
    switch (level) {
      case 'error': return 'error';
      case 'warn': return 'warn';
      case 'debug': return 'debug';
      default: return 'log';
    }
  }

  generateTroubleshootingHints(messageId, chatId, activeContexts) {
    const hints = [];

    if (activeContexts.length === 0) {
      hints.push('No tracking contexts exist - startMessageTracking may not have been called');
    }

    const hasOtherChats = activeContexts.some(ctx => ctx.chatId !== parseInt(chatId));
    if (hasOtherChats) {
      hints.push('Contexts exist for other chats - possible chat switching issue');
    }

    return hints;
  }

  sanitizeOptions(options) {
    return {
      limit: options.limit,
      isPreload: options.isPreload,
      abortSignal: options.abortSignal ? 'present' : 'absent'
    };
  }

  sanitizeCallStack(callStack) {
    // In production, don't expose full call stack
    return this.config.environment === 'development' ? callStack : '[call stack hidden in production]';
  }

  sanitizeContext(context) {
    // Remove sensitive data from context
    const sanitized = { ...context };
    delete sanitized.password;
    delete sanitized.token;
    delete sanitized.apiKey;
    return sanitized;
  }

  getPerformanceMetrics(duration, messageCount) {
    return {
      messagesPerSecond: messageCount / (duration / 1000),
      category: duration < 100 ? 'excellent' : duration < 500 ? 'good' : 'slow'
    };
  }

  sendToMonitoring(logs) {
    // Future: Send to external monitoring service
    // For now, just store locally or send to console in batches
    if (this.config.environment === 'development') {
      console.log('📊 [MONITORING] Batch log data:', logs);
    }
  }

  /**
   * 🧪 TESTING: Control logging for tests
   */
  enableCategory(category) {
    // Implementation needed
  }

  disableCategory(category) {
    // Implementation needed
  }

  setLogLevel(level) {
    this.config.logLevel = level;
  }

  getStats() {
    return {
      bufferSize: this.logBuffer.length,
      enabledCategories: Array.from(this.categories),
      logLevel: this.config.logLevel,
      environment: this.config.environment
    };
  }

  addToBuffer(logEntry) {
    this.logBuffer.push(logEntry);

    if (this.logBuffer.length > this.maxBufferSize) {
      this.logBuffer.shift(); // Remove oldest entry
    }
  }

  outputToMetrics(logEntry) {
    // Integration point for external metrics systems
    if (window.analytics?.track) {
      window.analytics.track('message_tracking_event', {
        category: logEntry.category,
        level: logEntry.level,
        message: logEntry.message,
        environment: logEntry.environment,
        timestamp: logEntry.timestamp
      });
    }

    // Emit event for other listeners
    const event = new CustomEvent('fechatter:tracking-log', {
      detail: logEntry
    });
    window.dispatchEvent(event);
  }

  reportError(logEntry) {
    // Integration point for error reporting services
    if (window.Sentry?.captureMessage) {
      window.Sentry.captureMessage(logEntry.message, {
        level: logEntry.level,
        tags: {
          category: logEntry.category,
          environment: logEntry.environment
        },
        extra: logEntry.data
      });
    }

    // Also emit error event
    const event = new CustomEvent('fechatter:tracking-error', {
      detail: logEntry
    });
    window.dispatchEvent(event);
  }
}

/**
 * 🏭 FACTORY: Create logger with environment-appropriate config
 */
export function createMessageTrackingLogger(config = {}) {
  const defaultConfig = {
    environment: import.meta.env.DEV ? 'development' : 'production',
    enableConsoleLogging: import.meta.env.DEV,
    enableMetricsLogging: true,
    enableErrorReporting: true,
    logLevel: import.meta.env.DEV ? 'debug' : 'warn',
    categories: import.meta.env.DEV
      ? ['tracking', 'fetching', 'clearing', 'errors']
      : ['errors'],
    outputs: import.meta.env.DEV ? ['console'] : ['buffer']
  };

  return new MessageTrackingLogger({ ...defaultConfig, ...config });
}

/**
 * MessageMetricsCollector - Handles performance and usage metrics
 */
class MessageMetricsCollector {
  constructor() {
    this.reset();
  }

  reset() {
    this.metrics = {
      trackingsStarted: 0,
      trackingsCompleted: 0,
      trackingsFailed: 0,
      messagesDisplayed: 0,
      visibilityChecks: 0,
      visibilityFailures: 0,
      averageTrackingTime: 0,
      performanceMetrics: new Map(),
      chatMetrics: new Map()
    };
  }

  recordTrackingStart(chatId, messageCount) {
    this.metrics.trackingsStarted++;

    if (!this.metrics.chatMetrics.has(chatId)) {
      this.metrics.chatMetrics.set(chatId, {
        trackings: 0,
        messagesDisplayed: 0,
        averageMessageCount: 0
      });
    }

    const chatMetric = this.metrics.chatMetrics.get(chatId);
    chatMetric.trackings++;
    chatMetric.averageMessageCount =
      (chatMetric.averageMessageCount + messageCount) / chatMetric.trackings;
  }

  recordTrackingComplete(chatId, summary) {
    this.metrics.trackingsCompleted++;

    if (summary.duration) {
      this.updateAverageTrackingTime(summary.duration);
    }
  }

  recordTrackingFailure(chatId, failure) {
    this.metrics.trackingsFailed++;
  }

  recordMessageDisplayed(messageId, chatId) {
    this.metrics.messagesDisplayed++;

    const chatMetric = this.metrics.chatMetrics.get(chatId);
    if (chatMetric) {
      chatMetric.messagesDisplayed++;
    }
  }

  recordVisibilityCheck(messageId, isVisible) {
    this.metrics.visibilityChecks++;

    if (!isVisible) {
      this.metrics.visibilityFailures++;
    }
  }

  recordPerformance(operation, duration) {
    if (!this.metrics.performanceMetrics.has(operation)) {
      this.metrics.performanceMetrics.set(operation, {
        count: 0,
        totalDuration: 0,
        averageDuration: 0,
        minDuration: Infinity,
        maxDuration: 0
      });
    }

    const metric = this.metrics.performanceMetrics.get(operation);
    metric.count++;
    metric.totalDuration += duration;
    metric.averageDuration = metric.totalDuration / metric.count;
    metric.minDuration = Math.min(metric.minDuration, duration);
    metric.maxDuration = Math.max(metric.maxDuration, duration);
  }

  updateAverageTrackingTime(newTime) {
    if (this.metrics.averageTrackingTime === 0) {
      this.metrics.averageTrackingTime = newTime;
    } else {
      this.metrics.averageTrackingTime =
        (this.metrics.averageTrackingTime * 0.8) + (newTime * 0.2);
    }
  }

  getSummary() {
    const successRate = this.metrics.trackingsStarted > 0
      ? ((this.metrics.trackingsCompleted / this.metrics.trackingsStarted) * 100).toFixed(2)
      : '100.00';

    const visibilitySuccessRate = this.metrics.visibilityChecks > 0
      ? (((this.metrics.visibilityChecks - this.metrics.visibilityFailures) / this.metrics.visibilityChecks) * 100).toFixed(2)
      : '100.00';

    return {
      trackings: {
        started: this.metrics.trackingsStarted,
        completed: this.metrics.trackingsCompleted,
        failed: this.metrics.trackingsFailed,
        successRate: `${successRate}%`
      },
      messages: {
        displayed: this.metrics.messagesDisplayed
      },
      visibility: {
        checks: this.metrics.visibilityChecks,
        failures: this.metrics.visibilityFailures,
        successRate: `${visibilitySuccessRate}%`
      },
      performance: {
        averageTrackingTime: this.metrics.averageTrackingTime,
        operations: Object.fromEntries(this.metrics.performanceMetrics)
      },
      chats: Object.fromEntries(this.metrics.chatMetrics)
    };
  }
} 