/**
 * 📖 Reading Position Manager - Smart scroll position memory
 * 
 * Features:
 * - First visit: Scroll to bottom (latest messages)
 * - Return visits: Restore to saved position
 * - Cross-session persistence with localStorage
 * - Automatic cleanup of old positions
 */

class ReadingPositionManager {
  constructor() {
    this.positions = new Map(); // chatId -> position data
    this.visitHistory = new Map(); // chatId -> visit count
    this.storageKey = 'fechatter_reading_positions';
    this.maxAge = 7 * 24 * 60 * 60 * 1000; // 7 days
    this.maxPositions = 50; // Maximum positions to store

    this.loadFromStorage();
  }

  /**
   * Check if this is the first visit to a chat
   */
  isFirstVisit(chatId) {
    const visits = this.visitHistory.get(parseInt(chatId)) || 0;
    return visits === 0;
  }

  /**
   * Record a visit to a chat
   */
  recordVisit(chatId) {
    const normalizedChatId = parseInt(chatId);
    const currentVisits = this.visitHistory.get(normalizedChatId) || 0;
    this.visitHistory.set(normalizedChatId, currentVisits + 1);

    if (import.meta.env.DEV) {
      console.log(`📖 [ReadingPosition] Recorded visit to chat ${chatId}, total visits: ${currentVisits + 1}`);
    }
  }

  /**
   * Save current reading position
   */
  savePosition(chatId, scrollTop, messageId = null) {
    const normalizedChatId = parseInt(chatId);
    const position = {
      scrollTop,
      messageId,
      timestamp: Date.now(),
      chatId: normalizedChatId
    };

    this.positions.set(normalizedChatId, position);
    this.saveToStorage();

    if (import.meta.env.DEV) {
      console.log(`📖 [ReadingPosition] Saved position for chat ${chatId}:`, {
        scrollTop,
        messageId,
        timestamp: new Date(position.timestamp).toLocaleTimeString()
      });
    }
  }

  /**
   * Get saved reading position
   */
  getPosition(chatId) {
    const normalizedChatId = parseInt(chatId);
    const position = this.positions.get(normalizedChatId);

    if (!position) {
      return null;
    }

    // Check if position is too old
    const age = Date.now() - position.timestamp;
    if (age > this.maxAge) {
      this.positions.delete(normalizedChatId);
      this.saveToStorage();
      return null;
    }

    return position;
  }

  /**
   * Determine scroll strategy for a chat
   */
  getScrollStrategy(chatId) {
    const normalizedChatId = parseInt(chatId);
    const isFirst = this.isFirstVisit(normalizedChatId);
    const savedPosition = this.getPosition(normalizedChatId);

    if (isFirst) {
      return {
        strategy: 'scroll-to-bottom',
        reason: 'first-visit',
        position: null
      };
    }

    if (savedPosition) {
      return {
        strategy: 'restore-position',
        reason: 'has-saved-position',
        position: savedPosition
      };
    }

    return {
      strategy: 'scroll-to-bottom',
      reason: 'no-saved-position',
      position: null
    };
  }

  /**
   * Apply scroll strategy to a container
   */
  async applyScrollStrategy(chatId, container, messages = []) {
    const strategy = this.getScrollStrategy(chatId);

    if (import.meta.env.DEV) {
      console.log(`📖 [ReadingPosition] Applying strategy for chat ${chatId}:`, strategy);
    }

    switch (strategy.strategy) {
      case 'scroll-to-bottom':
        this.scrollToBottom(container);
        this.recordVisit(chatId);
        break;

      case 'restore-position':
        await this.restorePosition(container, strategy.position, messages);
        break;
    }
  }

  /**
   * Scroll to bottom of container
   */
  scrollToBottom(container) {
    if (!container) return;

    // Use requestAnimationFrame for smooth execution
    requestAnimationFrame(() => {
      container.scrollTop = container.scrollHeight - container.clientHeight;

      if (import.meta.env.DEV) {
        console.log('📖 [ReadingPosition] Scrolled to bottom');
      }
    });
  }

  /**
   * Restore saved scroll position
   */
  async restorePosition(container, position, messages = []) {
    if (!container || !position) return;

    // Try to find the target message element
    if (position.messageId && messages.length > 0) {
      await this.scrollToMessage(container, position.messageId);
    } else {
      // Fallback to scroll position
      requestAnimationFrame(() => {
        container.scrollTop = position.scrollTop;

        if (import.meta.env.DEV) {
          console.log(`📖 [ReadingPosition] Restored scroll position: ${position.scrollTop}px`);
        }
      });
    }
  }

  /**
   * Scroll to a specific message
   */
  async scrollToMessage(container, messageId) {
    // Wait for DOM to be ready
    await new Promise(resolve => requestAnimationFrame(resolve));

    const messageElement = container.querySelector(`[data-message-id="${messageId}"]`);

    if (messageElement) {
      messageElement.scrollIntoView({
        behavior: 'smooth',
        block: 'center'
      });

      if (import.meta.env.DEV) {
        console.log(`📖 [ReadingPosition] Scrolled to message ${messageId}`);
      }
    } else {
      if (import.meta.env.DEV) {
        console.warn(`📖 [ReadingPosition] Message ${messageId} not found in DOM`);
      }
    }
  }

  /**
   * Get the currently visible message ID
   */
  getCurrentVisibleMessage(container) {
    if (!container) return null;

    const messageElements = container.querySelectorAll('[data-message-id]');
    const containerRect = container.getBoundingClientRect();
    const centerY = containerRect.top + containerRect.height / 2;

    let closestElement = null;
    let closestDistance = Infinity;

    messageElements.forEach(element => {
      const elementRect = element.getBoundingClientRect();
      const elementCenterY = elementRect.top + elementRect.height / 2;
      const distance = Math.abs(centerY - elementCenterY);

      if (distance < closestDistance) {
        closestDistance = distance;
        closestElement = element;
      }
    });

    return closestElement ? closestElement.getAttribute('data-message-id') : null;
  }

  /**
   * Auto-save current position
   */
  autoSavePosition(chatId, container) {
    if (!container) return;

    const scrollTop = container.scrollTop;
    const visibleMessageId = this.getCurrentVisibleMessage(container);

    this.savePosition(chatId, scrollTop, visibleMessageId);
  }

  /**
   * Load positions from localStorage
   */
  loadFromStorage() {
    try {
      const stored = localStorage.getItem(this.storageKey);
      if (stored) {
        const data = JSON.parse(stored);

        // Restore positions
        if (data.positions) {
          Object.entries(data.positions).forEach(([chatId, position]) => {
            this.positions.set(parseInt(chatId), position);
          });
        }

        // Restore visit history
        if (data.visitHistory) {
          Object.entries(data.visitHistory).forEach(([chatId, visits]) => {
            this.visitHistory.set(parseInt(chatId), visits);
          });
        }

        if (import.meta.env.DEV) {
          console.log(`📖 [ReadingPosition] Loaded ${this.positions.size} positions from storage`);
        }
      }
    } catch (error) {
      if (import.meta.env.DEV) {
        console.warn('📖 [ReadingPosition] Failed to load from storage:', error);
      }
    }
  }

  /**
   * Save positions to localStorage
   */
  saveToStorage() {
    try {
      // Clean up old positions
      this.cleanup();

      const data = {
        positions: Object.fromEntries(this.positions),
        visitHistory: Object.fromEntries(this.visitHistory),
        version: 1,
        timestamp: Date.now()
      };

      localStorage.setItem(this.storageKey, JSON.stringify(data));
    } catch (error) {
      if (import.meta.env.DEV) {
        console.warn('📖 [ReadingPosition] Failed to save to storage:', error);
      }
    }
  }

  /**
   * Clean up old positions
   */
  cleanup() {
    const now = Date.now();
    const toDelete = [];

    // Remove old positions
    this.positions.forEach((position, chatId) => {
      if (now - position.timestamp > this.maxAge) {
        toDelete.push(chatId);
      }
    });

    toDelete.forEach(chatId => {
      this.positions.delete(chatId);
      this.visitHistory.delete(chatId);
    });

    // Limit total positions
    if (this.positions.size > this.maxPositions) {
      const sorted = Array.from(this.positions.entries())
        .sort((a, b) => b[1].timestamp - a[1].timestamp) // Sort by timestamp, newest first
        .slice(this.maxPositions); // Keep only the newest

      this.positions.clear();
      sorted.forEach(([chatId, position]) => {
        this.positions.set(chatId, position);
      });
    }

    if (import.meta.env.DEV && toDelete.length > 0) {
      console.log(`📖 [ReadingPosition] Cleaned up ${toDelete.length} old positions`);
    }
  }

  /**
   * Clear all saved positions
   */
  clearAll() {
    this.positions.clear();
    this.visitHistory.clear();
    localStorage.removeItem(this.storageKey);

    if (import.meta.env.DEV) {
      console.log('📖 [ReadingPosition] Cleared all saved positions');
    }
  }

  /**
   * Get debug information
   */
  getDebugInfo() {
    return {
      totalPositions: this.positions.size,
      totalVisits: this.visitHistory.size,
      positions: Array.from(this.positions.entries()).map(([chatId, pos]) => ({
        chatId,
        scrollTop: pos.scrollTop,
        messageId: pos.messageId,
        age: Math.round((Date.now() - pos.timestamp) / 1000 / 60), // minutes
        timestamp: new Date(pos.timestamp).toLocaleString()
      })),
      visitHistory: Object.fromEntries(this.visitHistory)
    };
  }
}

// Create global instance
const readingPositionManager = new ReadingPositionManager();

// Export for module usage
export { readingPositionManager };

// Make available globally in development
if (import.meta.env.DEV) {
  window.readingPositionManager = readingPositionManager;

  // Add debug commands
  window.debugReadingPositions = () => {
    console.log('📖 [ReadingPosition] Debug Info:', readingPositionManager.getDebugInfo());
  };

  window.clearReadingPositions = () => {
    readingPositionManager.clearAll();
    console.log('📖 [ReadingPosition] All positions cleared');
  };
}