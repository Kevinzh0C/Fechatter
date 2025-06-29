/**
 * Search System Debugger
 * 调试和测试搜索系统的综合工具
 */

import searchService from '@/services/searchService.js';

class SearchDebugger {
  constructor() {
    this.debugResults = [];
    this.isDebugging = false;
  }

  /**
   * 启动完整的搜索系统诊断
   */
  async runFullDiagnosis(chatId = 3, query = 'Hi') {
    console.log('=== Search System Full Diagnosis ===');
    console.log(`📋 Testing Chat ID: ${chatId}, Query: "${query}"`);

    this.debugResults = [];
    this.isDebugging = true;

    try {
      // 1. Test backend API availability
      await this.testBackendAPIs(chatId, query);

      // 2. Test frontend search service
      await this.testSearchService(chatId, query);

      // 3. Test fallback mechanisms
      await this.testFallbackSearch(chatId, query);

      // 4. Display diagnosis summary
      this.printDiagnosisSummary();

    } catch (error) {
      console.error('🚨 Diagnosis failed:', error);
    } finally {
      this.isDebugging = false;
    }

    return this.debugResults;
  }

  /**
   * 测试后端API可用性
   */
  async testBackendAPIs(chatId, query) {
    console.log('\nSUBSCRIPTION: Testing Backend APIs...');

    const endpoints = [
      {
        name: 'Chat Search',
        url: `/chat/${chatId}/messages/search?q=${encodeURIComponent(query)}&limit=5`,
        critical: true
      },
      {
        name: 'Search Suggestions',
        url: `/search/suggestions?q=${encodeURIComponent(query)}&limit=5`,
        critical: false
      },
      {
        name: 'Global Search',
        url: `/search/messages?q=${encodeURIComponent(query)}&limit=5`,
        critical: false
      },
      {
        name: 'Chat List Search',
        url: `/workspace/chats/search?q=${encodeURIComponent(query)}&limit=5`,
        critical: false
      }
    ];

    for (const endpoint of endpoints) {
      try {
        const response = await fetch(`/api${endpoint.url}`, {
          method: 'GET',
          headers: {
            'Content-Type': 'application/json',
            'Authorization': `Bearer ${this.getAuthToken()}`
          }
        });

        const status = response.status;
        const statusText = response.statusText;

        if (response.ok) {
          try {
            const data = await response.json();
            this.addDebugResult('API', endpoint.name, 'SUCCESS',
              `${status} - Returned data structure: ${JSON.stringify(Object.keys(data)).substring(0, 100)}`);
          } catch (parseError) {
            this.addDebugResult('API', endpoint.name, 'SUCCESS',
              `${status} - Response received but not JSON`);
          }
        } else {
          const errorText = await response.text();
          const severity = endpoint.critical ? 'CRITICAL' : 'WARNING';
          this.addDebugResult('API', endpoint.name, severity,
            `${status} ${statusText} - ${errorText.substring(0, 200)}`);
        }
      } catch (error) {
        const severity = endpoint.critical ? 'CRITICAL' : 'WARNING';
        this.addDebugResult('API', endpoint.name, severity,
          `Network error: ${error.message}`);
      }
    }
  }

  /**
   * 测试SearchService
   */
  async testSearchService(chatId, query) {
    console.log('\nTesting SearchService...');

    try {
      // Test chat search
      const chatSearchResult = await searchService.searchInChat({
        chatId,
        query,
        limit: 5
      });

      if (chatSearchResult && typeof chatSearchResult === 'object') {
        const hitCount = chatSearchResult.hits?.length || 0;
        const isFallback = chatSearchResult.fallback ? ' (FALLBACK)' : '';
        this.addDebugResult('Service', 'Chat Search', 'SUCCESS',
          `Found ${hitCount} results in ${chatSearchResult.took_ms}ms${isFallback}`);
      } else {
        this.addDebugResult('Service', 'Chat Search', 'ERROR',
          'Invalid result format');
      }
    } catch (error) {
      this.addDebugResult('Service', 'Chat Search', 'ERROR',
        `Exception: ${error.message}`);
    }

    try {
      // Test suggestions
      const suggestions = await searchService.getSearchSuggestions(query, 3);
      this.addDebugResult('Service', 'Suggestions', 'SUCCESS',
        `Generated ${suggestions.length} suggestions`);
    } catch (error) {
      this.addDebugResult('Service', 'Suggestions', 'ERROR',
        `Exception: ${error.message}`);
    }
  }

  /**
   * 测试备用搜索机制
   */
  async testFallbackSearch(chatId, query) {
    console.log('\n🔄 Testing Fallback Mechanisms...');

    try {
      const fallbackResult = await searchService.fallbackChatSearch(chatId, query, 5, 0);

      if (fallbackResult) {
        const hitCount = fallbackResult.hits?.length || 0;
        this.addDebugResult('Fallback', 'Chat Search', 'SUCCESS',
          `Found ${hitCount} results from local messages`);
      } else {
        this.addDebugResult('Fallback', 'Chat Search', 'WARNING',
          'No fallback results available');
      }
    } catch (error) {
      this.addDebugResult('Fallback', 'Chat Search', 'ERROR',
        `Exception: ${error.message}`);
    }

    try {
      const fallbackSuggestions = searchService.fallbackSuggestions(query);
      this.addDebugResult('Fallback', 'Suggestions', 'SUCCESS',
        `Generated ${fallbackSuggestions.length} fallback suggestions`);
    } catch (error) {
      this.addDebugResult('Fallback', 'Suggestions', 'ERROR',
        `Exception: ${error.message}`);
    }
  }

  /**
   * 获取认证令牌
   */
  getAuthToken() {
    return localStorage.getItem('auth_token') ||
      window.tokenManager?.getAccessToken() ||
      '';
  }

  /**
   * 添加调试结果
   */
  addDebugResult(category, test, status, details) {
    const result = {
      category,
      test,
      status,
      details,
      timestamp: new Date().toISOString()
    };

    this.debugResults.push(result);

    const statusEmojis = {
      'SUCCESS': '',
      'WARNING': '⚠️',
      'ERROR': '❌',
      'CRITICAL': '🚨'
    };

    console.log(`${statusEmojis[status]} [${category}] ${test}: ${details}`);
  }

  /**
   * 打印诊断摘要
   */
  printDiagnosisSummary() {
    console.log('\n=== Search System Diagnosis Summary ===');

    const stats = {
      SUCCESS: 0,
      WARNING: 0,
      ERROR: 0,
      CRITICAL: 0
    };

    this.debugResults.forEach(result => {
      stats[result.status]++;
    });

    console.log(`Success: ${stats.SUCCESS}`);
    console.log(`WARNING: Warning: ${stats.WARNING}`);
    console.log(`ERROR: Error: ${stats.ERROR}`);
    console.log(`🚨 Critical: ${stats.CRITICAL}`);
    console.log(`📋 Total: ${this.debugResults.length}`);

    // 系统状态评估
    if (stats.CRITICAL > 0) {
      console.log('\n🚨 SYSTEM STATUS: CRITICAL - Essential APIs are failing');
      console.log('RECOMMENDATION: Use fallback search mode only');
    } else if (stats.ERROR > 0) {
      console.log('\nWARNING: SYSTEM STATUS: DEGRADED - Some features unavailable');
      console.log('RECOMMENDATION: Fallback mechanisms will be used');
    } else if (stats.WARNING > 0) {
      console.log('\nSYSTEM STATUS: FUNCTIONAL - Minor issues present');
      console.log('RECOMMENDATION: Search system is usable');
    } else {
      console.log('\n🎉 SYSTEM STATUS: FULLY OPERATIONAL');
      console.log('RECOMMENDATION: All search features available');
    }

    // 用户建议
    console.log('\nUSER GUIDANCE:');
    if (stats.SUCCESS > 0) {
      console.log('- Search functionality is available');
      console.log('- Results will be displayed when found');
    }
    if (stats.WARNING > 0 || stats.ERROR > 0) {
      console.log('- Some advanced features may be limited');
      console.log('- Basic text search will work via fallback');
    }
    if (stats.CRITICAL > 0) {
      console.log('- Contact system administrator for backend issues');
    }
  }

  /**
   * 快速健康检查
   */
  async quickHealthCheck() {
    console.log('🩺 Quick Search Health Check...');

    try {
      const result = await searchService.searchInChat({
        chatId: 3,
        query: 'test',
        limit: 1
      });

      if (result && result.hits !== undefined) {
        console.log('Search system is responsive');
        return true;
      } else {
        console.log('WARNING: Search system returned unexpected format');
        return false;
      }
    } catch (error) {
      console.log('ERROR: Search system is not responding:', error.message);
      return false;
    }
  }
}

// 导出单例
export default new SearchDebugger();

// 全局访问
if (typeof window !== 'undefined') {
  window.searchDebugger = new SearchDebugger();
} 