/**
 * 搜索功能诊断工具
 * 用于实时检测和修复搜索功能问题
 */

import { SearchService } from '@/services/api';
import tokenManager from '@/services/tokenManager';

export class SearchDiagnostic {
  constructor() {
    this.results = {};
  }

  /**
   * 执行完整的搜索功能诊断
   */
  async runFullDiagnostic() {
    if (import.meta.env.DEV) {
      console.log('🔍 [SearchDiagnostic] Starting comprehensive search diagnostic...');
    }
    
    const results = {
      timestamp: new Date().toISOString(),
      tests: {},
      summary: {
        passed: 0,
        failed: 0,
        warnings: 0
      }
    };

    // 1. 认证状态检查
    results.tests.authentication = await this.testAuthentication();
    
    // 2. Token有效性验证
    results.tests.tokenValidation = await this.testTokenValidation();
    
    // 3. API连通性测试
    results.tests.apiConnectivity = await this.testApiConnectivity();
    
    // 4. 搜索功能测试
    results.tests.searchFunctionality = await this.testSearchFunctionality();
    
    // 5. 错误处理测试
    results.tests.errorHandling = await this.testErrorHandling();

    // 计算摘要
    Object.values(results.tests).forEach(test => {
      if (test.status === 'PASS') results.summary.passed++;
      else if (test.status === 'FAIL') results.summary.failed++;
      else if (test.status === 'WARN') results.summary.warnings++;
    });

    this.results = results;
    this.printDiagnosticReport();
    
    return results;
  }

  /**
   * 测试认证状态
   */
  async testAuthentication() {
    try {
      const token = tokenManager.getAccessToken();
      const refreshToken = tokenManager.getRefreshToken();
      const tokenStatus = tokenManager.getStatus();
      
      if (!token) {
        return {
          status: 'FAIL',
          message: 'No access token available',
          details: { tokenStatus }
        };
      }

      return {
        status: 'PASS',
        message: 'Authentication tokens available',
        details: {
          hasAccessToken: !!token,
          hasRefreshToken: !!refreshToken,
          tokenPreview: token.substring(0, 20) + '...',
          tokenStatus
        }
      };
    } catch (error) {
      return {
        status: 'FAIL',
        message: 'Authentication check failed',
        error: error.message
      };
    }

  /**
   * 测试Token有效性
   */
  async testTokenValidation() {
    try {
      const token = tokenManager.getAccessToken();
      
      if (!token) {
        return {
          status: 'FAIL',
          message: 'No token to validate'
        };
      }

      // 解析JWT payload
      const payload = JSON.parse(atob(token.split('.')[1]));
      const currentTime = Math.floor(Date.now() / 1000);
      const isExpired = payload.exp <= currentTime;
      const timeUntilExpiry = payload.exp - currentTime;

      if (isExpired) {
        return {
          status: 'FAIL',
          message: 'Token has expired',
          details: {
            expiredAt: new Date(payload.exp * 1000).toISOString(),
            expiredSecondsAgo: Math.abs(timeUntilExpiry)
          }
        };
      }

      // 警告即将过期的token
      if (timeUntilExpiry < 300) { // 5分钟内过期
        return {
          status: 'WARN',
          message: 'Token expires soon',
          details: {
            expiresAt: new Date(payload.exp * 1000).toISOString(),
            secondsUntilExpiry: timeUntilExpiry
          }
        };
      }

      return {
        status: 'PASS',
        message: 'Token is valid and not expired',
        details: {
          expiresAt: new Date(payload.exp * 1000).toISOString(),
          secondsUntilExpiry: timeUntilExpiry,
          userId: payload.sub || payload.user_id
        }
      };
    } catch (error) {
      return {
        status: 'FAIL',
        message: 'Token validation failed',
        error: error.message
      };
    }

  /**
   * 测试API连通性
   */
  async testApiConnectivity() {
    try {
      // 测试基础API连通性（可以用健康检查端点）
      const response = await fetch('/api/health', {
        method: 'GET',
        headers: {
          'Authorization': `Bearer ${tokenManager.getAccessToken()}`
        }
      });

      if (response.ok) {
        return {
          status: 'PASS',
          message: 'API connectivity successful',
          details: {
            status: response.status,
            statusText: response.statusText
          }
        };
      } else {
        return {
          status: 'FAIL',
          message: 'API connectivity failed',
          details: {
            status: response.status,
            statusText: response.statusText
          }
        };
      }
    } catch (error) {
      return {
        status: 'FAIL',
        message: 'API connectivity test failed',
        error: error.message
      };
    }

  /**
   * 测试搜索功能
   */
  async testSearchFunctionality() {
    try {
      // 测试简单搜索
      const testParams = {
        query: 'test',
        limit: 1
      };

      const searchResponse = await SearchService.search(testParams);
      
      return {
        status: 'PASS',
        message: 'Search functionality working',
        details: {
          resultsCount: searchResponse.results?.length || 0,
          totalResults: searchResponse.total || 0,
          responseTime: searchResponse.took_ms || 0
        }
      };
    } catch (error) {
      // 分析错误类型
      if (error.message?.includes('Authentication failed') || error.response?.status === 401) {
        return {
          status: 'FAIL',
          message: 'Search failed due to authentication error',
          error: error.message,
          suggestion: 'Try logging out and logging back in'
        };
      } else if (error.response?.status === 403) {
        return {
          status: 'FAIL',
          message: 'Search failed due to permission error',
          error: error.message
        };
      } else {
        return {
          status: 'FAIL',
          message: 'Search functionality failed',
          error: error.message,
          details: {
            status: error.response?.status,
            data: error.response?.data
          }
        };
      }

  /**
   * 测试错误处理
   */
  async testErrorHandling() {
    try {
      // 测试无效搜索参数
      const invalidParams = {
        query: '', // 空查询
        limit: 0   // 无效限制
      };

      await SearchService.search(invalidParams);
      
      return {
        status: 'WARN',
        message: 'Error handling not triggered (unexpected)',
        details: { note: 'Invalid parameters were accepted' }
      };
    } catch (error) {
      // 期望的错误
      return {
        status: 'PASS',
        message: 'Error handling working correctly',
        details: {
          errorType: error.constructor.name,
          errorMessage: error.message
        }
      };
    }

  /**
   * 自动修复常见问题
   */
  async attemptAutoFix() {
    if (import.meta.env.DEV) {
      console.log('🔧 [SearchDiagnostic] Attempting automatic fixes...');
    }
    
    const fixes = [];

    // 1. 尝试刷新token
    try {
      if (tokenManager.shouldRefreshToken()) {
        await tokenManager.refreshToken();
        fixes.push('Token refreshed successfully');
      }
    } catch (error) {
      fixes.push(`Token refresh failed: ${error.message}`);
    }

    // 2. 检查并同步认证状态
    try {
      const authStore = window.authStore || await import('@/stores/auth').then(m => m.useAuthStore());
      if (authStore?.ensureAuthStateConsistency) {
        await authStore.ensureAuthStateConsistency();
        fixes.push('Auth state synchronized');
      }
    } catch (error) {
      fixes.push(`Auth sync failed: ${error.message}`);
    }

    return fixes;
  }

  /**
   * 打印诊断报告
   */
  printDiagnosticReport() {
    const { results } = this;
    
    if (import.meta.env.DEV) {
      console.log('\n🔍 ========== SEARCH DIAGNOSTIC REPORT ==========');
    if (import.meta.env.DEV) {
      console.log(`⏰ Timestamp: ${results.timestamp}`);
    if (import.meta.env.DEV) {
      console.log(`✅ Passed: ${results.summary.passed}`);
    if (import.meta.env.DEV) {
      console.log(`❌ Failed: ${results.summary.failed}`);
    if (import.meta.env.DEV) {
      console.log(`⚠️  Warnings: ${results.summary.warnings}`);
    if (import.meta.env.DEV) {
      console.log('\n📋 Test Results:');
    }
    
    Object.entries(results.tests).forEach(([testName, result]) => {
      const icon = result.status === 'PASS' ? '✅' : result.status === 'FAIL' ? '❌' : '⚠️';
      if (import.meta.env.DEV) {
        console.log(`${icon} ${testName}: ${result.message}`);
      }
      
      if (result.details) {
        if (import.meta.env.DEV) {
          console.log('   Details:', result.details);
        }
      
      if (result.error) {
        if (import.meta.env.DEV) {
          console.log('   Error:', result.error);
        }
      
      if (result.suggestion) {
        if (import.meta.env.DEV) {
          console.log('   💡 Suggestion:', result.suggestion);
        }
    });
    
    if (import.meta.env.DEV) {
      console.log('🔍 ============================================\n');
    }

  /**
   * 获取修复建议
   */
  getFixSuggestions() {
    const suggestions = [];
    
    Object.values(this.results.tests || {}).forEach(test => {
      if (test.suggestion) {
        suggestions.push(test.suggestion);
      }
    });

    // 通用建议
    if (this.results.summary?.failed > 0) {
      suggestions.push('Run window.searchDiagnostic.attemptAutoFix() to try automatic fixes');
      suggestions.push('Check browser console for detailed error logs');
      suggestions.push('Verify network connectivity and server status');
    }

    return suggestions;
  }

// 创建全局实例
const searchDiagnostic = new SearchDiagnostic();

// 暴露到window对象用于调试
if (typeof window !== 'undefined') {
  window.searchDiagnostic = searchDiagnostic;
  
  // 便捷方法
  window.diagnoseSeart = () => searchDiagnostic.runFullDiagnostic();
  window.fixSearch = () => searchDiagnostic.attemptAutoFix();
  
  if (import.meta.env.DEV) {
    console.log('🔍 Search diagnostic tools available:');
  if (import.meta.env.DEV) {
    console.log('   window.searchDiagnostic.runFullDiagnostic() - Full diagnostic');
  if (import.meta.env.DEV) {
    console.log('   window.diagnoseSeart() - Quick diagnostic');
  if (import.meta.env.DEV) {
    console.log('   window.fixSearch() - Attempt auto-fix');
  }

export default searchDiagnostic;