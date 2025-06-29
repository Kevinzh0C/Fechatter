/**
 * 搜索状态管理器
 * 消除搜索功能中的并发错误和平行错误路径
 * 
 * 核心功能：
 * 1. 防止重复搜索请求
 * 2. 管理搜索状态一致性
 * 3. 处理搜索请求竞态条件
 * 4. 提供搜索结果缓存
 */

class SearchStateManager {
  constructor() {
    this.state = {
      // 当前活跃的搜索请求
      activeRequests: new Map(),

      // 搜索结果缓存
      cache: new Map(),

      // 当前搜索状态
      currentSearch: {
        chatId: null,
        query: '',
        isSearching: false,
        results: null,
        error: null,
        timestamp: null
      },

      // 请求序列号，防止过期请求覆盖新请求
      requestSequence: 0,

      // 缓存配置
      cacheConfig: {
        maxSize: 100,        // 最大缓存条目数
        ttl: 5 * 60 * 1000,  // 缓存存活时间: 5分钟
        enabled: true        // 是否启用缓存
      },

      // 并发控制
      concurrencyConfig: {
        maxConcurrentRequests: 3,  // 最大并发搜索请求数
        debounceTime: 300,         // 搜索去抖时间
        timeout: 10000             // 请求超时时间
      }
    };

    // 去抖计时器
    this.debounceTimers = new Map();

    // 绑定方法
    this.search = this.search.bind(this);
    this.cancelSearch = this.cancelSearch.bind(this);
    this.clearCache = this.clearCache.bind(this);
  }

  /**
   * 生成缓存键
   */
  generateCacheKey(chatId, query, filters = {}) {
    const filterStr = Object.keys(filters)
      .sort()
      .map(key => `${key}:${filters[key]}`)
      .join('|');
    return `${chatId}:${query.toLowerCase().trim()}:${filterStr}`;
  }

  /**
   * 生成请求ID
   */
  generateRequestId(chatId, query) {
    this.state.requestSequence += 1;
    return `${chatId}:${query}:${this.state.requestSequence}`;
  }

  /**
   * 检查缓存是否有效
   */
  isCacheValid(cacheEntry) {
    if (!this.state.cacheConfig.enabled || !cacheEntry) return false;
    return (Date.now() - cacheEntry.timestamp) < this.state.cacheConfig.ttl;
  }

  /**
   * 从缓存获取结果
   */
  getCachedResult(chatId, query, filters = {}) {
    const cacheKey = this.generateCacheKey(chatId, query, filters);
    const cacheEntry = this.state.cache.get(cacheKey);

    if (this.isCacheValid(cacheEntry)) {
      return cacheEntry.data;
    }

    return null;
  }

  /**
   * 缓存搜索结果
   */
  setCachedResult(chatId, query, filters = {}, data) {
    if (!this.state.cacheConfig.enabled) return;

    const cacheKey = this.generateCacheKey(chatId, query, filters);

    // 如果缓存超出大小限制，删除最旧的条目
    if (this.state.cache.size >= this.state.cacheConfig.maxSize) {
      const oldestKey = this.state.cache.keys().next().value;
      this.state.cache.delete(oldestKey);
    }

    this.state.cache.set(cacheKey, {
      data: data,
      timestamp: Date.now(),
      chatId: chatId,
      query: query
    });

  /**
   * 取消指定的搜索请求
   */
  cancelSearch(requestId) {
    const activeRequest = this.state.activeRequests.get(requestId);
    if (activeRequest) {
      // 取消axios请求
      if (activeRequest.cancelToken) {
        activeRequest.cancelToken.cancel('Search cancelled by user');
      }

      // 清理状态
      this.state.activeRequests.delete(requestId);

      return true;
    return false;
  }

  /**
   * 取消所有活跃的搜索请求
   */
  cancelAllSearches() {
    for (const [requestId, request] of this.state.activeRequests) {
      if (request.cancelToken) {
        request.cancelToken.cancel('All searches cancelled');
      }

    this.state.activeRequests.clear();
    this.state.currentSearch.isSearching = false;
  }

  /**
   * 清理过期的缓存条目
   */
  cleanupCache() {
    const now = Date.now();
    const ttl = this.state.cacheConfig.ttl;

    for (const [key, entry] of this.state.cache) {
      if ((now - entry.timestamp) > ttl) {
        this.state.cache.delete(key);
      }

  /**
   * 主要的搜索方法
   * 处理去抖、缓存、并发控制
   */
  async search(searchParams, searchFunction) {
    const { chatId, query, ...filters } = searchParams;

    // 输入验证
    if (!chatId && chatId !== 0) {
    }

    if (typeof query !== 'string') {
      throw new Error('Search query must be a string');
    }

    const trimmedQuery = query.trim();
    if (trimmedQuery.length === 0 && !filters.hasFiles && !filters.hasLinks && !filters.senderId) {
      // Return empty results for empty search with no filters
      return { results: [], total: 0, took_ms: 0 };
    }

    // 检查缓存
    const cachedResult = this.getCachedResult(chatId, trimmedQuery, filters);
    if (cachedResult) {
      this.state.currentSearch = {
        chatId,
        query: trimmedQuery,
        isSearching: false,
        results: cachedResult,
        error: null,
        timestamp: Date.now()
      };
      return cachedResult;
    }

    // 生成请求ID
    const requestId = this.generateRequestId(chatId, trimmedQuery);

    // 检查并发限制
    if (this.state.activeRequests.size >= this.state.concurrencyConfig.maxConcurrentRequests) {
      const oldestRequestId = this.state.activeRequests.keys().next().value;
      this.cancelSearch(oldestRequestId);
    }

    // 创建取消token
    const cancelToken = {
      token: null,
      cancel: (reason) => {
      }
    };

    // 设置请求超时
    const timeoutId = setTimeout(() => {
      cancelToken.cancel('Search request timeout');
    }, this.state.concurrencyConfig.timeout);

    try {
      // 更新状态
      this.state.currentSearch = {
        chatId,
        query: trimmedQuery,
        isSearching: true,
        results: null,
        error: null,
        timestamp: Date.now()
      };

      // 注册活跃请求
      this.state.activeRequests.set(requestId, {
        cancelToken,
        timestamp: Date.now(),
        chatId,
        query: trimmedQuery
      });

      // 执行搜索
      const searchResult = await searchFunction({
        ...searchParams,
        query: trimmedQuery,
        cancelToken: cancelToken.token
      });

      // 检查请求是否仍然有效
      if (!this.state.activeRequests.has(requestId)) {
        return null;
      }

      // 缓存结果
      this.setCachedResult(chatId, trimmedQuery, filters, searchResult);

      // 更新状态
      this.state.currentSearch = {
        chatId,
        query: trimmedQuery,
        isSearching: false,
        results: searchResult,
        error: null,
        timestamp: Date.now()
      };

      return searchResult;

    } catch (error) {
      // 处理取消的请求
      if (error.message && error.message.includes('cancel')) {
        return null;
      }

      // 更新错误状态
      this.state.currentSearch = {
        chatId,
        query: trimmedQuery,
        isSearching: false,
        results: null,
        error: error.message,
        timestamp: Date.now()
      };

      throw error;

    } finally {
      // 清理
      clearTimeout(timeoutId);
      this.state.activeRequests.delete(requestId);
    }

  /**
   * 带去抖的搜索方法
   */
  searchWithDebounce(searchParams, searchFunction, debounceKey = 'default') {
    return new Promise((resolve, reject) => {
      // 清除之前的去抖计时器
      if (this.debounceTimers.has(debounceKey)) {
        clearTimeout(this.debounceTimers.get(debounceKey));
      }

      // 设置新的去抖计时器
      const timerId = setTimeout(async () => {
        try {
          const result = await this.search(searchParams, searchFunction);
          resolve(result);
        } catch (error) {
          reject(error);
        } finally {
          this.debounceTimers.delete(debounceKey);
        }
      }, this.state.concurrencyConfig.debounceTime);

      this.debounceTimers.set(debounceKey, timerId);
    });

  /**
   * 获取当前搜索状态
   */
  getCurrentState() {
    return {
      ...this.state.currentSearch,
      activeRequestsCount: this.state.activeRequests.size,
      cacheSize: this.state.cache.size
    };
  }

  /**
   * 清除所有缓存
   */
  clearCache() {
    this.state.cache.clear();
  }

  /**
   * 重置搜索状态
   */
  reset() {
    this.cancelAllSearches();
    this.clearCache();
    this.state.currentSearch = {
      chatId: null,
      query: '',
      isSearching: false,
      results: null,
      error: null,
      timestamp: null
    };
    this.state.requestSequence = 0;

    // 清除所有去抖计时器
    for (const timerId of this.debounceTimers.values()) {
      clearTimeout(timerId);
    this.debounceTimers.clear();
  }

  /**
   * 获取搜索统计信息
   */
  getStats() {
    return {
      activeRequests: this.state.activeRequests.size,
      cacheSize: this.state.cache.size,
      requestSequence: this.state.requestSequence,
      cacheConfig: this.state.cacheConfig,
      concurrencyConfig: this.state.concurrencyConfig,
      currentSearch: this.state.currentSearch
    };
  }

  /**
   * 更新配置
   */
  updateConfig(newConfig) {
    if (newConfig.cache) {
      Object.assign(this.state.cacheConfig, newConfig.cache);
    if (newConfig.concurrency) {
      Object.assign(this.state.concurrencyConfig, newConfig.concurrency);
    }

// 创建全局实例
const searchStateManager = new SearchStateManager();

// 暴露到window对象用于调试
// if (typeof window !== 'undefined') {
//   window.searchStateManager = searchStateManager;

//   console.log('Search State Manager loaded. Available commands:');
//   console.log('   window.searchStateManager.getCurrentState() - Get current state');
//   console.log('   window.searchStateManager.getStats() - Get statistics');
//   console.log('   window.searchStateManager.clearCache() - Clear cache');
//   console.log('   window.searchStateManager.reset() - Reset all state');
//   console.log('   window.searchStateManager.cancelAllSearches() - Cancel active searches');
// }

export default searchStateManager;