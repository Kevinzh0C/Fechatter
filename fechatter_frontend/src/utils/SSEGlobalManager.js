/**
 * 🚨 SSE全局管理器 - 防止无限重试的终极解决方案
 * 这个管理器在浏览器标签页级别控制所有SSE连接
 */

class SSEGlobalManager {
  constructor() {
    this.connections = new Map(); // 存储所有SSE连接
    this.globalLimits = {
      maxConnectionsPerTab: 1,           // 每个标签页最大连接数改为1
      maxRetriesPerUrl: 10,              // 每个URL的最大重试次数
      maxRetriesPerSession: 15,          // 每个会话的最大重试次数
      cooldownPeriodMs: 300000,          // 5分钟冷却期
      sessionStartTime: Date.now()
    };

    this.sessionStats = {
      totalConnections: 0,
      totalRetries: 0,
      failedUrls: new Set(),
      bannedUrls: new Map(), // URL -> 禁用时间戳
      permanentlyBanned: new Set()
    };

    // 监听页面卸载，清理连接
    if (typeof window !== 'undefined') {
      window.addEventListener('beforeunload', () => {
        this.cleanup();
      });

      // 监听页面隐藏，暂停连接
      document.addEventListener('visibilitychange', () => {
        if (document.hidden) {
          this.pauseAllConnections();
        } else {
          this.resumeAllConnections();
        }
      });
    }

    // 全局错误计数器
    this.errorHistory = [];
    this.maxErrorsPerMinute = 10;

    console.log('🚨 SSE Global Manager initialized');
  }

  /**
   * 注册新的SSE连接
   */
  registerConnection(url, eventSource, service) {
    const connectionId = this.generateConnectionId(url);

    // 检查是否已有相同URL的连接
    for (const [existingId, existingConn] of this.connections) {
      if (existingConn.url === url && existingConn.status !== 'terminated') {
        console.log(`🚨 Found existing connection for ${url}, reusing it`);
        return existingId;
      }
    }

    // 如果达到连接数限制，先关闭最旧的连接
    if (this.connections.size >= this.globalLimits.maxConnectionsPerTab) {
      console.warn(`🚨 Connection limit reached, closing oldest connection`);
      const oldestConnection = Array.from(this.connections.values())
        .sort((a, b) => a.createdAt - b.createdAt)[0];
      if (oldestConnection) {
        this.terminateConnection(oldestConnection.id, '达到连接数限制，关闭旧连接');
      }
    }

    // 检查URL是否被永久禁用
    if (this.sessionStats.permanentlyBanned.has(url)) {
      throw new Error(`URL ${url} 已被永久禁用，请刷新页面重试`);
    }

    // 检查URL是否在冷却期
    if (this.sessionStats.bannedUrls.has(url)) {
      const bannedTime = this.sessionStats.bannedUrls.get(url);
      const cooldownRemaining = bannedTime + this.globalLimits.cooldownPeriodMs - Date.now();
      if (cooldownRemaining > 0) {
        throw new Error(`URL ${url} 在冷却期内，请等待 ${Math.ceil(cooldownRemaining / 1000)}秒`);
      } else {
        // 冷却期结束，移除禁用
        this.sessionStats.bannedUrls.delete(url);
      }
    }

    const connection = {
      id: connectionId,
      url,
      eventSource,
      service,
      createdAt: Date.now(),
      retryCount: 0,
      lastError: null,
      status: 'connecting'
    };

    this.connections.set(connectionId, connection);
    this.sessionStats.totalConnections++;

    console.log(`🚨 SSE Connection registered: ${connectionId} (Total: ${this.connections.size})`);
    return connectionId;
  }

  /**
   * 记录连接错误
   */
  recordConnectionError(connectionId, error) {
    const connection = this.connections.get(connectionId);
    if (!connection) return false;

    connection.retryCount++;
    connection.lastError = error;
    connection.status = 'error';
    this.sessionStats.totalRetries++;

    // 记录错误历史
    this.errorHistory.push({
      timestamp: Date.now(),
      url: connection.url,
      error: error.message
    });

    // 清理1分钟前的错误记录
    const oneMinuteAgo = Date.now() - 60000;
    this.errorHistory = this.errorHistory.filter(e => e.timestamp > oneMinuteAgo);

    console.error(`🚨 SSE Error recorded for ${connectionId}: ${error.message} (Retry: ${connection.retryCount})`);

    // 检查是否触发限制
    return this.checkAndEnforceLimits(connection);
  }

  /**
   * 检查并执行限制措施
   */
  checkAndEnforceLimits(connection) {
    const { url, retryCount } = connection;

    // 1. 检查该连接的重试次数
    if (retryCount >= this.globalLimits.maxRetriesPerUrl) {
      console.error(`🚨 Connection ${connection.id} exceeded max retries (${retryCount}), terminating`);
      this.terminateConnection(connection.id, '超过连接最大重试次数');

      // 将URL加入临时禁用列表
      this.sessionStats.bannedUrls.set(url, Date.now());
      return { terminate: true, reason: 'max_retries_per_url' };
    }

    // 2. 检查全局会话重试次数
    if (this.sessionStats.totalRetries >= this.globalLimits.maxRetriesPerSession) {
      console.error(`🚨 Session exceeded max retries (${this.sessionStats.totalRetries}), banning all SSE`);
      this.banAllSSEConnections('超过会话最大重试次数');
      return { terminate: true, reason: 'max_retries_per_session' };
    }

    // 3. 检查错误频率
    if (this.errorHistory.length >= this.maxErrorsPerMinute) {
      console.error(`🚨 Too many errors in 1 minute (${this.errorHistory.length}), temporary ban`);
      this.sessionStats.bannedUrls.set(url, Date.now());
      this.terminateConnection(connection.id, '错误频率过高');
      return { terminate: true, reason: 'error_frequency' };
    }

    return { terminate: false };
  }

  /**
   * 强制终止连接
   */
  terminateConnection(connectionId, reason = '未知原因') {
    const connection = this.connections.get(connectionId);
    if (!connection) return;

    console.error(`🚨 Forcefully terminating SSE connection ${connectionId}: ${reason}`);

    // 关闭EventSource
    if (connection.eventSource) {
      try {
        connection.eventSource.close();
      } catch (error) {
        console.warn('Error closing EventSource:', error);
      }
    }

    // 通知服务停止重连
    if (connection.service && typeof connection.service.forceStop === 'function') {
      connection.service.forceStop(reason);
    }

    connection.status = 'terminated';
    this.connections.delete(connectionId);

    // 触发全局事件
    this.dispatchEvent('connection_terminated', {
      connectionId,
      url: connection.url,
      reason,
      retryCount: connection.retryCount
    });
  }

  /**
   * 禁用所有SSE连接
   */
  banAllSSEConnections(reason = '达到全局限制') {
    console.error(`🚨 Banning all SSE connections: ${reason}`);

    // 终止所有现有连接
    for (const [connectionId, connection] of this.connections) {
      this.terminateConnection(connectionId, reason);
    }

    // 标记所有URL为永久禁用
    for (const connection of this.connections.values()) {
      this.sessionStats.permanentlyBanned.add(connection.url);
    }

    // 清空连接列表
    this.connections.clear();

    // 触发全局禁用事件
    this.dispatchEvent('all_connections_banned', { reason });

    // 显示用户友好的错误消息
    this.showGlobalErrorMessage(reason);
  }

  /**
   * 显示全局错误消息
   */
  showGlobalErrorMessage(reason) {
    // 可以集成到应用的通知系统
    if (typeof window !== 'undefined' && window.errorHandler) {
      window.errorHandler.handle(new Error(`SSE连接已被全局禁用: ${reason}`), {
        type: 'sse_global_ban',
        context: 'SSE Global Manager',
        persistent: true
      });
    }

    console.error(`
🚨 =================================================
   SSE 连接已被全局管理器禁用
   原因: ${reason}
   
   解决方案:
   1. 刷新页面重试
   2. 检查网络连接
   3. 联系技术支持
=================================================`);
  }

  /**
   * 暂停所有连接
   */
  pauseAllConnections() {
    for (const connection of this.connections.values()) {
      if (connection.service && typeof connection.service.pause === 'function') {
        connection.service.pause();
      }
    }
  }

  /**
   * 恢复所有连接
   */
  resumeAllConnections() {
    for (const connection of this.connections.values()) {
      if (connection.service && typeof connection.service.resume === 'function') {
        connection.service.resume();
      }
    }
  }

  /**
   * 清理所有连接
   */
  cleanup() {
    console.log('🚨 SSE Global Manager cleanup');

    for (const [connectionId, connection] of this.connections) {
      this.terminateConnection(connectionId, '页面卸载');
    }

    this.connections.clear();
  }

  /**
   * 生成连接ID
   */
  generateConnectionId(url) {
    return `sse_${Date.now()}_${Math.random().toString(36).substr(2, 9)}_${url.replace(/[^a-zA-Z0-9]/g, '_')}`;
  }

  /**
   * 触发自定义事件
   */
  dispatchEvent(eventName, data) {
    if (typeof window !== 'undefined') {
      const event = new CustomEvent(`sse_global_${eventName}`, { detail: data });
      window.dispatchEvent(event);
    }
  }

  /**
   * 获取管理器状态
   */
  getStatus() {
    return {
      connections: Array.from(this.connections.values()).map(conn => ({
        id: conn.id,
        url: conn.url,
        status: conn.status,
        retryCount: conn.retryCount,
        createdAt: conn.createdAt,
        lastError: conn.lastError?.message
      })),
      limits: this.globalLimits,
      sessionStats: {
        ...this.sessionStats,
        failedUrls: Array.from(this.sessionStats.failedUrls),
        bannedUrls: Object.fromEntries(this.sessionStats.bannedUrls),
        permanentlyBanned: Array.from(this.sessionStats.permanentlyBanned)
      },
      errorHistory: this.errorHistory.slice(-10) // 最近10个错误
    };
  }

  /**
   * 重置管理器 (慎用)
   */
  reset() {
    console.warn('🚨 Resetting SSE Global Manager');
    this.cleanup();
    this.sessionStats = {
      totalConnections: 0,
      totalRetries: 0,
      failedUrls: new Set(),
      bannedUrls: new Map(),
      permanentlyBanned: new Set()
    };
    this.errorHistory = [];
  }
}

// 全局单例
const sseGlobalManager = new SSEGlobalManager();

// 暴露到window对象用于调试
if (typeof window !== 'undefined') {
  window.sseGlobalManager = sseGlobalManager;
}

export default sseGlobalManager;