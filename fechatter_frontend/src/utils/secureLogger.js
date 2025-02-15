/**
 * Secure Logger
 * Prevents accidental JWT token exposure in console logs
 */

export class SecureLogger {
  constructor() {
    this.sensitivePatterns = [
      /eyJ[A-Za-z0-9-_]*\.[A-Za-z0-9-_]*\.[A-Za-z0-9-_]*/g, // JWT pattern
      /Bearer\s+[A-Za-z0-9-_\.]+/gi, // Bearer token pattern
      /access_token[=:]\s*[A-Za-z0-9-_\.]+/gi, // access_token parameter
      /token[=:]\s*[A-Za-z0-9-_\.]+/gi, // generic token pattern
    ];
    this.replacement = '[TOKEN_REDACTED]';
  }

  /**
   * Sanitize a message to remove sensitive information
   */
  sanitize(message) {
    if (typeof message !== 'string') {
      // For objects, convert to string then sanitize
      try {
        message = JSON.stringify(message);
      } catch (e) {
        message = String(message);
      }
    }

    let sanitized = message;
    this.sensitivePatterns.forEach(pattern => {
      sanitized = sanitized.replace(pattern, this.replacement);
    });

    return sanitized;
  }

  /**
   * Sanitize an object by recursively checking all string values
   */
  sanitizeObject(obj) {
    if (!obj || typeof obj !== 'object') {
      return obj;
    }

    if (Array.isArray(obj)) {
      return obj.map(item => this.sanitizeObject(item));
    }

    const sanitized = {};
    for (const [key, value] of Object.entries(obj)) {
      if (typeof value === 'string') {
        sanitized[key] = this.sanitize(value);
      } else if (typeof value === 'object') {
        sanitized[key] = this.sanitizeObject(value);
      } else {
        sanitized[key] = value;
      }
    }

    return sanitized;
  }

  /**
   * Safe console.log
   */
  log(...args) {
    const sanitizedArgs = args.map(arg =>
      typeof arg === 'string' ? this.sanitize(arg) : this.sanitizeObject(arg)
    );
    console.log(...sanitizedArgs);
  }

  /**
   * Safe console.warn
   */
  warn(...args) {
    const sanitizedArgs = args.map(arg =>
      typeof arg === 'string' ? this.sanitize(arg) : this.sanitizeObject(arg)
    );
    console.warn(...sanitizedArgs);
  }

  /**
   * Safe console.error
   */
  error(...args) {
    const sanitizedArgs = args.map(arg =>
      typeof arg === 'string' ? this.sanitize(arg) : this.sanitizeObject(arg)
    );
    console.error(...sanitizedArgs);
  }

  /**
   * Safe console.debug
   */
  debug(...args) {
    const sanitizedArgs = args.map(arg =>
      typeof arg === 'string' ? this.sanitize(arg) : this.sanitizeObject(arg)
    );
    console.debug(...sanitizedArgs);
  }
}

// Global instance
export const secureLogger = new SecureLogger();

// Helper functions for common use
export const secureLog = (...args) => secureLogger.log(...args);
export const secureWarn = (...args) => secureLogger.warn(...args);
export const secureError = (...args) => secureLogger.error(...args);
export const secureDebug = (...args) => secureLogger.debug(...args);

// Export for use in other modules
export default secureLogger; 