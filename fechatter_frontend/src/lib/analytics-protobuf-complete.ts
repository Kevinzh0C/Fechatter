/**
 * Complete Protobuf-based Analytics Client for Fechatter
 * Uses protobuf.js for proper binary encoding
 */

import { isDevelopment } from '../utils/config.js';

// Import generated protobuf types
import {
  IAnalyticsEvent,
  IEventContext,
  ISystemInfo,
  IGeoLocation,
  IUserLoginEvent,
  IMessageSentEvent,
  INavigationEvent,
  IErrorOccurredEvent,
  IFileUploadedEvent,
  ISearchPerformedEvent,
  IBatchRecordEventsRequest,
  encodeAnalyticsEvent,
  encodeBatchRequest,
  AnalyticsEvent,
  BatchRecordEventsRequest,
} from './generated/analytics_pb.js';

export interface AnalyticsConfig {
  enabled: boolean;
  endpoint: string;
  batch_size: number;
  flush_interval: number;
  debug: boolean;
  fallback_to_json: boolean; // JSON fallback when protobuf fails
}

/**
 * Complete Protobuf Analytics Client
 */
export class CompleteProtobufAnalyticsClient {
  private config: AnalyticsConfig;
  private batch_buffer: IAnalyticsEvent[] = [];
  private flush_timer: number | null = null;
  private client_id: string;
  private session_id: string = '';
  private user_id: string = '';
  private protobuf_available: boolean = false;

  constructor(config: Partial<AnalyticsConfig> = {}) {
    this.config = {
      enabled: !isDevelopment(),
      endpoint: isDevelopment() ? 'http://127.0.0.1:6690' : '/api/analytics',
      batch_size: 50,
      flush_interval: 30000, // 30 seconds
      debug: isDevelopment(),
      fallback_to_json: true, // Enable JSON fallback by default
      ...config,
    };

    this.client_id = this.generateClientId();

    // Check protobuf availability
    this.checkProtobufAvailability();

    if (this.config.enabled) {
      this.startFlushTimer();
    }
  }

  /**
   * Check if protobuf library is available
   */
  private async checkProtobufAvailability(): Promise<void> {
    try {
      // Try to create a simple protobuf message
      const testEvent: IAnalyticsEvent = {
        context: {
          clientId: 'test',
          appVersion: '1.0.0',
          clientTs: Date.now(),
        },
        eventType: {
          appStart: {},
        },
      };

      const encoded = encodeAnalyticsEvent(testEvent);
      if (encoded && encoded.length > 0) {
        this.protobuf_available = true;
        if (this.config.debug) {
          console.log('✅ Protobuf encoding available');
        }
      }
    } catch (error) {
      console.warn('⚠️ Protobuf encoding not available, will use JSON fallback:', error);
      this.protobuf_available = false;
    }
  }

  /**
   * Set user ID
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
   * Track user login event
   */
  async trackUserLogin(email: string, method: string = 'password'): Promise<void> {
    if (!this.config.enabled) return;

    const event: IAnalyticsEvent = {
      context: this.createEventContext(),
      eventType: {
        userLogin: {
          email,
          loginMethod: method,
        },
      },
    };

    await this.sendEventImmediately(event);
  }

  /**
   * Track app start event
   */
  async trackAppStart(): Promise<void> {
    if (!this.config.enabled) return;

    const event: IAnalyticsEvent = {
      context: this.createEventContext(),
      eventType: {
        appStart: {},
      },
    };

    await this.sendEventImmediately(event);
  }

  /**
   * Track message sent event
   */
  async trackMessageSent(
    chatId: string,
    content: string,
    files: string[] = []
  ): Promise<void> {
    if (!this.config.enabled) return;

    const event: IAnalyticsEvent = {
      context: this.createEventContext(),
      eventType: {
        messageSent: {
          chatId,
          messageType: files.length > 0 ? 'file' : 'text',
          messageLength: content.length,
          hasAttachments: files.length > 0,
          hasMentions: content.includes('@'),
          hasReactions: false, // Initial state
        },
      },
    };

    await this.trackEvent(event);
  }

  /**
   * Track navigation event
   */
  async trackNavigation(from: string, to: string, startTime: number): Promise<void> {
    if (!this.config.enabled) return;

    const duration = Date.now() - startTime;
    const event: IAnalyticsEvent = {
      context: this.createEventContext(),
      eventType: {
        navigation: {
          from,
          to,
          durationMs: duration,
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

    const event: IAnalyticsEvent = {
      context: this.createEventContext(),
      eventType: {
        errorOccurred: {
          errorType,
          errorCode: error.name,
          errorMessage: error.message,
          stackTrace: error.stack || '',
          context,
        },
      },
    };

    await this.sendEventImmediately(event);
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

    const event: IAnalyticsEvent = {
      context: this.createEventContext(),
      eventType: {
        searchPerformed: {
          searchType,
          queryLength: query.length,
          resultsCount,
          searchDurationMs: duration,
          hasFilters,
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

    const event: IAnalyticsEvent = {
      context: this.createEventContext(),
      eventType: {
        fileUploaded: {
          fileType: file.type,
          fileSize: file.size,
          uploadMethod: method,
          uploadDurationMs: uploadDuration,
        },
      },
    };

    await this.trackEvent(event);
  }

  /**
   * Track app exit event
   */
  async trackAppExit(exitCode: number = 0): Promise<void> {
    if (!this.config.enabled) return;

    const event: IAnalyticsEvent = {
      context: this.createEventContext(),
      eventType: {
        appExit: { exitCode },
      },
    };

    await this.sendEventImmediately(event);
  }

  /**
   * Smart encoding - prefer protobuf, fallback to JSON
   */
  private async encodeEvent(event: IAnalyticsEvent): Promise<{
    data: Uint8Array;
    contentType: string;
  }> {
    if (this.protobuf_available) {
      try {
        const data = encodeAnalyticsEvent(event);
        return {
          data,
          contentType: 'application/protobuf',
        };
      } catch (error) {
        if (this.config.debug) {
          console.warn('Protobuf encoding failed, falling back to JSON:', error);
        }
      }
    }

    // Fallback to JSON
    if (this.config.fallback_to_json) {
      const json = JSON.stringify(this.convertToJsonFormat(event));
      const data = new TextEncoder().encode(json);
      return {
        data,
        contentType: 'application/json',
      };
    }

    throw new Error('Both protobuf and JSON encoding failed');
  }

  /**
   * Convert protobuf format to JSON format (for fallback)
   */
  private convertToJsonFormat(event: IAnalyticsEvent): any {
    return {
      context: {
        client_id: event.context?.clientId,
        session_id: event.context?.sessionId,
        user_id: event.context?.userId,
        app_version: event.context?.appVersion,
        client_ts: event.context?.clientTs,
        server_ts: event.context?.serverTs,
        user_agent: event.context?.userAgent,
        ip: event.context?.ip,
        system: event.context?.system ? {
          os: event.context.system.os,
          arch: event.context.system.arch,
          locale: event.context.system.locale,
          timezone: event.context.system.timezone,
          browser: event.context.system.browser,
          browser_version: event.context.system.browserVersion,
        } : undefined,
      },
      event_type: this.convertEventTypeToJson(event.eventType),
    };
  }

  /**
   * Convert event type to JSON format
   */
  private convertEventTypeToJson(eventType: any): any {
    if (!eventType) return {};

    const result: any = {};

    if (eventType.appStart) result.app_start = {};
    if (eventType.appExit) result.app_exit = { exit_code: eventType.appExit.exitCode };
    if (eventType.userLogin) {
      result.user_login = {
        email: eventType.userLogin.email,
        login_method: eventType.userLogin.loginMethod,
      };
    }
    if (eventType.messageSent) {
      result.message_sent = {
        chat_id: eventType.messageSent.chatId,
        type: eventType.messageSent.messageType,
        size: eventType.messageSent.messageLength,
        total_files: eventType.messageSent.hasAttachments ? 1 : 0,
        has_mentions: eventType.messageSent.hasMentions,
        has_links: false, // Can add detection logic if needed
      };
    }
    if (eventType.navigation) {
      result.navigation = {
        from: eventType.navigation.from,
        to: eventType.navigation.to,
        duration_ms: eventType.navigation.durationMs,
      };
    }
    if (eventType.errorOccurred) {
      result.error_occurred = {
        error_type: eventType.errorOccurred.errorType,
        error_code: eventType.errorOccurred.errorCode,
        error_message: eventType.errorOccurred.errorMessage,
        stack_trace: eventType.errorOccurred.stackTrace,
        context: eventType.errorOccurred.context,
      };
    }
    if (eventType.searchPerformed) {
      result.search_performed = {
        search_type: eventType.searchPerformed.searchType,
        query_length: eventType.searchPerformed.queryLength?.toString(),
        results_count: eventType.searchPerformed.resultsCount,
        search_duration_ms: eventType.searchPerformed.searchDurationMs,
        has_filters: eventType.searchPerformed.hasFilters,
      };
    }
    if (eventType.fileUploaded) {
      result.file_uploaded = {
        file_type: eventType.fileUploaded.fileType,
        file_size: eventType.fileUploaded.fileSize,
        upload_method: eventType.fileUploaded.uploadMethod,
        upload_duration_ms: eventType.fileUploaded.uploadDurationMs,
      };
    }

    return result;
  }

  /**
   * Send event immediately
   */
  private async sendEventImmediately(event: IAnalyticsEvent): Promise<void> {
    try {
      const { data, contentType } = await this.encodeEvent(event);

      const response = await fetch(`${this.config.endpoint}/api/event`, {
        method: 'POST',
        headers: {
          'Content-Type': contentType,
        },
        body: data,
      });

      if (!response.ok) {
        throw new Error(`Analytics request failed: ${response.status}`);
      }

      const result = await response.json();

      if (result.session_id) {
        this.session_id = result.session_id;
      }

      if (this.config.debug) {
        console.log('✅ Analytics event sent:', {
          eventType: Object.keys(event.eventType || {})[0],
          contentType,
          response: result,
        });
      }
    } catch (error) {
      console.warn('❌ Failed to send analytics event:', error);
    }
  }

  /**
   * Batch send events
   */
  public async flush(): Promise<void> {
    if (!this.config.enabled || this.batch_buffer.length === 0) return;

    const events = [...this.batch_buffer];
    this.batch_buffer = [];

    try {
      let data: Uint8Array;
      let contentType: string;

      if (this.protobuf_available) {
        try {
          const batchRequest: IBatchRecordEventsRequest = { events };
          data = encodeBatchRequest(batchRequest);
          contentType = 'application/protobuf';
        } catch (error) {
          if (this.config.debug) {
            console.warn('Batch protobuf encoding failed, falling back to JSON:', error);
          }
          throw error; // Trigger JSON fallback
        }
      } else {
        throw new Error('Protobuf not available'); // Trigger JSON fallback
      }

      // JSON fallback handling
      if (!data!) {
        const jsonBatch = {
          events: events.map(event => this.convertToJsonFormat(event)),
        };
        const json = JSON.stringify(jsonBatch);
        data = new TextEncoder().encode(json);
        contentType = 'application/json';
      }

      const response = await fetch(`${this.config.endpoint}/api/batch`, {
        method: 'POST',
        headers: {
          'Content-Type': contentType,
        },
        body: data,
      });

      if (!response.ok) {
        throw new Error(`Analytics batch request failed: ${response.status}`);
      }

      const result = await response.json();

      if (this.config.debug) {
        console.log(`✅ Analytics batch sent: ${events.length} events`, {
          contentType,
          response: result,
        });
      }
    } catch (error) {
      console.warn('❌ Failed to send analytics batch:', error);
      // Re-add events to buffer for retry
      this.batch_buffer.unshift(...events);
    }
  }

  /**
   * Add event to batch buffer
   */
  private async trackEvent(event: IAnalyticsEvent): Promise<void> {
    this.batch_buffer.push(event);

    if (this.batch_buffer.length >= this.config.batch_size) {
      await this.flush();
    }
  }

  /**
   * Create event context
   */
  private createEventContext(): IEventContext {
    return {
      clientId: this.client_id,
      sessionId: this.session_id,
      userId: this.user_id,
      appVersion: '1.0.0', // TODO: Get from package.json
      clientTs: Date.now(),
      serverTs: 0, // Set by server
      userAgent: navigator.userAgent,
      ip: '', // Extracted by server
      system: this.getSystemInfo(),
    };
  }

  /**
   * Get system information
   */
  private getSystemInfo(): ISystemInfo {
    return {
      os: this.detectOS(),
      arch: this.detectArch(),
      locale: navigator.language,
      timezone: Intl.DateTimeFormat().resolvedOptions().timeZone,
      browser: this.detectBrowser(),
      browserVersion: this.detectBrowserVersion(),
    };
  }

  /**
   * Generate client ID
   */
  private generateClientId(): string {
    let clientId = localStorage.getItem('fechatter_client_id');
    if (!clientId) {
      clientId = `client_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
      localStorage.setItem('fechatter_client_id', clientId);
    }
    return clientId;
  }

  /**
   * Start flush timer
   */
  private startFlushTimer(): void {
    if (this.flush_timer) {
      clearInterval(this.flush_timer);
    }

    this.flush_timer = window.setInterval(() => {
      this.flush();
    }, this.config.flush_interval);
  }

  // System detection methods
  private detectOS(): string {
    const platform = navigator.platform.toLowerCase();
    if (platform.includes('win')) return 'Windows';
    if (platform.includes('mac')) return 'macOS';
    if (platform.includes('linux')) return 'Linux';
    if (platform.includes('iphone')) return 'iOS';
    if (platform.includes('android')) return 'Android';
    return 'Unknown';
  }

  private detectArch(): string {
    if (navigator.platform.includes('64')) return 'x64';
    if (navigator.platform.includes('ARM')) return 'arm';
    return 'x86';
  }

  private detectBrowser(): string {
    const userAgent = navigator.userAgent;
    if (userAgent.includes('Chrome')) return 'Chrome';
    if (userAgent.includes('Firefox')) return 'Firefox';
    if (userAgent.includes('Safari')) return 'Safari';
    if (userAgent.includes('Edge')) return 'Edge';
    return 'Unknown';
  }

  private detectBrowserVersion(): string {
    const userAgent = navigator.userAgent;
    const match = userAgent.match(/(Chrome|Firefox|Safari|Edge)\/(\d+)/);
    return match ? match[2] : 'Unknown';
  }

  /**
   * Clean up resources
   */
  destroy(): void {
    if (this.flush_timer) {
      clearInterval(this.flush_timer);
      this.flush_timer = null;
    }
    this.flush(); // Send remaining events
  }

  /**
   * Get client status
   */
  getStatus(): {
    enabled: boolean;
    protobufAvailable: boolean;
    pendingEvents: number;
    clientId: string;
  } {
    return {
      enabled: this.config.enabled,
      protobufAvailable: this.protobuf_available,
      pendingEvents: this.batch_buffer.length,
      clientId: this.client_id,
    };
  }
}

// Export global instance
export const completeAnalytics = new CompleteProtobufAnalyticsClient({
  debug: isDevelopment(),
  endpoint: isDevelopment() ? 'http://127.0.0.1:6690' : '/api/analytics',
  fallback_to_json: true, // Enable JSON fallback
}); 