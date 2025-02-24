/**
 * Message Update Debouncer
 * Prevents UI jitter by managing message updates from multiple sources
 */

export class MessageUpdateDebouncer {
  constructor() {
    this.pendingUpdates = new Map(); // chatId -> { messages, source, timestamp }
    this.updateTimers = new Map(); // chatId -> timeoutId
    this.lastUpdateTime = new Map(); // chatId -> timestamp
    this.updateCallbacks = new Map(); // chatId -> callback function
    
    // Configuration
    this.debounceDelay = 50; // 50ms debounce
    this.minTimeBetweenUpdates = 100; // Minimum 100ms between updates
  }

  /**
   * Schedule a message update with debouncing
   * @param {number} chatId - Chat ID
   * @param {Array} messages - Messages to update
   * @param {string} source - Update source ('cache' | 'api' | 'sse' | 'local')
   * @param {Function} updateCallback - Function to call with final messages
   */
  scheduleUpdate(chatId, messages, source, updateCallback) {
    const now = Date.now();
    const lastUpdate = this.lastUpdateTime.get(chatId) || 0;
    
    if (import.meta.env.DEV) {
      console.log(`🔄 [Debouncer] Update scheduled from ${source} for chat ${chatId}`);
    }
    
    // Clear existing timer
    if (this.updateTimers.has(chatId)) {
      clearTimeout(this.updateTimers.get(chatId));
    }
    
    // Store the update
    this.pendingUpdates.set(chatId, {
      messages: [...messages],
      source,
      timestamp: now
    });
    
    // Store callback
    this.updateCallbacks.set(chatId, updateCallback);
    
    // Calculate delay
    const timeSinceLastUpdate = now - lastUpdate;
    const delay = Math.max(
      this.debounceDelay,
      this.minTimeBetweenUpdates - timeSinceLastUpdate
    );
    
    // Schedule the update
    const timerId = setTimeout(() => {
      this.executeUpdate(chatId);
    }, delay);
    
    this.updateTimers.set(chatId, timerId);
  }
  
  /**
   * Execute the pending update
   * @param {number} chatId - Chat ID
   */
  executeUpdate(chatId) {
    const update = this.pendingUpdates.get(chatId);
    const callback = this.updateCallbacks.get(chatId);
    
    if (!update || !callback) {
      return;
    }
    
    if (import.meta.env.DEV) {
      console.log(`✅ [Debouncer] Executing update from ${update.source} for chat ${chatId}`);
    }
    
    // Execute the update
    callback(update.messages);
    
    // Update last update time
    this.lastUpdateTime.set(chatId, Date.now());
    
    // Cleanup
    this.pendingUpdates.delete(chatId);
    this.updateTimers.delete(chatId);
    this.updateCallbacks.delete(chatId);
  }
  
  /**
   * Cancel pending updates for a chat
   * @param {number} chatId - Chat ID
   */
  cancelUpdates(chatId) {
    if (this.updateTimers.has(chatId)) {
      clearTimeout(this.updateTimers.get(chatId));
      this.updateTimers.delete(chatId);
    this.pendingUpdates.delete(chatId);
    this.updateCallbacks.delete(chatId);
  }
  
  /**
   * Clear all pending updates
   */
  clearAll() {
    this.updateTimers.forEach(timerId => clearTimeout(timerId));
    this.updateTimers.clear();
    this.pendingUpdates.clear();
    this.updateCallbacks.clear();
    this.lastUpdateTime.clear();
  }

// Create singleton instance
export const messageUpdateDebouncer = new MessageUpdateDebouncer();

// Export for debugging
if (typeof window !== 'undefined' && import.meta.env.DEV) {
  window.messageDebouncer = messageUpdateDebouncer;
}