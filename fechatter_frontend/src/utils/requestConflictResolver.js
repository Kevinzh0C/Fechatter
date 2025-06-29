/**
 * Request Conflict Resolver
 * Prevents race conditions and manages concurrent API requests
 */

class RequestConflictResolver {
  constructor() {
    this.activeRequests = new Map(); // operation -> Promise
    this.requestQueues = new Map(); // resource -> Queue
    this.abortControllers = new Map(); // operation -> AbortController
    this.conflictDetectors = new Map(); // resource -> ConflictDetector
    this.maxConcurrentRequests = 5;
    this.maxQueueSize = 20;

    // Request priorities
    this.priorities = {
      'navigation': 10,
      'message-fetch': 8,
      'member-fetch': 6,
      'presence-update': 4,
      'typing-indicator': 2
    };
  }

  /**
   * Execute request with conflict resolution
   * @param {string} operationId - Unique operation identifier
   * @param {string} resourceId - Resource being accessed
   * @param {Function} requestFn - Function that returns a Promise
   * @param {Object} options - Request options
   */
  async executeRequest(operationId, resourceId, requestFn, options = {}) {
    const {
      priority = 5,
      timeout = 30000,
      retryCount = 0,
      conflictStrategy = 'abort-previous',
      deduplicate = true
    } = options;

    // Check for duplicate requests if enabled
    if (deduplicate && this.activeRequests.has(operationId)) {
      console.debug(`🔄 [CONFLICT] Reusing existing request: ${operationId}`);
      return this.activeRequests.get(operationId);
    }

    // Handle conflicts based on strategy
    await this.handleConflicts(operationId, resourceId, conflictStrategy);

    // Create abort controller
    const controller = new AbortController();
    this.abortControllers.set(operationId, controller);

    // Create request promise with timeout
    const requestPromise = this.createTimeoutRequest(
      requestFn,
      controller.signal,
      timeout
    );

    // Track active request
    this.activeRequests.set(operationId, requestPromise);

    try {
      // Execute request
      console.debug(`[CONFLICT] Starting request: ${operationId}`);
      const result = await requestPromise;

      console.debug(`[CONFLICT] Request completed: ${operationId}`);
      return result;

    } catch (error) {
      if (error.name === 'AbortError') {
        console.debug(`🚫 [CONFLICT] Request aborted: ${operationId}`);
        throw new Error(`REQUEST_ABORTED: ${operationId}`);
      }

      if (import.meta.env.DEV) {
        console.error(`ERROR: [CONFLICT] Request failed: ${operationId}`, error.message);
      }
      throw error;

    } finally {
      // Cleanup
      this.activeRequests.delete(operationId);
      this.abortControllers.delete(operationId);
    }
  }

  /**
   * Handle request conflicts based on strategy
   */
  async handleConflicts(operationId, resourceId, strategy) {
    const conflictingRequests = this.findConflictingRequests(resourceId);

    if (conflictingRequests.length === 0) return;

    switch (strategy) {
      case 'abort-previous':
        await this.abortPreviousRequests(conflictingRequests);
        break;

      case 'queue':
        await this.queueRequest(operationId, resourceId);
        break;

      case 'merge':
        // For future implementation - merge similar requests
        console.debug(`🔀 [CONFLICT] Merge strategy not implemented for: ${operationId}`);
        break;

      case 'ignore':
        // Allow concurrent execution
        break;

      default:
        if (import.meta.env.DEV) {
          console.warn(`WARNING: [CONFLICT] Unknown strategy: ${strategy}`);
        }
    }
  }

  /**
   * Find requests that conflict with the given resource
   */
  findConflictingRequests(resourceId) {
    const conflicts = [];

    for (const [operationId, promise] of this.activeRequests.entries()) {
      if (this.operationAffectsResource(operationId, resourceId)) {
        conflicts.push(operationId);
      }
    }

    return conflicts;
  }

  /**
   * Check if operation affects the given resource
   */
  operationAffectsResource(operationId, resourceId) {
    // Extract resource from operation ID
    const operationResource = this.extractResourceFromOperation(operationId);
    return operationResource === resourceId;
  }

  /**
   * Extract resource identifier from operation ID
   */
  extractResourceFromOperation(operationId) {
    // Pattern: operation-type-resource-id
    const parts = operationId.split('-');
    if (parts.length >= 3) {
      return parts.slice(1).join('-'); // Everything after operation type
    }
    return operationId;
  }

  /**
   * Abort previous conflicting requests
   */
  async abortPreviousRequests(conflictingRequests) {
    const abortPromises = conflictingRequests.map(async (operationId) => {
      const controller = this.abortControllers.get(operationId);
      if (controller) {
        console.debug(`🚫 [CONFLICT] Aborting conflicting request: ${operationId}`);
        controller.abort();

        // Wait for cleanup
        try {
          await this.activeRequests.get(operationId);
        } catch (error) {
          // Expected abort error
        }
      }
    });

    await Promise.allSettled(abortPromises);
  }

  /**
   * Queue request for later execution
   */
  async queueRequest(operationId, resourceId) {
    if (!this.requestQueues.has(resourceId)) {
      this.requestQueues.set(resourceId, []);
    }

    const queue = this.requestQueues.get(resourceId);

    // Check queue size limit
    if (queue.length >= this.maxQueueSize) {
      throw new Error(`Request queue full for resource: ${resourceId}`);
    }

    return new Promise((resolve, reject) => {
      queue.push({ operationId, resolve, reject });
      this.processQueue(resourceId);
    });
  }

  /**
   * Process queued requests for a resource
   */
  async processQueue(resourceId) {
    const queue = this.requestQueues.get(resourceId);
    if (!queue || queue.length === 0) return;

    // Check if there are active requests for this resource
    const activeForResource = this.findConflictingRequests(resourceId);
    if (activeForResource.length > 0) {
      // Wait for active requests to complete
      setTimeout(() => this.processQueue(resourceId), 100);
      return;
    }

    // Process next queued request
    const nextRequest = queue.shift();
    if (nextRequest) {
      nextRequest.resolve();
    }
  }

  /**
   * Create request with timeout wrapper
   */
  createTimeoutRequest(requestFn, signal, timeout) {
    return Promise.race([
      requestFn(signal),
      new Promise((_, reject) => {
        setTimeout(() => {
          reject(new Error(`Request timeout after ${timeout}ms`));
        }, timeout);
      })
    ]);
  }

  /**
   * Abort all active requests
   */
  abortAllRequests() {
    console.debug(`🚫 [CONFLICT] Aborting all ${this.activeRequests.size} active requests`);

    for (const [operationId, controller] of this.abortControllers.entries()) {
      controller.abort();
    }

    this.activeRequests.clear();
    this.abortControllers.clear();
    this.requestQueues.clear();
  }

  /**
   * Abort requests for specific resource
   */
  abortResourceRequests(resourceId) {
    const conflictingRequests = this.findConflictingRequests(resourceId);

    conflictingRequests.forEach(operationId => {
      const controller = this.abortControllers.get(operationId);
      if (controller) {
        console.debug(`🚫 [CONFLICT] Aborting request for resource ${resourceId}: ${operationId}`);
        controller.abort();
      }
    });
  }

  /**
   * Get conflict resolver status
   */
  getStatus() {
    return {
      activeRequests: this.activeRequests.size,
      queuedRequests: Array.from(this.requestQueues.values()).reduce((sum, queue) => sum + queue.length, 0),
      maxConcurrentRequests: this.maxConcurrentRequests,
      maxQueueSize: this.maxQueueSize,
      resources: Array.from(this.requestQueues.keys())
    };
  }

  /**
   * Clear completed requests and queues
   */
  cleanup() {
    // Remove empty queues
    for (const [resourceId, queue] of this.requestQueues.entries()) {
      if (queue.length === 0) {
        this.requestQueues.delete(resourceId);
      }
    }

    console.debug('🧹 [CONFLICT] Cleanup completed');
  }
}

// Create singleton instance
const requestConflictResolver = new RequestConflictResolver();

// Add debug methods for development
if (import.meta.env.DEV) {
  window.conflictResolver = requestConflictResolver;
  window.debugConflicts = () => {
    const status = requestConflictResolver.getStatus();
    if (import.meta.env.DEV) {
      console.log('🔄 Request Conflict Resolver Status:', status);
    }
    return status;
  };
}

export default requestConflictResolver; 