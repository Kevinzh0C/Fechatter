/**
 * Minimal SSE Global Manager
 * Following Occam's Razor: Only prevent infinite loops, nothing else
 */

class MinimalSSEGlobalManager {
  constructor() {
    // Just track if we're in a failure loop
    this.failureCount = 0;
    this.lastFailureTime = 0;

    // Simple limit - if we fail 3 times in 1 minute, stop
    this.maxFailuresPerMinute = 3;
  }

  /**
   * Record a failure
   */
  recordFailure() {
    const now = Date.now();
    const oneMinuteAgo = now - 60000;

    // Reset counter if last failure was over a minute ago
    if (this.lastFailureTime < oneMinuteAgo) {
      this.failureCount = 0;
    }

    this.failureCount++;
    this.lastFailureTime = now;

    // Return true if we should stop trying
    return this.failureCount >= this.maxFailuresPerMinute;
  }

  /**
   * Reset the manager
   */
  reset() {
    this.failureCount = 0;
    this.lastFailureTime = 0;
  }

  /**
   * Get simple status
   */
  getStatus() {
    return {
      failureCount: this.failureCount,
      shouldStop: this.failureCount >= this.maxFailuresPerMinute
    };
  }
}

// Export singleton
export default new MinimalSSEGlobalManager();