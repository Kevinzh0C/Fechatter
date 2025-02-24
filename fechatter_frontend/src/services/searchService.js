/**
 * 🔍 Production Search Service - Advanced Multi-Strategy Backend Integration
 * 
 * Implements multiple search strategies with backend API support,
 * fallback mechanisms, and performance optimization
 */

import api from '@/services/api';
import { useNotifications } from '@/composables/useNotifications';

/**
 * Backend-Supported Search Strategies
 */
const BackendSearchStrategies = {
  // Full-text search with ranking
  FULL_TEXT: 'full_text',
  // Semantic search using vector embeddings  
  SEMANTIC: 'semantic',
  // Fuzzy search for typo tolerance
  FUZZY: 'fuzzy',
  // Exact phrase matching
  EXACT: 'exact',
  // Boolean search with operators
  BOOLEAN: 'boolean',
  // Date-range aware search
  TEMPORAL: 'temporal',
  // User-scoped search
  USER_SCOPED: 'user_scoped',
  // File content search
  FILE_CONTENT: 'file_content'
};

/**
 * Search Result Types
 */
const SearchResultTypes = {
  MESSAGE: 'message',
  FILE: 'file',
  USER: 'user',
  CHAT: 'chat',
  THREAD: 'thread'
};

/**
 * Advanced Production Search Service
 */
class AdvancedSearchService {
  constructor() {
    // Cache management
    this.searchCache = new Map();
    this.suggestionCache = new Map();
    this.maxCacheSize = 200;
    this.cacheTimeout = 5 * 60 * 1000; // 5 minutes

    // Request management
    this.activeRequests = new Map();
    this.searchHistory = [];
    this.maxHistorySize = 100;

    // Performance metrics
    this.metrics = {
      searchCount: 0,
      avgResponseTime: 0,
      cacheHitRate: 0,
      errorRate: 0,
      strategiesUsed: new Map()
    };

    // Search preferences
    this.userPreferences = {
      defaultStrategy: BackendSearchStrategies.FULL_TEXT,
      enableSemantic: true,
      enableFuzzy: true,
      maxResults: 50,
      enableAutoComplete: true
    };

    if (import.meta.env.DEV) {
      console.log('🔍 AdvancedSearchService initialized');
    }
  }

  /**
   * 🚀 Advanced Multi-Strategy Search
   * Automatically selects best strategy based on query analysis
   */
  async intelligentSearch(params) {
    const {
      query,
      chatId = null,
      strategies = null,
      filters = {},
      limit = 20,
      offset = 0
    } = params;

    if (!query?.trim()) {
      return { hits: [], total: 0, strategies_used: [], took_ms: 0 };
    }

    const startTime = performance.now();
    const queryAnalysis = this.analyzeQuery(query);
    const selectedStrategies = strategies || this.selectOptimalStrategies(queryAnalysis);

    // Try strategies in priority order
    let bestResult = null;
    let strategiesUsed = [];

    for (const strategy of selectedStrategies) {
      try {
        console.log(`🎯 [AdvancedSearch] Trying strategy: ${strategy}`);

        const result = await this.executeSearchStrategy(strategy, {
          query,
          chatId,
          filters,
          limit,
          offset,
          queryAnalysis
        });

        if (result && result.hits.length > 0) {
          bestResult = result;
          strategiesUsed.push(strategy);

          // If we get good results, we can stop
          if (result.hits.length >= Math.min(limit, 10)) {
            break;
          }
        }

        strategiesUsed.push(strategy);
      } catch (error) {
        console.warn(`⚠️ Strategy ${strategy} failed:`, error.message);
        // Continue with next strategy
      }
    }

    // Fallback if all backend strategies fail
    if (!bestResult || bestResult.hits.length === 0) {
      console.log('🔄 [AdvancedSearch] Using fallback search');
      bestResult = await this.fallbackSearch(query, chatId, limit, offset);
      strategiesUsed.push('fallback_local');
    }

    const took_ms = performance.now() - startTime;
    this.updateMetrics(took_ms, strategiesUsed, bestResult?.hits?.length || 0);

    return {
      ...bestResult,
      strategies_used: strategiesUsed,
      took_ms: Math.round(took_ms),
      query_analysis: queryAnalysis
    };
  }

  /**
   * 🧠 Query Analysis for Strategy Selection
   */
  analyzeQuery(query) {
    const analysis = {
      length: query.length,
      wordCount: query.split(/\s+/).length,
      hasQuotes: /["']/.test(query),
      hasOperators: /\b(AND|OR|NOT)\b/i.test(query),
      hasDateKeywords: /\b(today|yesterday|week|month|year)\b/i.test(query),
      hasUserMention: /@\w+/.test(query),
      hasFileExtension: /\.\w{2,5}\b/.test(query),
      isQuestion: /\?$/.test(query.trim()),
      hasTypos: false, // Would require spellcheck API
      semanticComplexity: 'simple' // simple, medium, complex
    };

    // Determine semantic complexity
    if (analysis.wordCount > 5 || analysis.isQuestion) {
      analysis.semanticComplexity = analysis.wordCount > 10 ? 'complex' : 'medium';
    }

    return analysis;
  }

  /**
   * 🎯 Strategy Selection Algorithm
   */
  selectOptimalStrategies(analysis) {
    const strategies = [];

    // Primary strategy selection
    if (analysis.hasQuotes) {
      strategies.push(BackendSearchStrategies.EXACT);
    } else if (analysis.hasOperators) {
      strategies.push(BackendSearchStrategies.BOOLEAN);
    } else if (analysis.semanticComplexity === 'complex') {
      strategies.push(BackendSearchStrategies.SEMANTIC);
    } else {
      strategies.push(BackendSearchStrategies.FULL_TEXT);
    }

    // Secondary strategies
    if (analysis.hasDateKeywords) {
      strategies.push(BackendSearchStrategies.TEMPORAL);
    }

    if (analysis.hasUserMention) {
      strategies.push(BackendSearchStrategies.USER_SCOPED);
    }

    if (analysis.hasFileExtension) {
      strategies.push(BackendSearchStrategies.FILE_CONTENT);
    }

    // Fallback fuzzy search for typo tolerance
    if (this.userPreferences.enableFuzzy && analysis.length > 3) {
      strategies.push(BackendSearchStrategies.FUZZY);
    }

    return strategies.slice(0, 3); // Limit to 3 strategies max
  }

  /**
   * 🔧 Execute Specific Search Strategy
   */
  async executeSearchStrategy(strategy, params) {
    const { query, chatId, filters, limit, offset, queryAnalysis } = params;

    // Build API endpoint based on strategy
    const endpoint = this.getSearchEndpoint(strategy, chatId);
    const searchParams = this.buildSearchParams(strategy, {
      query,
      filters,
      limit,
      offset,
      queryAnalysis
    });

    const cacheKey = `${strategy}:${JSON.stringify(searchParams)}`;

    // Check cache first
    const cached = this.getFromCache(cacheKey);
    if (cached) {
      return { ...cached, from_cache: true };
    }

    try {
      // 🔧 FIX: Backend expects parameters in query string for both GET and POST
      let response;

      if (searchParams.strategy || Object.keys(searchParams).length > 3) {
        // Advanced search with strategy - use POST with params in query string
        response = await api.post(endpoint, null, { params: searchParams });
      } else {
        // Simple search - use GET with params in query string
        response = await api.get(endpoint, { params: searchParams });
      }

      if (response.data?.success) {
        const result = this.normalizeSearchResult(response.data.data, strategy);

        // Cache successful results
        this.setCache(cacheKey, result);

        return result;
      }

      throw new Error(response.data?.error?.message || 'Search failed');
    } catch (error) {
      // Enhanced error handling for different backend scenarios
      if (error.response?.status === 404) {
        console.warn(`⚠️ Strategy ${strategy} not implemented in backend`);
      } else if (error.response?.status === 500) {
        console.warn(`⚠️ Strategy ${strategy} backend error: ${error.response.data?.error || error.message}`);
      }

      throw error;
    }
  }

  /**
   * 🔗 Get Search Endpoint for Strategy - FIXED to match backend routes
   */
  getSearchEndpoint(strategy, chatId = null) {
    // 🔧 FIX: Use actual backend routes from lib.rs
    // Backend provides: /chat/{id}/messages/search and /search/messages

    if (chatId) {
      // All chat searches use the same backend endpoint with different params
      return `/chat/${chatId}/messages/search`;
    } else {
      // Global search endpoint
      return '/search/messages';
    }
  }

  /**
   * 🛠️ Build Search Parameters for Strategy - UPDATED for unified backend endpoint
   */
  buildSearchParams(strategy, { query, filters, limit, offset, queryAnalysis }) {
    const baseParams = {
      q: query,
      limit,
      offset,
      strategy: strategy, // 🔧 FIX: Pass strategy to backend
      highlight: true
    };

    // Strategy-specific parameters (backend will interpret these)
    switch (strategy) {
      case BackendSearchStrategies.SEMANTIC:
        return {
          ...baseParams,
          similarity_threshold: 0.7,
          include_context: true,
          sort: 'relevance'
        };

      case BackendSearchStrategies.FUZZY:
        return {
          ...baseParams,
          max_edits: queryAnalysis.length > 6 ? 2 : 1,
          min_similarity: 0.8,
          sort: 'relevance'
        };

      case BackendSearchStrategies.TEMPORAL:
        return {
          ...baseParams,
          time_weight: 0.3,
          sort: 'relevance_time'
        };

      case BackendSearchStrategies.USER_SCOPED:
        return {
          ...baseParams,
          include_sender: true,
          group_by_user: filters.groupByUser || false,
          sort: 'relevance'
        };

      case BackendSearchStrategies.FILE_CONTENT:
        return {
          ...baseParams,
          include_file_content: true,
          file_types: filters.fileTypes || [],
          sort: 'relevance'
        };

      case BackendSearchStrategies.EXACT:
        return {
          ...baseParams,
          exact_match: true,
          sort: 'relevance'
        };

      case BackendSearchStrategies.BOOLEAN:
        return {
          ...baseParams,
          boolean_query: true,
          sort: 'relevance'
        };

      default:
        return {
          ...baseParams,
          sort: 'relevance'
        };
    }
  }

  /**
   * 🔄 Normalize Search Results from Different Strategies
   */
  normalizeSearchResult(data, strategy) {
    return {
      hits: (data.hits || data.results || []).map(hit => ({
        id: hit.id,
        type: hit.type || SearchResultTypes.MESSAGE,
        chat_id: hit.chat_id,
        sender_id: hit.sender_id,
        sender_name: hit.sender_name,
        content: hit.content,
        content_highlighted: hit.highlighted_content || hit.content,
        created_at: hit.created_at,
        relevance_score: hit.score || hit.relevance || 1.0,
        strategy_used: strategy,
        file_info: hit.file_info,
        context: hit.context
      })),
      total: data.total || data.count || 0,
      took_ms: data.took_ms || data.execution_time || 0,
      strategy: strategy,
      has_more: data.has_more || false
    };
  }

  /**
   * Enhanced fallback search with better local strategies  
   */
  async fallbackSearch(query, chatId, limit, offset) {
    console.log(`🔄 Starting fallback search for query="${query}", chatId=${chatId}`);

    try {
      let messages = [];

      // Try multiple sources for messages

      // 1. Try UnifiedMessageService (most reliable)
      if (window.unifiedMessageService) {
        if (chatId) {
          messages = window.unifiedMessageService.getMessagesForChat(chatId) || [];
          console.log(`📋 UnifiedMessageService: ${messages.length} messages for chat ${chatId}`);
        } else {
          // Global search across all chats
          const messageCache = window.unifiedMessageService.messageCache || {};
          messages = Object.values(messageCache).flatMap(cache => cache.messages || cache).filter(msg => msg && msg.content);
          console.log(`📋 UnifiedMessageService global: ${messages.length} messages`);
        }
      }

      // 2. Try chat store if no messages from UnifiedMessageService
      if (messages.length === 0) {
        try {
          const { useChatStore } = await import('@/stores/chat');
          const chatStore = useChatStore();

          if (chatId && chatStore.currentChatId == chatId) {
            messages = chatStore.messages || [];
            console.log(`📋 Chat store: ${messages.length} messages for current chat`);
          } else if (!chatId) {
            // Global search - try to get all messages from chat store
            messages = chatStore.messages || [];
            console.log(`📋 Chat store global: ${messages.length} messages`);
          }
        } catch (storeError) {
          console.warn('❌ Chat store access failed:', storeError);
        }
      }

      // 3. Try global window.messageCache as last resort
      if (messages.length === 0 && window.messageCache) {
        if (chatId && window.messageCache[chatId]) {
          messages = window.messageCache[chatId].messages || window.messageCache[chatId] || [];
          console.log(`📋 Global cache: ${messages.length} messages for chat ${chatId}`);
        } else if (!chatId) {
          messages = Object.values(window.messageCache).flatMap(cache => cache.messages || cache).filter(msg => msg && msg.content);
          console.log(`📋 Global cache: ${messages.length} messages total`);
        }
      }

      // 4. Try importing and using the message service directly
      if (messages.length === 0) {
        try {
          const { unifiedMessageService } = await import('@/services/messageSystem/UnifiedMessageService.js');
          if (unifiedMessageService) {
            if (chatId) {
              messages = unifiedMessageService.getMessagesForChat(chatId) || [];
              console.log(`📋 Imported UnifiedMessageService: ${messages.length} messages for chat ${chatId}`);
            }
          }
        } catch (importError) {
          console.warn('❌ Failed to import UnifiedMessageService:', importError);
        }
      }

      console.log(`🔍 Total messages available for search: ${messages.length}`);

      if (messages.length === 0) {
        console.log('💡 No messages found in any source');
        return {
          hits: [],
          total: 0,
          took_ms: 10,
          fallback: true,
          sources_tried: ['UnifiedMessageService', 'chat_store', 'global_cache', 'imported_service'],
          message: 'No local messages available for search'
        };
      }

      // Enhanced local search with multiple strategies
      const lowerQuery = query.toLowerCase();
      let results = [];

      messages.forEach(msg => {
        if (!msg || !msg.content || typeof msg.content !== 'string' || !msg.id) {
          return;
        }

        const content = msg.content.toLowerCase();
        let score = 0;

        // Exact match (highest score)
        if (content.includes(lowerQuery)) {
          score += 10;
        }

        // Word boundary match
        const wordRegex = new RegExp(`\\b${lowerQuery.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')}\\b`, 'i');
        if (wordRegex.test(msg.content)) {
          score += 5;
        }

        // Partial word match
        const words = lowerQuery.split(' ');
        words.forEach(word => {
          if (word.length > 2 && content.includes(word)) {
            score += 2;
          }
        });

        if (score > 0) {
          results.push({
            ...msg,
            search_score: score
          });
        }
      });

      // Sort by score and date
      results.sort((a, b) => {
        if (a.search_score !== b.search_score) {
          return b.search_score - a.search_score;
        }
        return new Date(b.created_at) - new Date(a.created_at);
      });

      // Apply pagination
      const total = results.length;
      const hits = results.slice(offset, offset + limit).map(msg => ({
        id: msg.id,
        type: SearchResultTypes.MESSAGE,
        chat_id: msg.chat_id || chatId,
        sender_id: msg.sender_id,
        sender_name: msg.sender_name || msg.sender?.fullname || 'Unknown',
        content: msg.content,
        content_highlighted: this.highlightSearchTerm(msg.content, query),
        created_at: msg.created_at,
        relevance_score: msg.search_score || 0.5,
        strategy_used: 'fallback_local'
      }));

      console.log(`✅ Fallback search completed: ${hits.length} results out of ${total} matches`);

      return {
        hits,
        total,
        took_ms: 50,
        has_more: (offset + limit) < total,
        fallback: true,
        sources_checked: messages.length,
        message: `Found ${total} matches in ${messages.length} local messages`
      };

    } catch (error) {
      console.error('❌ Fallback search failed:', error);
      return {
        hits: [],
        total: 0,
        took_ms: 10,
        fallback: true,
        error: error.message,
        message: 'Fallback search encountered an error'
      };
    }
  }

  /**
   * 🔤 Simple Fuzzy Matching Algorithm
   */
  fuzzyMatch(str, pattern, threshold = 0.8) {
    if (str === pattern) return true;
    if (pattern.length === 0) return true;
    if (str.length === 0) return false;

    const matrix = [];

    // Initialize matrix
    for (let i = 0; i <= str.length; i++) {
      matrix[i] = [i];
    }

    for (let j = 0; j <= pattern.length; j++) {
      matrix[0][j] = j;
    }

    // Fill matrix
    for (let i = 1; i <= str.length; i++) {
      for (let j = 1; j <= pattern.length; j++) {
        if (str.charAt(i - 1) === pattern.charAt(j - 1)) {
          matrix[i][j] = matrix[i - 1][j - 1];
        } else {
          matrix[i][j] = Math.min(
            matrix[i - 1][j - 1] + 1, // substitution
            matrix[i][j - 1] + 1,     // insertion
            matrix[i - 1][j] + 1      // deletion
          );
        }
      }
    }

    const distance = matrix[str.length][pattern.length];
    const similarity = 1 - (distance / Math.max(str.length, pattern.length));

    return similarity >= threshold;
  }

  /**
   * 🎨 Enhanced Search Term Highlighting
   */
  highlightSearchTerm(content, query) {
    if (!content || !query) return content;

    try {
      // Split query into terms and create regex for each
      const terms = query.toLowerCase().split(/\s+/).filter(term => term.length > 0);
      let highlighted = content;

      terms.forEach(term => {
        const regex = new RegExp(`(${term.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')})`, 'gi');
        highlighted = highlighted.replace(regex, '<mark class="search-highlight">$1</mark>');
      });

      return highlighted;
    } catch (e) {
      return content;
    }
  }

  /**
   * 📊 Update Performance Metrics
   */
  updateMetrics(responseTime, strategies, resultCount) {
    this.metrics.searchCount++;
    this.metrics.avgResponseTime = (this.metrics.avgResponseTime * 0.9) + (responseTime * 0.1);

    strategies.forEach(strategy => {
      const count = this.metrics.strategiesUsed.get(strategy) || 0;
      this.metrics.strategiesUsed.set(strategy, count + 1);
    });
  }

  /**
   * Cache management methods
   */
  generateCacheKey(type, params) {
    return `${type}:${JSON.stringify(params)}`;
  }

  getFromCache(key) {
    const cached = this.searchCache.get(key);
    if (cached && Date.now() - cached.timestamp < this.cacheTimeout) {
      this.metrics.cacheHitRate = (this.metrics.cacheHitRate * 0.9) + (1 * 0.1);
      return cached.data;
    }
    return null;
  }

  setCache(key, data) {
    if (this.searchCache.size >= this.maxCacheSize) {
      const firstKey = this.searchCache.keys().next().value;
      this.searchCache.delete(firstKey);
    }

    this.searchCache.set(key, {
      data,
      timestamp: Date.now()
    });
  }

  /**
   * 📈 Get Performance Metrics
   */
  getMetrics() {
    return {
      ...this.metrics,
      cacheSize: this.searchCache.size,
      activeRequests: this.activeRequests.size,
      strategiesUsed: Object.fromEntries(this.metrics.strategiesUsed)
    };
  }

  /**
   * 🔍 Execute search in chat with BACKEND-FIRST strategy
   * Always search from database, not frontend cache
   */
  async searchInChat(params) {
    const { chatId, query, limit = 20, offset = 0 } = params
    const startTime = Date.now()

    if (import.meta.env.DEV) {
      console.log(`🔍 [SearchService] Starting BACKEND-FIRST search in chat ${chatId} for: "${query}"`)
    }

    // 🎯 CRITICAL FIX: Always try backend API first, never fallback to frontend cache
    try {
      // Strategy 1: Try ChatService backend API
      const ChatService = (await import('@/services/ChatService')).default

      if (import.meta.env.DEV) {
        console.log(`🎯 [SearchService] Attempting backend API search via ChatService`)
      }

      const backendResults = await ChatService.searchMessages(chatId, query, {
        limit,
        offset,
        sort: 'relevance'
      })

      if (import.meta.env.DEV) {
        console.log(`✅ [SearchService] Backend search successful: ${backendResults.hits.length} results`)
      }

      return {
        hits: backendResults.hits.map(hit => ({
          id: hit.id,
          type: SearchResultTypes.MESSAGE,
          chat_id: hit.chat_id || chatId,
          sender_id: hit.sender_id,
          sender_name: hit.sender_name || hit.sender?.fullname || 'Unknown',
          content: hit.content,
          content_highlighted: this.highlightSearchTerm(hit.content, query),
          created_at: hit.created_at,
          relevance_score: hit.relevance_score || 0.8,
          strategy_used: 'backend_api'
        })),
        total: backendResults.total,
        took_ms: Date.now() - startTime,
        has_more: (offset + limit) < backendResults.total,
        strategy: 'backend_api',
        source: 'database'
      }

    } catch (backendError) {
      if (import.meta.env.DEV) {
        console.warn(`⚠️ [SearchService] Backend API failed: ${backendError.message}`)
      }

      // Strategy 2: Try direct API call
      try {
        if (import.meta.env.DEV) {
          console.log(`🎯 [SearchService] Attempting direct API call`)
        }

        const response = await api.get(`/chat/${chatId}/messages/search`, {
          params: {
            q: query,
            limit,
            offset,
            sort: 'relevance'
          }
        })

        const apiData = response.data?.data || response.data || {}

        if (import.meta.env.DEV) {
          console.log(`✅ [SearchService] Direct API search successful: ${apiData.hits?.length || 0} results`)
        }

        return {
          hits: (apiData.hits || []).map(hit => ({
            id: hit.id,
            type: SearchResultTypes.MESSAGE,
            chat_id: hit.chat_id || chatId,
            sender_id: hit.sender_id,
            sender_name: hit.sender_name || 'Unknown',
            content: hit.content,
            content_highlighted: this.highlightSearchTerm(hit.content, query),
            created_at: hit.created_at,
            relevance_score: hit.relevance_score || 0.7,
            strategy_used: 'direct_api'
          })),
          total: apiData.total || 0,
          took_ms: Date.now() - startTime,
          has_more: (offset + limit) < (apiData.total || 0),
          strategy: 'direct_api',
          source: 'database'
        }

      } catch (directApiError) {
        if (import.meta.env.DEV) {
          console.error(`❌ [SearchService] Direct API also failed: ${directApiError.message}`)
          console.error(`❌ [SearchService] CRITICAL: Backend search completely unavailable!`)
          console.error(`❌ [SearchService] This indicates a serious backend connectivity issue`)
        }

        // 🚨 CRITICAL ERROR: Backend search completely unavailable
        // This should NOT happen in production - indicates serious backend issues
        return {
          hits: [],
          total: 0,
          took_ms: Date.now() - startTime,
          has_more: false,
          strategy: 'backend_unavailable',
          source: 'none',
          error: 'Backend search completely unavailable',
          backend_errors: [backendError.message, directApiError.message],
          message: 'Search is currently unavailable. Backend API is not responding.'
        }
      }
    }
  }

  /**
   * 🌍 Execute global search across all chats - BACKEND ONLY
   */
  async searchGlobally(params) {
    const { query, limit = 50, offset = 0 } = params
    const startTime = Date.now()

    if (import.meta.env.DEV) {
      console.log(`🌍 [SearchService] Starting GLOBAL backend search for: "${query}"`)
    }

    try {
      // Strategy 1: Try global search API
      const response = await api.post('/search/messages', {
        q: query,
        limit,
        offset,
        strategy: 'full_text'
      })

      const apiData = response.data?.data || response.data || {}

      if (import.meta.env.DEV) {
        console.log(`✅ [SearchService] Global search successful: ${apiData.hits?.length || 0} results`)
      }

      return {
        hits: (apiData.hits || []).map(hit => ({
          id: hit.id,
          type: SearchResultTypes.MESSAGE,
          chat_id: hit.chat_id,
          sender_id: hit.sender_id,
          sender_name: hit.sender_name || 'Unknown',
          content: hit.content,
          content_highlighted: this.highlightSearchTerm(hit.content, query),
          created_at: hit.created_at,
          relevance_score: hit.relevance_score || 0.8,
          strategy_used: 'global_backend'
        })),
        total: apiData.total || 0,
        took_ms: Date.now() - startTime,
        has_more: (offset + limit) < (apiData.total || 0),
        strategy: 'global_backend',
        source: 'database'
      }

    } catch (globalError) {
      if (import.meta.env.DEV) {
        console.error(`❌ [SearchService] Global search failed: ${globalError.message}`)
      }

      // 🚨 CRITICAL: Global search unavailable
      return {
        hits: [],
        total: 0,
        took_ms: Date.now() - startTime,
        has_more: false,
        strategy: 'global_backend_unavailable',
        source: 'none',
        error: 'Global search unavailable',
        backend_error: globalError.message,
        message: 'Global search is currently unavailable. Please try again later.'
      }
    }
  }
}

// Export singleton instance
export default new AdvancedSearchService(); 