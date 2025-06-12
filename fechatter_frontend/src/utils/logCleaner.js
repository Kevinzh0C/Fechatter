/**
 * 生产环境日志清理工具
 * 用于包装和过滤开发环境的调试日志
 */

// 日志级别定义
const LogLevel = {
  ERROR: 0,
  WARN: 1, 
  INFO: 2,
  DEBUG: 3,
  TRACE: 4
};

// 生产环境允许的日志级别
const PRODUCTION_LOG_LEVEL = LogLevel.WARN;

class LogCleaner {
  constructor() {
    this.isDevelopment = import.meta.env.DEV;
    this.isProduction = import.meta.env.PROD;
    this.logLevel = this.isProduction ? PRODUCTION_LOG_LEVEL : LogLevel.TRACE;
    
    // 保存原始console方法
    this.originalConsole = {
      log: console.log,
      info: console.info,
      warn: console.warn,
      error: console.error,
      debug: console.debug,
      trace: console.trace
    };
  }

  /**
   * 检查是否应该输出此级别的日志
   */
  shouldLog(level) {
    return level <= this.logLevel;
  }

  /**
   * 包装的console.log - 开发环境输出，生产环境静默
   */
  log(...args) {
    if (this.shouldLog(LogLevel.INFO)) {
      this.originalConsole.log(...args);
    }
  }

  /**
   * 包装的console.info
   */
  info(...args) {
    if (this.shouldLog(LogLevel.INFO)) {
      this.originalConsole.info(...args);
    }
  }

  /**
   * 包装的console.warn - 生产环境也输出
   */
  warn(...args) {
    if (this.shouldLog(LogLevel.WARN)) {
      this.originalConsole.warn(...args);
    }
  }

  /**
   * 包装的console.error - 总是输出
   */
  error(...args) {
    if (this.shouldLog(LogLevel.ERROR)) {
      this.originalConsole.error(...args);
    }
  }

  /**
   * 包装的console.debug - 仅开发环境
   */
  debug(...args) {
    if (this.shouldLog(LogLevel.DEBUG)) {
      this.originalConsole.debug(...args);
    }
  }

  /**
   * 包装的console.trace - 仅开发环境
   */
  trace(...args) {
    if (this.shouldLog(LogLevel.TRACE)) {
      this.originalConsole.trace(...args);
    }
  }

  /**
   * 创建带上下文的日志器
   */
  createLogger(context) {
    return {
      log: (...args) => this.log(`[${context}]`, ...args),
      info: (...args) => this.info(`[${context}]`, ...args),
      warn: (...args) => this.warn(`[${context}]`, ...args),
      error: (...args) => this.error(`[${context}]`, ...args),
      debug: (...args) => this.debug(`[${context}]`, ...args),
      trace: (...args) => this.trace(`[${context}]`, ...args)
    };
  }

  /**
   * 开发专用日志 - 仅在开发环境输出
   */
  dev(...args) {
    if (this.isDevelopment) {
      this.originalConsole.log('🚧 [DEV]', ...args);
    }
  }

  /**
   * 性能日志 - 仅在开发环境或启用性能监控时输出
   */
  perf(label, ...args) {
    if (this.isDevelopment || window.enablePerformanceLogging) {
      this.originalConsole.log(`⚡ [PERF] ${label}`, ...args);
    }
  }

  /**
   * API请求日志 - 开发环境详细，生产环境仅错误
   */
  api(method, url, status, ...extra) {
    if (this.isDevelopment) {
      const emoji = status >= 400 ? '❌' : status >= 300 ? '⚠️' : '✅';
      this.originalConsole.log(`${emoji} [API] ${method} ${url} (${status})`, ...extra);
    } else if (status >= 400) {
      this.originalConsole.warn(`[API Error] ${method} ${url} (${status})`);
    }
  }

  /**
   * 清理特定模式的console调用
   * 这个方法会在构建时被调用，用于静态清理
   */
  static getCleanupPatterns() {
    return [
      // 清理调试用的console.log
      /console\.log\(['"`]🔍.*?['"`].*?\);?/g,
      /console\.log\(['"`]📊.*?['"`].*?\);?/g,
      /console\.log\(['"`]🚀.*?['"`].*?\);?/g,
      /console\.log\(['"`]✅.*?['"`].*?\);?/g,
      /console\.log\(['"`]📤.*?['"`].*?\);?/g,
      /console\.log\(['"`]📨.*?['"`].*?\);?/g,
      
      // 清理开发专用注释标记的console
      /\/\/ DEV:.*console\.\w+\(.*?\);?/g,
      /\/\* DEV \*\/.*console\.\w+\(.*?\);?/g,
      
      // 清理特定标识符的调试日志
      /console\.log\(.*\[CHAT_STORE\].*\);?/g,
      /console\.log\(.*\[AUTH\].*\);?/g,
      /console\.log\(.*\[DEBUG\].*\);?/g
    ];
  }

  /**
   * 生产环境性能优化：禁用特定的console方法
   */
  optimizeForProduction() {
    if (this.isProduction) {
      // 在生产环境中，重写console方法以提高性能
      console.log = () => {};
      console.debug = () => {};
      console.trace = () => {};
      
      // 保留warn和error用于重要信息
      console.info = this.shouldLog(LogLevel.INFO) ? this.originalConsole.info : () => {};
    }
  }

  /**
   * 恢复原始console方法
   */
  restore() {
    Object.assign(console, this.originalConsole);
  }
}

// 创建单例实例
const logCleaner = new LogCleaner();

// 在生产环境中自动优化
if (import.meta.env.PROD) {
  logCleaner.optimizeForProduction();
}

// 导出便捷方法
export const devLog = logCleaner.dev.bind(logCleaner);
export const perfLog = logCleaner.perf.bind(logCleaner);
export const apiLog = logCleaner.api.bind(logCleaner);
export const createLogger = logCleaner.createLogger.bind(logCleaner);

export default logCleaner;