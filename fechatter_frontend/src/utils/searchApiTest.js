/**
 * Search API Test Utility
 * 测试搜索API是否正常工作
 */

import api from '@/services/api.js';
import searchService from '@/services/searchService.js';

class SearchApiTest {
  constructor() {
    this.testResults = [];
  }

  /**
   * 执行完整的搜索API测试
   */
  async runCompleteTest(chatId = 1, query = 'hi') {
    console.log('🧪 Starting Search API Tests...');
    console.log(`📋 Testing with Chat ID: ${chatId}, Query: "${query}"`);

    this.testResults = [];

    // 测试1: 基础API连接
    await this.testBasicApiConnection();

    // 测试2: 直接API调用
    await this.testDirectApiCall(chatId, query);

    // 测试3: SearchService调用
    await this.testSearchService(chatId, query);

    // 测试4: 认证状态检查
    await this.testAuthStatus();

    // 输出结果
    this.printResults();
    return this.testResults;
  }

  /**
   * 测试基础API连接
   */
  async testBasicApiConnection() {
    try {
      console.log('🔗 Testing basic API connection...');

      const response = await fetch('/api/health', {
        method: 'GET',
        headers: {
          'Content-Type': 'application/json'
        }
      });

      if (response.ok) {
        this.addResult('Basic API Connection', 'PASS', 'API server is responding');
      } else {
        this.addResult('Basic API Connection', 'FAIL', `Status: ${response.status}`);
      }
    } catch (error) {
      this.addResult('Basic API Connection', 'ERROR', error.message);
    }
  }

  /**
   * 测试直接API调用
   */
  async testDirectApiCall(chatId, query) {
    try {
      console.log('SUBSCRIPTION: Testing direct API call...');

      const token = localStorage.getItem('auth_token') || window.tokenManager?.getAccessToken();

      const response = await fetch(`/api/chat/${chatId}/messages/search?q=${encodeURIComponent(query)}&limit=5`, {
        method: 'GET',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': token ? `Bearer ${token}` : '',
          'Accept': 'application/json'
        }
      });

      console.log('SUBSCRIPTION: Direct API Response:', {
        status: response.status,
        statusText: response.statusText,
        headers: Object.fromEntries(response.headers.entries())
      });

      if (response.ok) {
        const data = await response.json();
        console.log('SUBSCRIPTION: Direct API Data:', data);
        this.addResult('Direct API Call', 'PASS', `Returned ${data.data?.hits?.length || 0} results`);
      } else {
        const errorText = await response.text();
        this.addResult('Direct API Call', 'FAIL', `Status: ${response.status}, Body: ${errorText}`);
      }
    } catch (error) {
      this.addResult('Direct API Call', 'ERROR', error.message);
    }
  }

  /**
   * 测试SearchService
   */
  async testSearchService(chatId, query) {
    try {
      console.log('Testing SearchService...');

      const result = await searchService.searchInChat({
        chatId,
        query,
        limit: 5
      });

      console.log('SearchService Result:', result);

      if (result && typeof result === 'object') {
        const hitCount = result.hits?.length || 0;
        this.addResult('SearchService', 'PASS', `Returned ${hitCount} hits, took ${result.took_ms}ms`);
      } else {
        this.addResult('SearchService', 'FAIL', 'Invalid result format');
      }
    } catch (error) {
      this.addResult('SearchService', 'ERROR', error.message);
    }
  }

  /**
   * 测试认证状态
   */
  async testAuthStatus() {
    try {
      console.log('🔐 Testing auth status...');

      const token = localStorage.getItem('auth_token') || window.tokenManager?.getAccessToken();

      if (token) {
        // 验证token有效性
        const response = await api.get('/user/me');
        if (response.data.success) {
          this.addResult('Auth Status', 'PASS', `Authenticated as user ${response.data.data.id}`);
        } else {
          this.addResult('Auth Status', 'FAIL', 'Token invalid');
        }
      } else {
        this.addResult('Auth Status', 'FAIL', 'No auth token found');
      }
    } catch (error) {
      this.addResult('Auth Status', 'ERROR', error.message);
    }
  }

  /**
   * 添加测试结果
   */
  addResult(test, status, details) {
    const result = {
      test,
      status,
      details,
      timestamp: new Date().toISOString()
    };
    this.testResults.push(result);

    const statusEmoji = {
      'PASS': '',
      'FAIL': '❌',
      'ERROR': '🚨'
    };

    console.log(`${statusEmoji[status]} [${test}] ${status}: ${details}`);
  }

  /**
   * 打印测试结果摘要
   */
  printResults() {
    console.log('\nSearch API Test Results Summary:');
    console.log('=====================================');

    const passed = this.testResults.filter(r => r.status === 'PASS').length;
    const failed = this.testResults.filter(r => r.status === 'FAIL').length;
    const errors = this.testResults.filter(r => r.status === 'ERROR').length;

    console.log(`Passed: ${passed}`);
    console.log(`ERROR: Failed: ${failed}`);
    console.log(`🚨 Errors: ${errors}`);
    console.log(`📋 Total: ${this.testResults.length}`);

    if (failed === 0 && errors === 0) {
      console.log('\n🎉 All tests passed! Search functionality should work correctly.');
    } else {
      console.log('\nWARNING: Some tests failed. Check the details above.');
    }
  }

  /**
   * 快速诊断搜索问题
   */
  async quickDiagnose(chatId, query) {
    console.log('🚨 Quick Search Diagnosis');
    console.log('=========================');

    // 检查1: 认证
    const token = localStorage.getItem('auth_token') || window.tokenManager?.getAccessToken();
    console.log('🔐 Auth Token:', token ? 'Present' : 'ERROR: Missing');

    // 检查2: Chat ID
    console.log('MESSAGE: Chat ID:', chatId ? `${chatId}` : 'ERROR: Missing');

    // 检查3: Query
    console.log('Query:', query ? `"${query}"` : 'ERROR: Empty');

    // 检查4: SearchService
    try {
      console.log('SearchService:', searchService ? 'Available' : 'ERROR: Not found');
    } catch (error) {
      console.log('SearchService: ERROR: Error -', error.message);
    }

    // 检查5: API服务
    try {
      console.log('SUBSCRIPTION: API Service:', api ? 'Available' : 'ERROR: Not found');
    } catch (error) {
      console.log('SUBSCRIPTION: API Service: ERROR: Error -', error.message);
    }
  }
}

// 导出单例实例
export default new SearchApiTest();

// 全局方法，便于在控制台中使用
window.searchApiTest = new SearchApiTest(); 