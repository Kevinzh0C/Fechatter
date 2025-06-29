/**
 * Production-Grade Secure Logger
 * Implements environment-aware logging with complete debug suppression in production
 */

class SecureLogger {
  constructor() {
    this.environment = import.meta.env.MODE || 'development';
    this.isDevelopment = this.environment === 'development';
    this.isProduction = this.environment === 'production';

    // Production security settings
    this.productionConfig = {
      allowDebug: false,
      allowInfo: false,
      allowWarn: true,
      allowError: true,
      sanitizeData: true,
      maxLogLength: 100
    };

    // Development settings
    this.developmentConfig = {
      allowDebug: true,
      allowInfo: true,
      allowWarn: true,
      allowError: true,
      sanitizeData: false,
      maxLogLength: 1000
    };

    this.config = this.isProduction ? this.productionConfig : this.developmentConfig;
    this.sensitivePatterns = [
      /chatId/i,
      /messageId/i,
      /userId/i,
      /token/i,
      /password/i,
      /secret/i,
      /api[_-]?key/i,
      /authorization/i,
      /bearer/i,
      /timestamp/i,
      /debug.*raw/i,
      /performance/i
    ];

    this.initialize();
  }

  initialize() {
    if (this.isProduction) {
      this.suppressProductionLogs();
    }

    if (import.meta.env.DEV) {
      console.log(`🔒 SecureLogger initialized (${this.environment})`);
    }
  }

  /**
   * Completely suppress debug logs in production
   */
  suppressProductionLogs() {
    // Store original methods
    this.originalMethods = {
      log: console.log,
      debug: console.debug,
      info: console.info,
      warn: console.warn,
      error: console.error
    };

    // Override console methods in production
    console.log = this.isProduction ? () => { } : console.log;
    console.debug = this.isProduction ? () => { } : console.debug;
    console.info = this.isProduction ? () => { } : console.info;

    // Keep warnings and errors but sanitize them
    console.warn = (...args) => {
      if (this.config.allowWarn) {
        const sanitized = this.sanitizeArgs(args);
        this.originalMethods.warn(...sanitized);
      }
    };

    console.error = (...args) => {
      if (this.config.allowError) {
        const sanitized = this.sanitizeArgs(args);
        this.originalMethods.error(...sanitized);
      }
    };
  }

  /**
   * Sanitize arguments to remove sensitive information
   */
  sanitizeArgs(args) {
    if (!this.config.sanitizeData) {
      return args;
    }

    return args.map(arg => {
      if (typeof arg === 'string') {
        return this.sanitizeString(arg);
      } else if (typeof arg === 'object' && arg !== null) {
        return this.sanitizeObject(arg);
      }
      return arg;
    });
  }

  /**
   * Sanitize string content
   */
  sanitizeString(str) {
    let sanitized = str;

    // Remove sensitive patterns
    this.sensitivePatterns.forEach(pattern => {
      sanitized = sanitized.replace(pattern, '[REDACTED]');
    });

    // Limit length
    if (sanitized.length > this.config.maxLogLength) {
      sanitized = sanitized.substring(0, this.config.maxLogLength) + '...[TRUNCATED]';
    }

    return sanitized;
  }

  /**
   * Sanitize object content
   */
  sanitizeObject(obj) {
    if (Array.isArray(obj)) {
      return obj.map(item =>
        typeof item === 'object' ? this.sanitizeObject(item) : item
      );
    }

    const sanitized = {};
    for (const [key, value] of Object.entries(obj)) {
      // Skip sensitive keys entirely
      if (this.sensitivePatterns.some(pattern => pattern.test(key))) {
        sanitized[key] = '[REDACTED]';
      } else if (typeof value === 'string') {
        sanitized[key] = this.sanitizeString(value);
      } else if (typeof value === 'object' && value !== null) {
        sanitized[key] = this.sanitizeObject(value);
      } else {
        sanitized[key] = value;
      }
    }

    return sanitized;
  }

  /**
   * Safe console.debug
   */
  debug(...args) {
    if (this.config.allowDebug) {
      const sanitizedArgs = this.sanitizeArgs(args);
      console.debug(...sanitizedArgs);
    }
  }

  /**
   * Safe console.info
   */
  info(...args) {
    if (this.config.allowInfo) {
      const sanitizedArgs = this.sanitizeArgs(args);
      console.info(...sanitizedArgs);
    }
  }

  /**
   * Safe console.warn
   */
  warn(...args) {
    if (this.config.allowWarn) {
      const sanitizedArgs = this.sanitizeArgs(args);
      if (import.meta.env.DEV) {
        console.warn(...sanitizedArgs);
      }
    }
  }

  /**
   * Safe console.error
   */
  error(...args) {
    if (this.config.allowError) {
      const sanitizedArgs = this.sanitizeArgs(args);
      if (import.meta.env.DEV) {
        console.error(...sanitizedArgs);
      }
    }
  }

  /**
   * Development-only performance logging
   */
  performance(label, data) {
    if (this.isDevelopment) {
      if (import.meta.env.DEV) {
        console.log(`[PERF] ${label}:`, data);
      }
    }
  }

  /**
   * Restore original console methods (for debugging)
   */
  restore() {
    if (this.originalMethods) {
      Object.assign(console, this.originalMethods);
    }
  }
}

// Create singleton instance
const secureLogger = new SecureLogger();

export default secureLogger; 