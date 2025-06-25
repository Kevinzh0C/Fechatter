/**
 * Unified Error Handler
 * Single source of truth for all error handling
 * Ensures only one interception point following formal logic
 */

class UnifiedErrorHandler {
  constructor() {
    this.originalConsoleError = null;
    this.initialized = false;
    this.handlers = new Map();
    this.suppressedErrors = new Map();

    // Ensure singleton
    if (window._unifiedErrorHandler) {
      return window._unifiedErrorHandler;
    }
    window._unifiedErrorHandler = this;
  }

  /**
   * Initialize the unified error handler
   * This must be called BEFORE any other error handling system
   */
  initialize() {
    if (this.initialized) {
      console.warn('[UnifiedErrorHandler] Already initialized, skipping');
      return;
    }

    // Save the REAL original console.error
    this.originalConsoleError = console.error;

    // Override console.error ONCE
    console.error = (...args) => {
      this.handleError(...args);
    };

    this.initialized = true;
    console.log('🛡️ Unified Error Handler initialized');
  }

  /**
   * Central error handling logic
   */
  handleError(...args) {
    // Build error string for analysis
    const errorString = args.map(arg => {
      if (arg instanceof Error) {
        return arg.stack || arg.message || String(arg);
      }
      return String(arg);
    }).join(' ');

    // Check all registered handlers in order
    for (const [name, handler] of this.handlers) {
      const result = handler({
        args,
        errorString,
        suppressor: this
      });

      if (result === 'suppress') {
        // Record suppression
        this.recordSuppressed(errorString, name);
        return; // Don't log
      } else if (result === 'modify') {
        // Handler modified args, continue with modified args
        args = result.args || args;
      }
      // 'pass' or undefined means continue to next handler
    }

    // If we get here, log the error
    this.originalConsoleError.apply(console, args);
  }

  /**
   * Register an error handler
   * @param {string} name - Unique name for the handler
   * @param {Function} handler - Handler function that returns 'suppress', 'pass', or {action: 'modify', args: [...]}
   * @param {number} priority - Lower number = higher priority (default: 100)
   */
  registerHandler(name, handler, priority = 100) {
    if (this.handlers.has(name)) {
      console.warn(`[UnifiedErrorHandler] Handler '${name}' already registered, replacing`);
    }

    // Store with priority
    this.handlers.set(name, handler);

    // Re-sort handlers by priority
    this.handlers = new Map([...this.handlers.entries()].sort((a, b) => {
      const priorityA = a[2] || 100;
      const priorityB = b[2] || 100;
      return priorityA - priorityB;
    }));

    console.log(`[UnifiedErrorHandler] Registered handler: ${name}`);
  }

  /**
   * Unregister a handler
   */
  unregisterHandler(name) {
    if (this.handlers.delete(name)) {
      console.log(`[UnifiedErrorHandler] Unregistered handler: ${name}`);
    }
  }

  /**
   * Record suppressed error
   */
  recordSuppressed(errorString, handlerName) {
    const key = errorString.substring(0, 100);
    if (!this.suppressedErrors.has(key)) {
      this.suppressedErrors.set(key, {
        count: 0,
        firstSeen: Date.now(),
        lastSeen: Date.now(),
        handlers: new Set()
      });
    }

    const record = this.suppressedErrors.get(key);
    record.count++;
    record.lastSeen = Date.now();
    record.handlers.add(handlerName);
  }

  /**
   * Get suppression statistics
   */
  getStats() {
    const totalSuppressed = Array.from(this.suppressedErrors.values())
      .reduce((sum, record) => sum + record.count, 0);

    return {
      initialized: this.initialized,
      totalHandlers: this.handlers.size,
      handlers: Array.from(this.handlers.keys()),
      totalSuppressed,
      uniqueErrors: this.suppressedErrors.size
    };
  }

  /**
   * Show suppressed errors
   */
  showSuppressedErrors() {
    console.group('🔇 Suppressed Errors (Unified)');
    console.log(`Total: ${this.getStats().totalSuppressed}`);
    console.log(`Unique: ${this.suppressedErrors.size}`);

    this.suppressedErrors.forEach((record, key) => {
      console.log(`\n"${key}..." (×${record.count})`);
      console.log(`  Handlers: ${Array.from(record.handlers).join(', ')}`);
      console.log(`  First: ${new Date(record.firstSeen).toLocaleTimeString()}`);
      console.log(`  Last: ${new Date(record.lastSeen).toLocaleTimeString()}`);
    });
    console.groupEnd();
  }

  /**
   * Restore original console.error
   */
  restore() {
    if (this.originalConsoleError) {
      console.error = this.originalConsoleError;
      console.log('🔄 Restored original console.error');
    }
  }
}

// Create singleton instance
const unifiedErrorHandler = new UnifiedErrorHandler();

// Auto-initialize to ensure we're first
if (typeof window !== 'undefined') {
  // Initialize immediately to be the first interceptor
  unifiedErrorHandler.initialize();

  // Expose globally
  window.unifiedErrorHandler = unifiedErrorHandler;
}

export default unifiedErrorHandler; 