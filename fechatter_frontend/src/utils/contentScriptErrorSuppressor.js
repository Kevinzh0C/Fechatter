/**
 * Content Script Error Suppressor
 * Registers with unified error handler for content script error suppression
 */

class ContentScriptErrorSuppressor {
  constructor() {
    this.suppressedErrors = new Map();
    this.errorCount = 0;
    this.maxErrors = 100;
    this.initialized = false;
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
      console.debug('[ContentScript] Registered with unified error handler');
    } else {
      console.error('[ContentScript] Unified error handler not found!');
    }

    // Handle unhandled rejections (these still need direct handling)
    this.handleUnhandledRejections();

    // Handle global errors (these still need direct handling)
    this.handleGlobalErrors();

    // Provide utility functions
    this.provideUtilities();
  }

  /**
   * Handle error for unified system
   */
  handleError({ args, errorString }) {
    // Extract real source from call stack for annotation
    const stack = new Error().stack;
    const realSource = this.extractRealSource(stack);

    // Check if this is a content script error
    if (this.isContentScriptError(errorString)) {
      this.errorCount++;
      this.recordSuppressed(errorString, realSource);
      return 'suppress'; // Tell unified handler to suppress
    }

    // For non-content-script errors, add source annotation
    if (realSource && !realSource.includes('unifiedErrorHandler')) {
      // Modify args to include source
      const sourceTag = `\x1b[90m[from ${realSource}]\x1b[0m`;
      return {
        action: 'modify',
        args: [sourceTag, ...args]
      };
    }

    return 'pass'; // Let it through unchanged
  }

  /**
   * Extract the real source file and line from stack trace
   */
  extractRealSource(stack) {
    if (!stack) return null;

    const lines = stack.split('\n');

    // Skip first few lines that are from error handling system
    for (let i = 3; i < Math.min(lines.length, 8); i++) {
      const line = lines[i];

      // Skip our own files and error handling files
      if (line.includes('contentScriptErrorSuppressor.js')) continue;
      if (line.includes('unifiedErrorHandler.js')) continue;
      if (line.includes('developmentOptimizer.js')) continue;
      if (line.includes('consoleMonitor.js')) continue;
      if (line.includes('extensionConflictHandler.js')) continue;
      if (line.includes('<anonymous>')) continue;

      // Try to extract file:line info
      const match = line.match(/at\s+(?:.*?\s+\()?(.*?):(\d+):\d+\)?$/);
      if (match) {
        let filepath = match[1];
        const lineNum = match[2];

        // Clean up the filepath
        filepath = filepath.split('/').pop().split('?')[0];

        return `${filepath}:${lineNum}`;
      }
    }

    return null;
  }

  /**
   * Handle unhandled promise rejections
   */
  handleUnhandledRejections() {
    window.addEventListener('unhandledrejection', (event) => {
      const error = event.reason;
      const errorString = this.getErrorString(error);

      if (this.isContentScriptError(errorString)) {
        event.preventDefault();
        this.errorCount++;
        this.recordSuppressed(errorString, 'unhandled-rejection');
      }
    }, true);
  }

  /**
   * Handle global error events
   */
  handleGlobalErrors() {
    window.addEventListener('error', (event) => {
      const errorString = this.getErrorString(event.error || event);

      if (this.isContentScriptError(errorString)) {
        event.preventDefault();
        this.errorCount++;
        this.recordSuppressed(errorString, 'global-error');
      }
    }, true);
  }

  /**
   * Record suppressed error with metadata
   */
  recordSuppressed(errorString, source) {
    const key = errorString.substring(0, 100);

    if (!this.suppressedErrors.has(key)) {
      this.suppressedErrors.set(key, {
        fullError: errorString,
        count: 0,
        firstSeen: Date.now(),
        lastSeen: Date.now(),
        source: source || 'unknown'
      });
    }

    const record = this.suppressedErrors.get(key);
    record.count++;
    record.lastSeen = Date.now();

    // Limit map size
    if (this.suppressedErrors.size > this.maxErrors) {
      const firstKey = this.suppressedErrors.keys().next().value;
      this.suppressedErrors.delete(firstKey);
    }
  }

  /**
   * Provide utility functions
   */
  provideUtilities() {
    // Show suppressed errors
    window.showSuppressedErrors = () => {
      console.group('🔇 Suppressed Content Script Errors');
      console.log(`Total suppressed: ${this.errorCount}`);
      console.log(`Unique errors: ${this.suppressedErrors.size}`);
      console.log('');

      this.suppressedErrors.forEach((record, key) => {
        console.log(`[${record.source}] ${key}... (×${record.count})`);
        if (record.count > 1) {
          console.log(`  First seen: ${new Date(record.firstSeen).toLocaleTimeString()}`);
          console.log(`  Last seen: ${new Date(record.lastSeen).toLocaleTimeString()}`);
        }
      });
      console.groupEnd();
    };
  }

  /**
   * Get error string from various error types
   */
  getErrorString(error) {
    if (!error) return '';

    if (error.message && error.filename) {
      return `${error.message} at ${error.filename}:${error.lineno}:${error.colno}`;
    }

    if (error.stack) return error.stack;
    if (error.message) return error.message;

    return String(error);
  }

  /**
   * Check if an error is from a content script
   */
  isContentScriptError(errorString) {
    const patterns = [
      /content[\s-]?script\.js/i,
      /chrome-extension:\/\//,
      /moz-extension:\/\//,
      /extension:\/\//,
      /fetchError:\s*Failed to fetch/i,
      /at\s+[fFpP][l15]\.\s*sendMessage/i,
      /Failed to execute 'fetch' on 'Window'/i,
      /Extension context invalidated/i,
      /The message port closed before a response was received/i,
      /at\s+Zx\s+\(/i,
      /at\s+ul\.sendMessage/i,
      /at\s+async\s+ot\s+\(/i
    ];

    return patterns.some(pattern => pattern.test(errorString));
  }

  /**
   * Get suppression statistics
   */
  getStats() {
    return {
      totalSuppressed: this.errorCount,
      uniqueErrors: this.suppressedErrors.size,
      recentErrors: Array.from(this.suppressedErrors.keys()).slice(-5),
      initialized: this.initialized
    };
  }

  /**
   * Clear suppressed errors
   */
  clear() {
    this.suppressedErrors.clear();
    this.errorCount = 0;
  }
}

// Create and export singleton instance
const contentScriptSuppressor = new ContentScriptErrorSuppressor();

// Auto-initialize if in browser
if (typeof window !== 'undefined') {
  // Use setTimeout to ensure unified handler is ready
  setTimeout(() => {
    contentScriptSuppressor.initialize();
    window.contentScriptSuppressor = contentScriptSuppressor;

    console.log('💡 Content script error suppression registered');
  }, 50);
}

export default contentScriptSuppressor; 