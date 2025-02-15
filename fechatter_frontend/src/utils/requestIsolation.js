/**
 * Request Isolation Layer
 * Prevents browser extension interference by isolating network requests
 * Production-grade implementation following Occam's Razor principle
 */

import { v4 as uuidv4 } from 'uuid';

class RequestIsolationManager {
  constructor() {
    // Request queue to manage concurrent requests
    this.requestQueue = new Map();

    // Active abort controllers for request lifecycle management
    this.abortControllers = new Map();

    // Extension interference patterns
    this.interferencePatterns = [
      /message channel closed/i,
      /listener .* indicated an asynchronous response/i,
      /Failed to fetch/i
    ];

    // Request retry configuration
    this.retryConfig = {
      maxRetries: 3,
      baseDelay: 500,
      maxDelay: 5000
    };
  }

  /**
   * Execute isolated request with extension protection
   */
  async executeIsolatedRequest(requestFn, options = {}) {
    const requestId = uuidv4();
    const {
      maxRetries = this.retryConfig.maxRetries,
      timeout = 30000,
      fallbackFn = null
    } = options;

    let retryCount = 0;
    let lastError = null;

    while (retryCount <= maxRetries) {
      try {
        // Create abort controller for this request
        const abortController = new AbortController();
        this.abortControllers.set(requestId, abortController);

        // Set timeout
        const timeoutId = setTimeout(() => {
          abortController.abort();
        }, timeout);

        try {
          // Execute request with abort signal
          const result = await requestFn({ signal: abortController.signal });

          // Clear timeout and cleanup
          clearTimeout(timeoutId);
          this.abortControllers.delete(requestId);

          return result;
        } catch (error) {
          clearTimeout(timeoutId);
          throw error;
        }
      } catch (error) {
        lastError = error;

        // Check if error is due to extension interference
        if (this.isExtensionInterference(error)) {
          console.warn(`[RequestIsolation] Extension interference detected (attempt ${retryCount + 1}/${maxRetries + 1})`);

          // If we have a fallback function and max retries reached
          if (fallbackFn && retryCount === maxRetries) {
            console.log('[RequestIsolation] Using fallback mechanism');
            return await fallbackFn();
          }

          // Calculate retry delay with exponential backoff
          const delay = Math.min(
            this.retryConfig.baseDelay * Math.pow(2, retryCount),
            this.retryConfig.maxDelay
          );

          await this.delay(delay);
          retryCount++;
        } else {
          // Not an extension issue, throw immediately
          throw error;
        }
      } finally {
        // Cleanup abort controller
        this.abortControllers.delete(requestId);
      }
    }

    // All retries exhausted
    throw lastError;
  }

  /**
   * Check if error is caused by extension interference
   */
  isExtensionInterference(error) {
    const errorMessage = error.message || '';

    return this.interferencePatterns.some(pattern =>
      pattern.test(errorMessage)
    );
  }

  /**
   * Queue request to prevent concurrent conflicts
   */
  async queueRequest(key, requestFn) {
    // If request with same key is already in progress, wait for it
    if (this.requestQueue.has(key)) {
      console.log(`[RequestIsolation] Request ${key} already in progress, waiting...`);
      return await this.requestQueue.get(key);
    }

    // Create promise for this request
    const requestPromise = this.executeIsolatedRequest(requestFn);
    this.requestQueue.set(key, requestPromise);

    try {
      const result = await requestPromise;
      return result;
    } finally {
      // Remove from queue when done
      this.requestQueue.delete(key);
    }
  }

  /**
   * Abort all active requests
   */
  abortAll() {
    this.abortControllers.forEach((controller, requestId) => {
      controller.abort();
    });
    this.abortControllers.clear();
    this.requestQueue.clear();
  }

  /**
   * Delay helper
   */
  delay(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
  }

  /**
   * Get active request count
   */
  getActiveRequestCount() {
    return this.requestQueue.size;
  }
}

// Export singleton instance
const requestIsolation = new RequestIsolationManager();

// Expose to window for debugging
if (typeof window !== 'undefined') {
  window.requestIsolation = requestIsolation;
}

export default requestIsolation; 