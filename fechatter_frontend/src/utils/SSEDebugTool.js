/**
 * 🔧 SSE调试工具 - 帮助监控和诊断SSE连接问题
 */

class SSEDebugTool {
  constructor() {
    this.isEnabled = import.meta.env.DEV;
    this.logs = [];
    this.maxLogs = 100;
    
    if (typeof window !== 'undefined') {
      window.sseDebugTool = this;
    }

  /**
   * 启用调试模式
   */
  enable() {
    this.isEnabled = true;
    if (import.meta.env.DEV) {
      console.log('🔧 SSE Debug Tool enabled');
    this.showCurrentStatus();
  }

  /**
   * 禁用调试模式
   */
  disable() {
    this.isEnabled = false;
    if (import.meta.env.DEV) {
      console.log('🔧 SSE Debug Tool disabled');
    }

  /**
   * 记录调试信息
   */
  log(level, message, data = {}) {
    if (!this.isEnabled) return;

    const logEntry = {
      timestamp: new Date().toISOString(),
      level,
      message,
      data: JSON.parse(JSON.stringify(data))
    };

    this.logs.push(logEntry);
    
    // 保持日志数量限制
    if (this.logs.length > this.maxLogs) {
      this.logs.shift();
    }

    // 输出到控制台
    const consoleMethod = level === 'error' ? 'error' : level === 'warn' ? 'warn' : 'log';
    console[consoleMethod](`🔧 SSE Debug [${level.toUpperCase()}]: ${message}`, data);
  }

  /**
   * 显示当前状态
   */
  showCurrentStatus() {
    console.group('🔧 SSE系统当前状态');
    
    // 全局管理器状态
    if (typeof window !== 'undefined' && window.sseGlobalManager) {
      const status = window.sseGlobalManager.getStatus();
      if (import.meta.env.DEV) {
        console.log('📊 全局管理器状态:', status);
      }

    // 连接服务状态
    if (typeof window !== 'undefined' && window.realtimeCommunicationService) {
      const connectionState = window.realtimeCommunicationService.getConnectionState();
      if (import.meta.env.DEV) {
        console.log('🔌 连接服务状态:', connectionState);
      }

    console.groupEnd();
  }

  /**
   * 重置全局管理器
   */
  resetGlobalManager() {
    if (typeof window !== 'undefined' && window.sseGlobalManager) {
      if (import.meta.env.DEV) {
        console.warn('🚨 重置SSE全局管理器');
      window.sseGlobalManager.reset();
      this.log('warn', 'Global manager reset by debug tool');
    }

  /**
   * 强制终止所有连接
   */
  forceTerminateAll() {
    if (typeof window !== 'undefined' && window.sseGlobalManager) {
      if (import.meta.env.DEV) {
        console.warn('🚨 强制终止所有SSE连接');
      window.sseGlobalManager.banAllSSEConnections('调试工具强制终止');
      this.log('warn', 'All connections terminated by debug tool');
    }

  /**
   * 模拟网络错误
   */
  simulateNetworkError() {
    if (typeof window !== 'undefined' && window.realtimeCommunicationService) {
      if (import.meta.env.DEV) {
        console.warn('🚨 模拟网络错误');
      const mockError = new Error('Simulated network error');
      mockError.type = 'error';
      window.realtimeCommunicationService.handleError(mockError);
      this.log('warn', 'Simulated network error');
    }

  /**
   * 获取详细的连接报告
   */
  getConnectionReport() {
    const report = {
      timestamp: new Date().toISOString(),
      globalManager: null,
      connectionService: null,
      recentLogs: this.logs.slice(-20),
      browserInfo: {
        userAgent: navigator.userAgent,
        online: navigator.onLine,
        language: navigator.language,
        platform: navigator.platform
      }
    };

    if (typeof window !== 'undefined') {
      if (window.sseGlobalManager) {
        report.globalManager = window.sseGlobalManager.getStatus();
      }
      
      if (window.realtimeCommunicationService) {
        report.connectionService = window.realtimeCommunicationService.getConnectionState();
      }

    if (import.meta.env.DEV) {
      console.log('📋 SSE连接详细报告:', report);
    return report;
  }

  /**
   * 导出日志
   */
  exportLogs() {
    const logs = {
      exportedAt: new Date().toISOString(),
      logs: this.logs,
      status: this.getConnectionReport()
    };

    const blob = new Blob([JSON.stringify(logs, null, 2)], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    
    const a = document.createElement('a');
    a.href = url;
    a.download = `sse-debug-logs-${Date.now()}.json`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);

    if (import.meta.env.DEV) {
      console.log('📁 SSE调试日志已导出');
    }

  /**
   * 清除日志
   */
  clearLogs() {
    this.logs = [];
    if (import.meta.env.DEV) {
      console.log('🗑️ SSE调试日志已清除');
    }

  /**
   * 显示帮助信息
   */
  showHelp() {
    if (import.meta.env.DEV) {
      console.log(`
    }
🔧 SSE调试工具使用指南

可用命令:
  sseDebugTool.enable()              - 启用调试模式
  sseDebugTool.disable()             - 禁用调试模式
  sseDebugTool.showCurrentStatus()   - 显示当前状态
  sseDebugTool.getConnectionReport() - 获取详细报告
  sseDebugTool.exportLogs()          - 导出调试日志
  sseDebugTool.clearLogs()           - 清除日志
  sseDebugTool.resetGlobalManager()  - 重置全局管理器
  sseDebugTool.forceTerminateAll()   - 强制终止所有连接
  sseDebugTool.simulateNetworkError() - 模拟网络错误
  sseDebugTool.showHelp()            - 显示此帮助

全局对象:
  window.sseGlobalManager            - SSE全局管理器
  window.realtimeCommunicationService - SSE连接服务
    `);
  }

// 创建全局实例
const sseDebugTool = new SSEDebugTool();

// 在开发环境下自动启用
if (import.meta.env.DEV) {
  setTimeout(() => {
    sseDebugTool.enable();
    sseDebugTool.showHelp();
  }, 1000);
}

export default sseDebugTool;