// Error Monitor - 错误监控和日志系统
class ErrorMonitor {
  constructor() {
    this.errors = [];
    this.warningsCount = 0;
    this.criticalErrors = [];
    this.maxErrors = 100;
    this.listeners = new Set();
    this.environment = import.meta.env.MODE || 'development';

    // 自动初始化
    if (typeof window !== 'undefined') {
      this.setupGlobalErrorHandlers();
    }
  }

  setupGlobalErrorHandlers() {
    // 捕获未处理的错误
    window.addEventListener('error', (event) => {
      this.logError({
        type: 'RUNTIME_ERROR',
        message: event.message,
        filename: event.filename,
        lineno: event.lineno,
        colno: event.colno,
        error: event.error,
        stack: event.error?.stack
      });
    });

    // 捕获未处理的Promise拒绝
    window.addEventListener('unhandledrejection', (event) => {
      this.logError({
        type: 'UNHANDLED_REJECTION',
        message: `Unhandled Promise Rejection: ${event.reason}`,
        reason: event.reason,
        promise: event.promise
      });
    });

    // Vue错误处理器会在main.js中设置
  }

  logError(error, context = {}) {
    const errorEntry = {
      id: Date.now() + '_' + Math.random(),
      timestamp: new Date().toISOString(),
      error: this.normalizeError(error),
      context,
      environment: this.environment,
      userAgent: navigator?.userAgent,
      url: window?.location?.href
    };

    // 智能过滤 - 避免在开发环境中记录过多噪音
    const shouldSkipError = this.shouldSkipError(error, context);
    if (shouldSkipError) {
      console.debug('[ERROR_MONITOR] Skipping development noise:', errorEntry.error.message);
      return null;
    }

    // 添加到错误列表
    this.errors.unshift(errorEntry);
    if (this.errors.length > this.maxErrors) {
      this.errors = this.errors.slice(0, this.maxErrors);
    }

    // 检查是否是关键错误
    if (this.isCriticalError(error)) {
      this.criticalErrors.push(errorEntry);
      if (import.meta.env.DEV) {
        console.error('🚨 CRITICAL ERROR:', errorEntry);
      }
    } else {
      if (import.meta.env.DEV) {
        console.error('ERROR: ERROR:', errorEntry);
      }
    }

    // 通知监听器
    this.notifyListeners(errorEntry);

    // 在开发环境中显示详细信息
    if (this.environment === 'development') {
      this.showDevelopmentError(errorEntry);
    }

    // 持久化错误日志
    this.persistError(errorEntry);

    return errorEntry;
  }

  logWarning(message, context = {}) {
    this.warningsCount++;
    if (import.meta.env.DEV) {
      console.warn('WARNING: WARNING:', message, context);
    }

    const warningEntry = {
      id: Date.now() + '_' + Math.random(),
      timestamp: new Date().toISOString(),
      type: 'WARNING',
      message,
      context,
      environment: this.environment
    };

    this.notifyListeners(warningEntry);
    return warningEntry;
  }

  normalizeError(error) {
    if (error instanceof Error) {
      return {
        name: error.name,
        message: error.message,
        stack: error.stack,
        type: error.constructor.name
      };
    } else if (typeof error === 'object') {
      return error;
    } else {
      return {
        message: String(error),
        type: 'Unknown'
      };
    }
  }

  isCriticalError(error) {
    const criticalPatterns = [
      /cannot read prop/i,
      /undefined is not/i,
      /network error/i,
      /failed to fetch/i,
      /401|403|500|502|503/,
      /messageList/i,
      /chat.*not.*found/i
    ];

    const errorStr = JSON.stringify(error).toLowerCase();
    return criticalPatterns.some(pattern => pattern.test(errorStr));
  }

  showDevelopmentError(errorEntry) {
    // 在开发环境中创建一个可视化的错误提示
    if (typeof document !== 'undefined' && this.environment === 'development') {
      const existingToast = document.getElementById('error-monitor-toast');
      if (existingToast) {
        existingToast.remove();
      }

      const toast = document.createElement('div');
      toast.id = 'error-monitor-toast';
      toast.style.cssText = `
        position: fixed;
        top: 20px;
        right: 20px;
        background: #ff4444;
        color: white;
        padding: 16px;
        border-radius: 8px;
        max-width: 400px;
        z-index: 9999;
        font-family: monospace;
        font-size: 12px;
        box-shadow: 0 4px 12px rgba(0,0,0,0.3);
      `;

      toast.innerHTML = `
        <div style="font-weight: bold; margin-bottom: 8px;">
          ERROR: ${errorEntry.error.type || 'ERROR'}
        </div>
        <div style="margin-bottom: 8px;">${errorEntry.error.message}</div>
        <div style="opacity: 0.8; font-size: 10px;">
          ${errorEntry.context.component || errorEntry.url || ''}
        </div>
        <button onclick="this.parentElement.remove()" style="
          position: absolute;
          top: 8px;
          right: 8px;
          background: none;
          border: none;
          color: white;
          cursor: pointer;
          font-size: 16px;
        ">×</button>
      `;

      document.body.appendChild(toast);

      // 自动移除
      setTimeout(() => {
        if (document.getElementById('error-monitor-toast') === toast) {
          toast.remove();
        }
      }, 10000);
    }
  }

  persistError(errorEntry) {
    try {
      const storageKey = 'errorMonitor_logs';
      const existingLogs = JSON.parse(localStorage.getItem(storageKey) || '[]');
      existingLogs.unshift(errorEntry);

      // 只保留最近的50条错误
      const recentLogs = existingLogs.slice(0, 50);
      localStorage.setItem(storageKey, JSON.stringify(recentLogs));
    } catch (e) {
      if (import.meta.env.DEV) {
        console.warn('Failed to persist error log:', e);
      }
    }
  }

  subscribe(listener) {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  }

  notifyListeners(entry) {
    this.listeners.forEach(listener => {
      try {
        listener(entry);
      } catch (e) {
        if (import.meta.env.DEV) {
          console.error('Error in error monitor listener:', e);
        }
      }
    });
  }

  getErrors() {
    return [...this.errors];
  }

  getCriticalErrors() {
    return [...this.criticalErrors];
  }

  getErrorStats() {
    const stats = {
      total: this.errors.length,
      critical: this.criticalErrors.length,
      warnings: this.warningsCount,
      byType: {},
      byComponent: {},
      recent: this.errors.slice(0, 10)
    };

    this.errors.forEach(entry => {
      const type = entry.error.type || 'Unknown';
      stats.byType[type] = (stats.byType[type] || 0) + 1;

      const component = entry.context.component || 'Unknown';
      stats.byComponent[component] = (stats.byComponent[component] || 0) + 1;
    });

    return stats;
  }

  clearErrors() {
    this.errors = [];
    this.criticalErrors = [];
    this.warningsCount = 0;
    localStorage.removeItem('errorMonitor_logs');
  }

  // 导出错误报告
  exportErrorReport() {
    const report = {
      generated: new Date().toISOString(),
      environment: this.environment,
      stats: this.getErrorStats(),
      errors: this.errors,
      criticalErrors: this.criticalErrors
    };

    const blob = new Blob([JSON.stringify(report, null, 2)], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `error-report-${Date.now()}.json`;
    a.click();
    URL.revokeObjectURL(url);
  }

  // 新增：智能过滤方法
  shouldSkipError(error, context = {}) {
    // 只在开发环境启用过滤
    if (this.environment !== 'development') {
      return false;
    }

    // 跳过非关键的健康检查错误
    if (context.component === 'HealthCheck' && context.nonCritical) {
      const errorStr = JSON.stringify(error).toLowerCase();
      const developmentNoisePatterns = [
        /fetch.*failed/i,
        /network.*error/i,
        /timeout/i,
        /cors/i,
        /gateway.*not.*running/i,
        /connection.*refused/i
      ];

      return developmentNoisePatterns.some(pattern => pattern.test(errorStr));
    }

    // 跳过常见的开发环境噪音
    const developmentSkipPatterns = [
      /hmr/i, // Vite HMR
      /hot.*reload/i,
      /websocket.*connection/i,
      /dev.*server/i
    ];

    const errorStr = JSON.stringify(error).toLowerCase();
    return developmentSkipPatterns.some(pattern => pattern.test(errorStr));
  }
}

// 创建单例实例
const errorMonitor = new ErrorMonitor();

export default errorMonitor;

// 便捷方法
export const logError = (error, context) => errorMonitor.logError(error, context);
export const logWarning = (message, context) => errorMonitor.logWarning(message, context);
export const getErrorStats = () => errorMonitor.getErrorStats();
export const clearErrors = () => errorMonitor.clearErrors(); 