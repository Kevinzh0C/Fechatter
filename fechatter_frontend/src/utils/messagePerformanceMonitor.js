/**
 * Message Performance Monitor
 * 监控消息发送的前端和后端性能
 */

class MessagePerformanceMonitor {
  constructor() {
    this.metrics = [];
    this.maxMetrics = 100; // 最多保存100条记录
  }

  /**
   * 记录消息发送性能
   */
  recordMessageSend(chatId, metrics) {
    const record = {
      timestamp: new Date().toISOString(),
      chatId,
      ...metrics,
      slowBackend: metrics.backendTime > 3000,
      status: metrics.error ? 'failed' : 'success'
    };

    this.metrics.push(record);

    // 保持记录数量在限制内
    if (this.metrics.length > this.maxMetrics) {
      this.metrics.shift();
    }

    return record;
  }

  /**
   * 获取性能统计
   */
  getStats() {
    if (this.metrics.length === 0) {
      return {
        count: 0,
        avgFrontendTime: 0,
        avgBackendTime: 0,
        avgTotalTime: 0,
        slowBackendCount: 0,
        failureCount: 0
      };
    }

    const successMetrics = this.metrics.filter(m => m.status === 'success');

    const stats = {
      count: this.metrics.length,
      successCount: successMetrics.length,
      failureCount: this.metrics.filter(m => m.status === 'failed').length,
      avgFrontendTime: this.calculateAverage(successMetrics, 'frontendTime'),
      avgBackendTime: this.calculateAverage(successMetrics, 'backendTime'),
      avgTotalTime: this.calculateAverage(successMetrics, 'totalTime'),
      minBackendTime: Math.min(...successMetrics.map(m => m.backendTime || Infinity)),
      maxBackendTime: Math.max(...successMetrics.map(m => m.backendTime || 0)),
      slowBackendCount: this.metrics.filter(m => m.slowBackend).length,
      slowBackendPercentage: (this.metrics.filter(m => m.slowBackend).length / this.metrics.length * 100).toFixed(1)
    };

    return stats;
  }

  /**
   * 计算平均值
   */
  calculateAverage(metrics, field) {
    if (metrics.length === 0) return 0;
    const sum = metrics.reduce((acc, m) => acc + (m[field] || 0), 0);
    return sum / metrics.length;
  }

  /**
   * 获取最近的性能记录
   */
  getRecentMetrics(count = 10) {
    return this.metrics.slice(-count);
  }

  /**
   * 获取慢请求
   */
  getSlowRequests(threshold = 3000) {
    return this.metrics.filter(m => m.backendTime > threshold);
  }

  /**
   * 生成性能报告
   */
  generateReport() {
    const stats = this.getStats();
    const recent = this.getRecentMetrics(5);
    const slow = this.getSlowRequests();

    console.log('📊 Message Send Performance Report');
    console.log('==================================');
    console.log(`Total Messages: ${stats.count}`);
    console.log(`Success Rate: ${((stats.successCount / stats.count) * 100).toFixed(1)}%`);
    console.log('');
    console.log('Average Times:');
    console.log(`  Frontend: ${stats.avgFrontendTime.toFixed(2)}ms`);
    console.log(`  Backend: ${stats.avgBackendTime.toFixed(2)}ms`);
    console.log(`  Total: ${stats.avgTotalTime.toFixed(2)}ms`);
    console.log('');
    console.log('Backend Performance:');
    console.log(`  Min: ${stats.minBackendTime.toFixed(2)}ms`);
    console.log(`  Max: ${stats.maxBackendTime.toFixed(2)}ms`);
    console.log(`  Slow Requests (>3s): ${stats.slowBackendCount} (${stats.slowBackendPercentage}%)`);

    if (recent.length > 0) {
      console.log('');
      console.log('Recent Messages:');
      recent.forEach((m, i) => {
        const time = new Date(m.timestamp).toLocaleTimeString();
        console.log(`  ${i + 1}. [${time}] Frontend: ${m.frontendTime?.toFixed(0)}ms, Backend: ${m.backendTime?.toFixed(0)}ms${m.slowBackend ? ' ⚠️ SLOW' : ''}`);
      });
    }

    if (slow.length > 0) {
      console.log('');
      console.log('⚠️ Slow Backend Requests:');
      slow.slice(-5).forEach((m, i) => {
        const time = new Date(m.timestamp).toLocaleTimeString();
        console.log(`  ${i + 1}. [${time}] ${m.backendTime?.toFixed(0)}ms - Chat ${m.chatId}`);
      });
    }

    return stats;
  }

  /**
   * 清空记录
   */
  clear() {
    this.metrics = [];
    console.log('✅ Performance metrics cleared');
  }
}

// 创建全局实例
const messagePerformanceMonitor = new MessagePerformanceMonitor();

// 导出到window以便调试
if (typeof window !== 'undefined') {
  window.msgPerf = {
    monitor: messagePerformanceMonitor,
    report: () => messagePerformanceMonitor.generateReport(),
    stats: () => messagePerformanceMonitor.getStats(),
    recent: (count) => messagePerformanceMonitor.getRecentMetrics(count),
    slow: () => messagePerformanceMonitor.getSlowRequests(),
    clear: () => messagePerformanceMonitor.clear()
  };

  console.log('📊 Message Performance Monitor loaded');
  console.log('   Commands:');
  console.log('   - window.msgPerf.report() - Generate performance report');
  console.log('   - window.msgPerf.stats() - Get statistics');
  console.log('   - window.msgPerf.recent(5) - Get recent 5 messages');
  console.log('   - window.msgPerf.slow() - Get slow requests');
  console.log('   - window.msgPerf.clear() - Clear metrics');
}

export default messagePerformanceMonitor; 