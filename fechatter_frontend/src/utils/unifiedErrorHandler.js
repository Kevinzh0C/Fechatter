/**
 * Unified Error Handler - Transparent Version
 * Provides error processing without masking original error sources
 * Following Occam's Razor: Simple, transparent, production-grade
 */

class UnifiedErrorHandler {
  constructor() {
    this.initialized = false;
    this.handlers = new Map();
    this.suppressedErrors = new Map();
    this.originalOnError = null;
    this.originalOnUnhandledRejection = null;

    // Ensure singleton
    if (window._unifiedErrorHandler) {
      return window._unifiedErrorHandler;
    window._unifiedErrorHandler = this;
  }

  /**
   * Initialize transparent error handling
   * Uses global error events instead of console.error interception
   */
  initialize() {
    if (this.initialized) {
      if (import.meta.env.DEV) {
        console.warn('[UnifiedErrorHandler] Already initialized, skipping');
      return;
    }

    // Save original handlers
    this.originalOnError = window.onerror;
    this.originalOnUnhandledRejection = window.onunhandledrejection;

    // Handle uncaught JavaScript errors (but don't interfere with console.error)
    window.addEventListener('error', this.handleGlobalError.bind(this));
    window.addEventListener('unhandledrejection', this.handleUnhandledRejection.bind(this));

    this.initialized = true;
    if (import.meta.env.DEV) {
      console.log('🛡️ Unified Error Handler initialized (transparent mode)');
    }

  /**
   * Handle global JavaScript errors
   * This only processes actual runtime errors, not console.error calls
   */
  handleGlobalError(event) {
    const errorInfo = {
      message: event.message,
      filename: event.filename,
      lineno: event.lineno,
      colno: event.colno,
      error: event.error,
      type: 'global-error'
    };

    // Process through handlers for suppression logic only
    this.processError(errorInfo);

    // Call original handler if it exists
    if (this.originalOnError) {
      this.originalOnError.call(window, event.message, event.filename, event.lineno, event.colno, event.error);
    }

  /**
   * Handle unhandled promise rejections
   */
  handleUnhandledRejection(event) {
    const errorInfo = {
      reason: event.reason,
      promise: event.promise,
      type: 'unhandled-rejection'
    };

    // Process through handlers
    const shouldSuppress = this.processError(errorInfo);

    if (shouldSuppress) {
      event.preventDefault(); // Prevent default browser logging
    }

    // Call original handler if it exists
    if (this.originalOnUnhandledRejection) {
      this.originalOnUnhandledRejection.call(window, event);
    }

  /**
   * Process error through registered handlers
   * Returns true if error should be suppressed
   */
  processError(errorInfo) {
    // Build error string for analysis
    const errorString = this.buildErrorString(errorInfo);

    // Check all registered handlers in order
    for (const [name, handlerData] of this.handlers) {
      try {
        // Extract the actual handler function
        const handler = handlerData.handler;

        if (typeof handler !== 'function') {
          if (import.meta.env.DEV) {
            console.error(`[UnifiedErrorHandler] Handler '${name}' is not a function:`, typeof handler);
          continue;
        }

        const result = handler({
          errorInfo,
          errorString,
          suppressor: this
        });

        if (result === 'suppress') {
          this.recordSuppressed(errorString, name);
          return true; // Suppress this error
        }
      } catch (handlerError) {
        if (import.meta.env.DEV) {
          console.error(`[UnifiedErrorHandler] Handler '${name}' failed:`, handlerError);
        }

    return false; // Don't suppress
  }

  /**
   * Build error string from error info
   */
  buildErrorString(errorInfo) {
    if (errorInfo.type === 'global-error') {
      return `${errorInfo.message} at ${errorInfo.filename}:${errorInfo.lineno}:${errorInfo.colno}`;
    } else if (errorInfo.type === 'unhandled-rejection') {
      if (errorInfo.reason instanceof Error) {
        return errorInfo.reason.stack || errorInfo.reason.message || String(errorInfo.reason);
      return String(errorInfo.reason);
    return String(errorInfo);
  }

  /**
   * Register an error handler
   * Handlers receive error info and return 'suppress' to suppress or undefined to continue
   */
  registerHandler(name, handler, priority = 100) {
    if (this.handlers.has(name)) {
      if (import.meta.env.DEV) {
        console.warn(`[UnifiedErrorHandler] Handler '${name}' already registered, replacing`);
      }

    // Validate handler function
    if (typeof handler !== 'function') {
      if (import.meta.env.DEV) {
        console.error(`[UnifiedErrorHandler] Handler '${name}' must be a function, got:`, typeof handler);
      return;
    }

    // Store handler with metadata in a clean way
    const handlerData = {
      name,
      handler,
      priority: Number(priority) || 100,
      registered: Date.now()
    };

    this.handlers.set(name, handlerData);

    // Sort handlers by priority (lower number = higher priority)
    const sortedEntries = Array.from(this.handlers.entries())
      .sort((a, b) => {
        const priorityA = a[1].priority;
        const priorityB = b[1].priority;
        return priorityA - priorityB;
      });

    // Rebuild map with sorted order
    this.handlers.clear();
    for (const [handlerName, handlerData] of sortedEntries) {
      this.handlers.set(handlerName, handlerData);
    }

    if (import.meta.env.DEV) {
      console.log(`[UnifiedErrorHandler] Registered handler: ${name} (priority: ${priority})`);
    }

  /**
   * Unregister a handler
   */
  unregisterHandler(name) {
    if (this.handlers.delete(name)) {
      if (import.meta.env.DEV) {
        console.log(`[UnifiedErrorHandler] Unregistered handler: ${name}`);
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
      uniqueErrors: this.suppressedErrors.size,
      mode: 'transparent'
    };
  }

  /**
   * Show suppressed errors
   */
  showSuppressedErrors() {
    console.group('🔇 Suppressed Errors (Transparent Mode)');
    if (import.meta.env.DEV) {
      console.log(`Total: ${this.getStats().totalSuppressed}`);
    if (import.meta.env.DEV) {
      console.log(`Unique: ${this.suppressedErrors.size}`);
    }

    this.suppressedErrors.forEach((record, key) => {
      if (import.meta.env.DEV) {
        console.log(`\n"${key}..." (×${record.count})`);
      if (import.meta.env.DEV) {
        console.log(`  Handlers: ${Array.from(record.handlers).join(', ')}`);
      if (import.meta.env.DEV) {
        console.log(`  First: ${new Date(record.firstSeen).toLocaleTimeString()}`);
      if (import.meta.env.DEV) {
        console.log(`  Last: ${new Date(record.lastSeen).toLocaleTimeString()}`);
      }
    });
    console.groupEnd();
  }

  /**
   * Restore original error handlers
   */
  restore() {
    if (this.initialized) {
      window.removeEventListener('error', this.handleGlobalError.bind(this));
      window.removeEventListener('unhandledrejection', this.handleUnhandledRejection.bind(this));

      if (this.originalOnError) {
        window.onerror = this.originalOnError;
      if (this.originalOnUnhandledRejection) {
        window.onunhandledrejection = this.originalOnUnhandledRejection;
      }

      this.initialized = false;
      if (import.meta.env.DEV) {
        console.log('🔄 Restored original error handlers (transparent mode)');
      }

  /**
   * Manual error processing for specific cases
   * This allows selective error handling without affecting console.error
   */
  processSpecificError(error, context = {}) {
    const errorInfo = {
      error,
      context,
      type: 'manual'
    };

    return this.processError(errorInfo);
  }

// Create singleton instance
const unifiedErrorHandler = new UnifiedErrorHandler();

// Auto-initialize to ensure we're ready
if (typeof window !== 'undefined') {
  // Initialize with transparent mode
  unifiedErrorHandler.initialize();

  // Expose globally
  window.unifiedErrorHandler = unifiedErrorHandler;
}

export default unifiedErrorHandler; 