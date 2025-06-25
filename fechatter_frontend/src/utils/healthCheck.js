// Health Check System - 功能健康检查系统
import api from '../services/api';
import { useAuthStore } from '../stores/auth';
import { useChatStore } from '../stores/chat';
import { useWorkspaceStore } from '../stores/workspace';
import errorMonitor from './errorMonitor';

class HealthCheckSystem {
  constructor() {
    this.checks = new Map();
    this.results = new Map();
    this.isRunning = false;
    this.autoCheckInterval = null;

    // 注册核心检查项
    this.registerCoreChecks();
  }

  registerCoreChecks() {
    // API连接检查 - 开发环境优化版本
    this.registerCheck('api_connection', {
      name: 'API Connection',
      critical: false, // 在开发环境中不设为关键
      async check() {
        try {
          // 使用配置系统获取Gateway URL
          const { getApiConfig } = await import('./configLoader.js');
          const apiConfig = getApiConfig();

          // 通过Gateway访问健康检查端点
          const gatewayUrl = apiConfig.gateway_url || 'http://127.0.0.1:8080';
          const healthURL = `${gatewayUrl}/health`;

          // 开发环境：增加超时时间和重试机制
          const isDev = import.meta.env.DEV;
          const timeout = isDev ? 10000 : 5000; // 开发环境增加到10秒

          const response = await fetch(healthURL, {
            method: 'GET',
            headers: {
              'Content-Type': 'application/json'
            },
            signal: AbortSignal.timeout(timeout)
          });

          if (response.ok) {
            let healthData;
            try {
              healthData = await response.json();
            } catch {
              healthData = { status: 'ok' };
            }

            return {
              success: true, // 统一使用 success 字段
              healthy: true,
              details: {
                status: response.status,
                endpoint: healthURL,
                data: healthData,
                message: 'Gateway health check passed'
              }
            };
          } else {
            return {
              success: false,
              healthy: false,
              details: {
                status: response.status,
                endpoint: healthURL,
                message: `Gateway returned ${response.status}: ${response.statusText}`
              },
              warning: 'Gateway health check failed'
            };
          }
        } catch (error) {
          // 在开发环境中提供更友好的错误信息
          const isNetworkError = error.name === 'TypeError' && error.message.includes('fetch');
          const isCorsError = error.message.includes('CORS');
          const isTimeoutError = error.name === 'TimeoutError' || error.message.includes('timeout');
          const isDev = import.meta.env.DEV;

          // 开发环境下的网络错误不应该被记录为严重错误
          const shouldLogError = !isDev || (!isNetworkError && !isTimeoutError);

          return {
            success: false,
            healthy: false,
            details: {
              error: error.message,
              type: isNetworkError ? 'network' : isCorsError ? 'cors' : isTimeoutError ? 'timeout' : 'unknown',
              suggestion: isNetworkError
                ? 'Check if Gateway is running on port 8080'
                : isCorsError
                  ? 'CORS configuration issue'
                  : isTimeoutError
                    ? 'Gateway response timeout - backend may be starting'
                    : 'Unknown connection issue',
              shouldLogError // 标记是否应该记录错误
            },
            warning: isDev
              ? 'Backend services not started - this is normal in development'
              : 'Gateway connection failed'
          };
        }
      }
    });

    // 认证状态检查
    this.registerCheck('auth_status', {
      name: 'Authentication Status',
      critical: false,
      check() {
        try {
          // 检查当前路径，如果在登录页面或公开页面，认证不是必需的
          const currentPath = window.$router?.currentRoute?.value?.path || window.location.pathname;
          const publicPaths = ['/', '/login', '/register', '/auth', '/welcome'];
          const isPublicPath = publicPaths.some(path => currentPath.startsWith(path));

          const authStore = useAuthStore();
          const hasToken = !!authStore.token;
          const hasUser = !!authStore.user;
          const isAuthenticated = hasToken && hasUser;

          // 如果在公开页面，认证状态不是必需的
          if (isPublicPath) {
            return {
              success: true,
              details: {
                hasToken,
                hasUser,
                userId: authStore.user?.id,
                tokenLength: authStore.token?.length || 0,
                currentPath,
                isPublicPath: true,
                message: 'Authentication not required on public pages'
              }
            };
          }

          // 在私有页面，检查认证状态
          return {
            success: isAuthenticated,
            details: {
              hasToken,
              hasUser,
              userId: authStore.user?.id,
              tokenLength: authStore.token?.length || 0,
              currentPath,
              isPublicPath: false,
              message: isAuthenticated ? 'Authenticated' : 'Authentication required for this page'
            },
            // 即使失败也不抛出错误，只是标记状态
            warning: !isAuthenticated ? 'User not authenticated - login required' : null
          };
        } catch (error) {
          return {
            success: false,
            error: `Auth check failed: ${error.message}`,
            details: {
              error: error.message,
              isSystemError: true
            }
          };
        }
      }
    });

    // Store functionality check - Fixed for Vue context issues
    this.registerCheck('stores_functional', {
      name: 'Stores Functionality',
      critical: true,
      check() {
        try {
          // Ensure we're in Vue app context
          if (typeof window === 'undefined' || !window.app) {
            return {
              success: false,
              error: 'Vue app not available',
              details: { context: 'No window.app found' }
            };
          }

          // Check if Pinia is available
          if (!window.pinia) {
            return {
              success: false,
              error: 'Pinia not available',
              details: { context: 'No window.pinia found' }
            };
          }

          // Try to access stores within Vue app context
          let chatStore, workspaceStore;

          try {
            // Use the app's provide/inject system to get Pinia
            const piniaSymbol = Symbol.for('pinia');
            const pinia = window.app._instance?.appContext?.provides?.[piniaSymbol] || window.pinia;

            if (!pinia) {
              return {
                success: false,
                error: 'Pinia instance not found in app context',
                details: { context: 'Cannot access Pinia from app context' }
              };
            }

            // Import stores dynamically to avoid circular dependencies
            const { useChatStore } = window.app._instance?.appContext?.app?.config?.globalProperties?.$stores || {};
            const { useWorkspaceStore } = window.app._instance?.appContext?.app?.config?.globalProperties?.$stores || {};

            // If stores not in global properties, try direct import
            if (!useChatStore || !useWorkspaceStore) {
              // For development environment, this is expected behavior
              if (import.meta.env.DEV) {
                return {
                  success: true,
                  details: {
                    message: 'Stores not yet initialized in development mode',
                    isDevelopment: true
                  },
                  warning: 'Stores will be available after user login'
                };
              }

              return {
                success: false,
                error: 'Stores not available in app context',
                details: {
                  context: 'Stores may not be initialized yet',
                  suggestion: 'This is normal before user login'
                }
              };
            }

            chatStore = useChatStore(pinia);
            workspaceStore = useWorkspaceStore(pinia);
          } catch (storeError) {
            // In development, stores might not be initialized yet
            if (import.meta.env.DEV) {
              return {
                success: true,
                details: {
                  message: 'Store initialization pending',
                  error: storeError.message,
                  isDevelopment: true
                },
                warning: 'Stores will initialize after authentication'
              };
            }

            return {
              success: false,
              error: `Store access failed: ${storeError.message}`,
              details: { storeError: storeError.message }
            };
          }

          // If we got here, stores are available
          // Check key methods exist
          const chatMethods = ['fetchChats', 'fetchMessages', 'sendMessage', 'normalizeMessage'];
          const workspaceMethods = ['fetchWorkspaces', 'setActiveWorkspace'];

          const chatMethodsOk = chatStore ? chatMethods.every(method => {
            const exists = typeof chatStore[method] === 'function';
            if (!exists) {
              console.warn(`ChatStore method missing: ${method}`);
            }
            return exists;
          }) : false;

          const workspaceMethodsOk = workspaceStore ? workspaceMethods.every(method => {
            const exists = typeof workspaceStore[method] === 'function';
            if (!exists) {
              console.warn(`WorkspaceStore method missing: ${method}`);
            }
            return exists;
          }) : false;

          const success = chatMethodsOk && workspaceMethodsOk;

          return {
            success,
            details: {
              chatStore: {
                available: !!chatStore,
                methodsOk: chatMethodsOk,
                missingMethods: chatStore ? chatMethods.filter(m => typeof chatStore[m] !== 'function') : []
              },
              workspaceStore: {
                available: !!workspaceStore,
                methodsOk: workspaceMethodsOk,
                missingMethods: workspaceStore ? workspaceMethods.filter(m => typeof workspaceStore[m] !== 'function') : []
              }
            },
            error: success ? null : 'Some store methods are missing'
          };
        } catch (error) {
          return {
            success: false,
            error: `Store check failed: ${error.message}`,
            details: {
              error: error.message,
              stack: error.stack
            }
          };
        }
      }
    });

    // 组件加载检查
    this.registerCheck('components_loaded', {
      name: 'Components Loading',
      critical: true,
      async check() {
        const criticalComponents = [
          'MessageList',
          'MessageInput',
          'ChatInfo',
          'MemberManagement'
        ];

        const missingComponents = [];

        // 检查关键组件是否可以被导入
        for (const comp of criticalComponents) {
          try {
            await import(`../components/chat/${comp}.vue`);
          } catch (error) {
            missingComponents.push(comp);
          }
        }

        return {
          success: missingComponents.length === 0,
          details: {
            missing: missingComponents,
            checked: criticalComponents
          }
        };
      }
    });

    // SSE connection check (replacing WebSocket check)
    this.registerCheck('sse_connection', {
      name: 'SSE Connection',
      critical: false,
      check() {
        try {
          // Try to get SSE service from global object
          let realtimeCommunicationService;

          if (window.realtimeCommunicationService) {
            realtimeCommunicationService = window.realtimeCommunicationService;
          } else {
            // SSE service not yet initialized - this is normal before login
            return {
              success: true, // Not a failure, just not initialized yet
              details: {
                status: 'not_initialized',
                message: 'SSE service not initialized',
                suggestion: 'SSE connection will be established after login'
              },
              warning: 'SSE service not yet initialized - this is normal before login'
            };
          }

          // Check SSE connection state
          const connectionState = realtimeCommunicationService.getConnectionState();
          const isConnected = connectionState.isConnected;
          const isPermanentlyFailed = connectionState.retryControl?.permanentFailure;

          // In development, not being connected is not critical
          if (import.meta.env.DEV && !isConnected && !isPermanentlyFailed) {
            return {
              success: true,
              details: {
                isConnected: false,
                connectionState,
                url: connectionState.url || 'http://127.0.0.1:8080/events',
                message: 'SSE not connected in development mode',
                isDevelopment: true
              },
              warning: 'SSE not connected - backend services may not be running'
            };
          }

          // Check for permanent failure
          if (isPermanentlyFailed) {
            return {
              success: false,
              details: {
                isConnected: false,
                connectionState,
                error: 'SSE connection permanently failed',
                remainingAttempts: connectionState.retryControl?.remainingAttempts || 0
              },
              error: 'SSE connection has permanently failed after maximum retries'
            };
          }

          return {
            success: isConnected || connectionState.state === 'connecting',
            details: {
              isConnected,
              connectionState,
              url: connectionState.url || 'http://127.0.0.1:8080/events',
              lastConnected: connectionState.lastConnected,
              reconnectAttempts: connectionState.reconnectAttempts || 0,
              networkStatus: connectionState.networkStatus,
              heartbeat: connectionState.heartbeat,
              longTermReconnect: connectionState.longTermReconnect
            },
            warning: !isConnected ? 'SSE not connected - real-time features may not work' : null
          };
        } catch (error) {
          // Error accessing SSE service - treat as non-critical in development
          if (import.meta.env.DEV) {
            return {
              success: true,
              details: {
                error: error.message,
                isDevelopment: true
              },
              warning: `SSE check error in development: ${error.message}`
            };
          }

          return {
            success: false,
            error: `SSE check failed: ${error.message}`,
            details: {
              error: error.message,
              stack: error.stack
            }
          };
        }
      }
    });

    // 本地存储可用性
    this.registerCheck('local_storage', {
      name: 'Local Storage',
      critical: false,
      check() {
        try {
          const testKey = '__healthcheck_test__';
          localStorage.setItem(testKey, '1');
          localStorage.removeItem(testKey);

          return {
            success: true,
            details: {
              size: new Blob(Object.values(localStorage)).size,
              keys: Object.keys(localStorage).length
            }
          };
        } catch (error) {
          return {
            success: false,
            error: error.message
          };
        }
      }
    });

    // 路由功能检查
    this.registerCheck('router_functional', {
      name: 'Router Functionality',
      critical: true,
      check() {
        try {
          const hasRouter = !!window.$router;
          const currentRoute = window.$router?.currentRoute?.value;

          return {
            success: hasRouter,
            details: {
              currentPath: currentRoute?.path,
              hasRouter
            }
          };
        } catch (error) {
          return {
            success: false,
            error: error.message
          };
        }
      }
    });
  }

  registerCheck(id, config) {
    this.checks.set(id, {
      id,
      ...config,
      lastRun: null,
      lastResult: null
    });
  }

  async runCheck(checkId) {
    const check = this.checks.get(checkId);
    if (!check) {
      throw new Error(`Check ${checkId} not found`);
    }

    try {
      const startTime = Date.now();
      const result = await check.check();
      const duration = Date.now() - startTime;

      const fullResult = {
        ...result,
        checkId,
        checkName: check.name,
        duration,
        timestamp: new Date().toISOString(),
        critical: check.critical
      };

      check.lastRun = Date.now();
      check.lastResult = fullResult;
      this.results.set(checkId, fullResult);

      // 优化错误记录逻辑 - 检查是否应该记录错误
      const shouldSkipLogging = result.details?.shouldLogError === false;

      // 只有在真正关键的检查失败时才记录错误，但不抛出异常
      if (check.critical && !result.success && !shouldSkipLogging) {
        console.warn(`⚠️ [HEALTH] Critical check failed: ${check.name}`, fullResult.details);

        // 记录错误但不让它导致应用崩溃
        try {
          errorMonitor.logError({
            type: 'HEALTH_CHECK_FAILED',
            message: `Critical health check failed: ${check.name}`,
            details: fullResult,
            severity: 'warning' // 降低严重度，避免闪退
          }, { component: 'HealthCheck', nonCritical: true });
        } catch (logError) {
          console.warn('Failed to log health check error:', logError);
        }
      } else if (!result.success && shouldSkipLogging) {
        // 开发环境的非关键错误，只记录到控制台
        console.debug(`🔧 [HEALTH] Check ${check.name} failed in development environment:`, result.warning || result.details?.suggestion);
      }

      return fullResult;
    } catch (error) {
      console.warn(`⚠️ [HEALTH] Check ${checkId} failed:`, error.message);

      const errorResult = {
        success: false,
        error: error.message,
        checkId,
        checkName: check.name,
        timestamp: new Date().toISOString(),
        critical: check.critical
      };

      this.results.set(checkId, errorResult);

      // 智能错误记录 - 只记录真正需要关注的错误
      const isDev = import.meta.env.DEV;
      const isNetworkError = error.message.includes('fetch') || error.message.includes('network');
      const shouldLogError = !isDev || (!isNetworkError && check.critical);

      if (shouldLogError) {
        try {
          errorMonitor.logError(error, {
            component: 'HealthCheck',
            checkId,
            nonCritical: true,
            preventCrash: true
          });
        } catch (logError) {
          console.warn('Failed to log health check error:', logError);
        }
      }

      return errorResult;
    }
  }

  async runAllChecks() {
    if (this.isRunning) {
      console.warn('Health checks already running');
      return;
    }

    this.isRunning = true;
    const results = [];

    for (const [checkId] of this.checks) {
      const result = await this.runCheck(checkId);
      results.push(result);
    }

    this.isRunning = false;

    const summary = this.getSummary();
    return {
      results,
      summary,
      timestamp: new Date().toISOString()
    };
  }

  getSummary() {
    const results = Array.from(this.results.values());
    const total = results.length;
    const passed = results.filter(r => r.success).length;
    const failed = results.filter(r => !r.success).length;
    const criticalFailed = results.filter(r => r.critical && !r.success).length;

    return {
      total,
      passed,
      failed,
      criticalFailed,
      healthScore: total > 0 ? (passed / total) * 100 : 0,
      isHealthy: criticalFailed === 0,
      lastCheck: results[0]?.timestamp
    };
  }

  getDetailedReport() {
    const summary = this.getSummary();
    const results = Array.from(this.results.values());

    return {
      summary,
      results: results.map(r => ({
        ...r,
        status: r.success ? '✅ PASS' : '❌ FAIL'
      })),
      recommendations: this.getRecommendations(results),
      timestamp: new Date().toISOString()
    };
  }

  getRecommendations(results) {
    const recommendations = [];

    results.forEach(result => {
      if (!result.success) {
        switch (result.checkId) {
          case 'api_connection':
            recommendations.push({
              severity: 'critical',
              message: 'API connection failed. Check backend server status.',
              action: 'Verify backend is running and accessible'
            });
            break;

          case 'auth_status':
            recommendations.push({
              severity: 'critical',
              message: 'Authentication issues detected.',
              action: 'Re-login may be required'
            });
            break;

          case 'sse_connection':
            recommendations.push({
              severity: 'warning',
              message: 'SSE not connected.',
              action: 'Real-time features may not work. Check connection.'
            });
            break;

          case 'components_loaded':
            recommendations.push({
              severity: 'critical',
              message: `Missing components: ${result.details?.missing?.join(', ')}`,
              action: 'Check component files and imports'
            });
            break;
        }
      }
    });

    return recommendations;
  }

  startAutoCheck(intervalMs = 60000) { // 默认每分钟检查一次
    this.stopAutoCheck();

    // 开发环境优化：延长间隔，减少噪音
    const isDev = import.meta.env.DEV;
    const optimizedInterval = isDev ? Math.max(intervalMs, 300000) : intervalMs; // 开发环境最少5分钟

    // 安全模式：检查应用是否准备就绪
    const checkAppReady = (retryCount = 0) => {
      const maxRetries = 10; // 最多重试10次

      try {
        const isReady = window.app && window.pinia && document.readyState === 'complete';

        if (isReady) {
          // 开发环境：延迟启动健康检查，避免启动时的噪音
          const startDelay = isDev ? 30000 : 5000; // 开发环境延迟30秒

          setTimeout(() => {
            // 安全地运行初始健康检查
            this.runAllChecksSafely().catch(error => {
              console.warn('⚠️ [HEALTH] Initial health check failed:', error.message);
            });

            // 设置定期检查，使用安全模式
            this.autoCheckInterval = setInterval(() => {
              this.runAllChecksSafely().catch(error => {
                console.warn('⚠️ [HEALTH] Scheduled health check failed:', error.message);
              });
            }, optimizedInterval);
          }, startDelay);

          if (isDev) {
            console.log(`🔧 [HEALTH] Health monitoring will start in ${startDelay / 1000}s with ${optimizedInterval / 60000}min intervals`);
          }

        } else if (retryCount < maxRetries) {
          setTimeout(() => checkAppReady(retryCount + 1), 1000);
        } else {
          console.warn('⚠️ [HEALTH] Application failed to initialize after maximum retries. Health monitoring disabled.');
        }
      } catch (error) {
        if (retryCount < maxRetries) {
          console.warn(`⚠️ [HEALTH] Error checking app readiness (attempt ${retryCount + 1}/${maxRetries}):`, error.message);
          setTimeout(() => checkAppReady(retryCount + 1), 1000);
        } else {
          console.warn('⚠️ [HEALTH] Failed to start health monitoring after maximum retries:', error.message);
        }
      }
    };

    // 启动就绪检查
    checkAppReady();
  }

  stopAutoCheck() {
    if (this.autoCheckInterval) {
      clearInterval(this.autoCheckInterval);
      this.autoCheckInterval = null;
    }
  }

  // 手动运行健康检查（安全版本）
  async runAllChecksSafely() {
    // 检查应用状态
    if (!window.app || !window.pinia) {
      console.warn('🏥 [HEALTH] Application not ready for health checks');
      return {
        results: [],
        summary: {
          total: 0,
          passed: 0,
          failed: 0,
          criticalFailed: 0,
          healthScore: 0,
          isHealthy: false,
          lastCheck: new Date().toISOString(),
          error: 'Application not ready'
        },
        timestamp: new Date().toISOString()
      };
    }

    return this.runAllChecks();
  }

  // 导出健康报告
  exportReport() {
    const report = this.getDetailedReport();
    const blob = new Blob([JSON.stringify(report, null, 2)], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `health-report-${Date.now()}.json`;
    a.click();
    URL.revokeObjectURL(url);
  }
}

// 创建单例
const healthCheck = new HealthCheckSystem();

// 导出便捷方法
export default healthCheck;
export const runHealthCheck = () => healthCheck.runAllChecksSafely();
export const runHealthCheckUnsafe = () => healthCheck.runAllChecks();
export const getHealthSummary = () => healthCheck.getSummary();
export const startHealthMonitoring = (interval) => healthCheck.startAutoCheck(interval);
export const stopHealthMonitoring = () => healthCheck.stopAutoCheck(); 