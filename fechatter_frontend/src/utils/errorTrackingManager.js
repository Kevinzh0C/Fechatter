/**
 * Error Tracking Manager
 * Real-time error tracking and consistency management for multi-session conversations
 * Corresponds to ERROR_TRACKING_TABLE.md
 */

class ErrorTrackingManager {
  constructor() {
    this.errorTable = this.initializeErrorTable();
    this.sessionStartTime = Date.now();
    this.updateLog = [];
  }

  /**
   * Initialize error table with current status
   */
  initializeErrorTable() {
    return {
      E001: {
        id: 'E001',
        type: '导航系统',
        description: '404错误循环',
        status: 'FIXED',
        lastFixTime: '2025-01-07 06:50',
        lastOccurrence: null,
        fixedBy: 'chat.js:checkReadEndpointExists()',
        symptoms: '已优雅降级'
      },
      E002: {
        id: 'E002',
        type: '导航系统',
        description: '重复导航调用',
        status: 'FIXED',
        lastFixTime: '2025-01-07 06:55',
        lastOccurrence: null,
        fixedBy: 'channelNavigationFixScript.js',
        symptoms: '防抖+锁机制'
      },
      E003: {
        id: 'E003',
        type: '导航系统',
        description: 'API竞态条件',
        status: 'FIXED',
        lastFixTime: '2025-01-07 07:05',
        lastOccurrence: null,
        fixedBy: 'chat.js:queueApiCall()',
        symptoms: '队列化处理'
      },
      E004: {
        id: 'E004',
        type: '导航系统',
        description: 'Vue响应式失效',
        status: 'FIXED',
        lastFixTime: '2025-01-07 07:15',
        lastOccurrence: null,
        fixedBy: 'emergencyVueReactivityFix.js',
        symptoms: '强制响应式触发'
      },
      E005: {
        id: 'E005',
        type: '认证系统',
        description: 'Token过期401错误',
        status: 'ACTIVE',
        lastFixTime: null,
        lastOccurrence: '2025-01-07 07:19',
        fixedBy: 'tokenManager.js',
        symptoms: 'refresh失败无限循环'
      },
      E006: {
        id: 'E006',
        type: '认证系统',
        description: 'require is not defined',
        status: 'FIXED',
        lastFixTime: '2025-01-07 07:22',
        lastOccurrence: null,
        fixedBy: 'auth.js:561',
        symptoms: '已修复，使用ES6 import'
      },
      E007: {
        id: 'E007',
        type: '路由系统',
        description: '重复导航警告',
        status: 'ACTIVE',
        lastFixTime: null,
        lastOccurrence: '2025-01-07 07:19',
        fixedBy: 'router/index.js',
        symptoms: '多次push到/login'
      },
      E008: {
        id: 'E008',
        type: '验证系统',
        description: '认证检测失败',
        status: 'REGRESSED',
        lastFixTime: '2025-01-07 06:45',
        lastOccurrence: '2025-01-07 07:16',
        fixedBy: 'authAwareNavigationVerifier.js',
        symptoms: '错误判断认证状态'
      },
      E009: {
        id: 'E009',
        type: '扩展错误',
        description: '内容脚本错误未抑制',
        status: 'PARTIAL',
        lastFixTime: '2025-01-07 06:30',
        lastOccurrence: '2025-01-07 07:16',
        fixedBy: 'contentScriptErrorSuppressor.js',
        symptoms: '模式透明但未抑制'
      }
    };
  }

  /**
   * Get current error statistics
   */
  getStatistics() {
    const errors = Object.values(this.errorTable);
    const total = errors.length;
    const fixed = errors.filter(e => e.status === 'FIXED').length;
    const active = errors.filter(e => e.status === 'ACTIVE').length;
    const regressed = errors.filter(e => e.status === 'REGRESSED').length;
    const partial = errors.filter(e => e.status === 'PARTIAL').length;

    return {
      total,
      fixed,
      active,
      regressed,
      partial,
      successRate: Math.round((fixed / total) * 100)
    };
  }

  /**
   * Update error status
   */
  updateError(errorId, updates) {
    if (!this.errorTable[errorId]) {
      if (import.meta.env.DEV) {
        console.warn(`Unknown error ID: ${errorId}`);
      return false;
    }

    const oldStatus = this.errorTable[errorId].status;
    Object.assign(this.errorTable[errorId], {
      ...updates,
      lastUpdate: new Date().toISOString()
    });

    // Log the update
    this.logUpdate(errorId, oldStatus, updates.status || oldStatus, updates);

    if (import.meta.env.DEV) {
      console.log(`[ErrorTracker] Updated ${errorId}: ${oldStatus} → ${updates.status || oldStatus}`);
    return true;
  }

  /**
   * Mark error as fixed
   */
  markFixed(errorId, fixDetails) {
    return this.updateError(errorId, {
      status: 'FIXED',
      lastFixTime: new Date().toLocaleString('zh-CN'),
      lastOccurrence: null,
      ...fixDetails
    });

  /**
   * Mark error as regressed
   */
  markRegressed(errorId, details) {
    return this.updateError(errorId, {
      status: 'REGRESSED',
      lastOccurrence: new Date().toLocaleString('zh-CN'),
      ...details
    });

  /**
   * Log error occurrence
   */
  logOccurrence(errorId, details) {
    return this.updateError(errorId, {
      status: 'ACTIVE',
      lastOccurrence: new Date().toLocaleString('zh-CN'),
      ...details
    });

  /**
   * Log update for history tracking
   */
  logUpdate(errorId, oldStatus, newStatus, details) {
    this.updateLog.push({
      timestamp: new Date().toISOString(),
      errorId,
      oldStatus,
      newStatus,
      details
    });

    // Keep only last 50 updates
    if (this.updateLog.length > 50) {
      this.updateLog = this.updateLog.slice(-50);
    }

  /**
   * Generate comprehensive status report
   */
  generateReport() {
    const stats = this.getStatistics();

    if (import.meta.env.DEV) {
      console.log('Error Tracking Report');
    if (import.meta.env.DEV) {
      console.log('========================');
    if (import.meta.env.DEV) {
      console.log(`📅 Session: ${new Date(this.sessionStartTime).toLocaleString('zh-CN')}`);
    if (import.meta.env.DEV) {
      console.log(`Total Errors: ${stats.total}`);
    if (import.meta.env.DEV) {
      console.log(`Fixed: ${stats.fixed}`);
    if (import.meta.env.DEV) {
      console.log(`🔴 Active: ${stats.active}`);
    if (import.meta.env.DEV) {
      console.log(`🔄 Regressed: ${stats.regressed}`);
    if (import.meta.env.DEV) {
      console.log(`🟡 Partial: ${stats.partial}`);
    if (import.meta.env.DEV) {
      console.log(`📈 Success Rate: ${stats.successRate}%`);
    if (import.meta.env.DEV) {
      console.log('');
    }

    // Show active/regressed errors
    const problematicErrors = Object.values(this.errorTable)
      .filter(e => ['ACTIVE', 'REGRESSED'].includes(e.status));

    if (problematicErrors.length > 0) {
      if (import.meta.env.DEV) {
        console.log('🚨 Current Issues:');
      problematicErrors.forEach(error => {
        const status = error.status === 'ACTIVE' ? '🔴' : '🔄';
        if (import.meta.env.DEV) {
          console.log(`${status} [${error.id}] ${error.description}`);
        if (import.meta.env.DEV) {
          console.log(`   Last seen: ${error.lastOccurrence}`);
        if (import.meta.env.DEV) {
          console.log(`   Symptoms: ${error.symptoms}`);
        }
      });
      if (import.meta.env.DEV) {
        console.log('');
      }

    // Show recent updates
    if (this.updateLog.length > 0) {
      if (import.meta.env.DEV) {
        console.log('Recent Updates:');
      this.updateLog.slice(-5).forEach(update => {
        if (import.meta.env.DEV) {
          console.log(`   ${update.timestamp.substring(11, 19)} [${update.errorId}] ${update.oldStatus} → ${update.newStatus}`);
        }
      });

    return {
      statistics: stats,
      activeErrors: problematicErrors,
      recentUpdates: this.updateLog.slice(-10)
    };
  }

  /**
   * Get active critical errors requiring immediate attention
   */
  getCriticalErrors() {
    return Object.values(this.errorTable)
      .filter(e => e.status === 'ACTIVE' && ['认证系统', '路由系统'].includes(e.type))
      .sort((a, b) => {
        const priority = { '认证系统': 1, '路由系统': 2, '导航系统': 3 };
        return priority[a.type] - priority[b.type];
      });

  /**
   * Check for error patterns and suggest fixes
   */
  analyzePatterns() {
    const analysis = {
      authIssues: 0,
      navigationIssues: 0,
      recommendations: []
    };

    Object.values(this.errorTable).forEach(error => {
      if (['ACTIVE', 'REGRESSED'].includes(error.status)) {
        if (error.type === '认证系统') analysis.authIssues++;
        if (error.type.includes('导航') || error.type === '路由系统') analysis.navigationIssues++;
      }
    });

    if (analysis.authIssues > 0) {
      analysis.recommendations.push('运行 window.fixCriticalErrors() 修复认证问题');
    }

    if (analysis.navigationIssues > 0) {
      analysis.recommendations.push('使用 window.navigationDeduplicator.safePush() 进行安全导航');
    }

    return analysis;
  }

  /**
   * Sync with current console errors in real-time using non-intrusive monitoring
   */
  monitorConsoleErrors() {
    // Use window.addEventListener instead of hijacking console.error
    window.addEventListener('error', (event) => {
      const errorMessage = event.message || event.error?.message || '';
      this.analyzeErrorPattern(errorMessage, 'window.error');
    });

    window.addEventListener('unhandledrejection', (event) => {
      const errorMessage = event.reason?.message || event.reason || '';
      this.analyzeErrorPattern(String(errorMessage), 'unhandled.rejection');
    });

    // Monitor specific error patterns through performance observer
    try {
      if ('PerformanceObserver' in window) {
        const observer = new PerformanceObserver((list) => {
          for (const entry of list.getEntries()) {
            if (entry.entryType === 'navigation' && entry.type === 'navigate') {
              // Monitor navigation errors
              this.analyzeErrorPattern(entry.name, 'navigation');
            }
        });
        observer.observe({ entryTypes: ['navigation'] });
    } catch (err) {
      if (import.meta.env.DEV) {
        console.warn('Performance observer not available:', err);
      }

    if (import.meta.env.DEV) {
      console.log('👁️ Console error monitoring activated (non-intrusive, preserves error sources)');
    }

  /**
   * Analyze error patterns without interfering with console output
   */
  analyzeErrorPattern(errorText, source) {
    if (errorText.includes('401') && errorText.includes('Unauthorized')) {
      this.logOccurrence('E005', { source: `${source} monitor` });

    if (errorText.includes('require is not defined')) {
      this.logOccurrence('E006', { source: `${source} monitor` });

    if (errorText.includes('redundant navigation')) {
      this.logOccurrence('E007', { source: `${source} monitor` });

    if (errorText.includes('Auth detection failed')) {
      this.logOccurrence('E008', { source: `${source} monitor` });

// Create and expose instance
const errorTracker = new ErrorTrackingManager();

if (typeof window !== 'undefined') {
  window.errorTracker = errorTracker;
  window.getErrorReport = () => errorTracker.generateReport();
  window.getCriticalErrors = () => errorTracker.getCriticalErrors();
  window.analyzeErrorPatterns = () => errorTracker.analyzePatterns();

  // Auto-start monitoring
  errorTracker.monitorConsoleErrors();

  if (import.meta.env.DEV) {
    console.log('Error Tracking Manager loaded');
  if (import.meta.env.DEV) {
    console.log('   - window.getErrorReport() - Get current status');
  if (import.meta.env.DEV) {
    console.log('   - window.getCriticalErrors() - Get critical issues');
  if (import.meta.env.DEV) {
    console.log('   - window.analyzeErrorPatterns() - Pattern analysis');
  }

export default errorTracker; 