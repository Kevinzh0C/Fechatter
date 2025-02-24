/**
 * Minimal SSE Service
 * Following Frontend Design Principles:
 * - Local State First
 * - YAGNI (You Aren't Gonna Need It)
 * - Function > Class where possible
 * - One Screen Rule (errors should be unobtrusive)
 */

import { SIMPLE_SSE_CONFIG, getUserFriendlyError } from '@/config/sse-simple-config';

class MinimalSSEService {
  constructor() {
    // Minimal state - only what we actually need
    this.eventSource = null;
    this.retryCount = 0;
    this.listeners = new Map();

    // No complex state management - just connected or not
    this.connected = false;
  }

  /**
   * Connect to SSE - keep it simple
   */
  connect(token) {
    // Already connected? Done.
    if (this.connected && this.eventSource) {
      return;
    }

    // Offline? Don't even try.
    if (!navigator.onLine) {
      this.useMockMode();
      return;
    }

    try {
      // 使用相对路径通过vite代理访问，避免CORS问题
      const url = `${import.meta.env.VITE_SSE_URL || '/events'}?access_token=${token}`;
      this.eventSource = new EventSource(url);

      // Success handler
      this.eventSource.onopen = () => {
        this.connected = true;
        this.retryCount = 0;
        if (import.meta.env.DEV) {
          console.log('✅ Real-time updates connected');
        }
      };

      // Message handler - delegate to listeners
      this.eventSource.onmessage = (event) => {
        this.handleMessage(event);
      };

      // Error handler - keep it simple
      this.eventSource.onerror = (error) => {
        this.handleError(error);
      };

    } catch (error) {
      // Connection failed - silent fallback
      this.useMockMode();
    }
  }

  /**
   * Handle errors with minimal disruption
   */
  handleError(error) {
    this.connected = false;

    // Close the connection
    if (this.eventSource) {
      this.eventSource.close();
      this.eventSource = null;
    }

    // Should we retry?
    if (this.retryCount < SIMPLE_SSE_CONFIG.MAX_RETRIES) {
      this.retryCount++;

      // Simple retry with fixed delay
      setTimeout(() => {
        const token = this.getToken();
        if (token) {
          this.connect(token);
        }
      }, SIMPLE_SSE_CONFIG.RETRY_DELAY);

      // Silent retry - no error messages
      if (SIMPLE_SSE_CONFIG.SILENT_AFTER_FIRST_FAILURE && this.retryCount > 1) {
        return;
      }
    }

    // Max retries reached - show simple message if configured
    const userError = getUserFriendlyError(error);
    if (userError && !SIMPLE_SSE_CONFIG.SILENT_AFTER_FIRST_FAILURE) {
      this.showSimpleNotification(userError.message);
    }
  }

  /**
   * Mock mode for offline/error scenarios
   */
  useMockMode() {
    this.connected = true;  // Pretend we're connected
    if (import.meta.env.DEV) {
      console.log('📡 Using offline mode for real-time updates');
    }
  }

  /**
   * Handle incoming messages
   */
  handleMessage(event) {
    try {
      const data = JSON.parse(event.data);

      // Notify all listeners for this event type
      const listeners = this.listeners.get(data.type) || [];
      listeners.forEach(callback => {
        try {
          callback(data);
        } catch (err) {
          // Listener error - don't crash the service
          if (import.meta.env.DEV) {
            console.warn('SSE listener error:', err);
          }
        }
      });
    } catch (err) {
      // Invalid message - ignore
    }
  }

  /**
   * Subscribe to events
   */
  on(eventType, callback) {
    if (!this.listeners.has(eventType)) {
      this.listeners.set(eventType, []);
    }
    this.listeners.get(eventType).push(callback);

    // Return unsubscribe function
    return () => {
      const listeners = this.listeners.get(eventType) || [];
      const index = listeners.indexOf(callback);
      if (index > -1) {
        listeners.splice(index, 1);
      }
    };
  }

  /**
   * Disconnect and cleanup
   */
  disconnect() {
    this.connected = false;
    this.retryCount = 0;

    if (this.eventSource) {
      this.eventSource.close();
      this.eventSource = null;
    }

    this.listeners.clear();
  }

  /**
   * Get token helper
   */
  getToken() {
    try {
      // Try to get token from token manager
      const tokenManager = window.tokenManager;
      return tokenManager?.getAccessToken();
    } catch {
      return null;
    }
  }

  /**
   * Simple notification helper
   */
  showSimpleNotification(message) {
    // Use existing notification system if available
    const errorHandler = window.errorHandler;
    if (errorHandler?.showNotification) {
      errorHandler.showNotification('info', message);
    }
  }

  /**
   * Get simple connection status
   */
  getStatus() {
    return {
      connected: this.connected,
      retries: this.retryCount
    };
  }

  /**
   * Get connection state (compatibility method)
   */
  getConnectionState() {
    return {
      isConnected: this.connected,
      state: this.connected ? 'connected' : 'disconnected',
      reconnectAttempts: this.retryCount,
      latency: 0,
      connectionType: 'SSE'
    };
  }

  /**
   * Check if connected (compatibility getter)
   */
  get isConnected() {
    return this.connected;
  }
}

// Export singleton instance
const minimalSSEService = new MinimalSSEService();

// Expose to window for compatibility with existing code
if (typeof window !== 'undefined') {
  window.realtimeCommunicationService = minimalSSEService;
}

export default minimalSSEService; 