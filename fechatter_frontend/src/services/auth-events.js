/**
 * Auth Event Bus
 * 
 * A simple, centralized event emitter for authentication-related events.
 * This helps to decouple modules and avoid circular dependencies.
 * For example, tokenManager can emit an auth error event without needing
 * to know about the authStore or any other part of the application.
 */
class AuthEventBus {
  constructor() {
    this.listeners = new Map();
  }

  on(event, callback) {
    if (!this.listeners.has(event)) {
      this.listeners.set(event, []);
    }
    this.listeners.get(event).push(callback);
  }

  off(event, callback) {
    if (this.listeners.has(event)) {
      const eventListeners = this.listeners.get(event);
      const index = eventListeners.indexOf(callback);
      if (index > -1) {
        eventListeners.splice(index, 1);
      }
    }
  }

  emit(event, payload) {
    if (this.listeners.has(event)) {
      this.listeners.get(event).forEach(callback => {
        try {
          callback(payload);
        } catch (error) {
          if (import.meta.env.DEV) {
            console.error(`Error in event listener for ${event}:`, error);
          }
        }
      });
    }
  }
}

export const authEventBus = new AuthEventBus();

// Define event types to avoid magic strings
export const AuthEvents = {
  AUTH_ERROR: 'auth-error',
  LOGOUT: 'logout',
  TOKEN_REFRESHED: 'token-refreshed',
}; 