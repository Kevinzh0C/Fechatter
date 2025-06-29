/**
 * Enhanced Content Script Error Suppressor
 * Suppresses errors from browser extensions and content scripts to keep console clean
 * Now includes async response listener errors
 */

class ContentScriptErrorSuppressor {
  constructor() {
    this.initialized = false;
    this.mode = 'transparent';
    this.totalSuppressed = 0;
    this.uniqueErrors = 0;
    this.suppressedMessages = new Set();
    this.recentErrors = [];
    this.maxRecentErrors = 50;

    // FIX: Enhanced extension error patterns
    this.extensionErrorPatterns = [
      // Original patterns
      /content script/i,
      /script injected/i,
      /extension context/i,
      /\bextension\b.*\berror\b/i,
      /chrome-extension:/,
      /moz-extension:/,
      /webkit-extension:/,
      /safari-extension:/,

      // NEW: Async response listener errors
      /listener indicated an asynchronous response.*message channel closed/i,
      /asynchronous response.*channel closed/i,
      /listener.*returning true.*channel closed/i,

      // NEW: Common extension-related errors
      /cannot access contents of url/i,
      /cannot access a chrome:/i,
      /extension context invalidated/i,
      /receiving end does not exist/i,
      /disconnected port object/i,
      /port.*disconnected/i,

      // NEW: Specific Chrome extension errors
      /unchecked runtime\.lastError/i,
      /the message port closed before a response was received/i,
      /trying to use a disconnected port object/i,
      /could not establish connection.*receiving end does not exist/i
    ];

    this.sourcePatterns = [
      // Script injection sources
      /:\/\/[^\/]*\/content_script\.js/,
      /:\/\/[^\/]*\/inject\.js/,
      /:\/\/[^\/]*\/background\.js/,
      /chrome-extension:\/\//,
      /moz-extension:\/\//,
      /webkit-extension:\/\//,
      /safari-extension:\/\//,

      // NEW: Anonymous/generated script sources
      /^<anonymous>$/,
      /^eval$/,
      /^\s*$/,
      /^VM\d+/,

      // NEW: Extension-like stack traces
      /at Object\.\w+ \(chrome-extension:/,
      /at Object\.\w+ \(moz-extension:/,
      /at \w+\.js:\d+:\d+\)$/
    ];
  }

  /**
   * Initialize by registering with unified error handler
   */
  initialize() {
    if (this.initialized) return;

    // Register our handler with the unified system
    if (window.unifiedErrorHandler) {
      window.unifiedErrorHandler.registerHandler(
        'contentScriptSuppressor',
        this.handleError.bind(this),
        10 // High priority
      );

      this.initialized = true;
      console.debug('[ContentScript] Registered with unified error handler (transparent mode)');
    } else {
      if (import.meta.env.DEV) {
        console.warn('[ContentScript] Unified error handler not found, will initialize when available');
      }

      // Try again later
      setTimeout(() => {
        if (!this.initialized) {
          this.initialize();
        }
      }, 100);
    }

    // Provide utility functions
    this.provideUtilities();
  }

  /**
   * Handle error for unified system (transparent mode)
   * Only returns 'suppress' for content script errors, undefined for all others
   */
  handleError({ errorInfo, errorString }) {
    // Check if this is a content script error
    if (this.isContentScriptError(errorString)) {
      this.totalSuppressed++;
      this.uniqueErrors++;
      this.recordSuppressed(errorString, this.getErrorSource(errorInfo));
      return 'suppress'; // Tell unified handler to suppress this error
    }

    // For all other errors, return undefined to let them pass through transparently
    return undefined;
  }

  /**
   * Get error source from error info
   */
  getErrorSource(errorInfo) {
    if (errorInfo.type === 'global-error') {
      return `${errorInfo.filename}:${errorInfo.lineno}`;
    } else if (errorInfo.type === 'unhandled-rejection') {
      return 'unhandled-rejection';
    } else if (errorInfo.type === 'manual') {
      return 'manual';
    return 'unknown';
  }

  /**
   * Record suppressed error with metadata
   */
  recordSuppressed(errorString, source) {
    const key = errorString.substring(0, 100);

    if (!this.suppressedMessages.has(key)) {
      this.suppressedMessages.add(key);
    }

    // Limit recent errors
    if (this.recentErrors.length >= this.maxRecentErrors) {
      this.recentErrors.shift();
    this.recentErrors.push(key);
  }

  /**
   * Provide utility functions
   */
  provideUtilities() {
    // Show suppressed errors
    window.showSuppressedErrors = () => {
      console.group('🔇 Suppressed Content Script Errors');
      if (import.meta.env.DEV) {
        console.log(`Total suppressed: ${this.totalSuppressed}`);
      if (import.meta.env.DEV) {
        console.log(`Unique errors: ${this.uniqueErrors}`);
      if (import.meta.env.DEV) {
        console.log('');
      }

      this.suppressedMessages.forEach((key) => {
        if (import.meta.env.DEV) {
          console.log(`[${this.getErrorSource(errorInfo)}] ${key}...`);
        }
      });
      console.groupEnd();
    };

    // Enhanced debugging function
    window.debugContentScriptErrors = () => {
      const stats = this.getStats();
      console.group('Content Script Error Debug Info');
      if (import.meta.env.DEV) {
        console.log('Stats:', stats);
      if (import.meta.env.DEV) {
        console.log('Unified handler stats:', window.unifiedErrorHandler?.getStats());
      if (import.meta.env.DEV) {
        console.log('Handler initialized:', this.initialized);
      console.groupEnd();
    };
  }

  /**
   * Check if an error is from a content script
   */
  isContentScriptError(errorString) {
    return this.extensionErrorPatterns.some(pattern => pattern.test(errorString)) ||
      this.sourcePatterns.some(pattern => pattern.test(errorString));
  }

  /**
   * Get suppression statistics
   */
  getStats() {
    return {
      totalSuppressed: this.totalSuppressed,
      uniqueErrors: this.uniqueErrors,
      recentErrors: this.recentErrors,
      initialized: this.initialized,
      mode: this.mode
    };
  }

  /**
   * Clear suppressed errors
   */
  clear() {
    this.suppressedMessages.clear();
    this.totalSuppressed = 0;
    this.uniqueErrors = 0;
    this.recentErrors = [];
  }

  /**
   * Test content script error detection
   */
  testDetection() {
    const testErrors = [
      'Error: fetchError: Failed to fetch',
      'TypeError: Cannot read property at chrome-extension://abc123/content.js:45',
      'ReferenceError: ul.sendMessage is not defined',
      'Error: Extension context invalidated',
      'Normal application error that should not be suppressed'
    ];

    console.group('🧪 Content Script Error Detection Test');
    testErrors.forEach((error, index) => {
      const isContentScript = this.isContentScriptError(error);
      if (import.meta.env.DEV) {
        console.log(`${index + 1}. ${isContentScript ? '🔇' : ''} ${error.substring(0, 50)}...`);
      }
    });
    console.groupEnd();
  }

// Create and export singleton instance
const contentScriptSuppressor = new ContentScriptErrorSuppressor();

// Auto-initialize if in browser
if (typeof window !== 'undefined') {
  // Use setTimeout to ensure unified handler is ready
  setTimeout(() => {
    contentScriptSuppressor.initialize();
    window.contentScriptSuppressor = contentScriptSuppressor;

    if (import.meta.env.DEV) {
      console.log('Content script error suppression registered (transparent mode)');
    }
  }, 50);
}

export default contentScriptSuppressor; 