// Login Flow Monitor - 深度监控登录流程
export class LoginFlowMonitor {
  constructor() {
    this.events = [];
    this.startTime = null;
    this.eventId = 0;
    this.interceptors = new Map();
    this.storeWatchers = new Map();

    // 监控配置
    this.config = {
      logToConsole: true,
      logToServer: false,
      captureStackTrace: true,
      monitorStore: true,
      monitorRouter: true,
      monitorNetwork: true,
      monitorDOM: true
    };

    // 事件类型样式
    this.eventStyles = {
      USER_ACTION: 'background: #4fc3f7; color: #000; padding: 2px 5px; border-radius: 3px;',
      API_REQUEST: 'background: #ff9800; color: #000; padding: 2px 5px; border-radius: 3px;',
      API_RESPONSE: 'background: #4caf50; color: #fff; padding: 2px 5px; border-radius: 3px;',
      STORE_UPDATE: 'background: #9c27b0; color: #fff; padding: 2px 5px; border-radius: 3px;',
      ROUTE_CHANGE: 'background: #2196f3; color: #fff; padding: 2px 5px; border-radius: 3px;',
      ERROR: 'background: #f44336; color: #fff; padding: 2px 5px; border-radius: 3px;',
      VALIDATION: 'background: #ffc107; color: #000; padding: 2px 5px; border-radius: 3px;'
    };
  }

  // 开始监控
  start() {
    this.startTime = Date.now();
    this.events = [];
    this.eventId = 0;

    console.log('%c🔍 Login Flow Monitor Started', 'font-weight: bold; font-size: 14px;');
    this.logEvent('MONITOR_START', { timestamp: new Date().toISOString() });

    // 设置拦截器
    if (this.config.monitorNetwork) {
      this.setupNetworkInterceptors();
    }

    // 监听DOM事件
    if (this.config.monitorDOM) {
      this.setupDOMListeners();
    }

    return this;
  }

  // 记录事件
  logEvent(type, data = {}, status = 'info') {
    const event = {
      id: ++this.eventId,
      type,
      data,
      status,
      timestamp: Date.now(),
      elapsed: this.startTime ? Date.now() - this.startTime : 0,
      stackTrace: this.config.captureStackTrace ? this.getStackTrace() : null
    };

    this.events.push(event);

    // 输出到控制台
    if (this.config.logToConsole) {
      const style = this.eventStyles[type] || 'background: #666; color: #fff; padding: 2px 5px;';
      console.log(
        `%c${type}%c +${event.elapsed}ms`,
        style,
        'color: #666; font-size: 11px;',
        data
      );

      if (status === 'error') {
        console.error('Error details:', data);
      }
    }

    // 发送到服务器
    if (this.config.logToServer) {
      this.sendToServer(event);
    }

    // 触发自定义事件
    window.dispatchEvent(new CustomEvent('login-monitor-event', { detail: event }));

    return event;
  }

  // 监控Pinia Store
  monitorStore(store, storeName) {
    if (!this.config.monitorStore) return;

    console.log(`%c🏪 Monitoring ${storeName} store`, 'color: #9c27b0;');

    // 监控actions
    const originalActions = {};
    Object.keys(store).forEach(key => {
      if (typeof store[key] === 'function' && !key.startsWith('$') && !key.startsWith('_')) {
        originalActions[key] = store[key];
        store[key] = (...args) => {
          this.logEvent('STORE_ACTION', {
            store: storeName,
            action: key,
            args: args.map(arg => this.sanitizeData(arg))
          });

          try {
            const result = originalActions[key].apply(store, args);

            // 如果是Promise，监控其结果
            if (result && typeof result.then === 'function') {
              return result
                .then(res => {
                  this.logEvent('STORE_ACTION_SUCCESS', {
                    store: storeName,
                    action: key,
                    result: this.sanitizeData(res)
                  });
                  return res;
                })
                .catch(err => {
                  this.logEvent('STORE_ACTION_ERROR', {
                    store: storeName,
                    action: key,
                    error: err.message
                  }, 'error');
                  throw err;
                });
            }

            return result;
          } catch (error) {
            this.logEvent('STORE_ACTION_ERROR', {
              store: storeName,
              action: key,
              error: error.message
            }, 'error');
            throw error;
          }
        };
      }
    });

    // 监控state变化
    store.$subscribe((mutation, state) => {
      this.logEvent('STORE_UPDATE', {
        store: storeName,
        type: mutation.type,
        events: mutation.events,
        stateAfter: this.sanitizeData(state)
      });
    });

    this.storeWatchers.set(storeName, store);
  }

  // 监控Vue Router
  monitorRouter(router) {
    if (!this.config.monitorRouter) return;

    console.log('%c🛣️ Monitoring Router', 'color: #2196f3;');

    // 监控路由守卫
    router.beforeEach((to, from, next) => {
      this.logEvent('ROUTE_BEFORE_EACH', {
        from: from.path,
        to: to.path,
        params: to.params,
        query: to.query
      });

      // 包装next函数以监控其调用
      const wrappedNext = (arg) => {
        if (arg === false) {
          this.logEvent('ROUTE_CANCELLED', { to: to.path });
        } else if (typeof arg === 'string' || (arg && typeof arg === 'object')) {
          this.logEvent('ROUTE_REDIRECT', { to: to.path, redirect: arg });
        }
        next(arg);
      };

      // 返回原始守卫的结果
      const guardResult = next;
      if (guardResult && typeof guardResult.then === 'function') {
        return guardResult.then(wrappedNext);
      }

      return wrappedNext;
    });

    router.afterEach((to, from) => {
      this.logEvent('ROUTE_CHANGE', {
        from: from.path,
        to: to.path,
        duration: Date.now() - this.startTime
      });
    });

    router.onError(error => {
      this.logEvent('ROUTE_ERROR', {
        error: error.message,
        stack: error.stack
      }, 'error');
    });
  }

  // 设置网络拦截器
  setupNetworkInterceptors() {
    // Axios拦截器
    if (window.axios) {
      // 请求拦截器
      window.axios.interceptors.request.use(
        config => {
          const requestId = Date.now();
          config._requestId = requestId;
          config._requestTime = Date.now();

          this.logEvent('API_REQUEST', {
            id: requestId,
            method: config.method.toUpperCase(),
            url: config.url,
            params: config.params,
            data: this.sanitizeData(config.data),
            headers: this.sanitizeHeaders(config.headers)
          });

          return config;
        },
        error => {
          this.logEvent('API_REQUEST_ERROR', {
            error: error.message
          }, 'error');
          return Promise.reject(error);
        }
      );

      // 响应拦截器
      window.axios.interceptors.response.use(
        response => {
          const duration = Date.now() - (response.config._requestTime || 0);

          this.logEvent('API_RESPONSE', {
            id: response.config._requestId,
            status: response.status,
            statusText: response.statusText,
            duration,
            data: this.sanitizeData(response.data),
            headers: response.headers
          });

          return response;
        },
        error => {
          const duration = Date.now() - (error.config?._requestTime || 0);

          this.logEvent('API_RESPONSE_ERROR', {
            id: error.config?._requestId,
            status: error.response?.status,
            statusText: error.response?.statusText,
            duration,
            error: error.message,
            data: error.response?.data
          }, 'error');

          return Promise.reject(error);
        }
      );
    }

    // Fetch拦截器
    const originalFetch = window.fetch;
    window.fetch = async (...args) => {
      const [url, config = {}] = args;
      const requestId = Date.now();
      const startTime = Date.now();

      this.logEvent('API_REQUEST', {
        id: requestId,
        method: (config.method || 'GET').toUpperCase(),
        url,
        headers: this.sanitizeHeaders(config.headers),
        body: config.body ? this.sanitizeData(JSON.parse(config.body)) : undefined
      });

      try {
        const response = await originalFetch(...args);
        const duration = Date.now() - startTime;

        // 克隆响应以读取body
        const clonedResponse = response.clone();
        let responseData;
        try {
          responseData = await clonedResponse.json();
        } catch {
          responseData = await clonedResponse.text();
        }

        this.logEvent('API_RESPONSE', {
          id: requestId,
          status: response.status,
          statusText: response.statusText,
          duration,
          data: this.sanitizeData(responseData),
          headers: Object.fromEntries(response.headers.entries())
        }, response.ok ? 'info' : 'error');

        return response;
      } catch (error) {
        const duration = Date.now() - startTime;

        this.logEvent('API_RESPONSE_ERROR', {
          id: requestId,
          duration,
          error: error.message
        }, 'error');

        throw error;
      }
    };
  }

  // 设置DOM监听器
  setupDOMListeners() {
    // 监听表单提交
    document.addEventListener('submit', (e) => {
      const form = e.target;
      const formData = new FormData(form);
      const data = {};
      formData.forEach((value, key) => {
        data[key] = key.toLowerCase().includes('password') ? '***' : value;
      });

      this.logEvent('FORM_SUBMIT', {
        formId: form.id,
        formName: form.name,
        action: form.action,
        method: form.method,
        data
      });
    }, true);

    // 监听按钮点击
    document.addEventListener('click', (e) => {
      const target = e.target;
      if (target.tagName === 'BUTTON' || target.type === 'submit') {
        this.logEvent('BUTTON_CLICK', {
          text: target.textContent.trim(),
          id: target.id,
          className: target.className,
          type: target.type
        });
      }
    }, true);

    // 监听输入焦点
    document.addEventListener('focus', (e) => {
      const target = e.target;
      if (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA') {
        this.logEvent('INPUT_FOCUS', {
          name: target.name,
          id: target.id,
          type: target.type,
          placeholder: target.placeholder
        });
      }
    }, true);

    // 监听输入变化
    let inputTimeout;
    document.addEventListener('input', (e) => {
      const target = e.target;
      if (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA') {
        clearTimeout(inputTimeout);
        inputTimeout = setTimeout(() => {
          this.logEvent('INPUT_CHANGE', {
            name: target.name,
            id: target.id,
            type: target.type,
            valueLength: target.value.length,
            value: target.type === 'password' ? '***' : target.value.substring(0, 20)
          });
        }, 300);
      }
    }, true);
  }

  // 清理敏感数据
  sanitizeData(data) {
    if (!data) return data;

    const sensitiveKeys = ['password', 'token', 'secret', 'key', 'auth'];

    if (typeof data === 'object') {
      const sanitized = Array.isArray(data) ? [] : {};

      for (const key in data) {
        if (data.hasOwnProperty(key)) {
          const lowerKey = key.toLowerCase();
          if (sensitiveKeys.some(sensitive => lowerKey.includes(sensitive))) {
            sanitized[key] = '***REDACTED***';
          } else if (typeof data[key] === 'object') {
            sanitized[key] = this.sanitizeData(data[key]);
          } else {
            sanitized[key] = data[key];
          }
        }
      }

      return sanitized;
    }

    return data;
  }

  // 清理请求头
  sanitizeHeaders(headers) {
    if (!headers) return headers;

    const sanitized = { ...headers };
    const sensitiveHeaders = ['authorization', 'cookie', 'x-auth-token'];

    Object.keys(sanitized).forEach(key => {
      if (sensitiveHeaders.includes(key.toLowerCase())) {
        sanitized[key] = '***REDACTED***';
      }
    });

    return sanitized;
  }

  // 获取调用栈
  getStackTrace() {
    const stack = new Error().stack;
    if (!stack) return null;

    const lines = stack.split('\n');
    // 过滤掉monitor相关的调用栈
    return lines
      .filter(line => !line.includes('LoginFlowMonitor') && !line.includes('logEvent'))
      .slice(0, 5)
      .join('\n');
  }

  // 发送到服务器
  async sendToServer(event) {
    try {
      // Get API config
      const { getApiConfig } = await import('@/utils/configLoader');
      const apiConfig = getApiConfig();
      const baseURL = import.meta.env.VITE_API_BASE_URL || apiConfig.base_url || '/api';
      await fetch(`${baseURL}/monitor/events`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(event)
      });
    } catch (error) {
      console.error('Failed to send event to server:', error);
    }
  }

  // 生成报告
  generateReport() {
    const report = {
      summary: {
        totalEvents: this.events.length,
        duration: Date.now() - this.startTime,
        startTime: new Date(this.startTime).toISOString(),
        endTime: new Date().toISOString()
      },
      eventCounts: {},
      errors: [],
      timeline: this.events,
      apiCalls: [],
      storeUpdates: []
    };

    // 统计事件类型
    this.events.forEach(event => {
      report.eventCounts[event.type] = (report.eventCounts[event.type] || 0) + 1;

      if (event.status === 'error') {
        report.errors.push(event);
      }

      if (event.type.startsWith('API_')) {
        report.apiCalls.push(event);
      }

      if (event.type.startsWith('STORE_')) {
        report.storeUpdates.push(event);
      }
    });

    return report;
  }

  // 下载报告
  downloadReport() {
    const report = this.generateReport();
    const blob = new Blob([JSON.stringify(report, null, 2)], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `login-flow-report-${Date.now()}.json`;
    a.click();
    URL.revokeObjectURL(url);
  }

  // 在控制台显示摘要
  showSummary() {
    const report = this.generateReport();

    console.group('%c📊 Login Flow Summary', 'font-size: 16px; font-weight: bold;');
    console.log(`Duration: ${report.summary.duration}ms`);
    console.log(`Total Events: ${report.summary.totalEvents}`);
    console.log(`Errors: ${report.errors.length}`);

    console.group('Event Counts:');
    Object.entries(report.eventCounts).forEach(([type, count]) => {
      console.log(`${type}: ${count}`);
    });
    console.groupEnd();

    if (report.errors.length > 0) {
      console.group('%c❌ Errors:', 'color: red;');
      report.errors.forEach(error => {
        console.error(error);
      });
      console.groupEnd();
    }

    console.groupEnd();
  }

  // 清除事件
  clear() {
    this.events = [];
    this.startTime = null;
    this.eventId = 0;
    console.log('%c🧹 Monitor cleared', 'color: #666;');
  }
}

// 创建全局实例
window.loginMonitor = new LoginFlowMonitor();

// 导出供Vue应用使用
export default LoginFlowMonitor;