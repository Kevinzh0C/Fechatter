/**
 * Debug Search Response
 * 
 * Tool to intercept and display raw search API responses
 */

import api from '../services/api';

class SearchResponseDebugger {
  constructor() {
    this.enabled = true;
    this.responses = [];
  }

  /**
   * Enable response debugging
   */
  enable() {
    this.enabled = true;
    if (import.meta.env.DEV) {
      console.log('[SearchDebug] Response debugging enabled');
    }

  /**
   * Disable response debugging
   */
  disable() {
    this.enabled = false;
    if (import.meta.env.DEV) {
      console.log('[SearchDebug] Response debugging disabled');
    }

  /**
   * Intercept and log search response
   */
  interceptSearchResponse() {
    if (!this.enabled) return;

    // Save original post method
    const originalPost = api.post.bind(api);

    // Override post method
    api.post = async (url, data, config) => {
      // Check if this is a search request
      if (url.includes('search')) {
        if (import.meta.env.DEV) {
          console.log('[SearchDebug] Intercepting search request:', {
          url,
          data,
          config
        });

        try {
          // Make actual request
          const response = await originalPost(url, data, config);

          // Log raw response
          if (import.meta.env.DEV) {
            console.log('[SearchDebug] RAW RESPONSE:', {
        status: response.status,
            statusText: response.statusText,
            headers: response.headers,
            data: response.data,
            config: response.config
          });

          // Store response
          this.responses.push({
            timestamp: new Date().toISOString(),
            url,
            request: data,
            response: {
              status: response.status,
              data: response.data
            }
          });

          // Also log formatted response
          if (response.data) {
            if (import.meta.env.DEV) {
              console.log('[SearchDebug] Response Data Structure:', {
        hasData: !!response.data,
              dataType: typeof response.data,
              dataKeys: Object.keys(response.data || {}),
              isArray: Array.isArray(response.data),
              dataContent: response.data
            });

          return response;
        } catch (error) {
          if (import.meta.env.DEV) {
            console.error('[SearchDebug] Request failed:', {
            url,
            error: error.message,
            response: error.response?.data,
            status: error.response?.status
          });
          throw error;
        }

      // Non-search requests pass through
      return originalPost(url, data, config);
    };

    if (import.meta.env.DEV) {
      console.log('[SearchDebug] Interception activated');
    }

  /**
   * Get last response
   */
  getLastResponse() {
    return this.responses[this.responses.length - 1] || null;
  }

  /**
   * Clear stored responses
   */
  clearResponses() {
    this.responses = [];
    if (import.meta.env.DEV) {
      console.log('[SearchDebug] Responses cleared');
    }

  /**
   * Test search directly
   */
  async testSearchDirectly() {
    if (import.meta.env.DEV) {
      console.log('[SearchDebug] Testing search API directly...');
    }

    try {
      const testPayload = {
        q: 'test',
        limit: 5
      };

      if (import.meta.env.DEV) {
        console.log('[SearchDebug] Request payload:', testPayload);
      }

      const response = await api.post('/search/messages', testPayload);

      if (import.meta.env.DEV) {
        console.log('[SearchDebug] Direct test response:', {
        status: response.status,
        data: response.data,
        dataType: typeof response.data,
        dataKeys: Object.keys(response.data || {})
      });

      return response.data;
    } catch (error) {
      if (import.meta.env.DEV) {
        console.error('[SearchDebug] Direct test failed:', {
        message: error.message,
        response: error.response?.data,
        status: error.response?.status
      });
      throw error;
    }

  /**
   * Compare expected vs actual response format
   */
  analyzeResponseFormat(actualResponse) {
    if (import.meta.env.DEV) {
      console.log('[SearchDebug] Analyzing response format...');
    }

    const expectedFormats = [
      {
        name: 'Standard Backend Format',
        check: (data) => data.success && data.data,
        extract: (data) => ({ results: data.data.hits || data.data.results })
      },
      {
        name: 'Direct Hits Format',
        check: (data) => data.hits || data.results,
        extract: (data) => ({ results: data.hits || data.results })
      },
      {
        name: 'Meilisearch Format',
        check: (data) => data.hits && data.query !== undefined,
        extract: (data) => ({ results: data.hits })
      },
      {
        name: 'Array Format',
        check: (data) => Array.isArray(data),
        extract: (data) => ({ results: data })
      }
    ];

    for (const format of expectedFormats) {
      if (format.check(actualResponse)) {
        if (import.meta.env.DEV) {
          console.log(`[SearchDebug] Matches format: ${format.name}`);
        return format.extract(actualResponse);
      }

    if (import.meta.env.DEV) {
      console.error('ERROR: [SearchDebug] No matching format found!');
    if (import.meta.env.DEV) {
      console.log('[SearchDebug] Actual response structure:', {
        type: typeof actualResponse,
      keys: Object.keys(actualResponse || {}),
      sample: JSON.stringify(actualResponse).substring(0, 200) + '...'
    });

    return null;
  }

// Create global instance
const searchDebugger = new SearchResponseDebugger();

// Auto-enable in development
if (import.meta.env.DEV) {
  searchDebugger.interceptSearchResponse();

  // Add to window for easy access
  window.searchDebug = {
    enable: () => searchDebugger.enable(),
    disable: () => searchDebugger.disable(),
    test: () => searchDebugger.testSearchDirectly(),
    lastResponse: () => searchDebugger.getLastResponse(),
    clear: () => searchDebugger.clearResponses(),
    analyze: (data) => searchDebugger.analyzeResponseFormat(data)
  };

  if (import.meta.env.DEV) {
    console.log('[SearchDebug] Available commands:');
  if (import.meta.env.DEV) {
    console.log('   window.searchDebug.test() - Test search API directly');
  if (import.meta.env.DEV) {
    console.log('   window.searchDebug.lastResponse() - Get last response');
  if (import.meta.env.DEV) {
    console.log('   window.searchDebug.analyze(data) - Analyze response format');
  }

export default searchDebugger; 