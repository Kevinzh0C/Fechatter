/**
 * SSE Service - Production Stub
 * Server-Sent Events connection management
 */

import { useAuthStore } from '@/stores/auth';
import { useChatStore } from '@/stores/chat';
import { errorHandler } from '@/utils/errorHandler';

class SSEService {
  constructor() {
    this.eventSource = null;
    this.isConnected = false;
    this.connectionState = 'disconnected';
    this.reconnectAttempts = 0;
    this.maxReconnectAttempts = 5;
    this.reconnectDelay = 3000;
    this.listeners = new Map();
  }

  /**
   * Connect to SSE endpoint
   */
  async connect() {
    try {
      const authStore = useAuthStore();
      if (!authStore.isAuthenticated) {
        if (import.meta.env.DEV) {
          console.warn('🔌 SSE: Cannot connect - not authenticated');
        }
        return false;
      }

      if (this.isConnected) {
        if (import.meta.env.DEV) {
          console.log('🔌 SSE: Already connected');
        }
        return true;
      }

      const token = authStore.token;
      if (!token) {
        if (import.meta.env.DEV) {
          console.warn('🔌 SSE: No access token available');
        }
        return false;
      }

      const sseUrl = `${import.meta.env.VITE_API_BASE_URL}/realtime/sse?token=${encodeURIComponent(token)}`;

      this.eventSource = new EventSource(sseUrl);
      this.setupEventListeners();

      if (import.meta.env.DEV) {
        console.log('🔌 SSE: Connecting...');
      }

      return true;
    } catch (error) {
      if (import.meta.env.DEV) {
        console.error('🔌 SSE: Connection failed:', error);
      }
      errorHandler.handle(error, {
        context: 'SSE connection',
        silent: false
      });
      return false;
    }
  }

  /**
   * Setup event listeners
   */
  setupEventListeners() {
    if (!this.eventSource) return;

    this.eventSource.onopen = (event) => {
      this.isConnected = true;
      this.connectionState = 'connected';
      this.reconnectAttempts = 0;

      if (import.meta.env.DEV) {
        console.log('SSE: Connected');
      }

      this.emit('connected');
    };

    this.eventSource.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data);
        this.emit('message', data);
      } catch (error) {
        if (import.meta.env.DEV) {
          console.error('🔌 SSE: Failed to parse message:', error);
        }
      }
    };

    this.eventSource.onerror = (error) => {
      this.isConnected = false;
      this.connectionState = 'disconnected';

      if (import.meta.env.DEV) {
        console.error('🔌 SSE: Connection error:', error);
      }

      this.emit('error', error);
      this.scheduleReconnect();
    };

    // Custom event listeners
    this.eventSource.addEventListener('NewMessage', (event) => {
      try {
        const data = JSON.parse(event.data);
        this.handleNewMessage(data);
      } catch (error) {
        if (import.meta.env.DEV) {
          console.error('🔌 SSE: Failed to parse NewMessage:', error);
        }
      }
    });

    this.eventSource.addEventListener('TypingStatus', (event) => {
      try {
        const data = JSON.parse(event.data);
        this.emit('typing_status', data);
      } catch (error) {
        if (import.meta.env.DEV) {
          console.error('🔌 SSE: Failed to parse TypingStatus:', error);
        }
      }
    });
  }

  /**
   * Handle new message
   */
  handleNewMessage(message) {
    const chatStore = useChatStore();

    const formattedMessage = {
      id: parseInt(message.id),
      chat_id: message.chat_id,
      sender_id: message.sender_id,
      content: message.content,
      created_at: message.created_at,
      sender_name: message.sender_name,
      realtime: true
    };

    chatStore.addRealtimeMessage(formattedMessage);
    this.emit('new_message', formattedMessage);
  }

  /**
   * Schedule reconnection
   */
  scheduleReconnect() {
    if (this.reconnectAttempts >= this.maxReconnectAttempts) {
      if (import.meta.env.DEV) {
        console.error('🔌 SSE: Max reconnection attempts reached');
      }
      this.emit('permanently_failed');
      return;
    }

    const delay = this.reconnectDelay * Math.pow(2, this.reconnectAttempts);
    this.reconnectAttempts++;

    setTimeout(() => {
      if (import.meta.env.DEV) {
        console.log(`🔌 SSE: Reconnecting (attempt ${this.reconnectAttempts})...`);
      }
      this.connect();
    }, delay);
  }

  /**
   * Disconnect from SSE
   */
  disconnect() {
    if (this.eventSource) {
      this.eventSource.close();
      this.eventSource = null;
    }

    this.isConnected = false;
    this.connectionState = 'disconnected';

    if (import.meta.env.DEV) {
      console.log('🔌 SSE: Disconnected');
    }

    this.emit('disconnected');
  }

  /**
   * Event emitter methods
   */
  on(event, callback) {
    if (!this.listeners.has(event)) {
      this.listeners.set(event, []);
    }
    this.listeners.get(event).push(callback);
  }

  off(event, callback) {
    if (this.listeners.has(event)) {
      const callbacks = this.listeners.get(event);
      const index = callbacks.indexOf(callback);
      if (index > -1) {
        callbacks.splice(index, 1);
      }
    }
  }

  emit(event, data) {
    if (this.listeners.has(event)) {
      this.listeners.get(event).forEach(callback => {
        try {
          callback(data);
        } catch (error) {
          if (import.meta.env.DEV) {
            console.error(`🔌 SSE: Error in ${event} listener:`, error);
          }
        }
      });
    }
  }

  /**
   * Get connection state
   */
  getConnectionState() {
    return {
      isConnected: this.isConnected,
      state: this.connectionState,
      reconnectAttempts: this.reconnectAttempts
    };
  }
}

// Export singleton instance
export default new SSEService(); 