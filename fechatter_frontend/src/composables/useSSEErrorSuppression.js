/**
 * SSE Error Suppression Composable
 * Prevents excessive logging of repetitive SSE connection errors
 */
import { ref, reactive } from 'vue';

// Global error tracking state
const errorTracker = reactive({
  errorCounts: new Map(),
  lastLogTime: new Map(),
  suppressionRules: new Map()
});

// Default suppression configuration
const DEFAULT_CONFIG = {
  maxErrorsPerMinute: 3,           // Maximum errors logged per minute
  suppressionDurationMs: 60000,    // 1 minute suppression after limit
  similarErrorWindowMs: 5000,      // Group similar errors within 5 seconds
  resetCounterMs: 300000,          // Reset error counter after 5 minutes
  logSampleRate: 0.1               // Log 10% of suppressed errors for monitoring
};

export function useSSEErrorSuppression(config = {}) {
  const settings = { ...DEFAULT_CONFIG, ...config };

  /**
   * Generate error key for grouping similar errors
   */
  const generateErrorKey = (error, context = '') => {
    // Create key based on error type, message pattern, and context
    const errorType = error.name || error.constructor?.name || 'Error';
    const messagePattern = error.message?.substring(0, 50) || 'unknown';
    const contextKey = typeof context === 'string' ? context : context?.context || '';

    return `${errorType}:${messagePattern}:${contextKey}`.toLowerCase();
  };

  /**
   * Check if error should be suppressed
   */
  const shouldSuppressError = (errorKey) => {
    const now = Date.now();
    const errorData = errorTracker.errorCounts.get(errorKey) || {
      count: 0,
      firstSeen: now,
      lastSeen: now,
      suppressed: false,
      suppressedUntil: 0
    };

    // Reset counter if enough time has passed
    if (now - errorData.firstSeen > settings.resetCounterMs) {
      errorData.count = 0;
      errorData.firstSeen = now;
      errorData.suppressed = false;
      errorData.suppressedUntil = 0;
    }

    // Check if currently in suppression period
    if (errorData.suppressed && now < errorData.suppressedUntil) {
      // Randomly sample some suppressed errors for monitoring
      if (Math.random() < settings.logSampleRate) {
        return false; // Allow this one through for sampling
      return true; // Suppress
    }

    // Update error data
    errorData.count++;
    errorData.lastSeen = now;

    // Check if we've exceeded the limit
    const timeWindow = now - errorData.firstSeen;
    const errorsPerMinute = (errorData.count / timeWindow) * 60000;

    if (errorsPerMinute > settings.maxErrorsPerMinute) {
      errorData.suppressed = true;
      errorData.suppressedUntil = now + settings.suppressionDurationMs;
      errorTracker.errorCounts.set(errorKey, errorData);

      // Log suppression notice
      if (import.meta.env.DEV) {
        console.warn(`🚫 [SSE_ERROR_SUPPRESSION] Suppressing further "${errorKey}" errors for ${settings.suppressionDurationMs / 1000}s (${errorData.count} errors in ${Math.round(timeWindow / 1000)}s)`);
      }

      return true; // Suppress
    }

    errorTracker.errorCounts.set(errorKey, errorData);
    return false; // Don't suppress
  };

  /**
   * Enhanced error handler with intelligent suppression
   */
  const handleSSEError = (error, context = '', options = {}) => {
    const {
      forceLog = false,
      silent = false,
      notify = false
    } = options;

    const errorKey = generateErrorKey(error, context);
    const shouldSuppress = !forceLog && shouldSuppressError(errorKey);
    const now = Date.now();

    if (shouldSuppress && !silent) {
      // Update last suppressed time for statistics
      errorTracker.lastLogTime.set(errorKey, now);
      return false; // Error was suppressed
    }

    // Log the error
    const errorData = errorTracker.errorCounts.get(errorKey);
    const contextStr = typeof context === 'string' ? context : context?.context || '';

    if (errorData?.suppressed && errorData.count > 1) {
      if (import.meta.env.DEV) {
        console.error(`🔌 [SSE_ERROR] ${contextStr}:`, error, `(${errorData.count - 1} similar errors suppressed)`);
      }
    } else {
      if (import.meta.env.DEV) {
        console.error(`🔌 [SSE_ERROR] ${contextStr}:`, error);
      }

    // Show notification if requested and not a network error
    if (notify && !isNetworkError(error)) {
      const { notifyError } = useNotifications();
      notifyError('Connection issue detected. Retrying in background.');
    }

    errorTracker.lastLogTime.set(errorKey, now);
    return true; // Error was logged
  };

  /**
   * Check if error is a network-related error
   */
  const isNetworkError = (error) => {
    const message = error.message?.toLowerCase() || '';
    const type = error.name?.toLowerCase() || '';

    const networkIndicators = [
      'network',
      'connection',
      'timeout',
      'refused',
      'unreachable',
      'offline',
      'fetch'
    ];

    return networkIndicators.some(indicator =>
      message.includes(indicator) || type.includes(indicator)
    );
  };

  /**
   * Get suppression statistics
   */
  const getSuppressionStats = () => {
    const stats = {
      totalErrorTypes: errorTracker.errorCounts.size,
      activeSuppressions: 0,
      totalSuppressedErrors: 0,
      errorBreakdown: {}
    };

    const now = Date.now();

    for (const [errorKey, data] of errorTracker.errorCounts.entries()) {
      if (data.suppressed && now < data.suppressedUntil) {
        stats.activeSuppressions++;
      }

      stats.totalSuppressedErrors += Math.max(0, data.count - 1);
      stats.errorBreakdown[errorKey] = {
        count: data.count,
        suppressed: data.suppressed,
        firstSeen: new Date(data.firstSeen),
        lastSeen: new Date(data.lastSeen)
      };
    }

    return stats;
  };

  /**
   * Reset suppression state
   */
  const resetSuppression = (errorKey = null) => {
    if (errorKey) {
      errorTracker.errorCounts.delete(errorKey);
      errorTracker.lastLogTime.delete(errorKey);
      if (import.meta.env.DEV) {
        console.log(`🔄 [SSE_ERROR_SUPPRESSION] Reset suppression for: ${errorKey}`);
      }
    } else {
      errorTracker.errorCounts.clear();
      errorTracker.lastLogTime.clear();
      if (import.meta.env.DEV) {
        console.log('🔄 [SSE_ERROR_SUPPRESSION] Reset all error suppression');
      }
  };

  /**
   * Temporarily disable suppression
   */
  const disableSuppression = (durationMs = 30000) => {
    const originalMaxErrors = settings.maxErrorsPerMinute;
    settings.maxErrorsPerMinute = Infinity;

    setTimeout(() => {
      settings.maxErrorsPerMinute = originalMaxErrors;
      if (import.meta.env.DEV) {
        console.log('🔄 [SSE_ERROR_SUPPRESSION] Re-enabled error suppression');
      }
    }, durationMs);

    if (import.meta.env.DEV) {
      console.log(`🔄 [SSE_ERROR_SUPPRESSION] Temporarily disabled suppression for ${durationMs / 1000}s`);
    }
  };

  return {
    handleSSEError,
    shouldSuppressError,
    getSuppressionStats,
    resetSuppression,
    disableSuppression,
    isNetworkError,
    generateErrorKey
  };
}

/**
 * Enhanced error handler for SSE service integration
 */
export function createSSEErrorHandler(suppressionConfig = {}) {
  const { handleSSEError } = useSSEErrorSuppression(suppressionConfig);

  return {
    /**
     * Handle SSE connection errors with suppression
     */
    handleConnectionError: (error) => {
      return handleSSEError(error, 'SSE Connection', {
        notify: false, // Don't notify for connection errors
        silent: false
      });
    },

    /**
     * Handle SSE message parsing errors
     */
    handleMessageError: (error, messageType = 'unknown') => {
      return handleSSEError(error, `SSE Message (${messageType})`, {
        notify: false,
        silent: true // Parse errors are less critical
      });
    },

    /**
     * Handle SSE authentication errors
     */
    handleAuthError: (error) => {
      return handleSSEError(error, 'SSE Authentication', {
        forceLog: true, // Always log auth errors
        notify: true
      });
    },

    /**
     * Handle general SSE errors
     */
    handleGeneralError: (error, context = '') => {
      return handleSSEError(error, context, {
        notify: false
      });
  };
} 