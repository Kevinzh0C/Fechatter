/**
 * 🔄 REFACTOR: User Data Resolver Service
 * 
 * Follows frontend design principles:
 * - Single Responsibility: Only handles user data resolution
 * - Dependency Injection: No global variable access
 * - Open/Closed: Extensible resolver strategies
 * - Testable: All dependencies can be mocked
 */

export class UserDataResolver {
  constructor(dependencies = {}) {
    // 🔧 Dependency Injection instead of global access
    this.userStore = dependencies.userStore;
    this.workspaceStore = dependencies.workspaceStore;
    this.cacheProvider = dependencies.cacheProvider || new Map();
    this.logger = dependencies.logger || console;

    // 🔧 Strategy Pattern for user resolution
    this.resolutionStrategies = [
      new EmbeddedUserStrategy(),
      new UserStoreStrategy(this.userStore),
      new WorkspaceStoreStrategy(this.workspaceStore),
      new EmailPrefixStrategy(),
      new FallbackStrategy()
    ];

    // 🔧 Performance: TTL Cache with 5-minute expiration
    this.cache = new Map();
    this.cacheTTL = 5 * 60 * 1000; // 5 minutes
    this.cacheCleanupInterval = setInterval(() => this.cleanupCache(), 60000);

    // 🔧 Metrics for monitoring
    this.metrics = {
      totalQueries: 0,
      cacheHits: 0,
      strategyUsage: new Map(),
      averageResolutionTime: 0
    };
  }

  /**
   * Resolve user name using strategy pattern
   * @param {Object} message - Message object
   * @param {Object} context - Resolution context
   * @returns {Promise<string>} Resolved user name
   */
  async resolveUserName(message, context = {}) {
    const startTime = performance.now();
    this.metrics.totalQueries++;

    try {
      const cacheKey = this.generateCacheKey(message);

      // 🔧 Check cache first
      const cached = this.getCachedResult(cacheKey);
      if (cached) {
        this.metrics.cacheHits++;
        return cached;
      }

      // 🔧 Try strategies in order
      for (const strategy of this.resolutionStrategies) {
        try {
          const result = await strategy.resolve(message, context);
          if (result) {
            // Update metrics
            const strategyName = strategy.constructor.name;
            this.metrics.strategyUsage.set(
              strategyName,
              (this.metrics.strategyUsage.get(strategyName) || 0) + 1
            );

            // Cache result
            this.setCachedResult(cacheKey, result);

            this.logger.debug(`✅ [UserDataResolver] Resolved via ${strategyName}: ${result}`);
            return result;
          }
        } catch (error) {
          this.logger.warn(`⚠️ [UserDataResolver] Strategy ${strategy.constructor.name} failed:`, error);
          continue; // Try next strategy
        }
      }

      // All strategies failed
      const fallback = `User ${message.sender_id}`;
      this.setCachedResult(cacheKey, fallback);
      return fallback;

    } finally {
      // Update performance metrics
      const resolutionTime = performance.now() - startTime;
      this.updateAverageResolutionTime(resolutionTime);
    }
  }

  /**
   * Batch resolve multiple users for performance
   * @param {Array} messages - Array of messages
   * @returns {Promise<Map>} Map of message.id -> resolved name
   */
  async batchResolveUsers(messages) {
    const uniqueUsers = new Map();
    const results = new Map();

    // 🔧 Deduplicate users to reduce queries
    for (const message of messages) {
      const key = this.generateCacheKey(message);
      if (!uniqueUsers.has(key)) {
        uniqueUsers.set(key, message);
      }
    }

    // 🔧 Resolve unique users in parallel
    const resolutionPromises = Array.from(uniqueUsers.values()).map(async (message) => {
      const name = await this.resolveUserName(message);
      return { messageId: message.id, name };
    });

    const resolved = await Promise.all(resolutionPromises);

    // 🔧 Build result map
    for (const { messageId, name } of resolved) {
      results.set(messageId, name);
    }

    this.logger.debug(`✅ [UserDataResolver] Batch resolved ${uniqueUsers.size} unique users for ${messages.length} messages`);
    return results;
  }

  /**
   * Generate cache key for message
   */
  generateCacheKey(message) {
    return `user_${message.sender_id}_${message.sender?.email || 'no-email'}`;
  }

  /**
   * Get cached result with TTL check
   */
  getCachedResult(key) {
    const cached = this.cache.get(key);
    if (!cached) return null;

    if (Date.now() - cached.timestamp > this.cacheTTL) {
      this.cache.delete(key);
      return null;
    }

    return cached.value;
  }

  /**
   * Set cached result with timestamp
   */
  setCachedResult(key, value) {
    this.cache.set(key, {
      value,
      timestamp: Date.now()
    });
  }

  /**
   * Clean up expired cache entries
   */
  cleanupCache() {
    const now = Date.now();
    let cleanedCount = 0;

    for (const [key, cached] of this.cache.entries()) {
      if (now - cached.timestamp > this.cacheTTL) {
        this.cache.delete(key);
        cleanedCount++;
      }
    }

    if (cleanedCount > 0) {
      this.logger.debug(`🧹 [UserDataResolver] Cleaned up ${cleanedCount} expired cache entries`);
    }
  }

  /**
   * Update average resolution time metric
   */
  updateAverageResolutionTime(newTime) {
    if (this.metrics.averageResolutionTime === 0) {
      this.metrics.averageResolutionTime = newTime;
    } else {
      // Exponential moving average
      this.metrics.averageResolutionTime = (this.metrics.averageResolutionTime * 0.8) + (newTime * 0.2);
    }
  }

  /**
   * Get performance metrics
   */
  getMetrics() {
    const cacheHitRate = this.metrics.totalQueries > 0
      ? ((this.metrics.cacheHits / this.metrics.totalQueries) * 100).toFixed(2)
      : '0.00';

    return {
      ...this.metrics,
      cacheHitRate: `${cacheHitRate}%`,
      cacheSize: this.cache.size,
      strategiesCount: this.resolutionStrategies.length
    };
  }

  /**
   * Cleanup resources
   */
  destroy() {
    if (this.cacheCleanupInterval) {
      clearInterval(this.cacheCleanupInterval);
    }
    this.cache.clear();
  }
}

/**
 * Strategy Implementations
 */

class EmbeddedUserStrategy {
  async resolve(message) {
    return message.sender?.fullname || message.sender?.full_name;
  }
}

class UserStoreStrategy {
  constructor(userStore) {
    this.userStore = userStore;
  }

  async resolve(message) {
    if (!this.userStore?.getUserById) return null;

    try {
      const user = await this.userStore.getUserById(message.sender_id);
      return user?.fullname || user?.full_name || user?.name;
    } catch (error) {
      console.warn('UserStore resolution failed:', error);
      return null;
    }
  }
}

class WorkspaceStoreStrategy {
  constructor(workspaceStore) {
    this.workspaceStore = workspaceStore;
  }

  async resolve(message) {
    if (!this.workspaceStore?.workspaceUsers) return null;

    try {
      const user = this.workspaceStore.workspaceUsers.find(
        u => u.id === message.sender_id
      );
      return user?.fullname || user?.full_name || user?.name;
    } catch (error) {
      console.warn('WorkspaceStore resolution failed:', error);
      return null;
    }
  }
}

class EmailPrefixStrategy {
  async resolve(message) {
    const email = message.sender?.email;
    if (!email) return null;

    return email.split('@')[0];
  }
}

class FallbackStrategy {
  async resolve(message) {
    return `User ${message.sender_id}`;
  }
}

/**
 * 🏭 FACTORY: Create resolver with proper dependencies
 */
export function createUserDataResolver(stores) {
  return new UserDataResolver({
    userStore: stores.userStore,
    workspaceStore: stores.workspaceStore,
    cacheTTL: 5 * 60 * 1000 // 5 minutes
  });
} 