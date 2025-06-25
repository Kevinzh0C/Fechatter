/**
 * Protobuf-based analytics client for Fechatter frontend
 * Sends events to analytics_server using Protocol Buffers format
 */

import { isDevelopment } from '../utils/config.js';

// Define the proto structure manually (since we don't have protobuf.js included)
// In production, you'd want to generate these from the .proto files

export interface AnalyticsEvent {
  context?: EventContext;
  event_type?: {
    app_start?: {};
    app_exit?: { exit_code: number };
    user_login?: UserLoginEvent;
    user_logout?: { email: string };
    user_register?: UserRegisterEvent;
    chat_created?: ChatCreatedEvent;
    message_sent?: MessageSentEvent;
    chat_joined?: ChatJoinedEvent;
    chat_left?: ChatLeftEvent;
    navigation?: NavigationEvent;
    file_uploaded?: FileUploadedEvent;
    file_downloaded?: FileDownloadedEvent;
    search_performed?: SearchPerformedEvent;
    notification_received?: NotificationReceivedEvent;
    error_occurred?: ErrorOccurredEvent;
  };
}

export interface EventContext {
  client_id: string;
  session_id: string;
  user_id: string;
  app_version: string;
  client_ts: number;
  server_ts: number;
  user_agent: string;
  ip: string;
  system?: SystemInfo;
  geo?: GeoLocation;
}

export interface SystemInfo {
  os: string;
  arch: string;
  locale: string;
  timezone: string;
  browser: string;
  browser_version: string;
}

export interface GeoLocation {
  country: string;
  region: string;
  city: string;
}

export interface MessageSentEvent {
  chat_id: string;
  type: string;
  size: number;
  total_files: number;
  has_mentions: boolean;
  has_links: boolean;
}

export interface UserLoginEvent {
  email: string;
  login_method: string;
}

export interface UserRegisterEvent {
  email: string;
  workspace_id: string;
  registration_method: string;
}

export interface ChatCreatedEvent {
  workspace_id: string;
  chat_type: string;
  initial_members_count: number;
}

export interface ChatJoinedEvent {
  chat_id: string;
  join_method: string;
}

export interface ChatLeftEvent {
  chat_id: string;
  leave_reason: string;
}

export interface NavigationEvent {
  from: string;
  to: string;
  duration_ms: number;
}

export interface FileUploadedEvent {
  file_type: string;
  file_size: number;
  upload_method: string;
  upload_duration_ms: number;
}

export interface FileDownloadedEvent {
  file_type: string;
  file_size: number;
  download_duration_ms: number;
}

export interface SearchPerformedEvent {
  search_type: string;
  query_length: string;
  results_count: number;
  search_duration_ms: number;
  has_filters: boolean;
}

export interface NotificationReceivedEvent {
  notification_type: string;
  source: string;
  was_clicked: boolean;
}

export interface ErrorOccurredEvent {
  error_type: string;
  error_code: string;
  error_message: string;
  stack_trace: string;
  context: string;
}

export interface BatchRecordEventsRequest {
  events: AnalyticsEvent[];
}

export interface AnalyticsConfig {
  enabled: boolean;
  endpoint: string;
  batch_size: number;
  flush_interval: number;
  debug: boolean;
}

/**
 * Simple protobuf encoder for analytics events
 * This is a simplified version - in production, use a proper protobuf library
 */
function encodeAnalyticsEvent(event: AnalyticsEvent): Uint8Array {
  // For now, we'll use JSON encoding and let the server handle protobuf conversion
  // In production, you'd want to use protobuf.js or similar
  const json = JSON.stringify(event);
  return new TextEncoder().encode(json);
}

function encodeBatchRequest(request: BatchRecordEventsRequest): Uint8Array {
  const json = JSON.stringify(request);
  return new TextEncoder().encode(json);
}

export class ProtobufAnalyticsClient {
  private config: AnalyticsConfig;
  private batch_buffer: AnalyticsEvent[] = [];
  private flush_timer: number | null = null;
  private client_id: string;
  private session_id: string = '';
  private user_id: string = '';

  constructor(config: Partial<AnalyticsConfig> = {}) {
    this.config = {
      enabled: !isDevelopment(), // 开发环境中默认禁用，避免连接错误
      endpoint: 'http://127.0.0.1:6690', // Direct to analytics server
      batch_size: 50,
      flush_interval: 30000, // 30 seconds
      debug: isDevelopment(),
      ...config,
    };

    this.client_id = this.generateClientId();
    
    // 只有在启用时才启动定时器
    if (this.config.enabled) {
      this.startFlushTimer();
    }
  }

  /**
   * Set user ID for tracking authenticated events
   */
  setUserId(userId: string): void {
    this.user_id = userId;
  }

  /**
   * Set session ID
   */
  setSessionId(sessionId: string): void {
    this.session_id = sessionId;
  }

  /**
   * Track message sent event
   */
  async trackMessageSent(chatId: string, content: string, files: string[] = []): Promise<void> {
    if (!this.config.enabled) return;

    const event: AnalyticsEvent = {
      context: this.createEventContext(),
      event_type: {
        message_sent: {
          chat_id: chatId,
          type: files.length > 0 ? 'file' : 'text',
          size: content.length,
          total_files: files.length,
          has_mentions: content.includes('@'),
          has_links: content.includes('http://') || content.includes('https://'),
        },
      },
    };

    await this.trackEvent(event);
  }

  /**
   * Track user login event
   */
  async trackUserLogin(email: string, method: string = 'password'): Promise<void> {
    if (!this.config.enabled) return;

    const event: AnalyticsEvent = {
      context: this.createEventContext(),
      event_type: {
        user_login: {
          email,
          login_method: method,
        },
      },
    };

    await this.sendEventImmediately(event);
  }

  /**
   * Track navigation event
   */
  async trackNavigation(from: string, to: string, startTime: number): Promise<void> {
    if (!this.config.enabled) return;

    const duration = Date.now() - startTime;
    const event: AnalyticsEvent = {
      context: this.createEventContext(),
      event_type: {
        navigation: {
          from,
          to,
          duration_ms: duration,
        },
      },
    };

    await this.trackEvent(event);
  }

  /**
   * Track error event
   */
  async trackError(
    error: Error,
    context: string,
    errorType: string = 'JavaScriptError'
  ): Promise<void> {
    if (!this.config.enabled) return;

    const event: AnalyticsEvent = {
      context: this.createEventContext(),
      event_type: {
        error_occurred: {
          error_type: errorType,
          error_code: error.name,
          error_message: error.message,
          stack_trace: error.stack || '',
          context,
        },
      },
    };

    await this.sendEventImmediately(event); // Send errors immediately
  }

  /**
   * Track search event
   */
  async trackSearch(
    searchType: string,
    query: string,
    resultsCount: number,
    duration: number,
    hasFilters: boolean = false
  ): Promise<void> {
    if (!this.config.enabled) return;

    const event: AnalyticsEvent = {
      context: this.createEventContext(),
      event_type: {
        search_performed: {
          search_type: searchType,
          query_length: query.length.toString(),
          results_count: resultsCount,
          search_duration_ms: duration,
          has_filters: hasFilters,
        },
      },
    };

    await this.trackEvent(event);
  }

  /**
   * Track file upload event
   */
  async trackFileUpload(
    file: File,
    method: string,
    uploadDuration: number
  ): Promise<void> {
    if (!this.config.enabled) return;

    const event: AnalyticsEvent = {
      context: this.createEventContext(),
      event_type: {
        file_uploaded: {
          file_type: file.type,
          file_size: file.size,
          upload_method: method,
          upload_duration_ms: uploadDuration,
        },
      },
    };

    await this.trackEvent(event);
  }

  /**
   * Track app start event
   */
  async trackAppStart(): Promise<void> {
    if (!this.config.enabled) return;

    const event: AnalyticsEvent = {
      context: this.createEventContext(),
      event_type: {
        app_start: {},
      },
    };

    await this.sendEventImmediately(event);
  }

  /**
   * Track app exit event
   */
  async trackAppExit(exitCode: number = 1): Promise<void> {
    if (!this.config.enabled) return;

    const event: AnalyticsEvent = {
      context: this.createEventContext(),
      event_type: {
        app_exit: { exit_code: exitCode },
      },
    };

    await this.sendEventImmediately(event);
  }

  /**
   * Add event to batch buffer
   */
  private async trackEvent(event: AnalyticsEvent): Promise<void> {
    this.batch_buffer.push(event);

    if (this.batch_buffer.length >= this.config.batch_size) {
      await this.flush();
    }
  }

  /**
   * Send event immediately (for critical events)
   */
  private async sendEventImmediately(event: AnalyticsEvent): Promise<void> {
    try {
      const encoded = encodeAnalyticsEvent(event);
      
      const response = await fetch(`${this.config.endpoint}/api/event`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/protobuf',
        },
        body: encoded,
      });

      if (!response.ok) {
        throw new Error(`Analytics request failed: ${response.status}`);
      }

      const result = await response.json();
      
      // Update session ID if provided by server
      if (result.session_id) {
        this.session_id = result.session_id;
      }

      if (this.config.debug) {
        console.log('Analytics event sent:', event, 'Response:', result);
      }
    } catch (error) {
      console.warn('Failed to send analytics event:', error);
    }
  }

  /**
   * Flush pending events
   */
  public async flush(): Promise<void> {
    if (!this.config.enabled || this.batch_buffer.length === 0) return;

    const events = [...this.batch_buffer];
    this.batch_buffer = [];

    try {
      const batchRequest: BatchRecordEventsRequest = { events };
      const encoded = encodeBatchRequest(batchRequest);
      
      const response = await fetch(`${this.config.endpoint}/api/batch`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/protobuf',
        },
        body: encoded,
      });

      if (!response.ok) {
        throw new Error(`Analytics batch request failed: ${response.status}`);
      }

      const result = await response.json();

      if (this.config.debug) {
        console.log(`Analytics batch sent: ${events.length} events`, 'Response:', result);
      }
    } catch (error) {
      console.warn('Failed to send analytics batch:', error);
      // Re-add events to buffer for retry
      this.batch_buffer.unshift(...events);
    }
  }

  /**
   * Create event context
   */
  private createEventContext(): EventContext {
    return {
      client_id: this.client_id,
      session_id: this.session_id, // Let server manage this
      user_id: this.user_id,
      app_version: '1.0.0', // TODO: Get from package.json
      client_ts: Date.now(),
      server_ts: 0, // Will be set by server
      user_agent: navigator.userAgent,
      ip: '', // Will be extracted by server
      system: {
        os: this.detectOS(),
        arch: this.detectArch(),
        locale: navigator.language,
        timezone: Intl.DateTimeFormat().resolvedOptions().timeZone,
        browser: this.detectBrowser(),
        browser_version: this.detectBrowserVersion(),
      },
    };
  }

  /**
   * Generate unique client ID
   */
  private generateClientId(): string {
    // Try to get from localStorage first
    let clientId = localStorage.getItem('fechatter_client_id');
    if (!clientId) {
      clientId = `client_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
      localStorage.setItem('fechatter_client_id', clientId);
    }
    return clientId;
  }

  /**
   * Start automatic flush timer
   */
  private startFlushTimer(): void {
    if (this.flush_timer) {
      clearInterval(this.flush_timer);
    }

    this.flush_timer = window.setInterval(() => {
      this.flush();
    }, this.config.flush_interval);
  }

  /**
   * Detect operating system
   */
  private detectOS(): string {
    const platform = navigator.platform.toLowerCase();
    if (platform.includes('win')) return 'Windows';
    if (platform.includes('mac')) return 'macOS';
    if (platform.includes('linux')) return 'Linux';
    if (platform.includes('iphone')) return 'iOS';
    if (platform.includes('android')) return 'Android';
    return 'Unknown';
  }

  /**
   * Detect architecture
   */
  private detectArch(): string {
    // This is limited in browsers, but we can make educated guesses
    if (navigator.platform.includes('64')) return 'x64';
    if (navigator.platform.includes('ARM')) return 'arm';
    return 'x86';
  }

  /**
   * Detect browser
   */
  private detectBrowser(): string {
    const userAgent = navigator.userAgent;
    if (userAgent.includes('Chrome')) return 'Chrome';
    if (userAgent.includes('Firefox')) return 'Firefox';
    if (userAgent.includes('Safari')) return 'Safari';
    if (userAgent.includes('Edge')) return 'Edge';
    return 'Unknown';
  }

  /**
   * Detect browser version
   */
  private detectBrowserVersion(): string {
    const userAgent = navigator.userAgent;
    const match = userAgent.match(/(Chrome|Firefox|Safari|Edge)\/(\d+)/);
    return match ? match[2] : 'Unknown';
  }

  /**
   * Cleanup resources
   */
  destroy(): void {
    if (this.flush_timer) {
      clearInterval(this.flush_timer);
      this.flush_timer = null;
    }
    this.flush(); // Flush remaining events
  }
}

// Global analytics instance
export const analytics = new ProtobufAnalyticsClient({
  debug: isDevelopment(),
  endpoint: isDevelopment() ? 'http://127.0.0.1:6690' : '/api/analytics', // Use gateway in production
});