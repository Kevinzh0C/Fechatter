/**
 * Message Key Manager
 * Following Frontend Design Principles:
 * - Component State Layer: Ensure unique keys for Vue rendering
 * - One Screen Rule: Prevent visual glitches from duplicate keys
 */

class MessageKeyManager {
  constructor() {
    // Track used keys to prevent duplicates
    this.usedKeys = new Map();
    // Counter for fallback keys
    this.counter = 0;
  }

  /**
   * Generate unique key for message
   * YAGNI: Simple solution that works
   */
  getUniqueKey(message) {
    // Primary key: message ID
    let baseKey = message.id || message.temp_id || message._id;

    // If no ID, create one based on content and timestamp
    if (!baseKey) {
      const timestamp = message.created_at || message.timestamp || Date.now();
      const contentHash = this.simpleHash(message.content || '');
      baseKey = `msg_${timestamp}_${contentHash}`;
    }

    // Ensure uniqueness
    let uniqueKey = `msg-${baseKey}`;
    let attempts = 0;

    while (this.usedKeys.has(uniqueKey) && attempts < 100) {
      attempts++;
      uniqueKey = `msg-${baseKey}-${attempts}`;
    }

    // Fallback: use counter if still not unique
    if (this.usedKeys.has(uniqueKey)) {
      this.counter++;
      uniqueKey = `msg-fallback-${this.counter}`;
    }

    // Mark as used
    this.usedKeys.set(uniqueKey, true);

    // Clean up old keys periodically (keep last 1000)
    if (this.usedKeys.size > 1000) {
      const keysToRemove = Array.from(this.usedKeys.keys()).slice(0, 100);
      keysToRemove.forEach(key => this.usedKeys.delete(key));
    }

    return uniqueKey;
  }

  /**
   * Simple hash function for content
   */
  simpleHash(str) {
    let hash = 0;
    for (let i = 0; i < str.length; i++) {
      const char = str.charCodeAt(i);
      hash = ((hash << 5) - hash) + char;
      hash = hash & hash; // Convert to 32-bit integer
    }
    return Math.abs(hash).toString(36);
  }

  /**
   * Clear used keys for a specific chat
   */
  clearChatKeys(chatId) {
    // Clear keys that might belong to this chat
    const keysToRemove = [];
    this.usedKeys.forEach((value, key) => {
      if (key.includes(`chat${chatId}`)) {
        keysToRemove.push(key);
      }
    });
    keysToRemove.forEach(key => this.usedKeys.delete(key));
  }

  /**
   * Reset all keys
   */
  reset() {
    this.usedKeys.clear();
    this.counter = 0;
  }

  /**
   * Get status
   */
  getStatus() {
    return {
      usedKeysCount: this.usedKeys.size,
      counterValue: this.counter
    };
  }
}

// Export singleton instance
export const messageKeyManager = new MessageKeyManager();

/**
 * Vue composable for message keys
 */
export function useMessageKeys() {
  const getKey = (message) => {
    return messageKeyManager.getUniqueKey(message);
  };

  const clearKeys = (chatId) => {
    if (chatId) {
      messageKeyManager.clearChatKeys(chatId);
    } else {
      messageKeyManager.reset();
    }
  };

  const status = () => {
    return messageKeyManager.getStatus();
  };

  return {
    getKey,
    clearKeys,
    status
  };
} 