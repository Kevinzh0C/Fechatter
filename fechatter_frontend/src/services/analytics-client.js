/**
 * Analytics Client for Fechatter
 * Handles event tracking and data collection
 */

class AnalyticsClient {
  constructor(config = {}) {
    // Disable analytics in development environment to avoid API call errors
    this.enabled = this.shouldEnable(config);
    this.endpoint = config.endpoint || this.getEndpoint();
    this.debug = config.debug || false;
    this.batchSize = config.batchSize || 10;
    this.flushInterval = config.flushInterval || 5000;

    this.clientId = this.getOrCreateClientId();
    this.sessionId = this.createSessionId();
    this.queue = [];
    this.flushTimer = null;

    if (this.enabled) {
      this.startBatchTimer();
      this.setupUnloadHandler();
    }

    if (this.debug) {
      console.log('[Analytics] Initialized:', {
        endpoint: this.endpoint,
        clientId: this.clientId,
        sessionId: this.sessionId,
        enabled: this.enabled
      });
    }
  }

  shouldEnable(config) {
    // Disable by default in development environment
    if (process.env.NODE_ENV === 'development') {
      // Only enable if explicitly requested and backend is available
      return config.enabled === true && config.forceEnable === true;
    }

    // In production, check if backend is available
    return config.enabled !== false;
  }

  getEndpoint() {
    // Use production endpoint for both development and production
    return 'http://45.77.178.85:6690/api/event/json';
  }

  getOrCreateClientId() {
    let clientId = localStorage.getItem('fechatter_analytics_client_id');
    if (!clientId) {
      clientId = `client_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
      localStorage.setItem('fechatter_analytics_client_id', clientId);
    }
    return clientId;
  }

  createSessionId() {
    return `session_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }

  /**
   * Track an analytics event
   * @param {string} eventType - The type of event (e.g., 'user_login', 'message_sent')
   * @param {object} eventData - Additional data for the event
   */
  async track(eventType, eventData = {}) {
    if (!this.enabled) return;

    const event = {
      context: this.buildContext(),
      event_type: {
        [eventType]: eventData
      }
    };

    if (this.debug) {
      console.log('[Analytics] Track event:', eventType, eventData);
    }

    // Add to queue
    this.queue.push(event);

    // Flush if batch size reached
    if (this.queue.length >= this.batchSize) {
      await this.flush();
    }
  }

  buildContext() {
    const context = {
      client_id: this.clientId,
      session_id: this.sessionId,
      client_ts: Date.now(),
      user_agent: navigator.userAgent,
      system: {
        os: this.getOS(),
        browser: this.getBrowser(),
        locale: navigator.language,
        timezone: Intl.DateTimeFormat().resolvedOptions().timeZone
      }
    };

    // Add user ID if available
    const userId = this.getUserId();
    if (userId) {
      context.user_id = userId;
    }

    // Add app version if available
    const appVersion = window.APP_VERSION || '1.0.0';
    context.app_version = appVersion;

    return context;
  }

  getUserId() {
    // Try to get user ID from various sources
    // 1. From global store (Vuex/Pinia)
    if (window.$store?.state?.auth?.user?.id) {
      return window.$store.state.auth.user.id;
    }

    // 2. From localStorage
    const authData = localStorage.getItem('fechatter_auth');
    if (authData) {
      try {
        const auth = JSON.parse(authData);
        return auth.user?.id || '';
      } catch (e) {
        // Invalid JSON
      }
    }

    return '';
  }

  getOS() {
    const platform = navigator.platform?.toLowerCase() || '';
    const userAgent = navigator.userAgent.toLowerCase();

    if (platform.includes('win') || userAgent.includes('windows')) return 'windows';
    if (platform.includes('mac') || userAgent.includes('mac')) return 'macos';
    if (platform.includes('linux') || userAgent.includes('linux')) return 'linux';
    if (userAgent.includes('android')) return 'android';
    if (userAgent.includes('ios') || userAgent.includes('iphone') || userAgent.includes('ipad')) return 'ios';

    return 'unknown';
  }

  getBrowser() {
    const ua = navigator.userAgent;

    if (ua.includes('Edg/')) return 'edge';
    if (ua.includes('Chrome/') && !ua.includes('Edg/')) return 'chrome';
    if (ua.includes('Firefox/')) return 'firefox';
    if (ua.includes('Safari/') && !ua.includes('Chrome')) return 'safari';
    if (ua.includes('Opera/') || ua.includes('OPR/')) return 'opera';

    return 'unknown';
  }

  async flush() {
    if (this.queue.length === 0) return;

    const events = [...this.queue];
    this.queue = [];

    if (this.debug) {
      console.log(`[Analytics] Flushing ${events.length} events`);
    }

    try {
      // Use single event endpoint for now (batch endpoint needs different format)
      for (const event of events) {
        await this.sendEvent(event);
      }
    } catch (error) {
      console.error('[Analytics] Flush error:', error);
      // Re-queue failed events
      this.queue.unshift(...events);

      // Store failed events in localStorage for retry
      this.storeOfflineEvents(events);
    }
  }

  async sendEvent(event) {
    const response = await fetch(this.endpoint, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'X-Analytics-Version': '1.0'
      },
      body: JSON.stringify(event)
    });

    if (!response.ok) {
      throw new Error(`Analytics request failed: ${response.status}`);
    }

    return response.json();
  }

  startBatchTimer() {
    if (this.flushTimer) {
      clearInterval(this.flushTimer);
    }

    this.flushTimer = setInterval(() => {
      this.flush();
    }, this.flushInterval);
  }

  setupUnloadHandler() {
    // Flush events when page unloads
    window.addEventListener('beforeunload', () => {
      // Use sendBeacon for reliability
      if (navigator.sendBeacon && this.queue.length > 0) {
        const events = [...this.queue];
        this.queue = [];

        for (const event of events) {
          navigator.sendBeacon(this.endpoint, JSON.stringify(event));
        }
      } else {
        // Fallback to sync flush
        this.flush();
      }
    });
  }

  storeOfflineEvents(events) {
    try {
      const stored = localStorage.getItem('fechatter_analytics_offline') || '[]';
      const offline = JSON.parse(stored);
      offline.push(...events);

      // Keep only last 100 events to prevent storage overflow
      const trimmed = offline.slice(-100);
      localStorage.setItem('fechatter_analytics_offline', JSON.stringify(trimmed));
    } catch (e) {
      console.error('[Analytics] Failed to store offline events:', e);
    }
  }

  async retryOfflineEvents() {
    try {
      const stored = localStorage.getItem('fechatter_analytics_offline');
      if (!stored) return;

      const events = JSON.parse(stored);
      if (events.length === 0) return;

      console.log(`[Analytics] Retrying ${events.length} offline events`);

      // Clear storage first
      localStorage.removeItem('fechatter_analytics_offline');

      // Try to send events
      for (const event of events) {
        try {
          await this.sendEvent(event);
        } catch (e) {
          // Failed again, will be retried later
          this.storeOfflineEvents([event]);
        }
      }
    } catch (e) {
      console.error('[Analytics] Failed to retry offline events:', e);
    }
  }

  /**
   * Common event tracking methods
   */

  trackAppStart() {
    this.track('app_start', {});
  }

  trackUserLogin(email, method = 'password') {
    this.track('user_login', { email, login_method: method });
  }

  trackUserLogout(email) {
    this.track('user_logout', { email });
  }

  trackMessageSent(chatId, message) {
    this.track('message_sent', {
      chat_id: chatId,
      type: message.type || 'text',
      size: message.content?.length || 0,
      has_mentions: message.content?.includes('@') || false,
      has_links: /https?:\/\//.test(message.content || '')
    });
  }

  trackError(error, context = '') {
    this.track('error_occurred', {
      error_type: error.name || 'UnknownError',
      error_code: error.code || '',
      error_message: error.message || '',
      stack_trace: error.stack || '',
      context: context
    });
  }

  trackNavigation(from, to) {
    this.track('navigation', {
      from: from,
      to: to,
      duration_ms: 0 // Will be calculated server-side
    });
  }

  trackSearch(query, results) {
    this.track('search_performed', {
      search_type: 'messages',
      query_length: query.length.toString(),
      results_count: results,
      search_duration_ms: 0,
      has_filters: false
    });
  }

  /**
   * Cleanup
   */
  destroy() {
    if (this.flushTimer) {
      clearInterval(this.flushTimer);
    }
    this.flush(); // Final flush
  }
}

// Create singleton instance - disable in development to avoid API errors
const analytics = new AnalyticsClient({
  debug: process.env.NODE_ENV === 'development',
  enabled: process.env.NODE_ENV !== 'development' // Disable in development
});

// Only retry offline events and track app start if enabled
if (analytics.enabled) {
  // Retry offline events on startup
  setTimeout(() => {
    analytics.retryOfflineEvents();
  }, 5000);

  // Track app start
  analytics.trackAppStart();
} else {
  console.log('[Analytics] Disabled in development environment');
}

export default analytics; 