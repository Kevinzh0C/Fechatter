/**
 * Minimal SSE Service
 * Following Frontend Design Principles:
 * - Local State First
 * - YAGNI (You Aren't Gonna Need It)
 * - Function > Class where possible
 * - One Screen Rule (errors should be unobtrusive)
 * 
 * 🚀 CRITICAL FIX: Auto-reregister listeners on reconnection
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

    // 🚀 NEW: Store listener registration callbacks for auto-reregistration
    this.listenerRegistrators = new Set();
    this.lastToken = null;
  }

  /**
   * 🚀 NEW: Register a listener registrator for auto-reregistration
   */
  addListenerRegistrator(registrator) {
    if (typeof registrator === 'function') {
      this.listenerRegistrators.add(registrator);

      if (import.meta.env.DEV) {
        console.log(`📡 [SSE] Added listener registrator. Total: ${this.listenerRegistrators.size}`);
      }
    }
  }

  /**
   * 🚀 NEW: Remove a listener registrator
   */
  removeListenerRegistrator(registrator) {
    this.listenerRegistrators.delete(registrator);

    if (import.meta.env.DEV) {
      console.log(`📡 [SSE] Removed listener registrator. Total: ${this.listenerRegistrators.size}`);
    }
  }

  /**
   * 🚀 NEW: Auto-reregister all listeners after reconnection
   */
  _reregisterAllListeners() {
    if (this.listenerRegistrators.size === 0) {
      if (import.meta.env.DEV) {
        console.warn('⚠️ [SSE] No listener registrators available for reregistration');
      }
      return;
    }

    if (import.meta.env.DEV) {
      console.log(`🔄 [SSE] Auto-reregistering ${this.listenerRegistrators.size} listener registrators...`);
    }

    // Clear existing listeners
    this.listeners.clear();

    // Re-register all listeners
    this.listenerRegistrators.forEach(registrator => {
      try {
        registrator();
        if (import.meta.env.DEV) {
          console.log('✅ [SSE] Successfully re-registered listeners via registrator');
        }
      } catch (error) {
        if (import.meta.env.DEV) {
          console.error('❌ [SSE] Failed to re-register listeners:', error);
        }
      }
    });

    if (import.meta.env.DEV) {
      const totalListeners = Array.from(this.listeners.values()).reduce((sum, arr) => sum + arr.length, 0);
      console.log(`✅ [SSE] Reregistration complete. Total listeners: ${totalListeners}`);
    }
  }

  /**
   * Connect to SSE - keep it simple
   */
  connect(token) {
    // 🚀 CRITICAL DEBUG: Add comprehensive logging
    if (import.meta.env.DEV) {
      console.log('🔗 [SSE] connect() called with token:', token ? `${token.substring(0, 10)}...` : 'null');
    }

    // Store token for reconnection
    this.lastToken = token;

    // 🚀 CRITICAL FIX: Validate token before attempting connection
    if (!token) {
      if (import.meta.env.DEV) {
        console.error('❌ [SSE] Cannot connect: No token provided');
      }
      return;
    }

    // 🚀 CRITICAL FIX: Validate token format
    if (typeof token !== 'string' || token.length < 10) {
      if (import.meta.env.DEV) {
        console.error('❌ [SSE] Invalid token format:', typeof token, token?.length);
      }
      return;
    }

    // Already connected? Done.
    if (this.connected && this.eventSource) {
      if (import.meta.env.DEV) {
        console.log('✅ [SSE] Already connected, skipping');
      }
      return;
    }

    // 🚀 CRITICAL FIX: Always try real SSE connection, no mock mode
    // Removed offline check that was causing mock mode

    try {
      // 🚀 CRITICAL FIX: Enhanced URL construction with validation
      const baseUrl = import.meta.env.VITE_SSE_URL || '/events';
      const fullUrl = `${baseUrl}?access_token=${encodeURIComponent(token)}`;

      if (import.meta.env.DEV) {
        console.log('🚀 [SSE] Creating EventSource with URL:', fullUrl);
        console.log('🔍 [SSE] Environment check - VITE_SSE_URL:', import.meta.env.VITE_SSE_URL);
        console.log('🔍 [SSE] Base URL:', baseUrl);
        console.log('🔍 [SSE] Token length:', token.length);
        console.log('🔍 [SSE] Full URL length:', fullUrl.length);
      }

      // 🚀 CRITICAL FIX: Validate URL before EventSource creation
      try {
        new URL(fullUrl, window.location.origin);
        if (import.meta.env.DEV) {
          console.log('✅ [SSE] URL validation passed');
        }
      } catch (urlError) {
        if (import.meta.env.DEV) {
          console.error('❌ [SSE] Invalid URL construction:', urlError);
        }
        return;
      }

      if (import.meta.env.DEV) {
        console.log('🎯 [SSE] About to create EventSource...');
      }

      this.eventSource = new EventSource(fullUrl);

      if (import.meta.env.DEV) {
        console.log('✅ [SSE] EventSource object created successfully');
        console.log('🔍 [SSE] EventSource initial readyState:', this.eventSource.readyState);
        console.log('🔍 [SSE] EventSource URL property:', this.eventSource.url);
      }

      // Success handler - 🚀 CRITICAL FIX: Defensive against null EventSource
      this.eventSource.onopen = (event) => {
        this.connected = true;
        this.retryCount = 0;

        if (import.meta.env.DEV) {
          console.log('✅ [SSE] CONNECTION OPENED! Real-time updates connected');

          // 🚀 CRITICAL FIX: Use event.target instead of this.eventSource to avoid null reference
          const eventSource = event.target || this.eventSource;
          if (eventSource) {
            console.log('🌐 [SSE] EventSource URL:', eventSource.url);
            console.log('📡 [SSE] ReadyState:', eventSource.readyState);
          } else {
            console.log('⚠️ [SSE] EventSource reference is null, but connection opened');
          }
        }

        // 🚀 CRITICAL FIX: Auto-reregister listeners on successful connection
        if (this.listenerRegistrators.size > 0) {
          setTimeout(() => {
            this._reregisterAllListeners();
          }, 100); // Small delay to ensure connection is stable
        }
      };

      // Message handler - delegate to listeners
      this.eventSource.onmessage = (event) => {
        if (import.meta.env.DEV) {
          console.log('📨 [SSE] Message received:', event.data.substring(0, 100));
        }
        this.handleMessage(event);
      };

      // 🚀 CRITICAL FIX: Enhanced error handler with detailed diagnostics
      this.eventSource.onerror = (error) => {
        if (import.meta.env.DEV) {
          console.error('❌ [SSE] EventSource error event:', error);
          console.log('🔍 [SSE] Error event details:');
          console.log('  - Type:', error.type);
          console.log('  - Target:', error.target);

          // 🚀 CRITICAL FIX: Use event.target as fallback if this.eventSource is null
          const eventSource = error.target || this.eventSource;
          console.log('  - ReadyState at error:', eventSource?.readyState);
          console.log('  - URL at error:', eventSource?.url);

          // Log readyState meanings
          const readyStates = {
            0: 'CONNECTING',
            1: 'OPEN',
            2: 'CLOSED'
          };
          console.log(`  - ReadyState meaning: ${readyStates[eventSource?.readyState] || 'UNKNOWN'}`);

          // Check if this is an immediate failure (readyState still 0)
          if (eventSource?.readyState === 0) {
            console.error('🚨 [SSE] IMMEDIATE CONNECTION FAILURE - EventSource never connected');
            console.log('💡 [SSE] This suggests URL, token, or network issues');
            console.log('💡 [SSE] Check browser Network tab for failed requests');
          } else if (eventSource?.readyState === 2) {
            console.error('🚨 [SSE] CONNECTION CLOSED - EventSource was closed');
            console.log('💡 [SSE] This suggests server closed the connection');
          }
        }
        this.handleError(error);
      };

      // 🚀 CRITICAL DEBUG: Monitor EventSource state changes
      if (import.meta.env.DEV) {
        setTimeout(() => {
          console.log('🔍 [SSE] EventSource state after 1 second:');
          console.log(`  - ReadyState: ${this.eventSource?.readyState}`);
          console.log(`  - Connected flag: ${this.connected}`);

          if (this.eventSource?.readyState === 0) {
            console.warn('⚠️ [SSE] Still CONNECTING after 1 second - this may indicate issues');
          }
        }, 1000);

        setTimeout(() => {
          console.log('🔍 [SSE] EventSource state after 3 seconds:');
          console.log(`  - ReadyState: ${this.eventSource?.readyState}`);
          console.log(`  - Connected flag: ${this.connected}`);

          if (this.eventSource?.readyState === 0) {
            console.error('❌ [SSE] STILL CONNECTING after 3 seconds - connection likely failed');
            console.log('💡 [SSE] Check proxy logs for /events requests');
            console.log('💡 [SSE] If no /events requests, EventSource creation failed silently');
          }
        }, 3000);
      }

    } catch (error) {
      // 🚀 CRITICAL FIX: Enhanced error handling with detailed diagnostics
      this.connected = false;
      if (import.meta.env.DEV) {
        console.error('❌ [SSE] EventSource creation failed with exception:', error);
        console.log('🔍 [SSE] Exception details:');
        console.log('  - Name:', error.name);
        console.log('  - Message:', error.message);
        console.log('  - Stack:', error.stack);

        if (error.name === 'SecurityError') {
          console.error('🚨 [SSE] SECURITY ERROR - URL or origin issue');
          console.log('💡 [SSE] Check CORS configuration or URL format');
        } else if (error.name === 'SyntaxError') {
          console.error('🚨 [SSE] SYNTAX ERROR - Invalid URL format');
          console.log('💡 [SSE] Check URL construction logic');
        }
      }
    }
  }

  /**
   * Handle errors with minimal disruption and smart backoff
   */
  handleError(error) {
    this.connected = false;

    // 🚀 CRITICAL FIX: Enhanced error classification to prevent endless loops
    const eventSource = error.target || this.eventSource;
    const isImmediateFailure = eventSource?.readyState === 0;
    const isConnectionClosed = eventSource?.readyState === 2;

    if (import.meta.env.DEV) {
      console.log('🔍 [SSE] Error classification:');
      console.log(`  - Immediate failure: ${isImmediateFailure}`);
      console.log(`  - Connection closed: ${isConnectionClosed}`);
      console.log(`  - Current retry count: ${this.retryCount}`);
    }

    // Close the connection
    if (this.eventSource) {
      this.eventSource.close();
      this.eventSource = null;
    }

    // 🚀 CRITICAL FIX: Prevent endless retry loops for persistent failures
    if (isImmediateFailure && this.retryCount >= 1) {
      if (import.meta.env.DEV) {
        console.error('🚨 [SSE] PERSISTENT CONNECTION FAILURE - stopping retries');
        console.log('💡 [SSE] This suggests server/proxy issues that won\'t resolve with retries');
      }
      return; // Stop retrying for immediate failures after first attempt
    }

    // Should we retry?
    if (this.retryCount < SIMPLE_SSE_CONFIG.MAX_RETRIES) {
      this.retryCount++;

      // 🚀 ENHANCED: Progressive backoff for different error types
      let retryDelay = SIMPLE_SSE_CONFIG.RETRY_DELAY;

      if (isImmediateFailure) {
        // Longer delay for immediate failures (likely server/proxy issues)
        retryDelay = Math.min(10000, SIMPLE_SSE_CONFIG.RETRY_DELAY * Math.pow(2, this.retryCount));
        if (import.meta.env.DEV) {
          console.log(`⏱️ [SSE] Using extended backoff: ${retryDelay}ms for immediate failure`);
        }
      } else if (isConnectionClosed) {
        // Standard delay for normal disconnections
        retryDelay = SIMPLE_SSE_CONFIG.RETRY_DELAY;
        if (import.meta.env.DEV) {
          console.log(`⏱️ [SSE] Using standard retry delay: ${retryDelay}ms for closed connection`);
        }
      }

      if (import.meta.env.DEV) {
        console.log(`🔄 [SSE] Scheduling retry ${this.retryCount}/${SIMPLE_SSE_CONFIG.MAX_RETRIES} in ${retryDelay}ms`);
      }

      // Simple retry with calculated delay
      setTimeout(() => {
        if (this.lastToken && this.retryCount <= SIMPLE_SSE_CONFIG.MAX_RETRIES) {
          if (import.meta.env.DEV) {
            console.log(`🔄 [SSE] Executing retry ${this.retryCount}/${SIMPLE_SSE_CONFIG.MAX_RETRIES}`);
          }
          this.connect(this.lastToken);
        }
      }, retryDelay);

      // Silent retry - no error messages
      if (SIMPLE_SSE_CONFIG.SILENT_AFTER_FIRST_FAILURE && this.retryCount > 1) {
        return;
      }
    } else {
      // 🚀 ENHANCED: Better max retries handling
      if (import.meta.env.DEV) {
        console.error(`❌ [SSE] Max retries (${SIMPLE_SSE_CONFIG.MAX_RETRIES}) reached, giving up`);
        console.log('💡 [SSE] SSE will remain disconnected until manual reconnection or page refresh');
      }
    }

    // Max retries reached - show simple message if configured
    const userError = getUserFriendlyError(error);
    if (userError && !SIMPLE_SSE_CONFIG.SILENT_AFTER_FIRST_FAILURE) {
      this.showSimpleNotification(userError.message);
    }
  }

  // 🚀 CRITICAL FIX: Mock mode completely removed
  // No fake connections - only real SSE connections allowed

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
   * Get token helper - 🚀 ENHANCED with multiple fallback sources
   */
  getToken() {
    try {
      // 🚀 CRITICAL FIX: Try multiple token sources in priority order

      // Priority 1: Pinia authStore (same source used in main.js)
      if (window.__PINIA__?.state?.value?.auth?.token) {
        const authToken = window.__PINIA__.state.value.auth.token;
        if (import.meta.env.DEV) {
          console.log('🎫 [SSE] Token from Pinia authStore:', authToken ? authToken.substring(0, 20) + '...' : 'null');
        }
        return authToken;
      }

      // Priority 2: tokenManager (for consistency)
      const tokenManager = window.tokenManager;
      if (tokenManager?.getAccessToken) {
        const managerToken = tokenManager.getAccessToken();
        if (managerToken) {
          if (import.meta.env.DEV) {
            console.log('🎫 [SSE] Token from tokenManager:', managerToken.substring(0, 20) + '...');
          }
          return managerToken;
        }
      }

      // Priority 3: localStorage fallback
      const localToken = localStorage.getItem('auth_token');
      if (localToken) {
        if (import.meta.env.DEV) {
          console.log('🎫 [SSE] Token from localStorage fallback:', localToken.substring(0, 20) + '...');
        }
        return localToken;
      }

      if (import.meta.env.DEV) {
        console.warn('⚠️ [SSE] No token found in any source!');
      }
      return null;

    } catch (error) {
      if (import.meta.env.DEV) {
        console.error('❌ [SSE] Error getting token:', error);
      }
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