/**
 * Production Safety Wrapper
 * Enforces security policies and data sanitization in production builds
 */

class ProductionSafetyWrapper {
  constructor() {
    this.isProduction = import.meta.env.PROD;
    this.isDevelopment = import.meta.env.DEV;

    // Security policies
    this.policies = {
      stripDebugLogs: this.isProduction,
      sanitizeResourceIds: true,
      limitApiParams: true,
      enforceCSP: this.isProduction,
      disableDevTools: this.isProduction
    };

    this.maxApiLimit = 100; // Maximum allowed API limit parameter
    this.idObfuscationSalt = 'fechatter_2024_security';

    this.initialize();
  }

  initialize() {
    if (this.policies.stripDebugLogs) {
      this.suppressDebugOutput();
    }

    if (this.policies.disableDevTools) {
      this.disableDevTools();
    }

    this.setupResourceIdObfuscation();
    this.setupApiParameterLimiting();

    if (import.meta.env.DEV) {
      console.log('🛡️ Production Safety Wrapper active');
    }
  }

  /**
   * Suppress all debug output in production
   */
  suppressDebugOutput() {
    const noop = () => { };

    // Override debug methods with no-ops
    console.debug = noop;
    console.trace = noop;

    // Override info to only show non-sensitive information
    const originalInfo = console.info;
    console.info = (...args) => {
      const sanitized = args.map(arg => {
        if (typeof arg === 'string') {
          // Block messages containing sensitive patterns
          if (this.containsSensitiveInfo(arg)) {
            return;
          }
        }
        return arg;
      }).filter(Boolean);

      if (sanitized.length > 0) {
        originalInfo(...sanitized);
      }
    };
  }

  /**
   * Check if string contains sensitive information
   */
  containsSensitiveInfo(str) {
    const sensitivePatterns = [
      /\[DEBUG.*RAW\]/i,
      /chatId.*\d+/i,
      /messageIds.*\[/i,
      /lastFetchedTimestamp/i,
      /Performance.*\d+ms/i,
      /navigate.*in.*\d+ms/i,
      /API.*RAW/i,
      /Members.*Response/i,
      /Presence.*updated/i
    ];

    return sensitivePatterns.some(pattern => pattern.test(str));
  }

  /**
   * Disable development tools in production
   */
  disableDevTools() {
    // Disable right-click context menu
    document.addEventListener('contextmenu', (e) => {
      e.preventDefault();
      return false;
    });

    // Disable F12, Ctrl+Shift+I, etc.
    document.addEventListener('keydown', (e) => {
      // F12
      if (e.keyCode === 123) {
        e.preventDefault();
        return false;
      }

      // Ctrl+Shift+I (Developer Tools)
      if (e.ctrlKey && e.shiftKey && e.keyCode === 73) {
        e.preventDefault();
        return false;
      }

      // Ctrl+Shift+C (Inspect Element)
      if (e.ctrlKey && e.shiftKey && e.keyCode === 67) {
        e.preventDefault();
        return false;
      }

      // Ctrl+U (View Source)
      if (e.ctrlKey && e.keyCode === 85) {
        e.preventDefault();
        return false;
      }
    });
  }

  /**
   * Setup resource ID obfuscation for client-side display
   */
  setupResourceIdObfuscation() {
    window.securityUtils = {
      obfuscateId: this.obfuscateId.bind(this),
      deobfuscateId: this.deobfuscateId.bind(this),
      sanitizeDisplayId: this.sanitizeDisplayId.bind(this)
    };
  }

  /**
   * Simple ID obfuscation for frontend display
   * NOTE: This is NOT security - real security must be server-side
   */
  obfuscateId(id) {
    if (!this.policies.sanitizeResourceIds || !id) return id;

    // Simple obfuscation for display purposes only
    const hash = this.simpleHash(id + this.idObfuscationSalt);
    return `${hash.toString(36).padStart(6, '0')}`;
  }

  /**
   * Deobfuscate ID (development helper only)
   */
  deobfuscateId(obfuscatedId) {
    // This is intentionally not implemented for security
    // Real deobfuscation should happen server-side
    if (import.meta.env.DEV) {
      console.warn('🔒 ID deobfuscation not available - use server-side validation');
    }
    return null;
  }

  /**
   * Sanitize ID for display (hide patterns that could indicate structure)
   */
  sanitizeDisplayId(id) {
    if (!id) return id;

    if (this.isProduction) {
      // In production, show only last 4 characters with prefix
      return `***${String(id).slice(-4)}`;
    }

    return id; // Show full ID in development
  }

  /**
   * Simple hash function for obfuscation
   */
  simpleHash(str) {
    let hash = 0;
    for (let i = 0; i < str.length; i++) {
      const char = str.charCodeAt(i);
      hash = ((hash << 5) - hash) + char;
      hash = hash & hash; // Convert to 32-bit integer
    }
    return Math.abs(hash);
  }

  /**
   * Setup API parameter limiting
   */
  setupApiParameterLimiting() {
    window.securityUtils.enforceApiLimits = (params) => {
      if (!this.policies.limitApiParams) return params;

      const sanitized = { ...params };

      // Enforce maximum limit
      if (sanitized.limit && sanitized.limit > this.maxApiLimit) {
        if (import.meta.env.DEV) {
          console.warn(`🚫 API limit reduced from ${sanitized.limit} to ${this.maxApiLimit}`);
        }
        sanitized.limit = this.maxApiLimit;
      }

      // Enforce maximum page size
      if (sanitized.pageSize && sanitized.pageSize > this.maxApiLimit) {
        if (import.meta.env.DEV) {
          console.warn(`🚫 Page size reduced from ${sanitized.pageSize} to ${this.maxApiLimit}`);
        }
        sanitized.pageSize = this.maxApiLimit;
      }

      return sanitized;
    };
  }

  /**
   * Get security status
   */
  getSecurityStatus() {
    return {
      environment: this.isProduction ? 'production' : 'development',
      policies: this.policies,
      maxApiLimit: this.maxApiLimit,
      debugSuppressed: this.policies.stripDebugLogs,
      devToolsDisabled: this.policies.disableDevTools
    };
  }
}

// Initialize immediately
const productionSafety = new ProductionSafetyWrapper();

export default productionSafety; 