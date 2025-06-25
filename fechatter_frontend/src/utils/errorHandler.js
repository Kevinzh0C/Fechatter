/**
 * 统一错误处理工具
 * 提供一致的错误处理和用户提示机制
 */

import { useNotifications } from '@/composables/useNotifications';

// 错误类型映射
const ErrorTypes = {
  NETWORK: 'network',
  AUTH: 'auth',
  VALIDATION: 'validation',
  PERMISSION: 'permission',
  NOT_FOUND: 'not_found',
  SERVER: 'server',
  SSE_CONNECTION: 'sse_connection',
  SSE_RETRY_LIMIT: 'sse_retry_limit',
  UNKNOWN: 'unknown'
};

// 错误消息模板
const ErrorMessages = {
  [ErrorTypes.NETWORK]: {
    title: '网络错误',
    message: '网络连接失败，请检查您的网络设置',
    action: '重试'
  },
  [ErrorTypes.AUTH]: {
    title: '认证失败',
    message: '您的登录状态已过期，请重新登录',
    action: '重新登录'
  },
  [ErrorTypes.VALIDATION]: {
    title: '输入错误',
    message: '请检查您的输入内容',
    action: '修改'
  },
  [ErrorTypes.PERMISSION]: {
    title: '权限不足',
    message: '您没有权限执行此操作',
    action: '返回'
  },
  [ErrorTypes.NOT_FOUND]: {
    title: '资源不存在',
    message: '请求的资源未找到',
    action: '返回'
  },
  [ErrorTypes.SERVER]: {
    title: '服务器错误',
    message: '服务器暂时无法处理您的请求，请稍后重试',
    action: '重试'
  },
  [ErrorTypes.SSE_CONNECTION]: {
    title: '实时连接异常',
    message: '实时消息连接出现问题，正在尝试重连',
    action: '等待'
  },
  [ErrorTypes.SSE_RETRY_LIMIT]: {
    title: '连接失败',
    message: '实时消息连接已达到最大重试次数，请刷新页面重试',
    action: '刷新页面'
  },
  [ErrorTypes.UNKNOWN]: {
    title: '未知错误',
    message: '发生了意外错误，请稍后重试',
    action: '重试'
  }
};

/**
 * 判断错误类型
 */
function getErrorType(error) {
  // SSE相关错误检测
  if (error.message) {
    const message = error.message.toLowerCase();
    if (message.includes('sse') || message.includes('server-sent events')) {
      if (message.includes('最大重试次数') || message.includes('permanently failed')) {
        return ErrorTypes.SSE_RETRY_LIMIT;
      }
      return ErrorTypes.SSE_CONNECTION;
    }
  }

  // 网络错误
  if (!error.response) {
    return ErrorTypes.NETWORK;
  }

  const status = error.response?.status;
  
  // 根据HTTP状态码判断
  switch (status) {
    case 401:
      return ErrorTypes.AUTH;
    case 403:
      return ErrorTypes.PERMISSION;
    case 404:
      return ErrorTypes.NOT_FOUND;
    case 422:
    case 400:
      return ErrorTypes.VALIDATION;
    case 500:
    case 502:
    case 503:
      return ErrorTypes.SERVER;
    default:
      return ErrorTypes.UNKNOWN;
  }
}

/**
 * 获取错误消息
 */
function getErrorMessage(error, type) {
  // 优先使用后端返回的错误消息
  const backendMessage = error.response?.data?.message || 
                        error.response?.data?.error?.message ||
                        error.response?.data?.error;

  if (backendMessage && typeof backendMessage === 'string') {
    return backendMessage;
  }

  // 使用默认错误消息
  return ErrorMessages[type]?.message || ErrorMessages[ErrorTypes.UNKNOWN].message;
}

/**
 * 全局错误处理器
 */
export class ErrorHandler {
  constructor() {
    this.notifications = null;
    this.errorCallbacks = new Map();
    this.globalErrorHandler = null;
  }

  /**
   * 初始化错误处理器
   */
  initialize() {
    // 延迟获取 notifications，避免循环依赖
    this.notifications = useNotifications();
    
    // 设置全局错误处理
    if (typeof window !== 'undefined') {
      window.addEventListener('unhandledrejection', this.handleUnhandledRejection.bind(this));
    }
  }

  /**
   * 处理SSE重试相关错误
   * @param {Error} error - 错误对象
   * @param {Object} retryInfo - 重试信息
   */
  handleSSERetryError(error, retryInfo = {}) {
    const {
      totalAttempts = 0,
      maxTotalAttempts = 0,
      consecutiveFailures = 0,
      maxConsecutiveFailures = 0,
      errorTypeHistory = [],
      context = 'SSE Connection'
    } = retryInfo;

    const errorType = getErrorType(error);
    const isRetryLimitReached = totalAttempts >= maxTotalAttempts || 
                               consecutiveFailures >= maxConsecutiveFailures;

    // 详细日志记录
    console.group(`🔌 [${context}] Retry Error Analysis`);
    console.error('Error Details:', {
      type: errorType,
      message: error.message,
      totalAttempts,
      maxTotalAttempts,
      consecutiveFailures,
      maxConsecutiveFailures,
      retryLimitReached: isRetryLimitReached,
      errorTypeHistory: errorTypeHistory.slice(-5) // 最近5次错误类型
    });
    
    if (errorTypeHistory.length > 0) {
      const errorTypeCounts = errorTypeHistory.reduce((acc, type) => {
        acc[type] = (acc[type] || 0) + 1;
        return acc;
      }, {});
      console.log('Error Type Distribution:', errorTypeCounts);
    }
    
    console.groupEnd();

    // 根据重试状态调整处理方式
    const options = {
      context,
      silent: totalAttempts > 2 && !isRetryLimitReached, // 前2次显示，达到限制时总是显示
      log: true
    };

    if (isRetryLimitReached) {
      // 创建特殊的重试限制错误
      const retryLimitError = new Error(`SSE连接已达到最大重试次数(${totalAttempts}次)，请刷新页面重试`);
      return this.handle(retryLimitError, {
        ...options,
        silent: false, // 重试限制错误总是显示
        onRetry: () => window.location.reload()
      });
    }

    return this.handle(error, options);
  }

  /**
   * 处理错误
   * @param {Error} error - 错误对象
   * @param {Object} options - 处理选项
   * @param {string} options.context - 错误上下文
   * @param {boolean} options.silent - 是否静默处理
   * @param {Function} options.onRetry - 重试回调
   * @param {boolean} options.log - 是否记录日志
   */
  handle(error, options = {}) {
    const {
      context = '',
      silent = false,
      onRetry = null,
      log = true
    } = options;

    // 获取错误类型
    const errorType = getErrorType(error);
    const errorInfo = ErrorMessages[errorType];
    const message = getErrorMessage(error, errorType);

    // 记录日志
    if (log && process.env.NODE_ENV !== 'production') {
      console.error(`[${context || 'Error'}]`, {
        type: errorType,
        message,
        error
      });
    }

    // 执行特定错误类型的回调
    const callback = this.errorCallbacks.get(errorType);
    if (callback) {
      callback(error, { type: errorType, message, context });
    }

    // 显示用户提示
    if (!silent && this.notifications) {
      const { notifyError } = this.notifications;
      
      notifyError(message, errorInfo.title, {
        duration: errorType === ErrorTypes.SSE_RETRY_LIMIT ? 10000 : 5000, // 重试限制错误显示更久
        actions: onRetry ? [{
          label: errorInfo.action,
          handler: onRetry
        }] : []
      });
    }

    // 特殊处理认证错误
    if (errorType === ErrorTypes.AUTH && !options.skipAuthRedirect) {
      this.handleAuthError();
    }

    return {
      type: errorType,
      message,
      error
    };
  }

  /**
   * 处理异步操作
   * @param {Promise} promise - 异步操作
   * @param {Object} options - 错误处理选项
   */
  async handleAsync(promise, options = {}) {
    try {
      return await promise;
    } catch (error) {
      this.handle(error, options);
      throw error;
    }
  }

  /**
   * 注册错误类型处理器
   */
  registerHandler(errorType, callback) {
    this.errorCallbacks.set(errorType, callback);
  }

  /**
   * 处理认证错误
   */
  handleAuthError() {
    // 避免重复跳转
    if (window.location.pathname === '/login') return;
    
    // 清理认证状态
    const keysToRemove = ['auth_token', 'refresh_token', 'user'];
    keysToRemove.forEach(key => {
      localStorage.removeItem(key);
      sessionStorage.removeItem(key);
    });

    // 延迟跳转，避免中断其他操作
    setTimeout(() => {
      window.location.href = '/login';
    }, 1000);
  }

  /**
   * 处理未捕获的Promise错误
   */
  handleUnhandledRejection(event) {
    console.error('Unhandled promise rejection:', event.reason);
    
    // 在开发环境显示错误
    if (process.env.NODE_ENV !== 'production') {
      this.handle(event.reason, {
        context: 'Unhandled Promise Rejection',
        silent: false
      });
    }
  }

  /**
   * 创建错误边界
   */
  createErrorBoundary(component) {
    return {
      name: `${component.name}ErrorBoundary`,
      components: { WrappedComponent: component },
      data() {
        return {
          hasError: false,
          error: null
        };
      },
      errorCaptured(error, instance, info) {
        this.hasError = true;
        this.error = error;
        
        errorHandler.handle(error, {
          context: `Component: ${component.name}`,
          silent: false
        });
        
        // 阻止错误继续传播
        return false;
      },
      render() {
        if (this.hasError) {
          return h('div', { class: 'error-boundary' }, [
            h('h3', 'Something went wrong'),
            h('p', this.error?.message || 'An unexpected error occurred'),
            h('button', {
              onClick: () => {
                this.hasError = false;
                this.error = null;
              }
            }, 'Retry')
          ]);
        }
        
        return h(WrappedComponent, this.$attrs);
      }
    };
  }

  /**
   * 错误重试装饰器
   */
  withRetry(fn, maxRetries = 3, delay = 1000) {
    return async (...args) => {
      let lastError;
      
      for (let i = 0; i < maxRetries; i++) {
        try {
          return await fn(...args);
        } catch (error) {
          lastError = error;
          
          // 不重试认证错误和权限错误
          const errorType = getErrorType(error);
          if ([ErrorTypes.AUTH, ErrorTypes.PERMISSION, ErrorTypes.VALIDATION].includes(errorType)) {
            throw error;
          }
          
          // 最后一次尝试不等待
          if (i < maxRetries - 1) {
            await new Promise(resolve => setTimeout(resolve, delay * (i + 1)));
          }
        }
      }
      
      throw lastError;
    };
  }
}

// 创建单例实例
export const errorHandler = new ErrorHandler();

// 导出错误类型常量
export { ErrorTypes };

// 便捷方法
export const handleError = (error, options) => errorHandler.handle(error, options);
export const handleAsync = (promise, options) => errorHandler.handleAsync(promise, options);
export const withRetry = (fn, maxRetries, delay) => errorHandler.withRetry(fn, maxRetries, delay);