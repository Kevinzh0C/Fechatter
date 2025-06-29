/**
 * Perfect Search Service - Jobs-inspired Design
 * Intelligent search with multiple strategies and elegant UX
 */

import api from './api'

class PerfectSearchService {
  constructor() {
    this.cache = new Map()
    this.searchHistory = []
    this.isInitialized = false
    this.metrics = {
      totalSearches: 0,
      avgResponseTime: 0,
      cacheHitRate: 0,
      errorRate: 0
    }

    // Search strategies ordered by intelligence
    this.strategies = [
      'semantic',    // AI-powered semantic search
      'full_text',   // Full text search with ranking
      'fuzzy',       // Fuzzy matching for typos
      'exact',       // Exact phrase matching
      'recent',      // Time-weighted search
      'user_scoped', // User-specific search
      'file_content' // File content search
    ]
  }

  /**
   * Initialize the search service
   */
  async initialize() {
    if (this.isInitialized) return

    try {
      // Check backend connectivity
      const healthCheck = await api.get('/health')
      if (healthCheck.data?.services?.search?.status !== 'healthy') {
        throw new Error('Search service not healthy')
      }

      // Load search configuration
      await this.loadSearchConfig()

      this.isInitialized = true
      console.log('Perfect Search Service initialized')

    } catch (error) {
      console.error('ERROR: Failed to initialize search service:', error)
      throw error
    }
  }

  /**
   * Load search configuration from backend
   */
  async loadSearchConfig() {
    try {
      const response = await api.get('/api/search/config')
      this.config = response.data || {}
    } catch (error) {
      // Fallback configuration
      this.config = {
        maxResults: 50,
        timeout: 5000,
        enableCache: true,
        strategies: this.strategies
      }
    }
  }

  /**
   * Intelligent search with multiple strategies
   */
  async search(params) {
    const startTime = performance.now()
    this.metrics.totalSearches++

    try {
      // Validate parameters
      const searchParams = this.validateSearchParams(params)

      // Check cache first
      const cacheKey = this.generateCacheKey(searchParams)
      if (this.config.enableCache && this.cache.has(cacheKey)) {
        this.metrics.cacheHitRate++
        return this.cache.get(cacheKey)
      }

      // Execute search with intelligent strategy selection
      const results = await this.executeIntelligentSearch(searchParams)

      // Cache results
      if (this.config.enableCache && results.hits.length > 0) {
        this.cache.set(cacheKey, results)

        // Clean cache if too large
        if (this.cache.size > 100) {
          const firstKey = this.cache.keys().next().value
          this.cache.delete(firstKey)
        }
      }

      // Update metrics
      const responseTime = performance.now() - startTime
      this.updateMetrics(responseTime, true)

      // Store in search history
      this.addToHistory(searchParams, results)

      return results

    } catch (error) {
      console.error('Search failed:', error)
      this.updateMetrics(performance.now() - startTime, false)

      // Return fallback results
      return {
        hits: [],
        total: 0,
        took_ms: Math.round(performance.now() - startTime),
        strategy_used: 'fallback',
        error: error.message
      }
    }
  }

  /**
   * Execute intelligent search with strategy selection
   */
  async executeIntelligentSearch(params) {
    const { query, chatId, filters = {}, limit = 20, offset = 0 } = params

    // Determine best search strategy based on query characteristics
    const strategy = this.selectSearchStrategy(query)

    try {
      let endpoint = '/api/search/messages'
      let searchData = {
        q: query,
        limit,
        offset,
        strategy,
        ...filters
      }

      // Chat-specific search
      if (chatId) {
        endpoint = `/api/chat/${chatId}/messages/search`
      }

      // Execute search
      const response = await api.get(endpoint, {
        params: searchData,
        timeout: this.config.timeout
      })

      const results = response.data?.data || response.data || {}

      return {
        hits: results.hits || [],
        total: results.total || 0,
        took_ms: results.took_ms || 0,
        strategy_used: strategy,
        query_analysis: this.analyzeQuery(query)
      }

    } catch (error) {
      // Fallback to local search if backend fails
      console.warn('Backend search failed, trying local fallback:', error)
      return await this.fallbackLocalSearch(params)
    }
  }

  /**
   * Select the best search strategy based on query characteristics
   */
  selectSearchStrategy(query) {
    if (!query || query.length < 2) return 'recent'

    // Exact phrase search (quoted strings)
    if (query.startsWith('"') && query.endsWith('"')) {
      return 'exact'
    }

    // File search patterns
    if (query.includes('file:') || query.includes('attachment:')) {
      return 'file_content'
    }

    // User search patterns
    if (query.includes('from:') || query.includes('by:')) {
      return 'user_scoped'
    }

    // Boolean search patterns
    if (query.includes(' AND ') || query.includes(' OR ') || query.includes(' NOT ')) {
      return 'boolean'
    }

    // Recent search patterns
    if (query.includes('today') || query.includes('yesterday') || query.includes('recent')) {
      return 'recent'
    }

    // Semantic search for natural language queries
    if (query.split(' ').length > 3) {
      return 'semantic'
    }

    // Fuzzy search for short queries (possible typos)
    if (query.length < 6) {
      return 'fuzzy'
    }

    // Default to full text search
    return 'full_text'
  }

  /**
   * Analyze query characteristics
   */
  analyzeQuery(query) {
    return {
      length: query.length,
      words: query.split(' ').length,
      hasSpecialChars: /[^a-zA-Z0-9\s]/.test(query),
      isQuoted: query.startsWith('"') && query.endsWith('"'),
      hasFilters: /\b(from|to|file|attachment|recent|today):/i.test(query),
      language: this.detectLanguage(query)
    }
  }

  /**
   * Detect query language for better search results
   */
  detectLanguage(query) {
    // Simple language detection
    const patterns = {
      japanese: /[\u3040-\u309F\u30A0-\u30FF\u4E00-\u9FAF]/,
      chinese: /[\u4E00-\u9FFF]/,
      korean: /[\uAC00-\uD7AF]/,
      english: /^[a-zA-Z\s]+$/
    }

    for (const [lang, pattern] of Object.entries(patterns)) {
      if (pattern.test(query)) {
        return lang
      }
    }

    return 'unknown'
  }

  /**
   * Fallback local search when backend is unavailable
   */
  async fallbackLocalSearch(params) {
    // Implement basic local search using cached data
    // This would search through locally cached messages
    console.log('Executing fallback local search')

    return {
      hits: [],
      total: 0,
      took_ms: 0,
      strategy_used: 'fallback_local',
      note: 'Backend search unavailable, local search not implemented'
    }
  }

  /**
   * Validate and normalize search parameters
   */
  validateSearchParams(params) {
    const {
      query = '',
      chatId = null,
      filters = {},
      limit = 20,
      offset = 0,
      strategies = null
    } = params

    return {
      query: query.trim(),
      chatId: chatId ? parseInt(chatId) : null,
      filters: {
        timeRange: filters.timeRange || 'all',
        sender: filters.sender || '',
        fileType: filters.fileType || '',
        ...filters
      },
      limit: Math.min(Math.max(1, parseInt(limit)), 100),
      offset: Math.max(0, parseInt(offset)),
      strategies: strategies || this.strategies
    }
  }

  /**
   * Generate cache key for search results
   */
  generateCacheKey(params) {
    const key = `${params.query}:${params.chatId}:${JSON.stringify(params.filters)}:${params.limit}:${params.offset}`
    return btoa(key).replace(/[^a-zA-Z0-9]/g, '').substring(0, 32)
  }

  /**
   * Update search metrics
   */
  updateMetrics(responseTime, success) {
    this.metrics.avgResponseTime =
      (this.metrics.avgResponseTime * (this.metrics.totalSearches - 1) + responseTime) /
      this.metrics.totalSearches

    if (!success) {
      this.metrics.errorRate =
        (this.metrics.errorRate * (this.metrics.totalSearches - 1) + 1) /
        this.metrics.totalSearches
    }

    this.metrics.cacheHitRate = this.metrics.cacheHitRate / this.metrics.totalSearches
  }

  /**
   * Add search to history
   */
  addToHistory(params, results) {
    this.searchHistory.unshift({
      query: params.query,
      chatId: params.chatId,
      resultCount: results.hits.length,
      timestamp: new Date().toISOString(),
      strategy: results.strategy_used
    })

    // Keep only last 50 searches
    if (this.searchHistory.length > 50) {
      this.searchHistory.pop()
    }
  }

  /**
   * Get search suggestions based on history
   */
  getSuggestions(query = '') {
    if (!query) {
      // Return popular searches
      return this.getPopularSearches()
    }

    // Return matching searches from history
    return this.searchHistory
      .filter(search =>
        search.query.toLowerCase().includes(query.toLowerCase()) ||
        query.toLowerCase().includes(search.query.toLowerCase())
      )
      .slice(0, 5)
      .map(search => search.query)
  }

  /**
   * Get popular searches from history
   */
  getPopularSearches() {
    const queryCount = new Map()

    this.searchHistory.forEach(search => {
      const count = queryCount.get(search.query) || 0
      queryCount.set(search.query, count + 1)
    })

    return Array.from(queryCount.entries())
      .sort((a, b) => b[1] - a[1])
      .slice(0, 8)
      .map(entry => entry[0])
  }

  /**
   * Clear search cache
   */
  clearCache() {
    this.cache.clear()
    console.log('Search cache cleared')
  }

  /**
   * Get search metrics
   */
  getMetrics() {
    return {
      ...this.metrics,
      cacheSize: this.cache.size,
      historySize: this.searchHistory.length,
      isInitialized: this.isInitialized
    }
  }

  /**
   * Test search connectivity
   */
  async testConnectivity() {
    try {
      const testQuery = 'test'
      const result = await this.search({ query: testQuery, limit: 1 })
      return {
        success: true,
        responseTime: result.took_ms,
        strategy: result.strategy_used
      }
    } catch (error) {
      return {
        success: false,
        error: error.message
      }
    }
  }
}

// Create singleton instance
const perfectSearchService = new PerfectSearchService()

export default perfectSearchService 