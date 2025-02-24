/**
 * Minimal SSE Global Manager
 * Following Occam's Razor: Only prevent infinite loops, nothing else
 */

class MinimalSSEGlobalManager {
  constructor() {
    // Just track if we're in a failure loop
    this.failureCount = 0;
    this.lastFailureTime = 0;
    this.connections = new Map(); // Track active connections

    // Simple limit - if we fail 3 times in 1 minute, stop
    this.maxFailuresPerMinute = 3;
  }

  /**
   * Register a new SSE connection
   */
  registerConnection(url, eventSource, service) {
    const connectionId = `sse_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;

    this.connections.set(connectionId, {
      id: connectionId,
      url,
      eventSource,
      service,
      createdAt: Date.now(),
      status: 'connecting'
    });

    if (import.meta.env.DEV) {
      console.log(`🔗 [SSEGlobalManager] Registered connection: ${connectionId}`);
    return connectionId;
  }

  /**
   * Record a connection error
   */
  recordConnectionError(connectionId, error) {
    // Update connection status
    if (this.connections.has(connectionId)) {
      const connection = this.connections.get(connectionId);
      connection.status = 'error';
      connection.lastError = error;
      connection.errorTime = Date.now();
    }

    // Use existing failure tracking
    const shouldStop = this.recordFailure();

    if (shouldStop) {
      return {
        terminate: true,
        reason: 'Too many failures in short time'
      };
    }

    return { terminate: false };
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
    this.connections.clear();
  }

  /**
   * Get simple status
   */
  getStatus() {
    return {
      failureCount: this.failureCount,
      shouldStop: this.failureCount >= this.maxFailuresPerMinute,
      connections: Array.from(this.connections.values()),
      sessionStats: {
        permanentlyBanned: [],
        totalRetries: this.failureCount
      }
    };
  }

// Export singleton
const globalManager = new MinimalSSEGlobalManager();
export default globalManager;

// Make available for debugging
if (typeof window !== 'undefined') {
  window.sseGlobalManager = globalManager;
}