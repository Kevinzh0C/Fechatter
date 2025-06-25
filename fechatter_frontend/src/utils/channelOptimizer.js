/**
 * Channel Performance Optimizer
 * Following Frontend Design Principles:
 * - Experience Layer: 80/20 Rule, optimize most used channels
 * - UI Interaction Layer: Progressive Disclosure, instant feedback
 * - Component State Layer: Local-State-First, aggressive caching
 * - Build Resource Layer: Performance Budget, minimize latency
 */

import { useChatStore } from '@/stores/chat';

class ChannelOptimizer {
  constructor() {
    // Cache configuration
    this.preloadQueue = new Set();
    this.activePreloads = new Map();
    this.channelAccessHistory = new Map(); // Track access patterns
    this.messageCache = new Map(); // Enhanced message cache
    this.memberCache = new Map(); // Member cache

    // Performance settings
    this.config = {
      preloadMessageCount: 50, // Load more messages upfront
      maxPreloadChannels: 5, // Preload top 5 channels
      cacheTimeout: 5 * 60 * 1000, // 5 minutes
      parallelRequests: true, // Enable parallel loading
      instantSwitch: true, // Show cached data immediately
      backgroundRefresh: true // Refresh data in background
    };

    // Initialize
    this.chatStore = null;
    this.initializeOptimizer();
  }

  /**
   * Initialize optimizer
   */
  async initializeOptimizer() {
    // Wait for store to be available
    setTimeout(() => {
      this.chatStore = useChatStore();
      this.startPreloadingTopChannels();
    }, 100);
  }

  /**
   * Optimized channel switch
   * YAGNI: Simple but effective optimization
   */
  async switchChannel(chatId, options = {}) {
    const startTime = performance.now();

    // Record access for pattern learning
    this.recordChannelAccess(chatId);

    // 1. Instant UI update with cached data
    if (this.config.instantSwitch) {
      const cached = this.getCachedData(chatId);
      if (cached) {
        this.applyCachedData(cached);
      }
    }

    // 2. Parallel data fetching
    const fetchPromises = [];

    if (this.config.parallelRequests) {
      // Fetch messages, members, and chat info in parallel
      fetchPromises.push(this.fetchMessagesOptimized(chatId));
      fetchPromises.push(this.fetchMembersOptimized(chatId));
      fetchPromises.push(this.chatStore.setCurrentChat(chatId));
    } else {
      // Fallback to sequential loading
      await this.fetchMessagesOptimized(chatId);
      await this.fetchMembersOptimized(chatId);
      await this.chatStore.setCurrentChat(chatId);
    }

    // 3. Wait for all data
    const results = await Promise.allSettled(fetchPromises);

    // 4. Background refresh if needed
    if (this.config.backgroundRefresh) {
      this.scheduleBackgroundRefresh(chatId);
    }

    // 5. Preload adjacent channels
    this.preloadAdjacentChannels(chatId);

    const loadTime = performance.now() - startTime;
    console.log(`⚡ Channel switched in ${loadTime.toFixed(2)}ms`);

    return results;
  }

  /**
   * Fetch messages with optimization
   */
  async fetchMessagesOptimized(chatId) {
    // Check enhanced cache first
    const cached = this.messageCache.get(chatId);
    if (cached && Date.now() - cached.timestamp < this.config.cacheTimeout) {
      this.chatStore.messages = cached.messages;
      return cached.messages;
    }

    // Fetch with optimized count
    const messages = await this.chatStore.fetchMessages(
      chatId,
      this.config.preloadMessageCount
    );

    // Update enhanced cache
    this.messageCache.set(chatId, {
      messages,
      timestamp: Date.now(),
      hasMore: messages.length >= this.config.preloadMessageCount
    });

    return messages;
  }

  /**
   * Fetch members with caching
   */
  async fetchMembersOptimized(chatId) {
    // Check cache first
    const cached = this.memberCache.get(chatId);
    if (cached && Date.now() - cached.timestamp < this.config.cacheTimeout * 2) {
      return cached.members;
    }

    try {
      const members = await this.chatStore.fetchChatMembers(chatId);

      // Cache members
      this.memberCache.set(chatId, {
        members,
        timestamp: Date.now()
      });

      return members;
    } catch (error) {
      console.warn('Failed to fetch members:', error);
      return [];
    }
  }

  /**
   * Get all cached data for instant display
   */
  getCachedData(chatId) {
    const messages = this.messageCache.get(chatId);
    const members = this.memberCache.get(chatId);

    if (!messages) return null;

    return {
      messages: messages.messages,
      members: members?.members || [],
      timestamp: messages.timestamp
    };
  }

  /**
   * Apply cached data immediately
   */
  applyCachedData(cached) {
    if (!this.chatStore) return;

    this.chatStore.messages = cached.messages;
    if (cached.members.length > 0) {
      this.chatStore.chatMembers[this.chatStore.currentChatId] = cached.members;
    }
  }

  /**
   * Record channel access patterns
   */
  recordChannelAccess(chatId) {
    const access = this.channelAccessHistory.get(chatId) || {
      count: 0,
      lastAccess: 0,
      totalTime: 0
    };

    access.count++;
    access.lastAccess = Date.now();

    this.channelAccessHistory.set(chatId, access);
  }

  /**
   * Start preloading top channels based on access patterns
   */
  async startPreloadingTopChannels() {
    if (!this.chatStore) return;

    // Get top channels by access frequency
    const topChannels = this.getTopChannels();

    // Preload each channel in background
    for (const chatId of topChannels) {
      if (!this.activePreloads.has(chatId)) {
        this.preloadChannel(chatId);
      }
    }
  }

  /**
   * Get most accessed channels
   */
  getTopChannels() {
    const channels = Array.from(this.channelAccessHistory.entries())
      .sort((a, b) => {
        // Sort by access count and recency
        const scoreA = a[1].count + (Date.now() - a[1].lastAccess) / 1000000;
        const scoreB = b[1].count + (Date.now() - b[1].lastAccess) / 1000000;
        return scoreB - scoreA;
      })
      .slice(0, this.config.maxPreloadChannels)
      .map(([chatId]) => chatId);

    // Add current open chats if not in history
    if (this.chatStore?.chats) {
      const recentChats = this.chatStore.chats
        .slice(0, 3)
        .map(chat => chat.id);

      for (const chatId of recentChats) {
        if (!channels.includes(chatId)) {
          channels.push(chatId);
        }
      }
    }

    return channels.slice(0, this.config.maxPreloadChannels);
  }

  /**
   * Preload channel data in background
   */
  async preloadChannel(chatId) {
    if (this.activePreloads.has(chatId)) return;

    const preloadPromise = (async () => {
      try {
        // Fetch data silently
        const [messages, members] = await Promise.allSettled([
          this.fetchMessagesOptimized(chatId),
          this.fetchMembersOptimized(chatId)
        ]);

        console.log(`📦 Preloaded channel ${chatId}`);
      } catch (error) {
        console.warn(`Failed to preload channel ${chatId}:`, error);
      } finally {
        this.activePreloads.delete(chatId);
      }
    })();

    this.activePreloads.set(chatId, preloadPromise);
  }

  /**
   * Preload adjacent channels (next/previous in list)
   */
  preloadAdjacentChannels(currentChatId) {
    if (!this.chatStore?.chats) return;

    const currentIndex = this.chatStore.chats.findIndex(c => c.id === currentChatId);
    if (currentIndex === -1) return;

    // Preload next and previous channels
    const adjacentIndexes = [currentIndex - 1, currentIndex + 1];

    adjacentIndexes.forEach(index => {
      if (index >= 0 && index < this.chatStore.chats.length) {
        const chatId = this.chatStore.chats[index].id;
        this.preloadChannel(chatId);
      }
    });
  }

  /**
   * Schedule background refresh for stale data
   */
  scheduleBackgroundRefresh(chatId) {
    const cached = this.messageCache.get(chatId);
    if (!cached) return;

    const age = Date.now() - cached.timestamp;
    const refreshDelay = Math.max(0, this.config.cacheTimeout - age);

    setTimeout(() => {
      // Only refresh if still on same channel
      if (this.chatStore?.currentChatId === chatId) {
        this.fetchMessagesOptimized(chatId);
      }
    }, refreshDelay);
  }

  /**
   * Clear old cache entries
   */
  cleanupCache() {
    const now = Date.now();
    const timeout = this.config.cacheTimeout * 2;

    // Clean message cache
    for (const [chatId, data] of this.messageCache.entries()) {
      if (now - data.timestamp > timeout) {
        this.messageCache.delete(chatId);
      }
    }

    // Clean member cache
    for (const [chatId, data] of this.memberCache.entries()) {
      if (now - data.timestamp > timeout) {
        this.memberCache.delete(chatId);
      }
    }
  }

  /**
   * Get optimizer statistics
   */
  getStats() {
    return {
      cachedChannels: this.messageCache.size,
      cachedMembers: this.memberCache.size,
      accessHistory: this.channelAccessHistory.size,
      activePreloads: this.activePreloads.size,
      topChannels: this.getTopChannels()
    };
  }

  /**
   * Reset optimizer
   */
  reset() {
    this.messageCache.clear();
    this.memberCache.clear();
    this.channelAccessHistory.clear();
    this.activePreloads.clear();
    this.preloadQueue.clear();
  }
}

// Create singleton instance
const channelOptimizer = new ChannelOptimizer();

// Auto cleanup every 5 minutes
setInterval(() => {
  channelOptimizer.cleanupCache();
}, 5 * 60 * 1000);

// Export for debugging
if (import.meta.env.DEV) {
  window.channelOptimizer = channelOptimizer;
}

export default channelOptimizer;
export const optimizeChannelSwitch = (chatId, options) =>
  channelOptimizer.switchChannel(chatId, options); 